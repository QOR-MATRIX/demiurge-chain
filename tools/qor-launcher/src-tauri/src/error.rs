//! Unified error type for the launcher core.
//!
//! Every fallible boundary the UI can reach returns `QorError`. It serialises
//! to a tagged JSON object so the frontend can branch on `kind` without parsing
//! human-readable prose.

#[derive(Debug, thiserror::Error)]
pub enum QorError {
    #[error("vault is locked")]
    VaultLocked,

    #[error("no vault exists on this device")]
    NoVault,

    #[error("a vault already exists; refusing to overwrite it")]
    VaultExists,

    #[error("the vault is open; lock it before restoring another")]
    VaultOpen,

    #[error("incorrect passphrase")]
    BadPassphrase,

    /// A vault sealed with a passphrase before ADR-055, which opens once more
    /// with that passphrase to move to Windows Hello.
    #[error(
        "this vault was sealed with a passphrase; type it once more to move the vault to Windows \
         Hello, or restore from your recovery phrase"
    )]
    PassphraseVault,

    #[error("recovery phrase is not valid: {0}")]
    BadMnemonic(String),

    #[error("vault file is corrupt or was produced by a newer version: {0}")]
    VaultCorrupt(String),

    #[error("not signed in")]
    NotAuthenticated,

    #[error("identity service rejected the request: {0}")]
    Auth(String),

    #[error("chain rpc error: {0}")]
    Rpc(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("invalid address: {0}")]
    BadAddress(String),

    #[error("amount is not valid: {0}")]
    BadAmount(String),

    #[error("insufficient balance: holding {have} Sparks, need {need} Sparks")]
    InsufficientFunds { have: String, need: String },

    #[error("operating system keychain error: {0}")]
    Keychain(String),

    #[error("filesystem error: {0}")]
    Io(String),

    #[error("declined at the confirmation prompt; nothing was signed or changed")]
    Declined,

    /// Windows Hello could not be used. Carries a sentence a person can read.
    #[error("{0}")]
    Hello(String),

    /// Anything Qontrol refuses. Its own error already carries a sentence a
    /// person can read, so it is passed through rather than reworded.
    #[error("{0}")]
    Qontrol(String),

    /// A `qor://pay` request refused before anything was asked or signed (ADR-076). The sentence is for the person.
    #[error("{0}")]
    PaymentRefused(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl QorError {
    /// Stable machine-readable discriminant. The UI switches on this.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::VaultLocked => "vault_locked",
            Self::NoVault => "no_vault",
            Self::VaultExists => "vault_exists",
            Self::VaultOpen => "vault_open",
            Self::BadPassphrase => "bad_passphrase",
            Self::PassphraseVault => "passphrase_vault",
            Self::BadMnemonic(_) => "bad_mnemonic",
            Self::VaultCorrupt(_) => "vault_corrupt",
            Self::NotAuthenticated => "not_authenticated",
            Self::Auth(_) => "auth",
            Self::Rpc(_) => "rpc",
            Self::Network(_) => "network",
            Self::BadAddress(_) => "bad_address",
            Self::BadAmount(_) => "bad_amount",
            Self::InsufficientFunds { .. } => "insufficient_funds",
            Self::Keychain(_) => "keychain",
            Self::Io(_) => "io",
            Self::Qontrol(_) => "qontrol",
            Self::PaymentRefused(_) => "payment_refused",
            Self::Declined => "declined",
            Self::Hello(_) => "hello",
            Self::Internal(_) => "internal",
        }
    }
}

/// Wire shape sent to the frontend: `{ "kind": "...", "message": "..." }`.
///
/// `QorError` is our own type, so implementing `Serialize` directly is sound and
/// avoids the newtype dance that `#[tauri::command]` would otherwise require.
impl serde::Serialize for QorError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("QorError", 2)?;
        st.serialize_field("kind", self.kind())?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}

pub type QorResult<T> = Result<T, QorError>;

impl From<reqwest::Error> for QorError {
    fn from(e: reqwest::Error) -> Self {
        Self::Network(e.to_string())
    }
}

impl From<std::io::Error> for QorError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}
