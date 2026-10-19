//! The vault's key in the operating system's keychain (ADR-056).
//!
//! `vault.qor` holds the recovery phrase sealed (Argon2id, then
//! XChaCha20-Poly1305) under a random 32-byte key. That key lives in the
//! keychain of the person's operating-system account — Windows Credential
//! Manager, the macOS Keychain, the Secret Service — beside QOR ID's tokens,
//! and is read with no prompt. The vault therefore opens whenever the person is
//! signed in to their computer, and a copied `vault.qor` opens nothing without
//! their account's keychain.
//!
//! On Android there is no [`system`] keychain here. The `keyring` crate has no
//! Android store and would fall back to one in memory, which would lose a
//! wallet's key when the app closed. The Android shell supplies a [`KeyStore`]
//! of its own, wrapped by Android's Keystore (ADR-104 decision 4, DIRECTION L13.3).

#[cfg(not(target_os = "android"))]
use std::sync::Arc;

#[cfg(not(target_os = "android"))]
use crate::error::QorError;
use crate::error::QorResult;

/// The keychain service every launcher entry sits under.
#[cfg(not(target_os = "android"))]
const SERVICE: &str = "cloud.demiurge.qor-launcher";

/// Where a vault's key is kept, by the `key` id in its file.
pub fn entry_name(id: &str) -> String {
    format!("vault-key:{id}")
}

/// A place to keep a secret for this operating-system account.
pub trait KeyStore: Send + Sync {
    fn put(&self, name: &str, secret: &str) -> QorResult<()>;
    fn get(&self, name: &str) -> QorResult<String>;
    /// Remove a secret. One that is not there is already removed.
    fn delete(&self, name: &str) -> QorResult<()>;
}

/// The operating system's keychain.
#[cfg(not(target_os = "android"))]
pub fn system() -> Arc<dyn KeyStore> {
    Arc::new(OsKeychain)
}

#[cfg(not(target_os = "android"))]
struct OsKeychain;

#[cfg(not(target_os = "android"))]
fn entry(name: &str) -> QorResult<keyring::Entry> {
    keyring::Entry::new(SERVICE, name).map_err(|e| QorError::Keychain(e.to_string()))
}

#[cfg(not(target_os = "android"))]
impl KeyStore for OsKeychain {
    fn put(&self, name: &str, secret: &str) -> QorResult<()> {
        entry(name)?
            .set_password(secret)
            .map_err(|e| QorError::Keychain(e.to_string()))
    }

    fn get(&self, name: &str) -> QorResult<String> {
        entry(name)?.get_password().map_err(|e| match e {
            keyring::Error::NoEntry => QorError::Keychain(
                "this computer account no longer holds the vault's key. Restore the vault from your \
                 recovery phrase"
                    .into(),
            ),
            other => QorError::Keychain(other.to_string()),
        })
    }

    fn delete(&self, name: &str) -> QorResult<()> {
        match entry(name)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(other) => Err(QorError::Keychain(other.to_string())),
        }
    }
}

/// Test support: a keychain in memory.
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use super::KeyStore;
    use crate::error::{QorError, QorResult};
    use parking_lot::Mutex;
    use std::collections::HashMap;

    #[derive(Default)]
    pub struct MemoryKeychain(pub Mutex<HashMap<String, String>>);

    impl KeyStore for MemoryKeychain {
        fn put(&self, name: &str, secret: &str) -> QorResult<()> {
            self.0.lock().insert(name.into(), secret.into());
            Ok(())
        }
        fn get(&self, name: &str) -> QorResult<String> {
            self.0
                .lock()
                .get(name)
                .cloned()
                .ok_or_else(|| QorError::Keychain("no such entry".into()))
        }
        fn delete(&self, name: &str) -> QorResult<()> {
            self.0.lock().remove(name);
            Ok(())
        }
    }
}

/// Against the real keychain. Ignored by default: it writes a throwaway entry
/// to this computer account's keychain, reads it back and deletes it.
#[cfg(all(test, not(target_os = "android")))]
mod live {
    use super::{entry, entry_name, system};

    #[test]
    #[ignore = "writes a throwaway entry to this computer's real keychain"]
    fn the_os_keychain_keeps_a_secret_and_gives_it_back() {
        let name = entry_name(&format!("selftest-{}", std::process::id()));
        let keys = system();
        keys.put(&name, "a throwaway secret").unwrap();
        assert_eq!(keys.get(&name).unwrap(), "a throwaway secret");
        entry(&name).unwrap().delete_credential().unwrap();
        assert_eq!(keys.get(&name).unwrap_err().kind(), "keychain");
    }
}
