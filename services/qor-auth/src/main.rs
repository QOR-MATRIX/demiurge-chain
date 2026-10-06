//! # Qor Auth Service
//!
//! The Non-Dual Identity System for the Demiurge Ecosystem.
//!
//! ## Architecture
//!
//! - **Framework**: Axum (Rust 2024)
//! - **Database**: PostgreSQL 18
//! - **Cache**: Redis 7.4+
//! - **Auth**: JWT + Refresh Tokens
//!
//! ## Features
//!
//! - Battle.Net-style `username#discriminator` identity
//! - ZK-proof verification for privacy-preserving attestations
//! - On-chain identity linking via Substrate
//! - Session management with device tracking

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::HeaderValue,
    middleware::{Next, from_fn, from_fn_with_state},
    routing::{get, post},
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;
mod state;

#[cfg(test)]
mod log_hygiene;
#[cfg(test)]
mod migration_hygiene;
#[cfg(test)]
mod sql_hygiene;

use crate::config::AppConfig;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qor_auth=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("🎭 Qor Auth Service - Genesis Activation");

    // Load configuration
    let config = AppConfig::load()?;

    // Database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    tracing::info!("✅ Connected to PostgreSQL");

    // Run migrations
    sqlx::migrate!("./migrations").run(&db_pool).await?;

    tracing::info!("✅ Database migrations applied");

    // How many accounts migration 019 renamed to make every name unique (ADR-075). A count, never a
    // name: it says whether anyone needs telling.
    let renamed: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE renamed_from IS NOT NULL")
            .fetch_one(&db_pool)
            .await?;
    tracing::info!(
        renamed,
        "accounts renamed for one name per account (ADR-075)"
    );

    // Redis connection
    let redis_cfg = deadpool_redis::Config::from_url(&config.redis.url);
    let redis_pool = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;

    tracing::info!("✅ Connected to Redis");

    // Initialize email service
    let email_config = services::EmailConfig::from_env();
    let email_service = services::EmailService::new(email_config)
        .with_webhook_secret(std::env::var("RESEND_WEBHOOK_SECRET").ok());
    if email_service.webhook_secret().is_none() {
        tracing::warn!(
            "RESEND_WEBHOOK_SECRET is not set: bounce and complaint reports from Resend are refused"
        );
    }

    if email_service.is_configured() {
        tracing::info!("✅ Email service configured");
    } else {
        tracing::warn!(
            "Email service not configured: password reset by email is refused, and no verification email is sent"
        );
    }

    // Build application state
    let state = Arc::new(AppState::new(
        config.clone(),
        db_pool,
        redis_pool,
        email_service,
    ));

    let app = router(state);

    // Bind to the configured host. This previously ignored `server.host` and
    // always listened on every interface.
    let ip: std::net::IpAddr = config.server.host.parse().map_err(|_| {
        anyhow::anyhow!(
            "server.host must be an IP address, got {:?}",
            config.server.host
        )
    })?;
    let addr = SocketAddr::new(ip, config.server.port);
    tracing::info!("Environment: {}", config.server.environment);
    tracing::info!("🚀 Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

/// The whole service: every route and layer. Built here, rather than in `main`, so the log check
/// (`log_hygiene`) drives exactly what is served.
pub(crate) fn router(state: Arc<AppState>) -> Router {
    Router::new()
        // The pages the links in emails open. Opening one changes nothing; its button does.
        .route(
            "/verify-email",
            get(handlers::pages::verify_email_page).post(handlers::pages::confirm_email),
        )
        .route(
            "/reset-password",
            get(handlers::pages::reset_password_page).post(handlers::pages::set_password),
        )
        .route(
            "/resend-verification",
            post(handlers::pages::request_verification_link),
        )
        .route(
            "/forgot-password",
            post(handlers::pages::request_reset_link),
        )
        // A person's own account page (handlers/account.rs): every form asks for the current password.
        .route("/account", get(handlers::account::account))
        .route(
            "/account/password",
            post(handlers::account::change_password_page),
        )
        .route("/account/email", post(handlers::account::change_email_page))
        // An app's server reports a task it checked (ADR-078).
        .route("/oauth/progress", post(handlers::progress::app_progress))
        // Sign-in for other apps by redirect (ADR-043, ADR-073): QOR ID's own page, then a code, then
        // tokens for the app's server. Never `*` in CORS: these are page and server-to-server calls.
        .route(
            "/oauth/authorize",
            get(handlers::oauth::authorize_page).post(handlers::oauth::authorize_submit),
        )
        .route("/oauth/token", post(handlers::oauth::token))
        .route("/oauth/userinfo", get(handlers::oauth::userinfo))
        .route("/oauth/revoke", post(handlers::oauth::revoke))
        // Bounce and complaint reports from Resend, accepted only with a valid signature.
        .route("/api/v1/webhooks/resend", post(handlers::webhooks::resend))
        // Health endpoints
        .route("/health", get(handlers::health::health_check))
        .route("/ready", get(handlers::health::readiness_check))
        // Public auth endpoints
        .nest("/api/v1/auth", auth_routes())
        // Protected profile endpoints
        .nest("/api/v1/profile", profile_routes())
        // ZK verification endpoints
        .nest("/api/v1/zk", zk_routes())
        // Admin endpoints (protected - God-level)
        .nest("/api/v1/admin", admin_routes())
        // Agent management endpoints
        .nest("/api/v1/agents", agent_routes())
        // Music player endpoints
        // Music routes are disabled: see the note in src/handlers/mod.rs.
        // .nest("/api/v1/music", music_routes())
        // Middleware - inject state into extensions for nested routes
        .layer(from_fn_with_state(
            state.clone(),
            move |State(state): State<Arc<AppState>>, mut request: Request, next: Next| async move {
                request.extensions_mut().insert(state);
                next.run(request).await
            },
        ))
        // A request's span records its path, never its query: the links in emails carry their
        // tokens there, and a token must not reach a log.
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request| {
                tracing::debug_span!(
                    "request",
                    method = %request.method(),
                    path = %request.uri().path()
                )
            }),
        )
        .layer(CompressionLayer::new())
        .layer(cors_layer(&state.config))
        .with_state(state)
}

/// The cross-origin policy, built from configuration and never from `Any`.
///
/// `Any` was what this service used until 21 September 2026: every origin,
/// every method, every header, on every route including `/api/v1/admin/*`.
/// That let any page a person had open script this API in their browser and
/// read the answer.
///
/// An unknown origin now gets no `Access-Control-Allow-Origin` header at all,
/// so the browser refuses the response. Nothing changes for the launcher or
/// for agents: they are not browsers, they send no `Origin`, and CORS does not
/// apply to them. An empty allowlist is therefore a perfectly good
/// configuration for a deployment that serves no browser frontend.
///
/// Methods and headers stay broad on purpose. The origin is the security
/// boundary; narrowing the method list would only break a preflight for a
/// route someone adds later, without denying anything an allowed origin could
/// not already do.
fn cors_layer(config: &config::AppConfig) -> CorsLayer {
    let origins: Vec<HeaderValue> = config
        .server
        .allowed_origins
        .iter()
        .filter_map(|o| o.parse::<HeaderValue>().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(Any)
        .allow_headers(Any)
}

/// Authentication routes (public)
fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        // Traditional password auth
        // The owner's limit of three accounts per network address wraps both sign-ups (ADR-078).
        .route("/register", post(handlers::progress::register_limited))
        .route("/login", post(handlers::auth::login))
        .route("/refresh", post(handlers::auth::refresh_token))
        .route("/logout", post(handlers::auth::logout))
        .route("/verify-email", post(handlers::auth::verify_email))
        .route(
            "/resend-verification",
            post(handlers::auth::resend_verification),
        )
        .route("/forgot-password", post(handlers::auth::forgot_password))
        .route("/reset-password", post(handlers::auth::reset_password))
        .route(
            "/reset-password-backup",
            post(handlers::auth::reset_password_with_backup),
        )
        .route("/check-username", post(handlers::auth::check_username))
        .route("/check-email", post(handlers::auth::check_email))
        // Keypair-based auth (Nostr-style)
        .route("/challenge", get(handlers::auth::get_challenge))
        .route("/keypair-login", post(handlers::auth::keypair_login))
        .route(
            "/keypair-register",
            post(handlers::progress::keypair_register_limited),
        )
        // Binds a key to the account in the access token, so it requires one.
        .route(
            "/link-keypair",
            post(handlers::auth::link_keypair)
                .layer(from_fn(crate::middleware::auth::require_auth)),
        )
}

/// Profile routes (protected)
fn profile_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(handlers::profile::get_profile))
        .route("/", post(handlers::profile::update_profile))
        .route("/avatar", post(handlers::profile::upload_avatar))
        .route("/sessions", get(handlers::profile::list_sessions))
        .route(
            "/sessions/{id}",
            axum::routing::delete(handlers::profile::revoke_session),
        )
        .route("/link-wallet", post(handlers::profile::link_wallet))
        .route(
            "/backup-codes",
            post(handlers::profile::regenerate_backup_codes),
        )
        .route("/email", post(handlers::profile::request_email_change))
        .route("/password", post(handlers::account::change_password_api))
        .route("/progress", get(handlers::progress::get_progress))
        .route(
            "/progress/tutorial",
            post(handlers::progress::tutorial_done),
        )
        .layer(from_fn(crate::middleware::auth::require_auth))
}

/// ZK-proof verification routes
fn zk_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/verify", post(handlers::zk::verify_proof))
        .route("/attestations", get(handlers::zk::get_attestations))
        .route("/attestations", post(handlers::zk::create_attestation))
}

/// Admin routes (protected - God-level access)
fn admin_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/users", get(handlers::admin::list_users))
        .route("/users/{id}", get(handlers::admin::get_user))
        .route("/users/{id}/ban", post(handlers::admin::ban_user))
        .route("/users/{id}/unban", post(handlers::admin::unban_user))
        .route("/users/{id}/role", post(handlers::admin::update_role))
        .route("/tokens/transfer", post(handlers::admin::transfer_tokens))
        .route("/tokens/refund", post(handlers::admin::refund_tokens))
        .route("/stats", get(handlers::admin::get_stats))
        .route("/audit", get(handlers::admin::get_audit_log))
        // Admin-only by design: an account holder who could clear its own address's mark would make
        // the mark mean nothing.
        .route(
            "/email-suppressions/unmark",
            post(handlers::admin::unmark_undeliverable_address),
        )
        .layer(from_fn(crate::middleware::auth::require_god))
}

/// Agent management routes (protected)
fn agent_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/register", post(handlers::agents::register_agent))
        .route("/", get(handlers::agents::list_agents))
        .route("/{did}", get(handlers::agents::get_agent))
        .route(
            "/{did}/capabilities",
            axum::routing::put(handlers::agents::update_capabilities),
        )
        .route(
            "/{did}",
            axum::routing::delete(handlers::agents::deactivate_agent),
        )
        .layer(from_fn(crate::middleware::auth::require_auth))
}

// Disabled along with the music handlers; see src/handlers/mod.rs.
// /// Music player routes (mixed public/protected)
// fn music_routes() -> Router<Arc<AppState>> {
//     Router::new()
//         // Public endpoints
//         .route("/tracks", get(handlers::music::list_tracks))
//         .route("/tracks/{id}", get(handlers::music::get_track))
//         .route("/tracks/{id}/play", post(handlers::music::record_play))
//         .route("/playlists", get(handlers::music::list_playlists))
//         .route("/playlists/global", get(handlers::music::get_global_playlist))
//         .route("/playlists/{id}/tracks", get(handlers::music::get_playlist_tracks))
//         // Protected endpoints (require auth)
//         .route("/tracks/upload", post(handlers::music::upload_track))
//         .route("/tracks/{id}/like", post(handlers::music::like_track))
//         .route("/playlists/create", post(handlers::music::create_playlist))
//         .route("/playlists/{id}/add", post(handlers::music::add_to_playlist))
//         .route("/playlists/{playlist_id}/remove/{track_id}", delete(handlers::music::remove_from_playlist))
// }

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("🛑 Shutdown signal received, starting graceful shutdown");
}

#[cfg(test)]
mod cors_tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::ServiceExt;

    /// A router carrying only the real CORS layer, so the policy is tested and
    /// nothing else is.
    fn router_with(origins: Vec<&str>) -> Router {
        let mut config = AppConfig::default();
        config.server.allowed_origins = origins.into_iter().map(String::from).collect();
        Router::new()
            .route("/probe", axum::routing::get(|| async { "ok" }))
            .layer(cors_layer(&config))
    }

    async fn allow_origin_header(origins: Vec<&str>, asking: &str) -> Option<String> {
        let response = router_with(origins)
            .oneshot(
                Request::builder()
                    .uri("/probe")
                    .header(header::ORIGIN, asking)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        response
            .headers()
            .get(header::ACCESS_CONTROL_ALLOW_ORIGIN)
            .map(|v| v.to_str().unwrap().to_string())
    }

    /// The whole point. Until 21 September 2026 this service answered
    /// `Access-Control-Allow-Origin: *` to every origin on every route,
    /// including the admin routes. An origin nobody configured must now get no
    /// such header at all, so the browser refuses the response.
    #[tokio::test]
    async fn an_origin_that_is_not_configured_is_not_allowed() {
        let allowed = allow_origin_header(
            vec!["https://id.demiurge.cloud"],
            "https://id.demiurge.cloud",
        )
        .await;
        assert_eq!(
            allowed.as_deref(),
            Some("https://id.demiurge.cloud"),
            "a configured origin must be allowed, or the allowlist is useless"
        );

        let refused =
            allow_origin_header(vec!["https://id.demiurge.cloud"], "https://evil.example").await;
        assert_eq!(
            refused, None,
            "an unconfigured origin must get no Access-Control-Allow-Origin header"
        );
    }

    /// A wildcard must never come back, whatever is configured. This is the
    /// regression that would silently undo the change above.
    #[tokio::test]
    async fn the_policy_never_answers_with_a_wildcard() {
        for asking in ["https://evil.example", "http://localhost:1420", "null"] {
            let header = allow_origin_header(
                vec!["http://localhost:1420", "https://console.demiurge.cloud"],
                asking,
            )
            .await;
            assert_ne!(
                header.as_deref(),
                Some("*"),
                "the wildcard is what was wrong; it must not return for {asking}"
            );
        }
    }

    /// A deployment that serves no browser frontend configures no origins, and
    /// that must mean "no cross-origin browser request", not "all of them".
    #[tokio::test]
    async fn an_empty_allowlist_allows_nothing() {
        assert_eq!(
            allow_origin_header(vec![], "https://anything.example").await,
            None
        );
    }

    /// The defaults a developer gets are loopback only. A real hostname
    /// arriving here would mean a deployment origin had been compiled in.
    #[test]
    fn the_default_origins_are_loopback_only() {
        for origin in AppConfig::default().server.allowed_origins {
            assert!(
                origin.starts_with("http://localhost:") || origin.starts_with("http://127.0.0.1:"),
                "default origins must be loopback, found {origin}"
            );
        }
    }
}
