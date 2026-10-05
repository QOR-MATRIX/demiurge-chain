//! The pages the links in QOR ID's emails open.
//!
//! - GET /verify-email?token=…, POST /verify-email
//! - GET /reset-password?token=…, POST /reset-password
//! - POST /resend-verification, POST /forgot-password: a new link, asked for on a page
//!
//! Email security products open links on their own, often before the person does. A page that spent
//! its token on loading would burn a one-use link before anyone clicked it. So opening a link only
//! reads: the page says who the link is for and shows a button, or says the link no longer works.
//! The token is spent only by the form that button submits. This holds for all three kinds of link:
//! address verification, confirmation of a new address, and password reset.
//!
//! A used or expired link says what happened and offers a new one on the same page.
//!
//! The work is done by the same handlers as the JSON API, so the rules are the same. Pages run no
//! script, cannot be framed, cached or indexed, and send no referrer, so the token in the address is
//! not passed on.

use axum::{
    Json,
    extract::{Form, Query, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

use crate::error::AppError;
use crate::handlers::auth::{self, VerifyEmailRequest};
use crate::models::user::{
    ForgotPasswordRequest, ResendVerificationRequest, ResetPasswordWithTokenRequest,
};
use crate::services::email_service::escape;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LinkQuery {
    token: Option<String>,
}

#[derive(Deserialize)]
pub struct TokenForm {
    token: String,
}

#[derive(Deserialize)]
pub struct NewPasswordForm {
    token: String,
    new_password: String,
    confirm_password: String,
}

#[derive(Deserialize)]
pub struct IdentifierForm {
    identifier: String,
}

/// Tokens are 32 hexadecimal characters; anything far from that is not looked up.
fn plausible(token: &str) -> bool {
    !token.is_empty() && token.len() <= 128
}

// The Architect theme (`docs/design/DESIGN_SYSTEM.md`), as in the emails.
const STYLE: &str = r#"
:root { color-scheme: dark; }
* { box-sizing: border-box; }
body { margin: 0; padding: 48px 16px; background: #0B0C10; color: #D7D8DA; font-family: 'Segoe UI Variable Text', 'Segoe UI', -apple-system, BlinkMacSystemFont, Roboto, Helvetica, Arial, sans-serif; font-size: 15px; line-height: 1.6; }
main { max-width: 480px; margin: 0 auto; }
.eyebrow { margin: 0 0 16px; font-size: 11px; font-weight: 600; letter-spacing: 0.18em; text-transform: uppercase; color: #93A0AE; }
.panel { background: #12151C; border: 1px solid #333A47; border-radius: 2px; padding: 32px; }
h1 { margin: 0 0 16px; font-size: 24px; line-height: 1.3; font-weight: 600; color: #FFFFFF; }
p { margin: 0 0 16px; }
.muted { color: #93A0AE; font-size: 13px; }
.after { margin: 16px 0 0; }
.id { font-family: 'Cascadia Mono', 'SF Mono', Consolas, Menlo, monospace; color: #FFFFFF; overflow-wrap: anywhere; }
hr { border: 0; border-top: 1px solid #232936; margin: 24px 0; }
form { margin: 24px 0 0; }
label { display: block; margin: 0 0 6px; font-size: 11px; font-weight: 600; letter-spacing: 0.1em; text-transform: uppercase; color: #93A0AE; }
input[type=password], input[type=text] { display: block; width: 100%; margin: 0 0 16px; padding: 10px 12px; background: #0E1117; border: 1px solid #333A47; border-radius: 2px; color: #FFFFFF; font: inherit; }
input[type=password]:focus, input[type=text]:focus { outline: none; border-color: #FF6A00; }
button { display: inline-block; padding: 12px 24px; background: #FF6A00; border: 0; border-radius: 2px; color: #06070A; font: inherit; font-size: 13px; font-weight: 600; letter-spacing: 0.1em; text-transform: uppercase; cursor: pointer; }
button:hover { background: #FF9142; }
button:focus-visible, input:focus-visible { outline: 3px solid #FF9142; outline-offset: 2px; }
.problem { margin: 0 0 16px; padding: 10px 12px; border-left: 2px solid #CF6679; background: rgba(207, 102, 121, 0.05); color: #CF6679; font-size: 13px; }
"#;

/// A page, with the headers every link page carries.
fn page(status: StatusCode, title: &str, body: &str) -> Response {
    page_sending_to(status, title, body, None)
}

/// A page whose form, once submitted, may be redirected to one other origin as well as this one:
/// QOR ID's sign-in page for an app, which ends in a redirect to that app. Browsers apply
/// `form-action` to the redirect after a submission too, so without the app's origin here the
/// browser would refuse to return the person to it.
pub(crate) fn page_sending_to(
    status: StatusCode,
    title: &str,
    body: &str,
    origin: Option<&str>,
) -> Response {
    let title = escape(title);
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="dark">
<meta name="robots" content="noindex, nofollow">
<meta name="referrer" content="no-referrer">
<title>{title} · QOR ID</title>
<style>{STYLE}</style>
</head>
<body>
<main>
<p class="eyebrow">Demiurge &middot; QOR ID</p>
<section class="panel">
<h1>{title}</h1>
{body}
</section>
</main>
</body>
</html>
"#
    );

    let mut response = (status, html).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    let csp = format!(
        "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'{}; frame-ancestors 'none'; base-uri 'none'",
        origin.map(|o| format!(" {o}")).unwrap_or_default()
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_str(&csp).unwrap_or_else(|_| {
            HeaderValue::from_static(
                "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'; base-uri 'none'",
            )
        }),
    );
    headers.insert(
        "x-robots-tag",
        HeaderValue::from_static("noindex, nofollow"),
    );
    response
}

/// Anything that is not the link's own outcome. Nothing is described as done.
fn failure(error: AppError) -> Response {
    match error {
        AppError::ServiceUnavailable(_) => page(
            StatusCode::SERVICE_UNAVAILABLE,
            "Email is unavailable",
            "<p>This service cannot send email right now, so no link was sent. Try again later.</p>",
        ),
        other => {
            tracing::error!("A link page could not be completed: {:?}", other);
            page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Something went wrong",
                "<p>Nothing was saved. Try again in a moment.</p>",
            )
        }
    }
}

// ------------------------------------------------------------------------- verification

struct Confirmation {
    username: String,
    new_address: bool,
}

/// Who a verification token is for, read without changing anything. The conditions are those
/// `auth::verify_email` applies.
async fn confirmation_for(state: &AppState, token: &str) -> Result<Option<Confirmation>, AppError> {
    if !plausible(token) {
        return Ok(None);
    }
    let row: Option<(String, bool)> = sqlx::query_as(
        r#"
        SELECT username, FALSE FROM users
        WHERE email_verification_token = $1 AND email_verification_expires_at > NOW()
        UNION ALL
        SELECT username, TRUE FROM users
        WHERE pending_email_token = $1 AND pending_email_expires_at > NOW() AND status = 'active'
        LIMIT 1
        "#,
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await?;
    Ok(row.map(|(username, new_address)| Confirmation {
        username,
        new_address,
    }))
}

fn verification_link_gone(status: StatusCode) -> Response {
    page(
        status,
        "This link no longer works",
        r#"<p>It has already been used, or it has expired. Confirmation links last 24 hours and work once.</p>
<p>If you already confirmed this address, there is nothing more to do.</p>
<hr>
<form method="post" action="/resend-verification">
<label for="identifier">Username or email address</label>
<input type="text" id="identifier" name="identifier" autocomplete="username" required maxlength="255">
<button type="submit">Send a new link</button>
</form>
<p class="muted after">A new link goes to an account whose address is not yet confirmed. If you were changing an account's email address, sign in and ask for the change again.</p>"#,
    )
}

/// GET /verify-email?token=…: shows who the link is for and a button. Changes nothing.
pub async fn verify_email_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LinkQuery>,
) -> Response {
    let token = query.token.unwrap_or_default();
    let link = match confirmation_for(&state, &token).await {
        Ok(Some(link)) => link,
        Ok(None) => return verification_link_gone(StatusCode::GONE),
        Err(e) => return failure(e),
    };
    let (title, lead) = if link.new_address {
        (
            "Confirm your new email address",
            "Press the button to make this the email address of the QOR ID",
        )
    } else {
        (
            "Confirm your email address",
            "Press the button to confirm this address for the QOR ID",
        )
    };
    let username = escape(&link.username);
    let token = escape(&token);
    page(
        StatusCode::OK,
        title,
        &format!(
            r#"<p>{lead} <span class="id">{username}</span>.</p>
<p class="muted">Nothing changes until you press it.</p>
<form method="post" action="/verify-email">
<input type="hidden" name="token" value="{token}">
<button type="submit">Confirm email address</button>
</form>"#
        ),
    )
}

/// POST /verify-email: the button. Spends the token.
pub async fn confirm_email(
    State(state): State<Arc<AppState>>,
    Form(form): Form<TokenForm>,
) -> Response {
    if !plausible(&form.token) {
        return verification_link_gone(StatusCode::GONE);
    }
    match auth::verify_email(State(state), Json(VerifyEmailRequest { token: form.token })).await {
        Ok(Json(body)) => {
            let qor_id = escape(body["qor_id"].as_str().unwrap_or_default());
            page(
                StatusCode::OK,
                "Email address confirmed",
                &format!(
                    r#"<p>The QOR ID <span class="id">{qor_id}</span> now uses this address. Password reset links will come here.</p>
<p class="muted">You can close this page.</p>"#
                ),
            )
        }
        Err(AppError::ValidationError(message)) if message.starts_with("Invalid or expired") => {
            verification_link_gone(StatusCode::GONE)
        }
        Err(AppError::ValidationError(message)) => page(
            StatusCode::CONFLICT,
            "This address cannot be confirmed",
            &format!(
                r#"<p>{}</p>
<p class="muted">Sign in and choose a different address.</p>"#,
                escape(&message)
            ),
        ),
        Err(e) => failure(e),
    }
}

/// POST /resend-verification: a new verification link, asked for on the page.
pub async fn request_verification_link(
    State(state): State<Arc<AppState>>,
    Form(form): Form<IdentifierForm>,
) -> Response {
    let identifier = form.identifier.trim().to_string();
    if identifier.is_empty() {
        return verification_link_gone(StatusCode::BAD_REQUEST);
    }
    match auth::resend_verification(State(state), Json(ResendVerificationRequest { identifier }))
        .await
    {
        Ok(Json(body)) => link_on_its_way(&body),
        Err(e) => failure(e),
    }
}

// ------------------------------------------------------------------------- password reset

/// Whose a reset token is, read without changing anything. The conditions are those
/// `auth::reset_password` applies.
async fn reset_link_for(state: &AppState, token: &str) -> Result<Option<String>, AppError> {
    if !plausible(token) {
        return Ok(None);
    }
    Ok(sqlx::query_scalar(
        r#"
        SELECT u.username FROM password_resets r JOIN users u ON u.id = r.user_id
        WHERE r.token = $1 AND r.expires_at > NOW() AND r.used_at IS NULL
        LIMIT 1
        "#,
    )
    .bind(token)
    .fetch_optional(&state.db)
    .await?)
}

fn reset_link_gone(status: StatusCode) -> Response {
    page(
        status,
        "This link no longer works",
        r#"<p>It has already been used, or it has expired. Reset links last 1 hour and work once.</p>
<p>If you already set a new password with it, sign in with that password.</p>
<hr>
<form method="post" action="/forgot-password">
<label for="identifier">Username or email address</label>
<input type="text" id="identifier" name="identifier" autocomplete="username" required maxlength="255">
<button type="submit">Send a new reset link</button>
</form>
<p class="muted after">A reset link goes to the account's confirmed email address. An account without one resets its password with a backup code.</p>"#,
    )
}

fn password_form(
    status: StatusCode,
    token: &str,
    username: &str,
    min: usize,
    problem: Option<&str>,
) -> Response {
    let problem = problem
        .map(|problem| format!(r#"<p class="problem" role="alert">{}</p>"#, escape(problem)))
        .unwrap_or_default();
    let username = escape(username);
    let token = escape(token);
    page(
        status,
        "Choose a new password",
        &format!(
            r#"<p>For the QOR ID <span class="id">{username}</span>. Nothing changes until you press the button.</p>
{problem}<form method="post" action="/reset-password">
<input type="hidden" name="token" value="{token}">
<label for="new_password">New password</label>
<input type="password" id="new_password" name="new_password" autocomplete="new-password" required minlength="{min}">
<label for="confirm_password">Repeat new password</label>
<input type="password" id="confirm_password" name="confirm_password" autocomplete="new-password" required minlength="{min}">
<p class="muted">At least {min} characters. Setting it signs out every session on the account.</p>
<button type="submit">Set new password</button>
</form>"#
        ),
    )
}

/// GET /reset-password?token=…: shows the password form. Changes nothing.
pub async fn reset_password_page(
    State(state): State<Arc<AppState>>,
    Query(query): Query<LinkQuery>,
) -> Response {
    let token = query.token.unwrap_or_default();
    match reset_link_for(&state, &token).await {
        Ok(Some(username)) => password_form(
            StatusCode::OK,
            &token,
            &username,
            state.config.security.password_min_length,
            None,
        ),
        Ok(None) => reset_link_gone(StatusCode::GONE),
        Err(e) => failure(e),
    }
}

/// POST /reset-password: the button. Spends the token only when the new password is accepted.
pub async fn set_password(
    State(state): State<Arc<AppState>>,
    Form(form): Form<NewPasswordForm>,
) -> Response {
    let username = match reset_link_for(&state, &form.token).await {
        Ok(Some(username)) => username,
        Ok(None) => return reset_link_gone(StatusCode::GONE),
        Err(e) => return failure(e),
    };
    let min = state.config.security.password_min_length;
    if form.new_password != form.confirm_password {
        return password_form(
            StatusCode::BAD_REQUEST,
            &form.token,
            &username,
            min,
            Some("The two passwords are different. Nothing was changed."),
        );
    }
    if form.new_password.len() < min {
        return password_form(
            StatusCode::BAD_REQUEST,
            &form.token,
            &username,
            min,
            Some(&format!(
                "Use at least {min} characters. Nothing was changed."
            )),
        );
    }

    match auth::reset_password(
        State(state),
        Json(ResetPasswordWithTokenRequest {
            token: form.token,
            new_password: form.new_password,
        }),
    )
    .await
    {
        Ok(_) => page(
            StatusCode::OK,
            "Password changed",
            &format!(
                r#"<p>Sign in to the QOR ID <span class="id">{}</span> with your new password.</p>
<p class="muted">Every session that was signed in has been signed out.</p>"#,
                escape(&username)
            ),
        ),
        Err(AppError::ValidationError(message)) if message.starts_with("Invalid or expired") => {
            reset_link_gone(StatusCode::GONE)
        }
        Err(e) => failure(e),
    }
}

/// POST /forgot-password: a new reset link, asked for on the page.
pub async fn request_reset_link(
    State(state): State<Arc<AppState>>,
    Form(form): Form<IdentifierForm>,
) -> Response {
    let identifier = form.identifier.trim().to_string();
    if identifier.is_empty() {
        return reset_link_gone(StatusCode::BAD_REQUEST);
    }
    match auth::forgot_password(State(state), Json(ForgotPasswordRequest { identifier })).await {
        Ok(Json(body)) => link_on_its_way(&body),
        Err(e) => failure(e),
    }
}

/// The API's own answer, which is the same for every identifier.
fn link_on_its_way(body: &Value) -> Response {
    page(
        StatusCode::OK,
        "Check your email",
        &format!(
            r#"<p>{}</p>
<p class="muted">Open the newest message, then press the button on the page its link opens.</p>"#,
            escape(body["message"].as_str().unwrap_or_default())
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::auth_service::AuthService;
    use crate::services::{EmailConfig, EmailService};
    use axum::http::HeaderMap;
    use sqlx::PgPool;
    use uuid::Uuid;

    const PASSWORD: &str = "correct horse battery staple";
    const NEW_PASSWORD: &str = "a new password set on the page";
    const TOKEN: &str = "0123456789abcdef0123456789abcdef";
    const OTHER_TOKEN: &str = "fedcba9876543210fedcba9876543210";

    fn state(db: PgPool, email_configured: bool) -> Arc<AppState> {
        let (key, from) = if email_configured {
            ("re_test_key", "Test <noreply@example.invalid>")
        } else {
            ("", "")
        };
        let email = EmailService::new(EmailConfig {
            resend_api_key: key.into(),
            from: from.into(),
            base_url: "https://example.invalid".into(),
            // Nothing listens here, so no mail leaves the machine.
            api_url: "http://127.0.0.1:9".into(),
        });
        let redis_url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        let redis = deadpool_redis::Config::from_url(redis_url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        Arc::new(AppState::new(AppConfig::default(), db, redis, email))
    }

    async fn read(response: Response) -> (StatusCode, HeaderMap, String) {
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        (
            status,
            headers,
            String::from_utf8(bytes.to_vec()).expect("utf-8"),
        )
    }

    fn link(token: Option<&str>) -> Query<LinkQuery> {
        Query(LinkQuery {
            token: token.map(str::to_string),
        })
    }

    /// An account with an unverified address and a verification token lasting `lasts`.
    async fn unverified(db: &PgPool, username: &str, token: &str, lasts: &str) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified, email_verification_token, email_verification_expires_at)
             VALUES ($1, $2, 1, 'x', FALSE, $3, NOW() + $4::interval) RETURNING id",
        )
        .bind(format!("{username}@example.invalid"))
        .bind(username)
        .bind(token)
        .bind(lasts)
        .fetch_one(db)
        .await
        .expect("account")
    }

    async fn verification(db: &PgPool, user: Uuid) -> (bool, Option<String>) {
        sqlx::query_as("SELECT email_verified, email_verification_token FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(db)
            .await
            .expect("row")
    }

    /// An account with a verified address and a reset token lasting `lasts`.
    async fn with_reset(db: &PgPool, username: &str, token: &str, lasts: &str, used: bool) -> Uuid {
        let hash = AuthService::hash_password(PASSWORD).expect("hash");
        let user: Uuid = sqlx::query_scalar(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified) VALUES ($1, $2, 1, $3, TRUE) RETURNING id",
        )
        .bind(format!("{username}@example.invalid"))
        .bind(username)
        .bind(hash)
        .fetch_one(db)
        .await
        .expect("account");
        sqlx::query(
            "INSERT INTO password_resets (user_id, token, expires_at, used_at) VALUES ($1, $2, NOW() + $3::interval, CASE WHEN $4 THEN NOW() END)",
        )
        .bind(user)
        .bind(token)
        .bind(lasts)
        .bind(used)
        .execute(db)
        .await
        .expect("reset");
        user
    }

    async fn spent(db: &PgPool, token: &str) -> bool {
        sqlx::query_scalar("SELECT used_at IS NOT NULL FROM password_resets WHERE token = $1")
            .bind(token)
            .fetch_one(db)
            .await
            .expect("reset")
    }

    async fn password_hash(db: &PgPool, user: Uuid) -> String {
        sqlx::query_scalar("SELECT password_hash FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(db)
            .await
            .expect("hash")
    }

    fn password(new: &str, repeated: &str) -> Form<NewPasswordForm> {
        Form(NewPasswordForm {
            token: TOKEN.into(),
            new_password: new.into(),
            confirm_password: repeated.into(),
        })
    }

    #[sqlx::test]
    async fn opening_a_verification_link_changes_nothing_and_shows_a_button(db: PgPool) {
        let state = state(db.clone(), false);
        let user = unverified(&db, "opened", TOKEN, "1 day").await;

        // A mail scanner may open a link several times before the person does.
        for _ in 0..3 {
            let (status, _, html) =
                read(verify_email_page(State(state.clone()), link(Some(TOKEN))).await).await;
            assert_eq!(status, StatusCode::OK);
            assert!(html.contains(r#"<form method="post" action="/verify-email">"#));
            assert!(html.contains(&format!(r#"name="token" value="{TOKEN}""#)));
            assert!(html.contains("Confirm email address"));
            assert!(html.contains("opened"));
        }
        assert_eq!(
            verification(&db, user).await,
            (false, Some(TOKEN.to_string()))
        );
    }

    #[sqlx::test]
    async fn pressing_the_button_confirms_the_address(db: PgPool) {
        let state = state(db.clone(), false);
        let user = unverified(&db, "pressed", TOKEN, "1 day").await;

        let (status, _, html) = read(
            confirm_email(
                State(state),
                Form(TokenForm {
                    token: TOKEN.into(),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Email address confirmed"));
        assert!(html.contains("pressed"));
        assert!(!html.contains("pressed#"), "no #0001 is shown (ADR-075)");
        assert_eq!(verification(&db, user).await, (true, None));
    }

    #[sqlx::test]
    async fn a_new_address_is_confirmed_by_the_button_not_by_opening_the_link(db: PgPool) {
        let state = state(db.clone(), false);
        let user = with_reset(&db, "mover", OTHER_TOKEN, "1 hour", false).await;
        sqlx::query(
            "UPDATE users SET pending_email = 'moved@example.invalid', pending_email_token = $2, pending_email_expires_at = NOW() + INTERVAL '1 day' WHERE id = $1",
        )
        .bind(user)
        .bind(TOKEN)
        .execute(&db)
        .await
        .expect("pending");
        let row = || async {
            sqlx::query_as::<_, (Option<String>, Option<String>)>(
                "SELECT email, pending_email FROM users WHERE id = $1",
            )
            .bind(user)
            .fetch_one(&db)
            .await
            .expect("row")
        };

        let (status, _, html) =
            read(verify_email_page(State(state.clone()), link(Some(TOKEN))).await).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Confirm your new email address"));
        assert_eq!(
            row().await,
            (
                Some("mover@example.invalid".into()),
                Some("moved@example.invalid".into())
            )
        );

        let (status, _, _) = read(
            confirm_email(
                State(state),
                Form(TokenForm {
                    token: TOKEN.into(),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(row().await, (Some("moved@example.invalid".into()), None));
    }

    #[sqlx::test]
    async fn a_used_expired_or_missing_verification_link_says_so_and_offers_a_new_one(db: PgPool) {
        let state = state(db.clone(), false);
        let user = unverified(&db, "lapsed", TOKEN, "-1 day").await;

        for token in [Some(TOKEN), Some(OTHER_TOKEN), Some(""), None] {
            let (status, _, html) =
                read(verify_email_page(State(state.clone()), link(token)).await).await;
            assert_eq!(status, StatusCode::GONE, "{token:?}");
            assert!(html.contains("This link no longer works"));
            assert!(html.contains("Confirmation links last 24 hours and work once."));
            assert!(html.contains(r#"action="/resend-verification""#));
            assert!(!html.contains(TOKEN), "the page does not repeat the token");
        }

        let (status, _, _) = read(
            confirm_email(
                State(state),
                Form(TokenForm {
                    token: TOKEN.into(),
                }),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::GONE);
        assert_eq!(
            verification(&db, user).await,
            (false, Some(TOKEN.to_string()))
        );
    }

    #[sqlx::test]
    async fn opening_a_reset_link_changes_nothing(db: PgPool) {
        let state = state(db.clone(), false);
        let user = with_reset(&db, "resetting", TOKEN, "1 hour", false).await;
        let before = password_hash(&db, user).await;

        for _ in 0..3 {
            let (status, _, html) =
                read(reset_password_page(State(state.clone()), link(Some(TOKEN))).await).await;
            assert_eq!(status, StatusCode::OK);
            assert!(html.contains(r#"<form method="post" action="/reset-password">"#));
            assert!(html.contains(r#"name="new_password" autocomplete="new-password""#));
            assert!(html.contains("resetting"));
        }
        assert!(!spent(&db, TOKEN).await);
        assert_eq!(password_hash(&db, user).await, before);
    }

    #[sqlx::test]
    async fn a_mismatched_or_short_password_changes_nothing_and_keeps_the_link(db: PgPool) {
        let state = state(db.clone(), false);
        let user = with_reset(&db, "careful", TOKEN, "1 hour", false).await;
        let before = password_hash(&db, user).await;

        let (status, _, html) = read(
            set_password(
                State(state.clone()),
                password("one new password here", "another new password"),
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(html.contains("The two passwords are different. Nothing was changed."));
        assert!(
            html.contains(TOKEN),
            "the form is shown again with its link"
        );

        let (status, _, html) =
            read(set_password(State(state), password("short", "short")).await).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(html.contains("Use at least 12 characters."));

        assert!(!spent(&db, TOKEN).await);
        assert_eq!(password_hash(&db, user).await, before);
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn pressing_the_button_with_a_new_password_sets_it(db: PgPool) {
        let state = state(db.clone(), false);
        let user = with_reset(&db, "renewed", TOKEN, "1 hour", false).await;

        let (status, _, html) =
            read(set_password(State(state.clone()), password(NEW_PASSWORD, NEW_PASSWORD)).await)
                .await;
        assert_eq!(status, StatusCode::OK, "{html}");
        assert!(html.contains("Password changed"));
        assert!(spent(&db, TOKEN).await);
        assert!(
            AuthService::verify_password(NEW_PASSWORD, &password_hash(&db, user).await)
                .expect("verify")
        );

        let (status, _, _) = read(reset_password_page(State(state), link(Some(TOKEN))).await).await;
        assert_eq!(status, StatusCode::GONE);
    }

    #[sqlx::test]
    async fn a_used_or_expired_reset_link_says_so_and_offers_a_new_one(db: PgPool) {
        let state = state(db.clone(), false);
        let used = with_reset(&db, "usedup", TOKEN, "1 hour", true).await;
        with_reset(&db, "expiredup", OTHER_TOKEN, "-1 hour", false).await;
        let before = password_hash(&db, used).await;

        for token in [TOKEN, OTHER_TOKEN, "nope"] {
            let (status, _, html) =
                read(reset_password_page(State(state.clone()), link(Some(token))).await).await;
            assert_eq!(status, StatusCode::GONE, "{token}");
            assert!(html.contains("This link no longer works"));
            assert!(html.contains("Reset links last 1 hour and work once."));
            assert!(html.contains(r#"action="/forgot-password""#));
        }

        let (status, _, _) =
            read(set_password(State(state), password(NEW_PASSWORD, NEW_PASSWORD)).await).await;
        assert_eq!(status, StatusCode::GONE);
        assert_eq!(password_hash(&db, used).await, before);
    }

    #[sqlx::test]
    async fn asking_for_a_new_link_on_a_page_answers_as_the_api_does(db: PgPool) {
        let configured = state(db.clone(), true);
        let identifier = || {
            Form(IdentifierForm {
                identifier: "nobody".into(),
            })
        };

        let (status, _, html) =
            read(request_verification_link(State(configured.clone()), identifier()).await).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("Check your email"));
        assert!(html.contains("If an account with an unverified email address matches"));

        let (status, _, html) =
            read(request_reset_link(State(configured), identifier()).await).await;
        assert_eq!(status, StatusCode::OK);
        assert!(html.contains("If an account with a verified email address matches"));

        let (status, _, html) =
            read(request_reset_link(State(state(db, false)), identifier()).await).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        assert!(html.contains("no link was sent"));
    }

    #[sqlx::test]
    async fn pages_run_no_script_and_cannot_be_framed_cached_or_indexed(db: PgPool) {
        let state = state(db.clone(), false);
        unverified(&db, "headers", TOKEN, "1 day").await;

        for response in [
            verify_email_page(State(state.clone()), link(Some(TOKEN))).await,
            verify_email_page(State(state.clone()), link(None)).await,
            reset_password_page(State(state), link(Some(TOKEN))).await,
        ] {
            let (_, headers, html) = read(response).await;
            let get = |name: &str| {
                headers
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .unwrap_or_default()
                    .to_string()
            };
            assert_eq!(get("content-type"), "text/html; charset=utf-8");
            assert_eq!(get("x-frame-options"), "DENY");
            assert_eq!(get("cache-control"), "no-store");
            assert_eq!(get("referrer-policy"), "no-referrer");
            assert!(get("content-security-policy").contains("default-src 'none'"));
            assert!(get("content-security-policy").contains("frame-ancestors 'none'"));
            assert!(get("x-robots-tag").contains("noindex"));
            assert!(!html.contains("<script"));
        }
    }
}
