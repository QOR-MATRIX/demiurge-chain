//! QOR ID: the launcher's authentication client.
//!
//! Talks to the Rust auth service in `services/qor-auth`. Two sign-in routes are
//! supported, matching the two the service exposes:
//!
//! 1. **Passphrase** against `POST /api/v1/auth/login`, taking a QOR ID handle
//!    or an email plus a password.
//! 2. **Keypair** against `GET /api/v1/auth/challenge` followed by
//!    `POST /api/v1/auth/keypair-login`, where the vault signs a server-issued
//!    challenge. Nothing reusable crosses the wire, so a captured request is
//!    worthless.
//!
//! Keypair is the better route and the launcher prefers it wherever a vault is
//! present. It is also the only route that works for an account that never had a
//! password, which is how agent accounts are created.
//!
//! # Token handling
//!
//! Access and refresh tokens live in the OS keychain, not on disk and not in the
//! webview. `localStorage` in a webview is readable by anything that achieves
//! script execution in that webview; the keychain is not.

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::error::{QorError, QorResult};
use crate::vault::Vault;
use crate::{Confirm, Prompt};

/// Default auth service endpoint: QOR ID at `id.qorsync.dev` (ADR-042, ADR-063).
pub const DEFAULT_AUTH: &str = "https://id.qorsync.dev/api/v1";

/// Local development endpoint, matching `QOR_AUTH__SERVER__PORT`.
pub const LOCAL_AUTH: &str = "http://127.0.0.1:8080/api/v1";

/// Keychain service name under which tokens are stored.
const KEYCHAIN_SERVICE: &str = "cloud.demiurge.qor-launcher";

/// Domain tag for QOR ID challenge signatures (security track item 7).
///
/// The vault key that answers a QOR ID challenge also signs chain transactions,
/// so it never signs a challenge bare: it signs `CHALLENGE_DOMAIN || challenge`.
/// `services/qor-auth` verifies with the identical tag.
const CHALLENGE_DOMAIN: &str = "demiurge:qor-id:challenge:v1:";

/// The bytes the vault signs to answer `challenge`.
///
/// The challenge is checked against the only shape the service issues,
/// `demiurge:<unix seconds>:<64 lowercase hex>`, before anything is signed. An
/// endpoint that sends anything else, by mistake or by design, gets no
/// signature at all.
fn challenge_message(challenge: &str) -> QorResult<Vec<u8>> {
    let mut parts = challenge.splitn(3, ':');
    let well_formed = parts.next() == Some("demiurge")
        && parts.next().is_some_and(|seconds| {
            !seconds.is_empty() && seconds.bytes().all(|b| b.is_ascii_digit())
        })
        && parts.next().is_some_and(|nonce| {
            nonce.len() == 64
                && nonce
                    .bytes()
                    .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
        });

    if !well_formed {
        return Err(QorError::Auth(
            "the identity service sent a challenge in an unexpected form; nothing was signed"
                .into(),
        ));
    }

    Ok(format!("{CHALLENGE_DOMAIN}{challenge}").into_bytes())
}

/// The signed-in user, as shown in the launcher shell.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    /// Human-readable identity: the username alone, unique on its own (ADR-075).
    pub qor_id: String,
    pub username: String,
    /// Retired with ADR-075: QOR ID sends 1 for every account. Kept so the shape is unchanged.
    pub discriminator: u16,
    pub role: String,
    /// On-chain address, when the account has one linked.
    pub address: Option<String>,
    pub avatar_url: Option<String>,
}

/// Token pair returned by the auth service.
#[derive(Debug, Clone, Deserialize)]
struct TokenPair {
    access_token: String,
    refresh_token: String,
    #[serde(default)]
    expires_in: i64,
}

#[derive(Debug, Deserialize)]
struct ChallengeResponse {
    challenge: String,
}

/// Auth client. Holds no secrets in memory beyond the lifetime of a call.
pub struct IdentityClient {
    http: reqwest::Client,
    endpoint: parking_lot::RwLock<String>,
    session: parking_lot::RwLock<Option<Session>>,
}

impl IdentityClient {
    pub fn new(endpoint: impl Into<String>) -> QorResult<Self> {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(20))
            .connect_timeout(Duration::from_secs(8))
            .user_agent(concat!("QorLauncher/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|e| QorError::Internal(format!("cannot build http client: {e}")))?;

        Ok(Self {
            http,
            endpoint: parking_lot::RwLock::new(endpoint.into()),
            session: parking_lot::RwLock::new(None),
        })
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.read().clone()
    }

    pub fn set_endpoint(&self, endpoint: impl Into<String>) {
        *self.endpoint.write() = endpoint.into();
    }

    /// The signed-in user, if any.
    pub fn session(&self) -> Option<Session> {
        self.session.read().clone()
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.endpoint().trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    /// The host-side prompt for a QOR ID challenge signature (roadmap L1.4).
    ///
    /// The account is shown as SS58, the same string the person sees in the
    /// vault and everywhere else (ADR-024), so what they approve is what they
    /// can recognise.
    fn challenge_prompt(&self, title: &str, approve: &str, address: &str, effect: &str) -> Prompt {
        Prompt {
            title: title.into(),
            body: format!(
                "Account: {address}\nIdentity service: {}\n\n{effect} \
                 The signature proves you hold this account's key. It cannot move CGT.",
                self.endpoint()
            ),
            approve: approve.into(),
        }
    }

    /// Sign in with a QOR ID handle or email and a password.
    pub async fn login(&self, identifier: &str, password: &str) -> QorResult<Session> {
        let identifier = identifier.trim();
        if identifier.is_empty() {
            return Err(QorError::Auth("enter your QOR ID or email".into()));
        }
        if password.is_empty() {
            return Err(QorError::Auth("enter your password".into()));
        }

        let body = serde_json::json!({
            "identifier": identifier,
            "password": password,
            "device_id": device_id(),
        });

        let value = self.post_json("auth/login", &body).await?;
        self.accept_tokens(&value)?;
        self.load_profile().await
    }

    /// Sign in by proving control of a vault key.
    ///
    /// The server issues a short-lived challenge bound to the public key; the
    /// vault signs it; the server verifies against the same key. No password
    /// exists to phish and no reusable secret crosses the wire.
    pub async fn login_with_key(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        address: &str,
    ) -> QorResult<Session> {
        // One canonical form on the wire: the SS58 address (ADR-024). Parsing
        // it here means a malformed address fails before anything is signed.
        let address = crate::vault::canonical_address(address)?;

        let challenge = self.request_challenge(&address).await?;
        let message = challenge_message(&challenge)?;
        let prompt = self.challenge_prompt(
            "Sign in to QOR ID?",
            "Sign in",
            &address,
            "Your vault signs a sign-in challenge from this identity service.",
        );
        let signature = vault.sign(confirm, &prompt, &address, &message).await?;

        let body = serde_json::json!({
            "address": address,
            "challenge": challenge,
            "signature": signature,
            "device_id": device_id(),
        });

        let value = self.post_json("auth/keypair-login", &body).await?;
        self.accept_tokens(&value)?;
        self.load_profile().await
    }

    /// Create an account bound to a vault key.
    pub async fn register_with_key(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        address: &str,
        username: Option<&str>,
    ) -> QorResult<Session> {
        // One canonical form on the wire: the SS58 address (ADR-024). Parsing
        // it here means a malformed address fails before anything is signed.
        let address = crate::vault::canonical_address(address)?;

        if let Some(name) = username {
            validate_username(name)?;
        }

        let challenge = self.request_challenge(&address).await?;
        let message = challenge_message(&challenge)?;
        let prompt = self.challenge_prompt(
            "Create a QOR ID?",
            "Sign and create",
            &address,
            "Your vault signs a challenge to create a QOR ID bound to this account.",
        );
        let signature = vault.sign(confirm, &prompt, &address, &message).await?;

        let body = serde_json::json!({
            "address": address,
            "username": username,
            "challenge": challenge,
            "signature": signature,
            "device_id": device_id(),
        });

        let value = self
            .post_json("auth/keypair-register", &body)
            .await
            .map_err(guide_already_registered)?;
        self.accept_tokens(&value)?;
        self.load_profile().await
    }

    /// Bind a vault address to the signed-in QOR ID.
    ///
    /// Proves control of the key by signing a server-issued challenge, so the
    /// account cannot claim an address it does not hold. Until this is done the
    /// identity and the money are two unrelated facts about the same person.
    pub async fn link_wallet(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        address: &str,
    ) -> QorResult<Session> {
        // One canonical form on the wire: the SS58 address (ADR-024). Parsing
        // it here means a malformed address fails before anything is signed.
        let address = crate::vault::canonical_address(address)?;

        let challenge = self.request_challenge(&address).await?;
        let message = challenge_message(&challenge)?;
        let prompt = self.challenge_prompt(
            "Link this account to your QOR ID?",
            "Sign and link",
            &address,
            "Your vault signs a challenge that binds this account to the QOR ID you are signed in with.",
        );
        let signature = vault.sign(confirm, &prompt, &address, &message).await?;

        let token = read_token(TokenKind::Access)?;

        let response = self
            .http
            .post(self.url("auth/link-keypair"))
            .bearer_auth(token)
            .json(&serde_json::json!({
                "address": address,
                "challenge": challenge,
                "signature": signature,
            }))
            .send()
            .await
            .map_err(|e| QorError::Network(e.to_string()))?;

        read_json(response, "auth/link-keypair").await?;
        self.load_profile().await
    }

    /// Ask whether a handle is free. Used for live feedback during sign-up.
    pub async fn username_available(&self, username: &str) -> QorResult<bool> {
        validate_username(username)?;

        let value = self
            .post_json(
                "auth/check-username",
                &serde_json::json!({ "username": username }),
            )
            .await?;

        Ok(value
            .get("available")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false))
    }

    async fn request_challenge(&self, address: &str) -> QorResult<String> {
        let response = self
            .http
            .get(self.url("auth/challenge"))
            .query(&[("address", address)])
            .send()
            .await
            .map_err(|e| QorError::Network(e.to_string()))?;

        let value = read_json(response, "auth/challenge").await?;
        let parsed: ChallengeResponse = serde_json::from_value(value)
            .map_err(|e| QorError::Auth(format!("malformed challenge: {e}")))?;

        Ok(parsed.challenge)
    }

    /// Fetch the profile for the current access token.
    async fn load_profile(&self) -> QorResult<Session> {
        let token = read_token(TokenKind::Access)?;

        let response = self
            .http
            .get(self.url("profile"))
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| QorError::Network(e.to_string()))?;

        let value = read_json(response, "profile").await?;
        let session = parse_session(&value)?;

        *self.session.write() = Some(session.clone());
        Ok(session)
    }

    /// Exchange the refresh token for a new access token.
    pub async fn refresh(&self) -> QorResult<Session> {
        let refresh_token = read_token(TokenKind::Refresh)?;

        let value = self
            .post_json(
                "auth/refresh",
                &serde_json::json!({ "refresh_token": refresh_token }),
            )
            .await?;

        self.accept_tokens(&value)?;
        self.load_profile().await
    }

    /// Sign out and clear stored tokens.
    ///
    /// Local state is cleared even if the server call fails, because a user who
    /// pressed sign out must end up signed out on this device regardless of
    /// whether the network cooperated.
    pub async fn logout(&self) {
        if let Ok(token) = read_token(TokenKind::Access) {
            let _ = self
                .http
                .post(self.url("auth/logout"))
                .bearer_auth(token)
                .send()
                .await;
        }

        clear_token(TokenKind::Access);
        clear_token(TokenKind::Refresh);
        *self.session.write() = None;
    }

    /// Restore a session from keychain tokens at startup.
    pub async fn restore(&self) -> QorResult<Session> {
        match self.load_profile().await {
            Ok(session) => Ok(session),
            // An expired access token is the common case on a cold start.
            Err(_) => self.refresh().await,
        }
    }

    async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> QorResult<serde_json::Value> {
        let response = self
            .http
            .post(self.url(path))
            .json(body)
            .send()
            .await
            .map_err(|e| QorError::Network(e.to_string()))?;

        read_json(response, path).await
    }

    fn accept_tokens(&self, value: &serde_json::Value) -> QorResult<()> {
        let tokens: TokenPair = serde_json::from_value(value.clone()).map_err(|_| {
            QorError::Auth("the identity service did not return a usable token pair".into())
        })?;

        let _ = tokens.expires_in;
        write_token(TokenKind::Access, &tokens.access_token)?;
        write_token(TokenKind::Refresh, &tokens.refresh_token)?;
        Ok(())
    }
}

/// Which token slot in the keychain.
#[derive(Clone, Copy)]
enum TokenKind {
    Access,
    Refresh,
}

impl TokenKind {
    fn entry_name(self) -> &'static str {
        match self {
            Self::Access => "access_token",
            Self::Refresh => "refresh_token",
        }
    }
}

fn keychain(kind: TokenKind) -> QorResult<keyring::Entry> {
    keyring::Entry::new(KEYCHAIN_SERVICE, kind.entry_name())
        .map_err(|e| QorError::Keychain(e.to_string()))
}

fn write_token(kind: TokenKind, token: &str) -> QorResult<()> {
    keychain(kind)?
        .set_password(token)
        .map_err(|e| QorError::Keychain(e.to_string()))
}

fn read_token(kind: TokenKind) -> QorResult<String> {
    keychain(kind)?
        .get_password()
        .map_err(|_| QorError::NotAuthenticated)
}

fn clear_token(kind: TokenKind) {
    if let Ok(entry) = keychain(kind) {
        let _ = entry.delete_credential();
    }
}

/// Read a response body, turning non-2xx into a usable message.
async fn read_json(response: reqwest::Response, context: &str) -> QorResult<serde_json::Value> {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    let value: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);

    if status.is_success() {
        return Ok(value);
    }

    Err(match status.as_u16() {
        401 | 403 => QorError::NotAuthenticated,
        _ => QorError::Auth(error_detail(&value, &body, context, status)),
    })
}

/// The service's own explanation of a failed request.
///
/// QOR ID reports failures as `{"error": {"code": "...", "message": "..."}}`.
/// This read only a flat `error` or `message` string, so every refusal reached
/// the person as raw JSON. Both shapes are read; the body itself, cut to 200
/// characters, appears only when neither is present.
fn error_detail(
    value: &serde_json::Value,
    body: &str,
    context: &str,
    status: reqwest::StatusCode,
) -> String {
    let message = ["/error/message", "/error", "/message"]
        .iter()
        .find_map(|pointer| value.pointer(pointer).and_then(serde_json::Value::as_str));

    match message {
        Some(message) => message.to_owned(),
        None if body.trim().is_empty() => format!("{context} returned HTTP {status}"),
        None => body.chars().take(200).collect(),
    }
}

/// What QOR ID says when a key that already has an account tries to claim another.
const ALREADY_REGISTERED: &str = "This account already holds a QOR ID";

/// QOR ID allows one account per key. Its refusal names the problem but not the
/// way forward, which is to sign in with that key.
fn guide_already_registered(error: QorError) -> QorError {
    match &error {
        QorError::Auth(message) if message == ALREADY_REGISTERED => QorError::Auth(
            "this vault key already has a QOR ID. Go back and sign in with your vault key. \
             A second QOR ID needs a different key."
                .into(),
        ),
        _ => error,
    }
}

/// What QOR ID says when a key signs in but has no account yet.
const NO_ACCOUNT: &str = "No account holds this key. Please register first.";

/// Whether a key sign-in failed only because the key has no QOR ID yet, so the
/// next step is choosing a name rather than reporting an error.
pub fn needs_registration(error: &QorError) -> bool {
    matches!(error, QorError::Auth(message) if message == NO_ACCOUNT)
}

/// Build a [`Session`] from a profile payload.
///
/// # Matching the server, not a guess at it
///
/// This previously required a top-level `username` and failed the whole sign-in
/// when it was missing. The service does not send one. Its `/profile` response
/// looks like this:
///
/// ```json
/// {
///   "qor_id": "architect#0001",
///   "display_name": "architect",
///   "role": "user",
///   "on_chain": { "address": "0x2099…", "cgt_balance": "0.00" },
///   "avatar_url": null
/// }
/// ```
///
/// The effect was that every sign-in succeeded on the server, stored valid
/// tokens, and then reported failure to the user, who retyped their credentials
/// into a form that had reset. The tests below pin this exact payload so the two
/// cannot drift apart again.
///
/// The handle is recovered from `qor_id` when it is not sent separately. A QOR ID
/// is the username alone (ADR-075); an older service's `handle#0001` is read too,
/// and shown without its number.
fn parse_session(value: &serde_json::Value) -> QorResult<Session> {
    let root = value.get("user").unwrap_or(value);

    let sent = root
        .get("qor_id")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned);

    // A retired `handle#0001` splits into its two halves; only the handle is shown.
    let (handle_from_qor, discriminator_from_qor) = match sent.as_deref() {
        Some(id) => match id.rsplit_once('#') {
            Some((handle, number)) => (Some(handle.to_string()), number.parse::<u16>().ok()),
            None => (Some(id.to_string()), None),
        },
        None => (None, None),
    };
    let qor_id = handle_from_qor.clone();

    let username = root
        .get("username")
        .or_else(|| root.get("display_name"))
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .or(handle_from_qor)
        .ok_or_else(|| {
            QorError::Auth("profile response carried no name for this account".into())
        })?;

    let discriminator = root
        .get("discriminator")
        .and_then(serde_json::Value::as_u64)
        .map(|d| d as u16)
        .or(discriminator_from_qor)
        .unwrap_or(0);

    // The chain address is nested under `on_chain`; older shapes put it at the
    // root. Accept both rather than depending on one.
    let address = root
        .get("on_chain")
        .and_then(|o| o.get("address"))
        .or_else(|| root.get("on_chain_address"))
        .or_else(|| root.get("address"))
        .and_then(serde_json::Value::as_str)
        .filter(|a| !a.is_empty())
        .map(str::to_owned);

    Ok(Session {
        qor_id: qor_id.unwrap_or_else(|| username.to_lowercase()),
        username,
        discriminator,
        role: root
            .get("role")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("user")
            .to_string(),
        address,
        avatar_url: root
            .get("avatar_url")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
    })
}

/// QOR ID's username rules, exactly as the service applies them (`QorId::is_valid_username`):
/// 3 to 20 characters, ASCII letters, digits and underscore. A name is unique on its own (ADR-075).
fn validate_username(username: &str) -> QorResult<()> {
    let name = username.trim();

    if name.len() < 3 || name.len() > 20 {
        return Err(QorError::Auth("a QOR ID must be 3 to 20 characters".into()));
    }
    if name.contains('#') {
        return Err(QorError::Auth(
            "a QOR ID has no #number: it is just your name".into(),
        ));
    }
    if !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(QorError::Auth(
            "a QOR ID may contain only letters, numbers and underscore".into(),
        ));
    }
    Ok(())
}

/// Stable per-install device identifier, so the service can list and revoke
/// sessions per device. Derived from the hostname, never from anything that
/// identifies the person.
fn device_id() -> String {
    let host = hostname().unwrap_or_else(|| "unknown-host".to_string());
    format!("qor-launcher:{host}")
}

fn hostname() -> Option<String> {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .ok()
        .filter(|h| !h.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn username_rules_are_enforced() {
        assert!(validate_username("architect").is_ok());
        assert!(validate_username("qor_user_1").is_ok());
        assert!(
            validate_username("qor-user").is_err(),
            "QOR ID refuses a hyphen, so the launcher does too"
        );

        assert!(validate_username("ab").is_err(), "too short");
        assert!(validate_username(&"a".repeat(21)).is_err(), "too long");
        assert!(validate_username("has space").is_err());
        assert!(validate_username("emoji-🔥").is_err());

        let err = validate_username("architect#0001").unwrap_err();
        assert!(
            err.to_string().contains("#number"),
            "should say a QOR ID has no number"
        );
    }

    /// The exact payload `services/qor-auth` returns from `GET /profile`,
    /// captured from a running instance. This is the shape that used to make
    /// every sign-in report failure.
    #[test]
    fn parses_the_real_profile_payload() {
        let session = parse_session(&json!({
            "account_type": "human",
            "auth_method": "password",
            "avatar_url": null,
            "created_at": "2026-09-11T15:34:53.635616Z",
            "display_name": "architect",
            "email": null,
            "id": "f67c563e-f1f6-4eee-9b1f-2978b6ec0abd",
            "on_chain": {
                "address": "0x2099f5bc003f2061ce6860356dd0b5d61a8d86a4e309a7ccbeb7cb3ac2036825",
                "cgt_balance": "0.00"
            },
            "qor_id": "architect#0001",
            "role": "user",
            "status": "active"
        }))
        .unwrap();

        assert_eq!(
            session.qor_id, "architect",
            "an old #0001 is not shown (ADR-075)"
        );
        assert_eq!(session.username, "architect", "recovered from display_name");
        assert_eq!(session.discriminator, 1, "recovered from the qor_id suffix");
        assert_eq!(session.role, "user");
        assert_eq!(
            session.address.as_deref(),
            Some("0x2099f5bc003f2061ce6860356dd0b5d61a8d86a4e309a7ccbeb7cb3ac2036825"),
            "address is nested under on_chain"
        );
        assert!(session.avatar_url.is_none());
    }

    /// What QOR ID sends since ADR-075: the name alone, with a discriminator of 1.
    #[test]
    fn a_qor_id_is_the_name_alone() {
        let session = parse_session(&json!({
            "user": { "username": "godmode", "discriminator": 1, "qor_id": "godmode" }
        }))
        .unwrap();
        assert_eq!(session.qor_id, "godmode");
        assert_eq!(session.username, "godmode");
    }

    /// The handle must survive even when only `qor_id` is present.
    #[test]
    fn recovers_the_handle_from_qor_id_alone() {
        let session = parse_session(&json!({ "qor_id": "aeon#0042" })).unwrap();

        assert_eq!(session.username, "aeon");
        assert_eq!(session.discriminator, 42);
        assert_eq!(session.qor_id, "aeon");
    }

    /// The older flat shape must keep working.
    #[test]
    fn parses_a_flat_profile_payload() {
        let session = parse_session(&json!({
            "username": "architect",
            "discriminator": 1,
            "role": "god",
            "on_chain_address": "0xabc"
        }))
        .unwrap();

        assert_eq!(session.qor_id, "architect", "the name alone (ADR-075)");
        assert_eq!(session.role, "god");
        assert_eq!(session.address.as_deref(), Some("0xabc"));
    }

    #[test]
    fn parses_a_profile_nested_under_user() {
        let session = parse_session(&json!({
            "user": { "username": "aeon", "discriminator": 42, "qor_id": "aeon#0042" }
        }))
        .unwrap();

        assert_eq!(session.qor_id, "aeon");
        assert_eq!(session.role, "user", "role defaults when absent");
    }

    /// An empty address string must not be presented as a linked wallet, or the
    /// launcher would claim a binding that does not exist.
    #[test]
    fn treats_an_empty_address_as_unlinked() {
        let session =
            parse_session(&json!({ "qor_id": "x#0001", "on_chain": { "address": "" } })).unwrap();
        assert!(session.address.is_none());
    }

    #[test]
    fn rejects_a_profile_with_no_name_at_all() {
        assert!(parse_session(&json!({ "role": "user" })).is_err());
    }

    #[test]
    fn challenge_messages_carry_the_domain_tag() {
        let challenge = format!("demiurge:1757800000:{}", "ab".repeat(32));
        let message = challenge_message(&challenge).unwrap();
        assert_eq!(
            message,
            format!("demiurge:qor-id:challenge:v1:{challenge}").into_bytes()
        );
        assert_eq!(
            CHALLENGE_DOMAIN, "demiurge:qor-id:challenge:v1:",
            "must match services/qor-auth"
        );
    }

    /// Anything that is not the service's challenge shape is refused before the
    /// vault is asked to sign, so a hostile endpoint cannot choose the bytes.
    #[test]
    fn malformed_challenges_are_never_signed() {
        let nonce = "ab".repeat(32);
        for bad in [
            String::new(),
            format!("demiurge:1757800000:{}", "AB".repeat(32)),
            format!("demiurge:17578x0000:{nonce}"),
            format!("demiurge::{nonce}"),
            format!("other:1757800000:{nonce}"),
            format!("demiurge:1757800000:{}", "ab".repeat(31)),
            format!("demiurge:1757800000:{nonce}:extra"),
            "a transaction payload".to_string(),
        ] {
            assert_eq!(
                challenge_message(&bad).unwrap_err().kind(),
                "auth",
                "{bad:?} must be refused"
            );
        }
    }

    #[test]
    fn endpoint_joins_paths_without_doubling_slashes() {
        let client = IdentityClient::new("https://demiurge.cloud/api/v1/").unwrap();
        assert_eq!(
            client.url("/auth/login"),
            "https://demiurge.cloud/api/v1/auth/login"
        );
        assert_eq!(
            client.url("auth/login"),
            "https://demiurge.cloud/api/v1/auth/login"
        );
    }

    #[test]
    fn device_id_is_stable_within_a_run() {
        assert_eq!(device_id(), device_id());
        assert!(device_id().starts_with("qor-launcher:"));
    }

    #[tokio::test]
    async fn login_validates_input_before_the_network() {
        let client = IdentityClient::new("http://127.0.0.1:1/api/v1").unwrap();

        assert_eq!(
            client.login("", "password").await.unwrap_err().kind(),
            "auth"
        );
        assert_eq!(
            client.login("architect", "").await.unwrap_err().kind(),
            "auth"
        );
        assert!(client.session().is_none());
    }

    /// The refusal QOR ID sends, its shape captured from a running instance. It
    /// reached the person as raw JSON before the nested shape was read. The
    /// message is the service's wording since 2026-09-20 (`c43f81f`).
    #[test]
    fn service_refusals_are_read_in_its_own_words() {
        let status = reqwest::StatusCode::BAD_REQUEST;
        let nested = json!({
            "error": { "code": "VALIDATION_ERROR", "message": "This account already holds a QOR ID" }
        });
        assert_eq!(
            error_detail(
                &nested,
                &nested.to_string(),
                "auth/keypair-register",
                status
            ),
            ALREADY_REGISTERED
        );

        let flat = json!({ "error": "Username already taken" });
        assert_eq!(
            error_detail(&flat, "", "x", status),
            "Username already taken"
        );
        let message = json!({ "message": "Invalid or expired challenge" });
        assert_eq!(
            error_detail(&message, "", "x", status),
            "Invalid or expired challenge"
        );

        assert_eq!(
            error_detail(&serde_json::Value::Null, "  ", "profile", status),
            "profile returned HTTP 400 Bad Request"
        );
        let unreadable = "x".repeat(500);
        assert_eq!(
            error_detail(&serde_json::Value::Null, &unreadable, "profile", status).len(),
            200
        );
    }

    #[test]
    fn claiming_with_a_registered_key_says_to_sign_in() {
        let guided = guide_already_registered(QorError::Auth(ALREADY_REGISTERED.into()));
        assert!(
            guided.to_string().contains("sign in with your vault key"),
            "{guided}"
        );

        let other = guide_already_registered(QorError::Auth("Username already taken".into()));
        assert_eq!(
            other.to_string(),
            QorError::Auth("Username already taken".into()).to_string()
        );
        assert_eq!(
            guide_already_registered(QorError::Declined).kind(),
            "declined"
        );
    }

    /// The two refusals above are matched by their exact text, so the launcher's
    /// copies must be what the service actually sends. They drifted once: QOR ID
    /// reworded both on 2026-09-20 (`c43f81f`), and a brand-new vault's first
    /// sign-in showed the raw refusal instead of asking for a name. This reads the
    /// service's own handler, so a rewording on either side fails here.
    #[test]
    fn the_refusals_matched_here_are_the_ones_the_service_sends() {
        let handlers = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../services/qor-auth/src/handlers/auth.rs");
        let source = std::fs::read_to_string(&handlers).expect("QOR ID's auth handlers");
        for text in [NO_ACCOUNT, ALREADY_REGISTERED] {
            assert!(
                source.contains(&format!("\"{text}\"")),
                "QOR ID no longer sends {text:?} ({}); match what it sends now",
                handlers.display()
            );
        }
    }

    /// Unlocking goes straight to choosing a name only for this exact refusal;
    /// anything else is reported, not mistaken for a new key.
    #[test]
    fn only_a_key_without_an_account_is_sent_to_choose_a_name() {
        assert!(needs_registration(&QorError::Auth(NO_ACCOUNT.into())));
        assert!(!needs_registration(&QorError::Auth(
            "Account is not active".into()
        )));
        assert!(!needs_registration(&QorError::Network("timed out".into())));
        assert!(!needs_registration(&QorError::Declined));
    }
}
