//! Windows Hello, read only to move a vault off it (ADR-056, superseding ADR-055).
//!
//! From 28 September 2026 to ADR-056 the vault was sealed under a Windows Hello
//! signature: Hello holds an RSA key in the TPM and signs with it only after
//! the person's face, fingerprint or PIN, and RSA PKCS#1 v1.5 signatures are
//! deterministic, so the signature served as the sealing secret. A vault sealed
//! that way is opened here once, with one last Hello gesture, and resealed under
//! the keychain. Before that, ADR-054 kept an opt-in `vault.hello` copy beside a
//! passphrase vault, which is read here for the same move.

use serde::{Deserialize, Serialize};

use super::seal::SealedVault;
#[cfg(not(windows))]
use crate::error::QorError;
use crate::error::QorResult;

/// The value of `vault.qor`'s `unlock` field for a vault sealed under Hello.
pub const UNLOCK: &str = "windows-hello";

/// ADR-054's opt-in copy, beside a passphrase vault. Read only, to move it.
pub const LEGACY_COPY_FILE: &str = "vault.hello";

/// Domain tag for what Hello signs, so the signature cannot be asked for, or
/// reused, as anything else. Unchanged from ADR-054, so the key that signed for
/// an ADR-054 copy opens it here.
const CHALLENGE_TAG: &[u8] = b"demiurge:vault:windows-hello:v1:";

/// A person-present signer. The launcher's is [`system`]; tests use a stand-in.
pub trait Presence: Send + Sync {
    /// Whether the device can be asked at all: Hello set up, a PIN at least.
    fn available(&self) -> bool;
    /// Make the key if this Windows account has none yet, which asks the person.
    /// An existing key is kept, so a vault set aside under it still opens.
    fn ensure(&self) -> QorResult<()>;
    /// Sign with the key. Asks the person; a cancel is [`QorError::Declined`].
    ///
    /// [`QorError::Declined`]: crate::error::QorError::Declined
    fn sign(&self, challenge: &[u8]) -> QorResult<Vec<u8>>;
}

/// What sits in `vault.qor`. Nothing in it opens anything without Hello.
#[derive(Debug, Serialize, Deserialize)]
pub struct HelloVault {
    /// Always [`UNLOCK`]. It is what tells this file from a passphrase seal.
    pub unlock: String,
    pub version: u32,
    /// 32 random bytes, hex, mixed into the challenge.
    pub salt: String,
    /// The recovery phrase, sealed under the signature.
    pub sealed: SealedVault,
}

pub const FORMAT_VERSION: u32 = 2;

/// ADR-054's `vault.hello`: a second copy of a passphrase vault's phrase.
#[derive(Debug, Deserialize)]
pub struct LegacyCopy {
    pub salt: String,
    /// BLAKE3 of the `vault.qor` this copy was made from, hex.
    pub bound_to: String,
    pub sealed: SealedVault,
}

/// The bytes Hello signs for a given salt.
pub fn challenge(salt: &[u8]) -> Vec<u8> {
    [CHALLENGE_TAG, salt].concat()
}

/// The platform's [`Presence`].
pub fn system() -> Box<dyn Presence> {
    #[cfg(windows)]
    {
        Box::new(windows_hello::WindowsHello)
    }
    #[cfg(not(windows))]
    {
        Box::new(Unavailable)
    }
}

/// Every platform without Hello, until Touch ID and the Secret Service follow.
#[cfg(not(windows))]
struct Unavailable;

#[cfg(not(windows))]
impl Presence for Unavailable {
    fn available(&self) -> bool {
        false
    }
    fn ensure(&self) -> QorResult<()> {
        Err(QorError::Hello(
            "the vault needs Windows Hello, which is only available on Windows".into(),
        ))
    }
    fn sign(&self, _: &[u8]) -> QorResult<Vec<u8>> {
        Err(QorError::Hello(
            "the vault needs Windows Hello, which is only available on Windows".into(),
        ))
    }
}

#[cfg(windows)]
mod windows_hello {
    use super::Presence;
    use crate::error::{QorError, QorResult};

    use windows::core::{Array, HSTRING};
    use windows::Security::Credentials::{
        KeyCredentialCreationOption, KeyCredentialManager, KeyCredentialStatus,
    };
    use windows::Security::Cryptography::CryptographicBuffer;

    /// The key's name in the person's Windows Hello store.
    const KEY_NAME: &str = "Demiurge QOR Launcher vault";

    pub struct WindowsHello;

    fn os(error: windows::core::Error) -> QorError {
        QorError::Hello(format!("Windows Hello failed: {}", error.message()))
    }

    fn status(status: KeyCredentialStatus) -> QorResult<()> {
        match status {
            KeyCredentialStatus::Success => Ok(()),
            KeyCredentialStatus::UserCanceled | KeyCredentialStatus::UserPrefersPassword => {
                Err(QorError::Declined)
            }
            KeyCredentialStatus::NotFound => Err(QorError::Hello(
                "Windows Hello no longer has this vault's key. Resetting the Windows PIN removes \
                 it. Restore the vault from your recovery phrase"
                    .into(),
            )),
            KeyCredentialStatus::SecurityDeviceLocked => Err(QorError::Hello(
                "Windows Hello is locked after too many attempts. Unlock Windows Hello from the \
                 Windows sign-in screen, then try again"
                    .into(),
            )),
            _ => Err(QorError::Hello(
                "Windows Hello could not complete. Try again".into(),
            )),
        }
    }

    impl Presence for WindowsHello {
        fn available(&self) -> bool {
            KeyCredentialManager::IsSupportedAsync()
                .and_then(|op| op.join())
                .unwrap_or(false)
        }

        fn ensure(&self) -> QorResult<()> {
            let opened = KeyCredentialManager::OpenAsync(&HSTRING::from(KEY_NAME))
                .and_then(|op| op.join())
                .map_err(os)?;
            if opened.Status().map_err(os)? == KeyCredentialStatus::Success {
                return Ok(());
            }
            let created = KeyCredentialManager::RequestCreateAsync(
                &HSTRING::from(KEY_NAME),
                KeyCredentialCreationOption::FailIfExists,
            )
            .and_then(|op| op.join())
            .map_err(os)?;
            status(created.Status().map_err(os)?)
        }

        fn sign(&self, challenge: &[u8]) -> QorResult<Vec<u8>> {
            let opened = KeyCredentialManager::OpenAsync(&HSTRING::from(KEY_NAME))
                .and_then(|op| op.join())
                .map_err(os)?;
            status(opened.Status().map_err(os)?)?;
            let key = opened.Credential().map_err(os)?;

            let data = CryptographicBuffer::CreateFromByteArray(challenge).map_err(os)?;
            let signed = key
                .RequestSignAsync(&data)
                .and_then(|op| op.join())
                .map_err(os)?;
            status(signed.Status().map_err(os)?)?;

            let mut out = Array::<u8>::new();
            CryptographicBuffer::CopyToByteArray(&signed.Result().map_err(os)?, &mut out)
                .map_err(os)?;
            Ok(out.to_vec())
        }
    }
}

/// Test support: a deterministic stand-in for Hello, keyed like a TPM key.
#[cfg(any(test, feature = "testing"))]
pub mod testing {
    use super::Presence;
    use crate::error::{QorError, QorResult};
    use parking_lot::Mutex;

    pub struct FakeHello {
        pub key: Mutex<[u8; 32]>,
        pub decline: bool,
        pub present: bool,
        /// Every time the person would have been asked: making the key, or signing.
        pub asked: Mutex<usize>,
        /// Whether this Windows account has the key yet.
        pub enrolled: Mutex<bool>,
    }

    impl Default for FakeHello {
        fn default() -> Self {
            Self::new()
        }
    }

    impl FakeHello {
        pub fn new() -> Self {
            Self {
                key: Mutex::new([7u8; 32]),
                decline: false,
                present: true,
                asked: Mutex::new(0),
                enrolled: Mutex::new(false),
            }
        }
    }

    impl Presence for FakeHello {
        fn available(&self) -> bool {
            self.present
        }
        fn ensure(&self) -> QorResult<()> {
            if *self.enrolled.lock() {
                return Ok(());
            }
            *self.asked.lock() += 1;
            if self.decline {
                return Err(QorError::Declined);
            }
            *self.enrolled.lock() = true;
            Ok(())
        }
        fn sign(&self, challenge: &[u8]) -> QorResult<Vec<u8>> {
            *self.asked.lock() += 1;
            if self.decline {
                return Err(QorError::Declined);
            }
            let mut out = Vec::with_capacity(256);
            let mut hasher = blake3::Hasher::new_keyed(&self.key.lock());
            hasher.update(challenge);
            let mut reader = hasher.finalize_xof();
            out.resize(256, 0);
            reader.fill(&mut out);
            Ok(out)
        }
    }
}

/// Against the real Windows Hello. Ignored by default: `ensure` and `sign`
/// need a person, so only the question that shows no prompt is asked here.
#[cfg(all(test, windows))]
mod live {
    #[test]
    #[ignore = "asks the real Windows Hello whether it is set up on this machine"]
    fn windows_answers_whether_hello_is_set_up() {
        let available = super::system().available();
        println!("Windows Hello available on this machine: {available}");
    }
}
