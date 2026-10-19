//! The encrypted vault file format.
//!
//! # Threat model
//!
//! The attacker has the vault file: they stole the laptop, restored a backup, or
//! read a synced folder. They do not have the passphrase. The file must not
//! yield the recovery phrase to an offline attacker with a GPU cluster, and it
//! must not be silently modifiable.
//!
//! # Construction
//!
//! - **Argon2id** turns the passphrase into a 32-byte key. Memory-hard, so GPU
//!   and ASIC attackers lose most of their advantage. Parameters are stored in
//!   the header so they can be raised later without breaking old vaults.
//! - **XChaCha20-Poly1305** encrypts the payload. The 192-bit nonce is large
//!   enough to be drawn at random with no birthday-collision concern, which
//!   removes the nonce-reuse footgun that AES-GCM carries.
//! - **The header is the AEAD associated data.** Version, salt and Argon2
//!   parameters are therefore authenticated. An attacker cannot downgrade the
//!   work factor to make cracking cheaper: tampering fails the Poly1305 tag.
//!
//! This is deliberately stronger than the browser extension's PBKDF2-SHA256 at
//! 600k iterations with AES-256-GCM and a hand-rolled `sha256(key || ct)` MAC
//! (`apps/wallet-extension/background/keyring.ts`). PBKDF2 is not memory-hard,
//! and encrypt-and-MAC over the ciphertext alone leaves the KDF parameters
//! unauthenticated.

use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    XChaCha20Poly1305, XNonce,
};
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{QorError, QorResult};
use crate::vault::derive::fill_random;

/// Bump only on an incompatible format change.
pub const FORMAT_VERSION: u16 = 1;

/// Argon2id cost parameters.
///
/// 64 MiB with 3 passes is the OWASP-recommended floor and takes roughly a
/// tenth of a second on a current desktop CPU. That is imperceptible when
/// unlocking once per session, and it makes a large-scale offline guessing
/// campaign expensive in RAM rather than merely in cycles.
const ARGON_MEMORY_KIB: u32 = 65_536;
const ARGON_PASSES: u32 = 3;
const ARGON_LANES: u32 = 4;

const SALT_LEN: usize = 32;
const NONCE_LEN: usize = 24;
const KEY_LEN: usize = 32;

/// Authenticated, non-secret header. Serialised as the AEAD associated data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Header {
    pub version: u16,
    pub salt: String,
    pub memory_kib: u32,
    pub passes: u32,
    pub lanes: u32,
}

impl Header {
    fn fresh(salt: &[u8; SALT_LEN]) -> Self {
        Self {
            version: FORMAT_VERSION,
            salt: hex::encode(salt),
            memory_kib: ARGON_MEMORY_KIB,
            passes: ARGON_PASSES,
            lanes: ARGON_LANES,
        }
    }

    /// Canonical bytes bound into the AEAD tag.
    fn associated_data(&self) -> Vec<u8> {
        // serde_json on a struct with fixed field order is deterministic here,
        // and both seal and open derive it from the same struct, so the bytes
        // always match.
        serde_json::to_vec(self).expect("header is plain data and always serialises")
    }

    fn salt_bytes(&self) -> QorResult<[u8; SALT_LEN]> {
        let raw = hex::decode(&self.salt)
            .map_err(|_| QorError::VaultCorrupt("salt is not valid hex".into()))?;
        raw.try_into()
            .map_err(|_| QorError::VaultCorrupt("salt has the wrong length".into()))
    }
}

/// The on-disk vault file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SealedVault {
    pub header: Header,
    pub nonce: String,
    pub ciphertext: String,
}

/// A passphrase-derived key. Zeroized on drop.
#[derive(ZeroizeOnDrop)]
struct DerivedKey([u8; KEY_LEN]);

fn derive_key(passphrase: &str, header: &Header) -> QorResult<DerivedKey> {
    let params = Params::new(
        header.memory_kib,
        header.passes,
        header.lanes,
        Some(KEY_LEN),
    )
    .map_err(|e| QorError::VaultCorrupt(format!("unusable Argon2 parameters: {e}")))?;

    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon
        .hash_password_into(passphrase.as_bytes(), &header.salt_bytes()?, &mut key)
        .map_err(|e| QorError::Internal(format!("key derivation failed: {e}")))?;

    Ok(DerivedKey(key))
}

/// Encrypt `plaintext` under `passphrase`.
pub fn seal(plaintext: &[u8], passphrase: &str) -> QorResult<SealedVault> {
    let mut salt = [0u8; SALT_LEN];
    fill_random(&mut salt)?;

    let header = Header::fresh(&salt);
    let key = derive_key(passphrase, &header)?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    fill_random(&mut nonce_bytes)?;

    let cipher = XChaCha20Poly1305::new(key.0.as_ref().into());
    let ciphertext = cipher
        .encrypt(
            XNonce::from_slice(&nonce_bytes),
            Payload {
                msg: plaintext,
                aad: &header.associated_data(),
            },
        )
        .map_err(|_| QorError::Internal("vault encryption failed".into()))?;

    Ok(SealedVault {
        header,
        nonce: hex::encode(nonce_bytes),
        ciphertext: hex::encode(ciphertext),
    })
}

/// Decrypt a vault. Returns `BadPassphrase` on any authentication failure.
///
/// A wrong passphrase and a tampered file are deliberately indistinguishable to
/// the caller: both mean "this did not authenticate", and distinguishing them
/// would leak whether a guess was structurally close.
pub fn open(vault: &SealedVault, passphrase: &str) -> QorResult<Zeroizing> {
    if vault.header.version > FORMAT_VERSION {
        return Err(QorError::VaultCorrupt(format!(
            "vault format v{} was written by a newer launcher; upgrade to open it",
            vault.header.version
        )));
    }

    let key = derive_key(passphrase, &vault.header)?;

    let nonce_bytes = hex::decode(&vault.nonce)
        .map_err(|_| QorError::VaultCorrupt("nonce is not valid hex".into()))?;
    if nonce_bytes.len() != NONCE_LEN {
        return Err(QorError::VaultCorrupt("nonce has the wrong length".into()));
    }

    let ciphertext = hex::decode(&vault.ciphertext)
        .map_err(|_| QorError::VaultCorrupt("ciphertext is not valid hex".into()))?;

    let cipher = XChaCha20Poly1305::new(key.0.as_ref().into());
    let plaintext = cipher
        .decrypt(
            XNonce::from_slice(&nonce_bytes),
            Payload {
                msg: &ciphertext,
                aad: &vault.header.associated_data(),
            },
        )
        .map_err(|_| QorError::BadPassphrase)?;

    Ok(Zeroizing(plaintext))
}

/// Owned bytes wiped on drop.
pub struct Zeroizing(pub Vec<u8>);

impl Drop for Zeroizing {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

/// Redacting `Debug`.
///
/// This type holds a decrypted recovery phrase, so it must never be derived.
/// A derived impl would put the phrase into any log line, panic message or
/// `assert!` failure that happens to format a `Result` containing one.
impl std::fmt::Debug for Zeroizing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Zeroizing(<{} bytes redacted>)", self.0.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SECRET: &[u8] = b"abandon abandon abandon about";

    #[test]
    fn round_trip_recovers_the_plaintext() {
        let sealed = seal(SECRET, "a strong passphrase").unwrap();
        let opened = open(&sealed, "a strong passphrase").unwrap();
        assert_eq!(opened.0, SECRET);
    }

    #[test]
    fn wrong_passphrase_is_rejected() {
        let sealed = seal(SECRET, "correct").unwrap();
        let err = open(&sealed, "incorrect").unwrap_err();
        assert_eq!(err.kind(), "bad_passphrase");
    }

    #[test]
    fn each_seal_uses_fresh_salt_and_nonce() {
        let a = seal(SECRET, "same passphrase").unwrap();
        let b = seal(SECRET, "same passphrase").unwrap();

        assert_ne!(a.header.salt, b.header.salt, "salt must be per-vault");
        assert_ne!(a.nonce, b.nonce, "nonce must be per-encryption");
        assert_ne!(
            a.ciphertext, b.ciphertext,
            "identical plaintext must not repeat"
        );
    }

    #[test]
    fn tampering_with_the_ciphertext_is_detected() {
        let mut sealed = seal(SECRET, "passphrase").unwrap();
        let mut raw = hex::decode(&sealed.ciphertext).unwrap();
        raw[0] ^= 0x01;
        sealed.ciphertext = hex::encode(raw);

        assert_eq!(
            open(&sealed, "passphrase").unwrap_err().kind(),
            "bad_passphrase"
        );
    }

    /// The point of putting the header in the associated data: an attacker who
    /// rewrites the work factor down to make cracking cheap must fail the tag.
    ///
    /// The substituted parameters are deliberately *valid* Argon2 settings, just
    /// far weaker ones. Nonsensical values would be refused by `Params::new`
    /// before the tag is ever checked, which would test the wrong thing.
    #[test]
    fn argon_parameters_cannot_be_downgraded() {
        let mut sealed = seal(SECRET, "passphrase").unwrap();
        assert!(
            sealed.header.memory_kib > 1024,
            "precondition: the default is strong"
        );

        sealed.header.memory_kib = 1024;
        sealed.header.passes = 1;

        assert_eq!(
            open(&sealed, "passphrase").unwrap_err().kind(),
            "bad_passphrase",
            "a downgraded but valid header must fail the Poly1305 tag"
        );
    }

    /// Salt substitution is the other half of the same protection.
    #[test]
    fn salt_cannot_be_swapped() {
        let mut sealed = seal(SECRET, "passphrase").unwrap();
        let other = seal(SECRET, "passphrase").unwrap();
        sealed.header.salt = other.header.salt;

        assert_eq!(
            open(&sealed, "passphrase").unwrap_err().kind(),
            "bad_passphrase"
        );
    }

    /// Structurally impossible parameters are refused cleanly rather than
    /// panicking somewhere inside the KDF.
    #[test]
    fn impossible_parameters_are_reported_not_panicked() {
        let mut sealed = seal(SECRET, "passphrase").unwrap();
        sealed.header.memory_kib = 8;
        sealed.header.lanes = 4;

        let err = open(&sealed, "passphrase").unwrap_err();
        assert_eq!(err.kind(), "vault_corrupt");
    }

    #[test]
    fn future_format_versions_are_refused_clearly() {
        let mut sealed = seal(SECRET, "passphrase").unwrap();
        sealed.header.version = FORMAT_VERSION + 1;

        let err = open(&sealed, "passphrase").unwrap_err();
        assert_eq!(err.kind(), "vault_corrupt");
        assert!(err.to_string().contains("newer launcher"));
    }

    #[test]
    fn vault_file_serialises_as_json() {
        let sealed = seal(SECRET, "passphrase").unwrap();
        let json = serde_json::to_string(&sealed).unwrap();
        let parsed: SealedVault = serde_json::from_str(&json).unwrap();
        assert_eq!(open(&parsed, "passphrase").unwrap().0, SECRET);
    }
}
