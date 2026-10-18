//! Which QOR ID each of this PC's wallet accounts belongs to, as far as this launcher has seen, and the account last in
//! use (launcher 0.1.12).
//!
//! The money belongs to an account's key, and a QOR ID is a name bound to one key (QOR ID keeps one QOR ID per chain
//! account). On a PC shared by two people, or one person with two QOR IDs, the launcher must say whose account is on
//! screen and sign in as the right one at launch. QOR ID knows the binding; this file remembers what the launcher saw of
//! it, so the Vault can label accounts while signed out, and launch can sign in with the account last used rather than
//! always the first.
//!
//! Kept beside the vault, on this PC only, sent nowhere. It holds addresses and names, both public; nothing secret.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::{QorError, QorResult};
use crate::vault::canonical_address;

const FILE: &str = "account-owners.json";

/// What this PC knows of its accounts' QOR IDs.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Owners {
    /// Each account (SS58) the launcher has seen a QOR ID signed in as, and that QOR ID's name.
    pub owners: BTreeMap<String, String>,
    /// The account last in use, which launch signs in with.
    pub last: Option<String>,
}

pub fn read(dir: &Path) -> Owners {
    std::fs::read(dir.join(FILE))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

fn write(dir: &Path, owners: &Owners) -> QorResult<()> {
    std::fs::create_dir_all(dir).map_err(|e| QorError::Io(e.to_string()))?;
    let bytes = serde_json::to_vec_pretty(owners).map_err(|e| QorError::Internal(e.to_string()))?;
    std::fs::write(dir.join(FILE), bytes).map_err(|e| QorError::Io(e.to_string()))
}

/// Note that `qor_id` is bound to `address`, which is now the account in use. Any other account this PC had noted for
/// the same QOR ID is forgotten: a QOR ID has one account.
pub fn remember(dir: &Path, address: &str, qor_id: &str) -> QorResult<()> {
    let address = canonical_address(address)?;
    let mut owners = read(dir);
    owners
        .owners
        .retain(|a, name| name != qor_id || *a == address);
    owners.owners.insert(address.clone(), qor_id.to_string());
    owners.last = Some(address);
    write(dir, &owners)
}

/// Note the account now in use.
pub fn set_last(dir: &Path, address: &str) -> QorResult<()> {
    let address = canonical_address(address)?;
    let mut owners = read(dir);
    if owners.last.as_deref() == Some(address.as_str()) {
        return Ok(());
    }
    owners.last = Some(address);
    write(dir, &owners)
}

/// The account launch signs in with: the one last in use if this vault still holds it, else the first.
pub fn launch_account<'a>(owners: &Owners, accounts: &'a [String]) -> Option<&'a String> {
    owners
        .last
        .as_ref()
        .and_then(|last| accounts.iter().find(|a| *a == last))
        .or_else(|| accounts.first())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("qor-owners-{}", uuid::Uuid::new_v4().simple()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    /// Two QOR IDs on one PC: each account keeps its own name, a QOR ID that moves to another account is forgotten
    /// on the old one, and launch signs in with the account last in use while the vault holds it.
    #[test]
    fn each_account_keeps_its_own_qor_id_and_launch_uses_the_last() {
        let d = dir();
        let first = crate::vault::derive::address_of(&[1; 32]);
        let second = crate::vault::derive::address_of(&[2; 32]);
        assert_eq!(read(&d), Owners::default());

        remember(&d, &first, "godmode").unwrap();
        remember(&d, &second, "yaldobaoth").unwrap();
        let owners = read(&d);
        assert_eq!(
            owners.owners.get(&first).map(String::as_str),
            Some("godmode")
        );
        assert_eq!(
            owners.owners.get(&second).map(String::as_str),
            Some("yaldobaoth")
        );
        assert_eq!(owners.last.as_deref(), Some(second.as_str()));

        let accounts = vec![first.clone(), second.clone()];
        assert_eq!(launch_account(&owners, &accounts), Some(&second));
        set_last(&d, &first).unwrap();
        assert_eq!(launch_account(&read(&d), &accounts), Some(&first));
        assert_eq!(
            launch_account(&read(&d), std::slice::from_ref(&second)),
            Some(&second),
            "an account the vault no longer holds is not signed in with"
        );

        // A QOR ID has one account: bound elsewhere, it is forgotten here.
        remember(&d, &first, "yaldobaoth").unwrap();
        let owners = read(&d);
        assert_eq!(
            owners.owners.get(&first).map(String::as_str),
            Some("yaldobaoth")
        );
        assert_eq!(owners.owners.get(&second), None);
        assert!(remember(&d, "not an address", "x").is_err());
        let _ = std::fs::remove_dir_all(&d);
    }
}
