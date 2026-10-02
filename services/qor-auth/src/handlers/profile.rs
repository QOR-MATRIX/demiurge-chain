//! Profile management handlers.

use axum::{
    Json,
    extract::{Extension, Path, State},
    http::StatusCode,
};
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::handlers::auth::{
    ChainAccount, VERIFICATION_RESEND_COOLDOWN_MINUTES, VERIFICATION_RESEND_DAILY_LIMIT,
};
use crate::middleware::auth::CurrentSession;
use crate::models::user::{User, UserStatus};
use crate::models::{ChangeEmailRequest, RegenerateBackupCodesRequest};
use crate::services::SessionService;
use crate::services::auth_service::AuthService;
use crate::state::AppState;
use sqlx;

/// Get current user's profile
/// Extracts user_id from auth middleware and fetches real profile data
pub async fn get_profile(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> AppResult<Json<Value>> {
    // user_id is extracted from JWT by auth middleware
    // Fetch user data from database
    let user: User = sqlx::query_as(
        r#"
        SELECT id, email, username, discriminator, password_hash, email_verified,
               avatar_url, role, status, chain_account_id, login_attempts, locked_until,
               created_at, updated_at, email_verification_token,
               email_verification_expires_at, auth_method,
               account_type, controller_id, agent_did, agent_capabilities,
               agent_autonomy, agent_spending_limit, agent_model
        FROM users WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(&state.db)
    .await
    .map_err(|_| AppError::NotFound("User not found".into()))?;

    // Format QOR ID: username#discriminator
    let qor_id = format!("{}#{:04}", user.username, user.discriminator);

    let chain_account = user
        .chain_account_id
        .as_deref()
        .map(ChainAccount::from_stored)
        .transpose()?;

    // False when the address bounced permanently or its owner complained: nothing is sent to it,
    // so it recovers nothing, and the account should move to another address.
    let email_deliverable = match user.email.as_deref() {
        Some(email) => Some(!state.email_service.is_suppressed(email).await?),
        None => None,
    };

    Ok(Json(json!({
        "id": user.id.to_string(),
        "qor_id": qor_id,
        "email": user.email,
        "email_deliverable": email_deliverable,
        "display_name": user.username, // Use username as display name for now
        "avatar_url": user.avatar_url,
        "role": user.role,
        "status": user.status,
        "created_at": user.created_at,
        // No balance: QOR ID reads no chain, so it has no balance to report. This
        // returned a fixed "0.00" for every account.
        //
        // `null` until the account has proven a key (ADR-017). It is not an
        // omission: an account with no proven key has no chain identity, and
        // showing a derived one is what that decision removed.
        "on_chain": chain_account
            .map(|a| a.as_json(state.config.chain.ss58_prefix))
            .unwrap_or(Value::Null),
        "account_type": user.account_type,
        "auth_method": user.auth_method
    })))
}

/// Update the caller's profile: not implemented.
/// POST /api/v1/profile
///
/// This answered "Profile updated successfully" and changed nothing. There is no
/// display name to store, and changing an email address needs verifying again, so
/// it refuses until that is designed.
pub async fn update_profile(State(_state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    Err(AppError::NotImplemented(
        "Profile updates are not implemented; nothing was changed".into(),
    ))
}

/// Upload an avatar: not implemented.
/// POST /api/v1/profile/avatar
///
/// This answered "Avatar uploaded successfully. Minting as DRC-369 NFT...", but
/// stored nothing, minted nothing, and took the account from the form rather than
/// the access token. It refuses until there is somewhere to store images.
pub async fn upload_avatar(State(_state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    Err(AppError::NotImplemented(
        "Avatar upload is not implemented; nothing was stored or minted".into(),
    ))
}

/// List the caller's live sessions
/// GET /api/v1/profile/sessions
///
/// Only fields that are true today are listed. `last_used_at` is when the session
/// was signed in to or last minted new tokens with its refresh token, whichever is
/// later. It is not moved by every request, so it trails real use by up to one
/// access token's lifetime. A session's IP address is not taken from the request,
/// so it is not shown.
pub async fn list_sessions(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Extension(current): Extension<CurrentSession>,
) -> AppResult<Json<Value>> {
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let sessions: Vec<Value> = session_service
        .list_sessions(user_id)
        .await?
        .iter()
        .map(|session| {
            json!({
                "session_id": session.session_id,
                "device_id": session.device_id,
                "created_at": session.created_at,
                "last_used_at": session.last_activity,
                "expires_at": session.expires_at,
                "current": session.session_id == current.0,
            })
        })
        .collect();

    Ok(Json(json!({ "sessions": sessions })))
}

/// Revoke one of the caller's sessions
/// DELETE /api/v1/profile/sessions/{id}
///
/// Deletes the session, so its access and refresh tokens stop working at once.
/// Another user's session and one that does not exist are both 404.
pub async fn revoke_session(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Path(session_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    if session_service.revoke_session(user_id, session_id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound("Session not found".into()))
    }
}

/// Issue a new set of backup codes
/// POST /api/v1/profile/backup-codes
///
/// For an account without an email address, which recovers a forgotten password
/// with its backup codes. It needs two proofs: the access token, and the account's
/// password in the body. A stolen access token alone could otherwise mint recovery
/// codes, reset the password and keep the account. A wrong password counts as a
/// failed sign-in attempt, and a locked or inactive account is refused.
///
/// Every old code, spent or not, is deleted and ten new ones stored in one
/// transaction that first locks the account row: there is no moment when old and
/// new codes both work, and two requests at once cannot leave twenty. The new codes
/// are returned once; only their hashes are kept. It is not reachable without
/// signing in, so it is no route around a forgotten password.
pub async fn regenerate_backup_codes(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<RegenerateBackupCodesRequest>,
) -> AppResult<Json<Value>> {
    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidToken)?;

    // An account that registered with an email holds no codes and recovers by email.
    // One that added an address later keeps its codes, and may renew them.
    let holds_codes: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM backup_codes WHERE user_id = $1)")
            .bind(user.id)
            .fetch_one(&state.db)
            .await?;
    if user.email.is_some() && !holds_codes {
        return Err(AppError::ValidationError(
            "This account recovers a forgotten password by email, and holds no backup codes to regenerate.".into(),
        ));
    }

    confirm_password(&state, &user, &req.password).await?;

    let codes = AuthService::generate_backup_codes();
    let mut tx = state.db.begin().await?;
    sqlx::query("SELECT id FROM users WHERE id = $1 FOR UPDATE")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM backup_codes WHERE user_id = $1")
        .bind(user.id)
        .execute(&mut *tx)
        .await?;
    for code in &codes {
        sqlx::query("INSERT INTO backup_codes (user_id, code_hash) VALUES ($1, $2)")
            .bind(user.id)
            .bind(AuthService::hash_backup_code(code))
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, details) VALUES ($1, 'backup_codes_regenerated', $2)",
    )
    .bind(user.id)
    .bind(json!({ "count": codes.len() }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;

    Ok(Json(json!({
        "backup_codes": codes,
        "message": "New backup codes issued. Every earlier code has stopped working; each new one resets your password once.",
    })))
}

/// The second proof for changes a stolen access token alone must not be able to make: the
/// account's password. A wrong password counts as a failed sign-in attempt, and a locked or
/// inactive account is refused whatever password is given.
async fn confirm_password(state: &AppState, user: &User, password: &str) -> AppResult<()> {
    let password_ok = AuthService::verify_password(password, &user.password_hash)?;
    let open = !user.is_locked() && user.status == UserStatus::Active;
    if password_ok && open {
        return Ok(());
    }
    if !password_ok && open {
        AuthService::new(state.db.clone())
            .increment_login_attempts(
                user.id,
                state.config.security.max_login_attempts,
                state.config.security.lockout_duration_secs,
            )
            .await?;
    }
    Err(AppError::InvalidCredentials)
}

/// Add an email address, or change the current one
/// POST /api/v1/profile/email
///
/// Puts an account without an email address on the recoverable path, or moves an
/// account to a new address. It needs the access token and the account's password,
/// as regenerating backup codes does: a stolen access token alone must not be able
/// to attach an attacker's address.
///
/// The new address is held as pending until the link sent to it is followed. Until
/// then the account is unchanged: an account without an email keeps only its backup
/// codes, and one with an address keeps that address for sign-in and recovery.
/// When there is a current address, it is told a change was requested, with no
/// link, before the change can complete. Confirming touches nothing else, so
/// backup codes are kept and the account ends with both routes. A password reset
/// cancels a pending change.
///
/// A new request replaces a pending one. Sending counts against the verification
/// limit of one message every 5 minutes and 5 in any 24 hours. If either message
/// cannot be sent, the pending change is withdrawn, except that a current address
/// marked undeliverable is not told: a dead address is the usual reason to change
/// it. A new address marked undeliverable is refused. Refused with 503 when email
/// is not configured.
pub async fn request_email_change(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<ChangeEmailRequest>,
) -> AppResult<Json<Value>> {
    if !state.email_service.is_configured() {
        return Err(AppError::ServiceUnavailable(
            "Adding or changing an email address is unavailable, because email is not configured on this service.".into(),
        ));
    }

    let new_email = req.email.trim().to_string();
    if !new_email.contains('@') || !new_email.contains('.') {
        return Err(AppError::ValidationError("Invalid email format".into()));
    }

    let user: User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidToken)?;

    confirm_password(&state, &user, &req.password).await?;

    if user
        .email
        .as_deref()
        .is_some_and(|current| current.eq_ignore_ascii_case(&new_email))
    {
        return Err(AppError::ValidationError(
            "That is already this account's email address".into(),
        ));
    }
    let taken: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM users WHERE LOWER(email) = LOWER($1) AND id <> $2 LIMIT 1",
    )
    .bind(&new_email)
    .bind(user.id)
    .fetch_optional(&state.db)
    .await?;
    if taken.is_some() {
        return Err(AppError::ValidationError("Email already registered".into()));
    }
    // Checked before the send is counted, so a dead address does not use up the limit.
    if state.email_service.is_suppressed(&new_email).await? {
        return Err(AppError::Undeliverable(
            "That address does not accept email from this service. Use a different address.".into(),
        ));
    }

    // One statement checks the limit, records the send and stores the pending address.
    let token = AuthService::generate_verification_token();
    let requested: Option<(Option<String>, String)> = sqlx::query_as(
        r#"
        UPDATE users SET
            pending_email = $2,
            pending_email_token = $3,
            pending_email_expires_at = NOW() + INTERVAL '24 hours',
            email_verification_sent_at = NOW(),
            email_verification_send_count = CASE
                WHEN email_verification_count_since IS NULL
                  OR email_verification_count_since <= NOW() - INTERVAL '24 hours' THEN 1
                ELSE email_verification_send_count + 1
            END,
            email_verification_count_since = CASE
                WHEN email_verification_count_since IS NULL
                  OR email_verification_count_since <= NOW() - INTERVAL '24 hours' THEN NOW()
                ELSE email_verification_count_since
            END,
            updated_at = NOW()
        WHERE id = $1
          AND (email_verification_sent_at IS NULL
               OR email_verification_sent_at <= NOW() - make_interval(mins => $4))
          AND (email_verification_count_since IS NULL
               OR email_verification_count_since <= NOW() - INTERVAL '24 hours'
               OR email_verification_send_count < $5)
        RETURNING email, username
        "#,
    )
    .bind(user.id)
    .bind(&new_email)
    .bind(&token)
    .bind(VERIFICATION_RESEND_COOLDOWN_MINUTES)
    .bind(VERIFICATION_RESEND_DAILY_LIMIT)
    .fetch_optional(&state.db)
    .await?;
    let Some((current_email, username)) = requested else {
        return Err(AppError::RateLimited(
            "A verification link can be sent once every 5 minutes, and 5 times a day.".into(),
        ));
    };

    // The current address hears first. If either message cannot be sent, the change is withdrawn,
    // except that a current address marked undeliverable is skipped.
    let sent: AppResult<bool> = async {
        let mut notified = false;
        if let Some(current) = current_email.as_deref() {
            match state
                .email_service
                .send_email_change_notice(current, &username)
                .await
            {
                Ok(()) => notified = true,
                Err(AppError::Undeliverable(_)) => {}
                Err(e) => return Err(e),
            }
        }
        state
            .email_service
            .send_email_change_verification(&new_email, &username, &token)
            .await?;
        Ok(notified)
    }
    .await;
    let notified = match sent {
        Ok(notified) => notified,
        Err(e) => {
            sqlx::query(
                "UPDATE users SET pending_email = NULL, pending_email_token = NULL, pending_email_expires_at = NULL WHERE id = $1 AND pending_email_token = $2",
            )
            .bind(user.id)
            .bind(&token)
            .execute(&state.db)
            .await?;
            return Err(e);
        }
    };

    Ok(Json(json!({
        "message": "A link was sent to the new address. The change completes when it is followed; until then the account does not change.",
        "pending_email": new_email,
        "current_email_notified": notified,
    })))
}

/// Link an on-chain key to the authenticated account
/// POST /api/v1/profile/link-wallet
///
/// Identical to `POST /api/v1/auth/link-keypair`. The request carries the public
/// key, a challenge issued for it and the key's signature over that challenge.
/// The key is bound to the account named by the access token.
pub async fn link_wallet(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<crate::models::LinkKeypairRequest>,
) -> AppResult<Json<Value>> {
    crate::handlers::auth::link_verified_key(&state, user_id, &req)
        .await
        .map(Json)
}

/// Adding and changing an email address, against a real Postgres and a stand-in for
/// Resend. The test of a password reset also needs Redis.
#[cfg(test)]
mod email_change_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::handlers::auth::{VerifyEmailRequest, reset_password_with_backup, verify_email};
    use crate::models::{RegisterRequest, ResetPasswordWithBackupRequest};
    use crate::services::{EmailConfig, EmailService};
    use axum::{Json as AxumJson, Router, extract::State as AxumState, routing::post};
    use sqlx::PgPool;
    use std::sync::Mutex;

    const PASSWORD: &str = "correct horse battery staple";

    type Inbox = Arc<Mutex<Vec<Value>>>;

    /// A stand-in for Resend's `POST /emails` that keeps every message.
    async fn fake_resend() -> (String, Inbox) {
        let inbox: Inbox = Arc::default();
        let app = Router::new()
            .route(
                "/emails",
                post(
                    |AxumState(inbox): AxumState<Inbox>, AxumJson(body): AxumJson<Value>| async move {
                        inbox.lock().expect("lock").push(body);
                        AxumJson(json!({ "id": "email-1" }))
                    },
                ),
            )
            .with_state(inbox.clone());
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let url = format!("http://{}", listener.local_addr().expect("address"));
        tokio::spawn(async move { axum::serve(listener, app).await.expect("serve") });
        (url, inbox)
    }

    fn state_with(db: PgPool, api_url: Option<String>) -> Arc<AppState> {
        let (key, from) = if api_url.is_some() {
            ("re_test_key", "Test <noreply@example.invalid>")
        } else {
            ("", "")
        };
        let email = EmailService::new(EmailConfig {
            resend_api_key: key.into(),
            from: from.into(),
            base_url: "https://example.invalid".into(),
            api_url: api_url.unwrap_or_else(|| "https://api.resend.com".into()),
        });
        let redis_url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        let redis = deadpool_redis::Config::from_url(redis_url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(AppConfig::default(), db, redis, email))
    }

    async fn register(state: &Arc<AppState>, username: &str) -> (Uuid, Vec<String>) {
        let (_, Json(body)) = crate::handlers::auth::register(
            State(state.clone()),
            Json(RegisterRequest {
                email: None,
                password: PASSWORD.into(),
                username: username.into(),
            }),
        )
        .await
        .expect("registration");
        let id = body["user_id"]
            .as_str()
            .and_then(|id| Uuid::parse_str(id).ok())
            .expect("user id");
        let codes = body["backup_codes"]
            .as_array()
            .expect("codes")
            .iter()
            .map(|c| c.as_str().expect("code").to_string())
            .collect();
        (id, codes)
    }

    /// An account with a verified address, whose last verification message was long ago.
    async fn account_with_email(db: &PgPool, username: &str, email: &str) -> Uuid {
        let hash = AuthService::hash_password(PASSWORD).expect("hash");
        sqlx::query_scalar(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified) VALUES ($1, $2, 1, $3, TRUE) RETURNING id",
        )
        .bind(email)
        .bind(username)
        .bind(hash)
        .fetch_one(db)
        .await
        .expect("account")
    }

    async fn change(
        state: &Arc<AppState>,
        user: Uuid,
        email: &str,
        password: &str,
    ) -> AppResult<Json<Value>> {
        request_email_change(
            State(state.clone()),
            Extension(user),
            Json(ChangeEmailRequest {
                email: email.into(),
                password: password.into(),
            }),
        )
        .await
    }

    async fn row(db: &PgPool, user: Uuid) -> (Option<String>, bool, Option<String>) {
        sqlx::query_as("SELECT email, email_verified, pending_email FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(db)
            .await
            .expect("row")
    }

    fn messages_to(inbox: &Inbox, address: &str) -> Vec<String> {
        inbox
            .lock()
            .expect("lock")
            .iter()
            .filter(|m| m["to"] == json!([address]))
            .map(|m| m["html"].as_str().unwrap_or_default().to_string())
            .collect()
    }

    fn token_in(html: &str) -> String {
        html.split("verify-email?token=")
            .nth(1)
            .and_then(|rest| rest.split(|c: char| !c.is_ascii_hexdigit()).next())
            .expect("a link")
            .to_string()
    }

    async fn verify(state: &Arc<AppState>, token: String) -> AppResult<Json<Value>> {
        verify_email(State(state.clone()), Json(VerifyEmailRequest { token })).await
    }

    async fn unused_codes(db: &PgPool, user: Uuid) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM backup_codes WHERE user_id = $1 AND used_at IS NULL",
        )
        .bind(user)
        .fetch_one(db)
        .await
        .expect("count")
    }

    #[sqlx::test]
    async fn without_email_configured_the_request_is_refused(db: PgPool) {
        let state = state_with(db.clone(), None);
        let (user, _) = register(&state, "noservice").await;
        assert!(matches!(
            change(&state, user, "added@example.invalid", PASSWORD).await,
            Err(AppError::ServiceUnavailable(_))
        ));
        assert_eq!(row(&db, user).await, (None, true, None));
    }

    #[sqlx::test]
    async fn a_wrong_password_is_refused_and_attaches_nothing(db: PgPool) {
        let (url, inbox) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        let (user, _) = register(&state, "wrongpw").await;

        assert!(matches!(
            change(&state, user, "attacker@example.invalid", "not the password").await,
            Err(AppError::InvalidCredentials)
        ));
        assert_eq!(row(&db, user).await, (None, true, None));
        assert!(inbox.lock().expect("lock").is_empty());
        let attempts: i32 = sqlx::query_scalar("SELECT login_attempts FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db)
            .await
            .expect("attempts");
        assert_eq!(attempts, 1);
    }

    #[sqlx::test]
    async fn an_account_without_an_email_adds_one_and_keeps_its_backup_codes(db: PgPool) {
        let (url, inbox) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        let (user, _) = register(&state, "adding").await;

        let Json(body) = change(&state, user, "added@example.invalid", PASSWORD)
            .await
            .expect("requested");
        assert_eq!(body["current_email_notified"], json!(false));
        // Until the link is followed, the account is unchanged.
        assert_eq!(
            row(&db, user).await,
            (None, true, Some("added@example.invalid".into()))
        );
        let sent = messages_to(&inbox, "added@example.invalid");
        assert_eq!(sent.len(), 1);

        let Json(_) = verify(&state, token_in(&sent[0])).await.expect("confirmed");
        assert_eq!(
            row(&db, user).await,
            (Some("added@example.invalid".into()), true, None)
        );
        assert_eq!(
            unused_codes(&db, user).await,
            10,
            "confirming must keep every backup code"
        );
    }

    #[sqlx::test]
    async fn changing_an_address_warns_the_old_one_and_keeps_it_until_the_new_one_is_confirmed(
        db: PgPool,
    ) {
        let (url, inbox) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        let user = account_with_email(&db, "moving", "old@example.invalid").await;

        let Json(body) = change(&state, user, "new@example.invalid", PASSWORD)
            .await
            .expect("requested");
        assert_eq!(body["current_email_notified"], json!(true));

        let notice = messages_to(&inbox, "old@example.invalid");
        assert_eq!(notice.len(), 1, "the old address is told");
        assert!(!notice[0].contains("token="), "the notice carries no link");
        assert_eq!(
            row(&db, user).await,
            (
                Some("old@example.invalid".into()),
                true,
                Some("new@example.invalid".into())
            )
        );

        let confirmation = messages_to(&inbox, "new@example.invalid");
        let Json(_) = verify(&state, token_in(&confirmation[0]))
            .await
            .expect("confirmed");
        assert_eq!(
            row(&db, user).await,
            (Some("new@example.invalid".into()), true, None)
        );
    }

    #[sqlx::test]
    async fn a_second_request_within_five_minutes_is_refused_and_changes_nothing(db: PgPool) {
        let (url, _) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        let (user, _) = register(&state, "hasty").await;

        let Json(_) = change(&state, user, "first@example.invalid", PASSWORD)
            .await
            .expect("first request");
        assert!(matches!(
            change(&state, user, "second@example.invalid", PASSWORD).await,
            Err(AppError::RateLimited(_))
        ));
        assert_eq!(
            row(&db, user).await,
            (None, true, Some("first@example.invalid".into()))
        );
    }

    #[sqlx::test]
    async fn an_address_another_account_holds_is_refused(db: PgPool) {
        let (url, _) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        account_with_email(&db, "holder", "held@example.invalid").await;
        let (user, _) = register(&state, "wanting").await;

        assert!(matches!(
            change(&state, user, "HELD@example.invalid", PASSWORD).await,
            Err(AppError::ValidationError(_))
        ));
        assert_eq!(row(&db, user).await, (None, true, None));
    }

    #[sqlx::test]
    async fn if_a_message_cannot_be_sent_the_change_is_withdrawn(db: PgPool) {
        // Configured, but nothing listens at the API address.
        let state = state_with(db.clone(), Some("http://127.0.0.1:9".into()));
        let (user, _) = register(&state, "unsent").await;

        assert!(
            change(&state, user, "added@example.invalid", PASSWORD)
                .await
                .is_err()
        );
        assert_eq!(row(&db, user).await, (None, true, None));
    }

    #[sqlx::test]
    async fn an_account_that_added_an_address_can_still_regenerate_its_codes(db: PgPool) {
        let (url, inbox) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        let (user, _) = register(&state, "bothroutes").await;
        let Json(_) = change(&state, user, "both@example.invalid", PASSWORD)
            .await
            .expect("requested");
        let sent = messages_to(&inbox, "both@example.invalid");
        let Json(_) = verify(&state, token_in(&sent[0])).await.expect("confirmed");

        let Json(body) = regenerate_backup_codes(
            State(state),
            Extension(user),
            Json(RegenerateBackupCodesRequest {
                password: PASSWORD.into(),
            }),
        )
        .await
        .expect("regenerated");
        assert_eq!(body["backup_codes"].as_array().map(Vec::len), Some(10));
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_password_reset_cancels_a_pending_change(db: PgPool) {
        let (url, inbox) = fake_resend().await;
        let state = state_with(db.clone(), Some(url));
        let (user, codes) = register(&state, "reclaimed").await;
        let Json(_) = change(&state, user, "attacker@example.invalid", PASSWORD)
            .await
            .expect("requested");
        let pending_token = token_in(&messages_to(&inbox, "attacker@example.invalid")[0]);

        let Json(_) = reset_password_with_backup(
            State(state.clone()),
            Json(ResetPasswordWithBackupRequest {
                username: "reclaimed".into(),
                backup_code: codes[0].clone(),
                new_password: "the owner's new password".into(),
            }),
        )
        .await
        .expect("reset");

        assert_eq!(row(&db, user).await, (None, true, None));
        assert!(matches!(
            verify(&state, pending_token).await,
            Err(AppError::ValidationError(_))
        ));
    }
}

/// Backup-code regeneration against a real Postgres. No test touches Redis.
#[cfg(test)]
mod backup_code_regeneration_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::models::RegisterRequest;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;
    use std::collections::HashSet;

    const PASSWORD: &str = "correct horse battery staple";

    fn state(db: PgPool) -> Arc<AppState> {
        let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:1")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        ))
    }

    /// Register through the real handler; returns the account id and its codes.
    async fn register(
        state: &Arc<AppState>,
        username: &str,
        email: Option<&str>,
    ) -> (Uuid, Vec<String>) {
        let (_, Json(body)) = crate::handlers::auth::register(
            State(state.clone()),
            Json(RegisterRequest {
                email: email.map(str::to_string),
                password: PASSWORD.into(),
                username: username.into(),
            }),
        )
        .await
        .expect("registration");
        let id = body["user_id"]
            .as_str()
            .and_then(|id| Uuid::parse_str(id).ok())
            .expect("user id");
        let codes = body["backup_codes"]
            .as_array()
            .map(|codes| {
                codes
                    .iter()
                    .filter_map(|c| c.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        (id, codes)
    }

    async fn stored_hashes(db: &PgPool, user: Uuid) -> HashSet<String> {
        sqlx::query_scalar::<_, String>("SELECT code_hash FROM backup_codes WHERE user_id = $1")
            .bind(user)
            .fetch_all(db)
            .await
            .expect("hashes")
            .into_iter()
            .collect()
    }

    fn hashes(codes: &[String]) -> HashSet<String> {
        codes
            .iter()
            .map(|c| AuthService::hash_backup_code(c))
            .collect()
    }

    fn with_password(password: &str) -> Json<RegenerateBackupCodesRequest> {
        Json(RegenerateBackupCodesRequest {
            password: password.into(),
        })
    }

    #[sqlx::test]
    async fn a_wrong_password_is_refused_keeps_the_old_codes_and_counts_as_a_failed_attempt(
        db: PgPool,
    ) {
        let state = state(db.clone());
        let (user, codes) = register(&state, "wrongpw", None).await;

        assert!(matches!(
            regenerate_backup_codes(
                State(state),
                Extension(user),
                with_password("not the password")
            )
            .await,
            Err(AppError::InvalidCredentials)
        ));
        assert_eq!(stored_hashes(&db, user).await, hashes(&codes));
        let attempts: i32 = sqlx::query_scalar("SELECT login_attempts FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db)
            .await
            .expect("attempts");
        assert_eq!(attempts, 1);
    }

    #[sqlx::test]
    async fn the_password_replaces_every_old_code_with_ten_new_ones(db: PgPool) {
        let state = state(db.clone());
        let (user, old) = register(&state, "renew", None).await;
        // One old code already spent: it must go too.
        sqlx::query(
            "UPDATE backup_codes SET used_at = NOW() WHERE user_id = $1 AND code_hash = $2",
        )
        .bind(user)
        .bind(AuthService::hash_backup_code(&old[0]))
        .execute(&db)
        .await
        .expect("spend one");

        let Json(body) =
            regenerate_backup_codes(State(state), Extension(user), with_password(PASSWORD))
                .await
                .expect("regenerated");
        let new: Vec<String> = body["backup_codes"]
            .as_array()
            .expect("codes")
            .iter()
            .map(|c| c.as_str().expect("code").to_string())
            .collect();
        assert_eq!(new.len(), AuthService::BACKUP_CODE_COUNT);

        let stored = stored_hashes(&db, user).await;
        assert_eq!(stored, hashes(&new), "exactly the new codes are stored");
        assert!(stored.is_disjoint(&hashes(&old)), "no old code survives");
        let unused: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM backup_codes WHERE user_id = $1 AND used_at IS NULL",
        )
        .bind(user)
        .fetch_one(&db)
        .await
        .expect("count");
        assert_eq!(unused, 10);
        let audited: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_log WHERE user_id = $1 AND action = 'backup_codes_regenerated'",
        )
        .bind(user)
        .fetch_one(&db)
        .await
        .expect("audit");
        assert_eq!(audited, 1);
    }

    #[sqlx::test]
    async fn an_account_with_an_email_address_is_refused(db: PgPool) {
        let state = state(db.clone());
        let (user, codes) = register(&state, "hasemail", Some("hasemail@example.invalid")).await;
        assert!(codes.is_empty());

        assert!(matches!(
            regenerate_backup_codes(State(state), Extension(user), with_password(PASSWORD)).await,
            Err(AppError::ValidationError(_))
        ));
        assert!(stored_hashes(&db, user).await.is_empty());
    }

    #[sqlx::test]
    async fn a_locked_account_is_refused_even_with_the_password(db: PgPool) {
        let state = state(db.clone());
        let (user, codes) = register(&state, "locked", None).await;
        sqlx::query("UPDATE users SET locked_until = NOW() + INTERVAL '1 hour' WHERE id = $1")
            .bind(user)
            .execute(&db)
            .await
            .expect("lock");

        assert!(matches!(
            regenerate_backup_codes(State(state), Extension(user), with_password(PASSWORD)).await,
            Err(AppError::InvalidCredentials)
        ));
        assert_eq!(stored_hashes(&db, user).await, hashes(&codes));
    }

    #[sqlx::test]
    async fn two_requests_at_once_leave_exactly_one_set_of_ten(db: PgPool) {
        let state = state(db.clone());
        let (user, _) = register(&state, "racing", None).await;

        let (first, second) = tokio::join!(
            regenerate_backup_codes(
                State(state.clone()),
                Extension(user),
                with_password(PASSWORD)
            ),
            regenerate_backup_codes(
                State(state.clone()),
                Extension(user),
                with_password(PASSWORD)
            ),
        );
        let sets: Vec<HashSet<String>> = [first, second]
            .into_iter()
            .map(|answer| {
                let Json(body) = answer.expect("regenerated");
                body["backup_codes"]
                    .as_array()
                    .expect("codes")
                    .iter()
                    .map(|c| AuthService::hash_backup_code(c.as_str().expect("code")))
                    .collect()
            })
            .collect();

        let stored = stored_hashes(&db, user).await;
        assert_eq!(stored.len(), 10);
        assert!(
            stored == sets[0] || stored == sets[1],
            "the stored codes are one response's set"
        );
    }
}

#[cfg(test)]
mod session_use_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use axum::body::Body;
    use axum::http::Request;
    use chrono::{DateTime, Utc};
    use sqlx::PgPool;
    use tower::ServiceExt;

    const PASSWORD: &str = "correct horse battery staple";

    /// The whole service, as it is served. Sessions live in Redis, so these need one at
    /// `QOR_AUTH_TEST_REDIS_URL`; CI provides it and runs them with `--include-ignored`.
    fn service(db: PgPool) -> axum::Router {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL").expect("QOR_AUTH_TEST_REDIS_URL");
        let redis = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        crate::router(Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        )))
    }

    async fn call(
        app: &axum::Router,
        request: axum::http::request::Builder,
        bearer: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut request = request.header("content-type", "application/json");
        if let Some(bearer) = bearer {
            request = request.header("authorization", format!("Bearer {bearer}"));
        }
        let body = body.map_or_else(Body::empty, |body| Body::from(body.to_string()));
        let response = app
            .clone()
            .oneshot(request.body(body).expect("request"))
            .await
            .expect("response");
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        (
            status,
            serde_json::from_slice(&bytes).unwrap_or(Value::Null),
        )
    }

    async fn sign_in(app: &axum::Router) -> Value {
        let (status, tokens) = call(
            app,
            Request::post("/api/v1/auth/login"),
            None,
            Some(json!({ "identifier": "lastused", "password": PASSWORD })),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{tokens}");
        tokens
    }

    async fn refresh(app: &axum::Router, tokens: &Value) -> (StatusCode, Value) {
        call(
            app,
            Request::post("/api/v1/auth/refresh"),
            None,
            Some(json!({ "refresh_token": tokens["refresh_token"] })),
        )
        .await
    }

    /// The caller's sessions, oldest first, as the route lists them.
    async fn sessions(app: &axum::Router, tokens: &Value) -> Vec<Value> {
        let (status, body) = call(
            app,
            Request::get("/api/v1/profile/sessions"),
            tokens["access_token"].as_str(),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{body}");
        body["sessions"].as_array().expect("sessions").clone()
    }

    fn time(session: &Value, field: &str) -> DateTime<Utc> {
        session[field]
            .as_str()
            .and_then(|text| text.parse().ok())
            .unwrap_or_else(|| panic!("{field} should be a time, in {session}"))
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_refresh_moves_last_used_on_that_session_only_and_its_owner_sees_it(db: PgPool) {
        let app = service(db);
        let (status, body) = call(
            &app,
            Request::post("/api/v1/auth/register"),
            None,
            Some(json!({ "username": "lastused", "password": PASSWORD })),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "{body}");
        let first = sign_in(&app).await;
        let second = sign_in(&app).await;

        // At sign-in a session was last used when it was created.
        let listed = sessions(&app, &first).await;
        assert_eq!(listed.len(), 2);
        for session in &listed {
            assert_eq!(
                time(session, "last_used_at"),
                time(session, "created_at"),
                "{session}"
            );
        }

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        let before = Utc::now();
        let (status, refreshed) = refresh(&app, &first).await;
        assert_eq!(status, StatusCode::OK, "{refreshed}");
        let after = Utc::now();

        // The new access token belongs to the same session, and lists both.
        let listed = sessions(&app, &refreshed).await;
        assert_eq!(listed.len(), 2);
        let (used, untouched) = if listed[0]["current"] == json!(true) {
            (&listed[0], &listed[1])
        } else {
            (&listed[1], &listed[0])
        };
        assert_eq!(used["current"], json!(true));
        assert_eq!(untouched["current"], json!(false));
        let last_used = time(used, "last_used_at");
        assert!(
            last_used >= before && last_used <= after,
            "the refreshed session was last used at the refresh: {used}"
        );
        assert!(last_used > time(used, "created_at"));
        assert_eq!(
            time(untouched, "last_used_at"),
            time(untouched, "created_at"),
            "the other session was not used: {untouched}"
        );

        // What is listed is these fields and no others: no address, no user agent (neither is
        // recorded), and nothing of a token.
        for session in &listed {
            let mut keys: Vec<&str> = session
                .as_object()
                .expect("an object")
                .keys()
                .map(String::as_str)
                .collect();
            keys.sort_unstable();
            assert_eq!(
                keys,
                [
                    "created_at",
                    "current",
                    "device_id",
                    "expires_at",
                    "last_used_at",
                    "session_id"
                ]
            );
        }

        // A revoked session's refresh token mints nothing, and does not bring the session back.
        let revoked = untouched["session_id"].as_str().expect("id");
        let (status, _) = call(
            &app,
            Request::delete(format!("/api/v1/profile/sessions/{revoked}")),
            refreshed["access_token"].as_str(),
            None,
        )
        .await;
        assert_eq!(status, StatusCode::NO_CONTENT);
        let (status, body) = refresh(&app, &second).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{body}");
        assert!(body["access_token"].is_null());
        let listed = sessions(&app, &refreshed).await;
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0]["session_id"], used["session_id"]);
    }
}
