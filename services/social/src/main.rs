//! Social: rooms, direct messages, presence and profiles per QOR ID (ADR-091; docs/blueprints/social.md).
//!
//! Configuration, from the environment:
//! - `DATABASE_URL`: Social's own Postgres database. Migrations run at start.
//! - `SOCIAL_QOR_ID_URL`: QOR ID's origin, `https://id.qorsync.dev` unless set.
//! - `SOCIAL_CLIENT_ID` (`social` unless set) and `SOCIAL_CLIENT_SECRET`: Social's own credential with QOR ID, registered
//!   there as a service (`"introspect": true`). Only its SHA-256 is registered with QOR ID.
//! - `SOCIAL_MESSAGE_KEYS`: the keys message bodies are sealed with, `1:<base64 of 32 bytes>,2:…` (`seal.rs`).
//! - `PORT`: 8090 unless set.
//!
//! None of these values is logged.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tracing_subscriber::layer::SubscriberExt as _;
use tracing_subscriber::util::SubscriberInitExt as _;
use tracing_subscriber::{EnvFilter, Layer as _};

mod api;
mod error;
mod hub;
mod identity;
mod seal;
mod socket;
mod state;
mod store;
mod text;

#[cfg(test)]
mod log_hygiene;
#[cfg(test)]
mod migration_hygiene;
#[cfg(test)]
mod sql_hygiene;
#[cfg(test)]
mod tests;

use state::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/socket", get(socket::upgrade))
        .with_state(state)
}

/// Whether a log record may be written at all. tungstenite prints every WebSocket frame with its payload at debug and
/// trace, which would put tokens and people's words in the log (AGENTS.md section 9), so it is held to info and above
/// whatever `RUST_LOG` asks. The log check (`log_hygiene.rs`) captures through this same filter.
pub fn may_log(meta: &tracing::Metadata<'_>) -> bool {
    let frames =
        meta.target().starts_with("tungstenite") || meta.target().starts_with("tokio_tungstenite");
    !frames || *meta.level() <= tracing::Level::INFO
}

fn required(name: &str) -> anyhow::Result<String> {
    std::env::var(name).map_err(|_| anyhow::anyhow!("{name} is not set"))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let wanted =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("social=info"));
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_filter(wanted)
                .with_filter(tracing_subscriber::filter::filter_fn(may_log)),
        )
        .init();

    let db = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(&required("DATABASE_URL")?)
        .await
        .map_err(|_| anyhow::anyhow!("Social's database could not be reached"))?;
    sqlx::migrate!("./migrations").run(&db).await?;

    let sealer = seal::Sealer::from_spec(&required("SOCIAL_MESSAGE_KEYS")?)?;
    let qor_id = identity::QorId::new(
        &std::env::var("SOCIAL_QOR_ID_URL").unwrap_or_else(|_| "https://id.qorsync.dev".into()),
        std::env::var("SOCIAL_CLIENT_ID").unwrap_or_else(|_| "social".into()),
        required("SOCIAL_CLIENT_SECRET")?,
    )?;
    let state = Arc::new(AppState {
        store: store::Store::new(db, sealer),
        identity: Arc::new(qor_id),
        hub: hub::Hub::default(),
    });

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8090);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
    tracing::info!(port, "Social is listening");
    axum::serve(listener, router(state)).await?;
    Ok(())
}
