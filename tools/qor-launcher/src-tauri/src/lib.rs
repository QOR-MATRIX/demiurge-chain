//! QOR Launcher: the sovereign gateway to the Demiurge ecosystem.
//!
//! # Why the host owns everything that matters
//!
//! The webview renders. It does not custody keys, it does not hold tokens, and
//! it cannot read a recovery phrase. Every security-relevant operation is a
//! command into this Rust host, which owns the vault, the keychain and the
//! network clients. A compromise of the frontend then costs an attacker the
//! ability to *ask* for signatures, not the ability to *take* keys, and asking
//! is something the user can be shown and can refuse.
//!
//! This is the structural advantage the launcher has over the browser extension
//! in `apps/wallet-extension`, where key material and page-reachable script share
//! one JavaScript heap.

pub mod arqade;
// Moved into the core shared with the Android launcher (ADR-104, DIRECTION L13.2), and kept at their old paths.
pub use qor_launcher_core::cgt;
pub mod chain;
pub mod config;
pub mod content;
pub use qor_launcher_core::error;
pub mod gates;
pub mod host;
pub mod identity;
pub mod library;
pub mod listings;
pub mod media;
pub mod mesh;
pub mod partners;
pub mod pay;
pub use qor_launcher_core::profiles;
pub mod qontrol;
pub mod qq;
pub mod social;
pub mod translate;
pub use qor_launcher_core::vault;

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::Manager;

use chain::{ChainClient, ChainStatus, ClaimResult, HistoryEntry, TransferReceipt};
use error::{QorError, QorResult};
use identity::{IdentityClient, Session};
use vault::{AccountView, Vault, VaultStatus};

// ─────────────────────────── host-side confirmation ─────────────────────────

// What the person is asked to approve, and the trait that asks them, are the core's (ADR-104), with the vault that
// asks through them; kept at their old paths. The launcher's own [`Confirm`] is the native dialog below.
pub use qor_launcher_core::confirm::{Answer, Confirm, Prompt};

/// The launcher's [`Confirm`]: a native dialog drawn by the host.
pub(crate) struct HostDialog(pub(crate) tauri::AppHandle);

impl Confirm for HostDialog {
    fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a> {
        use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

        let app = self.0.clone();
        let prompt = prompt.clone();

        Box::pin(async move {
            // `blocking_show` waits for the answer, so it runs off the async
            // runtime. The plugin itself draws the dialog on the main thread.
            tauri::async_runtime::spawn_blocking(move || {
                let mut dialog = app
                    .dialog()
                    .message(prompt.body)
                    .title(prompt.title)
                    .kind(MessageDialogKind::Warning)
                    .buttons(MessageDialogButtons::OkCancelCustom(
                        prompt.approve,
                        "Cancel".into(),
                    ));
                if let Some(window) = app.get_webview_window("main") {
                    dialog = dialog.parent(&window);
                }
                dialog.blocking_show()
            })
            .await
            // A dialog that could not be shown is not an approval.
            .unwrap_or(false)
        })
    }
}

/// A requested change to one of the launcher's endpoints.
struct EndpointChange<'a> {
    name: &'static str,
    consequence: &'static str,
    current: String,
    requested: &'a str,
}

impl<'a> EndpointChange<'a> {
    fn chain(current: String, requested: &'a str) -> Self {
        Self {
            name: "Chain node",
            consequence:
                "The launcher reads balances from this node and sends it your signed transactions.",
            current,
            requested,
        }
    }

    fn identity(current: String, requested: &'a str) -> Self {
        Self {
            name: "QOR ID service",
            consequence: "The launcher signs in here, and your vault signs the challenges this service issues.",
            current,
            requested,
        }
    }

    fn is_change(&self) -> bool {
        self.current.trim_end_matches('/') != self.requested.trim_end_matches('/')
    }
}

/// Confirm, host-side, that the launcher may be repointed (roadmap L1.4).
///
/// Whoever runs an endpoint sees what the launcher sends it and, for QOR ID,
/// chooses the challenges the vault signs. So a change the webview asks for
/// takes effect only once the person at the keyboard approves it. One prompt
/// covers every endpoint in the request, and setting an endpoint to the value it
/// already has asks nothing.
async fn confirm_endpoint_changes(
    confirm: &dyn Confirm,
    changes: &[EndpointChange<'_>],
) -> QorResult<()> {
    let changed: Vec<&EndpointChange<'_>> = changes.iter().filter(|c| c.is_change()).collect();
    if changed.is_empty() {
        return Ok(());
    }

    let mut body = String::new();
    for change in &changed {
        body.push_str(&format!(
            "{}\nFrom: {}\nTo: {}\n{}\n\n",
            change.name, change.current, change.requested, change.consequence
        ));
    }
    body.push_str("Approve only an address you trust.");

    let prompt = Prompt {
        title: "Change where the launcher connects?".into(),
        body,
        approve: "Change".into(),
    };

    if confirm.confirm(&prompt).await {
        Ok(())
    } else {
        Err(QorError::Declined)
    }
}

// ──────────────────── the sign-in that comes with unlocking ───────────────────

/// How long an unlock stands in for the sign-in dialog (ADR-016).
const ARRIVAL_WINDOW: Duration = Duration::from_secs(5 * 60);

/// Permission, opened by unlocking or creating the vault, for its one account
/// (ADR-099) to answer QOR ID challenges without a second approval (ADR-016).
///
/// Opening the vault is the approval: since ADR-056 nothing is asked for it, so
/// the grant rests on the person being signed in to their computer. The grant covers one account and
/// only the challenges [`IdentityConsent`] guards, which are domain-tagged and
/// cannot move CGT. It closes when a QOR ID session is established, when the
/// vault locks, or after [`ARRIVAL_WINDOW`]. Transfers, grant claims, wallet
/// links and endpoint changes never consult it.
#[derive(Default)]
pub struct ArrivalGrant(parking_lot::Mutex<Option<(String, Instant)>>);

impl ArrivalGrant {
    fn open(&self, address: &str) {
        self.open_until(address, Instant::now() + ARRIVAL_WINDOW);
    }

    fn open_until(&self, address: &str, until: Instant) {
        *self.0.lock() = Some((address.to_string(), until));
    }

    fn close(&self) {
        *self.0.lock() = None;
    }

    /// Whether the grant is open, unexpired and for `address`, in either form.
    fn covers(&self, address: &str) -> bool {
        let Ok(wanted) = vault::normalise_address(address) else {
            return false;
        };
        self.0.lock().as_ref().is_some_and(|(granted, until)| {
            Instant::now() < *until && vault::normalise_address(granted).ok() == Some(wanted)
        })
    }
}

/// Approval for a QOR ID challenge: the arrival grant when it covers the account,
/// and the host dialog otherwise.
struct IdentityConsent<'a> {
    grant: &'a ArrivalGrant,
    address: &'a str,
    otherwise: &'a dyn Confirm,
}

impl Confirm for IdentityConsent<'_> {
    fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a> {
        if self.grant.covers(self.address) {
            return Box::pin(async { true });
        }
        self.otherwise.confirm(prompt)
    }
}

/// Shared application state.
pub struct AppState {
    /// The signed-in profile's wallet (ADR-099), placed in its folder while it is signed in.
    pub vault: Arc<Vault>,
    pub chain: Arc<ChainClient>,
    pub identity: Arc<IdentityClient>,
    /// Social's socket (ADR-091, L8.5), held here so no token reaches the webview.
    pub social: Arc<social::Social>,
    /// Translation on this PC (ADR-092): Mozilla's catalogue, and the engine and models kept by hash.
    pub translate: Arc<translate::Translator>,
    /// QOR Media, docked inside the launcher when a file is opened from the Library (ADR-094).
    pub media: media::Media,
    /// ARQADE's Qt app, docked inside the launcher on its ARQADE screen (ADR-102).
    pub arqade_app: arqade::app::App,
    /// Release-gate progress for the development dashboard (roadmap L2.2).
    pub dashboard: Arc<gates::Dashboard>,
    /// Open briefly after an unlock, for the QOR ID sign-in (ADR-016).
    pub arrival: ArrivalGrant,
    /// The signed-in profile's own things (ADR-099): its folder, its Mesh and its Library. None while no profile is
    /// signed in, and then nothing personal can be read or done.
    pub personal: parking_lot::RwLock<Option<Arc<Personal>>>,
    /// The launcher's data directory: the device's settings, `profiles.json`, and each profile's folder under it.
    pub data_dir: std::path::PathBuf,
    /// The launcher's local data directory: translation models, and each profile's Library index under it.
    pub local_dir: std::path::PathBuf,
}

/// What belongs to the signed-in profile and runs only while it is signed in (ADR-099).
pub struct Personal {
    pub id: String,
    /// Its folder: wallet, avatar, trade partners, listing drafts, paid requests, published work's bytes, the Mesh.
    pub dir: std::path::PathBuf,
    /// The Mesh (ADR-095): its node, seeding only when the person turned it on.
    pub mesh: Arc<mesh::Mesh>,
    /// The Library's index of the person's own folders (ADR-094), and the watcher that follows them.
    pub library: Arc<library::Library>,
    pub library_watch: Option<library::watch::Watch>,
}

impl AppState {
    fn new(data_dir: std::path::PathBuf, local_dir: std::path::PathBuf) -> QorResult<Self> {
        Self::build(data_dir, local_dir, vault::keychain::system())
    }

    /// The state over `data_dir` and `local_dir`, with wallet keys kept in `keys`. Tests keep them in memory.
    fn build(
        data_dir: std::path::PathBuf,
        local_dir: std::path::PathBuf,
        keys: Arc<dyn vault::keychain::KeyStore>,
    ) -> QorResult<Self> {
        let settings = config::Settings::load(&data_dir);
        tracing::info!(rpc = %settings.rpc_url, auth = %settings.auth_url, "endpoints");

        // The one wallet from before profiles becomes the first profile (ADR-099). Its QOR ID tokens were kept under
        // no profile's name; they are dropped, and the wallet's key signs in again.
        if profiles::migrate(&data_dir, &local_dir)?.is_some() {
            identity::forget_unscoped_tokens();
        }

        Ok(Self {
            vault: Arc::new(Vault::unplaced(keys)),
            chain: Arc::new(ChainClient::new(settings.rpc_url)?),
            identity: Arc::new(IdentityClient::new(settings.auth_url)?),
            social: Arc::new(social::Social::new(settings.social_url)),
            translate: Arc::new(translate::Translator::new(local_dir.join("translations"))?),
            media: media::Media::default(),
            arqade_app: arqade::app::App::default(),
            dashboard: Arc::new(gates::Dashboard::new()?),
            arrival: ArrivalGrant::default(),
            personal: parking_lot::RwLock::new(None),
            data_dir,
            local_dir,
        })
    }

    /// The signed-in profile's own things, or [`QorError::NoProfile`].
    pub fn personal(&self) -> QorResult<Arc<Personal>> {
        self.personal.read().clone().ok_or(QorError::NoProfile)
    }

    /// The signed-in profile's folder.
    pub fn personal_dir(&self) -> QorResult<std::path::PathBuf> {
        Ok(self.personal()?.dir.clone())
    }

    /// The signed-in profile's Mesh.
    pub fn mesh(&self) -> QorResult<Arc<mesh::Mesh>> {
        Ok(Arc::clone(&self.personal()?.mesh))
    }

    /// The signed-in profile's Library.
    pub fn library(&self) -> QorResult<Arc<library::Library>> {
        Ok(Arc::clone(&self.personal()?.library))
    }

    /// Write the current endpoints to disk so they survive a restart.
    fn persist(&self) -> QorResult<()> {
        config::Settings {
            rpc_url: self.chain.endpoint(),
            auth_url: self.identity.endpoint(),
            social_url: self.social.endpoint(),
        }
        .save(&self.data_dir)
    }
}

/// Told when the Library's index changed, so the interface refreshes.
type Changed = Arc<dyn Fn() + Send + Sync>;

fn library_changed(app: &tauri::AppHandle) -> Changed {
    let told = app.clone();
    Arc::new(move || library::commands::changed(&told))
}

/// [`enter_profile`] for the running app.
fn enter(app: &tauri::AppHandle, state: &AppState, id: &str) -> QorResult<()> {
    enter_profile(state, id, library_changed(app))
}

/// [`leave_profile`] for the running app.
async fn leave(app: &tauri::AppHandle, state: &AppState) {
    leave_profile(state, &SocialEvents(app.clone())).await
}

/// Make `id` the signed-in profile (ADR-099): the wallet and QOR ID's tokens kept in its folder and under its name, and
/// its Library and Mesh started. The wallet is opened by the caller.
fn enter_profile(state: &AppState, id: &str, changed: Changed) -> QorResult<()> {
    if state.personal.read().is_some() {
        return Err(QorError::VaultOpen);
    }
    let dir = profiles::dir(&state.data_dir, id);
    let local = profiles::local_dir(&state.local_dir, id);
    std::fs::create_dir_all(&dir)?;
    std::fs::create_dir_all(&local)?;
    state.vault.place(Some(dir.clone()))?;
    state.identity.set_profile(Some(id));

    // The Mesh is paid for its work through the vault and the chain, once the person allows it (ADR-097).
    let mesh = mesh::Mesh::new(&dir, mesh::Net::Public);
    mesh.connect(Arc::clone(&state.vault), Arc::clone(&state.chain));
    let library = library::Library::open(&local.join("library"))?;
    // The Library follows its folders while the profile is signed in, and looks again for what changed while it was
    // not (ADR-094).
    let library_watch =
        match library::watch::Watch::start(Arc::clone(&library), Arc::clone(&changed)) {
            Ok(watch) => Some(watch),
            Err(e) => {
                tracing::warn!("the Library will not follow its folders: {e}");
                None
            }
        };
    {
        let library = Arc::clone(&library);
        tauri::async_runtime::spawn_blocking(move || {
            if library.scan_all().is_ok() {
                changed();
            }
        });
    }
    // The Mesh seeds again if the person left seeding on (ADR-095). Off, nothing is started.
    {
        let mesh = Arc::clone(&mesh);
        tauri::async_runtime::spawn(async move {
            if let Err(e) = mesh.resume().await {
                tracing::warn!("the Mesh did not start: {e}");
            }
        });
    }

    *state.personal.write() = Some(Arc::new(Personal {
        id: id.to_string(),
        dir,
        mesh,
        library,
        library_watch,
    }));
    profiles::set_signed_in(&state.data_dir, Some(id))?;
    Ok(())
}

/// Close the signed-in profile completely (ADR-099): Social and QOR ID signed out and its tokens deleted, the Mesh
/// stopped, the Library closed, the wallet locked and its key taken out of the keychain, so it opens again only with
/// the profile's PIN. Nothing of it stays in memory.
async fn leave_profile(state: &AppState, events: &dyn social::Events) {
    // ARQADE holds a session with its server for this profile's QOR ID: it closes with the profile.
    state.arqade_app.quit();
    state.social.disconnect(events);
    state.identity.logout().await;
    state.identity.set_profile(None);
    state.arrival.close();
    let personal = state.personal.write().take();
    if let Some(personal) = personal {
        personal.mesh.stop().await;
    }
    if let Err(e) = state.vault.forget_key() {
        tracing::warn!(error = %e, "the wallet's key could not be taken out of the keychain");
    }
    let _ = state.vault.place(None);
    if let Err(e) = profiles::set_signed_in(&state.data_dir, None) {
        tracing::warn!(error = %e, "which profile is signed in could not be noted");
    }
}

/// Everything the shell needs to decide what to render on startup.
#[derive(Serialize)]
pub struct LauncherState {
    pub version: String,
    /// This PC's profiles, for *Who's playing?* (ADR-099).
    pub profiles: Vec<profiles::ProfileView>,
    pub vault: VaultStatus,
    pub session: Option<Session>,
    pub chain_endpoint: String,
    pub auth_endpoint: String,
    pub token: TokenInfo,
}

/// Static facts about CGT, so the UI never hardcodes them.
///
/// **No supply figure.** This struct carried `total_supply_embers` and
/// `total_supply_cgt`, which the Settings surface showed as "Total supply", at
/// the fixed 13,000,000,000 ADR-003 superseded. There is nothing to replace it
/// with: the base supply is released over a decay curve and complemented by
/// perpetual issuance and burn (ADR-004), so no fixed total exists, and the
/// chain declares no such constant for the launcher to read. A figure that
/// cannot be pinned to the node the launcher is connected to does not belong in
/// a panel of facts about it.
#[derive(Serialize)]
pub struct TokenInfo {
    pub symbol: &'static str,
    pub name: &'static str,
    pub decimals: u32,
    /// The unit people see and type: the Spark, a thousandth of a CGT (ADR-101).
    pub sub_unit: &'static str,
    pub sub_units_per_cgt: u32,
    /// The chain's atomic unit: the ember, 10^-18 CGT (ADR-101).
    pub atomic_unit: &'static str,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            symbol: cgt::SYMBOL,
            name: cgt::NAME,
            decimals: cgt::DECIMALS,
            sub_unit: "Spark",
            sub_units_per_cgt: cgt::SPARKS_PER_CGT as u32,
            atomic_unit: "ember",
        }
    }
}

/// A balance, rendered every way the UI might want it.
#[derive(Serialize)]
pub struct Balance {
    pub address: String,
    pub embers: String,
    pub cgt: String,
    pub display: String,
    /// The same amount in Sparks (ADR-101), or in full as CGT when it is finer than a Spark.
    pub in_sparks: String,
}

impl Balance {
    fn of(address: String, embers: u128) -> Self {
        Self {
            address,
            embers: embers.to_string(),
            cgt: cgt::format_cgt(embers),
            display: format!("{} {}", cgt::format_cgt_grouped(embers), cgt::SYMBOL),
            in_sparks: cgt::format_sparks(embers),
        }
    }
}

// ───────────────────────────── commands: shell ──────────────────────────────

#[tauri::command]
fn launcher_state(state: tauri::State<'_, AppState>) -> LauncherState {
    LauncherState {
        version: env!("CARGO_PKG_VERSION").to_string(),
        profiles: profiles::read(&state.data_dir).views(profiles::now()),
        vault: state.vault.status(),
        session: state.identity.session(),
        chain_endpoint: state.chain.endpoint(),
        auth_endpoint: state.identity.endpoint(),
        token: TokenInfo::default(),
    }
}

/// Emit a line from the frontend into the host's log.
///
/// The webview's own console is not visible when the app is launched normally,
/// so without this there is no way to see what the interface actually measured
/// or decided. Diagnostics only; it carries no authority.
#[tauri::command]
fn log_diagnostic(message: String) {
    tracing::info!(target: "qor::ui", "{message}");
}

// ───────────────────────────── commands: vault ──────────────────────────────

#[tauri::command]
fn vault_status(state: tauri::State<'_, AppState>) -> VaultStatus {
    state.vault.status()
}

/// Generate a recovery phrase for the user to write down.
///
/// This returns the phrase to the UI exactly once, before any vault exists, so
/// it can be displayed for transcription. It is not persisted anywhere until
/// `vault_create` is called with it.
#[tauri::command]
fn vault_generate_phrase() -> Result<String, QorError> {
    vault::derive::generate_mnemonic()
}

/// Where unlocking or creating the vault led, so the Gate can go straight on.
#[derive(Serialize)]
pub struct Arrival {
    pub accounts: Vec<AccountView>,
    /// The wallet's account, which sign-in was tried with. A name, when one is needed, is claimed for it.
    pub address: Option<String>,
    /// Signed in to QOR ID.
    pub session: Option<Session>,
    /// The vault's key has no QOR ID yet, so the only thing left to ask is a name.
    pub needs_name: bool,
    /// Why signing in did not happen, when it could not. The Gate then offers
    /// the manual routes.
    pub sign_in_problem: Option<String>,
}

/// Carry on from the vault into QOR ID without a second approval (ADR-016).
///
/// A session that is still valid is kept. Otherwise the wallet's one account (ADR-099) answers a sign-in challenge
/// under the grant the unlock has just opened. When the key has no QOR ID yet, the grant stays open for the name claim.
async fn arrive(app: tauri::AppHandle, state: &AppState, accounts: Vec<AccountView>) -> Arrival {
    let chosen = accounts.first().map(|a| a.address.clone());
    let mut arrival = Arrival {
        accounts,
        address: chosen.clone(),
        session: state.identity.session(),
        needs_name: false,
        sign_in_problem: None,
    };
    let Some(address) = chosen else {
        arrival.sign_in_problem = Some("the vault holds no account to sign in with".into());
        return arrival;
    };
    note_profile(state, Some(&address), None);
    if let Some(session) = arrival.session.as_ref() {
        adopt(state, session);
        return arrival;
    }

    state.arrival.open(&address);
    let dialog = HostDialog(app);
    let consent = IdentityConsent {
        grant: &state.arrival,
        address: &address,
        otherwise: &dialog,
    };

    match state
        .identity
        .login_with_key(&state.vault, &consent, &address)
        .await
    {
        Ok(session) => {
            state.arrival.close();
            adopt(state, &session);
            arrival.session = Some(session);
        }
        Err(error) if identity::needs_registration(&error) => arrival.needs_name = true,
        Err(error) => {
            state.arrival.close();
            arrival.sign_in_problem = Some(error.to_string());
        }
    }
    arrival
}

/// A QOR ID is signed in: its name is the profile's on *Who's playing?* (ADR-099).
fn adopt(state: &AppState, session: &Session) {
    note_profile(state, None, Some(&session.qor_id));
}

/// Open the signed-in profile's wallet and note its account in `profiles.json`, so the account is known to be this
/// profile's from the moment it opens, whether or not QOR ID is reached.
fn unlock_profile(state: &AppState) -> QorResult<Vec<AccountView>> {
    let accounts = state.vault.unlock(&*vault::hello::system())?;
    note_profile(state, accounts.first().map(|a| a.address.as_str()), None);
    Ok(accounts)
}

/// Note the signed-in profile's account or name in `profiles.json`. A note that cannot be written fails nothing.
fn note_profile(state: &AppState, address: Option<&str>, name: Option<&str>) {
    let Ok(personal) = state.personal() else {
        return;
    };
    if let Err(e) = profiles::note(&state.data_dir, &personal.id, address, name) {
        tracing::warn!(error = %e, "the profile could not be noted");
    }
}

/// Run vault work off the async runtime: the keychain, a last Windows Hello
/// gesture and Argon2id all take a moment.
async fn off_runtime<T: Send + 'static>(
    work: impl FnOnce() -> QorResult<T> + Send + 'static,
) -> QorResult<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| QorError::Internal(format!("vault work did not finish: {e}")))?
}

/// A new profile with a new wallet from `phrase`, its key in the keychain, signed in; then try QOR ID (ADR-016,
/// ADR-056, ADR-099). Refused while another profile is signed in: it signs out first.
#[tauri::command]
async fn vault_create(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    phrase: String,
) -> Result<Arrival, QorError> {
    vault::derive::validate_mnemonic(&phrase)?;
    if state.personal.read().is_some() {
        return Err(QorError::VaultOpen);
    }
    let profile = profiles::add(&state.data_dir, &state.local_dir)?;
    enter(&app, &state, &profile.id)?;
    let vault = Arc::clone(&state.vault);
    match off_runtime(move || vault.create(&phrase)).await {
        Ok(accounts) => Ok(arrive(app, &state, accounts).await),
        Err(error) => {
            leave(&app, &state).await;
            let _ = profiles::remove(&state.data_dir, &state.local_dir, &profile.id);
            Err(error)
        }
    }
}

/// The address a recovery phrase opens, before anything is stored: its first account, or the one at `index`.
#[tauri::command]
fn vault_preview_phrase(phrase: String, index: Option<u32>) -> Result<String, QorError> {
    Vault::preview_at(&phrase, index.unwrap_or(0))
}

/// Seal a wallet from a phrase the person already holds, for the account at `index` (the first by default); then try
/// QOR ID (ADR-016).
///
/// Signed in to a profile whose wallet will not open, it replaces that wallet, setting the old one aside once the
/// person approves in a host dialog. Signed out, it opens the profile already on this PC for that account, as a
/// forgotten PIN is replaced, or makes a new profile (ADR-099).
#[tauri::command]
async fn vault_restore(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    phrase: String,
    index: Option<u32>,
) -> Result<Arrival, QorError> {
    let index = index.unwrap_or(0);
    let address = Vault::preview_at(&phrase, index)?;
    let dialog = HostDialog(app.clone());
    let made = if state.personal.read().is_some() {
        None
    } else {
        let registry = profiles::read(&state.data_dir);
        let known = registry
            .profiles
            .iter()
            .find(|p| p.address == address)
            .map(|p| p.id.clone());
        match known {
            Some(id) => {
                enter(&app, &state, &id)?;
                // The PIN sealed the key the old wallet had; a restored wallet has a new key, so a new PIN is chosen.
                profiles::clear_pin(&state.data_dir, &id)?;
                None
            }
            None => {
                let profile = profiles::add(&state.data_dir, &state.local_dir)?;
                enter(&app, &state, &profile.id)?;
                Some(profile.id)
            }
        }
    };
    match state.vault.restore_at(&dialog, &phrase, index).await {
        Ok((accounts, _aside)) => Ok(arrive(app, &state, accounts).await),
        Err(error) => {
            if let Some(id) = made {
                leave(&app, &state).await;
                let _ = profiles::remove(&state.data_dir, &state.local_dir, &id);
            }
            Err(error)
        }
    }
}

/// Open the vault, then try QOR ID (ADR-016, ADR-056). A keychain vault opens
/// with nothing asked; a Windows Hello vault asks Hello one last time and moves.
#[tauri::command]
async fn vault_unlock(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Arrival, QorError> {
    let vault = Arc::clone(&state.vault);
    let accounts = off_runtime(move || vault.unlock(&*vault::hello::system())).await?;
    Ok(arrive(app, &state, accounts).await)
}

/// Move a passphrase vault from before ADR-055 to the keychain, with its
/// passphrase typed once more, then try QOR ID.
#[tauri::command]
async fn vault_move_to_keychain(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    passphrase: String,
) -> Result<Arrival, QorError> {
    let vault = Arc::clone(&state.vault);
    let accounts = off_runtime(move || vault.move_to_keychain(&passphrase)).await?;
    Ok(arrive(app, &state, accounts).await)
}

#[tauri::command]
fn vault_lock(state: tauri::State<'_, AppState>) {
    state.arrival.close();
    state.vault.lock();
}

/// The account the rail shows, and what a product's first publish signs as (ADR-090): since ADR-099 the wallet's one
/// account.
#[tauri::command]
fn vault_set_active(state: tauri::State<'_, AppState>, address: String) -> Result<(), QorError> {
    state.vault.set_active(&address)
}

/// Re-export the recovery phrase, once the person approves a host dialog.
#[tauri::command]
async fn vault_export_phrase(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, QorError> {
    state.vault.export_phrase(&HostDialog(app)).await
}

/// Try QOR ID again with the wallet's account, as opening it does (ADR-016): for when the service was unreachable at
/// launch.
#[tauri::command]
async fn qor_sign_in(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Arrival, QorError> {
    let VaultStatus::Unlocked { accounts } = state.vault.status() else {
        return Err(QorError::VaultLocked);
    };
    Ok(arrive(app, &state, accounts).await)
}

// ───────────────────────────── commands: profiles (ADR-099) ─────────────────────────────

/// This PC's profiles, for *Who's playing?*: names, no balances, nothing secret.
#[tauri::command]
fn profiles_list(state: tauri::State<'_, AppState>) -> Vec<profiles::ProfileView> {
    profiles::read(&state.data_dir).views(profiles::now())
}

/// Open a profile with its PIN: its wallet's key is unsealed and put back in the keychain, the profile is signed in, its
/// wallet opens and QOR ID is tried. Refused while another profile is signed in.
#[tauri::command]
async fn profile_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
    pin: String,
) -> Result<Arrival, QorError> {
    let accounts = open_profile(
        &state,
        &id,
        pin,
        library_changed(&app),
        &SocialEvents(app.clone()),
    )
    .await?;
    Ok(arrive(app, &state, accounts).await)
}

/// The core of [`profile_open`]: unseal the key with the PIN, sign the profile in and open its wallet. A profile that does
/// not open is left as it was found, signed out.
async fn open_profile(
    state: &AppState,
    id: &str,
    pin: String,
    changed: Changed,
    events: &dyn social::Events,
) -> QorResult<Vec<AccountView>> {
    if state.personal.read().is_some() {
        return Err(QorError::VaultOpen);
    }
    let root = state.data_dir.clone();
    let opening = id.to_string();
    let key = off_runtime(move || {
        let key = profiles::open_with_pin(&root, &opening, &pin, profiles::now())?;
        String::from_utf8(key.0.clone())
            .map(zeroize::Zeroizing::new)
            .map_err(|_| QorError::VaultCorrupt("the sealed key is not text".into()))
    })
    .await?;
    enter_profile(state, id, changed)?;
    let opened = state
        .vault
        .restore_key(&key)
        .and_then(|()| unlock_profile(state));
    if opened.is_err() {
        leave_profile(state, events).await;
    }
    opened
}

/// Sign the profile out completely (ADR-099). A profile with no PIN yet chooses one now, so it can be opened again; a
/// PIN given to a profile that has one replaces it.
#[tauri::command]
async fn profile_sign_out(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    pin: Option<String>,
) -> Result<(), QorError> {
    sign_out_profile(&state, pin, &SocialEvents(app)).await
}

/// The core of [`profile_sign_out`].
async fn sign_out_profile(
    state: &AppState,
    pin: Option<String>,
    events: &dyn social::Events,
) -> QorResult<()> {
    let personal = state.personal()?;
    let has_pin = profiles::read(&state.data_dir)
        .find(&personal.id)
        .is_some_and(|p| p.pin.is_some());
    match (pin, state.vault.key_secret()) {
        (Some(pin), Ok(key)) => seal_pin(state, &personal.id, key, pin).await?,
        (None, Ok(_)) if !has_pin => {
            return Err(QorError::Pin(
                "Choose a PIN for this QOR ID first, so only you can open it again.".into(),
            ))
        }
        // No key in the keychain: the wallet cannot be opened silently anyway, so there is nothing to seal.
        _ => {}
    }
    leave_profile(state, events).await;
    Ok(())
}

/// Choose a new PIN for the signed-in profile.
#[tauri::command]
async fn profile_set_pin(state: tauri::State<'_, AppState>, pin: String) -> Result<(), QorError> {
    let personal = state.personal()?;
    let key = state.vault.key_secret()?;
    seal_pin(&state, &personal.id, key, pin).await
}

async fn seal_pin(
    state: &AppState,
    id: &str,
    key: zeroize::Zeroizing<String>,
    pin: String,
) -> QorResult<()> {
    profiles::validate_pin(&pin)?;
    let root = state.data_dir.clone();
    let id = id.to_string();
    off_runtime(move || profiles::set_pin(&root, &id, &key, &pin)).await
}

/// Another account an older wallet's phrase has used, which can be made a profile of its own.
#[derive(Serialize)]
pub struct OtherAccount {
    pub index: u32,
    pub address: String,
    pub balance: String,
}

/// The accounts of the signed-in wallet's phrase that have been used on the chain and are not this profile or another on
/// this PC (ADR-099 decision 5). Only wallets from before profiles share a phrase with any other account.
#[tauri::command]
async fn profile_other_accounts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<OtherAccount>, QorError> {
    const LOOK: u32 = 8;
    let own = state.vault.index()?;
    let known: Vec<String> = profiles::read(&state.data_dir)
        .profiles
        .into_iter()
        .map(|p| p.address)
        .collect();
    let mut found = Vec::new();
    for index in (0..LOOK).filter(|i| *i != own) {
        let address = state.vault.address_at(index)?;
        if known.contains(&address) {
            continue;
        }
        let nonce = state.chain.nonce(&address).await.unwrap_or(0);
        let embers = state.chain.balance(&address).await.unwrap_or(0);
        if nonce > 0 || embers > 0 {
            found.push(OtherAccount {
                index,
                address,
                balance: format!("{} {}", cgt::format_cgt_grouped(embers), cgt::SYMBOL),
            });
        }
    }
    Ok(found)
}

/// Make another account of the signed-in wallet's phrase a profile of its own, and switch to it (ADR-099 decision 5).
/// The profile signed in now signs out, with `pin` chosen if it has none.
#[tauri::command]
async fn profile_split(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    index: u32,
    pin: Option<String>,
) -> Result<Arrival, QorError> {
    let accounts = split_profile(
        &state,
        index,
        pin,
        library_changed(&app),
        &SocialEvents(app.clone()),
    )
    .await?;
    Ok(arrive(app, &state, accounts).await)
}

/// The core of [`profile_split`].
async fn split_profile(
    state: &AppState,
    index: u32,
    pin: Option<String>,
    changed: Changed,
    events: &dyn social::Events,
) -> QorResult<Vec<AccountView>> {
    let personal = state.personal()?;
    if index == state.vault.index()? {
        return Err(QorError::VaultExists);
    }
    let address = state.vault.address_at(index)?;
    let registry = profiles::read(&state.data_dir);
    if registry.profiles.iter().any(|p| p.address == address) {
        return Err(QorError::VaultExists);
    }
    let has_pin = registry.find(&personal.id).is_some_and(|p| p.pin.is_some());
    if pin.is_none() && !has_pin {
        return Err(QorError::Pin(
            "Choose a PIN for this QOR ID first, so only you can open it again.".into(),
        ));
    }
    if let Some(pin) = &pin {
        profiles::validate_pin(pin)?;
    }
    let profile = profiles::add(&state.data_dir, &state.local_dir)?;
    profiles::note(&state.data_dir, &profile.id, Some(&address), None)?;
    if let Err(error) = state
        .vault
        .copy_to(&profiles::dir(&state.data_dir, &profile.id), index)
    {
        let _ = profiles::remove(&state.data_dir, &state.local_dir, &profile.id);
        return Err(error);
    }
    if let Some(pin) = pin {
        let key = state.vault.key_secret()?;
        seal_pin(state, &personal.id, key, pin).await?;
    }
    leave_profile(state, events).await;
    enter_profile(state, &profile.id, changed)?;
    unlock_profile(state)
}

// ───────────────────────────── commands: chain ──────────────────────────────

#[tauri::command]
async fn chain_status(state: tauri::State<'_, AppState>) -> Result<ChainStatus, QorError> {
    Ok(state.chain.status().await)
}

#[tauri::command]
async fn chain_set_endpoint(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    endpoint: String,
) -> Result<(), QorError> {
    // Checked before the dialog, so nobody is asked to approve a non-address,
    // and normalised first, so the dialog shows what would actually be used.
    let endpoint = chain::normalise_endpoint(&endpoint)?;

    confirm_endpoint_changes(
        &HostDialog(app),
        &[EndpointChange::chain(state.chain.endpoint(), &endpoint)],
    )
    .await?;
    state.chain.set_endpoint(&endpoint)?;
    state.persist()?;
    Ok(())
}

#[tauri::command]
async fn cgt_balance(
    state: tauri::State<'_, AppState>,
    address: String,
) -> Result<Balance, QorError> {
    let embers = state.chain.balance(&address).await?;
    Ok(Balance::of(address, embers))
}

/// Parse a human amount and report it back, so the UI can preview a send
/// without duplicating denomination logic in TypeScript.
#[tauri::command]
fn cgt_parse_amount(amount: String) -> Result<Balance, QorError> {
    // CGT or Sparks, as typed (ADR-101).
    let embers = cgt::parse_amount(&amount)?;
    Ok(Balance::of(String::new(), embers))
}

/// Sign and submit a CGT transfer.
#[tauri::command]
async fn cgt_send(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    to: String,
    amount: String,
) -> Result<TransferReceipt, QorError> {
    let embers = cgt::parse_amount(&amount)?;
    let receipt = state
        .chain
        .transfer(&state.vault, &HostDialog(app), &from, &to, embers)
        .await?;
    Ok(receipt)
}

/// The nonce the account's next transaction must carry.
///
/// Exposed because a non-zero nonce is proof the account has signed something
/// that the chain accepted, which is what the Ascent reads to know whether the
/// holder has ever actually moved value. A Substrate account has one nonce; the
/// custom devnet's separate request nonce is gone with it (ADR-040).
#[tauri::command]
async fn cgt_nonce(state: tauri::State<'_, AppState>, address: String) -> Result<u64, QorError> {
    state.chain.nonce(&address).await
}

/// Claim the starter grant with the account's own key.
#[tauri::command]
async fn cgt_claim_starter(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    address: String,
) -> Result<ClaimResult, QorError> {
    let result = state
        .chain
        .claim_starter(&state.vault, &HostDialog(app), &address)
        .await?;
    Ok(result)
}

/// Transactions involving an account, most recent first.
#[tauri::command]
async fn cgt_history(
    state: tauri::State<'_, AppState>,
    address: String,
    limit: Option<u64>,
) -> Result<Vec<HistoryEntry>, QorError> {
    state.chain.history(&address, limit.unwrap_or(50)).await
}

// ─────────────────────────── DRC-369 (M4.1) ─────────────────────────────────

/// Mint the commit a project's HEAD points at as a DRC-369 asset.
///
/// The project is read, its commit's files fingerprinted and its manifest built
/// before anyone is asked; the bytes go into the temporary content store only
/// after the person has approved and the vault has signed, so a declined mint
/// writes nothing and moves nothing.
#[tauri::command]
async fn qontrol_mint(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
    from: String,
) -> Result<chain::assets::MintReceipt, QorError> {
    let path = qontrol::resolve(&path)?;
    let snapshot = qontrol::publish::snapshot(&path)?;
    let request = chain::assets::MintRequest {
        name: snapshot.name.clone(),
        reference: snapshot.reference,
        commit: snapshot.commit,
        branch: snapshot.branch.clone(),
        files: snapshot.files(),
    };
    let store = content::TemporaryStore::in_data_dir(&state.personal_dir()?);

    let receipt = state
        .chain
        .mint(&state.vault, &HostDialog(app), &from, &request, || {
            snapshot.write_to(&path, &store).map_err(QorError::from)
        })
        .await?;
    Ok(receipt)
}

/// Every DRC-369 asset an account holds, read from chain storage.
#[tauri::command]
async fn drc369_assets(
    state: tauri::State<'_, AppState>,
    address: String,
) -> Result<Vec<chain::assets::OwnedAsset>, QorError> {
    state.chain.assets_of(&address).await
}

/// Make an asset permanent, through the same dialog and vault as a mint.
#[tauri::command]
async fn drc369_make_permanent(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    collection: u32,
    item: u32,
) -> Result<chain::assets::PermanenceReceipt, QorError> {
    let receipt = state
        .chain
        .make_permanent(&state.vault, &HostDialog(app), &from, collection, item)
        .await?;
    Ok(receipt)
}

/// Send assets to another account, in one transaction, through the same dialog
/// and vault as every other signature (L4.5).
#[tauri::command]
async fn drc369_trade(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    to: String,
    items: Vec<chain::assets::TradeItem>,
    message: Option<String>,
    // `label` is what the recipient was reached by, when that was a QOR ID
    // rather than an address. It is kept beside the address in this machine's
    // own list and sent nowhere.
    label: Option<String>,
) -> Result<chain::assets::TradeReceipt, QorError> {
    let receipt = state
        .chain
        .trade(
            &state.vault,
            &HostDialog(app),
            &from,
            &to,
            &items,
            message.as_deref(),
        )
        .await?;
    // Remembered only now, from the address the chain accepted rather than the
    // text that was typed. A convenience that cannot be written does not fail a
    // trade that already happened.
    if let Err(e) = state
        .personal_dir()
        .and_then(|dir| partners::remember(&dir, &receipt.to, label.as_deref()))
    {
        tracing::warn!(error = %e, "the trade partner could not be remembered");
    }
    Ok(receipt)
}

// ─────────────────────── selling and buying (L4.6) ──────────────────────────

/// One asset, whoever holds it: what it is, whether it is for sale, and what a
/// sale would pay. `asset` is whatever the person pasted — an asset's number,
/// or the reference its holder copied from the asset's menu. There is no
/// catalogue to browse (ADR-028), so this is how a buyer reaches an asset.
#[tauri::command]
async fn drc369_sale(
    state: tauri::State<'_, AppState>,
    asset: String,
    viewer: Option<String>,
) -> Result<chain::sales::SaleView, QorError> {
    state.chain.sale(&asset, viewer.as_deref()).await
}

/// What a sale of an asset at a typed price would pay, part by part, before
/// anything is listed. The price is parsed exactly; no float touches it.
#[tauri::command]
async fn drc369_sale_preview(
    state: tauri::State<'_, AppState>,
    collection: u32,
    item: u32,
    price: String,
) -> Result<chain::sales::Breakdown, QorError> {
    let embers = cgt::parse_amount(&price)?;
    state.chain.sale_preview(collection, item, embers).await
}

/// List an asset for sale at a price in CGT, or change its price, through the
/// same dialog and vault as every other signature.
#[tauri::command]
async fn drc369_list(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    collection: u32,
    item: u32,
    price: String,
) -> Result<chain::sales::ListReceipt, QorError> {
    // Exact, or refused: excess precision is never rounded away (AGENTS.md §5).
    let embers = cgt::parse_amount(&price)?;
    state
        .chain
        .list(
            &state.vault,
            &HostDialog(app),
            &from,
            collection,
            item,
            embers,
        )
        .await
}

/// Withdraw an asset's listing.
#[tauri::command]
async fn drc369_unlist(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    collection: u32,
    item: u32,
) -> Result<chain::sales::UnlistReceipt, QorError> {
    state
        .chain
        .unlist(&state.vault, &HostDialog(app), &from, collection, item)
        .await
}

/// Buy a listed asset. `price_embers` and `root` are what the buyer was shown:
/// if either has changed on chain, nobody is asked and nothing is sent.
#[tauri::command]
async fn drc369_buy(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    collection: u32,
    item: u32,
    price_embers: String,
    root: String,
) -> Result<chain::sales::SaleReceipt, QorError> {
    let seen = price_embers
        .parse::<u128>()
        .map_err(|_| QorError::BadAmount("the price that was shown is not a number".into()))?;
    state
        .chain
        .buy(
            &state.vault,
            &HostDialog(app),
            &from,
            collection,
            item,
            chain::sales::Seen {
                price_embers: seen,
                root: &root,
            },
        )
        .await
}

// ───────────────────────────── the Market (L7.2) ────────────────────────────

/// Every asset offered for sale on the chain, as the connected node holds it
/// at its latest finalised block: arranged, and one window of it in detail.
/// There is no indexer (ADR-028), so this walks the chain's listing storage,
/// paged and bounded (`chain/market.rs`). It reads and signs nothing; buying
/// from it goes through `drc369_buy` like every other purchase.
#[tauri::command]
async fn drc369_market(
    state: tauri::State<'_, AppState>,
    viewer: Option<String>,
    show: Option<chain::market::Show>,
    order: Option<chain::market::Order>,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<chain::market::MarketPage, QorError> {
    state
        .chain
        .market(
            viewer.as_deref(),
            show.unwrap_or_default(),
            order.unwrap_or_default(),
            offset.unwrap_or(0),
            limit.unwrap_or(chain::market::PAGE),
        )
        .await
}

// ─────────────────── listing drafts, on this machine only ───────────────────

/// The categories a listing can have, and what each one asks. One table, in the
/// host, so the form cannot invent a vocabulary of its own (`listings.rs`).
#[tauri::command]
async fn listing_vocabulary() -> Result<&'static [listings::Category], QorError> {
    Ok(listings::CATEGORIES)
}

/// Save an asset's description as a draft. **A draft is not published**: the
/// chain holds a listing's price and nothing else, and no storefront exists to
/// show a description (M5.4), so this stays on this machine.
#[tauri::command]
async fn listing_save(
    state: tauri::State<'_, AppState>,
    draft: listings::Draft,
) -> Result<listings::Listing, QorError> {
    listings::save(&state.personal_dir()?, &draft)
}

/// Every draft on this machine, most recently changed first.
#[tauri::command]
async fn listing_drafts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<listings::Listing>, QorError> {
    Ok(listings::list(&state.personal_dir()?))
}

/// Throw a draft away.
#[tauri::command]
async fn listing_discard(
    state: tauri::State<'_, AppState>,
    collection: u32,
    item: u32,
) -> Result<(), QorError> {
    listings::discard(&state.personal_dir()?, collection, item)
}

/// Who this machine has traded with, most recent first. Local, and never sent
/// anywhere (`partners.rs`).
#[tauri::command]
async fn trade_partners(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<partners::Partner>, QorError> {
    Ok(partners::list(&state.personal_dir()?))
}

/// A candidate endpoint and whether it answered.
#[derive(Serialize)]
pub struct Probe {
    pub label: String,
    pub rpc_url: String,
    pub auth_url: String,
    pub rpc_ok: bool,
    pub auth_ok: bool,
    pub detail: Option<String>,
}

/// Probe the known endpoint pairs so the Gate can show what is actually up.
///
/// This exists because the compiled-in production endpoints are currently
/// unreachable, and a launcher that simply says "offline" without telling you
/// what it tried, or letting you point somewhere else, is a dead end.
#[tauri::command]
async fn probe_endpoints(state: tauri::State<'_, AppState>) -> Result<Vec<Probe>, QorError> {
    let current_rpc = state.chain.endpoint();
    let current_auth = state.identity.endpoint();

    let mut candidates = vec![
        (
            "Local stack".to_string(),
            chain::LOCAL_RPC.to_string(),
            identity::LOCAL_AUTH.to_string(),
        ),
        (
            "Demiurge Cloud".to_string(),
            chain::DEFAULT_RPC.to_string(),
            identity::DEFAULT_AUTH.to_string(),
        ),
    ];

    // Include whatever is configured now, if it is neither of the known pairs.
    if !candidates.iter().any(|(_, r, _)| *r == current_rpc) {
        candidates.insert(0, ("Current".to_string(), current_rpc, current_auth));
    }

    let mut probes = Vec::with_capacity(candidates.len());

    for (label, rpc_url, auth_url) in candidates {
        // A short timeout: this runs while someone is waiting at the Gate, and a
        // dead host should be reported in seconds, not after a default timeout.
        let rpc_probe = ChainClient::new(rpc_url.clone())?;
        let status = rpc_probe.status().await;

        let auth_ok = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .ok()
            .map(|client| (client, auth_url.trim_end_matches("/api/v1").to_string()))
            .map(|(client, base)| async move {
                client
                    .get(format!("{base}/health"))
                    .send()
                    .await
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
            });

        let auth_ok = match auth_ok {
            Some(future) => future.await,
            None => false,
        };

        probes.push(Probe {
            label,
            rpc_url,
            auth_url,
            rpc_ok: status.reachable,
            auth_ok,
            detail: status.detail,
        });
    }

    Ok(probes)
}

/// Point the launcher at an endpoint pair and remember the choice, once the
/// change is approved in a host dialog.
#[tauri::command]
async fn use_endpoints(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    rpc_url: String,
    auth_url: String,
) -> Result<(), QorError> {
    // The two are not the same kind of address: the chain speaks WebSocket
    // (ADR-040), QOR ID speaks HTTP. Both are checked before the dialog.
    let rpc_url = chain::normalise_endpoint(&rpc_url)?;
    let auth_url = auth_url.trim().to_string();
    if !config::is_http_url(&auth_url) {
        return Err(QorError::Rpc(
            "the identity endpoint must start with http:// or https://".into(),
        ));
    }

    confirm_endpoint_changes(
        &HostDialog(app),
        &[
            EndpointChange::chain(state.chain.endpoint(), &rpc_url),
            EndpointChange::identity(state.identity.endpoint(), &auth_url),
        ],
    )
    .await?;

    state.chain.set_endpoint(&rpc_url)?;
    state.identity.set_endpoint(&auth_url);
    state.persist()?;
    Ok(())
}

// ──────────────────────────── commands: identity ────────────────────────────

#[tauri::command]
async fn qor_login(
    state: tauri::State<'_, AppState>,
    identifier: String,
    password: String,
) -> Result<Session, QorError> {
    state.identity.login(&identifier, &password).await
}

#[tauri::command]
async fn qor_login_with_key(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    address: String,
) -> Result<Session, QorError> {
    let dialog = HostDialog(app);
    let consent = IdentityConsent {
        grant: &state.arrival,
        address: &address,
        otherwise: &dialog,
    };
    let session = state
        .identity
        .login_with_key(&state.vault, &consent, &address)
        .await?;
    state.arrival.close();
    Ok(session)
}

/// Claim a QOR ID for a vault key. Straight after an unlock that found no QOR ID
/// for the key, the unlock has already approved it (ADR-016).
#[tauri::command]
async fn qor_register_with_key(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    address: String,
    username: Option<String>,
) -> Result<Session, QorError> {
    let dialog = HostDialog(app);
    let consent = IdentityConsent {
        grant: &state.arrival,
        address: &address,
        otherwise: &dialog,
    };
    let session = state
        .identity
        .register_with_key(&state.vault, &consent, &address, username.as_deref())
        .await?;
    state.arrival.close();
    adopt(&state, &session);
    Ok(session)
}

#[tauri::command]
async fn qor_username_available(
    state: tauri::State<'_, AppState>,
    username: String,
) -> Result<bool, QorError> {
    state.identity.username_available(&username).await
}

/// Bind a vault address to the signed-in QOR ID.
#[tauri::command]
async fn qor_link_wallet(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    address: String,
) -> Result<Session, QorError> {
    let session = state
        .identity
        .link_wallet(&state.vault, &HostDialog(app), &address)
        .await?;
    adopt(&state, &session);
    Ok(session)
}

#[tauri::command]
async fn qor_restore(state: tauri::State<'_, AppState>) -> Result<Session, QorError> {
    let session = state.identity.restore().await?;
    adopt(&state, &session);
    Ok(session)
}

// ── The account's avatar (ADR-079) ────────────────────────────────────────────

/// The largest file offered for upload; QOR ID refuses more and cleans the rest.
const AVATAR_UPLOAD_LIMIT: u64 = 4 * 1024 * 1024;

fn avatar_cache(state: &AppState) -> QorResult<(std::path::PathBuf, std::path::PathBuf)> {
    let dir = state.personal_dir()?;
    Ok((dir.join("avatar.bin"), dir.join("avatar.type")))
}

/// Keep a copy of the account's avatar on this machine, so it shows offline. No avatar clears the copy.
async fn refresh_avatar_copy(state: &AppState, session: &Session) {
    let Ok((bytes_path, type_path)) = avatar_cache(state) else {
        return;
    };
    match session.avatar_url.as_deref() {
        Some(url) => match state.identity.fetch_avatar(url).await {
            Ok((bytes, content_type)) => {
                let _ = std::fs::write(&bytes_path, bytes);
                let _ = std::fs::write(&type_path, content_type);
            }
            Err(e) => tracing::warn!("the avatar could not be kept for offline: {e}"),
        },
        None => {
            let _ = std::fs::remove_file(&bytes_path);
            let _ = std::fs::remove_file(&type_path);
        }
    }
}

/// Choose an image or GIF in a native file picker and make it the account's avatar. The webview never reads files:
/// the host opens the picker, reads the one file chosen, and uploads it. Cancelling changes nothing.
#[tauri::command]
async fn qor_choose_avatar(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Option<Session>, QorError> {
    let picked = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        app.dialog()
            .file()
            .set_title("Choose your avatar")
            .add_filter("Images and GIFs", &["png", "jpg", "jpeg", "webp", "gif"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| QorError::Internal(e.to_string()))?;
    let Some(picked) = picked else {
        return Ok(None);
    };
    let path = picked
        .into_path()
        .map_err(|e| QorError::Io(format!("that file could not be opened: {e}")))?;
    let size = std::fs::metadata(&path)
        .map_err(|e| QorError::Io(e.to_string()))?
        .len();
    if size > AVATAR_UPLOAD_LIMIT {
        return Err(QorError::Io("that file is larger than 4 MB".into()));
    }
    let bytes = std::fs::read(&path).map_err(|e| QorError::Io(e.to_string()))?;
    let content_type = match path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "application/octet-stream",
    };
    let session = state.identity.upload_avatar(bytes, content_type).await?;
    refresh_avatar_copy(&state, &session).await;
    Ok(Some(session))
}

/// Remove the account's avatar.
#[tauri::command]
async fn qor_remove_avatar(state: tauri::State<'_, AppState>) -> Result<Session, QorError> {
    let session = state.identity.remove_avatar().await?;
    refresh_avatar_copy(&state, &session).await;
    Ok(session)
}

/// The account's own avatar from this machine's copy, as a data URL, or none. Refreshes the copy first when online.
#[tauri::command]
async fn qor_avatar(
    state: tauri::State<'_, AppState>,
    refresh: bool,
) -> Result<Option<String>, QorError> {
    if refresh {
        if let Ok(session) = state.identity.restore().await {
            refresh_avatar_copy(&state, &session).await;
        }
    }
    let Ok((bytes_path, type_path)) = avatar_cache(&state) else {
        return Ok(None);
    };
    let (Ok(bytes), Ok(content_type)) = (
        std::fs::read(bytes_path),
        std::fs::read_to_string(type_path),
    ) else {
        return Ok(None);
    };
    use base64::Engine as _;
    Ok(Some(format!(
        "data:{};base64,{}",
        content_type.trim(),
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )))
}

/// The signed-in account's level and XP (ADR-078).
#[tauri::command]
async fn qor_progress(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, QorError> {
    state.identity.progress().await
}

/// The tutorial is finished: QOR ID grants its XP once (ADR-078).
#[tauri::command]
async fn qor_tutorial_done(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, QorError> {
    state.identity.tutorial_done().await
}

/// Choose this account's QOR ID name, once, while it is on a name QOR ID made (ADR-091 decision 5).
#[tauri::command]
async fn qor_claim_username(
    state: tauri::State<'_, AppState>,
    username: String,
) -> Result<Session, QorError> {
    state.identity.claim_username(&username).await
}

/// How a translation download is going, for the interface (ADR-092).
struct TranslateProgress(tauri::AppHandle);

impl translate::Progress for TranslateProgress {
    fn progress(&self, file: &translate::File, done: u64) {
        use tauri::Emitter;
        let _ = self.0.emit(
            "translate://progress",
            serde_json::json!({ "hash": file.hash, "kind": file.kind, "done": done, "total": file.download_size() }),
        );
    }
}

/// The languages Mozilla can translate between on this PC, and the language this PC is set to (ADR-092).
#[tauri::command]
async fn translate_languages(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, QorError> {
    let languages = state.translate.languages().await?;
    Ok(serde_json::json!({ "languages": languages, "system": translate::system_language() }))
}

/// What translating from one language to another needs, and how much is still to download; null when nothing is needed
/// or nothing can be done.
#[tauri::command]
async fn translate_plan(
    state: tauri::State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Option<translate::Plan>, QorError> {
    state.translate.plan(&from, &to).await
}

/// Fetch what a translation needs, reporting `translate://progress` as it goes.
#[tauri::command]
async fn translate_prepare(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    to: String,
) -> Result<Option<translate::Plan>, QorError> {
    let plan = state.translate.plan(&from, &to).await?;
    if let Some(plan) = &plan {
        state
            .translate
            .prepare(plan, &TranslateProgress(app))
            .await?;
    }
    Ok(plan)
}

/// A checked file's bytes, for the engine in the webview, as raw bytes rather than JSON.
#[tauri::command]
fn translate_file(
    state: tauri::State<'_, AppState>,
    hash: String,
) -> Result<tauri::ipc::Response, QorError> {
    Ok(tauri::ipc::Response::new(state.translate.read(&hash)?))
}

/// The language some words are in, told on this PC, or null when there are too few to tell. The words are not kept or
/// logged.
#[tauri::command]
fn translate_detect(text: String) -> Option<String> {
    translate::detect(&text)
}

/// Social's connection reports to the interface as Tauri events (ADR-091 decision 3).
struct SocialEvents(tauri::AppHandle);

impl social::Events for SocialEvents {
    fn status(&self, status: &social::Status) {
        use tauri::Emitter;
        let _ = self.0.emit("social://status", status);
    }
    fn welcome(&self, welcome: &serde_json::Value) {
        use tauri::Emitter;
        let _ = self.0.emit("social://welcome", welcome);
    }
    fn event(&self, event: &serde_json::Value) {
        use tauri::Emitter;
        let _ = self.0.emit("social://event", event);
    }
}

/// Connect to Social as the signed-in person, and stay connected. The host holds the socket and the token.
///
/// Async, so Tauri runs it on its Tokio runtime: the connection is a task spawned there, and a synchronous command runs
/// on the main thread, where spawning panicked and aborted the launcher at start-up (16 October 2026).
#[tauri::command]
async fn social_connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), QorError> {
    state
        .social
        .connect(state.identity.clone(), Arc::new(SocialEvents(app)))
}

#[tauri::command]
fn social_disconnect(app: tauri::AppHandle, state: tauri::State<'_, AppState>) {
    state.social.disconnect(&SocialEvents(app));
}

/// Where the connection stands, and its last welcome, for an interface that opens after it connected.
#[tauri::command]
fn social_status(state: tauri::State<'_, AppState>) -> serde_json::Value {
    serde_json::json!({
        "status": state.social.status(),
        "welcome": state.social.welcome(),
        "endpoint": state.social.endpoint(),
    })
}

/// Ask Social something (`services/social/src/api.rs` lists the methods); Social checks every rule itself.
#[tauri::command]
async fn social_request(
    state: tauri::State<'_, AppState>,
    method: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, QorError> {
    if method.is_empty()
        || method.len() > 40
        || !method
            .chars()
            .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_')
    {
        return Err(QorError::Social {
            code: "bad_request".into(),
            message: "not a Social method".into(),
        });
    }
    state.social.request(&method, params).await
}

/// ARQADE in a window of its own (ADR-078 decision 5): opened, or brought forward if it already is.
#[tauri::command]
async fn arqade_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), QorError> {
    arqade::open(&app, &state.identity.endpoint()).map_err(QorError::Internal)
}

#[tauri::command]
async fn qor_set_auth_endpoint(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    endpoint: String,
) -> Result<(), QorError> {
    let endpoint = endpoint.trim();
    if !endpoint.starts_with("http://") && !endpoint.starts_with("https://") {
        return Err(QorError::Auth(
            "endpoint must start with http:// or https://".into(),
        ));
    }
    confirm_endpoint_changes(
        &HostDialog(app),
        &[EndpointChange::identity(
            state.identity.endpoint(),
            endpoint,
        )],
    )
    .await?;
    state.identity.set_endpoint(endpoint);
    state.persist()?;
    Ok(())
}

// ─────────────────────── commands: development dashboard ────────────────────

/// Release-gate progress, computed from `docs/GATES.toml` (roadmap L2.2).
///
/// `refresh` bypasses the short caches on CI and service readings. A file that
/// cannot be read is reported inside the report, not as an error.
#[tauri::command]
async fn gates_report(
    state: tauri::State<'_, AppState>,
    refresh: Option<bool>,
) -> Result<gates::Report, QorError> {
    Ok(state
        .dashboard
        .report(&state.data_dir, refresh.unwrap_or(false))
        .await)
}

/// Run a suite `docs/GATES.toml` defines, once approved in a host dialog.
///
/// A suite builds and runs repository code, so the webview can ask for a run by
/// name but cannot start one unseen.
#[tauri::command]
async fn gates_run_suite(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    suite: String,
) -> Result<gates::TestRun, QorError> {
    state
        .dashboard
        .run_suite(&state.data_dir, &suite, &HostDialog(app))
        .await
}

// ─────────────────────────────── entrypoint ─────────────────────────────────

// ── qor://pay (ADR-076, ADR-077) ────────────────────────────────────────────

/// Request ids being paid right now, so two clicks on one link cannot both reach the dialog.
static PAYING: parking_lot::Mutex<std::collections::BTreeSet<String>> =
    parking_lot::Mutex::new(std::collections::BTreeSet::new());

/// A `qor://pay` link opened the launcher. It is checked (`pay::parse`), asked in the host dialog, paid, and the
/// outcome told in a native message. Nothing is paid without the dialog's approval.
fn handle_pay_link(app: tauri::AppHandle, link: String) {
    tauri::async_runtime::spawn(async move {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.set_focus();
        }
        let (title, body) = match pay_link(&app, &link).await {
            Ok((request, receipt)) => (
                "Paid".to_string(),
                format!(
                    "{} {} paid to {} for \"{}\".\n\nIt is final on Demiurge Devnet. {} will see it shortly.\n\n\
                     Transaction: {}",
                    cgt::format_cgt_grouped(request.amount_embers),
                    cgt::SYMBOL,
                    request.to,
                    request.label,
                    request.app_name,
                    receipt.tx_hash
                ),
            ),
            Err(QorError::Declined) => (
                "Not paid".to_string(),
                "You declined. Nothing was paid.".to_string(),
            ),
            Err(e) => ("Not paid".to_string(), format!("Nothing was paid: {e}.")),
        };
        let _ = tauri::async_runtime::spawn_blocking(move || {
            use tauri_plugin_dialog::{DialogExt, MessageDialogKind};
            let mut dialog = app
                .dialog()
                .message(body)
                .title(title)
                .kind(MessageDialogKind::Info);
            if let Some(window) = app.get_webview_window("main") {
                dialog = dialog.parent(&window);
            }
            dialog.blocking_show();
        })
        .await;
    });
}

async fn pay_link(
    app: &tauri::AppHandle,
    link: &str,
) -> QorResult<(pay::PayRequest, chain::pay::PayReceipt)> {
    let request = pay::parse(link, chrono::Utc::now().timestamp())?;
    let state = app.state::<AppState>();
    let paid = pay::PaidRequests::in_dir(state.personal_dir().map_err(|_| {
        QorError::PaymentRefused(
            "no QOR ID is signed in. Open the QOR Launcher, choose yours, then click the link again".into(),
        )
    })?);
    if paid.is_paid(&request) {
        return Err(QorError::PaymentRefused(
            "this request was already paid".into(),
        ));
    }
    let key = format!("{}:{}", request.app_id, request.id);
    if !PAYING.lock().insert(key.clone()) {
        return Err(QorError::PaymentRefused(
            "this request is already waiting for your answer".into(),
        ));
    }
    let outcome = async {
        // The signed-in profile's account pays, the only one open (ADR-099): a signed-out QOR ID's CGT cannot be
        // reached from here. The dialog names it.
        let from = match state.vault.status() {
            VaultStatus::Unlocked { accounts } if accounts.is_empty() => {
                return Err(QorError::PaymentRefused(
                    "the vault holds no account".into(),
                ))
            }
            VaultStatus::Unlocked { .. } => state.vault.active_address()?,
            _ => {
                return Err(QorError::PaymentRefused(
                    "the vault is not open. Open the QOR Launcher, then click the link again"
                        .into(),
                ))
            }
        };
        state
            .chain
            .pay_request(&state.vault, &HostDialog(app.clone()), &from, &request)
            .await
    }
    .await;
    PAYING.lock().remove(&key);
    let receipt = outcome?;
    // The chain holds the payment whatever happens here; a failure to remember it only means a second click would
    // reach the dialog, where the app's own record of the payment answers it.
    if let Err(e) = paid.record(&request, chrono::Utc::now().timestamp()) {
        tracing::warn!("a paid request could not be remembered: {e}");
    }
    Ok((request, receipt))
}

/// Build and run the launcher.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qor_launcher_lib=info,warn".into()),
        )
        .init();

    tauri::Builder::default()
        // First, so a second launch (a `qor://` link clicked while the launcher runs) hands its link to this one
        // instead of starting another; with the `deep-link` feature the link arrives through `on_open_url` below.
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| format!("cannot locate the application data directory: {e}"))?;

            std::fs::create_dir_all(&data_dir)
                .map_err(|e| format!("cannot create {}: {e}", data_dir.display()))?;

            tracing::info!(dir = %data_dir.display(), "QOR Launcher starting");

            // Translation models and each profile's Library index are large and this machine's own, so they are kept with
            // its local data, not roaming.
            let local_dir = app
                .path()
                .app_local_data_dir()
                .map_err(|e| format!("cannot locate the local data directory: {e}"))?;
            app.manage(AppState::new(data_dir, local_dir).map_err(|e| e.to_string())?);

            // The profile signed in last comes back with nothing asked (ADR-056, ADR-099); its wallet is opened by the
            // interface before its first frame. Signed out, the interface shows Who's playing?.
            {
                let state = app.state::<AppState>();
                let registry = profiles::read(&state.data_dir);
                if let Some(id) = registry
                    .signed_in
                    .as_deref()
                    .filter(|id| registry.find(id).is_some())
                {
                    if let Err(e) = enter(app.handle(), &state, id) {
                        tracing::warn!("the profile signed in last could not be opened: {e}");
                    }
                }
            }

            // `qor://pay` (ADR-076). The installer registers the scheme; registering again here covers a launcher run
            // without installing, and points the scheme at this copy.
            {
                use tauri_plugin_deep_link::DeepLinkExt;
                #[cfg(any(windows, target_os = "linux"))]
                if let Err(e) = app.deep_link().register_all() {
                    tracing::warn!("the qor:// scheme could not be registered: {e}");
                }
                let handle = app.handle().clone();
                app.deep_link().on_open_url(move |event| {
                    for url in event.urls() {
                        if url.scheme() == "qor" {
                            handle_pay_link(handle.clone(), url.to_string());
                        }
                    }
                });
                // A link that started the launcher.
                if let Ok(Some(urls)) = app.deep_link().get_current() {
                    for url in urls {
                        if url.scheme() == "qor" {
                            handle_pay_link(app.handle().clone(), url.to_string());
                        }
                    }
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            qontrol::commands::qontrol_pick_folder,
            qontrol::commands::qontrol_open,
            qontrol::commands::qontrol_create,
            qontrol::commands::qontrol_read,
            qontrol::commands::qontrol_diff,
            qontrol::commands::qontrol_check,
            qontrol::commands::qontrol_commit,
            qontrol::commands::qontrol_branch,
            qontrol::commands::qontrol_switch,
            qontrol::commands::qontrol_discard,
            qontrol_mint,
            qq::qq_save_scene,
            qq::qq_load_scene,
            qq::qq_list_scenes,
            qq::qq_open_studio,
            drc369_assets,
            drc369_make_permanent,
            drc369_trade,
            drc369_sale,
            drc369_sale_preview,
            drc369_list,
            drc369_unlist,
            drc369_buy,
            drc369_market,
            trade_partners,
            listing_vocabulary,
            listing_save,
            listing_drafts,
            listing_discard,
            launcher_state,
            log_diagnostic,
            vault_status,
            vault_generate_phrase,
            vault_create,
            vault_preview_phrase,
            vault_restore,
            vault_unlock,
            vault_move_to_keychain,
            vault_lock,
            vault_set_active,
            vault_export_phrase,
            profiles_list,
            profile_open,
            profile_sign_out,
            profile_set_pin,
            profile_other_accounts,
            profile_split,
            chain_status,
            chain_set_endpoint,
            probe_endpoints,
            use_endpoints,
            cgt_balance,
            cgt_parse_amount,
            cgt_send,
            cgt_nonce,
            cgt_claim_starter,
            cgt_history,
            qor_login,
            qor_sign_in,
            qor_login_with_key,
            qor_register_with_key,
            qor_username_available,
            qor_link_wallet,
            qor_restore,
            qor_progress,
            qor_choose_avatar,
            qor_remove_avatar,
            qor_avatar,
            qor_tutorial_done,
            qor_claim_username,
            social_connect,
            social_disconnect,
            social_status,
            social_request,
            translate_languages,
            translate_plan,
            translate_prepare,
            translate_file,
            translate_detect,
            library::commands::library_folders,
            library::commands::library_add_folder,
            library::commands::library_remove_folder,
            library::commands::library_scan,
            library::commands::library_search,
            library::commands::library_shelf,
            library::commands::library_counts,
            library::commands::library_thumbnail,
            library::commands::library_reveal,
            library::commands::library_publish,
            library::commands::library_opened,
            library::commands::library_open,
            media::media_place,
            media::media_hide,
            media::media_theme,
            mesh::commands::mesh_status,
            mesh::commands::mesh_held,
            mesh::commands::mesh_configure,
            mesh::commands::mesh_ticket,
            mesh::commands::mesh_fetch,
            mesh::commands::mesh_allow_paid,
            mesh::commands::mesh_fetch_root,
            mesh::commands::mesh_open_escrow,
            qor_set_auth_endpoint,
            arqade_open,
            arqade::app::arqade_app_open,
            arqade::app::arqade_app_place,
            arqade::app::arqade_app_hide,
            arqade::app::arqade_app_theme,
            arqade::app::arqade_app_player,
            arqade::app::arqade_app_prefs,
            host::launcher_motion,
            gates_report,
            gates_run_suite,
        ])
        .run(tauri::generate_context!())
        .expect("QOR Launcher failed to start");
}

/// Test support: a [`Confirm`] with a fixed answer that records every prompt.
#[cfg(test)]
pub(crate) mod testing {
    // The whole test process's log (moved from host::tests so Social's check shares the one logger).
    #[derive(Clone, Default)]
    pub struct Captured(pub std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl std::io::Write for Captured {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Captured {
        type Writer = Captured;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    /// The whole test process's log, at every level, from the first time it is asked for. One logger for the process,
    /// not one for a test: `tracing` caches across threads whether each log line is wanted, and a logger set for one test's
    /// thread loses lines that other tests, running without one, first decided nobody wanted. With this one installed
    /// everything is wanted, always.
    pub fn captured_log() -> &'static Captured {
        static CAPTURED: std::sync::OnceLock<Captured> = std::sync::OnceLock::new();
        CAPTURED.get_or_init(|| {
            let captured = Captured::default();
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::TRACE)
                .with_ansi(false)
                .with_writer(captured.clone())
                .finish();
            tracing::subscriber::set_global_default(subscriber)
                .expect("the only process-wide logger in these tests");
            captured
        })
    }

    // The core's, moved with the vault that its tests confirm through (ADR-104).
    pub use qor_launcher_core::confirm::testing::Scripted;
}

#[cfg(test)]
mod tests {
    use super::testing::Scripted;
    use super::*;

    #[tokio::test]
    async fn an_endpoint_change_takes_effect_only_once_approved() {
        let declining = Scripted::declining();
        let refused = confirm_endpoint_changes(
            &declining,
            &[EndpointChange::chain(
                "https://rpc.demiurge.cloud".into(),
                "https://attacker.example",
            )],
        )
        .await
        .unwrap_err();
        assert_eq!(refused.kind(), "declined");

        {
            let asked = declining.asked.lock();
            assert_eq!(asked.len(), 1);
            assert!(
                asked[0].body.contains("https://rpc.demiurge.cloud")
                    && asked[0].body.contains("https://attacker.example"),
                "the prompt names both the current and the requested endpoint"
            );
        }

        let approving = Scripted::approving();
        confirm_endpoint_changes(
            &approving,
            &[EndpointChange::identity(
                "https://demiurge.cloud/api/v1".into(),
                "http://127.0.0.1:8080/api/v1",
            )],
        )
        .await
        .unwrap();
        assert_eq!(approving.times_asked(), 1);
    }

    #[tokio::test]
    async fn setting_an_endpoint_to_its_current_value_asks_nothing() {
        let declining = Scripted::declining();
        confirm_endpoint_changes(
            &declining,
            &[
                EndpointChange::chain("http://127.0.0.1:9944".into(), "http://127.0.0.1:9944/"),
                EndpointChange::identity(
                    "http://127.0.0.1:8080/api/v1".into(),
                    "http://127.0.0.1:8080/api/v1",
                ),
            ],
        )
        .await
        .unwrap();
        assert_eq!(declining.times_asked(), 0);
    }

    #[tokio::test]
    async fn one_prompt_covers_every_endpoint_that_changes() {
        let approving = Scripted::approving();
        confirm_endpoint_changes(
            &approving,
            &[
                EndpointChange::chain("http://127.0.0.1:9944".into(), "https://rpc.example"),
                EndpointChange::identity(
                    "http://127.0.0.1:8080/api/v1".into(),
                    "https://id.example/api/v1",
                ),
            ],
        )
        .await
        .unwrap();

        let asked = approving.asked.lock();
        assert_eq!(asked.len(), 1);
        assert!(asked[0].body.contains("https://rpc.example"));
        assert!(asked[0].body.contains("https://id.example/api/v1"));
    }

    async fn approved(
        grant: &ArrivalGrant,
        address: &str,
        dialog: &dyn Confirm,
        prompt: &Prompt,
    ) -> bool {
        IdentityConsent {
            grant,
            address,
            otherwise: dialog,
        }
        .confirm(prompt)
        .await
    }

    /// ADR-016: the unlock approves the QOR ID sign-in for the vault's first
    /// account, briefly, and for nothing and nobody else.
    #[tokio::test]
    async fn an_unlock_approves_the_qor_id_sign_in_for_its_account_only() {
        let grant = ArrivalGrant::default();
        let first = vault::derive::address_of(&[0x11u8; 32]);
        let other = vault::derive::address_of(&[0x22u8; 32]);
        let prompt = Prompt {
            title: "Sign in to QOR ID?".into(),
            body: String::new(),
            approve: "Sign in".into(),
        };
        let dialog = Scripted::declining();

        assert!(
            !approved(&grant, &first, &dialog, &prompt).await,
            "nothing unlocked: the dialog decides"
        );
        assert_eq!(dialog.times_asked(), 1);

        grant.open(&first);
        assert!(
            approved(&grant, &first, &dialog, &prompt).await,
            "the unlocked account signs in without a dialog"
        );
        assert!(
            approved(&grant, &format!("0x{}", "11".repeat(32)), &dialog, &prompt).await,
            "in either address form: SS58 for people, hex for advanced views"
        );
        assert_eq!(dialog.times_asked(), 1);

        assert!(
            !approved(&grant, &other, &dialog, &prompt).await,
            "another account still asks"
        );
        assert_eq!(dialog.times_asked(), 2);

        grant.close();
        assert!(
            !approved(&grant, &first, &dialog, &prompt).await,
            "a closed grant asks again"
        );

        let past = Instant::now()
            .checked_sub(Duration::from_secs(1))
            .unwrap_or_else(Instant::now);
        grant.open_until(&first, past);
        assert!(
            !approved(&grant, &first, &dialog, &prompt).await,
            "an expired grant asks again"
        );
        assert_eq!(dialog.times_asked(), 4);
    }

    /// Social's reports, for a profile signing out in a test: nowhere to send them.
    struct Quiet;
    impl social::Events for Quiet {
        fn status(&self, _: &social::Status) {}
        fn welcome(&self, _: &serde_json::Value) {}
        fn event(&self, _: &serde_json::Value) {}
    }

    fn unchanged() -> Changed {
        Arc::new(|| {})
    }

    /// ADR-099, the whole path through the host: the wallet from before profiles becomes the first profile and opens
    /// with nothing asked; it cannot be left without a PIN; signed out, nothing of it is open or in the keychain, and it
    /// opens only with its PIN; another account its phrase used becomes a profile of its own, and the first profile's
    /// key leaves the keychain as it does.
    #[tokio::test(flavor = "multi_thread")]
    async fn a_profile_is_left_completely_and_opens_again_only_with_its_pin() {
        const PHRASE: &str =
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon \
                              abandon about";
        let base =
            std::env::temp_dir().join(format!("qor-lifecycle-{}", uuid::Uuid::new_v4().simple()));
        let (data, local) = (base.join("roaming"), base.join("local"));
        std::fs::create_dir_all(&data).unwrap();
        std::fs::create_dir_all(&local).unwrap();
        let keys = Arc::new(vault::keychain::testing::MemoryKeychain::default());

        // A launcher from before 0.1.13: one wallet in the data directory, and personal files beside it.
        let old = Vault::with_keys(&data, keys.clone());
        let first = old.create(PHRASE).unwrap()[0].address.clone();
        old.lock();
        std::fs::write(data.join("avatar.bin"), b"me").unwrap();
        std::fs::write(data.join("account-owners.json"), b"{}").unwrap();

        let state = AppState::build(data.clone(), local.clone(), keys.clone()).unwrap();
        let id = profiles::read(&data)
            .signed_in
            .expect("the old wallet is the first profile");
        assert!(
            state.personal().is_err(),
            "nothing personal before the profile is entered"
        );
        enter_profile(&state, &id, unchanged()).unwrap();
        let opened = unlock_profile(&state).unwrap();
        assert_eq!(opened.len(), 1);
        assert_eq!(
            opened[0].address, first,
            "it opens as the account it was, with nothing asked"
        );
        assert_eq!(
            std::fs::read(state.personal_dir().unwrap().join("avatar.bin")).unwrap(),
            b"me"
        );
        assert!(!data.join("account-owners.json").exists());
        assert_eq!(
            profiles::read(&data).find(&id).unwrap().address,
            first,
            "its account is noted as it opens"
        );

        // Leaving needs a PIN, and a PIN must be one.
        let refused = sign_out_profile(&state, None, &Quiet).await.unwrap_err();
        assert_eq!(refused.kind(), "pin");
        assert_eq!(
            sign_out_profile(&state, Some("12".into()), &Quiet)
                .await
                .unwrap_err()
                .kind(),
            "pin"
        );
        assert!(
            state.personal().is_ok(),
            "a refused sign-out leaves the profile signed in"
        );

        sign_out_profile(&state, Some("123456".into()), &Quiet)
            .await
            .unwrap();
        assert!(state.personal().is_err());
        assert!(matches!(state.vault.status(), VaultStatus::Absent));
        assert!(
            keys.0.lock().is_empty(),
            "the wallet's key left the keychain"
        );
        assert_eq!(profiles::read(&data).signed_in, None);
        // The wallet itself, placed again by hand, does not open: its key is gone.
        state.vault.place(Some(profiles::dir(&data, &id))).unwrap();
        assert_eq!(
            state
                .vault
                .unlock(&*vault::hello::system())
                .unwrap_err()
                .kind(),
            "keychain"
        );
        state.vault.place(None).unwrap();

        // A wrong PIN opens nothing; the right one opens the same wallet.
        let wrong = open_profile(&state, &id, "654321".into(), unchanged(), &Quiet)
            .await
            .unwrap_err();
        assert_eq!(wrong.kind(), "pin");
        assert!(state.personal().is_err() && keys.0.lock().is_empty());
        let again = open_profile(&state, &id, "123456".into(), unchanged(), &Quiet)
            .await
            .unwrap();
        assert_eq!(again[0].address, first);
        assert_eq!(
            profiles::read(&data).signed_in.as_deref(),
            Some(id.as_str())
        );
        assert_eq!(
            open_profile(&state, &id, "123456".into(), unchanged(), &Quiet)
                .await
                .unwrap_err()
                .kind(),
            "vault_open",
            "one profile at a time"
        );

        // The phrase's second account becomes a profile of its own; the first signs out as it does.
        let second = Vault::preview_at(PHRASE, 1).unwrap();
        let split = split_profile(&state, 1, None, unchanged(), &Quiet)
            .await
            .unwrap();
        assert_eq!(split.len(), 1);
        assert_eq!(split[0].address, second);
        let registry = profiles::read(&data);
        assert_eq!(registry.profiles.len(), 2);
        let new_id = registry.signed_in.clone().unwrap();
        assert_ne!(new_id, id);
        assert_eq!(
            keys.0.lock().len(),
            1,
            "only the profile signed in has its key in the keychain"
        );
        assert_eq!(
            split_profile(&state, 0, Some("111111".into()), unchanged(), &Quiet)
                .await
                .unwrap_err()
                .kind(),
            "vault_exists",
            "an account that is already a profile is not made one again"
        );

        sign_out_profile(&state, Some("222222".into()), &Quiet)
            .await
            .unwrap();
        assert!(keys.0.lock().is_empty());
        let back = open_profile(&state, &id, "123456".into(), unchanged(), &Quiet)
            .await
            .unwrap();
        assert_eq!(
            back[0].address, first,
            "each profile opens as its own account, with its own PIN"
        );
        sign_out_profile(&state, None, &Quiet).await.unwrap();
        let other = open_profile(&state, &new_id, "222222".into(), unchanged(), &Quiet)
            .await
            .unwrap();
        assert_eq!(other[0].address, second);
        sign_out_profile(&state, None, &Quiet).await.unwrap();
        let _ = std::fs::remove_dir_all(&base);
    }
}
