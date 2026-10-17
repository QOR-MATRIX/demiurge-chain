//! The Mesh's commands for the interface (ADR-095, L10.1). No command here moves CGT or touches the vault.

use super::{Fetched, Held, Settings, Status};
use crate::error::QorError;
use crate::AppState;

#[tauri::command]
pub async fn mesh_status(state: tauri::State<'_, AppState>) -> Result<Status, QorError> {
    Ok(state.mesh.status().await)
}

/// What the Mesh holds, own published work first.
#[tauri::command]
pub async fn mesh_held(state: tauri::State<'_, AppState>) -> Result<Vec<Held>, QorError> {
    Ok(state.mesh.held().await)
}

/// Turn seeding on or off, and set its limits. On needs both limits.
#[tauri::command]
pub async fn mesh_configure(
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> Result<Status, QorError> {
    state.mesh.configure(settings).await
}

/// A ticket for a root this launcher holds, for someone else to fetch it with.
#[tauri::command]
pub async fn mesh_ticket(
    state: tauri::State<'_, AppState>,
    root: String,
) -> Result<String, QorError> {
    state.mesh.ticket(&root).await
}

/// Fetch what a ticket names, verified against its fingerprint.
#[tauri::command]
pub async fn mesh_fetch(
    state: tauri::State<'_, AppState>,
    ticket: String,
) -> Result<Fetched, QorError> {
    state.mesh.fetch(&ticket).await
}
