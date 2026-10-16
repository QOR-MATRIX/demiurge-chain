//! Message bodies sealed at rest (ADR-091 decision 4).
//!
//! XChaCha20-Poly1305 under the service's message key of the newest version, with a random 24-byte nonce, and the room
//! and the message's sequence as associated data, so a sealed body opens only in the place it was written. Keys come
//! from the environment, never the database; every version given stays readable, so a key is rotated by adding a newer
//! one and re-sealing at leisure. Nothing here is logged, and `Debug` names versions only.

use std::collections::BTreeMap;

use base64::Engine as _;
use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore as _;
use uuid::Uuid;

/// The keys, by version.
#[derive(Clone)]
pub struct Sealer {
    keys: BTreeMap<i16, XChaCha20Poly1305>,
}

impl std::fmt::Debug for Sealer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sealer")
            .field("versions", &self.keys.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// A body as it is stored.
pub struct Sealed {
    pub key_version: i16,
    pub nonce: Vec<u8>,
    pub sealed: Vec<u8>,
}

fn associated(room: Uuid, seq: i64) -> Vec<u8> {
    let mut data = Vec::with_capacity(24);
    data.extend_from_slice(room.as_bytes());
    data.extend_from_slice(&seq.to_be_bytes());
    data
}

impl Sealer {
    /// Read `"1:<base64 of 32 bytes>,2:…"`. Refuses an empty list, a version given twice, a version below 1 and a key
    /// that is not exactly 32 bytes, saying which version, never the key.
    pub fn from_spec(spec: &str) -> anyhow::Result<Self> {
        let mut keys = BTreeMap::new();
        for entry in spec.split(',').map(str::trim).filter(|e| !e.is_empty()) {
            let (version, key) = entry
                .split_once(':')
                .ok_or_else(|| anyhow::anyhow!("a message key is written version:base64"))?;
            let version: i16 = version
                .trim()
                .parse()
                .map_err(|_| anyhow::anyhow!("a message key's version is a number"))?;
            anyhow::ensure!(version >= 1, "message key versions start at 1");
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(key.trim())
                .map_err(|_| anyhow::anyhow!("message key {version} is not base64"))?;
            anyhow::ensure!(bytes.len() == 32, "message key {version} is not 32 bytes");
            let cipher = XChaCha20Poly1305::new_from_slice(&bytes)
                .map_err(|_| anyhow::anyhow!("message key {version} is not usable"))?;
            anyhow::ensure!(
                keys.insert(version, cipher).is_none(),
                "message key {version} is given twice"
            );
        }
        anyhow::ensure!(!keys.is_empty(), "no message key is configured");
        Ok(Self { keys })
    }

    /// A sealer with one random key, for tests.
    #[cfg(test)]
    pub fn random() -> Self {
        let mut key = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut key);
        Self::from_spec(&format!(
            "1:{}",
            base64::engine::general_purpose::STANDARD.encode(key)
        ))
        .expect("a key")
    }

    fn newest(&self) -> (i16, &XChaCha20Poly1305) {
        let (version, cipher) = self.keys.iter().next_back().expect("at least one key");
        (*version, cipher)
    }

    pub fn seal(&self, room: Uuid, seq: i64, body: &str) -> anyhow::Result<Sealed> {
        let (key_version, cipher) = self.newest();
        let mut nonce = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut nonce);
        let aad = associated(room, seq);
        let sealed = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: body.as_bytes(),
                    aad: &aad,
                },
            )
            .map_err(|_| anyhow::anyhow!("a message could not be sealed"))?;
        Ok(Sealed {
            key_version,
            nonce: nonce.to_vec(),
            sealed,
        })
    }

    pub fn open(
        &self,
        room: Uuid,
        seq: i64,
        key_version: i16,
        nonce: &[u8],
        sealed: &[u8],
    ) -> anyhow::Result<String> {
        let cipher = self
            .keys
            .get(&key_version)
            .ok_or_else(|| anyhow::anyhow!("message key {key_version} is not configured"))?;
        anyhow::ensure!(nonce.len() == 24, "a stored nonce is not 24 bytes");
        let aad = associated(room, seq);
        let body = cipher
            .decrypt(
                XNonce::from_slice(nonce),
                Payload {
                    msg: sealed,
                    aad: &aad,
                },
            )
            .map_err(|_| anyhow::anyhow!("a message did not open with its key"))?;
        String::from_utf8(body).map_err(|_| anyhow::anyhow!("an opened message is not UTF-8"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> String {
        base64::engine::general_purpose::STANDARD.encode([byte; 32])
    }

    #[test]
    fn a_body_opens_only_where_it_was_written() {
        let sealer = Sealer::random();
        let room = Uuid::new_v4();
        let s = sealer.seal(room, 7, "hello, world").expect("sealed");
        assert!(
            !s.sealed.windows(5).any(|w| w == b"hello"),
            "sealed, not stored plain"
        );
        assert_eq!(
            sealer
                .open(room, 7, s.key_version, &s.nonce, &s.sealed)
                .expect("opens"),
            "hello, world"
        );
        assert!(
            sealer
                .open(room, 8, s.key_version, &s.nonce, &s.sealed)
                .is_err(),
            "moved in its room"
        );
        assert!(
            sealer
                .open(Uuid::new_v4(), 7, s.key_version, &s.nonce, &s.sealed)
                .is_err(),
            "moved to another room"
        );
        let mut tampered = s.sealed.clone();
        tampered[0] ^= 1;
        assert!(
            sealer
                .open(room, 7, s.key_version, &s.nonce, &tampered)
                .is_err()
        );
    }

    #[test]
    fn the_newest_key_seals_and_every_key_given_still_opens() {
        let old = Sealer::from_spec(&format!("1:{}", key(1))).expect("one key");
        let room = Uuid::new_v4();
        let before = old.seal(room, 1, "before").expect("sealed");
        let rotated = Sealer::from_spec(&format!("1:{}, 2:{}", key(1), key(2))).expect("two keys");
        let after = rotated.seal(room, 2, "after").expect("sealed");
        assert_eq!((before.key_version, after.key_version), (1, 2));
        assert_eq!(
            rotated
                .open(room, 1, 1, &before.nonce, &before.sealed)
                .expect("old opens"),
            "before"
        );
        assert!(
            old.open(room, 2, 2, &after.nonce, &after.sealed).is_err(),
            "unknown version"
        );
    }

    #[test]
    fn a_bad_key_list_is_refused_without_quoting_a_key() {
        let short = base64::engine::general_purpose::STANDARD.encode([9u8; 16]);
        for (spec, why) in [
            (String::new(), "no message key"),
            (format!("0:{}", key(1)), "start at 1"),
            (format!("1:{short}"), "not 32 bytes"),
            ("1:not base64!".into(), "not base64"),
            (format!("1:{},1:{}", key(1), key(2)), "given twice"),
        ] {
            let refused = Sealer::from_spec(&spec).expect_err(why).to_string();
            assert!(refused.contains(why), "{refused}");
            assert!(
                !refused.contains(&key(1)) && !refused.contains(&short),
                "{refused}"
            );
        }
        assert_eq!(
            format!(
                "{:?}",
                Sealer::from_spec(&format!("1:{}", key(1))).expect("a key")
            ),
            "Sealer { versions: [1] }"
        );
    }
}
