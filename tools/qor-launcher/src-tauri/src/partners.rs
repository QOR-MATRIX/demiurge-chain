//! Who this machine has traded with (L4.5).
//!
//! # Local, and only local
//!
//! There is no indexer (ADR-028), so nothing can ask the chain "who have I
//! traded with?" — a Substrate node answers questions about state, not about a
//! history. This file is the launcher's own memory of the trades it sent: an
//! address, what to call it, when it was last used and how often. It is written
//! only after a trade the chain finalised, so it records what happened rather
//! than what was attempted.
//!
//! **It never leaves this machine.** It is not published, not synchronised and
//! not sent to QOR ID; a person who wants their trading history private need
//! only delete the file, and the launcher rebuilds it from nothing.
//!
//! When the directory lookup exists, `label` holds the QOR ID a person typed,
//! so the list reads as names rather than addresses. Until then it is empty and
//! the interface shortens the address itself.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{QorError, QorResult};

/// The file, inside the launcher's data directory.
const FILE: &str = "trade-partners.json";

/// How many are kept. Enough that the people someone actually trades with are
/// one click away, few enough that the list stays a list.
const KEPT: usize = 12;

/// One account this machine has sent assets to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Partner {
    /// SS58, as this chain writes it.
    pub address: String,
    /// The QOR ID this address was reached by, when it was reached by one.
    #[serde(default)]
    pub label: Option<String>,
    /// Unix seconds, from the machine's clock: this is a convenience, not a
    /// record of anything the chain agreed to.
    pub last_traded: u64,
    /// How many trades have been sent to it.
    pub trades: u32,
}

fn path(data_dir: &Path) -> PathBuf {
    data_dir.join(FILE)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or_default()
}

/// Everyone this machine has traded with, most recent first.
///
/// A file that cannot be read is an empty list, not an error: a missing or
/// corrupt convenience must never stop a trade.
pub fn list(data_dir: &Path) -> Vec<Partner> {
    let Ok(bytes) = std::fs::read(path(data_dir)) else {
        return Vec::new();
    };
    let mut partners: Vec<Partner> = serde_json::from_slice(&bytes).unwrap_or_default();
    partners.sort_by_key(|one| std::cmp::Reverse(one.last_traded));
    partners.truncate(KEPT);
    partners
}

/// Record a finalised trade. Called only after the chain has reported one.
pub fn remember(data_dir: &Path, address: &str, label: Option<&str>) -> QorResult<()> {
    let mut partners = list(data_dir);
    let partner = match partners.iter().position(|one| one.address == address) {
        Some(at) => {
            let mut known = partners.remove(at);
            known.last_traded = now();
            known.trades = known.trades.saturating_add(1);
            // A name learnt later is kept; a trade sent to a bare address does
            // not erase the name an earlier one knew.
            if label.is_some() {
                known.label = label.map(str::to_string);
            }
            known
        }
        None => Partner {
            address: address.to_string(),
            label: label.map(str::to_string),
            last_traded: now(),
            trades: 1,
        },
    };
    // At the front before the stable sort: the clock counts whole seconds, so a
    // partner traded with twice in the second of another's trade must still
    // come first, not keep its older place in a tie.
    partners.insert(0, partner);
    partners.sort_by_key(|one| std::cmp::Reverse(one.last_traded));
    partners.truncate(KEPT);

    let json = serde_json::to_vec_pretty(&partners)
        .map_err(|e| QorError::Internal(format!("cannot write the trade partners: {e}")))?;
    std::fs::write(path(data_dir), json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("qor-partners-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a directory to test in");
        dir
    }

    #[test]
    fn nothing_is_remembered_until_a_trade_is() {
        let dir = dir("empty");
        assert!(list(&dir).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_partner_is_counted_once_and_kept_most_recent_first() {
        let dir = dir("counted");
        remember(&dir, "5Alice", None).unwrap();
        remember(&dir, "5Bob", Some("bob")).unwrap();
        remember(&dir, "5Alice", None).unwrap();

        let partners = list(&dir);
        assert_eq!(partners.len(), 2, "{partners:?}");
        let alice = partners
            .iter()
            .find(|one| one.address == "5Alice")
            .expect("alice");
        assert_eq!(alice.trades, 2, "the same address is one partner");
        assert_eq!(
            partners[0].address, "5Alice",
            "the most recent is first, whatever the order they arrived in"
        );

        // A name learnt once survives a later trade that knew no name.
        remember(&dir, "5Bob", None).unwrap();
        let bob = list(&dir)
            .into_iter()
            .find(|one| one.address == "5Bob")
            .expect("bob");
        assert_eq!(bob.label.as_deref(), Some("bob"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// The clock counts whole seconds. Bob traded with this second and Alice
    /// traded with again in it tie; Alice, the latest, comes first. Before,
    /// she kept her older place, and the test above failed whenever the second
    /// turned between its first two trades.
    #[test]
    fn a_partner_traded_with_again_in_the_same_second_comes_first() {
        let dir = dir("tie");
        let earlier = Partner {
            address: "5Alice".into(),
            label: None,
            last_traded: 1,
            trades: 1,
        };
        let this_second = Partner {
            address: "5Bob".into(),
            label: None,
            last_traded: now(),
            trades: 1,
        };
        std::fs::write(
            path(&dir),
            serde_json::to_vec(&[this_second, earlier]).unwrap(),
        )
        .unwrap();
        remember(&dir, "5Alice", None).unwrap();
        assert_eq!(list(&dir)[0].address, "5Alice");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_list_is_bounded_and_a_broken_file_is_no_list_at_all() {
        let dir = dir("bounded");
        for i in 0..KEPT + 5 {
            remember(&dir, &format!("5Account{i}"), None).unwrap();
        }
        assert_eq!(list(&dir).len(), KEPT);

        std::fs::write(path(&dir), b"not json").unwrap();
        assert!(
            list(&dir).is_empty(),
            "a convenience that cannot be read is empty, never an error"
        );
        // And it recovers: the next trade writes a fresh list.
        remember(&dir, "5Fresh", None).unwrap();
        assert_eq!(list(&dir).len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
