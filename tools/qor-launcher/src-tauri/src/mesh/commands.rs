//! The Mesh's commands for the interface (ADR-095, ADR-097).
//!
//! No command here signs anything itself. Turning seeding on asks, in a host dialog, for the Mesh grant (ADR-097), under
//! which only the host's Mesh module signs; opening an escrow moves CGT, so it has its own dialog.

use super::paid::PaidStatus;
use super::{Fetched, Held, Settings, Status};
use crate::chain::mesh::EscrowTerms;
use crate::error::QorError;
use crate::{AppState, HostDialog};

#[tauri::command]
pub async fn mesh_status(state: tauri::State<'_, AppState>) -> Result<Status, QorError> {
    Ok(state.mesh()?.status().await)
}

/// What the Mesh holds, own published work first.
#[tauri::command]
pub async fn mesh_held(state: tauri::State<'_, AppState>) -> Result<Vec<Held>, QorError> {
    Ok(state.mesh()?.held().await)
}

/// Turn seeding on or off, and set its limits. On needs both limits. Turning it on asks once whether the Mesh may be
/// paid for its work (ADR-097); declining leaves seeding on, unpaid.
#[tauri::command]
pub async fn mesh_configure(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    settings: Settings,
) -> Result<Status, QorError> {
    let status = state.mesh()?.configure(settings).await?;
    if settings.seeding && state.mesh()?.grant().is_none() && state.vault.active_address().is_ok() {
        match state.mesh()?.allow_paid(&HostDialog(app)).await {
            Ok(_) | Err(QorError::Declined) => {}
            Err(e) => tracing::warn!(error = %e, "the Mesh could not be allowed to sign its work"),
        }
        return Ok(state.mesh()?.status().await);
    }
    Ok(status)
}

/// Ask again for the Mesh grant, for a person who declined it when they turned seeding on.
#[tauri::command]
pub async fn mesh_allow_paid(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<PaidStatus, QorError> {
    state.mesh()?.allow_paid(&HostDialog(app)).await
}

/// Fetch a file by its fingerprint alone, from whoever the chain says holds it.
#[tauri::command]
pub async fn mesh_fetch_root(
    state: tauri::State<'_, AppState>,
    root: String,
) -> Result<Fetched, QorError> {
    state.mesh()?.fetch_root(&root).await
}

/// Pay seeders to hold one's own published work, at one's own prices, after a host dialog. Returns the escrow's number.
#[tauri::command]
pub async fn mesh_open_escrow(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    terms: EscrowTerms,
) -> Result<u64, QorError> {
    let from = state.vault.active_address()?;
    state
        .chain
        .mesh_open_escrow(&state.vault, &HostDialog(app), &from, &terms)
        .await
}

/// A ticket for a root this launcher holds, for someone else to fetch it with.
#[tauri::command]
pub async fn mesh_ticket(
    state: tauri::State<'_, AppState>,
    root: String,
) -> Result<String, QorError> {
    state.mesh()?.ticket(&root).await
}

/// Fetch what a ticket names, verified against its fingerprint.
#[tauri::command]
pub async fn mesh_fetch(
    state: tauri::State<'_, AppState>,
    ticket: String,
) -> Result<Fetched, QorError> {
    state.mesh()?.fetch(&ticket).await
}
