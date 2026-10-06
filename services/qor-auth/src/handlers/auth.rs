//! Authentication handlers.

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use chrono::{Duration, Utc};
use rand::Rng;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::error::{AppError, AppResult};
use crate::models::{
    ChallengeRequest, ChallengeResponse, ForgotPasswordRequest, KeypairLoginRequest,
    KeypairRegisterRequest, LinkKeypairRequest, LoginRequest, RegisterRequest,
    ResendVerificationRequest, ResetPasswordWithBackupRequest, ResetPasswordWithTokenRequest,
    TokenPair,
};
use crate::services::{auth_service::AuthService, session_service::SessionService};
use crate::state::AppState;

/// Register a new user
///
/// Creates the account and nothing else. Registration creates no CGT and calls no
/// chain: an identity service must never be able to trigger a mint (migration
/// inventory R-3).
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<(StatusCode, Json<Value>)> {
    // Validate username
    if !crate::models::QorId::is_valid_username(&req.username) {
        return Err(AppError::ValidationError(
            "Username must be 3-20 characters, alphanumeric and underscores only".into(),
        ));
    }

    // Validate password strength against the configured minimum. This was
    // previously a hard-coded 6, and the configured value was never read.
    let min = state.config.security.password_min_length;
    if req.password.len() < min {
        return Err(AppError::ValidationError(format!(
            "Safe word must be at least {min} characters"
        )));
    }

    // Validate email format if provided
    if let Some(ref email) = req.email
        && (!email.contains('@') || !email.contains('.'))
    {
        return Err(AppError::ValidationError("Invalid email format".into()));
    }

    let auth_service = AuthService::new(state.db.clone());
    let username_lower = req.username.to_lowercase();

    // Check if username already exists
    if let Some(_existing) = auth_service.find_by_username(&username_lower).await? {
        return Err(AppError::ValidationError("Username already taken".into()));
    }

    // Check if email already exists (if provided)
    if let Some(ref email) = req.email
        && auth_service.find_by_email(email).await?.is_some()
    {
        return Err(AppError::ValidationError("Email already registered".into()));
    }

    // One name per account (ADR-075): the discriminator is always 1, and the unique index on
    // LOWER(username) refuses a name taken between the check above and this insert.
    let discriminator: i16 = 1;

    // Hash password
    let password_hash = AuthService::hash_password(&req.password)?;

    // Backup codes for an account without an email address. Each resets the password
    // once, and only their hashes are stored (migration 014).
    let backup_codes = if req.email.is_none() {
        AuthService::generate_backup_codes()
    } else {
        Vec::new()
    };

    // Generate email verification token if email provided
    let (email_verification_token, email_verification_expires_at) = if req.email.is_some() {
        let token = AuthService::generate_verification_token();
        let expires_at = Utc::now() + Duration::hours(24);
        (Some(token.clone()), Some(expires_at))
    } else {
        (None, None)
    };

    // No chain account (ADR-017). Registration used to derive one by hashing
    // the name, the discriminator and the clock, which no key corresponds to:
    // anything sent to it could never be moved, while every interface showed it
    // as a real account. A password-only account has a chain account once, and
    // only once, it proves it holds a key, through `link-keypair`.

    // An email address starts unverified; only verify-email marks it verified. An
    // account with no email address has nothing to verify. This was written
    // inverted, marking every supplied email verified (migration 012).
    let email_verified = req.email.is_none();

    // A verification message is sent only when an address was given and email is
    // configured. Sending it counts towards the resend limit (migration 015).
    let sending_verification = req.email.is_some() && state.email_service.is_configured();

    // The account and its backup codes are written together.
    let mut tx = state.db.begin().await?;
    let user_id = sqlx::query_scalar::<_, uuid::Uuid>(
        r#"
        INSERT INTO users (
            email, username, discriminator, password_hash, 
            email_verified, email_verification_token, email_verification_expires_at,
            role, status,
            email_verification_sent_at, email_verification_send_count, email_verification_count_since
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'user', 'active',
                CASE WHEN $8 THEN NOW() END, CASE WHEN $8 THEN 1 ELSE 0 END, CASE WHEN $8 THEN NOW() END)
        RETURNING id
        "#,
    )
    .bind(&req.email)
    .bind(&username_lower)
    .bind(discriminator)
    .bind(&password_hash)
    .bind(email_verified)
    .bind(&email_verification_token)
    .bind(email_verification_expires_at)
    .bind(sending_verification)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        if matches!(&e, sqlx::Error::Database(db) if db.constraint() == Some("users_username_unique")) {
            AppError::ValidationError("Username already taken".into())
        } else {
            AppError::DatabaseError(e)
        }
    })?;

    for code in &backup_codes {
        sqlx::query("INSERT INTO backup_codes (user_id, code_hash) VALUES ($1, $2)")
            .bind(user_id)
            .bind(AuthService::hash_backup_code(code))
            .execute(&mut *tx)
            .await?;
    }
    tx.commit().await?;

    // Send a verification email if an address was given. With email not configured
    // none is sent, and the response says so rather than asking the user to check
    // their mail.
    let response = if let Some(ref email) = req.email {
        let message = if state.email_service.is_configured() {
            let email_service = state.email_service.clone();
            let email_clone = email.clone();
            let username_clone = username_lower.clone();
            let token_clone = email_verification_token.clone().unwrap_or_default();

            tokio::spawn(async move {
                if let Err(e) = email_service
                    .send_verification_email(&email_clone, &username_clone, &token_clone)
                    .await
                {
                    tracing::error!("A verification email could not be sent: {:?}", e);
                }
            });
            "Account created! Please verify your email."
        } else {
            "Account created. No verification email was sent, because email is not configured on this service."
        };

        json!({
            "qor_id": username_lower,
            "user_id": user_id,
            "email_verified": email_verified,
            "message": message
        })
    } else {
        json!({
            "qor_id": username_lower,
            "user_id": user_id,
            "backup_codes": backup_codes,
            "email_verified": email_verified,
            "message": "Account created! Save your backup codes: each one resets your password once."
        })
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Sign in with a username or email address and a password
/// POST /api/v1/auth/login
///
/// Every refusal is the same `InvalidCredentials` answer: an account that does not
/// exist, a wrong password, and an account that is locked or not active are not
/// told apart. Every attempt also does the same work before it is answered, so its
/// timing does not tell them apart either: one lookup, one Argon2 verification
/// (against a stand-in hash when no account matches), and one attempt-counter
/// update, which counts only a wrong password on an open account and otherwise
/// targets an id no account has.
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<TokenPair>> {
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let user = authenticate_password(&state, &req.identifier, &req.password).await?;

    // Create session
    let device_id = req.device_id.unwrap_or_else(|| "unknown".to_string());
    let (_session, tokens) = session_service
        .create_session(crate::services::session_service::NewSession {
            user_id: user.id,
            qor_id: &user.qor_id(),
            role: Some(user.role.as_str()),
            device_id: &device_id,
            ip_address: "0.0.0.0", // Not yet taken from the request
            user_agent: None,
            scopes: crate::models::Session::default_scopes(),
        })
        .await?;
    record_sign_in(&state.db, user.id, "password").await?;

    Ok(Json(tokens))
}

/// The account a name or address and a password sign in to, or `InvalidCredentials`.
///
/// Every refusal is the same answer after the same work: an unknown name is checked against a
/// stand-in hash, and a failed attempt is counted against a real account only when it is open. Both
/// sign-in routes use this, the JSON API and QOR ID's own page for other apps (`oauth`), so they
/// cannot drift apart.
pub(crate) async fn authenticate_password(
    state: &AppState,
    identifier: &str,
    password: &str,
) -> AppResult<crate::models::User> {
    let auth_service = AuthService::new(state.db.clone());

    let user = if AuthService::is_email(identifier) {
        auth_service.find_by_email(identifier).await?
    } else {
        auth_service.find_by_username(identifier).await?
    };

    let password_hash = user
        .as_ref()
        .map(|u| u.password_hash.as_str())
        .unwrap_or_else(|| AuthService::stand_in_password_hash());
    let password_ok = AuthService::verify_password(password, password_hash)?;

    let open =
        |u: &crate::models::User| !u.is_locked() && u.status == crate::models::UserStatus::Active;
    // Only a wrong password on an open account counts as a failed attempt. A lock is
    // not extended by attempts made while it holds, an account that is not active is
    // not counted against, and an unknown name has no row: those target an id no
    // account has, so the same update still runs.
    let counted = match &user {
        Some(u) if !password_ok && open(u) => u.id,
        _ => uuid::Uuid::new_v4(),
    };
    let user = match user {
        Some(u) if password_ok && open(&u) => u,
        _ => {
            auth_service
                .increment_login_attempts(
                    counted,
                    state.config.security.max_login_attempts,
                    state.config.security.lockout_duration_secs,
                )
                .await?;
            return Err(AppError::InvalidCredentials);
        }
    };

    // Reset login attempts on successful login
    auth_service.reset_login_attempts(user.id).await?;
    Ok(user)
}

/// Record a successful sign-in in the audit log. The admin statistics count these
/// rows (`logins_24h`); nothing else writes them.
pub(crate) async fn record_sign_in(
    db: &sqlx::PgPool,
    user_id: uuid::Uuid,
    method: &str,
) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_log (user_id, action, details) VALUES ($1, 'login', $2)")
        .bind(user_id)
        .bind(json!({ "method": method }))
        .execute(db)
        .await?;
    Ok(())
}

/// Refresh access token
/// POST /api/v1/auth/refresh
///
/// Mints new tokens for a session that still exists, and records on the session that it was used.
pub async fn refresh_token(
    State(state): State<Arc<AppState>>,
    Json(req): Json<crate::models::RefreshRequest>,
) -> AppResult<Json<TokenPair>> {
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());

    // Validate refresh token
    let claims = session_service.validate_refresh_token(&req.refresh_token)?;

    // Get session
    let session_id = uuid::Uuid::parse_str(&claims.sid).map_err(|_| AppError::InvalidToken)?;

    let session = session_service
        .get_session(session_id)
        .await?
        .ok_or(AppError::InvalidToken)?;

    if session.is_expired() {
        return Err(AppError::TokenExpired);
    }

    // An app's session refreshes only through `/oauth/token`, which rotates its refresh token and
    // detects a reused one (ADR-043 decision 2). Accepting it here would be a way around both.
    if session.client_id.is_some() {
        return Err(AppError::InvalidToken);
    }

    // The session was used now, which its owner sees in the list of sessions. A session revoked
    // since it was read above is not written back, and mints nothing.
    if !session_service.record_use(&session).await? {
        return Err(AppError::InvalidToken);
    }

    // Generate new tokens
    let tokens = session_service.generate_tokens(&session, claims.role.as_deref())?;

    Ok(Json(tokens))
}

/// Logout: revoke the session named by the access token
/// POST /api/v1/auth/logout
///
/// Deletes the session. Its refresh token can no longer mint access tokens, and
/// its access token is refused at once, because every access token is checked
/// against its session (`SessionService::authenticate_access_token`).
pub async fn logout(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> AppResult<StatusCode> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::InvalidToken)?;

    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let claims = session_service.authenticate_access_token(token).await?;
    let session_id = uuid::Uuid::parse_str(&claims.sid).map_err(|_| AppError::InvalidToken)?;
    let user_id = uuid::Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    session_service.delete_session(session_id, user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Verify email address
#[derive(serde::Deserialize)]
pub struct VerifyEmailRequest {
    pub token: String,
}

pub async fn verify_email(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyEmailRequest>,
) -> AppResult<Json<Value>> {
    // Find user by verification token
    let user: Option<crate::models::User> = sqlx::query_as::<_, crate::models::User>(
        r#"
        SELECT * FROM users 
        WHERE email_verification_token = $1 
        AND email_verification_expires_at > NOW()
        "#,
    )
    .bind(&req.token)
    .fetch_optional(&state.db)
    .await?;

    // A token that is not the account's own verification token may confirm an
    // address that was added or changed.
    let Some(user) = user else {
        return confirm_pending_email(&state, &req.token).await;
    };

    // Update user to verified
    sqlx::query(
        r#"
        UPDATE users 
        SET email_verified = TRUE, 
            email_verification_token = NULL,
            email_verification_expires_at = NULL
        WHERE id = $1
        "#,
    )
    .bind(user.id)
    .execute(&state.db)
    .await?;
    crate::handlers::progress::award(&state.db, user.id, "verify-email").await?;

    Ok(Json(json!({
        "message": "Email verified successfully",
        "qor_id": user.qor_id()
    })))
}

/// Confirm an added or changed address with the token that was sent to it.
///
/// The pending address becomes the account's verified address, and nothing else
/// changes: backup codes are kept, so an account that had them ends with both
/// recovery routes. If another account has registered the address meanwhile, the
/// confirmation is refused.
async fn confirm_pending_email(state: &AppState, token: &str) -> AppResult<Json<Value>> {
    let confirmed: Option<(uuid::Uuid, String)> = sqlx::query_as(
        r#"
        UPDATE users
        SET email = pending_email,
            email_verified = TRUE,
            email_verification_token = NULL,
            email_verification_expires_at = NULL,
            pending_email = NULL,
            pending_email_token = NULL,
            pending_email_expires_at = NULL,
            updated_at = NOW()
        WHERE pending_email_token = $1
          AND pending_email_expires_at > NOW()
          AND status = 'active'
        RETURNING id, username
        "#,
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| {
        if matches!(&e, sqlx::Error::Database(db) if db.is_unique_violation()) {
            AppError::ValidationError(
                "That email address is now registered to another account".into(),
            )
        } else {
            AppError::DatabaseError(e)
        }
    })?;

    let (confirmed_id, username) = confirmed.ok_or(AppError::ValidationError(
        "Invalid or expired verification token".into(),
    ))?;
    crate::handlers::progress::award(&state.db, confirmed_id, "verify-email").await?;
    Ok(Json(json!({
        "message": "Email verified successfully",
        "qor_id": username.to_lowercase(),
    })))
}

/// The answer `forgot-password` gives every identifier it accepts, so the response discloses nothing
/// about which accounts exist, which have an email address, or which are verified.
const FORGOT_PASSWORD_RESPONSE: &str = "If an account with a verified email address matches, a reset link will be sent to that address. An account without one resets with its backup code.";

/// Request a password reset by email
/// POST /api/v1/auth/forgot-password
///
/// With email not configured, the request is refused with 503 before any lookup:
/// no link could be delivered, and a link is never written anywhere else, such as
/// a log. Otherwise the answer is identical for every identifier, and the lookup
/// and sending happen after the response, so neither its body nor its timing
/// shows whether an account exists. A link goes only to a verified address, since
/// an unverified one was never shown to belong to the account holder.
pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<Json<Value>> {
    if !state.email_service.is_configured() {
        return Err(AppError::ServiceUnavailable(
            "Password reset by email is unavailable, because email is not configured on this service. An account without an email address resets with its backup code.".into(),
        ));
    }

    let identifier = req.identifier;
    tokio::spawn(async move {
        if let Err(e) = send_password_reset(&state, &identifier).await {
            tracing::error!("A password reset request could not be completed: {:?}", e);
        }
    });

    Ok(Json(json!({ "message": FORGOT_PASSWORD_RESPONSE })))
}

/// An account is sent a verification message at most once every this many minutes.
pub(crate) const VERIFICATION_RESEND_COOLDOWN_MINUTES: i32 = 5;

/// An account is sent at most this many verification messages in any 24 hours.
pub(crate) const VERIFICATION_RESEND_DAILY_LIMIT: i32 = 5;

/// The answer `resend-verification` gives every identifier it accepts, so the response discloses nothing
/// about which accounts exist, which are verified, or which have reached their limit.
const RESEND_VERIFICATION_RESPONSE: &str = "If an account with an unverified email address matches, a new verification link will be sent to that address, and any earlier link stops working. A link can be requested once every 5 minutes, and 5 times a day.";

/// Request a new email verification message
/// POST /api/v1/auth/resend-verification
///
/// For an account whose verification link lapsed or never arrived, such as one
/// that registered while email was not configured. A new token replaces the old
/// one, so an earlier link stops working.
///
/// Refused with 503 before any lookup when email is not configured. Otherwise the
/// answer is identical for every identifier, whether the account exists, is
/// already verified, has no email address or has reached its limit, and the lookup
/// and sending happen after the response. Each account is sent at most one message
/// every 5 minutes and 5 in any 24 hours, checked and recorded in the statement
/// that replaces the token.
pub async fn resend_verification(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResendVerificationRequest>,
) -> AppResult<Json<Value>> {
    if !state.email_service.is_configured() {
        return Err(AppError::ServiceUnavailable(
            "Verification email is unavailable, because email is not configured on this service."
                .into(),
        ));
    }

    let identifier = req.identifier;
    tokio::spawn(async move {
        if let Err(e) = send_new_verification(&state, &identifier).await {
            tracing::error!("A verification resend could not be completed: {:?}", e);
        }
    });

    Ok(Json(json!({ "message": RESEND_VERIFICATION_RESPONSE })))
}

/// Replace the verification token and send it, if `identifier` names an active
/// account with an unverified email address that is within its limit.
async fn send_new_verification(state: &AppState, identifier: &str) -> AppResult<()> {
    let auth_service = AuthService::new(state.db.clone());
    let user = if AuthService::is_email(identifier) {
        auth_service.find_by_email(identifier).await?
    } else {
        auth_service.find_by_username(identifier).await?
    };
    let Some(user) = user else {
        return Ok(());
    };

    // One statement checks the limit, records the send and replaces the token, so two
    // requests at once cannot both pass the limit.
    let token = AuthService::generate_verification_token();
    let issued: Option<(String, String)> = sqlx::query_as(
        r#"
        UPDATE users SET
            email_verification_token = $2,
            email_verification_expires_at = NOW() + INTERVAL '24 hours',
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
          AND email IS NOT NULL
          AND email_verified = FALSE
          AND status = 'active'
          AND (email_verification_sent_at IS NULL
               OR email_verification_sent_at <= NOW() - make_interval(mins => $3))
          AND (email_verification_count_since IS NULL
               OR email_verification_count_since <= NOW() - INTERVAL '24 hours'
               OR email_verification_send_count < $4)
        RETURNING email, username
        "#,
    )
    .bind(user.id)
    .bind(&token)
    .bind(VERIFICATION_RESEND_COOLDOWN_MINUTES)
    .bind(VERIFICATION_RESEND_DAILY_LIMIT)
    .fetch_optional(&state.db)
    .await?;

    let Some((email, username)) = issued else {
        return Ok(());
    };
    state
        .email_service
        .send_verification_email(&email, &username, &token)
        .await
}

/// Issue and send a reset link, if `identifier` names an active account with a verified email address.
async fn send_password_reset(state: &AppState, identifier: &str) -> AppResult<()> {
    let auth_service = AuthService::new(state.db.clone());
    let user = if AuthService::is_email(identifier) {
        auth_service.find_by_email(identifier).await?
    } else {
        auth_service.find_by_username(identifier).await?
    };

    let Some(user) = user else {
        return Ok(());
    };
    let Some(email) = user.email.as_deref() else {
        return Ok(());
    };
    if !user.email_verified || user.status != crate::models::UserStatus::Active {
        return Ok(());
    }

    let token = AuthService::generate_verification_token();
    let expires_at = Utc::now() + Duration::hours(1);
    sqlx::query("INSERT INTO password_resets (user_id, token, expires_at) VALUES ($1, $2, $3)")
        .bind(user.id)
        .bind(&token)
        .bind(expires_at)
        .execute(&state.db)
        .await?;

    state
        .email_service
        .send_password_reset_email(email, &user.username, &token)
        .await
}

/// Reset a password with a backup code (accounts without an email address)
/// POST /api/v1/auth/reset-password-backup
///
/// Each code works once. It is spent in the same transaction that sets the
/// password, and a spent code is refused. A wrong code, a spent code and an
/// unknown username all get the same answer. The response says how many codes
/// remain, and every session of the account ends (see `reset_password`).
pub async fn reset_password_with_backup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResetPasswordWithBackupRequest>,
) -> AppResult<Json<Value>> {
    // Validate password against the configured minimum
    let min = state.config.security.password_min_length;
    if req.new_password.len() < min {
        return Err(AppError::ValidationError(format!(
            "Safe word must be at least {min} characters"
        )));
    }

    let auth_service = AuthService::new(state.db.clone());
    let account = auth_service
        .find_by_username(&req.username.to_lowercase())
        .await?
        .map(|user| user.id);
    let code_hash = AuthService::hash_backup_code(&req.backup_code);

    let mut tx = state.db.begin().await?;
    // An unknown username runs the same statement against an id no account has,
    // so it is refused exactly as a wrong or spent code is.
    let spent: Option<uuid::Uuid> = sqlx::query_scalar(
        "UPDATE backup_codes SET used_at = NOW() WHERE user_id = $1 AND code_hash = $2 AND used_at IS NULL RETURNING user_id",
    )
    .bind(account.unwrap_or_else(uuid::Uuid::new_v4))
    .bind(&code_hash)
    .fetch_optional(&mut *tx)
    .await?;
    let Some(user_id) = spent else {
        return Err(AppError::InvalidCredentials);
    };

    let password_hash = AuthService::hash_password(&req.new_password)?;
    // A reset also cancels any pending change of email address, so someone who had
    // the password cannot complete one after the owner resets it.
    sqlx::query(
        "UPDATE users SET password_hash = $1, pending_email = NULL, pending_email_token = NULL, pending_email_expires_at = NULL, updated_at = NOW() WHERE id = $2",
    )
        .bind(&password_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM backup_codes WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    // Every session ends, revoked before the change commits (see `reset_password`).
    SessionService::new(state.redis.clone(), state.config.jwt.clone())
        .delete_all_sessions(user_id)
        .await?;
    tx.commit().await?;

    let message = if remaining == 0 {
        "Password reset successfully. No backup codes remain."
    } else {
        "Password reset successfully"
    };
    Ok(Json(json!({
        "message": message,
        "backup_codes_remaining": remaining,
    })))
}

/// Reset password with token (email-based accounts)
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ResetPasswordWithTokenRequest>,
) -> AppResult<Json<Value>> {
    // Validate password against the configured minimum
    let min = state.config.security.password_min_length;
    if req.new_password.len() < min {
        return Err(AppError::ValidationError(format!(
            "Safe word must be at least {min} characters"
        )));
    }

    // Find valid reset token
    let reset: Option<(uuid::Uuid, uuid::Uuid)> = sqlx::query_as(
        r#"
        SELECT user_id, id FROM password_resets
        WHERE token = $1 
        AND expires_at > NOW()
        AND used_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(&req.token)
    .fetch_optional(&state.db)
    .await?;

    let (user_id, reset_id) = reset.ok_or(AppError::ValidationError(
        "Invalid or expired reset token".into(),
    ))?;

    let password_hash = AuthService::hash_password(&req.new_password)?;

    // Spend the token and set the password in one transaction, as two statements.
    // They were sent as one prepared statement, which Postgres refuses (42601), so
    // every reset failed with 500. The token is spent only if it is still unused,
    // so two requests carrying the same token cannot both succeed.
    let mut tx = state.db.begin().await?;
    let spent =
        sqlx::query("UPDATE password_resets SET used_at = NOW() WHERE id = $1 AND used_at IS NULL")
            .bind(reset_id)
            .execute(&mut *tx)
            .await?;
    if spent.rows_affected() != 1 {
        return Err(AppError::ValidationError(
            "Invalid or expired reset token".into(),
        ));
    }
    // A reset also cancels any pending change of email address, so someone who had
    // the password cannot complete one after the owner resets it.
    sqlx::query(
        "UPDATE users SET password_hash = $1, pending_email = NULL, pending_email_token = NULL, pending_email_expires_at = NULL, updated_at = NOW() WHERE id = $2",
    )
        .bind(&password_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    // A reset usually means someone else had the account, so every session ends. They
    // are revoked before the change commits: if they cannot be, nothing is reset.
    SessionService::new(state.redis.clone(), state.config.jwt.clone())
        .delete_all_sessions(user_id)
        .await?;
    tx.commit().await?;

    Ok(Json(json!({
        "message": "Password reset successfully"
    })))
}

/// Check username availability
#[derive(serde::Deserialize)]
pub struct CheckUsernameRequest {
    username: String,
}

pub async fn check_username(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckUsernameRequest>,
) -> AppResult<Json<Value>> {
    // Validate username format
    if !crate::models::QorId::is_valid_username(&req.username) {
        return Ok(Json(json!({
            "available": false,
            "reason": "invalid_format"
        })));
    }

    // Check if username exists in database
    let username_lower = req.username.to_lowercase();
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE LOWER(username) = $1")
            .bind(&username_lower)
            .fetch_optional(&state.db)
            .await?;

    let available = exists.map(|count| count == 0).unwrap_or(true);

    Ok(Json(json!({
        "available": available,
        "username": username_lower,
    })))
}

/// Check email availability
#[derive(serde::Deserialize)]
pub struct CheckEmailRequest {
    email: String,
}

pub async fn check_email(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CheckEmailRequest>,
) -> AppResult<Json<Value>> {
    // Validate email format
    if !req.email.contains('@') || !req.email.contains('.') {
        return Ok(Json(json!({
            "available": false,
            "reason": "invalid_format"
        })));
    }

    // Check if email exists in database
    let email_lower = req.email.to_lowercase();
    let exists: Option<i64> =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE LOWER(email) = $1")
            .bind(&email_lower)
            .fetch_optional(&state.db)
            .await?;

    let available = exists.map(|count| count == 0).unwrap_or(true);

    Ok(Json(json!({
        "available": available,
        "email": email_lower,
    })))
}

// =============================================================================
// Keypair-Based Authentication
// =============================================================================

/// Issue a challenge for an account to sign.
/// GET /api/v1/auth/challenge?address=<SS58>
///
/// The account is named as SS58, or as `account_id` in hex for an advanced
/// caller (ADR-024). The challenge is bound to the account's bytes, so the form
/// it was asked for in cannot be used to get two challenges for one account.
pub async fn get_challenge(
    State(state): State<Arc<AppState>>,
    Query(req): Query<ChallengeRequest>,
) -> AppResult<Json<ChallengeResponse>> {
    let account = ChainAccount::from_request(
        req.address.as_deref(),
        req.account_id.as_deref(),
        state.config.chain.ss58_prefix,
    )?;

    // Generate random challenge
    let random_bytes: [u8; 32] = rand::thread_rng().r#gen();
    let random_hex = hex::encode(random_bytes);
    let timestamp = Utc::now().timestamp();
    let challenge = format!("demiurge:{}:{}", timestamp, random_hex);
    let expires_at = Utc::now() + Duration::minutes(5);

    sqlx::query(
        r#"
        INSERT INTO auth_challenges (chain_account_id, challenge, expires_at)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(account.as_bytes())
    .bind(&challenge)
    .bind(expires_at)
    .execute(&state.db)
    .await?;

    Ok(Json(ChallengeResponse {
        challenge,
        expires_at,
    }))
}

/// Domain tag for QOR ID challenge signatures (security track item 7).
///
/// A key signs `CHALLENGE_DOMAIN || challenge`, never the bare challenge. The
/// same key signs chain transactions, and without the tag a hostile or
/// compromised endpoint could ask for a signature over bytes that mean
/// something else. The launcher uses the identical tag
/// (`tools/qor-launcher/src-tauri/src/vault/derive.rs` signs it).
///
/// **What the tag does not do yet.** ADR-024 asks for the chain's genesis hash
/// and network name in the challenge, so that a signature proves the holder
/// signed for *this* network. That is not here: there is no deployed network
/// whose genesis hash could be configured, and a value invented now would be
/// wrong for the chain that is eventually run. It lands when the chain has an
/// identity to bind to; until then a challenge signature proves possession of a
/// key and says nothing about a network, which is what F-Q9 records.
pub(crate) const CHALLENGE_DOMAIN: &str = "demiurge:qor-id:challenge:v1:";

/// The exact bytes a key signs to answer `challenge`.
pub(crate) fn challenge_message(challenge: &str) -> String {
    format!("{CHALLENGE_DOMAIN}{challenge}")
}

// =============================================================================
// Chain accounts (ADR-023, ADR-024)
// =============================================================================

/// A chain account: the 32 bytes that identify it.
///
/// One account is one value here, whichever form it arrived in, so a lookup
/// cannot miss a row because the caller wrote the other form. The bytes are
/// what is stored; an SS58 string is a rendering at a prefix that is not
/// decided for mainnet, and Postgres can neither produce nor check one
/// (ADR-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ChainAccount([u8; 32]);

impl ChainAccount {
    /// Read an account from a request.
    ///
    /// `address` is SS58 at the chain's prefix, the form a person sees. Hex is
    /// accepted only in `account_id`, the explicitly advanced field, so an
    /// interface that means to take an address cannot quietly take a raw
    /// account ID instead. Exactly one of the two is given.
    pub(crate) fn from_request(
        address: Option<&str>,
        account_id: Option<&str>,
        ss58_prefix: u16,
    ) -> Result<Self, AppError> {
        let address = address.map(str::trim).filter(|v| !v.is_empty());
        let account_id = account_id.map(str::trim).filter(|v| !v.is_empty());

        match (address, account_id) {
            (Some(address), None) => Self::from_ss58(address, ss58_prefix),
            (None, Some(account_id)) => Self::from_hex(account_id),
            (Some(_), Some(_)) => Err(AppError::ValidationError(
                "Give either address or account_id, not both".into(),
            )),
            (None, None) => Err(AppError::ValidationError(
                "An account is required: address (SS58), or account_id in hex".into(),
            )),
        }
    }

    /// Decode an SS58 address, checksum and prefix included.
    ///
    /// **The prefix check is an input-shape check, not chain identification**
    /// (ADR-024, clarified 17 September 2026). It catches a Polkadot or Kusama
    /// address pasted by mistake. Prefix 42 is shared by many networks, so an
    /// address from another of them passes it, and what tells networks apart is
    /// the genesis hash.
    fn from_ss58(address: &str, ss58_prefix: u16) -> Result<Self, AppError> {
        use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};

        let (public, format) = sp_core::sr25519::Public::from_ss58check_with_version(address)
            .map_err(|_| {
                AppError::ValidationError(
                    "That is not an address: check it for a typo, since the last characters are a \
                     checksum"
                        .into(),
                )
            })?;

        if format != Ss58AddressFormat::custom(ss58_prefix) {
            return Err(AppError::ValidationError(format!(
                "That address is written for another network (prefix {}, this one uses {})",
                u16::from(format),
                ss58_prefix
            )));
        }

        Ok(Self(public.0))
    }

    /// Decode a raw account ID, with or without `0x`.
    fn from_hex(account_id: &str) -> Result<Self, AppError> {
        let body = account_id.strip_prefix("0x").unwrap_or(account_id);

        if body.len() != 64 || !body.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(AppError::ValidationError(
                "An account ID is 64 hexadecimal characters".into(),
            ));
        }

        let bytes = hex::decode(body)
            .map_err(|_| AppError::ValidationError("An account ID is hexadecimal".into()))?;

        Ok(Self(bytes.try_into().map_err(|_| {
            AppError::ValidationError("An account ID is 32 bytes".into())
        })?))
    }

    /// Read an account back from the column it is stored in.
    pub(crate) fn from_stored(bytes: &[u8]) -> Result<Self, AppError> {
        bytes.try_into().map(Self).map_err(|_| {
            AppError::InternalError(anyhow::anyhow!("a stored chain account is not 32 bytes"))
        })
    }

    /// The bytes, as the column holds them.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// The address a person sees (ADR-024).
    pub(crate) fn address(&self, ss58_prefix: u16) -> String {
        use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
        sp_core::sr25519::Public::from(self.0)
            .to_ss58check_with_version(Ss58AddressFormat::custom(ss58_prefix))
    }

    /// The raw account ID, for advanced views (ADR-024).
    pub(crate) fn account_id_hex(&self) -> String {
        format!("0x{}", hex::encode(self.0))
    }

    /// Both forms, as every response that names an account returns them.
    pub(crate) fn as_json(&self, ss58_prefix: u16) -> Value {
        json!({
            "address": self.address(ss58_prefix),
            "account_id": self.account_id_hex(),
        })
    }

    /// Verify a signature over a QOR ID challenge, with the domain tag applied.
    ///
    /// Sr25519 (ADR-023), verified by `sp-core` at the version the chain is
    /// built against (ADR-033 rule 1), under schnorrkel's `substrate` signing
    /// context. A signature over the bare challenge, or under any other
    /// context, does not verify.
    pub(crate) fn verify_challenge(&self, challenge: &str, signature: &str) -> bool {
        use sp_core::Pair as _;

        let Ok(bytes) = hex::decode(signature.strip_prefix("0x").unwrap_or(signature)) else {
            return false;
        };
        let Ok(bytes) = <[u8; 64]>::try_from(bytes.as_slice()) else {
            return false;
        };

        sp_core::sr25519::Pair::verify(
            &sp_core::sr25519::Signature::from(bytes),
            challenge_message(challenge).as_bytes(),
            &sp_core::sr25519::Public::from(self.0),
        )
    }
}

/// Prove possession of a key.
///
/// The caller must present a signature, by the account's key, over a live
/// challenge that was issued for exactly that account and has not been used.
/// The challenge is consumed atomically, so a signature can be spent once.
pub(crate) async fn consume_signed_challenge(
    db: &sqlx::PgPool,
    account: ChainAccount,
    challenge: &str,
    signature: &str,
) -> Result<(), AppError> {
    let record: Option<(uuid::Uuid, String)> = sqlx::query_as(
        r#"
        SELECT id, challenge FROM auth_challenges
        WHERE chain_account_id = $1 AND challenge = $2 AND expires_at > NOW() AND used = FALSE
        LIMIT 1
        "#,
    )
    .bind(account.as_bytes())
    .bind(challenge)
    .fetch_optional(db)
    .await?;

    let (challenge_id, challenge) = record.ok_or(AppError::ValidationError(
        "Invalid or expired challenge".into(),
    ))?;

    if !account.verify_challenge(&challenge, signature) {
        return Err(AppError::InvalidCredentials);
    }

    let consumed =
        sqlx::query("UPDATE auth_challenges SET used = TRUE WHERE id = $1 AND used = FALSE")
            .bind(challenge_id)
            .execute(db)
            .await?;

    if consumed.rows_affected() != 1 {
        return Err(AppError::ValidationError(
            "Invalid or expired challenge".into(),
        ));
    }

    Ok(())
}

/// Login with keypair signature
/// POST /api/v1/auth/keypair-login
pub async fn keypair_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<KeypairLoginRequest>,
) -> AppResult<Json<TokenPair>> {
    let account = ChainAccount::from_request(
        req.address.as_deref(),
        req.account_id.as_deref(),
        state.config.chain.ss58_prefix,
    )?;

    // One proof-of-possession path for every route that has one, so a challenge
    // is consumed the same way wherever it is answered.
    consume_signed_challenge(&state.db, account, &req.challenge, &req.signature).await?;

    let user: Option<crate::models::User> =
        sqlx::query_as("SELECT * FROM users WHERE chain_account_id = $1")
            .bind(account.as_bytes())
            .fetch_optional(&state.db)
            .await?;

    let user = user.ok_or(AppError::ValidationError(
        "No account holds this key. Please register first.".into(),
    ))?;

    // Check account status
    if user.status != crate::models::UserStatus::Active {
        return Err(AppError::ValidationError("Account is not active".into()));
    }

    // Create session
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());

    let device_id = req
        .device_id
        .unwrap_or_else(|| "keypair-client".to_string());
    let (_session, tokens) = session_service
        .create_session(crate::services::session_service::NewSession {
            user_id: user.id,
            qor_id: &user.qor_id(),
            role: Some(user.role.as_str()),
            device_id: &device_id,
            ip_address: "0.0.0.0", // Not yet taken from the request
            user_agent: None,
            scopes: crate::models::Session::default_scopes(),
        })
        .await?;
    record_sign_in(&state.db, user.id, "keypair").await?;

    Ok(Json(tokens))
}

/// Register with keypair (create new account)
/// POST /api/v1/auth/keypair-register
pub async fn keypair_register(
    State(state): State<Arc<AppState>>,
    Json(req): Json<KeypairRegisterRequest>,
) -> AppResult<(StatusCode, Json<Value>)> {
    let account = ChainAccount::from_request(
        req.address.as_deref(),
        req.account_id.as_deref(),
        state.config.chain.ss58_prefix,
    )?;

    consume_signed_challenge(&state.db, account, &req.challenge, &req.signature).await?;

    let existing: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE chain_account_id = $1")
            .bind(account.as_bytes())
            .fetch_optional(&state.db)
            .await?;

    if existing.is_some() {
        return Err(AppError::ValidationError(
            "This account already holds a QOR ID".into(),
        ));
    }

    // A name derived from the account's own bytes, when none was asked for.
    let username = req.username.unwrap_or_else(|| {
        format!(
            "key_{}",
            &account.account_id_hex()[2..10] // the first four bytes, in hex
        )
    });
    let username_lower = username.to_lowercase();

    // Validate username
    if !crate::models::QorId::is_valid_username(&username_lower) {
        return Err(AppError::ValidationError(
            "Username must be 3-20 characters, alphanumeric and underscores only".into(),
        ));
    }

    let auth_service = AuthService::new(state.db.clone());

    // Check username availability
    if auth_service
        .find_by_username(&username_lower)
        .await?
        .is_some()
    {
        return Err(AppError::ValidationError("Username already taken".into()));
    }

    // One name per account (ADR-075).
    let discriminator: i16 = 1;

    // Generate a random password hash (keypair-only accounts don't need password)
    let random_password: [u8; 32] = rand::thread_rng().r#gen();
    let password_hash = AuthService::hash_password(&hex::encode(random_password))?;

    // For Sr25519 the account ID *is* the public key (ADR-023), so the account
    // and the key it is proven by are one value, stored once.
    let user_id: uuid::Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO users (
            username, discriminator, password_hash,
            email_verified, role, status,
            auth_method, chain_account_id
        )
        VALUES ($1, $2, $3, TRUE, 'user', 'active', 'keypair', $4)
        RETURNING id
        "#,
    )
    .bind(&username_lower)
    .bind(discriminator)
    .bind(&password_hash)
    .bind(account.as_bytes())
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if matches!(&e, sqlx::Error::Database(db) if db.constraint() == Some("users_username_unique")) {
            AppError::ValidationError("Username already taken".into())
        } else {
            AppError::DatabaseError(e)
        }
    })?;

    // Sign the new account in, exactly as keypair_login does. Registering consumed
    // the only challenge the client signed, so without tokens here the client
    // would need a second challenge and a second signature before it could use
    // the account it has just proved it holds.
    let user: crate::models::User = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let device_id = req
        .device_id
        .unwrap_or_else(|| "keypair-client".to_string());
    let (_session, tokens) = session_service
        .create_session(crate::services::session_service::NewSession {
            user_id: user.id,
            qor_id: &user.qor_id(),
            role: Some(user.role.as_str()),
            device_id: &device_id,
            ip_address: "0.0.0.0", // Not yet taken from the request
            user_agent: None,
            scopes: crate::models::Session::default_scopes(),
        })
        .await?;
    record_sign_in(&state.db, user.id, "keypair").await?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "qor_id": username_lower,
            "user_id": user_id,
            "address": account.address(state.config.chain.ss58_prefix),
            "account_id": account.account_id_hex(),
            "auth_method": "keypair",
            "message": "Account created successfully with keypair authentication",
            "access_token": tokens.access_token,
            "refresh_token": tokens.refresh_token,
            "expires_in": tokens.expires_in,
            "token_type": tokens.token_type,
        })),
    ))
}

/// Link a key to the authenticated account
/// POST /api/v1/auth/link-keypair (requires authentication)
///
/// `POST /api/v1/profile/link-wallet` behaves identically. The caller proves
/// possession of the key by signing a challenge issued for it, and the key is
/// bound to the account named by the access token, never to one named in the
/// request.
pub async fn link_keypair(
    State(state): State<Arc<AppState>>,
    axum::extract::Extension(user_id): axum::extract::Extension<uuid::Uuid>,
    Json(req): Json<LinkKeypairRequest>,
) -> AppResult<Json<Value>> {
    link_verified_key(&state, user_id, &req).await.map(Json)
}

/// Bind a key whose possession has just been proven to `user_id`.
pub(crate) async fn link_verified_key(
    state: &AppState,
    user_id: uuid::Uuid,
    req: &LinkKeypairRequest,
) -> AppResult<Value> {
    let account = ChainAccount::from_request(
        req.address.as_deref(),
        req.account_id.as_deref(),
        state.config.chain.ss58_prefix,
    )?;

    consume_signed_challenge(&state.db, account, &req.challenge, &req.signature).await?;

    // An account belongs to at most one QOR ID.
    let holder: Option<uuid::Uuid> =
        sqlx::query_scalar("SELECT id FROM users WHERE chain_account_id = $1 LIMIT 1")
            .bind(account.as_bytes())
            .fetch_optional(&state.db)
            .await?;

    if matches!(holder, Some(id) if id != user_id) {
        return Err(AppError::ValidationError(
            "This account is already linked to another QOR ID".into(),
        ));
    }

    let current: Option<Option<Vec<u8>>> = sqlx::query_scalar(
        "SELECT chain_account_id FROM users WHERE id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;

    let current = current.ok_or(AppError::InsufficientPermissions)?;

    // Replacing a linked key is refused. Otherwise anyone holding a stolen
    // access token could swap in a key of their own and keep the account.
    if let Some(existing) = current
        && existing.as_slice() != account.as_bytes()
    {
        return Err(AppError::ValidationError(
            "This QOR ID already has a different account linked".into(),
        ));
    }

    let qor_id: String = sqlx::query_scalar(
        r#"
        UPDATE users
        SET chain_account_id = $1,
            auth_method = COALESCE(auth_method, 'password'),
            updated_at = NOW()
        WHERE id = $2
        RETURNING LOWER(username)
        "#,
    )
    .bind(account.as_bytes())
    .bind(user_id)
    .fetch_one(&state.db)
    .await?;
    crate::handlers::progress::award(&state.db, user_id, "link-key").await?;

    Ok(json!({
        "message": "Key linked",
        "qor_id": qor_id,
        "address": account.address(state.config.chain.ss58_prefix),
        "account_id": account.account_id_hex(),
    }))
}

/// Registration and email verification against a real Postgres. `#[sqlx::test]` creates a fresh
/// database per test from `DATABASE_URL`, which building this crate already requires.
#[cfg(test)]
mod registration_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

    const PASSWORD: &str = "correct horse battery staple";

    fn state(db: PgPool) -> Arc<AppState> {
        // Registration never touches Redis; the pool connects only when used.
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

    async fn stored(db: &PgPool, username: &str) -> (bool, Option<String>) {
        sqlx::query_as(
            "SELECT email_verified, email_verification_token FROM users WHERE username = $1",
        )
        .bind(username)
        .fetch_one(db)
        .await
        .expect("registered row")
    }

    fn request(username: &str, email: Option<&str>) -> RegisterRequest {
        RegisterRequest {
            email: email.map(str::to_string),
            password: PASSWORD.into(),
            username: username.into(),
        }
    }

    /// ADR-075: a name belongs to one account, whatever its letter case, and the QOR ID is the name
    /// alone. The database holds the rule itself, so a path that skipped the check is refused too.
    #[sqlx::test]
    async fn a_name_belongs_to_one_account_and_is_the_whole_qor_id(db: PgPool) {
        let (_, Json(body)) = register(State(state(db.clone())), Json(request("Godmode", None)))
            .await
            .expect("registration");
        assert_eq!(body["qor_id"], json!("godmode"), "no #0001");

        for taken in ["godmode", "GODMODE", "GodMode"] {
            let refused = register(State(state(db.clone())), Json(request(taken, None))).await;
            assert!(
                matches!(&refused, Err(AppError::ValidationError(m)) if m == "Username already taken"),
                "{taken} must be refused"
            );
        }

        let bypassed = sqlx::query(
            "INSERT INTO users (email, username, password_hash) VALUES ('other@example.invalid', 'GodMode', 'x')",
        )
        .execute(&db)
        .await;
        assert!(
            matches!(&bypassed, Err(sqlx::Error::Database(e)) if e.constraint() == Some("users_username_unique")),
            "the database refuses a second account with the name"
        );

        let found = AuthService::new(db.clone())
            .find_by_username("GODMODE")
            .await
            .expect("lookup")
            .expect("the one account");
        assert_eq!(found.qor_id(), "godmode");
    }

    #[sqlx::test]
    async fn registering_with_an_email_leaves_it_unverified(db: PgPool) {
        let (status, Json(body)) = register(
            State(state(db.clone())),
            Json(request("withemail", Some("withemail@example.invalid"))),
        )
        .await
        .expect("registration");

        assert_eq!(status, StatusCode::CREATED);
        let (verified, token) = stored(&db, "withemail").await;
        assert!(
            !verified,
            "a supplied email address must be stored unverified"
        );
        assert!(token.is_some(), "a verification token must be outstanding");
        assert_eq!(body["email_verified"], json!(false));
    }

    #[sqlx::test]
    async fn registering_without_an_email_has_nothing_to_verify(db: PgPool) {
        let (_, Json(body)) = register(State(state(db.clone())), Json(request("noemail", None)))
            .await
            .expect("registration");

        let (verified, token) = stored(&db, "noemail").await;
        assert!(verified);
        assert!(token.is_none());
        assert_eq!(body["email_verified"], json!(true));
    }

    #[sqlx::test]
    async fn verifying_the_email_marks_it_verified(db: PgPool) {
        let state = state(db.clone());
        let (status, _) = register(
            State(state.clone()),
            Json(request("toverify", Some("toverify@example.invalid"))),
        )
        .await
        .expect("registration");
        assert_eq!(status, StatusCode::CREATED);
        let (_, token) = stored(&db, "toverify").await;

        let Json(body) = verify_email(
            State(state),
            Json(VerifyEmailRequest {
                token: token.expect("token"),
            }),
        )
        .await
        .expect("verification");
        assert!(
            body["qor_id"].as_str().is_some_and(|id| id == "toverify"),
            "the QOR ID is the name alone (ADR-075)"
        );

        let (verified, token) = stored(&db, "toverify").await;
        assert!(verified);
        assert!(token.is_none());
    }

    const MIGRATIONS_BEFORE_012: [&str; 11] = [
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_add_god_role.sql"),
        include_str!("../../migrations/003_add_backup_code.sql"),
        include_str!("../../migrations/004_music_schema.sql"),
        include_str!("../../migrations/005_donations.sql"),
        include_str!("../../migrations/006_keypair_auth.sql"),
        include_str!("../../migrations/007_agent_accounts.sql"),
        include_str!("../../migrations/008_seed_godmode.sql"),
        include_str!("../../migrations/009_fix_on_chain_address.sql"),
        include_str!("../../migrations/010_disable_seeded_admin.sql"),
        include_str!("../../migrations/011_remove_server_generated_agent_keys.sql"),
    ];

    #[sqlx::test(migrations = false)]
    async fn migration_012_corrects_rows_written_inverted(db: PgPool) {
        for sql in MIGRATIONS_BEFORE_012 {
            sqlx::raw_sql(sql)
                .execute(&db)
                .await
                .expect("earlier migration");
        }

        // Rows as the inverted registration left them, plus one genuinely verified address.
        sqlx::raw_sql(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified, email_verification_token) VALUES
                ('pending@example.invalid', 'pending', 1, 'x', TRUE, 'outstanding'),
                ('verified@example.invalid', 'verified', 1, 'x', TRUE, NULL),
                (NULL, 'noemail', 1, 'x', FALSE, NULL)",
        )
        .execute(&db)
        .await
        .expect("rows written by the old registration");

        sqlx::raw_sql(include_str!(
            "../../migrations/012_correct_inverted_email_verified.sql"
        ))
        .execute(&db)
        .await
        .expect("migration 012");

        let rows: Vec<(String, bool)> =
            sqlx::query_as("SELECT username::text, email_verified FROM users ORDER BY username")
                .fetch_all(&db)
                .await
                .expect("rows");
        assert_eq!(
            rows,
            vec![
                ("noemail".to_string(), true),
                ("pending".to_string(), false),
                ("verified".to_string(), true),
            ]
        );

        let refused = sqlx::raw_sql(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified, email_verification_token)
             VALUES ('again@example.invalid', 'again', 1, 'x', TRUE, 'outstanding')",
        )
        .execute(&db)
        .await;
        assert!(
            refused.is_err(),
            "an email must not be storable as verified while its token is outstanding"
        );
    }
}

/// `forgot-password` against a real Postgres.
#[cfg(test)]
mod forgot_password_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

    const PASSWORD: &str = "correct horse battery staple";

    /// A configured service points at a port nothing listens on, so no mail leaves the machine;
    /// sending fails after the response, which is what these tests look at.
    fn state(db: PgPool, email_configured: bool) -> Arc<AppState> {
        let credential = if email_configured { "test" } else { "" };
        let email = EmailService::new(EmailConfig {
            resend_api_key: credential.into(),
            from: if email_configured {
                "Test <noreply@example.invalid>".into()
            } else {
                String::new()
            },
            base_url: "https://example.invalid".into(),
            api_url: "http://127.0.0.1:9".into(),
        });
        assert_eq!(email.is_configured(), email_configured);
        let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:1")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(AppConfig::default(), db, redis, email))
    }

    async fn register_user(state: &Arc<AppState>, username: &str, email: Option<&str>) {
        let (status, _) = register(
            State(state.clone()),
            Json(RegisterRequest {
                email: email.map(str::to_string),
                password: PASSWORD.into(),
                username: username.into(),
            }),
        )
        .await
        .expect("registration");
        assert_eq!(status, StatusCode::CREATED);
    }

    async fn resets_for(db: &PgPool, username: &str) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM password_resets r JOIN users u ON u.id = r.user_id WHERE u.username = $1",
        )
        .bind(username)
        .fetch_one(db)
        .await
        .expect("count")
    }

    async fn ask(state: &Arc<AppState>, identifier: &str) -> AppResult<Json<Value>> {
        forgot_password(
            State(state.clone()),
            Json(ForgotPasswordRequest {
                identifier: identifier.into(),
            }),
        )
        .await
    }

    #[sqlx::test]
    async fn without_email_configured_every_request_is_refused_and_nothing_is_issued(db: PgPool) {
        let state = state(db.clone(), false);
        register_user(&state, "hasemail", Some("hasemail@example.invalid")).await;
        register_user(&state, "noemail", None).await;

        for identifier in ["hasemail@example.invalid", "hasemail", "noemail", "nobody"] {
            assert!(matches!(
                ask(&state, identifier).await,
                Err(AppError::ServiceUnavailable(_))
            ));
        }
        assert_eq!(resets_for(&db, "hasemail").await, 0);
    }

    #[sqlx::test]
    async fn the_answer_is_identical_for_every_identifier(db: PgPool) {
        let state = state(db.clone(), true);
        register_user(&state, "verified", Some("verified@example.invalid")).await;
        let token: String = sqlx::query_scalar(
            "SELECT email_verification_token FROM users WHERE username = 'verified'",
        )
        .fetch_one(&db)
        .await
        .expect("token");
        let Json(verified) = verify_email(State(state.clone()), Json(VerifyEmailRequest { token }))
            .await
            .expect("verification");
        assert!(
            verified["qor_id"]
                .as_str()
                .is_some_and(|id| id == "verified"),
            "the QOR ID is the name alone (ADR-075)"
        );
        register_user(&state, "unverified", Some("unverified@example.invalid")).await;
        register_user(&state, "noemail", None).await;

        let mut answers = Vec::new();
        for identifier in [
            "verified@example.invalid",
            "verified",
            "unverified@example.invalid",
            "unverified",
            "noemail",
            "nobody",
            "nobody@example.invalid",
        ] {
            let Json(body) = ask(&state, identifier).await.expect("accepted");
            answers.push(body.to_string());
        }
        assert!(
            answers.windows(2).all(|pair| pair[0] == pair[1]),
            "answers differ: {answers:?}"
        );

        // Issuing happens after the answer. Only the verified address gets a link.
        for _ in 0..100 {
            if resets_for(&db, "verified").await >= 2 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        assert_eq!(resets_for(&db, "verified").await, 2);
        assert_eq!(resets_for(&db, "unverified").await, 0);
        assert_eq!(resets_for(&db, "noemail").await, 0);
    }
}

/// Password reset by token against a real Postgres.
#[cfg(test)]
mod password_reset_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

    const OLD_PASSWORD: &str = "an old password long enough";
    const NEW_PASSWORD: &str = "a brand new password";

    /// A reset revokes sessions in Redis. Tests that complete one need a Redis at
    /// `QOR_AUTH_TEST_REDIS_URL`; CI provides it and runs them with `--include-ignored`.
    fn state(db: PgPool) -> Arc<AppState> {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        state_with_redis(db, &url)
    }

    fn state_with_redis(db: PgPool, redis_url: &str) -> Arc<AppState> {
        let redis = deadpool_redis::Config::from_url(redis_url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        ))
    }

    async fn stored_hash(db: &PgPool, user: uuid::Uuid) -> String {
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(db)
            .await
            .expect("hash")
    }

    async fn account_with_reset(db: &PgPool, token: &str) -> uuid::Uuid {
        let hash = AuthService::hash_password(OLD_PASSWORD).expect("hash");
        let user: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified) VALUES ('reset@example.invalid', 'resetme', 1, $1, TRUE) RETURNING id",
        )
        .bind(hash)
        .fetch_one(db)
        .await
        .expect("account");
        sqlx::query(
            "INSERT INTO password_resets (user_id, token, expires_at) VALUES ($1, $2, NOW() + INTERVAL '1 hour')",
        )
        .bind(user)
        .bind(token)
        .execute(db)
        .await
        .expect("reset token");
        user
    }

    fn request(token: &str) -> Json<ResetPasswordWithTokenRequest> {
        Json(ResetPasswordWithTokenRequest {
            token: token.into(),
            new_password: NEW_PASSWORD.into(),
        })
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_valid_token_resets_the_password_exactly_once(db: PgPool) {
        let state = state(db.clone());
        let user = account_with_reset(&db, "reset-token").await;

        let Json(body) = reset_password(State(state.clone()), request("reset-token"))
            .await
            .expect("reset");
        assert_eq!(body["message"], json!("Password reset successfully"));

        let hash: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db)
            .await
            .expect("hash");
        assert!(AuthService::verify_password(NEW_PASSWORD, &hash).expect("verify"));
        assert!(!AuthService::verify_password(OLD_PASSWORD, &hash).expect("verify"));

        let spent: bool = sqlx::query_scalar(
            "SELECT used_at IS NOT NULL FROM password_resets WHERE token = 'reset-token'",
        )
        .fetch_one(&db)
        .await
        .expect("token row");
        assert!(spent);

        assert!(matches!(
            reset_password(State(state), request("reset-token")).await,
            Err(AppError::ValidationError(_))
        ));
    }

    fn session_for(user_id: uuid::Uuid) -> crate::services::session_service::NewSession<'static> {
        crate::services::session_service::NewSession {
            user_id,
            qor_id: "resetme#0001",
            role: None,
            device_id: "test-device",
            ip_address: "127.0.0.1",
            user_agent: None,
            scopes: vec![],
        }
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn every_live_session_dies_on_reset(db: PgPool) {
        let state = state(db.clone());
        let user = account_with_reset(&db, "reset-token").await;
        let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
        let (first, first_tokens) = sessions
            .create_session(session_for(user))
            .await
            .expect("first session");
        let (second, second_tokens) = sessions
            .create_session(session_for(user))
            .await
            .expect("second session");

        let Json(body) = reset_password(State(state), request("reset-token"))
            .await
            .expect("reset");
        assert_eq!(body["message"], json!("Password reset successfully"));

        for (session, tokens) in [(first, first_tokens), (second, second_tokens)] {
            assert!(
                sessions
                    .get_session(session.session_id)
                    .await
                    .expect("lookup")
                    .is_none()
            );
            assert!(matches!(
                sessions
                    .authenticate_access_token(&tokens.access_token)
                    .await,
                Err(AppError::InvalidToken)
            ));
        }
    }

    #[sqlx::test]
    async fn if_sessions_cannot_be_revoked_nothing_is_reset(db: PgPool) {
        let state = state_with_redis(db.clone(), "redis://127.0.0.1:1");
        let user = account_with_reset(&db, "reset-token").await;

        assert!(
            reset_password(State(state), request("reset-token"))
                .await
                .is_err()
        );

        let hash = stored_hash(&db, user).await;
        assert!(AuthService::verify_password(OLD_PASSWORD, &hash).expect("verify"));
        let spent: bool = sqlx::query_scalar(
            "SELECT used_at IS NOT NULL FROM password_resets WHERE token = 'reset-token'",
        )
        .fetch_one(&db)
        .await
        .expect("token row");
        assert!(!spent, "the token must still be usable");
    }
}

/// Backup codes against a real Postgres. Tests that complete a reset also need Redis.
#[cfg(test)]
mod backup_code_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

    const PASSWORD: &str = "correct horse battery staple";

    fn state(db: PgPool) -> Arc<AppState> {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        let redis = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        ))
    }

    async fn register_without_email(state: &Arc<AppState>, username: &str) -> Vec<String> {
        let (status, Json(body)) = register(
            State(state.clone()),
            Json(RegisterRequest {
                email: None,
                password: PASSWORD.into(),
                username: username.into(),
            }),
        )
        .await
        .expect("registration");
        assert_eq!(status, StatusCode::CREATED);
        body["backup_codes"]
            .as_array()
            .expect("backup codes")
            .iter()
            .map(|code| code.as_str().expect("code").to_string())
            .collect()
    }

    fn reset(
        username: &str,
        code: &str,
        new_password: &str,
    ) -> Json<ResetPasswordWithBackupRequest> {
        Json(ResetPasswordWithBackupRequest {
            username: username.into(),
            backup_code: code.into(),
            new_password: new_password.into(),
        })
    }

    async fn unused_codes(db: &PgPool, username: &str) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM backup_codes b JOIN users u ON u.id = b.user_id WHERE u.username = $1 AND b.used_at IS NULL",
        )
        .bind(username)
        .fetch_one(db)
        .await
        .expect("count")
    }

    #[sqlx::test]
    async fn registration_issues_ten_codes_and_stores_only_their_hashes(db: PgPool) {
        let state = state(db.clone());
        let codes = register_without_email(&state, "codes").await;
        assert_eq!(codes.len(), AuthService::BACKUP_CODE_COUNT);

        let stored: Vec<String> = sqlx::query_scalar(
            "SELECT b.code_hash FROM backup_codes b JOIN users u ON u.id = b.user_id WHERE u.username = 'codes'",
        )
        .fetch_all(&db)
        .await
        .expect("stored codes");
        assert_eq!(stored.len(), codes.len());
        for code in &codes {
            assert!(stored.contains(&AuthService::hash_backup_code(code)));
            assert!(
                !stored.contains(code),
                "a code must never be stored as itself"
            );
        }
    }

    #[sqlx::test]
    async fn a_wrong_code_and_an_unknown_username_are_refused_alike_and_spend_nothing(db: PgPool) {
        // Refusals happen before any session is touched, so no Redis is needed.
        let state = state(db.clone());
        let codes = register_without_email(&state, "refused").await;

        for request in [
            reset(
                "refused",
                &AuthService::generate_backup_code(),
                "a new password long enough",
            ),
            reset("nobody", &codes[0], "a new password long enough"),
        ] {
            assert!(matches!(
                reset_password_with_backup(State(state.clone()), request).await,
                Err(AppError::InvalidCredentials)
            ));
        }
        assert_eq!(unused_codes(&db, "refused").await, 10);
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_code_works_once_and_the_count_of_those_left_falls(db: PgPool) {
        let state = state(db.clone());
        let codes = register_without_email(&state, "onceonly").await;
        let user_id: uuid::Uuid =
            sqlx::query_scalar("SELECT id FROM users WHERE username = 'onceonly'")
                .fetch_one(&db)
                .await
                .expect("account");
        let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
        let (session, _) = sessions
            .create_session(crate::services::session_service::NewSession {
                user_id,
                qor_id: "onceonly#0001",
                role: None,
                device_id: "test-device",
                ip_address: "127.0.0.1",
                user_agent: None,
                scopes: vec![],
            })
            .await
            .expect("session");

        let Json(body) = reset_password_with_backup(
            State(state.clone()),
            reset("onceonly", &codes[0], "the first new password"),
        )
        .await
        .expect("first code");
        assert_eq!(body["backup_codes_remaining"], json!(9));
        assert!(
            sessions
                .get_session(session.session_id)
                .await
                .expect("lookup")
                .is_none()
        );

        assert!(matches!(
            reset_password_with_backup(
                State(state.clone()),
                reset("onceonly", &codes[0], "the second new password")
            )
            .await,
            Err(AppError::InvalidCredentials)
        ));

        let Json(body) = reset_password_with_backup(
            State(state),
            reset(
                "onceonly",
                &codes[1].to_lowercase(),
                "the second new password",
            ),
        )
        .await
        .expect("second code");
        assert_eq!(body["backup_codes_remaining"], json!(8));
        assert_eq!(unused_codes(&db, "onceonly").await, 8);
    }

    const MIGRATIONS_BEFORE_014: [&str; 13] = [
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_add_god_role.sql"),
        include_str!("../../migrations/003_add_backup_code.sql"),
        include_str!("../../migrations/004_music_schema.sql"),
        include_str!("../../migrations/005_donations.sql"),
        include_str!("../../migrations/006_keypair_auth.sql"),
        include_str!("../../migrations/007_agent_accounts.sql"),
        include_str!("../../migrations/008_seed_godmode.sql"),
        include_str!("../../migrations/009_fix_on_chain_address.sql"),
        include_str!("../../migrations/010_disable_seeded_admin.sql"),
        include_str!("../../migrations/011_remove_server_generated_agent_keys.sql"),
        include_str!("../../migrations/012_correct_inverted_email_verified.sql"),
        include_str!("../../migrations/013_refuse_nil_user_id.sql"),
    ];

    #[sqlx::test(migrations = false)]
    async fn migration_014_hashes_existing_codes_and_drops_the_plain_column(db: PgPool) {
        for sql in MIGRATIONS_BEFORE_014 {
            sqlx::raw_sql(sql)
                .execute(&db)
                .await
                .expect("earlier migration");
        }
        let code = AuthService::generate_backup_code();
        sqlx::query(
            "INSERT INTO users (username, discriminator, password_hash, email_verified, backup_code) VALUES ('legacy', 1, 'x', TRUE, $1)",
        )
        .bind(&code)
        .execute(&db)
        .await
        .expect("legacy account");

        sqlx::raw_sql(include_str!(
            "../../migrations/014_single_use_backup_codes.sql"
        ))
        .execute(&db)
        .await
        .expect("migration 014");

        let stored: Vec<String> = sqlx::query_scalar("SELECT code_hash FROM backup_codes")
            .fetch_all(&db)
            .await
            .expect("stored codes");
        assert_eq!(stored, vec![AuthService::hash_backup_code(&code)]);

        let plain_columns: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.columns WHERE table_name = 'users' AND column_name = 'backup_code'",
        )
        .fetch_one(&db)
        .await
        .expect("columns");
        assert_eq!(plain_columns, 0);
    }
}

/// Sign-in refusals against a real Postgres. No refusal touches Redis.
#[cfg(test)]
mod sign_in_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

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

    async fn account(db: &PgPool, username: &str, email: Option<&str>, status: &str, locked: bool) {
        let hash = AuthService::hash_password(PASSWORD).expect("hash");
        sqlx::query(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified, status, locked_until)
             VALUES ($1, $2, 1, $3, TRUE, $4::user_status, CASE WHEN $5 THEN NOW() + INTERVAL '1 hour' END)",
        )
        .bind(email)
        .bind(username)
        .bind(hash)
        .bind(status)
        .bind(locked)
        .execute(db)
        .await
        .expect("account");
    }

    fn attempt(identifier: &str, password: &str) -> Json<LoginRequest> {
        Json(LoginRequest {
            identifier: identifier.into(),
            password: password.into(),
            device_id: None,
        })
    }

    async fn attempts_and_lock(
        db: &PgPool,
        username: &str,
    ) -> (i32, Option<chrono::DateTime<Utc>>) {
        sqlx::query_as("SELECT login_attempts, locked_until FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(db)
            .await
            .expect("row")
    }

    #[sqlx::test]
    async fn every_refusal_is_the_same_answer(db: PgPool) {
        let state = state(db.clone());
        account(
            &db,
            "active",
            Some("active@example.invalid"),
            "active",
            false,
        )
        .await;
        account(&db, "lockedout", None, "active", true).await;
        account(&db, "banned", None, "banned", false).await;

        for request in [
            attempt("nobody", PASSWORD),
            attempt("nobody@example.invalid", PASSWORD),
            attempt("active", "not the password"),
            attempt("active@example.invalid", "not the password"),
            attempt("lockedout", PASSWORD),
            attempt("banned", PASSWORD),
        ] {
            assert!(matches!(
                login(State(state.clone()), request).await,
                Err(AppError::InvalidCredentials)
            ));
        }
    }

    #[sqlx::test]
    async fn only_a_wrong_password_on_an_open_account_counts_as_a_failed_attempt(db: PgPool) {
        let state = state(db.clone());
        account(&db, "active", None, "active", false).await;
        account(&db, "lockedout", None, "active", true).await;
        account(&db, "banned", None, "banned", false).await;
        let (_, lock_before) = attempts_and_lock(&db, "lockedout").await;

        for request in [
            attempt("nobody", "not the password"),
            attempt("active", "not the password"),
            attempt("lockedout", "not the password"),
            attempt("banned", "not the password"),
        ] {
            let _ = login(State(state.clone()), request).await;
        }

        assert_eq!(attempts_and_lock(&db, "active").await.0, 1);
        let (locked_attempts, lock_after) = attempts_and_lock(&db, "lockedout").await;
        assert_eq!(locked_attempts, 0);
        assert_eq!(lock_after, lock_before, "attempts must not extend a lock");
        assert_eq!(attempts_and_lock(&db, "banned").await.0, 0);
    }
}

/// `resend-verification` against a real Postgres. No test touches Redis.
#[cfg(test)]
mod resend_verification_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

    /// A configured service points at a port nothing listens on: sending fails after
    /// the token is replaced, which is what these tests look at.
    fn state(db: PgPool, email_configured: bool) -> Arc<AppState> {
        let credential = if email_configured { "test" } else { "" };
        let email = EmailService::new(EmailConfig {
            resend_api_key: credential.into(),
            from: if email_configured {
                "Test <noreply@example.invalid>".into()
            } else {
                String::new()
            },
            base_url: "https://example.invalid".into(),
            api_url: "http://127.0.0.1:9".into(),
        });
        assert_eq!(email.is_configured(), email_configured);
        let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:1")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(AppConfig::default(), db, redis, email))
    }

    /// An account whose verification token `token` lapsed a day ago, last sent
    /// `sent_minutes_ago` minutes ago (never, if `None`).
    async fn account(
        db: &PgPool,
        username: &str,
        email: Option<&str>,
        verified: bool,
        token: Option<&str>,
        sent_minutes_ago: Option<i32>,
    ) {
        sqlx::query(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified,
                                email_verification_token, email_verification_expires_at,
                                email_verification_sent_at, email_verification_send_count, email_verification_count_since)
             VALUES ($1, $2, 1, 'x', $3, $4, NOW() - INTERVAL '1 day',
                     NOW() - make_interval(mins => $5), CASE WHEN $5 IS NULL THEN 0 ELSE 1 END,
                     NOW() - make_interval(mins => $5))",
        )
        .bind(email)
        .bind(username)
        .bind(verified)
        .bind(token)
        .bind(sent_minutes_ago)
        .execute(db)
        .await
        .expect("account");
    }

    async fn token_of(db: &PgPool, username: &str) -> Option<String> {
        sqlx::query_scalar("SELECT email_verification_token FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(db)
            .await
            .expect("token")
    }

    async fn ask(state: &Arc<AppState>, identifier: &str) -> AppResult<Json<Value>> {
        resend_verification(
            State(state.clone()),
            Json(ResendVerificationRequest {
                identifier: identifier.into(),
            }),
        )
        .await
    }

    /// The request was accepted with the one answer every identifier gets.
    fn accepted(answer: AppResult<Json<Value>>) {
        let Json(body) = answer.expect("accepted");
        assert_eq!(body["message"], json!(RESEND_VERIFICATION_RESPONSE));
    }

    /// Wait for the token to stop being `old`. Issuing happens after the answer.
    async fn replaced(db: &PgPool, username: &str, old: Option<&str>) -> Option<String> {
        for _ in 0..100 {
            let now = token_of(db, username).await;
            if now.as_deref() != old {
                return now;
            }
            tokio::time::sleep(std::time::Duration::from_millis(20)).await;
        }
        token_of(db, username).await
    }

    /// Give a request that should change nothing time to have changed something.
    async fn settle() {
        tokio::time::sleep(std::time::Duration::from_millis(600)).await;
    }

    #[sqlx::test]
    async fn without_email_configured_every_request_is_refused_and_no_token_changes(db: PgPool) {
        let state = state(db.clone(), false);
        account(
            &db,
            "lapsed",
            Some("lapsed@example.invalid"),
            false,
            Some("old-token"),
            Some(2880),
        )
        .await;

        for identifier in ["lapsed", "lapsed@example.invalid", "nobody"] {
            assert!(matches!(
                ask(&state, identifier).await,
                Err(AppError::ServiceUnavailable(_))
            ));
        }
        settle().await;
        assert_eq!(token_of(&db, "lapsed").await.as_deref(), Some("old-token"));
    }

    #[sqlx::test]
    async fn an_account_with_an_expired_token_recovers_through_a_new_one(db: PgPool) {
        let state = state(db.clone(), true);
        account(
            &db,
            "lapsed",
            Some("lapsed@example.invalid"),
            false,
            Some("old-token"),
            Some(2880),
        )
        .await;

        accepted(ask(&state, "lapsed@example.invalid").await);
        let new_token = replaced(&db, "lapsed", Some("old-token"))
            .await
            .expect("a new token");
        assert_ne!(new_token, "old-token");
        let live: bool = sqlx::query_scalar(
            "SELECT email_verification_expires_at > NOW() FROM users WHERE username = 'lapsed'",
        )
        .fetch_one(&db)
        .await
        .expect("expiry");
        assert!(live, "the new token must not already be expired");

        assert!(matches!(
            verify_email(
                State(state.clone()),
                Json(VerifyEmailRequest {
                    token: "old-token".into()
                })
            )
            .await,
            Err(AppError::ValidationError(_))
        ));
        let Json(body) = verify_email(State(state), Json(VerifyEmailRequest { token: new_token }))
            .await
            .expect("the new link verifies");
        assert!(
            body["qor_id"].as_str().is_some_and(|id| id == "lapsed"),
            "the QOR ID is the name alone (ADR-075)"
        );
        let verified: bool =
            sqlx::query_scalar("SELECT email_verified FROM users WHERE username = 'lapsed'")
                .fetch_one(&db)
                .await
                .expect("verified");
        assert!(verified);
    }

    #[sqlx::test]
    async fn a_second_request_within_five_minutes_changes_nothing(db: PgPool) {
        let state = state(db.clone(), true);
        account(
            &db,
            "eager",
            Some("eager@example.invalid"),
            false,
            Some("old-token"),
            Some(60),
        )
        .await;

        accepted(ask(&state, "eager").await);
        let first = replaced(&db, "eager", Some("old-token")).await;
        assert_ne!(first.as_deref(), Some("old-token"));

        accepted(ask(&state, "eager").await);
        settle().await;
        assert_eq!(token_of(&db, "eager").await, first);
    }

    #[sqlx::test]
    async fn no_more_than_five_messages_in_a_day(db: PgPool) {
        let state = state(db.clone(), true);
        account(
            &db,
            "capped",
            Some("capped@example.invalid"),
            false,
            Some("old-token"),
            None,
        )
        .await;
        sqlx::query(
            "UPDATE users SET email_verification_sent_at = NOW() - INTERVAL '10 minutes',
                              email_verification_send_count = 5,
                              email_verification_count_since = NOW() - INTERVAL '1 hour'
             WHERE username = 'capped'",
        )
        .execute(&db)
        .await
        .expect("five sends today");

        accepted(ask(&state, "capped").await);
        settle().await;
        assert_eq!(token_of(&db, "capped").await.as_deref(), Some("old-token"));
    }

    #[sqlx::test]
    async fn the_answer_is_identical_and_only_an_unverified_account_changes(db: PgPool) {
        let state = state(db.clone(), true);
        account(
            &db,
            "pending",
            Some("pending@example.invalid"),
            false,
            Some("old-token"),
            Some(60),
        )
        .await;
        account(
            &db,
            "verified",
            Some("verified@example.invalid"),
            true,
            None,
            Some(60),
        )
        .await;
        account(&db, "noemail", None, true, None, None).await;

        let mut answers = Vec::new();
        for identifier in [
            "pending",
            "verified",
            "noemail",
            "nobody",
            "nobody@example.invalid",
        ] {
            let Json(body) = ask(&state, identifier).await.expect("accepted");
            answers.push(body.to_string());
        }
        assert!(
            answers.windows(2).all(|pair| pair[0] == pair[1]),
            "answers differ: {answers:?}"
        );

        assert_ne!(
            replaced(&db, "pending", Some("old-token")).await.as_deref(),
            Some("old-token")
        );
        settle().await;
        assert_eq!(token_of(&db, "verified").await, None);
        assert_eq!(token_of(&db, "noemail").await, None);
    }
}

/// Migration 018, against a real Postgres. What it does to rows that already
/// exist matters more than what it does to an empty database: a derived address
/// must not survive it (ADR-017), and neither must a key from the scheme this
/// project has left (ADR-023).
#[cfg(test)]
mod migration_018_tests {
    use sqlx::PgPool;

    const BEFORE_018: [&str; 17] = [
        include_str!("../../migrations/001_initial_schema.sql"),
        include_str!("../../migrations/002_add_god_role.sql"),
        include_str!("../../migrations/003_add_backup_code.sql"),
        include_str!("../../migrations/004_music_schema.sql"),
        include_str!("../../migrations/005_donations.sql"),
        include_str!("../../migrations/006_keypair_auth.sql"),
        include_str!("../../migrations/007_agent_accounts.sql"),
        include_str!("../../migrations/008_seed_godmode.sql"),
        include_str!("../../migrations/009_fix_on_chain_address.sql"),
        include_str!("../../migrations/010_disable_seeded_admin.sql"),
        include_str!("../../migrations/011_remove_server_generated_agent_keys.sql"),
        include_str!("../../migrations/012_correct_inverted_email_verified.sql"),
        include_str!("../../migrations/013_refuse_nil_user_id.sql"),
        include_str!("../../migrations/014_single_use_backup_codes.sql"),
        include_str!("../../migrations/015_verification_resend_limit.sql"),
        include_str!("../../migrations/016_pending_email_change.sql"),
        include_str!("../../migrations/017_email_suppressions.sql"),
    ];

    const MIGRATION_018: &str = include_str!("../../migrations/018_chain_accounts_as_bytes.sql");

    #[sqlx::test(migrations = false)]
    async fn no_account_keeps_a_key_or_an_address_that_can_no_longer_be_proven(db: PgPool) {
        for sql in BEFORE_018 {
            sqlx::raw_sql(sql)
                .execute(&db)
                .await
                .expect("earlier migration");
        }

        // Two rows as the old write paths left them: one with a key it proved
        // under the old scheme, and one with an address derived from its name.
        // Values are bound, never written into the statement (sql_hygiene).
        let proven = "11".repeat(32);
        sqlx::query(
            "INSERT INTO users (username, discriminator, password_hash, email_verified, primary_pubkey, on_chain_address, auth_method) VALUES ('proven', 1, 'x', TRUE, $1, $2, 'keypair')",
        )
        .bind(&proven)
        .bind(format!("0x{proven}"))
        .execute(&db)
        .await
        .expect("a row with a proven key");

        sqlx::query(
            "INSERT INTO users (username, discriminator, password_hash, email_verified, on_chain_address, auth_method) VALUES ('derived', 1, 'x', TRUE, $1, 'password')",
        )
        .bind(format!("0x{}", "22".repeat(32)))
        .execute(&db)
        .await
        .expect("a row with a derived address");

        sqlx::query(
            "INSERT INTO auth_challenges (pubkey, challenge, expires_at) VALUES ($1, 'demiurge:1:old', NOW() + INTERVAL '5 minutes')",
        )
        .bind(&proven)
        .execute(&db)
        .await
        .expect("a live challenge for the old key");

        sqlx::raw_sql(MIGRATION_018)
            .execute(&db)
            .await
            .expect("migration 018");

        let accounts: Vec<(String, Option<Vec<u8>>)> =
            sqlx::query_as("SELECT username, chain_account_id FROM users ORDER BY username")
                .fetch_all(&db)
                .await
                .expect("rows");

        for (username, account) in &accounts {
            assert_eq!(
                account, &None,
                "{username} kept an account nobody can prove a key for"
            );
        }

        // The columns that held the old forms are gone, so nothing can read
        // them by habit.
        for column in ["on_chain_address", "primary_pubkey"] {
            let present: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'users' AND column_name = $1)",
            )
            .bind(column)
            .fetch_one(&db)
            .await
            .expect("catalogue");
            assert!(!present, "users.{column} should have been dropped");
        }

        // A challenge issued for a key of the old scheme cannot be answered, so
        // it is not carried over.
        let challenges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_challenges")
            .fetch_one(&db)
            .await
            .expect("count");
        assert_eq!(challenges, 0);
    }

    /// The column takes 32 bytes and nothing else, and an account belongs to
    /// one row (ADR-024).
    #[sqlx::test(migrations = false)]
    async fn the_column_refuses_anything_but_one_account_of_32_bytes(db: PgPool) {
        for sql in BEFORE_018 {
            sqlx::raw_sql(sql)
                .execute(&db)
                .await
                .expect("earlier migration");
        }
        sqlx::raw_sql(MIGRATION_018)
            .execute(&db)
            .await
            .expect("migration 018");

        let insert = |username: &'static str, account: Vec<u8>| {
            let db = db.clone();
            async move {
                sqlx::query(
                    "INSERT INTO users (username, discriminator, password_hash, email_verified, chain_account_id) VALUES ($1, 1, 'x', TRUE, $2)",
                )
                .bind(username)
                .bind(account)
                .execute(&db)
                .await
            }
        };

        insert("first", vec![0xab; 32])
            .await
            .expect("32 bytes are accepted");
        assert!(
            insert("tooshort", vec![0xab; 31]).await.is_err(),
            "31 bytes must be refused"
        );
        assert!(
            insert("duplicate", vec![0xab; 32]).await.is_err(),
            "one account belongs to one row"
        );

        // Two rows with no account at all are fine: most accounts have none
        // until they prove a key (ADR-017).
        sqlx::raw_sql(
            "INSERT INTO users (username, discriminator, password_hash, email_verified) VALUES ('none_a', 1, 'x', TRUE), ('none_b', 1, 'x', TRUE)",
        )
        .execute(&db)
        .await
        .expect("accounts without a chain identity");
    }
}

/// The routes that take an account, against a real Postgres: one account is one
/// row whichever form it arrived in, nothing is stored without proof of
/// possession, and a password-only account has no chain identity (ADR-017,
/// ADR-023, ADR-024).
#[cfg(test)]
mod chain_account_tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use sp_core::Pair as _;
    use sqlx::PgPool;

    const PASSWORD: &str = "a sufficiently long password";

    /// Creating a session needs Redis, so the tests that register or sign in
    /// carry the same marker as the rest of the suite.
    fn state(db: PgPool) -> Arc<AppState> {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        let redis = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        ))
    }

    fn key(seed: u8) -> sp_core::sr25519::Pair {
        sp_core::sr25519::Pair::from_seed(&[seed; 32])
    }

    fn account(pair: &sp_core::sr25519::Pair) -> ChainAccount {
        ChainAccount::from_hex(&hex::encode(pair.public().0)).expect("32 bytes")
    }

    /// Ask for a challenge the way a client does, then answer it.
    async fn signed_challenge(
        state: &Arc<AppState>,
        pair: &sp_core::sr25519::Pair,
        as_hex: bool,
    ) -> (Option<String>, Option<String>, String, String) {
        let account = account(pair);
        let (address, account_id) = if as_hex {
            (None, Some(account.account_id_hex()))
        } else {
            (Some(account.address(state.config.chain.ss58_prefix)), None)
        };

        let Json(issued) = get_challenge(
            State(state.clone()),
            Query(ChallengeRequest {
                address: address.clone(),
                account_id: account_id.clone(),
            }),
        )
        .await
        .expect("a challenge is issued");

        let signature = hex::encode(pair.sign(challenge_message(&issued.challenge).as_bytes()).0);

        (address, account_id, issued.challenge, signature)
    }

    async fn stored_account(db: &PgPool, username: &str) -> Option<Vec<u8>> {
        sqlx::query_scalar("SELECT chain_account_id FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(db)
            .await
            .expect("the account row")
    }

    /// ADR-017: registering with a password creates no chain identity. It used
    /// to derive one by hashing the name and the clock, which no key could ever
    /// sign for.
    #[sqlx::test]
    async fn a_password_account_gets_no_chain_identity(db: PgPool) {
        let state = state(db.clone());

        let (_, Json(body)) = register(
            State(state),
            Json(RegisterRequest {
                username: "nochain".into(),
                email: None,
                password: PASSWORD.into(),
            }),
        )
        .await
        .expect("registered");

        assert!(
            body.get("on_chain_address").is_none() && body.get("address").is_none(),
            "registration must not report an address: {body}"
        );
        assert_eq!(stored_account(&db, "nochain").await, None);
    }

    /// The two forms of one account reach the same row (ADR-024): registered as
    /// SS58, signed in as hex.
    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn ss58_and_hex_reach_the_same_account(db: PgPool) {
        let state = state(db.clone());
        let pair = key(0x21);

        let (address, _, challenge, signature) = signed_challenge(&state, &pair, false).await;
        let (_, Json(body)) = keypair_register(
            State(state.clone()),
            Json(KeypairRegisterRequest {
                address: address.clone(),
                account_id: None,
                username: Some("bothforms".into()),
                challenge,
                signature,
                device_id: None,
            }),
        )
        .await
        .expect("registered");

        assert_eq!(body["address"].as_str(), address.as_deref());
        assert_eq!(
            body["account_id"].as_str(),
            Some(account(&pair).account_id_hex().as_str())
        );
        assert_eq!(
            stored_account(&db, "bothforms").await.as_deref(),
            Some(account(&pair).as_bytes()),
            "the row holds the account's bytes"
        );

        let (_, account_id, challenge, signature) = signed_challenge(&state, &pair, true).await;
        keypair_login(
            State(state),
            Json(KeypairLoginRequest {
                address: None,
                account_id,
                challenge,
                signature,
                device_id: None,
            }),
        )
        .await
        .map(|Json(_tokens)| ())
        .expect("the hex form signs the same account in");
    }

    /// A challenge is bound to the account it was issued for, so another
    /// account's signature over it proves nothing.
    #[sqlx::test]
    async fn a_challenge_answers_only_for_the_account_it_was_issued_for(db: PgPool) {
        let state = state(db.clone());
        let holder = key(0x31);
        let stranger = key(0x32);

        let (address, _, challenge, _) = signed_challenge(&state, &holder, false).await;
        let strangers_signature =
            hex::encode(stranger.sign(challenge_message(&challenge).as_bytes()).0);

        let refused = keypair_register(
            State(state.clone()),
            Json(KeypairRegisterRequest {
                address,
                account_id: None,
                username: Some("wrongsigner".into()),
                challenge: challenge.clone(),
                signature: strangers_signature.clone(),
                device_id: None,
            }),
        )
        .await;
        assert!(refused.is_err(), "another key's signature must be refused");

        // And the stranger cannot spend the challenge by naming itself either:
        // it was issued for the holder's account.
        let refused = keypair_register(
            State(state),
            Json(KeypairRegisterRequest {
                address: Some(account(&stranger).address(42)),
                account_id: None,
                username: Some("wrongaccount".into()),
                challenge,
                signature: strangers_signature,
                device_id: None,
            }),
        )
        .await;
        assert!(refused.is_err(), "a challenge is not transferable");
    }

    /// A challenge is spent once, so a captured request cannot be replayed.
    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_challenge_is_spent_once(db: PgPool) {
        let state = state(db.clone());
        let pair = key(0x41);
        let (address, _, challenge, signature) = signed_challenge(&state, &pair, false).await;

        let request = || {
            Json(KeypairRegisterRequest {
                address: address.clone(),
                account_id: None,
                username: Some("replayed".into()),
                challenge: challenge.clone(),
                signature: signature.clone(),
                device_id: None,
            })
        };

        keypair_register(State(state.clone()), request())
            .await
            .map(|(_status, Json(_body))| ())
            .expect("the first use is accepted");
        assert!(
            keypair_register(State(state), request()).await.is_err(),
            "the second use must be refused"
        );
    }

    /// An account belongs to one QOR ID: the second attempt to link it is
    /// refused, and so is replacing a linked account with a different one.
    #[sqlx::test]
    async fn an_account_belongs_to_one_qor_id(db: PgPool) {
        let state = state(db.clone());
        let pair = key(0x51);

        let first: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (username, discriminator, password_hash, email_verified) VALUES ('linker', 1, 'x', TRUE) RETURNING id",
        )
        .fetch_one(&db)
        .await
        .expect("an account");
        let second: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (username, discriminator, password_hash, email_verified) VALUES ('other', 1, 'x', TRUE) RETURNING id",
        )
        .fetch_one(&db)
        .await
        .expect("another account");

        let link = |state: Arc<AppState>, user: uuid::Uuid, pair: sp_core::sr25519::Pair| async move {
            let (address, _, challenge, signature) = signed_challenge(&state, &pair, false).await;
            link_verified_key(
                &state,
                user,
                &LinkKeypairRequest {
                    address,
                    account_id: None,
                    challenge,
                    signature,
                },
            )
            .await
        };

        let bound = link(state.clone(), first, pair.clone())
            .await
            .expect("the key links");
        assert_eq!(
            bound["address"].as_str(),
            Some(account(&pair).address(42).as_str())
        );
        assert_eq!(
            stored_account(&db, "linker").await.as_deref(),
            Some(account(&pair).as_bytes())
        );

        assert!(
            link(state.clone(), second, pair.clone()).await.is_err(),
            "a second QOR ID cannot claim an account that is already linked"
        );

        // Linking the same account again is not a change, so it is allowed;
        // linking a different one is refused, or a stolen access token could
        // swap in a key of its own and keep the account.
        link(state.clone(), first, pair)
            .await
            .expect("relinking the same account is not a change");
        assert!(
            link(state, first, key(0x52)).await.is_err(),
            "a linked account cannot be replaced"
        );
    }

    /// An account offered without a signature over a live challenge is not
    /// stored, on the route that stores one (F-Q9).
    #[sqlx::test]
    async fn an_account_without_proof_is_not_stored(db: PgPool) {
        let state = state(db.clone());
        let pair = key(0x61);
        let user: uuid::Uuid = sqlx::query_scalar(
            "INSERT INTO users (username, discriminator, password_hash, email_verified) VALUES ('unproven', 1, 'x', TRUE) RETURNING id",
        )
        .fetch_one(&db)
        .await
        .expect("an account");

        // A challenge nobody issued, and a signature that answers nothing.
        let refused = link_verified_key(
            &state,
            user,
            &LinkKeypairRequest {
                address: Some(account(&pair).address(42)),
                account_id: None,
                challenge: format!("demiurge:1757800000:{}", "ab".repeat(32)),
                signature: hex::encode([0u8; 64]),
            },
        )
        .await;

        assert!(refused.is_err());
        assert_eq!(
            stored_account(&db, "unproven").await,
            None,
            "nothing is stored for an account that was not proven"
        );
    }

    /// An address written for another network never reaches the database, on
    /// the route that would store it.
    #[sqlx::test]
    async fn an_address_for_another_network_is_refused(db: PgPool) {
        use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};

        let state = state(db.clone());
        let elsewhere = key(0x71)
            .public()
            .to_ss58check_with_version(Ss58AddressFormat::custom(2));

        let refused = get_challenge(
            State(state),
            Query(ChallengeRequest {
                address: Some(elsewhere),
                account_id: None,
            }),
        )
        .await;

        assert!(refused.is_err(), "another network's prefix is refused");
        let challenges: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM auth_challenges")
            .fetch_one(&db)
            .await
            .expect("count");
        assert_eq!(challenges, 0, "nothing was issued or stored");
    }
}

#[cfg(test)]
mod possession_tests {
    use super::*;
    use sp_core::Pair as _;

    const PREFIX: u16 = 42;

    fn key(seed: u8) -> sp_core::sr25519::Pair {
        sp_core::sr25519::Pair::from_seed(&[seed; 32])
    }

    fn account_of(pair: &sp_core::sr25519::Pair) -> ChainAccount {
        ChainAccount::from_hex(&hex::encode(pair.public().0)).expect("a public key is 32 bytes")
    }

    fn challenge() -> String {
        format!("demiurge:1757800000:{}", "ab".repeat(32))
    }

    /// An account arrives as SS58, or as hex in the advanced field, and both
    /// forms must reach the same 32 bytes (ADR-024).
    #[test]
    fn ss58_and_hex_name_the_same_account() {
        let account = account_of(&key(3));
        let address = account.address(PREFIX);

        assert!(!address.starts_with("0x"), "an address is SS58: {address}");
        assert_eq!(
            ChainAccount::from_request(Some(&address), None, PREFIX).unwrap(),
            account
        );
        assert_eq!(
            ChainAccount::from_request(None, Some(&account.account_id_hex()), PREFIX).unwrap(),
            account
        );
    }

    /// The advanced field is explicit, so a hex account ID cannot arrive where
    /// an address is meant, and an account must be named exactly once.
    #[test]
    fn the_two_forms_are_not_interchangeable() {
        let account = account_of(&key(3));
        let address = account.address(PREFIX);
        let raw = account.account_id_hex();

        assert!(
            ChainAccount::from_request(Some(&raw), None, PREFIX).is_err(),
            "hex in the address field is refused"
        );
        assert!(
            ChainAccount::from_request(None, Some(&address), PREFIX).is_err(),
            "SS58 in the account_id field is refused"
        );
        assert!(ChainAccount::from_request(Some(&address), Some(&raw), PREFIX).is_err());
        assert!(ChainAccount::from_request(None, None, PREFIX).is_err());
        assert!(ChainAccount::from_request(Some("  "), None, PREFIX).is_err());
    }

    /// An address written for another network is refused, so a Polkadot or
    /// Kusama address pasted by mistake does not become an account here. It is
    /// an input-shape check and not chain identification (ADR-024).
    #[test]
    fn an_address_for_another_prefix_is_refused() {
        use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};

        let public = key(3).public();
        for prefix in [0u16, 2, 777] {
            let elsewhere = public.to_ss58check_with_version(Ss58AddressFormat::custom(prefix));
            assert!(
                ChainAccount::from_request(Some(&elsewhere), None, PREFIX).is_err(),
                "prefix {prefix} must be refused: {elsewhere}"
            );
        }
    }

    /// One altered character fails the checksum, which raw hex never had.
    #[test]
    fn a_bad_checksum_is_refused() {
        let address = account_of(&key(3)).address(PREFIX);
        let mut characters: Vec<char> = address.chars().collect();
        let last = characters.len() - 1;
        characters[last] = if characters[last] == 'A' { 'B' } else { 'A' };
        let mistyped: String = characters.into_iter().collect();

        assert!(ChainAccount::from_request(Some(&mistyped), None, PREFIX).is_err());
        assert!(ChainAccount::from_request(None, Some("0x1234"), PREFIX).is_err());
        assert!(ChainAccount::from_request(None, Some(&"zz".repeat(32)), PREFIX).is_err());
    }

    /// A signature nobody made, for a small-order key: R is the identity point
    /// and s is zero. Non-strict Ed25519 verification accepts it for the
    /// identity key on every message (checked against ed25519-dalek 2.2.0 on
    /// 2026-09-14), and QOR ID once verified that way.
    ///
    /// **This test is kept, and still covers something real** (ADR-023 point 5,
    /// the owner's decision of 15 September 2026). QOR ID now verifies Sr25519,
    /// so the forgery is refused because it is not a signature this service
    /// accepts at all, rather than because a strict check caught it. What the
    /// test asserts is unchanged: the bytes that were once accepted are
    /// refused. The Ed25519 half is measured alongside it, so a future change
    /// that reintroduces Ed25519 verification cannot reintroduce the forgery
    /// unnoticed.
    #[test]
    fn a_forged_signature_for_a_small_order_key_is_rejected() {
        use ed25519_dalek::{Signature, VerifyingKey};

        let mut identity = [0u8; 32];
        identity[0] = 1;
        let mut forged = [0u8; 64];
        forged[0] = 1;

        let account = ChainAccount::from_hex(&hex::encode(identity)).expect("32 bytes");

        for i in 0..16 {
            let challenge = format!("demiurge:{i}:challenge");
            assert!(
                !account.verify_challenge(&challenge, &hex::encode(forged)),
                "forged signature accepted for message {i}"
            );

            // The same bytes, measured as Ed25519: there it is strict
            // verification that refuses them, which is what R-1 names.
            if let Ok(key) = VerifyingKey::from_bytes(&identity) {
                assert!(
                    key.verify_strict(
                        challenge_message(&challenge).as_bytes(),
                        &Signature::from_bytes(&forged),
                    )
                    .is_err(),
                    "strict Ed25519 accepted the forgery for message {i}"
                );
            }
        }
    }

    /// Challenge signatures are domain-separated, so a signature made to answer
    /// a challenge can never double as a signature over anything else, such as
    /// a chain transaction (security track item 7).
    #[test]
    fn challenge_signatures_must_carry_the_domain_tag() {
        let pair = key(9);
        let account = account_of(&pair);
        let challenge = challenge();

        let bare = hex::encode(pair.sign(challenge.as_bytes()).0);
        let tagged = hex::encode(pair.sign(challenge_message(&challenge).as_bytes()).0);

        assert!(
            !account.verify_challenge(&challenge, &bare),
            "a signature over the bare challenge must be refused"
        );
        assert!(account.verify_challenge(&challenge, &tagged));
        assert_eq!(
            CHALLENGE_DOMAIN, "demiurge:qor-id:challenge:v1:",
            "must match the launcher"
        );
    }

    /// The scheme is Sr25519 (ADR-023), verified under the context the chain
    /// verifies under, so the key that opens an account here opens it there.
    #[test]
    fn a_genuine_sr25519_signature_is_accepted() {
        let pair = key(7);
        let account = account_of(&pair);
        let challenge = challenge();
        let signature = hex::encode(pair.sign(challenge_message(&challenge).as_bytes()).0);

        assert!(account.verify_challenge(&challenge, &signature));
        assert!(
            account.verify_challenge(&challenge, &format!("0x{signature}")),
            "a 0x-prefixed signature is the same signature"
        );

        // Another account's signature, a signature over another challenge, and
        // malformed bytes are all refused.
        let stranger = account_of(&key(8));
        assert!(!stranger.verify_challenge(&challenge, &signature));
        assert!(!account.verify_challenge("demiurge:1:other", &signature));
        assert!(!account.verify_challenge(&challenge, "not hex"));
        assert!(!account.verify_challenge(&challenge, &hex::encode([0u8; 64])));
    }

    /// An Ed25519 signature, from the scheme QOR ID used to verify, is not
    /// accepted for the same 32 bytes. One scheme, chosen once: ADR-023 ruled
    /// out a compatibility path carrying two.
    #[test]
    fn an_ed25519_signature_is_not_accepted() {
        use ed25519_dalek::{Signer, SigningKey};

        let signing = SigningKey::from_bytes(&[11u8; 32]);
        let account = ChainAccount::from_hex(&hex::encode(signing.verifying_key().to_bytes()))
            .expect("32 bytes");
        let challenge = challenge();
        let signature = hex::encode(
            signing
                .sign(challenge_message(&challenge).as_bytes())
                .to_bytes(),
        );

        assert!(!account.verify_challenge(&challenge, &signature));
    }
}
