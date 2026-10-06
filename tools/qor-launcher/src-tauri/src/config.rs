//! Persisted launcher settings.
//!
//! # Why this exists
//!
//! Endpoints were originally overridable only through environment variables.
//! That is fine for a developer starting the app from a shell and useless for
//! everyone else: a normal launch goes to the compiled-in defaults, and if those
//! are unreachable the app is stuck at the Gate with no way to change them,
//! because Settings lives *behind* sign-in.
//!
//! That is exactly what happened. `rpc.demiurge.cloud` and `demiurge.cloud` are
//! both currently unreachable, so every ordinary launch produced an app that
//! could not connect and offered no way to say where to connect instead.
//!
//! Settings are therefore written to disk next to the vault, changeable from the
//! Gate, and loaded before anything tries to reach the network.
//!
//! (That was September 2026. Since ADR-056 nothing waits on sign-in, so Settings
//! is reachable without it too, and the defaults are the live devnet and QOR ID.)
//!
//! # Precedence
//!
//! 1. `QOR_RPC_URL` / `QOR_AUTH_URL` environment variables, when set. These win
//!    for the session and are deliberately **not** persisted, so a one-off
//!    override cannot quietly become permanent.
//! 2. The stored settings file.
//! 3. The compiled-in defaults.
//!
//! # The two endpoints are not the same kind of address
//!
//! The chain endpoint is a WebSocket address, `ws://` or `wss://` (ADR-040);
//! QOR ID's is `http://` or `https://`. A chain endpoint stored by a launcher
//! from before ADR-040 is an `http://` one naming the same node on the same
//! port, so it is upgraded rather than refused.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{QorError, QorResult};

const FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub rpc_url: String,
    pub auth_url: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            rpc_url: crate::chain::DEFAULT_RPC.to_string(),
            auth_url: crate::identity::DEFAULT_AUTH.to_string(),
        }
    }
}

impl Settings {
    /// Load from disk, then apply any environment overrides.
    ///
    /// A corrupt or unreadable file falls back to defaults rather than failing
    /// the launch: bad settings should never lock someone out of their vault.
    pub fn load(dir: &Path) -> Self {
        let mut settings = std::fs::read_to_string(dir.join(FILE))
            .ok()
            .and_then(|raw| serde_json::from_str::<Settings>(&raw).ok())
            .unwrap_or_default();

        if let Some(rpc) = env_url("QOR_RPC_URL") {
            settings.rpc_url = rpc;
        }
        if let Some(auth) = env_url("QOR_AUTH_URL") {
            settings.auth_url = auth;
        }

        // A stored or overridden chain endpoint from before ADR-040 names the
        // same node; upgrade it. Anything that is not an endpoint at all falls
        // back to the default, for the same reason a corrupt file does.
        settings.rpc_url = match crate::chain::normalise_endpoint(&settings.rpc_url) {
            Ok(endpoint) => endpoint,
            Err(_) => {
                tracing::warn!(
                    rpc = %settings.rpc_url,
                    "stored chain endpoint is not a node address; using the default"
                );
                crate::chain::DEFAULT_RPC.to_string()
            }
        };

        if !is_http_url(&settings.auth_url) {
            tracing::warn!(
                auth = %settings.auth_url,
                "stored identity endpoint is not an http(s) URL; using the default"
            );
            settings.auth_url = crate::identity::DEFAULT_AUTH.to_string();
        }

        settings
    }

    pub fn save(&self, dir: &Path) -> QorResult<()> {
        std::fs::create_dir_all(dir)?;
        let json = serde_json::to_vec_pretty(self)
            .map_err(|e| QorError::Internal(format!("cannot serialise settings: {e}")))?;
        std::fs::write(dir.join(FILE), json)?;
        Ok(())
    }
}

/// Read a URL from the environment. What counts as one is checked afterwards,
/// by whichever endpoint it is for, so a stray value cannot silently redirect
/// the app somewhere odd.
fn env_url(variable: &str) -> Option<String> {
    let value = std::env::var(variable).ok()?;
    let trimmed = value.trim();

    if trimmed.is_empty() {
        tracing::warn!(%variable, "ignoring override: empty");
        return None;
    }

    Some(trimmed.to_string())
}

pub fn is_http_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

/// Where settings and the vault live.
pub fn data_dir_or_default(dir: PathBuf) -> PathBuf {
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "qor-settings-{}-{:?}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn defaults_when_no_file_exists() {
        let dir = scratch();
        let settings = Settings::load(&dir);

        assert_eq!(settings.rpc_url, crate::chain::DEFAULT_RPC);
        assert_eq!(settings.auth_url, crate::identity::DEFAULT_AUTH);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn round_trips_through_disk() {
        let dir = scratch();

        let saved = Settings {
            rpc_url: crate::chain::LOCAL_RPC.into(),
            auth_url: "http://127.0.0.1:8080/api/v1".into(),
        };
        saved.save(&dir).unwrap();

        let loaded = Settings::load(&dir);
        assert_eq!(loaded.rpc_url, saved.rpc_url);
        assert_eq!(loaded.auth_url, saved.auth_url);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A launcher that stored `http://127.0.0.1:9944` before ADR-040 points at
    /// the same node on the same port. Upgrading it keeps that launcher
    /// working; refusing it would silently send it to an endpoint that does not
    /// answer.
    #[test]
    fn a_chain_endpoint_from_before_adr_040_is_upgraded() {
        let dir = scratch();

        Settings {
            rpc_url: "http://127.0.0.1:9944".into(),
            auth_url: "http://127.0.0.1:8080/api/v1".into(),
        }
        .save(&dir)
        .unwrap();

        assert_eq!(Settings::load(&dir).rpc_url, crate::chain::LOCAL_RPC);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Neither endpoint may be something that is not an address at all.
    #[test]
    fn nonsense_endpoints_fall_back_to_the_defaults() {
        let dir = scratch();

        Settings {
            rpc_url: "file:///etc/passwd".into(),
            auth_url: "ws://localhost".into(),
        }
        .save(&dir)
        .unwrap();

        let loaded = Settings::load(&dir);
        assert_eq!(loaded.rpc_url, crate::chain::DEFAULT_RPC);
        assert_eq!(loaded.auth_url, crate::identity::DEFAULT_AUTH);

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A damaged settings file must not stop the launcher from opening.
    #[test]
    fn corrupt_file_falls_back_to_defaults() {
        let dir = scratch();
        std::fs::write(dir.join(FILE), b"{ this is not json").unwrap();

        let settings = Settings::load(&dir);
        assert_eq!(settings.rpc_url, crate::chain::DEFAULT_RPC);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_non_http_urls() {
        assert!(is_http_url("http://127.0.0.1:9944"));
        assert!(is_http_url("https://rpc.demiurge.cloud"));

        for bad in ["ws://localhost", "file:///etc/passwd", "127.0.0.1:9944", ""] {
            assert!(!is_http_url(bad), "{bad:?} should be rejected");
        }
    }
}
