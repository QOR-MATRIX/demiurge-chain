//! The launcher's update channel (L6.2, ADR-111).
//!
//! The host checks one fixed address for a signed manifest ([`CHANNEL`]): fifteen seconds after start and every six
//! hours while the launcher runs, unless the person turned that off in Settings. It tells the interface what it found
//! (the event [`EVENT`]) and installs only when the person asks ([`install`]).
//!
//! Tauri's updater verifies each download against the public key in `tauri.conf.json` before handing it back.
//! [`verify_release`] then checks it again, refusing a legacy signature, and requires the signed file to be the
//! announced version's installer. That is what the plugin's `requireSignedVersion` would check if the Tauri CLI that
//! builds this launcher recorded a version in its signatures; it records the file's name, which carries the version.
//!
//! The interface can ask to check and to install, and nothing else: it never chooses the address, the key or the file.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::Engine as _;
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::error::{QorError, QorResult};
use crate::AppState;

/// Where the signed manifest is: an asset of the pre-release `launcher-channel`, which holds nothing else and is
/// replaced at each release (ADR-111 decision 1). Also `tauri.conf.json`'s `plugins.updater.endpoints`.
pub const CHANNEL: &str =
    "https://github.com/QOR-MATRIX/demiurge-chain/releases/download/launcher-channel/latest.json";

/// The channel's public key, as `tauri.conf.json` carries it (`plugins.updater.pubkey`): the base64 of minisign's
/// public key file. Its private half is on the PC that builds releases, never here (`scripts/updater-key.ps1`). The host
/// reads the key from the configuration built into the launcher, the one the plugin verifies with, so the two checks
/// can never use different keys; a test holds the configuration to this one, so the key changes only on purpose.
pub const PUBKEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDJEODVGRjhGMUY4N0IxMjQKUldRa3NZY2ZqLytGTGNKejNtVGJyM1ZrSHlZamRkamZNbW82aFVRckdMU29uKzVORVc3MFpERmEK";

/// The public key the launcher was built with: its configuration's `plugins.updater.pubkey`.
fn configured_pubkey(app: &AppHandle) -> QorResult<String> {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|updater| updater.get("pubkey"))
        .and_then(|key| key.as_str())
        .map(str::to_string)
        .ok_or_else(|| {
            QorError::Update(
                "The launcher was built without an update key. Nothing was installed.".into(),
            )
        })
}

/// What the interface listens to: every change of [`Status`].
pub const EVENT: &str = "update-status";

/// How long after start the first check waits, so it never competes with the launcher opening.
const FIRST_CHECK: Duration = Duration::from_secs(15);
/// How often the launcher checks while it runs.
const EVERY: Duration = Duration::from_secs(6 * 60 * 60);

/// What the channel knows.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    /// Not checked since the launcher started.
    Unchecked,
    Checking,
    UpToDate,
    /// A newer version is ready to fetch, with its notes.
    Available {
        version: String,
        notes: Option<String>,
        date: Option<String>,
    },
    /// Being fetched, after the person asked.
    Downloading {
        version: String,
        received: u64,
        total: Option<u64>,
    },
    /// Fetched and verified: the launcher closes and the installer opens.
    Installing {
        version: String,
    },
    /// The last check or install did not succeed. The sentence is for the person.
    Failed {
        message: String,
    },
}

/// What the interface reads ([`update_status`]).
#[derive(Debug, Clone, Serialize)]
pub struct View {
    /// The running launcher's version.
    pub current: String,
    pub status: Status,
    /// Whether the launcher checks by itself.
    pub automatic: bool,
    /// When the last check finished, in seconds since 1970.
    pub checked_at: Option<u64>,
}

/// The channel's state, held by the host for the whole run.
pub struct Updates {
    status: RwLock<Status>,
    automatic: AtomicBool,
    checked_at: RwLock<Option<u64>>,
    /// The update the last check found, ready for [`install`].
    pending: Mutex<Option<Update>>,
    /// A check or an install is under way, so another waits for nothing and changes nothing.
    busy: AtomicBool,
}

impl Updates {
    pub fn new(automatic: bool) -> Self {
        Self {
            status: RwLock::new(Status::Unchecked),
            automatic: AtomicBool::new(automatic),
            checked_at: RwLock::new(None),
            pending: Mutex::new(None),
            busy: AtomicBool::new(false),
        }
    }

    pub fn automatic(&self) -> bool {
        self.automatic.load(Ordering::SeqCst)
    }

    pub fn set_automatic(&self, on: bool) {
        self.automatic.store(on, Ordering::SeqCst);
    }

    pub fn status(&self) -> Status {
        self.status.read().clone()
    }

    pub fn view(&self, current: &str) -> View {
        View {
            current: current.to_string(),
            status: self.status(),
            automatic: self.automatic(),
            checked_at: *self.checked_at.read(),
        }
    }

    fn set(&self, app: &AppHandle, status: Status) {
        *self.status.write() = status.clone();
        if let Err(e) = app.emit(EVENT, &status) {
            tracing::warn!(error = %e, "could not tell the interface about the update channel");
        }
    }

    /// Take the busy flag, or say what is under way.
    fn begin(&self) -> QorResult<Busy<'_>> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(QorError::Update(
                "The launcher is already checking for or installing an update.".into(),
            ));
        }
        Ok(Busy(&self.busy))
    }
}

/// Holds [`Updates::busy`] until dropped.
struct Busy<'a>(&'a AtomicBool);

impl Drop for Busy<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

/// The Windows installer the release script signs for `version` of `product`: Tauri's NSIS name.
pub fn installer_name(product: &str, version: &str) -> String {
    format!("{product}_{version}_x64-setup.exe")
}

/// Check a downloaded installer against the channel's key a second time: the signature must be minisign's current kind
/// (a legacy one is refused) and verify over `bytes`, and the file it was made for, which its trusted comment names
/// and its global signature covers, must be `expected_file`. So a manifest cannot pair a version number with another
/// release's installer, nor offer an installer under a name it was not signed with.
pub fn verify_release(
    bytes: &[u8],
    signature: &str,
    pubkey: &str,
    expected_file: &str,
) -> QorResult<()> {
    let refuse = |why: &str| {
        QorError::Update(format!(
            "The update was refused: {why}. Nothing was installed."
        ))
    };
    let text = |b64: &str| -> QorResult<String> {
        let raw = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|_| refuse("its signature is not readable"))?;
        String::from_utf8(raw).map_err(|_| refuse("its signature is not readable"))
    };
    let key = minisign_verify::PublicKey::decode(&text(pubkey)?)
        .map_err(|_| refuse("the launcher's key is not readable"))?;
    let signature = minisign_verify::Signature::decode(&text(signature)?)
        .map_err(|_| refuse("its signature is not readable"))?;
    key.verify(bytes, &signature, false)
        .map_err(|_| refuse("its signature does not match the launcher's key"))?;
    let signed_for = signature
        .trusted_comment()
        .split('\t')
        .find_map(|field| field.strip_prefix("file:"));
    if signed_for != Some(expected_file) {
        return Err(refuse(&format!(
            "it was signed as {}, not as {expected_file}",
            signed_for.unwrap_or("an unnamed file")
        )));
    }
    Ok(())
}

/// An endpoint given for testing (`QOR_UPDATE_ENDPOINT`), in place of [`CHANNEL`]. The plugin refuses anything but
/// HTTPS unless the build's configuration allows otherwise, which no release configuration does; and whatever it
/// fetches must still verify against the launcher's key.
fn endpoint_override() -> Option<tauri::Url> {
    let value = std::env::var("QOR_UPDATE_ENDPOINT").ok()?;
    match tauri::Url::parse(value.trim()) {
        Ok(url) => Some(url),
        Err(_) => {
            tracing::warn!("ignoring QOR_UPDATE_ENDPOINT: not a URL");
            None
        }
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// A sentence for the person from the plugin's error, which names no secret: an address, a status, a reason.
fn failed(what: &str, e: impl std::fmt::Display) -> Status {
    Status::Failed {
        message: format!("{what}: {e}"),
    }
}

/// Ask the channel whether a newer version is ready. Changes nothing installed.
pub async fn check(app: &AppHandle, updates: &Updates) -> QorResult<Status> {
    let _busy = updates.begin()?;
    updates.set(app, Status::Checking);
    let result = async {
        let mut builder = app.updater_builder();
        if let Some(endpoint) = endpoint_override() {
            builder = builder.endpoints(vec![endpoint])?;
        }
        builder.build()?.check().await
    }
    .await;
    let status = match result {
        Ok(Some(update)) => {
            let status = Status::Available {
                version: update.version.clone(),
                notes: update.body.clone(),
                date: update.date.map(|d| d.to_string()),
            };
            tracing::info!(version = %update.version, "an update is ready");
            *updates.pending.lock() = Some(update);
            status
        }
        Ok(None) => {
            *updates.pending.lock() = None;
            Status::UpToDate
        }
        Err(e) => {
            tracing::warn!(error = %e, "could not check for an update");
            failed("The launcher could not check for updates", e)
        }
    };
    *updates.checked_at.write() = Some(now());
    updates.set(app, status.clone());
    Ok(status)
}

/// Fetch the update the last check found, verify it twice, and run its installer. On Windows the installer takes over:
/// the launcher closes, Windows asks once for permission, and the new version opens.
pub async fn install(app: &AppHandle, updates: &Updates) -> QorResult<()> {
    let _busy = updates.begin()?;
    let update =
        updates.pending.lock().clone().ok_or_else(|| {
            QorError::Update("No update is ready. Check for updates first.".into())
        })?;
    let version = update.version.clone();
    updates.set(
        app,
        Status::Downloading {
            version: version.clone(),
            received: 0,
            total: None,
        },
    );

    let mut received: u64 = 0;
    let mut told: u64 = 0;
    let progress = app.clone();
    let result = update
        .download(
            |chunk, total| {
                received += chunk as u64;
                // Tell the interface every 256 KiB, not every chunk.
                if received - told >= 256 * 1024 || Some(received) == total {
                    told = received;
                    let status = Status::Downloading {
                        version: version.clone(),
                        received,
                        total,
                    };
                    if let Some(state) = progress.try_state::<AppState>() {
                        state.updates.set(&progress, status);
                    }
                }
            },
            || {},
        )
        .await;
    let bytes = match result {
        Ok(bytes) => bytes,
        Err(e) => {
            tracing::warn!(error = %e, "an update download was refused or failed");
            let status = failed(
                "The update could not be fetched or did not verify, and nothing was installed",
                e,
            );
            updates.set(app, status);
            return Err(QorError::Update(
                "The update could not be fetched or verified. Nothing was installed.".into(),
            ));
        }
    };

    let expected = installer_name(&app.package_info().name, &version);
    let checked = configured_pubkey(app)
        .and_then(|pubkey| verify_release(&bytes, &update.signature, &pubkey, &expected));
    if let Err(e) = checked {
        tracing::warn!(error = %e, "an update failed the launcher's own check");
        updates.set(
            app,
            Status::Failed {
                message: e.to_string(),
            },
        );
        return Err(e);
    }

    tracing::info!(%version, "installing an update");
    updates.set(
        app,
        Status::Installing {
            version: version.clone(),
        },
    );
    update.install(bytes).map_err(|e| {
        let error = QorError::Update(format!(
            "The installer could not be started: {e}. Nothing was installed."
        ));
        updates.set(
            app,
            Status::Failed {
                message: error.to_string(),
            },
        );
        error
    })
}

/// Check now and then, for the whole run, while the person leaves it on.
pub fn watch(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(FIRST_CHECK).await;
        loop {
            let state = app.state::<AppState>();
            if state.updates.automatic() {
                // A check already under way is enough; a failure is shown, not retried until the next turn.
                let _ = check(&app, &state.updates).await;
            }
            tokio::time::sleep(EVERY).await;
        }
    });
}

#[tauri::command]
pub fn update_status(app: AppHandle, state: tauri::State<'_, AppState>) -> View {
    state.updates.view(&app.package_info().version.to_string())
}

#[tauri::command]
pub async fn update_check(app: AppHandle, state: tauri::State<'_, AppState>) -> QorResult<View> {
    check(&app, &state.updates).await?;
    Ok(state.updates.view(&app.package_info().version.to_string()))
}

/// Install the update the last check found. Only ever runs because the person pressed *Update and restart*.
#[tauri::command]
pub async fn update_install(app: AppHandle, state: tauri::State<'_, AppState>) -> QorResult<()> {
    install(&app, &state.updates).await
}

/// Turn the launcher's own checks on or off, kept in its settings.
#[tauri::command]
pub fn update_set_automatic(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
    on: bool,
) -> QorResult<View> {
    state.updates.set_automatic(on);
    state.persist()?;
    Ok(state.updates.view(&app.package_info().version.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    // A throwaway key made for these tests with `tauri signer generate`; its private half was never kept. The installer
    // is a few bytes named as the release script names one, and signed with `tauri signer sign`.
    const TEST_KEY: &str = include_str!("../tests/fixtures/update/test.key.pub.b64");
    const SIGNED: &[u8] =
        include_bytes!("../tests/fixtures/update/QOR Launcher_9.9.9_x64-setup.exe");
    const SIGNATURE: &str =
        include_str!("../tests/fixtures/update/QOR Launcher_9.9.9_x64-setup.exe.sig");

    #[test]
    fn a_release_signed_with_the_key_for_its_own_name_is_accepted() {
        verify_release(
            SIGNED,
            SIGNATURE,
            TEST_KEY,
            &installer_name("QOR Launcher", "9.9.9"),
        )
        .unwrap();
    }

    #[test]
    fn a_changed_byte_is_refused() {
        let mut tampered = SIGNED.to_vec();
        tampered[0] ^= 1;
        let error = verify_release(
            &tampered,
            SIGNATURE,
            TEST_KEY,
            &installer_name("QOR Launcher", "9.9.9"),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("does not match the launcher's key"),
            "{error}"
        );
    }

    #[test]
    fn another_versions_installer_is_refused_whatever_the_manifest_says() {
        // A manifest announcing 9.9.10 with 9.9.9's installer and signature: the signature is good, the name is not.
        let error = verify_release(
            SIGNED,
            SIGNATURE,
            TEST_KEY,
            &installer_name("QOR Launcher", "9.9.10"),
        )
        .unwrap_err()
        .to_string();
        assert!(
            error.contains("signed as QOR Launcher_9.9.9_x64-setup.exe"),
            "{error}"
        );
    }

    #[test]
    fn a_signature_from_another_key_is_refused() {
        // The channel's real key did not sign the fixture.
        let error = verify_release(
            SIGNED,
            SIGNATURE,
            PUBKEY,
            &installer_name("QOR Launcher", "9.9.9"),
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("does not match"), "{error}");
    }

    #[test]
    fn nothing_unreadable_is_taken_for_a_signature() {
        for bad in ["", "not base64!", "aGVsbG8="] {
            assert!(
                verify_release(SIGNED, bad, TEST_KEY, "QOR Launcher_9.9.9_x64-setup.exe").is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn the_configuration_names_the_channel_and_its_key_and_allows_nothing_dangerous() {
        let base: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.conf.json")).unwrap();
        let updater = &base["plugins"]["updater"];
        assert_eq!(updater["pubkey"], PUBKEY);
        assert_eq!(updater["endpoints"], serde_json::json!([CHANNEL]));
        assert!(CHANNEL.starts_with("https://"));
        assert_eq!(updater["windows"]["installMode"], "passive");
        let release: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.release.conf.json")).unwrap();
        let bundle: serde_json::Value =
            serde_json::from_str(include_str!("../tauri.bundle.conf.json")).unwrap();
        for (name, config) in [
            ("tauri.conf.json", &base),
            ("tauri.release.conf.json", &release),
            ("tauri.bundle.conf.json", &bundle),
        ] {
            let text = config.to_string();
            for flag in ["dangerous", "allowDowngrades", "allow-downgrades"] {
                assert!(!text.contains(flag), "{name} sets {flag}");
            }
        }
        // A release build makes the signed installer; an ordinary build needs no key.
        assert_eq!(release["bundle"]["createUpdaterArtifacts"], true);
        assert!(base["bundle"].get("createUpdaterArtifacts").is_none());
    }

    #[test]
    fn a_new_channel_checks_by_itself_unless_told_not_to() {
        let updates = Updates::new(true);
        assert!(updates.automatic());
        assert_eq!(updates.status(), Status::Unchecked);
        updates.set_automatic(false);
        assert!(!updates.automatic());
        let view = updates.view("0.1.15");
        assert_eq!(view.current, "0.1.15");
        assert!(!view.automatic);
        assert_eq!(view.checked_at, None);
    }

    #[test]
    fn only_one_check_or_install_at_a_time() {
        let updates = Updates::new(true);
        let first = updates.begin().unwrap();
        assert!(updates.begin().is_err());
        drop(first);
        assert!(updates.begin().is_ok());
    }
}
