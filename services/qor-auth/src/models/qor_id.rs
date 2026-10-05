//! Qor ID specific models and utilities.
//!
//! A QOR ID is the account's username alone, unique on its own whatever its letter case (ADR-075).
//! Until 5 October 2026 it carried a Battle.net-style `#0001` discriminator; an old string in that
//! form is still read, and its number ignored.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A QOR ID: a username.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QorId {
    pub username: String,
}

impl QorId {
    /// Parse a QOR ID: `name`, or the retired `name#0001` form, whose number is ignored.
    pub fn parse(s: &str) -> Option<Self> {
        let name = match s.split_once('#') {
            Some((name, number))
                if !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()) =>
            {
                name
            }
            Some(_) => return None,
            None => s,
        };
        Self::is_valid_username(name).then(|| Self {
            username: name.to_lowercase(),
        })
    }

    /// Validate username format
    pub fn is_valid_username(username: &str) -> bool {
        // 3-20 characters, alphanumeric and underscores only
        let len = username.len();
        if !(3..=20).contains(&len) {
            return false;
        }

        username
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

impl fmt::Display for QorId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.username)
    }
}

impl Serialize for QorId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for QorId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        QorId::parse(&s).ok_or_else(|| serde::de::Error::custom("Invalid Qor ID format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_qor_id_is_the_username_alone() {
        let qor_id = QorId::parse("Alaustrup").unwrap();
        assert_eq!(qor_id.username, "alaustrup");
        assert_eq!(qor_id.to_string(), "alaustrup", "no #0001 is shown");
    }

    #[test]
    fn the_retired_number_is_read_and_ignored() {
        assert_eq!(
            QorId::parse("alaustrup#1337").unwrap().to_string(),
            "alaustrup"
        );
    }

    #[test]
    fn test_invalid_qor_id() {
        assert!(QorId::parse("ab").is_none());
        assert!(QorId::parse("user#").is_none());
        assert!(QorId::parse("user#-1").is_none());
        assert!(QorId::parse("user name").is_none());
    }

    #[test]
    fn test_username_validation() {
        assert!(QorId::is_valid_username("alaustrup"));
        assert!(QorId::is_valid_username("user_123"));
        assert!(!QorId::is_valid_username("ab")); // too short
        assert!(!QorId::is_valid_username("user name")); // space
        assert!(!QorId::is_valid_username("user@name")); // special char
    }
}
