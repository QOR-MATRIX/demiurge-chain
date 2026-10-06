//! Levels, tasks and the welcome grant (ADR-078).
//!
//! One level per QOR ID, shared by every app. XP is granted only here, by QOR ID itself for what it sees (an email
//! verified, a key linked, a sign-in to ARQADE), or by a registered app's server for what it checks (a first match, a
//! first payment), and each task once: `progress_events` has one row per account and task. The launcher reports a finished
//! tutorial, which is the one task an account reports for itself; it grants XP once and never CGT on its own.
//!
//! The welcome grant (100 CGT, the owner's amount) is recorded as owed when the tutorial, a verified email address and a
//! linked key are all present, once per account, per email address and per chain account (migration 020's unique keys).
//! QOR ID moves no CGT; what pays an owed grant is a separate service.
//!
//! Also here: the owner's limit of three accounts per network address, kept as a keyed hash, never the address.
//!
//! - GET  /api/v1/profile/progress    the signed-in account's level, XP, tasks and next unlock
//! - POST /api/v1/profile/progress/tutorial the launcher: the tutorial is finished
//! - POST /oauth/progress             an app's server: a task it checked, for the person whose token it holds

use axum::{
    Extension, Form, Json,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::Sha256;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::session_service::SessionService;
use crate::state::AppState;

/// Who may report a task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// The account itself (the launcher's tutorial). Once, XP only.
    Account,
    /// QOR ID, from what it sees.
    QorId,
    /// The server of the registered app with this id, with the person's token.
    App(&'static str),
}

pub struct Task {
    pub key: &'static str,
    pub name: &'static str,
    pub xp: i32,
    pub source: Source,
}

/// ADR-078 decision 3. The XP values are game design, the owner's to change.
pub const TASKS: &[Task] = &[
    Task {
        key: "tutorial",
        name: "Finish the launcher's tutorial",
        xp: 50,
        source: Source::Account,
    },
    Task {
        key: "verify-email",
        name: "Verify your email address",
        xp: 25,
        source: Source::QorId,
    },
    Task {
        key: "link-key",
        name: "Link a key to your QOR ID",
        xp: 25,
        source: Source::QorId,
    },
    Task {
        key: "sign-in-arqade",
        name: "Sign in to ARQADE",
        xp: 10,
        source: Source::QorId,
    },
    Task {
        key: "first-match",
        name: "Finish a multiplayer match on ARQADE",
        xp: 25,
        source: Source::App("arqade"),
    },
    Task {
        key: "first-payment",
        name: "Make a first tip or payment",
        xp: 25,
        source: Source::App("arqade"),
    },
];

/// What each level unlocks: cosmetic only (ADR-078 decision 4). Ring styles are colour and pattern until the owner
/// decides on glow.
pub const UNLOCKS: &[(u32, &str)] = &[
    (1, "Ring style: Ember"),
    (2, "Theme: Dusk"),
    (3, "Ring style: Lattice"),
    (4, "Badge: Pathfinder"),
    (5, "Theme: Nocturne"),
    (6, "Ring style: Meridian"),
];

/// The welcome grant: 100 CGT in Sparks (18 decimals), set by the owner with ADR-078.
pub const WELCOME_SPARKS: &str = "100000000000000000000";

/// The three tasks that make an account owed the welcome grant.
const WELCOME_TASKS: [&str; 3] = ["tutorial", "verify-email", "link-key"];

/// Total XP a level needs: 50 x L x (L + 1).
pub fn xp_for_level(level: u32) -> i64 {
    50 * i64::from(level) * (i64::from(level) + 1)
}

/// The level a total of XP reaches.
pub fn level_for(xp: i64) -> u32 {
    let mut level = 0;
    while xp_for_level(level + 1) <= xp {
        level += 1;
    }
    level
}

pub fn task(key: &str) -> Option<&'static Task> {
    TASKS.iter().find(|t| t.key == key)
}

/// Grant a task's XP to an account, once. Returns whether it was new. Then records the welcome grant if this completed
/// what it needs.
pub async fn award(db: &sqlx::PgPool, user_id: Uuid, key: &str) -> AppResult<bool> {
    let Some(task) = task(key) else {
        return Err(AppError::ValidationError("Unknown task".into()));
    };
    let inserted = sqlx::query(
        "INSERT INTO progress_events (user_id, task, xp) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING",
    )
    .bind(user_id)
    .bind(task.key)
    .bind(task.xp)
    .execute(db)
    .await?
    .rows_affected()
        == 1;
    if inserted && WELCOME_TASKS.contains(&task.key) {
        record_welcome(db, user_id).await?;
    }
    Ok(inserted)
}

/// Record the welcome grant as owed if the account has finished the tutorial, verified its email and linked a key, is
/// active, and neither its email address nor its chain account has had a grant before. Nothing if not.
async fn record_welcome(db: &sqlx::PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO welcome_grants (user_id, email_lower, chain_account_id, amount_sparks)
        SELECT u.id, LOWER(u.email), u.chain_account_id, $2
        FROM users u
        WHERE u.id = $1
          AND u.status = 'active'
          AND u.email IS NOT NULL
          AND u.email_verified
          AND u.chain_account_id IS NOT NULL
          AND (SELECT COUNT(*) FROM progress_events p
               WHERE p.user_id = u.id AND p.task = ANY($3)) = 3
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(user_id)
    .bind(WELCOME_SPARKS)
    .bind(&WELCOME_TASKS[..])
    .execute(db)
    .await?;
    Ok(())
}

/// An account's level, XP, tasks, next unlock and welcome grant, as every app shows them.
pub async fn summary(db: &sqlx::PgPool, user_id: Uuid) -> AppResult<Value> {
    let done: Vec<(String,)> =
        sqlx::query_as("SELECT task FROM progress_events WHERE user_id = $1")
            .bind(user_id)
            .fetch_all(db)
            .await?;
    let xp: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(xp), 0)::BIGINT FROM progress_events WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?;
    let welcome: Option<(String,)> =
        sqlx::query_as("SELECT status FROM welcome_grants WHERE user_id = $1")
            .bind(user_id)
            .fetch_optional(db)
            .await?;
    Ok(summary_of(
        xp,
        &done.into_iter().map(|(t,)| t).collect::<Vec<_>>(),
        welcome.map(|(s,)| s),
    ))
}

fn summary_of(xp: i64, done: &[String], welcome: Option<String>) -> Value {
    let level = level_for(xp);
    let next = level + 1;
    let unlock = |l: u32| {
        UNLOCKS
            .iter()
            .find(|(at, _)| *at == l)
            .map(|(_, what)| *what)
    };
    json!({
        "level": level,
        "xp": xp,
        "level_xp": xp_for_level(level),
        "next_level_xp": xp_for_level(next),
        "next_unlock": unlock(next),
        "unlocked": UNLOCKS.iter().filter(|(at, _)| *at <= level).map(|(_, what)| *what).collect::<Vec<_>>(),
        "tasks": TASKS.iter().map(|t| json!({
            "key": t.key, "name": t.name, "xp": t.xp, "done": done.iter().any(|d| d == t.key),
        })).collect::<Vec<_>>(),
        "welcome": {
            "amount_cgt": "100",
            "status": welcome.unwrap_or_else(|| "not_yet".into()),
        },
    })
}

/// GET /api/v1/profile/progress
pub async fn get_progress(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> AppResult<Json<Value>> {
    Ok(Json(summary(&state.db, user_id).await?))
}

/// POST /api/v1/profile/progress/tutorial: the launcher reports the tutorial finished. Once; XP only.
pub async fn tutorial_done(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> AppResult<Json<Value>> {
    award(&state.db, user_id, "tutorial").await?;
    Ok(Json(summary(&state.db, user_id).await?))
}

#[derive(Deserialize)]
pub struct AppProgressForm {
    client_id: Option<String>,
    client_secret: Option<String>,
    /// The person's access token, issued to this app.
    token: Option<String>,
    task: Option<String>,
}

/// POST /oauth/progress: a registered app's server reports a task it checked, for the person whose token it holds. The
/// app authenticates with its secret, the token must be live and issued to that app, and the task must be one that app
/// may report.
pub async fn app_progress(
    State(state): State<Arc<AppState>>,
    Form(form): Form<AppProgressForm>,
) -> Response {
    let refused =
        |status: StatusCode, error: &str| (status, Json(json!({ "error": error }))).into_response();
    let Some(client) = crate::handlers::oauth::authenticated_client(
        &state,
        form.client_id.as_deref(),
        form.client_secret.as_deref(),
    ) else {
        return refused(StatusCode::UNAUTHORIZED, "invalid_client");
    };
    let Some(task) = form.task.as_deref().and_then(task) else {
        return refused(StatusCode::BAD_REQUEST, "unknown_task");
    };
    if task.source != Source::App(leak_free(&client.id)) {
        return refused(StatusCode::FORBIDDEN, "task_not_this_apps");
    }
    let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
    let claims = match sessions
        .authenticate_access_token(form.token.as_deref().unwrap_or_default())
        .await
    {
        Ok(claims) if claims.cid.as_deref() == Some(client.id.as_str()) => claims,
        _ => return refused(StatusCode::UNAUTHORIZED, "invalid_token"),
    };
    let Ok(user_id) = Uuid::parse_str(&claims.sub) else {
        return refused(StatusCode::UNAUTHORIZED, "invalid_token");
    };
    match award(&state.db, user_id, task.key).await {
        Ok(_) => match summary(&state.db, user_id).await {
            Ok(body) => Json(body).into_response(),
            Err(e) => e.into_response(),
        },
        Err(e) => e.into_response(),
    }
}

/// The registered app ids that may report tasks, as `'static` names for comparison with `Source::App`.
fn leak_free(id: &str) -> &'static str {
    TASKS
        .iter()
        .find_map(|t| match t.source {
            Source::App(app) if app == id => Some(app),
            _ => None,
        })
        .unwrap_or("")
}

// ── Three accounts per network address (ADR-078 decision 9) ──────────────────────────────────────────────────

/// How many accounts one address may create in 30 days.
pub const PER_ADDRESS: i64 = 3;

/// The address a request came from: `X-Real-IP`, which Railway's edge sets to the client's address
/// (docs.railway.com, networking specifications). `X-Forwarded-For` is not read: a client can write anything into it.
/// None when the header is absent, as in a direct local run, and then no limit applies.
pub fn client_address(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_owned)
}

/// A keyed hash of an address: the same address gives the same hash, and the address cannot be read back from it
/// without QOR ID's secret.
pub fn address_hash(state: &AppState, address: &str) -> String {
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(
        format!("signup-address:{}", state.config.jwt.access_secret).as_bytes(),
    )
    .expect("HMAC takes a key of any length");
    mac.update(address.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Refuse a sign-up from an address that created three accounts in the last 30 days.
pub async fn check_address(state: &AppState, hash: &str) -> AppResult<()> {
    sqlx::query("DELETE FROM signup_addresses WHERE created_at < NOW() - INTERVAL '30 days'")
        .execute(&state.db)
        .await?;
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM signup_addresses WHERE addr_hash = $1")
            .bind(hash)
            .fetch_one(&state.db)
            .await?;
    if count >= PER_ADDRESS {
        return Err(AppError::RateLimited(
            "Too many QOR IDs were created from this network recently. Try again later, or from another network.".into(),
        ));
    }
    Ok(())
}

pub async fn record_address(state: &AppState, hash: &str, user_id: Uuid) -> AppResult<()> {
    sqlx::query("INSERT INTO signup_addresses (addr_hash, user_id) VALUES ($1, $2)")
        .bind(hash)
        .bind(user_id)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// The new account's id from a sign-up's response body.
fn created_user(body: &Value) -> Option<Uuid> {
    body["user_id"]
        .as_str()
        .and_then(|id| Uuid::parse_str(id).ok())
}

/// POST /api/v1/auth/register, with the address limit around it.
pub async fn register_limited(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<crate::models::RegisterRequest>,
) -> AppResult<(StatusCode, Json<Value>)> {
    let hash = client_address(&headers).map(|a| address_hash(&state, &a));
    if let Some(hash) = &hash {
        check_address(&state, hash).await?;
    }
    let created = crate::handlers::auth::register(State(state.clone()), Json(req)).await?;
    if let (Some(hash), Some(user)) = (&hash, created_user(&created.1)) {
        record_address(&state, hash, user).await?;
    }
    Ok(created)
}

/// POST /api/v1/auth/keypair-register, with the address limit around it. A key-made account has its key linked from the
/// start, so it is granted that task at once.
pub async fn keypair_register_limited(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(req): Json<crate::models::user::KeypairRegisterRequest>,
) -> AppResult<(StatusCode, Json<Value>)> {
    let hash = client_address(&headers).map(|a| address_hash(&state, &a));
    if let Some(hash) = &hash {
        check_address(&state, hash).await?;
    }
    let created = crate::handlers::auth::keypair_register(State(state.clone()), Json(req)).await?;
    if let Some(user) = created_user(&created.1) {
        if let Some(hash) = &hash {
            record_address(&state, hash, user).await?;
        }
        award(&state.db, user, "link-key").await?;
    }
    Ok(created)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_fifty_l_l_plus_one() {
        assert_eq!(level_for(0), 0);
        assert_eq!(level_for(99), 0);
        assert_eq!(level_for(100), 1, "the tutorial, an email and a key");
        assert_eq!(level_for(299), 1);
        assert_eq!(level_for(300), 2);
        assert_eq!(level_for(600), 3);
        assert_eq!(xp_for_level(10), 5500);
    }

    #[test]
    fn setup_alone_reaches_level_one() {
        let setup: i32 = ["tutorial", "verify-email", "link-key"]
            .iter()
            .map(|k| task(k).unwrap().xp)
            .sum();
        assert_eq!(level_for(i64::from(setup)), 1);
    }

    #[test]
    fn the_summary_names_the_next_unlock_and_the_bar() {
        let s = summary_of(
            150,
            &["tutorial".into(), "verify-email".into(), "link-key".into()],
            Some("owed".into()),
        );
        assert_eq!(s["level"], 1);
        assert_eq!(s["level_xp"], 100);
        assert_eq!(s["next_level_xp"], 300);
        assert_eq!(s["next_unlock"], "Theme: Dusk");
        assert_eq!(s["unlocked"], json!(["Ring style: Ember"]));
        assert_eq!(s["welcome"]["status"], "owed");
    }

    #[test]
    fn the_address_is_railways_x_real_ip_and_nothing_a_client_writes() {
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", "1.2.3.4".parse().unwrap());
        assert_eq!(
            client_address(&headers),
            None,
            "X-Forwarded-For is not trusted"
        );
        headers.insert("x-real-ip", "203.0.113.7".parse().unwrap());
        assert_eq!(client_address(&headers).as_deref(), Some("203.0.113.7"));
    }
}

/// Against a real Postgres (and Redis for the app's report): tasks once, the welcome grant's conditions and uniqueness,
/// the address limit, and what an app may report.
#[cfg(test)]
mod db_tests {
    use super::*;
    use crate::config::{AppConfig, OAuthClient};
    use crate::models::Session;
    use crate::services::session_service::NewSession;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

    fn state(db: PgPool, secret: Option<&str>) -> Arc<AppState> {
        let url = std::env::var("QOR_AUTH_TEST_REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1:1".into());
        let redis = deadpool_redis::Config::from_url(url)
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("redis pool");
        let mut config = AppConfig::default();
        config.oauth.clients = vec![
            OAuthClient {
                id: "arqade".into(),
                name: "ARQADE".into(),
                redirect_uris: vec!["https://arqade.test/api/auth/callback".into()],
                secret_sha256: secret
                    .map(|s| hex::encode(<Sha256 as sha2::Digest>::digest(s.as_bytes()))),
            },
            OAuthClient {
                id: "other".into(),
                name: "Other".into(),
                redirect_uris: vec!["https://other.test/cb".into()],
                secret_sha256: Some(hex::encode(<Sha256 as sha2::Digest>::digest(
                    b"other-secret",
                ))),
            },
        ];
        Arc::new(AppState::new(
            config,
            db,
            redis,
            EmailService::new(EmailConfig::unconfigured()),
        ))
    }

    async fn account(
        db: &PgPool,
        name: &str,
        email: Option<&str>,
        verified: bool,
        key: Option<u8>,
    ) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO users (username, email, password_hash, email_verified, chain_account_id) VALUES ($1, $2, 'x', $3, $4) RETURNING id",
        )
        .bind(name)
        .bind(email)
        .bind(verified)
        .bind(key.map(|k| vec![k; 32]))
        .fetch_one(db)
        .await
        .expect("account")
    }

    async fn grants(db: &PgPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM welcome_grants")
            .fetch_one(db)
            .await
            .expect("count")
    }

    #[sqlx::test]
    async fn a_task_counts_once_and_setup_reaches_level_one(db: PgPool) {
        let user = account(&db, "player", Some("p@example.invalid"), true, Some(1)).await;
        assert!(award(&db, user, "tutorial").await.unwrap());
        assert!(!award(&db, user, "tutorial").await.unwrap(), "once");
        assert!(award(&db, user, "verify-email").await.unwrap());
        assert!(award(&db, user, "link-key").await.unwrap());
        assert!(matches!(
            award(&db, user, "made-up").await,
            Err(AppError::ValidationError(_))
        ));
        let s = summary(&db, user).await.unwrap();
        assert_eq!(s["xp"], 100);
        assert_eq!(s["level"], 1);
        assert_eq!(s["welcome"]["status"], "owed");
        assert_eq!(
            s["tasks"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|t| t["done"] == true)
                .count(),
            3
        );
    }

    #[sqlx::test]
    async fn the_welcome_grant_needs_all_three_and_is_owed_once_per_email_and_key(db: PgPool) {
        // No verified email: no grant, whatever else is done.
        let unverified =
            account(&db, "unverified", Some("u@example.invalid"), false, Some(2)).await;
        for t in ["tutorial", "verify-email", "link-key"] {
            award(&db, unverified, t).await.unwrap();
        }
        assert_eq!(grants(&db).await, 0, "the email must actually be verified");
        // No key: no grant.
        let keyless = account(&db, "keyless", Some("k@example.invalid"), true, None).await;
        for t in ["tutorial", "verify-email", "link-key"] {
            award(&db, keyless, t).await.unwrap();
        }
        assert_eq!(grants(&db).await, 0, "a chain account must be linked");
        // Everything: one grant, of 100 CGT, owed.
        let first = account(&db, "first", Some("Same@Example.invalid"), true, Some(3)).await;
        for t in ["tutorial", "verify-email", "link-key"] {
            award(&db, first, t).await.unwrap();
        }
        assert_eq!(grants(&db).await, 1);
        let (amount, status): (String, String) =
            sqlx::query_as("SELECT amount_sparks, status FROM welcome_grants WHERE user_id = $1")
                .bind(first)
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(amount, "100000000000000000000");
        assert_eq!(status, "owed");
        // The same key on another account: no second grant.
        let same_key = account(&db, "samekey", Some("other@example.invalid"), true, None).await;
        sqlx::query("UPDATE users SET chain_account_id = NULL WHERE id = $1")
            .bind(first)
            .execute(&db)
            .await
            .unwrap();
        sqlx::query("UPDATE users SET chain_account_id = $2 WHERE id = $1")
            .bind(same_key)
            .bind(vec![3u8; 32])
            .execute(&db)
            .await
            .unwrap();
        for t in ["tutorial", "verify-email", "link-key"] {
            award(&db, same_key, t).await.unwrap();
        }
        assert_eq!(grants(&db).await, 1, "one grant per key");
        // The same email address, in another case, on another account with another key: no second grant.
        sqlx::query("UPDATE users SET email = 'gone@example.invalid' WHERE id = $1")
            .bind(first)
            .execute(&db)
            .await
            .unwrap();
        let same_email = account(
            &db,
            "sameemail",
            Some("same@example.INVALID"),
            true,
            Some(4),
        )
        .await;
        for t in ["tutorial", "verify-email", "link-key"] {
            award(&db, same_email, t).await.unwrap();
        }
        assert_eq!(grants(&db).await, 1, "one grant per email address");
    }

    #[sqlx::test]
    async fn a_fourth_account_from_one_address_is_refused(db: PgPool) {
        let state = state(db.clone(), None);
        let here = address_hash(&state, "203.0.113.7");
        assert_ne!(here, address_hash(&state, "203.0.113.8"));
        assert!(!here.contains("203.0.113.7"), "the address is not kept");
        for i in 0..3 {
            check_address(&state, &here).await.expect("allowed");
            let user = account(&db, &format!("net{i}"), None, false, None).await;
            record_address(&state, &here, user).await.unwrap();
        }
        assert!(matches!(
            check_address(&state, &here).await,
            Err(AppError::RateLimited(_))
        ));
        check_address(&state, &address_hash(&state, "198.51.100.1"))
            .await
            .expect("another address is not limited");
        // After 30 days the address is forgotten.
        sqlx::query("UPDATE signup_addresses SET created_at = NOW() - INTERVAL '31 days'")
            .execute(&db)
            .await
            .unwrap();
        check_address(&state, &here).await.expect("forgotten");
        let kept: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM signup_addresses")
            .fetch_one(&db)
            .await
            .unwrap();
        assert_eq!(kept, 0, "and deleted");
    }

    async fn token_for(state: &AppState, user: Uuid, client: &str) -> String {
        let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
        let session = sessions
            .create_app_session(
                NewSession {
                    user_id: user,
                    qor_id: "player",
                    role: Some("user"),
                    device_id: "test",
                    ip_address: "0.0.0.0",
                    user_agent: None,
                    scopes: Session::default_scopes(),
                },
                client,
                3600,
            )
            .await
            .expect("app session");
        sessions
            .generate_tokens(&session, Some("user"))
            .expect("tokens")
            .access_token
    }

    async fn report(
        state: &Arc<AppState>,
        id: &str,
        secret: &str,
        token: &str,
        task: &str,
    ) -> StatusCode {
        app_progress(
            State(state.clone()),
            Form(AppProgressForm {
                client_id: Some(id.into()),
                client_secret: Some(secret.into()),
                token: Some(token.into()),
                task: Some(task.into()),
            }),
        )
        .await
        .status()
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn an_app_reports_only_its_own_tasks_for_its_own_players(db: PgPool) {
        let state = state(db.clone(), Some("arqade-secret"));
        let user = account(&db, "player", None, false, None).await;
        let arqade = token_for(&state, user, "arqade").await;
        let other = token_for(&state, user, "other").await;
        assert_eq!(
            report(&state, "arqade", "wrong", &arqade, "first-match").await,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            report(&state, "arqade", "arqade-secret", &other, "first-match").await,
            StatusCode::UNAUTHORIZED,
            "another app's token"
        );
        assert_eq!(
            report(&state, "other", "other-secret", &other, "first-match").await,
            StatusCode::FORBIDDEN,
            "ARQADE's task"
        );
        assert_eq!(
            report(&state, "arqade", "arqade-secret", &arqade, "tutorial").await,
            StatusCode::FORBIDDEN,
            "not an app's task"
        );
        assert_eq!(
            report(&state, "arqade", "arqade-secret", &arqade, "verify-email").await,
            StatusCode::FORBIDDEN,
            "QOR ID's own"
        );
        assert_eq!(
            summary(&db, user).await.unwrap()["xp"],
            0,
            "nothing granted yet"
        );
        assert_eq!(
            report(&state, "arqade", "arqade-secret", &arqade, "first-match").await,
            StatusCode::OK
        );
        assert_eq!(
            report(&state, "arqade", "arqade-secret", &arqade, "first-match").await,
            StatusCode::OK,
            "again: once"
        );
        assert_eq!(summary(&db, user).await.unwrap()["xp"], 25);
    }
}
