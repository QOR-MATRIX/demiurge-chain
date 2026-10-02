//! Health check endpoints.

use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;

use crate::state::AppState;

/// Basic health check
///
/// `email_leaves_this_machine` is true only when this service sends mail to real inboxes: email is
/// configured, and not for a stand-in on this machine. The end-to-end scripts read it and refuse to
/// run when it is true, because they register addresses nobody holds, and that mail bounces. It is
/// the one fact about configuration given here. Whether email is configured at all is already
/// public, since `forgot-password` answers 503 without it; a deployment has no stand-in, so there
/// this says nothing more than that.
pub async fn health_check(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "service": "qor-auth",
            "version": env!("CARGO_PKG_VERSION"),
            "email_leaves_this_machine": state.email_service.leaves_this_machine()
        })),
    )
}

/// Readiness check (verifies database and cache connectivity)
pub async fn readiness_check(State(state): State<Arc<AppState>>) -> (StatusCode, Json<Value>) {
    // Check database
    let db_ok = sqlx::query("SELECT 1").fetch_one(&state.db).await.is_ok();

    // Check Redis
    let redis_ok = state.redis.get().await.is_ok();

    let status = if db_ok && redis_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (
        status,
        Json(json!({
            "status": if db_ok && redis_ok { "ready" } else { "not_ready" },
            "checks": {
                "database": if db_ok { "ok" } else { "failed" },
                "cache": if redis_ok { "ok" } else { "failed" }
            }
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use axum::body::Body;
    use axum::http::Request;
    use sqlx::PgPool;
    use tower::ServiceExt;

    /// `/health` through the real router, for a service with this email configuration.
    async fn health(db: PgPool, email: EmailConfig) -> Value {
        // Nothing here reaches Redis, so the pool is never connected.
        let redis = deadpool_redis::Config::from_url("redis://127.0.0.1:1")
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        let state = Arc::new(AppState::new(
            AppConfig::default(),
            db,
            redis,
            EmailService::new(email),
        ));
        let response = crate::router(state)
            .oneshot(
                Request::get("/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body");
        serde_json::from_slice(&bytes).expect("json")
    }

    /// The end-to-end scripts refuse to run unless this field is exactly `false`, so it has to be
    /// there, and a boolean. The true case cannot be built here: a test build refuses to send
    /// anywhere but this machine. `email_service` checks that rule on its own.
    #[sqlx::test]
    async fn health_says_whether_mail_leaves_this_machine_and_nothing_else_about_email(db: PgPool) {
        let stand_in = EmailConfig {
            resend_api_key: "re_health_check_key".into(),
            from: "Health Check <noreply@sender.healthcheck.invalid>".into(),
            base_url: "https://links.healthcheck.invalid".into(),
            api_url: "http://127.0.0.1:9".into(),
        };
        for email in [EmailConfig::unconfigured(), stand_in] {
            let body = health(db.clone(), email).await;
            assert_eq!(body["email_leaves_this_machine"], json!(false), "{body}");

            // One boolean, and none of the configuration it was worked out from.
            let mut keys: Vec<&str> = body
                .as_object()
                .expect("an object")
                .keys()
                .map(String::as_str)
                .collect();
            keys.sort_unstable();
            assert_eq!(
                keys,
                ["email_leaves_this_machine", "service", "status", "version"]
            );
            let text = body.to_string();
            for configured in ["re_health_check_key", "healthcheck.invalid", "127.0.0.1"] {
                assert!(
                    !text.contains(configured),
                    "/health gives away {configured}"
                );
            }
        }
    }
}
