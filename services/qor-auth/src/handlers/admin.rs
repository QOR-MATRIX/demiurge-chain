//! Admin handlers (protected - God-level access only).
//!
//! Every route here does what it reports, or refuses. A change an administrator
//! makes is written to the audit log, in the same transaction, under the
//! administrator's own id from the access token. There is no placeholder actor,
//! and migration 013 refuses the nil id outright.

use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use uuid::Uuid;

use crate::error::{AppError, AppResult};
use crate::services::SessionService;
use crate::services::email_service::address_hash;
use crate::state::AppState;

/// Pagination for the listing routes: `page` starts at 1, and `per_page` is 1 to 100.
#[derive(Debug, Deserialize)]
pub struct PageQuery {
    page: Option<u32>,
    per_page: Option<u32>,
}

/// Validated page number, page size and row offset.
fn page_bounds(query: &PageQuery, default_per_page: u32) -> AppResult<(u32, u32, i64)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(default_per_page);
    if page == 0 {
        return Err(AppError::ValidationError("page starts at 1".into()));
    }
    if !(1..=100).contains(&per_page) {
        return Err(AppError::ValidationError(
            "per_page must be between 1 and 100".into(),
        ));
    }
    Ok((page, per_page, i64::from(page - 1) * i64::from(per_page)))
}

/// Write an audit row for an administrator's action, inside the caller's transaction.
async fn audit(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    admin_id: Uuid,
    action: &str,
    details: Value,
) -> AppResult<()> {
    sqlx::query("INSERT INTO audit_log (user_id, action, details) VALUES ($1, $2, $3)")
        .bind(admin_id)
        .bind(action)
        .bind(details)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

/// List all users (paginated)
pub async fn list_users(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    let (page, per_page, offset) = page_bounds(&query, 20)?;

    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    // `email` is optional: accounts without one used to fail the whole listing.
    let users: Vec<Value> = sqlx::query_as::<_, (Uuid, Option<String>, String, i16, String, String, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, email, username, discriminator, role::text, status::text, created_at FROM users ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(i64::from(per_page))
    .bind(offset)
    .fetch_all(&state.db)
    .await?
    .iter()
    .map(|(id, email, username, _discriminator, role, status, created_at)| {
        json!({
            "id": id,
            "email": email,
            "qor_id": username.to_lowercase(),
            "role": role,
            "status": status,
            "created_at": created_at,
        })
    })
    .collect();

    Ok(Json(json!({
        "users": users,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": (total + i64::from(per_page) - 1) / i64::from(per_page),
    })))
}

/// Get a specific user by ID
pub async fn get_user(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let user = sqlx::query_as::<_, crate::models::User>("SELECT * FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;

    match user {
        Some(u) => Ok(Json(json!({
            "id": u.id,
            "email": u.email,
            "qor_id": u.qor_id(),
            "role": format!("{:?}", u.role).to_lowercase(),
            "status": format!("{:?}", u.status).to_lowercase(),
            "email_verified": u.email_verified,
            "avatar_url": u.avatar_url,
            // Null until the account proves a key (ADR-017); SS58 and hex once
            // it has (ADR-024).
            "on_chain": u
                .chain_account_id
                .as_deref()
                .map(crate::handlers::auth::ChainAccount::from_stored)
                .transpose()?
                .map(|a| a.as_json(state.config.chain.ss58_prefix))
                .unwrap_or(Value::Null),
            "created_at": u.created_at,
            "updated_at": u.updated_at,
        }))),
        None => Err(AppError::UserNotFound),
    }
}

/// Ban a user
#[derive(Debug, Deserialize)]
pub struct BanRequest {
    reason: Option<String>,
}

/// Ban a user and revoke their sessions
/// POST /api/v1/admin/users/{id}/ban
///
/// A user that does not exist is 404, and nothing is written. The ban and its
/// audit row are one transaction; the account's sessions are revoked after it
/// commits, so every token it holds stops working at once.
pub async fn ban_user(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<BanRequest>,
) -> AppResult<Json<Value>> {
    if user_id == admin_id {
        return Err(AppError::ValidationError(
            "An administrator cannot ban their own account".into(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let banned: Option<Uuid> = sqlx::query_scalar(
        "UPDATE users SET status = 'banned', updated_at = NOW() WHERE id = $1 RETURNING id",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    if banned.is_none() {
        return Err(AppError::NotFound("User not found".into()));
    }
    audit(
        &mut tx,
        admin_id,
        "user_banned",
        json!({ "target_user_id": user_id, "reason": req.reason }),
    )
    .await?;
    tx.commit().await?;

    SessionService::new(state.redis.clone(), state.config.jwt.clone())
        .delete_all_sessions(user_id)
        .await?;

    Ok(Json(json!({
        "message": "User banned, and their sessions revoked",
        "user_id": user_id,
    })))
}

/// Lift a ban
/// POST /api/v1/admin/users/{id}/unban
///
/// A user that does not exist is 404; one that is not banned is refused.
pub async fn unban_user(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let mut tx = state.db.begin().await?;
    let unbanned: Option<Uuid> = sqlx::query_scalar(
        "UPDATE users SET status = 'active', updated_at = NOW() WHERE id = $1 AND status = 'banned' RETURNING id",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;

    if unbanned.is_none() {
        let exists: Option<Uuid> = sqlx::query_scalar("SELECT id FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(&mut *tx)
            .await?;
        return Err(match exists {
            None => AppError::NotFound("User not found".into()),
            Some(_) => AppError::ValidationError("User is not banned".into()),
        });
    }

    audit(
        &mut tx,
        admin_id,
        "user_unbanned",
        json!({ "target_user_id": user_id }),
    )
    .await?;
    tx.commit().await?;

    Ok(Json(json!({
        "message": "User unbanned",
        "user_id": user_id,
    })))
}

/// Update user role
#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    role: String,
}

/// Change a user's role
/// POST /api/v1/admin/users/{id}/role
///
/// A user that does not exist is 404. Access tokens carry the role, so the
/// account's sessions are revoked after the change commits; it takes effect at
/// the user's next sign-in.
pub async fn update_role(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateRoleRequest>,
) -> AppResult<Json<Value>> {
    let valid_roles = ["user", "moderator", "admin", "god", "system"];
    if !valid_roles.contains(&req.role.as_str()) {
        return Err(AppError::ValidationError(format!(
            "Invalid role. Must be one of: {}",
            valid_roles.join(", ")
        )));
    }
    if user_id == admin_id {
        return Err(AppError::ValidationError(
            "An administrator cannot change their own role".into(),
        ));
    }

    let mut tx = state.db.begin().await?;
    // The column is an enum; binding the role as text without the cast is refused by Postgres.
    let updated: Option<Uuid> = sqlx::query_scalar(
        "UPDATE users SET role = $1::user_role, updated_at = NOW() WHERE id = $2 RETURNING id",
    )
    .bind(&req.role)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?;
    if updated.is_none() {
        return Err(AppError::NotFound("User not found".into()));
    }
    audit(
        &mut tx,
        admin_id,
        "user_role_changed",
        json!({ "target_user_id": user_id, "new_role": req.role }),
    )
    .await?;
    tx.commit().await?;

    SessionService::new(state.redis.clone(), state.config.jwt.clone())
        .delete_all_sessions(user_id)
        .await?;

    Ok(Json(json!({
        "message": "Role updated, and the user's sessions revoked",
        "user_id": user_id,
        "new_role": req.role,
    })))
}

/// Transfer CGT for customer support: not implemented.
///
/// This previously answered "Token transfer initiated" with an all-zero
/// transaction hash while doing nothing. It now refuses, so no operator can
/// believe CGT moved. CGT moves only in signed transactions (D-008).
pub async fn transfer_tokens(State(_state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    Err(AppError::NotImplemented(
        "Admin CGT transfers are not implemented; no transaction was submitted".into(),
    ))
}

/// Refund CGT for customer support: not implemented.
///
/// Like `transfer_tokens`, this previously reported success with an all-zero
/// transaction hash. It now refuses.
pub async fn refund_tokens(State(_state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    Err(AppError::NotImplemented(
        "Admin CGT refunds are not implemented; no transaction was submitted".into(),
    ))
}

/// Get system statistics
/// GET /api/v1/admin/stats
///
/// `active_sessions` counts the sessions held in Redis, which expire on their
/// own. `logins_24h` counts the `login` rows that sign-in writes to the audit
/// log; sign-ins from before those rows were written are not counted.
pub async fn get_stats(State(state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    let total_users: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db)
        .await?;

    let active_sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone())
        .count_sessions()
        .await?;

    let registrations_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM users WHERE created_at > NOW() - INTERVAL '24 hours'",
    )
    .fetch_one(&state.db)
    .await?;

    let logins_24h: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'login' AND created_at > NOW() - INTERVAL '24 hours'"
    )
    .fetch_one(&state.db)
    .await?;

    let users_by_role: Vec<(String, i64)> =
        sqlx::query_as("SELECT role::text, COUNT(*) FROM users GROUP BY role")
            .fetch_all(&state.db)
            .await?;

    Ok(Json(json!({
        "total_users": total_users,
        "active_sessions": active_sessions,
        "registrations_24h": registrations_24h,
        "logins_24h": logins_24h,
        "users_by_role": users_by_role.iter().map(|(role, count)| json!({
            "role": role,
            "count": count
        })).collect::<Vec<_>>(),
    })))
}

/// Get audit log
pub async fn get_audit_log(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    let (page, per_page, offset) = page_bounds(&query, 50)?;

    // `ip_address` is INET; it is read as text.
    let logs: Vec<Value> = sqlx::query_as::<_, (Uuid, Option<Uuid>, String, Value, Option<String>, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, user_id, action, details, host(ip_address), created_at FROM audit_log ORDER BY created_at DESC LIMIT $1 OFFSET $2"
    )
    .bind(i64::from(per_page))
    .bind(offset)
    .fetch_all(&state.db)
    .await?
    .iter()
    .map(|(id, user_id, action, details, ip_address, created_at)| {
        json!({
            "id": id,
            "user_id": user_id,
            "action": action,
            "details": details,
            "ip_address": ip_address,
            "created_at": created_at,
        })
    })
    .collect();

    Ok(Json(json!({
        "logs": logs,
        "page": page,
        "per_page": per_page,
    })))
}

/// Take an address off the list of undeliverable addresses. Deliberately not `Debug`: it carries an
/// address.
#[derive(Deserialize)]
pub struct UnmarkAddressRequest {
    email: String,
    reason: String,
}

/// Take an address off the list of undeliverable addresses
/// POST /api/v1/admin/email-suppressions/unmark
///
/// For an address marked after a permanent bounce, a Resend suppression or a complaint, once an
/// administrator has established that it receives mail again and that its owner wants mail there.
///
/// Admin-only, like every route here, and never for the account holder: if the account that holds an
/// address could clear its mark, the mark would mean nothing. So a god account cannot clear an address
/// that is its own, or pending on its own account, either.
///
/// The address travels in the body, never the path, because request paths are logged. A reason is
/// required. Removing the mark and writing its audit row, under the acting administrator, are one
/// transaction. The audit row holds the address's hash, the reason it was marked and Resend's email id,
/// never the address. An address that is not marked is 404, and nothing is written.
///
/// Resend keeps a suppression list of its own. An address Resend suppresses stays refused by Resend
/// until it is removed there too, and its next report marks it here again.
pub async fn unmark_undeliverable_address(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Json(req): Json<UnmarkAddressRequest>,
) -> AppResult<Json<Value>> {
    let address = req.email.trim().to_lowercase();
    if !address.contains('@') || !address.contains('.') {
        return Err(AppError::ValidationError("Invalid email format".into()));
    }
    let reason = req.reason.trim();
    if reason.is_empty() || reason.chars().count() > 500 {
        return Err(AppError::ValidationError(
            "A reason of 1 to 500 characters is required".into(),
        ));
    }

    let own: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM users WHERE id = $1 AND (LOWER(email) = $2 OR LOWER(pending_email) = $2))",
    )
    .bind(admin_id)
    .bind(&address)
    .fetch_one(&state.db)
    .await?;
    if own {
        return Err(AppError::InsufficientPermissions);
    }

    let hash = address_hash(&address);
    let mut tx = state.db.begin().await?;
    let removed: Option<(String, Option<String>)> = sqlx::query_as(
        "DELETE FROM email_suppressions WHERE address_hash = $1 RETURNING reason, resend_email_id",
    )
    .bind(&hash)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((was, resend_email_id)) = removed else {
        return Err(AppError::NotFound(
            "That address is not marked undeliverable".into(),
        ));
    };
    let accounts: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE LOWER(email) = $1")
        .bind(&address)
        .fetch_one(&mut *tx)
        .await?;
    audit(
        &mut tx,
        admin_id,
        "email_unsuppressed",
        json!({
            "address_hash": hash,
            "was": was,
            "resend_email_id": resend_email_id,
            "reason": reason,
            "accounts": accounts,
        }),
    )
    .await?;
    tx.commit().await?;

    Ok(Json(json!({
        "message": "The address is no longer marked undeliverable, and email to it is sent again.",
        "was": was,
        "accounts": accounts,
    })))
}

/// The welcome grants that are owed and not yet paid, oldest first (ADR-078 decision 7).
///
/// GET /api/v1/admin/grants/owed
///
/// Each grant is named by its account's id, because an account has at most one. The chain account is given as hex
/// and as SS58 (the configured prefix), and the amount in Sparks as a decimal string, as stored. No email address is
/// returned: paying a grant needs only the chain account.
pub async fn list_owed_grants(
    State(state): State<Arc<AppState>>,
    Query(query): Query<PageQuery>,
) -> AppResult<Json<Value>> {
    let (page, per_page, offset) = page_bounds(&query, 50)?;

    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM welcome_grants WHERE status = 'owed'")
            .fetch_one(&state.db)
            .await?;

    let rows: Vec<(Uuid, Vec<u8>, String)> = sqlx::query_as(
        "SELECT user_id, chain_account_id, amount_sparks FROM welcome_grants WHERE status = 'owed' ORDER BY created_at ASC, user_id ASC LIMIT $1 OFFSET $2",
    )
    .bind(i64::from(per_page))
    .bind(offset)
    .fetch_all(&state.db)
    .await?;

    let prefix = state.config.chain.ss58_prefix;
    let grants: Vec<Value> = rows
        .iter()
        .map(|(user_id, account, amount_sparks)| {
            let chain = crate::handlers::auth::ChainAccount::from_stored(account)
                .map(|a| a.as_json(prefix))
                .unwrap_or(Value::Null);
            json!({
                "user_id": user_id,
                "chain_account": chain,
                "amount_sparks": amount_sparks,
            })
        })
        .collect();

    Ok(Json(json!({
        "grants": grants,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": (total + i64::from(per_page) - 1) / i64::from(per_page),
    })))
}

/// The evidence that an owed grant was paid: the hash of the finalised transfer, or of the block that holds it.
#[derive(Debug, Deserialize)]
pub struct MarkPaidRequest {
    /// 32 bytes as `0x` and 64 hex digits, the shape of every Demiurge hash.
    tx_hash: String,
}

/// Whether `value` is `0x` followed by 64 hex digits.
fn is_chain_hash(value: &str) -> bool {
    value
        .strip_prefix("0x")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|b| b.is_ascii_hexdigit()))
}

/// Record that an owed welcome grant was paid, once the transfer is finalised on the chain (ADR-078 decision 7).
///
/// POST /api/v1/admin/grants/{user_id}/mark-paid
///
/// This moves no CGT: the transfer is made from the Welcome account first, and its hash is the evidence kept in
/// `paid_block`. Only an owed grant can be marked; one already paid, or none at all, is a 404 and nothing is written.
pub async fn mark_grant_paid(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<MarkPaidRequest>,
) -> AppResult<Json<Value>> {
    let tx_hash = req.tx_hash.trim().to_ascii_lowercase();
    if !is_chain_hash(&tx_hash) {
        return Err(AppError::ValidationError(
            "tx_hash must be 0x followed by 64 hex digits".into(),
        ));
    }

    let mut tx = state.db.begin().await?;
    let updated = sqlx::query(
        "UPDATE welcome_grants SET status = 'paid', paid_at = NOW(), paid_block = $1 WHERE user_id = $2 AND status = 'owed'",
    )
    .bind(&tx_hash)
    .bind(user_id)
    .execute(&mut *tx)
    .await?
    .rows_affected();

    if updated == 0 {
        return Err(AppError::NotFound(
            "No owed welcome grant for that account".into(),
        ));
    }

    audit(
        &mut tx,
        admin_id,
        "welcome_grant_paid",
        json!({ "user_id": user_id, "paid_block": tx_hash }),
    )
    .await?;
    tx.commit().await?;

    Ok(Json(json!({ "status": "paid", "paid_block": tx_hash })))
}

/// The admin handlers, called directly against a real Postgres (`#[sqlx::test]`). Tests that also
/// need Redis are ignored by default: `QOR_AUTH_TEST_REDIS_URL=redis://... cargo test -- --ignored`.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::session_service::NewSession;
    use crate::services::{EmailConfig, EmailService};
    use sqlx::PgPool;

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

    async fn account(db: &PgPool, username: &str, email: Option<&str>) -> Uuid {
        sqlx::query_scalar(
            "INSERT INTO users (email, username, discriminator, password_hash, email_verified) VALUES ($1, $2, 1, 'x', $3) RETURNING id",
        )
        .bind(email)
        .bind(username)
        .bind(email.is_none())
        .fetch_one(db)
        .await
        .expect("account")
    }

    async fn audit_rows(db: &PgPool) -> Vec<(Option<Uuid>, String, Value)> {
        sqlx::query_as("SELECT user_id, action, details FROM audit_log ORDER BY created_at")
            .fetch_all(db)
            .await
            .expect("audit rows")
    }

    async fn status_of(db: &PgPool, user_id: Uuid) -> String {
        sqlx::query_scalar("SELECT status::text FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(db)
            .await
            .expect("status")
    }

    fn page(page: Option<u32>, per_page: Option<u32>) -> Query<PageQuery> {
        Query(PageQuery { page, per_page })
    }

    async fn mark(db: &PgPool, address: &str) {
        sqlx::query(
            "INSERT INTO email_suppressions (address_hash, reason, resend_email_id) VALUES ($1, 'bounce', 'email-1')",
        )
        .bind(address_hash(address))
        .execute(db)
        .await
        .expect("mark");
    }

    fn unmark(email: &str, reason: &str) -> Json<UnmarkAddressRequest> {
        Json(UnmarkAddressRequest {
            email: email.into(),
            reason: reason.into(),
        })
    }

    #[sqlx::test]
    async fn an_administrator_unmarks_an_address_and_the_action_is_audited(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        account(&db, "holder", Some("holder@example.invalid")).await;
        mark(&db, "holder@example.invalid").await;
        assert!(
            state
                .email_service
                .is_suppressed("holder@example.invalid")
                .await
                .expect("check")
        );

        let Json(body) = unmark_undeliverable_address(
            State(state.clone()),
            Extension(admin),
            unmark("Holder@Example.invalid", "The mailbox was recreated."),
        )
        .await
        .expect("unmarked");
        assert_eq!(body["was"], json!("bounce"));
        assert_eq!(body["accounts"], json!(1));
        assert!(
            !state
                .email_service
                .is_suppressed("holder@example.invalid")
                .await
                .expect("check")
        );

        let rows = audit_rows(&db).await;
        assert_eq!(rows.len(), 1);
        let (actor, action, details) = &rows[0];
        assert_eq!(*actor, Some(admin));
        assert_eq!(action, "email_unsuppressed");
        assert_eq!(details["reason"], json!("The mailbox was recreated."));
        assert_eq!(details["was"], json!("bounce"));
        assert!(
            !details.to_string().contains('@'),
            "the audit row holds no address"
        );
    }

    #[sqlx::test]
    async fn an_address_that_is_not_marked_is_not_found_and_nothing_is_written(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        assert!(matches!(
            unmark_undeliverable_address(
                State(state),
                Extension(admin),
                unmark("fine@example.invalid", "Checking."),
            )
            .await,
            Err(AppError::NotFound(_))
        ));
        assert!(audit_rows(&db).await.is_empty());
    }

    #[sqlx::test]
    async fn unmarking_needs_an_address_and_a_reason(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        mark(&db, "marked@example.invalid").await;
        let long = "x".repeat(501);
        for (email, reason) in [
            ("marked@example.invalid", ""),
            ("marked@example.invalid", "   "),
            ("marked@example.invalid", long.as_str()),
            ("not an address", "A reason."),
        ] {
            assert!(matches!(
                unmark_undeliverable_address(
                    State(state.clone()),
                    Extension(admin),
                    unmark(email, reason),
                )
                .await,
                Err(AppError::ValidationError(_))
            ));
        }
        assert!(
            state
                .email_service
                .is_suppressed("marked@example.invalid")
                .await
                .expect("check")
        );
        assert!(audit_rows(&db).await.is_empty());
    }

    #[sqlx::test]
    async fn an_administrator_cannot_unmark_their_own_address(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "selfadmin", Some("self@example.invalid")).await;
        mark(&db, "self@example.invalid").await;
        assert!(matches!(
            unmark_undeliverable_address(
                State(state.clone()),
                Extension(admin),
                unmark("SELF@example.invalid", "It is mine."),
            )
            .await,
            Err(AppError::InsufficientPermissions)
        ));

        sqlx::query("UPDATE users SET pending_email = 'next@example.invalid' WHERE id = $1")
            .bind(admin)
            .execute(&db)
            .await
            .expect("pending");
        mark(&db, "next@example.invalid").await;
        assert!(matches!(
            unmark_undeliverable_address(
                State(state.clone()),
                Extension(admin),
                unmark("next@example.invalid", "It will be mine."),
            )
            .await,
            Err(AppError::InsufficientPermissions)
        ));

        for address in ["self@example.invalid", "next@example.invalid"] {
            assert!(
                state
                    .email_service
                    .is_suppressed(address)
                    .await
                    .expect("check")
            );
        }
        assert!(audit_rows(&db).await.is_empty());
    }

    #[sqlx::test]
    async fn users_without_an_email_are_listed(db: PgPool) {
        account(&db, "noemail", None).await;
        account(&db, "hasemail", Some("hasemail@example.invalid")).await;

        let Json(body) = list_users(State(state(db)), page(None, None))
            .await
            .expect("listing");
        assert_eq!(body["total"], json!(2));
        assert_eq!(body["users"].as_array().map(Vec::len), Some(2));
        assert_eq!(body["total_pages"], json!(1));
    }

    #[sqlx::test]
    async fn out_of_range_pages_are_refused(db: PgPool) {
        let state = state(db);
        for (p, per_page) in [(Some(0), None), (None, Some(0)), (None, Some(101))] {
            assert!(matches!(
                list_users(State(state.clone()), page(p, per_page)).await,
                Err(AppError::ValidationError(_))
            ));
            assert!(matches!(
                get_audit_log(State(state.clone()), page(p, per_page)).await,
                Err(AppError::ValidationError(_))
            ));
        }
    }

    #[sqlx::test]
    async fn actions_on_a_missing_user_are_not_found_and_write_nothing(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let missing = Uuid::new_v4();

        assert!(matches!(
            ban_user(
                State(state.clone()),
                Extension(admin),
                Path(missing),
                Json(BanRequest { reason: None })
            )
            .await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            unban_user(State(state.clone()), Extension(admin), Path(missing)).await,
            Err(AppError::NotFound(_))
        ));
        assert!(matches!(
            update_role(
                State(state.clone()),
                Extension(admin),
                Path(missing),
                Json(UpdateRoleRequest {
                    role: "moderator".into()
                })
            )
            .await,
            Err(AppError::NotFound(_))
        ));
        assert!(audit_rows(&db).await.is_empty());
    }

    #[sqlx::test]
    async fn an_administrator_cannot_ban_or_change_the_role_of_their_own_account(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;

        assert!(matches!(
            ban_user(
                State(state.clone()),
                Extension(admin),
                Path(admin),
                Json(BanRequest { reason: None })
            )
            .await,
            Err(AppError::ValidationError(_))
        ));
        assert!(matches!(
            update_role(
                State(state.clone()),
                Extension(admin),
                Path(admin),
                Json(UpdateRoleRequest {
                    role: "user".into()
                })
            )
            .await,
            Err(AppError::ValidationError(_))
        ));
        assert_eq!(status_of(&db, admin).await, "active");
        assert!(audit_rows(&db).await.is_empty());
    }

    #[sqlx::test]
    async fn an_unknown_role_is_refused(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let target = account(&db, "target", None).await;
        assert!(matches!(
            update_role(
                State(state),
                Extension(admin),
                Path(target),
                Json(UpdateRoleRequest {
                    role: "emperor".into()
                })
            )
            .await,
            Err(AppError::ValidationError(_))
        ));
    }

    #[sqlx::test]
    async fn unbanning_an_account_that_is_not_banned_is_refused(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let target = account(&db, "target", None).await;
        assert!(matches!(
            unban_user(State(state), Extension(admin), Path(target)).await,
            Err(AppError::ValidationError(_))
        ));
        assert!(audit_rows(&db).await.is_empty());
    }

    #[sqlx::test]
    async fn unbanning_is_audited_under_the_acting_administrator(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let target = account(&db, "target", None).await;
        sqlx::query("UPDATE users SET status = 'banned' WHERE id = $1")
            .bind(target)
            .execute(&db)
            .await
            .expect("ban");

        let Json(body) = unban_user(State(state), Extension(admin), Path(target))
            .await
            .expect("unban");
        assert_eq!(body["user_id"], json!(target));
        assert_eq!(status_of(&db, target).await, "active");
        assert_eq!(
            audit_rows(&db).await,
            vec![(
                Some(admin),
                "user_unbanned".to_string(),
                json!({ "target_user_id": target })
            )]
        );
    }

    #[sqlx::test]
    async fn the_nil_user_id_cannot_be_an_audit_actor_or_an_account(db: PgPool) {
        let nil_audit = sqlx::query(
            "INSERT INTO audit_log (user_id, action, details) VALUES ('00000000-0000-0000-0000-000000000000', 'user_banned', '{}')",
        )
        .execute(&db)
        .await;
        assert!(nil_audit.is_err());

        let nil_account = sqlx::query(
            "INSERT INTO users (id, username, discriminator, password_hash, email_verified) VALUES ('00000000-0000-0000-0000-000000000000', 'nil', 1, 'x', TRUE)",
        )
        .execute(&db)
        .await;
        assert!(nil_account.is_err());
    }

    #[sqlx::test]
    async fn audit_rows_with_an_ip_address_are_listed(db: PgPool) {
        let admin = account(&db, "admin", None).await;
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, details, ip_address) VALUES ($1, 'login', '{}', '127.0.0.1')",
        )
        .bind(admin)
        .execute(&db)
        .await
        .expect("audit row");

        let Json(body) = get_audit_log(State(state(db)), page(None, None))
            .await
            .expect("audit log");
        assert_eq!(body["logs"][0]["ip_address"], json!("127.0.0.1"));
    }

    fn new_session(user_id: Uuid) -> NewSession<'static> {
        NewSession {
            user_id,
            qor_id: "target#0001",
            role: None,
            device_id: "test-device",
            ip_address: "127.0.0.1",
            user_agent: None,
            scopes: vec![],
        }
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn banning_revokes_sessions_and_is_audited_under_the_acting_administrator(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let target = account(&db, "target", None).await;
        let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
        let (session, _) = sessions
            .create_session(new_session(target))
            .await
            .expect("session");

        let Json(body) = ban_user(
            State(state),
            Extension(admin),
            Path(target),
            Json(BanRequest {
                reason: Some("test".into()),
            }),
        )
        .await
        .expect("ban");
        assert_eq!(body["user_id"], json!(target));
        assert_eq!(status_of(&db, target).await, "banned");
        assert!(
            sessions
                .get_session(session.session_id)
                .await
                .expect("lookup")
                .is_none()
        );
        assert_eq!(
            audit_rows(&db).await,
            vec![(
                Some(admin),
                "user_banned".to_string(),
                json!({ "target_user_id": target, "reason": "test" })
            )]
        );
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn changing_a_role_is_audited_and_revokes_sessions(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let target = account(&db, "target", None).await;
        let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());
        let (session, _) = sessions
            .create_session(new_session(target))
            .await
            .expect("session");

        let Json(body) = update_role(
            State(state),
            Extension(admin),
            Path(target),
            Json(UpdateRoleRequest {
                role: "moderator".into(),
            }),
        )
        .await
        .expect("role change");
        assert_eq!(body["new_role"], json!("moderator"));
        let role: String = sqlx::query_scalar("SELECT role::text FROM users WHERE id = $1")
            .bind(target)
            .fetch_one(&db)
            .await
            .expect("role");
        assert_eq!(role, "moderator");
        assert!(
            sessions
                .get_session(session.session_id)
                .await
                .expect("lookup")
                .is_none()
        );
        assert_eq!(
            audit_rows(&db).await,
            vec![(
                Some(admin),
                "user_role_changed".to_string(),
                json!({ "target_user_id": target, "new_role": "moderator" })
            )]
        );
    }

    #[sqlx::test]
    #[ignore = "needs a Redis at QOR_AUTH_TEST_REDIS_URL"]
    async fn statistics_count_live_sessions_and_recorded_logins(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let sessions = SessionService::new(state.redis.clone(), state.config.jwt.clone());

        sessions
            .create_session(new_session(admin))
            .await
            .expect("session");
        sessions
            .create_session(new_session(admin))
            .await
            .expect("session");
        for _ in 0..3 {
            sqlx::query(
                "INSERT INTO audit_log (user_id, action, details) VALUES ($1, 'login', '{}')",
            )
            .bind(admin)
            .execute(&db)
            .await
            .expect("login row");
        }

        let Json(body) = get_stats(State(state)).await.expect("stats");
        assert_eq!(body["total_users"], json!(1));
        assert_eq!(body["logins_24h"], json!(3));
        // Other tests share the Redis and create and delete sessions concurrently, so a
        // count taken before this call is no baseline. This account's two sessions are
        // its own and stay live until cleanup, so they bound the count from below.
        assert_eq!(sessions.list_sessions(admin).await.expect("own").len(), 2);
        assert!(body["active_sessions"].as_u64().expect("count") >= 2);

        sessions.delete_all_sessions(admin).await.expect("cleanup");
    }

    async fn owed_grant(db: &PgPool, username: &str, key_byte: u8) -> Uuid {
        let user = account(db, username, Some(&format!("{username}@example.invalid"))).await;
        sqlx::query(
            "INSERT INTO welcome_grants (user_id, email_lower, chain_account_id, amount_sparks) VALUES ($1, $2, $3, '100000000000000000000')",
        )
        .bind(user)
        .bind(format!("{username}@example.invalid"))
        .bind(vec![key_byte; 32])
        .execute(db)
        .await
        .expect("owed grant");
        user
    }

    fn paid(hash: &str) -> Json<MarkPaidRequest> {
        Json(MarkPaidRequest {
            tx_hash: hash.into(),
        })
    }

    #[sqlx::test]
    async fn owed_grants_are_listed_by_account_with_no_email_address(db: PgPool) {
        let state = state(db.clone());
        let first = owed_grant(&db, "first", 1).await;
        owed_grant(&db, "second", 2).await;

        let Json(body) = list_owed_grants(State(state), page(None, None))
            .await
            .expect("listed");
        assert_eq!(body["total"], json!(2));
        let grants = body["grants"].as_array().expect("grants");
        assert_eq!(grants.len(), 2);
        assert_eq!(grants[0]["user_id"], json!(first));
        assert_eq!(grants[0]["amount_sparks"], json!("100000000000000000000"));
        assert_eq!(
            grants[0]["chain_account"]["account_id"],
            json!(format!("0x{}", "01".repeat(32)))
        );
        assert!(
            !body.to_string().contains('@'),
            "the listing holds no address"
        );
    }

    #[sqlx::test]
    async fn marking_a_grant_paid_records_the_hash_audits_it_and_removes_it_from_the_owed_list(
        db: PgPool,
    ) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let user = owed_grant(&db, "player", 7).await;
        let hash = format!("0x{}", "AB".repeat(32));

        let Json(body) = mark_grant_paid(
            State(state.clone()),
            Extension(admin),
            Path(user),
            paid(&hash),
        )
        .await
        .expect("marked");
        let stored = hash.to_ascii_lowercase();
        assert_eq!(body["paid_block"], json!(stored));

        let (status, paid_block): (String, Option<String>) =
            sqlx::query_as("SELECT status, paid_block FROM welcome_grants WHERE user_id = $1")
                .bind(user)
                .fetch_one(&db)
                .await
                .expect("grant");
        assert_eq!(status, "paid");
        assert_eq!(paid_block.as_deref(), Some(stored.as_str()));

        let rows = audit_rows(&db).await;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].0, Some(admin));
        assert_eq!(rows[0].1, "welcome_grant_paid");

        let Json(listed) = list_owed_grants(State(state.clone()), page(None, None))
            .await
            .expect("listed");
        assert_eq!(listed["total"], json!(0));

        // Paid once is paid: a second mark is not found and writes nothing.
        assert!(matches!(
            mark_grant_paid(State(state), Extension(admin), Path(user), paid(&hash)).await,
            Err(AppError::NotFound(_))
        ));
        assert_eq!(audit_rows(&db).await.len(), 1);
    }

    #[sqlx::test]
    async fn a_grant_is_not_marked_paid_without_a_well_formed_hash_or_an_owed_grant(db: PgPool) {
        let state = state(db.clone());
        let admin = account(&db, "admin", None).await;
        let user = owed_grant(&db, "player", 9).await;

        for bad in [
            "",
            "0x1234",
            &"ab".repeat(32),
            &format!("0x{}", "zz".repeat(32)),
            &format!("0x{}", "ab".repeat(40)),
        ] {
            assert!(
                matches!(
                    mark_grant_paid(
                        State(state.clone()),
                        Extension(admin),
                        Path(user),
                        paid(bad)
                    )
                    .await,
                    Err(AppError::ValidationError(_))
                ),
                "{bad:?} is refused"
            );
        }
        let nobody = account(&db, "nobody", None).await;
        assert!(matches!(
            mark_grant_paid(
                State(state),
                Extension(admin),
                Path(nobody),
                paid(&format!("0x{}", "cd".repeat(32)))
            )
            .await,
            Err(AppError::NotFound(_))
        ));

        let status: String =
            sqlx::query_scalar("SELECT status FROM welcome_grants WHERE user_id = $1")
                .bind(user)
                .fetch_one(&db)
                .await
                .expect("grant");
        assert_eq!(status, "owed");
        assert!(audit_rows(&db).await.is_empty());
    }
}
