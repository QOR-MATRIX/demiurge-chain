//! The Library's commands for the interface (ADR-094). Every one that names a file takes only a path the index holds.

use serde::Serialize;
use tauri::{Emitter as _, Manager as _};

use super::{Item, Scanned};
use crate::error::QorError;
use crate::AppState;

/// The folders in the Library, and those a person might add first.
#[derive(Serialize)]
pub struct Folders {
    pub folders: Vec<String>,
    pub suggested: Vec<String>,
}

/// Tell the interface the index changed, so it reads its shelves again.
pub fn changed(app: &tauri::AppHandle) {
    let _ = app.emit("library://changed", ());
}

#[tauri::command]
pub fn library_folders(state: tauri::State<'_, AppState>) -> Result<Folders, QorError> {
    let folders = state.library.folders()?;
    let suggested = super::Library::suggested()
        .into_iter()
        .filter(|s| !folders.contains(s))
        .collect();
    Ok(Folders { folders, suggested })
}

/// Add a folder, the one named or one the person picks, then index it in the background and follow it.
#[tauri::command]
pub async fn library_add_folder(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<String>, QorError> {
    let path = match path {
        Some(p) => Some(p),
        None => {
            use tauri_plugin_dialog::DialogExt as _;
            let picker = app.clone();
            tauri::async_runtime::spawn_blocking(move || {
                let mut dialog = picker
                    .dialog()
                    .file()
                    .set_title("Add a folder to the Library");
                if let Some(window) = picker.get_webview_window("main") {
                    dialog = dialog.set_parent(&window);
                }
                dialog.blocking_pick_folder()
            })
            .await
            .map_err(|e| QorError::Internal(format!("the folder dialog did not answer: {e}")))?
            .map(|p| p.to_string())
        }
    };
    let Some(path) = path else { return Ok(None) };
    let folder = state.library.add_folder(std::path::Path::new(&path))?;
    if let Some(watch) = state.library_watch.get() {
        watch.add(&folder);
    }
    let library = std::sync::Arc::clone(&state.library);
    let scanning = folder.clone();
    let told = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _ = library.scan(&scanning);
        changed(&told);
    });
    changed(&app);
    Ok(Some(folder))
}

#[tauri::command]
pub fn library_remove_folder(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<(), QorError> {
    if let Some(watch) = state.library_watch.get() {
        watch.remove(&path);
    }
    state.library.remove_folder(&path)?;
    changed(&app);
    Ok(())
}

/// Walk every folder again: what came, changed and went since the launcher last looked.
#[tauri::command]
pub async fn library_scan(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<Scanned, QorError> {
    let library = std::sync::Arc::clone(&state.library);
    let scanned = tauri::async_runtime::spawn_blocking(move || library.scan_all())
        .await
        .map_err(|e| QorError::Internal(format!("the scan stopped: {e}")))??;
    changed(&app);
    Ok(scanned)
}

#[tauri::command]
pub fn library_search(
    state: tauri::State<'_, AppState>,
    query: String,
    kind: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Item>, QorError> {
    state
        .library
        .search(&query, kind.as_deref(), limit.unwrap_or(60))
}

#[tauri::command]
pub fn library_shelf(
    state: tauri::State<'_, AppState>,
    kind: Option<String>,
    order: Option<String>,
    limit: Option<u32>,
) -> Result<Vec<Item>, QorError> {
    state.library.shelf(
        kind.as_deref(),
        order.as_deref().unwrap_or("added"),
        limit.unwrap_or(40),
    )
}

#[tauri::command]
pub fn library_counts(state: tauri::State<'_, AppState>) -> Result<Vec<(String, i64)>, QorError> {
    state.library.counts()
}

/// A picture's thumbnail, as raw JPEG bytes.
#[tauri::command]
pub async fn library_thumbnail(
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<tauri::ipc::Response, QorError> {
    let library = std::sync::Arc::clone(&state.library);
    let bytes = tauri::async_runtime::spawn_blocking(move || library.thumbnail(&path))
        .await
        .map_err(|e| QorError::Internal(format!("the thumbnail stopped: {e}")))??;
    Ok(tauri::ipc::Response::new(bytes))
}

/// Show a file the Library holds in the system's file manager.
#[tauri::command]
pub fn library_reveal(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
) -> Result<(), QorError> {
    use tauri_plugin_opener::OpenerExt as _;
    state.library.item(&path)?;
    app.opener()
        .reveal_item_in_dir(&path)
        .map_err(|e| QorError::Io(e.to_string()))
}

/// Note that a file was opened from the Library, and where playback stopped.
#[tauri::command]
pub fn library_opened(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    path: String,
    position: Option<f64>,
) -> Result<(), QorError> {
    state.library.opened(&path, position)?;
    changed(&app);
    Ok(())
}
