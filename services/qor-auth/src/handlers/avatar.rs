//! Avatars (ADR-079): one per QOR ID, kept in QOR ID's own database, served to anyone.
//!
//! - POST   /api/v1/profile/avatar         the signed-in account uploads an image or GIF (the body is the file)
//! - DELETE /api/v1/profile/avatar         and removes its own
//! - GET    /avatars/{hash}                anyone fetches one, by the SHA-256 of what is stored; cached for good
//! - GET    /avatars/{hash}/still          an animated one's first frame, for less motion (decision 6)
//! - POST   /api/v1/avatars/report         a signed-in account reports another's avatar
//! - GET    /api/v1/admin/avatar-reports   the owner's queue of reported avatars
//! - POST   /api/v1/admin/avatars/{hash}/remove   the owner removes one; its accounts fall back to their letter
//!
//! What is uploaded is never stored: `avatar_image::clean` decodes and re-encodes it (decision 3).

use axum::{
    Extension, Json,
    body::Bytes,
    extract::{Path, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

use crate::avatar_image::{self, Refusal};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// The reasons a report may give.
pub const REASONS: &[&str] = &["nudity", "violence", "hate", "harassment", "spam", "other"];

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// POST /api/v1/profile/avatar
pub async fn upload_avatar(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    body: Bytes,
) -> AppResult<Json<Value>> {
    // Decoding is work for a CPU, not for the async runtime.
    let cleaned = tokio::task::spawn_blocking(move || avatar_image::clean(&body))
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("avatar cleaning stopped: {e}")))?;
    let clean =
        cleaned.map_err(|refusal: Refusal| AppError::ValidationError(refusal.message().into()))?;
    let hash = hex::encode(Sha256::digest(&clean.bytes));
    let path = format!("/avatars/{hash}");

    let mut tx = state.db.begin().await?;
    sqlx::query(
        "INSERT INTO avatars (user_id, hash, content_type, bytes, still) VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id) DO UPDATE SET hash = EXCLUDED.hash, content_type = EXCLUDED.content_type,
             bytes = EXCLUDED.bytes, still = EXCLUDED.still, created_at = NOW()",
    )
    .bind(user_id)
    .bind(&hash)
    .bind(clean.content_type)
    .bind(&clean.bytes)
    .bind(&clean.still)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE users SET avatar_url = $1, updated_at = NOW() WHERE id = $2")
        .bind(&path)
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    // A new avatar answers any notice of a removed one.
    sqlx::query("UPDATE avatar_removals SET seen = TRUE WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(
        json!({ "avatar_url": path, "content_type": clean.content_type }),
    ))
}

/// DELETE /api/v1/profile/avatar
pub async fn delete_avatar(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> AppResult<Json<Value>> {
    let mut tx = state.db.begin().await?;
    sqlx::query("DELETE FROM avatars WHERE user_id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("UPDATE users SET avatar_url = NULL, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(Json(json!({ "avatar_url": null })))
}

/// GET /avatars/{hash}: the stored bytes, cached for good (the URL changes when the picture does), drawn as an image
/// only, and allowed onto other sites' pages (ARQADE shows them).
pub async fn serve_avatar(
    State(state): State<Arc<AppState>>,
    Path(hash): Path<String>,
) -> Response {
    if !valid_hash(&hash) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let row: Result<Option<(String, Vec<u8>)>, _> =
        sqlx::query_as("SELECT content_type, bytes FROM avatars WHERE hash = $1 LIMIT 1")
            .bind(&hash)
            .fetch_optional(&state.db)
            .await;
    let Ok(Some((content_type, bytes))) = row else {
        return StatusCode::NOT_FOUND.into_response();
    };
    image_response(&content_type, bytes)
}

/// An avatar's type, bytes and, when animated, its still first frame.
type StillRow = (String, Vec<u8>, Option<Vec<u8>>);

/// GET /avatars/{hash}/still: an animated avatar's first frame; a still avatar is its own still (decision 6).
pub async fn serve_still(State(state): State<Arc<AppState>>, Path(hash): Path<String>) -> Response {
    if !valid_hash(&hash) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let row: Result<Option<StillRow>, _> =
        sqlx::query_as("SELECT content_type, bytes, still FROM avatars WHERE hash = $1 LIMIT 1")
            .bind(&hash)
            .fetch_optional(&state.db)
            .await;
    match row {
        Ok(Some((_, _, Some(still)))) => image_response("image/png", still),
        Ok(Some((content_type, bytes, None))) => image_response(&content_type, bytes),
        _ => StatusCode::NOT_FOUND.into_response(),
    }
}

fn image_response(content_type: &str, bytes: Vec<u8>) -> Response {
    let mut response = bytes.into_response();
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(content_type) {
        headers.insert(header::CONTENT_TYPE, value);
    }
    headers.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("public, max-age=31536000, immutable"),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; sandbox"),
    );
    headers.insert(
        "cross-origin-resource-policy",
        HeaderValue::from_static("cross-origin"),
    );
    response
}

#[derive(Deserialize)]
pub struct ReportRequest {
    pub hash: String,
    pub reason: String,
}

/// POST /api/v1/avatars/report: once per reporter and avatar.
pub async fn report_avatar(
    State(state): State<Arc<AppState>>,
    Extension(reporter): Extension<Uuid>,
    Json(req): Json<ReportRequest>,
) -> AppResult<Json<Value>> {
    let hash = req.hash.to_ascii_lowercase();
    if !valid_hash(&hash) || !REASONS.contains(&req.reason.as_str()) {
        return Err(AppError::ValidationError(
            "Choose an avatar and a reason".into(),
        ));
    }
    let exists: bool = sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM avatars WHERE hash = $1)")
        .bind(&hash)
        .fetch_one(&state.db)
        .await?;
    if !exists {
        return Err(AppError::NotFound(
            "That avatar is not shown any more".into(),
        ));
    }
    sqlx::query("INSERT INTO avatar_reports (hash, reporter, reason) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
        .bind(&hash)
        .bind(reporter)
        .bind(&req.reason)
        .execute(&state.db)
        .await?;
    Ok(Json(json!({ "reported": true })))
}

/// GET /api/v1/admin/avatar-reports: reported avatars still shown, most reported first.
pub async fn list_reports(State(state): State<Arc<AppState>>) -> AppResult<Json<Value>> {
    let rows: Vec<(String, i64, Vec<String>, Option<String>)> = sqlx::query_as(
        r#"
        SELECT r.hash, COUNT(*)::BIGINT, ARRAY_AGG(DISTINCT r.reason),
               (SELECT STRING_AGG(LOWER(u.username), ', ') FROM avatars a JOIN users u ON u.id = a.user_id WHERE a.hash = r.hash)
        FROM avatar_reports r
        WHERE EXISTS (SELECT 1 FROM avatars a WHERE a.hash = r.hash)
        GROUP BY r.hash
        ORDER BY COUNT(*) DESC, MAX(r.created_at) DESC
        LIMIT 200
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(json!({
        "reports": rows.into_iter().map(|(hash, count, reasons, accounts)| json!({
            "avatar_url": format!("/avatars/{hash}"), "hash": hash, "reports": count, "reasons": reasons,
            "accounts": accounts,
        })).collect::<Vec<_>>()
    })))
}

/// POST /api/v1/admin/avatars/{hash}/remove: every account showing it falls back to its letter and is told.
pub async fn remove_avatar(
    State(state): State<Arc<AppState>>,
    Extension(admin_id): Extension<Uuid>,
    Path(hash): Path<String>,
) -> AppResult<Json<Value>> {
    if !valid_hash(&hash) {
        return Err(AppError::NotFound("No such avatar".into()));
    }
    let mut tx = state.db.begin().await?;
    let owners: Vec<Uuid> =
        sqlx::query_scalar("DELETE FROM avatars WHERE hash = $1 RETURNING user_id")
            .bind(&hash)
            .fetch_all(&mut *tx)
            .await?;
    for owner in &owners {
        sqlx::query("UPDATE users SET avatar_url = NULL, updated_at = NOW() WHERE id = $1")
            .bind(owner)
            .execute(&mut *tx)
            .await?;
        sqlx::query("INSERT INTO avatar_removals (user_id, hash, removed_by) VALUES ($1, $2, $3)")
            .bind(owner)
            .bind(&hash)
            .bind(admin_id)
            .execute(&mut *tx)
            .await?;
    }
    sqlx::query(
        "INSERT INTO audit_log (user_id, action, details) VALUES ($1, 'avatar_removed', $2)",
    )
    .bind(admin_id)
    .bind(json!({ "hash": hash, "accounts": owners.len() }))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(Json(json!({ "removed": owners.len() })))
}

/// Whether the account has an avatar removal it has not yet answered with a new one.
pub async fn removal_notice(db: &sqlx::PgPool, user_id: Uuid) -> AppResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM avatar_removals WHERE user_id = $1 AND NOT seen)",
    )
    .bind(user_id)
    .fetch_one(db)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::services::{EmailConfig, EmailService};
    use axum::body::to_bytes;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use sqlx::PgPool;
    use std::io::Cursor;

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

    async fn account(db: &PgPool, name: &str) -> Uuid {
        sqlx::query_scalar("INSERT INTO users (username, password_hash, email_verified) VALUES ($1, 'x', TRUE) RETURNING id")
            .bind(name)
            .fetch_one(db)
            .await
            .expect("account")
    }

    fn png(colour: u8) -> Bytes {
        let image = RgbaImage::from_pixel(300, 200, Rgba([colour, 40, 200, 255]));
        let mut bytes = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .unwrap();
        Bytes::from(bytes)
    }

    async fn upload(state: &Arc<AppState>, user: Uuid, body: Bytes) -> AppResult<String> {
        let Json(answer) = upload_avatar(State(state.clone()), Extension(user), body).await?;
        Ok(answer["avatar_url"]
            .as_str()
            .expect("url")
            .trim_start_matches("/avatars/")
            .to_string())
    }

    async fn avatar_url(db: &PgPool, user: Uuid) -> Option<String> {
        sqlx::query_scalar("SELECT avatar_url FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(db)
            .await
            .unwrap()
    }

    #[sqlx::test]
    async fn an_upload_is_cleaned_stored_and_served_for_good(db: PgPool) {
        let state = state(db.clone());
        let user = account(&db, "pictured").await;
        let hash = upload(&state, user, png(10)).await.expect("uploaded");
        assert_eq!(
            avatar_url(&db, user).await.as_deref(),
            Some(format!("/avatars/{hash}").as_str())
        );

        let response = serve_avatar(State(state.clone()), Path(hash.clone())).await;
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers().clone();
        assert_eq!(headers[header::CONTENT_TYPE], "image/png");
        assert!(
            headers[header::CACHE_CONTROL]
                .to_str()
                .unwrap()
                .contains("immutable")
        );
        assert_eq!(headers[header::X_CONTENT_TYPE_OPTIONS], "nosniff");
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        assert_eq!(
            hex::encode(Sha256::digest(&bytes)),
            hash,
            "named by what is stored"
        );
        let stored = image::load_from_memory(&bytes).unwrap();
        assert_eq!((stored.width(), stored.height()), (256, 256));

        assert_eq!(
            serve_avatar(State(state.clone()), Path("0".repeat(64)))
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            serve_avatar(State(state), Path("../etc/passwd".into()))
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
    }

    #[sqlx::test]
    async fn a_new_upload_replaces_the_old_and_deleting_falls_back_to_the_letter(db: PgPool) {
        let state = state(db.clone());
        let user = account(&db, "changer").await;
        let first = upload(&state, user, png(10)).await.unwrap();
        let second = upload(&state, user, png(200)).await.unwrap();
        assert_ne!(first, second);
        assert_eq!(
            serve_avatar(State(state.clone()), Path(first))
                .await
                .status(),
            StatusCode::NOT_FOUND,
            "the old one is gone"
        );
        let _ = delete_avatar(State(state.clone()), Extension(user))
            .await
            .unwrap();
        assert_eq!(avatar_url(&db, user).await, None);
        assert_eq!(
            serve_avatar(State(state.clone()), Path(second))
                .await
                .status(),
            StatusCode::NOT_FOUND
        );
        assert!(matches!(
            upload(&state, user, Bytes::from_static(b"not an image")).await,
            Err(AppError::ValidationError(_))
        ));
    }

    #[sqlx::test]
    async fn a_report_needs_a_shown_avatar_and_a_reason_and_counts_once(db: PgPool) {
        let state = state(db.clone());
        let owner = account(&db, "owner").await;
        let reporter = account(&db, "reporter").await;
        let hash = upload(&state, owner, png(10)).await.unwrap();
        let report = |hash: &str, reason: &str| {
            report_avatar(
                State(state.clone()),
                Extension(reporter),
                Json(ReportRequest {
                    hash: hash.into(),
                    reason: reason.into(),
                }),
            )
        };
        assert!(matches!(
            report(&hash, "because").await,
            Err(AppError::ValidationError(_))
        ));
        assert!(matches!(
            report(&"0".repeat(64), "spam").await,
            Err(AppError::NotFound(_))
        ));
        let _ = report(&hash, "spam").await.unwrap();
        let _ = report(&hash, "spam").await.unwrap();
        let Json(queue) = list_reports(State(state)).await.unwrap();
        assert_eq!(queue["reports"][0]["reports"], 1, "once per reporter");
        assert_eq!(queue["reports"][0]["accounts"], "owner");
    }

    #[sqlx::test]
    async fn the_owner_removes_an_avatar_from_every_account_showing_it_and_they_are_told(
        db: PgPool,
    ) {
        let state = state(db.clone());
        let admin = account(&db, "admin").await;
        let a = account(&db, "same_a").await;
        let b = account(&db, "same_b").await;
        let hash = upload(&state, a, png(77)).await.unwrap();
        assert_eq!(
            upload(&state, b, png(77)).await.unwrap(),
            hash,
            "the same picture is the same avatar"
        );
        let Json(done) = remove_avatar(State(state.clone()), Extension(admin), Path(hash.clone()))
            .await
            .unwrap();
        assert_eq!(done["removed"], 2);
        for user in [a, b] {
            assert_eq!(avatar_url(&db, user).await, None);
            assert!(removal_notice(&db, user).await.unwrap(), "told");
        }
        upload(&state, a, png(5)).await.unwrap();
        assert!(
            !removal_notice(&db, a).await.unwrap(),
            "a new avatar answers the notice"
        );
        let audited: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'avatar_removed'")
                .fetch_one(&db)
                .await
                .unwrap();
        assert_eq!(audited, 1);
    }
}
