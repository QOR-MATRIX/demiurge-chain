//! Music player API handlers. **Dead code: this file is not compiled and none of its routes exist**
//! (`pub mod music` is commented out in `handlers/mod.rs`, which says why).

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::error::AppError;
use crate::middleware::auth::Claims;
use crate::state::AppState;

// ============================================================================
// Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct MusicTrack {
    pub id: Uuid,
    pub uploader_id: Uuid,
    pub uploader_qor_id: Option<String>,
    pub title: String,
    pub artist: Option<String>,
    pub file_url: String,
    pub duration_ms: Option<i32>,
    pub genre: Option<String>,
    pub plays: i32,
    pub likes: i32,
    pub is_public: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Playlist {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub owner_id: Option<Uuid>,
    pub owner_qor_id: Option<String>,
    pub is_global: bool,
    pub is_public: bool,
    pub cover_url: Option<String>,
    pub track_count: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UploadTrackRequest {
    pub title: String,
    pub artist: Option<String>,
    pub file_url: String,
    pub duration_ms: Option<i32>,
    pub genre: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePlaylistRequest {
    pub name: String,
    pub description: Option<String>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct AddToPlaylistRequest {
    pub track_id: Uuid,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ListTracksQuery {
    pub genre: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ============================================================================
// Track Handlers
// ============================================================================

/// Upload a new music track
pub async fn upload_track(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<UploadTrackRequest>,
) -> Result<Json<MusicTrack>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::unauthorized("Invalid user ID"))?;

    let track = sqlx::query_as!(
        MusicTrack,
        r#"
        INSERT INTO music_tracks (uploader_id, title, artist, file_url, duration_ms, genre)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING 
            id, uploader_id, NULL as "uploader_qor_id: _", 
            title, artist, file_url, duration_ms, genre,
            plays, likes, is_public, created_at
        "#,
        user_id,
        req.title,
        req.artist,
        req.file_url,
        req.duration_ms,
        req.genre,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to upload track: {}", e)))?;

    Ok(Json(track))
}

/// Get all public tracks
pub async fn list_tracks(
    State(state): State<AppState>,
    Query(query): Query<ListTracksQuery>,
) -> Result<Json<Vec<MusicTrack>>, AppError> {
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let tracks = if let Some(genre) = query.genre {
        sqlx::query_as!(
            MusicTrack,
            r#"
            SELECT 
                t.id, t.uploader_id, u.qor_id as "uploader_qor_id: _",
                t.title, t.artist, t.file_url, t.duration_ms, t.genre,
                t.plays, t.likes, t.is_public, t.created_at
            FROM music_tracks t
            LEFT JOIN users u ON t.uploader_id = u.id
            WHERE t.is_public = true AND t.genre = $1
            ORDER BY t.created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            genre,
            limit,
            offset,
        )
        .fetch_all(&state.db)
        .await
    } else {
        sqlx::query_as!(
            MusicTrack,
            r#"
            SELECT 
                t.id, t.uploader_id, u.qor_id as "uploader_qor_id: _",
                t.title, t.artist, t.file_url, t.duration_ms, t.genre,
                t.plays, t.likes, t.is_public, t.created_at
            FROM music_tracks t
            LEFT JOIN users u ON t.uploader_id = u.id
            WHERE t.is_public = true
            ORDER BY t.created_at DESC
            LIMIT $1 OFFSET $2
            "#,
            limit,
            offset,
        )
        .fetch_all(&state.db)
        .await
    }
    .map_err(|e| AppError::internal(format!("Failed to list tracks: {}", e)))?;

    Ok(Json(tracks))
}

/// Get a single track by ID
pub async fn get_track(
    State(state): State<AppState>,
    Path(track_id): Path<Uuid>,
) -> Result<Json<MusicTrack>, AppError> {
    let track = sqlx::query_as!(
        MusicTrack,
        r#"
        SELECT 
            t.id, t.uploader_id, u.qor_id as "uploader_qor_id: _",
            t.title, t.artist, t.file_url, t.duration_ms, t.genre,
            t.plays, t.likes, t.is_public, t.created_at
        FROM music_tracks t
        LEFT JOIN users u ON t.uploader_id = u.id
        WHERE t.id = $1
        "#,
        track_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to get track: {}", e)))?
    .ok_or_else(|| AppError::not_found("Track not found"))?;

    Ok(Json(track))
}

/// Increment play count
pub async fn record_play(
    State(state): State<AppState>,
    Path(track_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    sqlx::query!(
        "UPDATE music_tracks SET plays = plays + 1 WHERE id = $1",
        track_id,
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to record play: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// Like a track
pub async fn like_track(
    State(state): State<AppState>,
    claims: Claims,
    Path(track_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::unauthorized("Invalid user ID"))?;

    // Insert like (ignore if already exists)
    let result = sqlx::query!(
        r#"
        INSERT INTO track_likes (user_id, track_id)
        VALUES ($1, $2)
        ON CONFLICT (user_id, track_id) DO NOTHING
        "#,
        user_id,
        track_id,
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to like track: {}", e)))?;

    // Update like count if we inserted a new like
    if result.rows_affected() > 0 {
        sqlx::query!(
            "UPDATE music_tracks SET likes = likes + 1 WHERE id = $1",
            track_id,
        )
        .execute(&state.db)
        .await
        .map_err(|e| AppError::internal(format!("Failed to update like count: {}", e)))?;
    }

    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// Playlist Handlers
// ============================================================================

/// Get all public playlists
pub async fn list_playlists(
    State(state): State<AppState>,
) -> Result<Json<Vec<Playlist>>, AppError> {
    let playlists = sqlx::query_as!(
        Playlist,
        r#"
        SELECT 
            p.id, p.name, p.description, p.owner_id, u.qor_id as "owner_qor_id: _",
            p.is_global, p.is_public, p.cover_url,
            (SELECT COUNT(*) FROM playlist_tracks pt WHERE pt.playlist_id = p.id) as "track_count!",
            p.created_at
        FROM playlists p
        LEFT JOIN users u ON p.owner_id = u.id
        WHERE p.is_public = true
        ORDER BY p.is_global DESC, p.created_at DESC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to list playlists: {}", e)))?;

    Ok(Json(playlists))
}

/// Get the global playlist (Demiurge Radio)
pub async fn get_global_playlist(
    State(state): State<AppState>,
) -> Result<Json<Vec<MusicTrack>>, AppError> {
    let tracks = sqlx::query_as!(
        MusicTrack,
        r#"
        SELECT 
            t.id, t.uploader_id, u.qor_id as "uploader_qor_id: _",
            t.title, t.artist, t.file_url, t.duration_ms, t.genre,
            t.plays, t.likes, t.is_public, t.created_at
        FROM music_tracks t
        INNER JOIN playlist_tracks pt ON t.id = pt.track_id
        LEFT JOIN users u ON t.uploader_id = u.id
        WHERE pt.playlist_id = '00000000-0000-0000-0000-000000000001'
        ORDER BY pt.position ASC
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to get global playlist: {}", e)))?;

    Ok(Json(tracks))
}

/// Get playlist tracks
pub async fn get_playlist_tracks(
    State(state): State<AppState>,
    Path(playlist_id): Path<Uuid>,
) -> Result<Json<Vec<MusicTrack>>, AppError> {
    let tracks = sqlx::query_as!(
        MusicTrack,
        r#"
        SELECT 
            t.id, t.uploader_id, u.qor_id as "uploader_qor_id: _",
            t.title, t.artist, t.file_url, t.duration_ms, t.genre,
            t.plays, t.likes, t.is_public, t.created_at
        FROM music_tracks t
        INNER JOIN playlist_tracks pt ON t.id = pt.track_id
        LEFT JOIN users u ON t.uploader_id = u.id
        WHERE pt.playlist_id = $1
        ORDER BY pt.position ASC
        "#,
        playlist_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to get playlist tracks: {}", e)))?;

    Ok(Json(tracks))
}

/// Create a new playlist
pub async fn create_playlist(
    State(state): State<AppState>,
    claims: Claims,
    Json(req): Json<CreatePlaylistRequest>,
) -> Result<Json<Playlist>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::unauthorized("Invalid user ID"))?;

    let is_public = req.is_public.unwrap_or(true);

    let playlist = sqlx::query_as!(
        Playlist,
        r#"
        INSERT INTO playlists (name, description, owner_id, is_public)
        VALUES ($1, $2, $3, $4)
        RETURNING 
            id, name, description, owner_id, NULL as "owner_qor_id: _",
            is_global, is_public, cover_url, 0::bigint as "track_count!", created_at
        "#,
        req.name,
        req.description,
        user_id,
        is_public,
    )
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to create playlist: {}", e)))?;

    Ok(Json(playlist))
}

/// Add track to playlist
pub async fn add_to_playlist(
    State(state): State<AppState>,
    claims: Claims,
    Path(playlist_id): Path<Uuid>,
    Json(req): Json<AddToPlaylistRequest>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::unauthorized("Invalid user ID"))?;

    // Verify ownership (or global playlist for admins)
    let playlist = sqlx::query!(
        "SELECT owner_id, is_global FROM playlists WHERE id = $1",
        playlist_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to check playlist: {}", e)))?
    .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    // Only owner can add to non-global playlists
    // For global playlist, check if user is admin (simplified: any authenticated user can add)
    if !playlist.is_global && playlist.owner_id != Some(user_id) {
        return Err(AppError::forbidden("Not authorized to modify this playlist"));
    }

    // Get next position if not specified
    let position = if let Some(pos) = req.position {
        pos
    } else {
        let max_pos: Option<i32> = sqlx::query_scalar!(
            "SELECT MAX(position) FROM playlist_tracks WHERE playlist_id = $1",
            playlist_id,
        )
        .fetch_one(&state.db)
        .await
        .map_err(|e| AppError::internal(format!("Failed to get max position: {}", e)))?;
        
        max_pos.unwrap_or(0) + 1
    };

    sqlx::query!(
        r#"
        INSERT INTO playlist_tracks (playlist_id, track_id, position)
        VALUES ($1, $2, $3)
        ON CONFLICT (playlist_id, track_id) DO UPDATE SET position = $3
        "#,
        playlist_id,
        req.track_id,
        position,
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to add track to playlist: {}", e)))?;

    Ok(StatusCode::CREATED)
}

/// Remove track from playlist
pub async fn remove_from_playlist(
    State(state): State<AppState>,
    claims: Claims,
    Path((playlist_id, track_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::unauthorized("Invalid user ID"))?;

    // Verify ownership
    let playlist = sqlx::query!(
        "SELECT owner_id, is_global FROM playlists WHERE id = $1",
        playlist_id,
    )
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to check playlist: {}", e)))?
    .ok_or_else(|| AppError::not_found("Playlist not found"))?;

    if !playlist.is_global && playlist.owner_id != Some(user_id) {
        return Err(AppError::forbidden("Not authorized to modify this playlist"));
    }

    sqlx::query!(
        "DELETE FROM playlist_tracks WHERE playlist_id = $1 AND track_id = $2",
        playlist_id,
        track_id,
    )
    .execute(&state.db)
    .await
    .map_err(|e| AppError::internal(format!("Failed to remove track: {}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}
