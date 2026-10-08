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
pub mod cgt;
pub mod chain;
pub mod config;
pub mod content;
pub mod error;
pub mod gates;
pub mod identity;
pub mod listings;
pub mod partners;
pub mod pay;
pub mod qontrol;
pub mod qq;
pub mod vault;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::Manager;

use chain::{ChainClient, ChainStatus, ClaimResult, HistoryEntry, TransferReceipt};
use error::{QorError, QorResult};
use identity::{IdentityClient, Session};
use vault::{AccountView, Vault, VaultStatus};

// ─────────────────────────── host-side confirmation ─────────────────────────

/// What the person at the keyboard is asked to approve.
///
/// The host builds it from the values it is about to act on, never from text the
/// webview supplies, so the prompt describes what will actually happen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prompt {
    pub title: String,
    pub body: String,
    /// Label of the approving button. The other button is always "Cancel".
    pub approve: String,
}

/// The answer to a [`Prompt`]: `true` only if the person approved.
pub type Answer<'a> = Pin<Box<dyn Future<Output = bool> + Send + 'a>>;

/// Asks the person at the keyboard to approve an action (roadmap L1.4).
///
/// Every signature and every endpoint change goes through one of these. The
/// launcher's implementation is [`HostDialog`], a native dialog the host draws
/// and the webview can neither draw, dismiss nor answer. A compromised webview
/// can therefore ask for a signature, but cannot obtain one unseen.
///
/// The one approval that is not a dialog is [`IdentityConsent`] under an open
/// [`ArrivalGrant`]: straight after the vault opens, it approves the QOR ID
/// sign-in for the vault's first account (ADR-016, ADR-056).
pub trait Confirm: Send + Sync {
    fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a>;
}

/// The launcher's [`Confirm`]: a native dialog drawn by the host.
struct HostDialog(tauri::AppHandle);

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

/// Permission, opened by unlocking or creating the vault, for its first account
/// to answer QOR ID challenges without a second approval (ADR-016).
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
    pub vault: Arc<Vault>,
    pub chain: Arc<ChainClient>,
    pub identity: Arc<IdentityClient>,
    /// Release-gate progress for the development dashboard (roadmap L2.2).
    pub dashboard: Arc<gates::Dashboard>,
    /// Open briefly after an unlock, for the QOR ID sign-in (ADR-016).
    pub arrival: ArrivalGrant,
    /// Where the vault and settings live. Needed to persist endpoint changes.
    pub data_dir: std::path::PathBuf,
}

impl AppState {
    fn new(data_dir: std::path::PathBuf) -> QorResult<Self> {
        let settings = config::Settings::load(&data_dir);
        tracing::info!(rpc = %settings.rpc_url, auth = %settings.auth_url, "endpoints");

        Ok(Self {
            vault: Arc::new(Vault::new(&data_dir)),
            chain: Arc::new(ChainClient::new(settings.rpc_url)?),
            identity: Arc::new(IdentityClient::new(settings.auth_url)?),
            dashboard: Arc::new(gates::Dashboard::new()?),
            arrival: ArrivalGrant::default(),
            data_dir,
        })
    }

    /// Write the current endpoints to disk so they survive a restart.
    fn persist(&self) -> QorResult<()> {
        config::Settings {
            rpc_url: self.chain.endpoint(),
            auth_url: self.identity.endpoint(),
        }
        .save(&self.data_dir)
    }
}

/// Everything the shell needs to decide what to render on startup.
#[derive(Serialize)]
pub struct LauncherState {
    pub version: String,
    pub vault: VaultStatus,
    pub session: Option<Session>,
    pub chain_endpoint: String,
    pub auth_endpoint: String,
    pub token: TokenInfo,
}

/// Static facts about CGT, so the UI never hardcodes them.
///
/// **No supply figure.** This struct carried `total_supply_sparks` and
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
    pub sub_unit: &'static str,
}

impl Default for TokenInfo {
    fn default() -> Self {
        Self {
            symbol: cgt::SYMBOL,
            name: cgt::NAME,
            decimals: cgt::DECIMALS,
            sub_unit: "Spark",
        }
    }
}

/// A balance, rendered every way the UI might want it.
#[derive(Serialize)]
pub struct Balance {
    pub address: String,
    pub sparks: String,
    pub cgt: String,
    pub display: String,
}

// ───────────────────────────── commands: shell ──────────────────────────────

#[tauri::command]
fn launcher_state(state: tauri::State<'_, AppState>) -> LauncherState {
    LauncherState {
        version: env!("CARGO_PKG_VERSION").to_string(),
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
/// A session that is still valid is kept. Otherwise the vault's first account
/// answers a sign-in challenge under the grant the unlock has just opened.
/// When the key has no QOR ID yet, the grant stays open for the name claim.
async fn arrive(app: tauri::AppHandle, state: &AppState, accounts: Vec<AccountView>) -> Arrival {
    let mut arrival = Arrival {
        accounts,
        session: state.identity.session(),
        needs_name: false,
        sign_in_problem: None,
    };
    let Some(address) = arrival.accounts.first().map(|a| a.address.clone()) else {
        arrival.sign_in_problem = Some("the vault holds no account to sign in with".into());
        return arrival;
    };
    if arrival.session.is_some() {
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

/// Run vault work off the async runtime: the keychain, a last Windows Hello
/// gesture and Argon2id all take a moment.
async fn off_runtime<T: Send + 'static>(
    work: impl FnOnce() -> QorResult<T> + Send + 'static,
) -> QorResult<T> {
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| QorError::Internal(format!("vault work did not finish: {e}")))?
}

/// Create the vault, its key in the keychain, then try QOR ID (ADR-016, ADR-056).
#[tauri::command]
async fn vault_create(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    phrase: String,
) -> Result<Arrival, QorError> {
    let vault = Arc::clone(&state.vault);
    let accounts = off_runtime(move || vault.create(&phrase)).await?;
    Ok(arrive(app, &state, accounts).await)
}

/// The address a recovery phrase opens, before anything is stored.
#[tauri::command]
fn vault_preview_phrase(phrase: String) -> Result<String, QorError> {
    Vault::preview(&phrase)
}

/// Seal a vault from a phrase the person already holds, setting aside any vault
/// already here once they approve it in a host dialog; then try QOR ID (ADR-016).
#[tauri::command]
async fn vault_restore(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    phrase: String,
) -> Result<Arrival, QorError> {
    let dialog = HostDialog(app.clone());
    let (accounts, _aside) = state.vault.restore(&dialog, &phrase).await?;
    Ok(arrive(app, &state, accounts).await)
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

#[tauri::command]
fn vault_add_account(
    state: tauri::State<'_, AppState>,
    label: String,
) -> Result<AccountView, QorError> {
    state.vault.add_account(&label)
}

/// Re-export the recovery phrase, once the person approves a host dialog.
#[tauri::command]
async fn vault_export_phrase(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, QorError> {
    state.vault.export_phrase(&HostDialog(app)).await
}

/// Try QOR ID again with the open vault's first account, as opening it does
/// (ADR-016): for when the service was unreachable at launch.
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
    let sparks = state.chain.balance(&address).await?;
    Ok(Balance {
        address,
        sparks: sparks.to_string(),
        cgt: cgt::format_cgt(sparks),
        display: format!("{} {}", cgt::format_cgt_grouped(sparks), cgt::SYMBOL),
    })
}

/// Parse a human amount and report it back, so the UI can preview a send
/// without duplicating denomination logic in TypeScript.
#[tauri::command]
fn cgt_parse_amount(amount: String) -> Result<Balance, QorError> {
    let sparks = cgt::parse_cgt(&amount)?;
    Ok(Balance {
        address: String::new(),
        sparks: sparks.to_string(),
        cgt: cgt::format_cgt(sparks),
        display: format!("{} {}", cgt::format_cgt_grouped(sparks), cgt::SYMBOL),
    })
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
    let sparks = cgt::parse_cgt(&amount)?;
    let receipt = state
        .chain
        .transfer(&state.vault, &HostDialog(app), &from, &to, sparks)
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
    let store = content::TemporaryStore::in_data_dir(&state.data_dir);

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
    if let Err(e) = partners::remember(&state.data_dir, &receipt.to, label.as_deref()) {
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
    let sparks = cgt::parse_cgt(&price)?;
    state.chain.sale_preview(collection, item, sparks).await
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
    let sparks = cgt::parse_cgt(&price)?;
    state
        .chain
        .list(
            &state.vault,
            &HostDialog(app),
            &from,
            collection,
            item,
            sparks,
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

/// Buy a listed asset. `price_sparks` and `root` are what the buyer was shown:
/// if either has changed on chain, nobody is asked and nothing is sent.
#[tauri::command]
async fn drc369_buy(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    from: String,
    collection: u32,
    item: u32,
    price_sparks: String,
    root: String,
) -> Result<chain::sales::SaleReceipt, QorError> {
    let seen = price_sparks
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
                price_sparks: seen,
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
    listings::save(&state.data_dir, &draft)
}

/// Every draft on this machine, most recently changed first.
#[tauri::command]
async fn listing_drafts(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<listings::Listing>, QorError> {
    Ok(listings::list(&state.data_dir))
}

/// Throw a draft away.
#[tauri::command]
async fn listing_discard(
    state: tauri::State<'_, AppState>,
    collection: u32,
    item: u32,
) -> Result<(), QorError> {
    listings::discard(&state.data_dir, collection, item)
}

/// Who this machine has traded with, most recent first. Local, and never sent
/// anywhere (`partners.rs`).
#[tauri::command]
async fn trade_partners(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<partners::Partner>, QorError> {
    Ok(partners::list(&state.data_dir))
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
    state
        .identity
        .link_wallet(&state.vault, &HostDialog(app), &address)
        .await
}

#[tauri::command]
async fn qor_restore(state: tauri::State<'_, AppState>) -> Result<Session, QorError> {
    state.identity.restore().await
}

// ── The account's avatar (ADR-079) ────────────────────────────────────────────

/// The largest file offered for upload; QOR ID refuses more and cleans the rest.
const AVATAR_UPLOAD_LIMIT: u64 = 4 * 1024 * 1024;

fn avatar_cache(state: &AppState) -> (std::path::PathBuf, std::path::PathBuf) {
    (
        state.data_dir.join("avatar.bin"),
        state.data_dir.join("avatar.type"),
    )
}

/// Keep a copy of the account's avatar on this machine, so it shows offline. No avatar clears the copy.
async fn refresh_avatar_copy(state: &AppState, session: &Session) {
    let (bytes_path, type_path) = avatar_cache(state);
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
    let (bytes_path, type_path) = avatar_cache(&state);
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

#[tauri::command]
async fn qor_logout(state: tauri::State<'_, AppState>) -> Result<(), QorError> {
    state.identity.logout().await;
    Ok(())
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
                    cgt::format_cgt_grouped(request.amount_sparks),
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
    let paid = pay::PaidRequests::in_dir(&state.data_dir);
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
        // The vault's first account pays: the account the launcher signs in with (ADR-016).
        let from = match state.vault.status() {
            VaultStatus::Unlocked { accounts } => accounts
                .first()
                .map(|account| account.address.clone())
                .ok_or_else(|| QorError::PaymentRefused("the vault holds no account".into()))?,
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

            app.manage(AppState::new(data_dir).map_err(|e| e.to_string())?);

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
            vault_add_account,
            vault_export_phrase,
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
            qor_logout,
            qor_set_auth_endpoint,
            arqade_open,
            gates_report,
            gates_run_suite,
        ])
        .run(tauri::generate_context!())
        .expect("QOR Launcher failed to start");
}

/// Test support: a [`Confirm`] with a fixed answer that records every prompt.
#[cfg(test)]
pub(crate) mod testing {
    use super::{Answer, Confirm, Prompt};

    pub struct Scripted {
        answer: bool,
        pub asked: parking_lot::Mutex<Vec<Prompt>>,
    }

    impl Scripted {
        pub fn approving() -> Self {
            Self {
                answer: true,
                asked: parking_lot::Mutex::new(Vec::new()),
            }
        }

        pub fn declining() -> Self {
            Self {
                answer: false,
                asked: parking_lot::Mutex::new(Vec::new()),
            }
        }

        pub fn times_asked(&self) -> usize {
            self.asked.lock().len()
        }
    }

    impl Confirm for Scripted {
        fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a> {
            self.asked.lock().push(prompt.clone());
            let answer = self.answer;
            Box::pin(async move { answer })
        }
    }
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
}
