//! A person's own account page: change the password, add or change the email address.
//!
//! - GET /account: the page, two forms. Changes nothing.
//! - POST /account/password: the current password, then the new one twice.
//! - POST /account/email: the password, then the address; a confirmation link goes to the address.
//! - POST /api/v1/profile/password: the same password change for an app holding an access token.
//!
//! Every form asks for the account's current password, so there is no session on this page to steal
//! and no request another site could forge: nothing here works without the password. The password is
//! checked by the same function as sign-in (`auth::authenticate_password`), with the same refusals and
//! lockout. A changed password signs out every session on the account, as a reset does, and cancels a
//! pending change of email address.
//!
//! Before this page an account without a confirmed email address could neither reset nor change its
//! password, which is how the owner was locked out of their own account on 5 October 2026.

use axum::{
    Extension, Json,
    extract::{Form, State},
    http::StatusCode,
    response::Response,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::handlers::auth::authenticate_password;
use crate::handlers::pages::{failure, page};
use crate::handlers::profile::request_email_change;
use crate::models::user::ChangeEmailRequest;
use crate::services::auth_service::AuthService;
use crate::services::email_service::escape;
use crate::services::session_service::SessionService;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct PasswordForm {
    identifier: String,
    current_password: String,
    new_password: String,
    confirm_password: String,
}

#[derive(Deserialize)]
pub struct EmailForm {
    identifier: String,
    password: String,
    email: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Set a new password on an account whose current one was just proven. Every session ends before
/// the change commits: if they cannot be ended, nothing changes.
pub(crate) async fn change_password(
    state: &AppState,
    user_id: Uuid,
    new_password: &str,
) -> AppResult<()> {
    let min = state.config.security.password_min_length;
    if new_password.len() < min {
        return Err(AppError::ValidationError(format!(
            "Use at least {min} characters"
        )));
    }
    let password_hash = AuthService::hash_password(new_password)?;
    let mut tx = state.db.begin().await?;
    sqlx::query(
        "UPDATE users SET password_hash = $1, pending_email = NULL, pending_email_token = NULL, pending_email_expires_at = NULL, updated_at = NOW() WHERE id = $2",
    )
    .bind(&password_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;
    SessionService::new(state.redis.clone(), state.config.jwt.clone())
        .delete_all_sessions(user_id)
        .await?;
    tx.commit().await?;
    Ok(())
}

fn problem(message: Option<&str>) -> String {
    message
        .map(|m| format!(r#"<p class="problem" role="alert">{}</p>"#, escape(m)))
        .unwrap_or_default()
}

fn account_page(
    status: StatusCode,
    min: usize,
    password_problem: Option<&str>,
    email_problem: Option<&str>,
) -> Response {
    let password_problem = problem(password_problem);
    let email_problem = problem(email_problem);
    page(
        status,
        "Your QOR ID",
        &format!(
            r#"<p>Change your password, or add the email address your account recovers through. Each form asks for your current password; nothing changes until you press its button.</p>
<hr>
<h2 id="password">Change your password</h2>
{password_problem}<form method="post" action="/account/password">
<label for="p_identifier">Username or email</label>
<input type="text" id="p_identifier" name="identifier" autocomplete="username" required>
<label for="current_password">Current password</label>
<input type="password" id="current_password" name="current_password" autocomplete="current-password" required>
<label for="new_password">New password</label>
<input type="password" id="new_password" name="new_password" autocomplete="new-password" required minlength="{min}">
<label for="confirm_password">Repeat new password</label>
<input type="password" id="confirm_password" name="confirm_password" autocomplete="new-password" required minlength="{min}">
<p class="muted">At least {min} characters. Changing it signs out every session on the account.</p>
<button type="submit">Change password</button>
</form>
<hr>
<h2 id="email">Add or change your email address</h2>
{email_problem}<form method="post" action="/account/email">
<label for="e_identifier">Username or email</label>
<input type="text" id="e_identifier" name="identifier" autocomplete="username" required>
<label for="e_password">Password</label>
<input type="password" id="e_password" name="password" autocomplete="current-password" required>
<label for="email">Email address</label>
<input type="email" id="email" name="email" autocomplete="email" required>
<p class="muted">A link goes to the address; it becomes your account's address once you open it. Password resets go to a confirmed address.</p>
<button type="submit">Send confirmation link</button>
</form>
<p class="muted after">Forgotten your password? <a href="/reset-password">Get a reset link</a>.</p>"#
        ),
    )
}

/// GET /account: the page. Changes nothing.
pub async fn account(State(state): State<Arc<AppState>>) -> Response {
    account_page(
        StatusCode::OK,
        state.config.security.password_min_length,
        None,
        None,
    )
}

/// POST /account/password: the current password proves the account; the new one replaces it.
pub async fn change_password_page(
    State(state): State<Arc<AppState>>,
    Form(form): Form<PasswordForm>,
) -> Response {
    let min = state.config.security.password_min_length;
    let refuse = |message: &str| account_page(StatusCode::BAD_REQUEST, min, Some(message), None);
    if form.new_password != form.confirm_password {
        return refuse("The two new passwords are different. Nothing was changed.");
    }
    if form.new_password.len() < min {
        return refuse(&format!(
            "Use at least {min} characters. Nothing was changed."
        ));
    }
    let user = match authenticate_password(&state, form.identifier.trim(), &form.current_password)
        .await
    {
        Ok(user) => user,
        Err(AppError::InvalidCredentials) => {
            return account_page(
                StatusCode::UNAUTHORIZED,
                min,
                Some(
                    "That username or email and current password do not match. Nothing was changed.",
                ),
                None,
            );
        }
        Err(e) => return failure(e),
    };
    match change_password(&state, user.id, &form.new_password).await {
        Ok(()) => page(
            StatusCode::OK,
            "Password changed",
            &format!(
                r#"<p>Sign in to the QOR ID <span class="id">{}</span> with your new password.</p>
<p class="muted">Every session that was signed in has been signed out.</p>"#,
                escape(&user.qor_id())
            ),
        ),
        Err(e) => failure(e),
    }
}

/// POST /account/email: the password proves the account; a confirmation link goes to the address.
pub async fn change_email_page(
    State(state): State<Arc<AppState>>,
    Form(form): Form<EmailForm>,
) -> Response {
    let min = state.config.security.password_min_length;
    let user = match authenticate_password(&state, form.identifier.trim(), &form.password).await {
        Ok(user) => user,
        Err(AppError::InvalidCredentials) => {
            return account_page(
                StatusCode::UNAUTHORIZED,
                min,
                None,
                Some("That username or email and password do not match. Nothing was changed."),
            );
        }
        Err(e) => return failure(e),
    };
    match request_email_change(
        State(state),
        Extension(user.id),
        Json(ChangeEmailRequest {
            email: form.email,
            password: form.password,
        }),
    )
    .await
    {
        Ok(Json(body)) => page(
            StatusCode::OK,
            "Check your email",
            &format!(
                r#"<p>{}</p>
<p class="muted">Open the newest message at that address, then press the button on the page its link opens.</p>"#,
                escape(
                    body["message"]
                        .as_str()
                        .unwrap_or("A confirmation link is on its way.")
                )
            ),
        ),
        Err(AppError::ValidationError(message)) => {
            account_page(StatusCode::BAD_REQUEST, min, None, Some(&message))
        }
        Err(AppError::RateLimited(message)) => {
            account_page(StatusCode::TOO_MANY_REQUESTS, min, None, Some(&message))
        }
        Err(AppError::Undeliverable(message)) => {
            account_page(StatusCode::UNPROCESSABLE_ENTITY, min, None, Some(&message))
        }
        Err(e) => failure(e),
    }
}

/// POST /api/v1/profile/password: the same change for an app with an access token. The access token
/// alone is not enough: the current password is asked too, so a stolen token cannot lock the owner out.
pub async fn change_password_api(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<Value>> {
    let identifier: String = sqlx::query_scalar("SELECT username FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidToken)?;
    let user = authenticate_password(&state, &identifier, &req.current_password).await?;
    if user.id != user_id {
        return Err(AppError::InvalidCredentials);
    }
    change_password(&state, user_id, &req.new_password).await?;
    Ok(Json(json!({
        "message": "Password changed. Every session on the account has been signed out."
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::models::Session;
    use crate::services::session_service::NewSession;
    use crate::services::{EmailConfig, EmailService};
    use axum::body::to_bytes;
    use sqlx::PgPool;

    const PASSWORD: &str = "the current password";
    const NEW: &str = "a brand new password";

    fn state(db: PgPool) -> Arc<AppState> {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL").expect("QOR_AUTH_TEST_REDIS_URL");
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

    async fn account_with(db: &PgPool, username: &str) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO users (username, password_hash, email_verified) VALUES ($1, $2, TRUE) RETURNING id",
        )
        .bind(username)
        .bind(AuthService::hash_password(PASSWORD).expect("hash"))
        .fetch_one(db)
        .await
        .expect("account")
    }

    async fn signed_in(state: &AppState, user: Uuid) {
        SessionService::new(state.redis.clone(), state.config.jwt.clone())
            .create_session(NewSession {
                user_id: user,
                qor_id: "owner",
                role: Some("user"),
                device_id: "test-device",
                ip_address: "127.0.0.1",
                user_agent: None,
                scopes: Session::default_scopes(),
            })
            .await
            .expect("session");
    }

    async fn sessions(state: &AppState, user: Uuid) -> usize {
        SessionService::new(state.redis.clone(), state.config.jwt.clone())
            .list_sessions(user)
            .await
            .expect("sessions")
            .len()
    }

    async fn html(response: Response) -> (StatusCode, String) {
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        (status, String::from_utf8_lossy(&bytes).into_owned())
    }

    fn form(identifier: &str, current: &str, new: &str, confirm: &str) -> Form<PasswordForm> {
        Form(PasswordForm {
            identifier: identifier.into(),
            current_password: current.into(),
            new_password: new.into(),
            confirm_password: confirm.into(),
        })
    }

    async fn signs_in_with(state: &AppState, password: &str) -> bool {
        authenticate_password(state, "owner", password)
            .await
            .is_ok()
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn the_page_asks_for_the_current_password_and_runs_no_script(db: PgPool) {
        let (status, page) = html(account(State(state(db))).await).await;
        assert_eq!(status, StatusCode::OK);
        assert!(page.contains(r#"action="/account/password""#));
        assert!(page.contains(r#"action="/account/email""#));
        assert!(page.contains(r#"name="current_password""#));
        assert!(!page.contains("<script"));
    }

    /// A password changes only with the current one, both new entries agreeing, and enough length;
    /// a change signs every session out, and the old password stops working.
    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_password_changes_only_with_the_current_one_and_signs_everything_out(db: PgPool) {
        let state = state(db.clone());
        let user = account_with(&db, "owner").await;
        signed_in(&state, user).await;
        signed_in(&state, user).await;
        assert_eq!(sessions(&state, user).await, 2);

        let refused = change_password_page(
            State(state.clone()),
            form("owner", "wrong password", NEW, NEW),
        )
        .await;
        let (status, page) = html(refused).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(page.contains("do not match"));
        let mismatched = change_password_page(
            State(state.clone()),
            form("owner", PASSWORD, NEW, "something else"),
        )
        .await;
        assert_eq!(mismatched.status(), StatusCode::BAD_REQUEST);
        let short = change_password_page(
            State(state.clone()),
            form("owner", PASSWORD, "short", "short"),
        )
        .await;
        assert_eq!(short.status(), StatusCode::BAD_REQUEST);
        assert!(signs_in_with(&state, PASSWORD).await, "nothing changed yet");
        assert_eq!(sessions(&state, user).await, 2);

        let changed =
            change_password_page(State(state.clone()), form("OWNER", PASSWORD, NEW, NEW)).await;
        let (status, page) = html(changed).await;
        assert_eq!(status, StatusCode::OK, "{page}");
        assert!(page.contains("Password changed"));
        assert!(!page.contains(NEW), "the page never repeats the password");
        assert!(signs_in_with(&state, NEW).await);
        assert!(!signs_in_with(&state, PASSWORD).await);
        assert_eq!(sessions(&state, user).await, 0, "every session signed out");
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn an_app_changes_the_password_only_with_the_current_one_too(db: PgPool) {
        let state = state(db.clone());
        let user = account_with(&db, "owner").await;
        let refused = change_password_api(
            State(state.clone()),
            Extension(user),
            Json(ChangePasswordRequest {
                current_password: "wrong password".into(),
                new_password: NEW.into(),
            }),
        )
        .await;
        assert!(matches!(refused, Err(AppError::InvalidCredentials)));
        let Json(answer) = change_password_api(
            State(state.clone()),
            Extension(user),
            Json(ChangePasswordRequest {
                current_password: PASSWORD.into(),
                new_password: NEW.into(),
            }),
        )
        .await
        .expect("changed");
        assert!(
            answer["message"]
                .as_str()
                .is_some_and(|m| m.contains("signed out"))
        );
        assert!(signs_in_with(&state, NEW).await);
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn an_email_address_is_added_only_with_the_password(db: PgPool) {
        let state = state(db.clone());
        account_with(&db, "owner").await;
        let refused = change_email_page(
            State(state.clone()),
            Form(EmailForm {
                identifier: "owner".into(),
                password: "wrong password".into(),
                email: "owner@example.invalid".into(),
            }),
        )
        .await;
        let (status, page) = html(refused).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(page.contains("do not match"));
        // With the right password the request reaches the email step; this service has no email, and
        // says so rather than claiming a link was sent.
        let unavailable = change_email_page(
            State(state),
            Form(EmailForm {
                identifier: "owner".into(),
                password: PASSWORD.into(),
                email: "owner@example.invalid".into(),
            }),
        )
        .await;
        assert_eq!(unavailable.status(), StatusCode::SERVICE_UNAVAILABLE);
        let pending: Option<String> =
            sqlx::query_scalar("SELECT pending_email FROM users WHERE username = 'owner'")
                .fetch_one(&db)
                .await
                .expect("row");
        assert!(pending.is_none(), "nothing pending without a link sent");
    }
}
