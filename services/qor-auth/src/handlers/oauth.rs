//! Sign-in for other apps by redirect (ADR-043; how it is built, ADR-073): OAuth 2.1 authorization code
//! with PKCE.
//!
//! - GET  /oauth/authorize — QOR ID's own sign-in page, for a registered app and its exact redirect URI
//! - POST /oauth/authorize — the password checked as `/api/v1/auth/login` checks it; on success a
//!   single-use code goes back to the app
//! - POST /oauth/token     — the code exchanged for tokens (PKCE, and the app's secret if it has one),
//!   or a refresh token for new ones, rotated on every use
//! - GET  /oauth/userinfo  — who an access token belongs to, answered by QOR ID itself, so an app's
//!   server never needs QOR ID's signing secret
//! - POST /oauth/revoke    — sign out: the app's session ends
//!
//! The person types their password on QOR ID's origin only. The app never sees it, and never holds a
//! 30-day refresh token: its session lives `oauth.session_lifetime_secs`, and each refresh token works
//! once. Presenting one that was already used ends the whole session, because only a copy could have
//! been presented twice.
//!
//! Errors before the redirect URI is known to be the app's (an unknown app, an unregistered URI) are
//! shown on QOR ID's own page and never redirected: a code or an error sent to an address nobody
//! registered is the attack this guards. Login CSRF — a code for someone else's account delivered to
//! a person's browser — is stopped at the app, which must check `state` against the one it set in
//! that browser.

use axum::{
    Json,
    extract::{Form, Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::config::OAuthClient;
use crate::error::{AppError, AppResult};
use crate::handlers::auth::{ChainAccount, authenticate_password, record_sign_in};
use crate::handlers::pages::page_sending_to;
use crate::services::SessionService;
use crate::services::email_service::escape;
use crate::services::session_service::NewSession;
use crate::state::AppState;

fn random_hex() -> String {
    let bytes: [u8; 32] = rand::thread_rng().r#gen();
    hex::encode(bytes)
}

/// Equal without leaking where they differ.
fn same(a: &[u8], b: &[u8]) -> bool {
    a.len() == b.len() && a.iter().zip(b).fold(0u8, |acc, (x, y)| acc | (x ^ y)) == 0
}

/// RFC 7636: 43 to 128 characters of `[A-Za-z0-9-._~]`.
fn pkce_shaped(value: &str) -> bool {
    (43..=128).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~'))
}

/// `BASE64URL-ENCODE(SHA256(ASCII(code_verifier)))`, the S256 method.
fn s256(verifier: &str) -> String {
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn origin_of(uri: &str) -> Option<String> {
    let url = reqwest::Url::parse(uri).ok()?;
    let origin = url.origin();
    origin.is_tuple().then(|| origin.ascii_serialization())
}

async fn redis_set(state: &AppState, key: &str, ttl: u64, value: &str) -> AppResult<()> {
    let mut conn = state.redis.get().await?;
    deadpool_redis::redis::cmd("SET")
        .arg(key)
        .arg(value)
        .arg("EX")
        .arg(ttl)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))
}

async fn redis_get(state: &AppState, key: &str, take: bool) -> AppResult<Option<String>> {
    let mut conn = state.redis.get().await?;
    deadpool_redis::redis::cmd(if take { "GETDEL" } else { "GET" })
        .arg(key)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Redis error: {}", e)))
}

/// A sign-in page opened for an app, waiting for the person's password.
#[derive(Serialize, Deserialize)]
struct Pending {
    client_id: String,
    redirect_uri: String,
    state: String,
    code_challenge: String,
}

/// A code an app may exchange once, for the session it names.
#[derive(Serialize, Deserialize)]
struct Issued {
    session_id: Uuid,
    client_id: String,
    redirect_uri: String,
    code_challenge: String,
    role: String,
}

#[derive(Deserialize)]
pub struct AuthorizeQuery {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
    state: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizeForm {
    request: String,
    identifier: String,
    password: String,
}

fn no_store(mut response: Response) -> Response {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(header::PRAGMA, HeaderValue::from_static("no-cache"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

/// Back to the app, with `params` added to its redirect URI.
fn back_to(redirect_uri: &str, params: &[(&str, &str)]) -> Response {
    let Ok(mut url) = reqwest::Url::parse(redirect_uri) else {
        return refused_page(StatusCode::BAD_REQUEST, "This sign-in link is not valid.");
    };
    {
        let mut query = url.query_pairs_mut();
        for (k, v) in params {
            query.append_pair(k, v);
        }
    }
    let mut response = StatusCode::SEE_OTHER.into_response();
    if let Ok(location) = HeaderValue::from_str(url.as_str()) {
        response.headers_mut().insert(header::LOCATION, location);
    }
    no_store(response)
}

fn refused_page(status: StatusCode, message: &str) -> Response {
    page_sending_to(
        status,
        "Sign-in refused",
        &format!(
            "<p>{}</p><p class=\"muted\">Nothing was shared. Go back to the app and try again.</p>",
            escape(message)
        ),
        None,
    )
}

fn sign_in_page(
    status: StatusCode,
    client: &OAuthClient,
    redirect_uri: &str,
    request: &str,
    problem: Option<&str>,
) -> Response {
    let name = escape(&client.name);
    let origin = origin_of(redirect_uri).unwrap_or_default();
    let problem = problem
        .map(|p| format!("<p class=\"problem\" role=\"alert\">{}</p>", escape(p)))
        .unwrap_or_default();
    let body = format!(
        r#"<p>{name} asks who you are. You sign in here, on QOR ID: {name} never sees your password.</p>
<p class="muted">Afterwards you return to <span class="id">{origin_shown}</span>.</p>
{problem}<form method="post" action="/oauth/authorize">
<input type="hidden" name="request" value="{request}">
<label for="identifier">Username or email</label>
<input type="text" id="identifier" name="identifier" autocomplete="username" required>
<label for="password">Password</label>
<input type="password" id="password" name="password" autocomplete="current-password" required>
<button type="submit">Sign in</button>
</form>
<p class="muted after">Not expecting this? Close the page. Nothing is shared until you sign in.</p>"#,
        origin_shown = escape(&origin),
        request = escape(request),
    );
    page_sending_to(
        status,
        &format!("Sign in to {}", client.name),
        &body,
        Some(&origin),
    )
}

/// GET /oauth/authorize
pub async fn authorize_page(
    State(state): State<Arc<AppState>>,
    Query(q): Query<AuthorizeQuery>,
) -> Response {
    // Until the app and its redirect URI are known, nothing is sent anywhere.
    let client = match q
        .client_id
        .as_deref()
        .and_then(|id| state.config.oauth.client(id))
    {
        Some(client) => client.clone(),
        None => {
            return refused_page(
                StatusCode::BAD_REQUEST,
                "This sign-in link is not for an app QOR ID knows.",
            );
        }
    };
    let redirect_uri = match q.redirect_uri.as_deref() {
        Some(uri) if client.redirect_uris.iter().any(|r| r == uri) => uri.to_string(),
        _ => {
            return refused_page(
                StatusCode::BAD_REQUEST,
                "This sign-in link would return you to an address the app did not register.",
            );
        }
    };

    // From here an error goes back to the app, as OAuth expects.
    let app_state = q.state.clone().unwrap_or_default();
    let invalid = |description: &str| {
        let mut params = vec![
            ("error", "invalid_request"),
            ("error_description", description),
        ];
        if !app_state.is_empty() {
            params.push(("state", app_state.as_str()));
        }
        back_to(&redirect_uri, &params)
    };
    if q.response_type.as_deref() != Some("code") {
        return invalid("response_type must be code");
    }
    if q.code_challenge_method.as_deref() != Some("S256") {
        return invalid("code_challenge_method must be S256");
    }
    let challenge = match q.code_challenge.as_deref() {
        Some(c) if pkce_shaped(c) => c.to_string(),
        _ => return invalid("code_challenge is required"),
    };
    if app_state.is_empty() || app_state.len() > 512 {
        return invalid("state is required, at most 512 characters");
    }

    let request = random_hex();
    let pending = Pending {
        client_id: client.id.clone(),
        redirect_uri: redirect_uri.clone(),
        state: app_state,
        code_challenge: challenge,
    };
    let stored = serde_json::to_string(&pending).map_err(|e| AppError::InternalError(e.into()));
    match stored {
        Ok(json) => {
            if redis_set(
                &state,
                &format!("oauth_request:{request}"),
                state.config.oauth.request_ttl_secs,
                &json,
            )
            .await
            .is_err()
            {
                return refused_page(
                    StatusCode::SERVICE_UNAVAILABLE,
                    "QOR ID could not start the sign-in. Try again in a moment.",
                );
            }
        }
        Err(_) => {
            return refused_page(
                StatusCode::INTERNAL_SERVER_ERROR,
                "QOR ID could not start the sign-in.",
            );
        }
    }
    sign_in_page(StatusCode::OK, &client, &redirect_uri, &request, None)
}

/// POST /oauth/authorize
pub async fn authorize_submit(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AuthorizeForm>,
) -> Response {
    let expired = || {
        refused_page(
            StatusCode::GONE,
            "This sign-in page has expired. Go back to the app and start again.",
        )
    };
    if form.request.len() != 64 || !form.request.bytes().all(|b| b.is_ascii_hexdigit()) {
        return expired();
    }
    let key = format!("oauth_request:{}", form.request);
    let pending: Pending = match redis_get(&state, &key, false).await {
        Ok(Some(json)) => match serde_json::from_str(&json) {
            Ok(p) => p,
            Err(_) => return expired(),
        },
        Ok(None) => return expired(),
        Err(_) => {
            return refused_page(
                StatusCode::SERVICE_UNAVAILABLE,
                "QOR ID is unavailable. Try again in a moment.",
            );
        }
    };
    let Some(client) = state.config.oauth.client(&pending.client_id).cloned() else {
        return expired();
    };

    let user = match authenticate_password(&state, &form.identifier, &form.password).await {
        Ok(user) => user,
        Err(AppError::InvalidCredentials) => {
            return sign_in_page(
                StatusCode::UNAUTHORIZED,
                &client,
                &pending.redirect_uri,
                &form.request,
                Some("That name and password do not match an account that can sign in."),
            );
        }
        Err(_) => {
            return refused_page(
                StatusCode::SERVICE_UNAVAILABLE,
                "QOR ID is unavailable. Try again in a moment.",
            );
        }
    };

    // The page works once: taken now, so a second submission cannot mint a second code.
    match redis_get(&state, &key, true).await {
        Ok(Some(_)) => {}
        _ => return expired(),
    }

    let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let device = format!("oauth:{}", client.id);
    let session = match sessions
        .create_app_session(
            NewSession {
                user_id: user.id,
                qor_id: &user.qor_id(),
                role: Some(user.role.as_str()),
                device_id: &device,
                ip_address: "0.0.0.0",
                user_agent: None,
                scopes: crate::models::Session::default_scopes(),
            },
            &client.id,
            state.config.oauth.session_lifetime_secs,
        )
        .await
    {
        Ok(session) => session,
        Err(_) => {
            return refused_page(
                StatusCode::SERVICE_UNAVAILABLE,
                "QOR ID is unavailable. Try again in a moment.",
            );
        }
    };

    let code = random_hex();
    let issued = Issued {
        session_id: session.session_id,
        client_id: client.id.clone(),
        redirect_uri: pending.redirect_uri.clone(),
        code_challenge: pending.code_challenge.clone(),
        role: user.role.as_str().to_string(),
    };
    let Ok(json) = serde_json::to_string(&issued) else {
        return refused_page(
            StatusCode::INTERNAL_SERVER_ERROR,
            "QOR ID could not finish the sign-in.",
        );
    };
    if redis_set(
        &state,
        &format!("oauth_code:{code}"),
        state.config.oauth.code_ttl_secs,
        &json,
    )
    .await
    .is_err()
    {
        return refused_page(
            StatusCode::SERVICE_UNAVAILABLE,
            "QOR ID is unavailable. Try again in a moment.",
        );
    }
    if record_sign_in(&state.db, user.id, "oauth").await.is_err() {
        return refused_page(
            StatusCode::SERVICE_UNAVAILABLE,
            "QOR ID is unavailable. Try again in a moment.",
        );
    }
    back_to(
        &pending.redirect_uri,
        &[("code", &code), ("state", &pending.state)],
    )
}

#[derive(Deserialize)]
pub struct TokenForm {
    grant_type: Option<String>,
    client_id: Option<String>,
    client_secret: Option<String>,
    code: Option<String>,
    redirect_uri: Option<String>,
    code_verifier: Option<String>,
    refresh_token: Option<String>,
}

/// An OAuth error answer (RFC 6749 §5.2), never cached.
fn oauth_error(status: StatusCode, error: &str, description: &str) -> Response {
    no_store(
        (
            status,
            Json(json!({ "error": error, "error_description": description })),
        )
            .into_response(),
    )
}

fn invalid_grant() -> Response {
    oauth_error(
        StatusCode::BAD_REQUEST,
        "invalid_grant",
        "the code or refresh token is not valid",
    )
}

/// The app, if it proved itself: its id, and its secret when it has one.
fn authenticated_client(
    state: &AppState,
    id: Option<&str>,
    secret: Option<&str>,
) -> Option<OAuthClient> {
    let client = state.config.oauth.client(id?)?.clone();
    match (&client.secret_sha256, secret) {
        (Some(expected), Some(secret)) => {
            let got = hex::encode(Sha256::digest(secret.as_bytes()));
            same(got.as_bytes(), expected.to_ascii_lowercase().as_bytes()).then_some(client)
        }
        (None, None) => Some(client),
        _ => None,
    }
}

fn tokens_response(tokens: crate::models::TokenPair) -> Response {
    no_store(
        Json(json!({
            "access_token": tokens.access_token,
            "token_type": "Bearer",
            "expires_in": tokens.expires_in,
            "refresh_token": tokens.refresh_token,
        }))
        .into_response(),
    )
}

/// POST /oauth/token
pub async fn token(State(state): State<Arc<AppState>>, Form(form): Form<TokenForm>) -> Response {
    let Some(client) = authenticated_client(
        &state,
        form.client_id.as_deref(),
        form.client_secret.as_deref(),
    ) else {
        return oauth_error(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "the app could not be authenticated",
        );
    };
    let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());

    match form.grant_type.as_deref() {
        Some("authorization_code") => {
            let Some(code) = form
                .code
                .as_deref()
                .filter(|c| c.len() == 64 && c.bytes().all(|b| b.is_ascii_hexdigit()))
            else {
                return invalid_grant();
            };
            // Taken at once, so a code is used at most once whatever follows.
            let issued: Issued = match redis_get(&state, &format!("oauth_code:{code}"), true).await
            {
                Ok(Some(json)) => match serde_json::from_str(&json) {
                    Ok(issued) => issued,
                    Err(_) => return invalid_grant(),
                },
                Ok(None) => return invalid_grant(),
                Err(_) => {
                    return oauth_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "temporarily_unavailable",
                        "try again",
                    );
                }
            };
            let verifier_ok = form.code_verifier.as_deref().is_some_and(|v| {
                pkce_shaped(v) && same(s256(v).as_bytes(), issued.code_challenge.as_bytes())
            });
            if issued.client_id != client.id
                || form.redirect_uri.as_deref() != Some(issued.redirect_uri.as_str())
                || !verifier_ok
            {
                return invalid_grant();
            }
            let session = match sessions.get_session(issued.session_id).await {
                Ok(Some(s))
                    if !s.is_expired() && s.client_id.as_deref() == Some(client.id.as_str()) =>
                {
                    s
                }
                Ok(_) => return invalid_grant(),
                Err(_) => {
                    return oauth_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "temporarily_unavailable",
                        "try again",
                    );
                }
            };
            match sessions.generate_tokens(&session, Some(&issued.role)) {
                Ok(tokens) => tokens_response(tokens),
                Err(_) => oauth_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "tokens could not be issued",
                ),
            }
        }
        Some("refresh_token") => {
            let Some(presented) = form.refresh_token.as_deref() else {
                return invalid_grant();
            };
            let Ok(claims) = sessions.validate_refresh_token(presented) else {
                return invalid_grant();
            };
            if claims.cid.as_deref() != Some(client.id.as_str()) {
                return invalid_grant();
            }
            let Ok(session_id) = Uuid::parse_str(&claims.sid) else {
                return invalid_grant();
            };
            let session = match sessions.get_session(session_id).await {
                Ok(Some(s))
                    if !s.is_expired() && s.client_id.as_deref() == Some(client.id.as_str()) =>
                {
                    s
                }
                Ok(_) => return invalid_grant(),
                Err(_) => {
                    return oauth_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "temporarily_unavailable",
                        "try again",
                    );
                }
            };
            // Only the latest refresh token may be used. An older one means a copy exists: the session ends.
            if claims.jti.is_none() || claims.jti != session.refresh_jti {
                let _ = sessions
                    .delete_session(session.session_id, session.user_id)
                    .await;
                return invalid_grant();
            }
            let next = match sessions.rotate_refresh(&session).await {
                Ok(Some(next)) => next,
                Ok(None) => return invalid_grant(),
                Err(_) => {
                    return oauth_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "temporarily_unavailable",
                        "try again",
                    );
                }
            };
            match sessions.generate_tokens(&next, claims.role.as_deref()) {
                Ok(tokens) => tokens_response(tokens),
                Err(_) => oauth_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "server_error",
                    "tokens could not be issued",
                ),
            }
        }
        _ => oauth_error(
            StatusCode::BAD_REQUEST,
            "unsupported_grant_type",
            "grant_type must be authorization_code or refresh_token",
        ),
    }
}

/// GET /oauth/userinfo — who an access token belongs to. Checked here, against a live session, so an
/// app's server learns it without QOR ID's signing secret (ADR-069 decision 4).
pub async fn userinfo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> AppResult<Response> {
    let token = headers
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AppError::InvalidToken)?;
    let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let claims = sessions.authenticate_access_token(token).await?;
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;
    let row: Option<(String, Option<Vec<u8>>)> = sqlx::query_as(
        "SELECT username, chain_account_id FROM users WHERE id = $1 AND status = 'active'",
    )
    .bind(user_id)
    .fetch_optional(&state.db)
    .await?;
    let (username, chain) = row.ok_or(AppError::InvalidToken)?;
    let chain_account = chain
        .as_deref()
        .map(ChainAccount::from_stored)
        .transpose()?
        .map(|a| a.address(state.config.chain.ss58_prefix));
    Ok(no_store(
        Json(json!({
            "sub": claims.sub,
            "qor_id": claims.qor_id,
            "username": username,
            "chain_account": chain_account,
            "client_id": claims.cid,
        }))
        .into_response(),
    ))
}

#[derive(Deserialize)]
pub struct RevokeForm {
    client_id: Option<String>,
    client_secret: Option<String>,
    token: Option<String>,
}

/// POST /oauth/revoke — the app's session ends. Answers 200 whether or not the token was live
/// (RFC 7009), once the app has proved itself.
pub async fn revoke(State(state): State<Arc<AppState>>, Form(form): Form<RevokeForm>) -> Response {
    let Some(client) = authenticated_client(
        &state,
        form.client_id.as_deref(),
        form.client_secret.as_deref(),
    ) else {
        return oauth_error(
            StatusCode::UNAUTHORIZED,
            "invalid_client",
            "the app could not be authenticated",
        );
    };
    let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    if let Some(token) = form.token.as_deref() {
        let claims = sessions
            .validate_refresh_token(token)
            .or_else(|_| sessions.validate_access_token(token));
        if let Ok(claims) = claims
            && claims.cid.as_deref() == Some(client.id.as_str())
            && let (Ok(session_id), Ok(user_id)) =
                (Uuid::parse_str(&claims.sid), Uuid::parse_str(&claims.sub))
        {
            let _ = sessions.delete_session(session_id, user_id).await;
        }
    }
    no_store(StatusCode::OK.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use axum::{Router, body::Body, http::Request};
    use sqlx::PgPool;
    use tower::ServiceExt;

    const CALLBACK: &str = "https://arqade.test/api/auth/callback";
    const SECRET: &str = "arqade-test-client-secret-0123456789";
    const PASSWORD: &str = "a sign-in test password";
    const VERIFIER: &str = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk-and-some-more";

    fn app(db: PgPool) -> Router {
        let mut config = AppConfig::default();
        config.jwt.access_secret = "access-secret-for-the-oauth-tests-0123456789".into();
        config.jwt.refresh_secret = "refresh-secret-for-the-oauth-tests-0123456789".into();
        config.oauth.clients = vec![OAuthClient {
            id: "arqade".into(),
            name: "ARQADE".into(),
            redirect_uris: vec![CALLBACK.into()],
            secret_sha256: Some(hex::encode(Sha256::digest(SECRET.as_bytes()))),
        }];
        config.oauth.validate().expect("a valid registry");
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        let redis = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        crate::router(Arc::new(AppState::new(
            config,
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        )))
    }

    struct Reply {
        status: StatusCode,
        headers: HeaderMap,
        body: String,
    }

    impl Reply {
        fn json(&self) -> serde_json::Value {
            serde_json::from_str(&self.body).unwrap_or(serde_json::Value::Null)
        }
        fn location(&self) -> reqwest::Url {
            let raw = self.headers.get(header::LOCATION).expect("a redirect");
            reqwest::Url::parse(raw.to_str().expect("ascii")).expect("a URL")
        }
        fn param(&self, name: &str) -> Option<String> {
            self.location()
                .query_pairs()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.into_owned())
        }
    }

    async fn send(app: &Router, request: Request<Body>) -> Reply {
        let response = app.clone().oneshot(request).await.expect("response");
        let status = response.status();
        let headers = response.headers().clone();
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        Reply {
            status,
            headers,
            body: String::from_utf8_lossy(&bytes).into_owned(),
        }
    }

    fn encode(fields: &[(&str, &str)]) -> String {
        let mut url = reqwest::Url::parse("http://x/").expect("url");
        url.query_pairs_mut().extend_pairs(fields);
        url.query().unwrap_or_default().to_string()
    }

    async fn get(app: &Router, path_and_query: &str) -> Reply {
        send(
            app,
            Request::get(path_and_query)
                .body(Body::empty())
                .expect("request"),
        )
        .await
    }

    async fn form(app: &Router, path: &str, fields: &[(&str, &str)]) -> Reply {
        send(
            app,
            Request::post(path)
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from(encode(fields)))
                .expect("request"),
        )
        .await
    }

    fn authorize_url(client: &str, redirect: &str, challenge: &str, state: &str) -> String {
        format!(
            "/oauth/authorize?{}",
            encode(&[
                ("response_type", "code"),
                ("client_id", client),
                ("redirect_uri", redirect),
                ("code_challenge", challenge),
                ("code_challenge_method", "S256"),
                ("state", state),
            ])
        )
    }

    fn request_id(html: &str) -> String {
        html.split(r#"name="request" value=""#)
            .nth(1)
            .expect("a request field")
            .chars()
            .take(64)
            .collect()
    }

    async fn register(app: &Router, username: &str) {
        let r = send(
            app,
            Request::post("/api/v1/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "username": username, "password": PASSWORD }).to_string(),
                ))
                .expect("request"),
        )
        .await;
        assert_eq!(r.status, StatusCode::CREATED, "{}", r.body);
    }

    /// Open the page, sign in, and return the code the app receives.
    async fn sign_in(app: &Router, username: &str) -> String {
        let page = get(
            app,
            &authorize_url("arqade", CALLBACK, &s256(VERIFIER), "st-1"),
        )
        .await;
        assert_eq!(page.status, StatusCode::OK, "{}", page.body);
        let r = form(
            app,
            "/oauth/authorize",
            &[
                ("request", &request_id(&page.body)),
                ("identifier", username),
                ("password", PASSWORD),
            ],
        )
        .await;
        assert_eq!(r.status, StatusCode::SEE_OTHER, "{}", r.body);
        assert_eq!(r.param("state").as_deref(), Some("st-1"));
        r.param("code").expect("a code")
    }

    async fn exchange(app: &Router, code: &str, verifier: &str, secret: &str) -> Reply {
        form(
            app,
            "/oauth/token",
            &[
                ("grant_type", "authorization_code"),
                ("client_id", "arqade"),
                ("client_secret", secret),
                ("code", code),
                ("redirect_uri", CALLBACK),
                ("code_verifier", verifier),
            ],
        )
        .await
    }

    async fn refresh(app: &Router, token: &str) -> Reply {
        form(
            app,
            "/oauth/token",
            &[
                ("grant_type", "refresh_token"),
                ("client_id", "arqade"),
                ("client_secret", SECRET),
                ("refresh_token", token),
            ],
        )
        .await
    }

    async fn userinfo(app: &Router, access: &str) -> Reply {
        send(
            app,
            Request::get("/oauth/userinfo")
                .header("authorization", format!("Bearer {access}"))
                .body(Body::empty())
                .expect("request"),
        )
        .await
    }

    #[test]
    fn s256_matches_the_rfc_7636_example() {
        assert_eq!(
            s256("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_person_signs_in_on_qor_id_and_the_app_learns_who_they_are(db: PgPool) {
        let app = app(db);
        register(&app, "arqplayer").await;

        // The page belongs to QOR ID, names the app, and may send its form on to the app only.
        let page = get(
            &app,
            &authorize_url("arqade", CALLBACK, &s256(VERIFIER), "st-1"),
        )
        .await;
        assert_eq!(page.status, StatusCode::OK);
        assert!(page.body.contains("Sign in to ARQADE"));
        assert!(page.body.contains("https://arqade.test"));
        let csp = page
            .headers
            .get(header::CONTENT_SECURITY_POLICY)
            .expect("csp")
            .to_str()
            .expect("ascii")
            .to_string();
        assert!(
            csp.contains("form-action 'self' https://arqade.test;"),
            "{csp}"
        );
        assert!(csp.contains("frame-ancestors 'none'"));

        // A wrong password is answered on the page, and the page still works.
        let request = request_id(&page.body);
        let r = form(
            &app,
            "/oauth/authorize",
            &[
                ("request", &request),
                ("identifier", "arqplayer"),
                ("password", "not the password"),
            ],
        )
        .await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED);
        assert!(r.body.contains("do not match"));
        assert!(r.headers.get(header::LOCATION).is_none());

        let r = form(
            &app,
            "/oauth/authorize",
            &[
                ("request", &request),
                ("identifier", "arqplayer"),
                ("password", PASSWORD),
            ],
        )
        .await;
        assert_eq!(r.status, StatusCode::SEE_OTHER, "{}", r.body);
        assert!(r.location().as_str().starts_with(CALLBACK));
        let code = r.param("code").expect("a code");
        assert_eq!(r.param("state").as_deref(), Some("st-1"));

        // The page works once.
        let again = form(
            &app,
            "/oauth/authorize",
            &[
                ("request", &request),
                ("identifier", "arqplayer"),
                ("password", PASSWORD),
            ],
        )
        .await;
        assert_eq!(again.status, StatusCode::GONE);

        // The app's server exchanges the code, and asks QOR ID who the token belongs to.
        let tokens = exchange(&app, &code, VERIFIER, SECRET).await;
        assert_eq!(tokens.status, StatusCode::OK, "{}", tokens.body);
        assert_eq!(
            tokens
                .headers
                .get(header::CACHE_CONTROL)
                .and_then(|v| v.to_str().ok()),
            Some("no-store")
        );
        let access = tokens.json()["access_token"]
            .as_str()
            .expect("access")
            .to_string();
        let first_refresh = tokens.json()["refresh_token"]
            .as_str()
            .expect("refresh")
            .to_string();
        let who = userinfo(&app, &access).await;
        assert_eq!(who.status, StatusCode::OK, "{}", who.body);
        assert_eq!(who.json()["username"], "arqplayer");
        assert_eq!(who.json()["client_id"], "arqade");
        assert!(
            who.json()["chain_account"].is_null(),
            "no key proven, no chain identity (ADR-017)"
        );
        assert!(
            who.json()["sub"]
                .as_str()
                .is_some_and(|s| Uuid::parse_str(s).is_ok())
        );

        // The code works once.
        assert_eq!(
            exchange(&app, &code, VERIFIER, SECRET).await.json()["error"],
            "invalid_grant"
        );

        // An app's refresh token is refused on the launcher's refresh route, which does not rotate.
        let side = send(
            &app,
            Request::post("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({ "refresh_token": first_refresh }).to_string(),
                ))
                .expect("request"),
        )
        .await;
        assert_eq!(side.status, StatusCode::UNAUTHORIZED);

        // Refresh rotates: the new one works, and presenting the old one again ends the session.
        let second = refresh(&app, &first_refresh).await;
        assert_eq!(second.status, StatusCode::OK, "{}", second.body);
        let second_access = second.json()["access_token"]
            .as_str()
            .expect("access")
            .to_string();
        let second_refresh = second.json()["refresh_token"]
            .as_str()
            .expect("refresh")
            .to_string();
        assert_ne!(second_refresh, first_refresh);
        assert_eq!(userinfo(&app, &second_access).await.status, StatusCode::OK);
        assert_eq!(
            refresh(&app, &first_refresh).await.json()["error"],
            "invalid_grant"
        );
        assert_eq!(
            userinfo(&app, &second_access).await.status,
            StatusCode::UNAUTHORIZED,
            "reuse ended the session"
        );
        assert_eq!(
            refresh(&app, &second_refresh).await.json()["error"],
            "invalid_grant"
        );
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn sign_out_from_the_app_ends_its_session(db: PgPool) {
        let app = app(db);
        register(&app, "arqleaver").await;
        let code = sign_in(&app, "arqleaver").await;
        let tokens = exchange(&app, &code, VERIFIER, SECRET).await;
        let access = tokens.json()["access_token"]
            .as_str()
            .expect("access")
            .to_string();
        let refresh_token = tokens.json()["refresh_token"]
            .as_str()
            .expect("refresh")
            .to_string();
        let wrong = form(
            &app,
            "/oauth/revoke",
            &[
                ("client_id", "arqade"),
                ("client_secret", "wrong"),
                ("token", &refresh_token),
            ],
        )
        .await;
        assert_eq!(wrong.status, StatusCode::UNAUTHORIZED);
        assert_eq!(userinfo(&app, &access).await.status, StatusCode::OK);
        let r = form(
            &app,
            "/oauth/revoke",
            &[
                ("client_id", "arqade"),
                ("client_secret", SECRET),
                ("token", &refresh_token),
            ],
        )
        .await;
        assert_eq!(r.status, StatusCode::OK);
        assert_eq!(
            userinfo(&app, &access).await.status,
            StatusCode::UNAUTHORIZED
        );
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn nothing_is_sent_to_an_app_or_an_address_qor_id_does_not_know(db: PgPool) {
        let app = app(db);
        for url in [
            authorize_url("stranger", CALLBACK, &s256(VERIFIER), "s"),
            authorize_url("arqade", "https://evil.test/callback", &s256(VERIFIER), "s"),
            authorize_url(
                "arqade",
                "https://arqade.test/api/auth/callback/../../steal",
                &s256(VERIFIER),
                "s",
            ),
            "/oauth/authorize?response_type=code&client_id=arqade".to_string(),
        ] {
            let r = get(&app, &url).await;
            assert_eq!(r.status, StatusCode::BAD_REQUEST, "{url}");
            assert!(
                r.headers.get(header::LOCATION).is_none(),
                "{url} must not redirect"
            );
        }
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_malformed_request_goes_back_to_the_app_as_an_error_with_its_state(db: PgPool) {
        let app = app(db);
        let challenge = s256(VERIFIER);
        let plain = format!(
            "/oauth/authorize?{}",
            encode(&[
                ("response_type", "code"),
                ("client_id", "arqade"),
                ("redirect_uri", CALLBACK),
                ("code_challenge", &challenge),
                ("code_challenge_method", "plain"),
                ("state", "s9"),
            ])
        );
        let r = get(&app, &plain).await;
        assert_eq!(r.status, StatusCode::SEE_OTHER);
        assert_eq!(r.param("error").as_deref(), Some("invalid_request"));
        assert_eq!(r.param("state").as_deref(), Some("s9"));
        assert!(r.param("code").is_none());
        let no_state = get(&app, &authorize_url("arqade", CALLBACK, &challenge, "")).await;
        assert_eq!(no_state.param("error").as_deref(), Some("invalid_request"));
        let short = get(&app, &authorize_url("arqade", CALLBACK, "too-short", "s")).await;
        assert_eq!(short.param("error").as_deref(), Some("invalid_request"));
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn a_code_needs_its_verifier_its_app_and_its_redirect_and_dies_on_first_use(db: PgPool) {
        let app = app(db);
        register(&app, "arqcareful").await;

        let code = sign_in(&app, "arqcareful").await;
        let other = "a-different-verifier-that-is-long-enough-to-pass-the-shape";
        assert_eq!(
            exchange(&app, &code, other, SECRET).await.json()["error"],
            "invalid_grant"
        );
        // Spent by the failed attempt: the right verifier cannot rescue it.
        assert_eq!(
            exchange(&app, &code, VERIFIER, SECRET).await.json()["error"],
            "invalid_grant"
        );

        let code = sign_in(&app, "arqcareful").await;
        let r = exchange(&app, &code, VERIFIER, "not the secret").await;
        assert_eq!(r.status, StatusCode::UNAUTHORIZED);
        assert_eq!(r.json()["error"], "invalid_client");

        let code = sign_in(&app, "arqcareful").await;
        let r = form(
            &app,
            "/oauth/token",
            &[
                ("grant_type", "authorization_code"),
                ("client_id", "arqade"),
                ("client_secret", SECRET),
                ("code", &code),
                ("redirect_uri", "https://arqade.test/other"),
                ("code_verifier", VERIFIER),
            ],
        )
        .await;
        assert_eq!(r.json()["error"], "invalid_grant");

        let r = form(
            &app,
            "/oauth/token",
            &[
                ("grant_type", "password"),
                ("client_id", "arqade"),
                ("client_secret", SECRET),
            ],
        )
        .await;
        assert_eq!(r.json()["error"], "unsupported_grant_type");

        // An expired or invented page is refused, not signed into.
        let invented = "ab".repeat(32);
        let r = form(
            &app,
            "/oauth/authorize",
            &[
                ("request", &invented),
                ("identifier", "arqcareful"),
                ("password", PASSWORD),
            ],
        )
        .await;
        assert_eq!(r.status, StatusCode::GONE);
    }

    #[test]
    fn the_registry_refuses_what_would_send_codes_astray() {
        let mut config = crate::config::OAuthConfig::default();
        let client = |uri: &str| OAuthClient {
            id: "app".into(),
            name: "App".into(),
            redirect_uris: vec![uri.into()],
            secret_sha256: None,
        };
        for bad in [
            "http://example.com/cb",
            "ftp://example.com/cb",
            "https://example.com/cb#frag",
            "not a url",
        ] {
            config.clients = vec![client(bad)];
            assert!(config.validate().is_err(), "{bad}");
        }
        for good in [
            "https://example.com/cb",
            "http://127.0.0.1:8787/api/auth/callback",
            "http://localhost:3000/cb",
        ] {
            config.clients = vec![client(good)];
            assert!(config.validate().is_ok(), "{good}");
        }
        config.clients = vec![client("https://a.test/cb"), client("https://b.test/cb")];
        assert!(config.validate().is_err(), "an id registered twice");
        config.clients = vec![OAuthClient {
            secret_sha256: Some("abc".into()),
            ..client("https://a.test/cb")
        }];
        assert!(
            config.validate().is_err(),
            "a secret hash that is not 64 hex characters"
        );
    }
}
