//! Authentication middleware.

use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use std::sync::Arc;

use crate::error::AppError;
use crate::models::{Claims, UserRole};
use crate::services::SessionService;
use crate::state::AppState;

#[cfg(test)]
mod tests {
    use super::*;

    fn claims_with(role: Option<&str>) -> Claims {
        Claims {
            sub: uuid::Uuid::new_v4().to_string(),
            qor_id: "tester#0001".into(),
            sid: uuid::Uuid::new_v4().to_string(),
            role: role.map(str::to_string),
            scopes: vec![],
            iss: "test".into(),
            iat: 0,
            exp: 0,
            jti: None,
            cid: None,
        }
    }

    #[test]
    fn a_god_token_as_sign_in_writes_it_is_admitted() {
        assert!(is_god(&claims_with(Some(UserRole::God.as_str()))));
    }

    #[test]
    fn other_roles_the_old_debug_form_and_no_role_are_refused() {
        for role in [
            Some(UserRole::Admin.as_str()),
            Some(UserRole::User.as_str()),
            Some("God"),
            None,
        ] {
            assert!(!is_god(&claims_with(role)), "{role:?} must not be admitted");
        }
    }
}

/// Helper to extract state from request (for use in nested routes)
fn get_state_from_request(request: &Request) -> Option<Arc<AppState>> {
    request.extensions().get::<Arc<AppState>>().cloned()
}

/// A refused token is 401. A session lookup that failed is 500, so an outage is not reported as a
/// signed-out user; the request is refused either way.
fn rejection(error: AppError) -> StatusCode {
    match error {
        AppError::InvalidToken | AppError::TokenExpired => StatusCode::UNAUTHORIZED,
        other => {
            tracing::error!("Session lookup failed: {:?}", other);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

/// The session that made the request, for handlers that need to tell it apart from the user's
/// other sessions.
#[derive(Clone, Copy, Debug)]
pub struct CurrentSession(pub uuid::Uuid);

/// Middleware to require authentication (state comes from the parent router)
pub async fn require_auth(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get state from extensions (set by parent router)
    let state = get_state_from_request(&request).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate token and require its session to still exist
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());

    let claims = session_service
        .authenticate_access_token(token)
        .await
        .map_err(rejection)?;

    // Attach user info to request extensions
    if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
        request.extensions_mut().insert(user_id);
    }
    if let Ok(session_id) = uuid::Uuid::parse_str(&claims.sid) {
        request.extensions_mut().insert(CurrentSession(session_id));
    }
    if let Some(role) = &claims.role {
        request.extensions_mut().insert(role.clone());
    }
    request.extensions_mut().insert(claims.qor_id.clone());

    Ok(next.run(request).await)
}

/// Whether a token grants God-level access. Sign-in writes the role with `UserRole::as_str`, and
/// this compares with the same function, so the two cannot drift apart again.
fn is_god(claims: &Claims) -> bool {
    claims.role.as_deref() == Some(UserRole::God.as_str())
}

/// Middleware to require God-level access
pub async fn require_god(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    // Get state from extensions
    let state = get_state_from_request(&request).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;
    // First check authentication
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    let token = match auth_header {
        Some(token) => token,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate token and require its session to still exist
    let session_service = SessionService::new(state.redis.clone(), state.config.jwt.clone());

    let claims = session_service
        .authenticate_access_token(token)
        .await
        .map_err(rejection)?;

    if !is_god(&claims) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Attach user info to request extensions for handlers
    if let Ok(user_id) = uuid::Uuid::parse_str(&claims.sub) {
        request.extensions_mut().insert(user_id);
    }
    if let Ok(session_id) = uuid::Uuid::parse_str(&claims.sid) {
        request.extensions_mut().insert(CurrentSession(session_id));
    }
    request.extensions_mut().insert(claims.qor_id.clone());

    Ok(next.run(request).await)
}
