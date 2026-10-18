//! QOR Media, docked inside the launcher (ADR-094 decision 1; docs/blueprints/library.md sections 1 and 4).
//!
//! QOR Media is a Qt 6 player, viewer and editor the launcher starts under the product host (ADR-046): its own private
//! channel and a one-time token, ended with the launcher. It keeps one request waiting, `media.next`, which the launcher
//! answers with its next command (open a file, sit at a rectangle of the launcher's window, hide), or with nothing after
//! [`WAIT`] so it asks again; what it has to tell the launcher (where playback stopped, a file's length and size) rides on
//! its next `media.next`. The protocol is unchanged: `media.next` is a method the launcher announces.
//!
//! It is found where `QOR_MEDIA` points, or in the launcher's own install folder, where the installer puts it (since
//! 0.1.10), or where `tools/qor-media/build.ps1 -Deploy` puts it, under the local app data folder. When it is in none of
//! them, a file opens in the program the system chooses (`library_open`).

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{Emitter as _, Manager as _};

use crate::error::{QorError, QorResult};
use crate::host::{self, Handler, Refusal};
use crate::library::Library;

/// How long `media.next` waits for a command before answering with nothing.
pub const WAIT: Duration = Duration::from_secs(25);

/// A rectangle of the launcher's window, in physical pixels from its client area's top left.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// What the launcher tells QOR Media to do.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Show this file, at this rectangle of the launcher's window, from where it stopped.
    Open {
        path: String,
        kind: String,
        position: Option<f64>,
        rect: Option<Rect>,
        parent: Option<i64>,
    },
    /// Sit here: the launcher moved, resized, or its content area changed.
    Place { rect: Rect, parent: Option<i64> },
    /// Get out of the way: the person left the Library.
    Hide,
    /// Wear the launcher's colours: its theme's tokens (`void`, `surface`, `accent`, `ink`, …) as `#rrggbb`.
    Theme {
        colors: std::collections::BTreeMap<String, String>,
    },
}

/// A theme's tokens, kept only when each is a colour written `#rrggbb` and named in letters and dashes.
pub fn theme(
    colors: std::collections::BTreeMap<String, String>,
) -> std::collections::BTreeMap<String, String> {
    colors
        .into_iter()
        .filter(|(name, value)| {
            name.len() <= 24
                && name.chars().all(|c| c.is_ascii_lowercase() || c == '-')
                && value.len() == 7
                && value.starts_with('#')
                && value[1..].chars().all(|c| c.is_ascii_hexdigit())
        })
        .take(32)
        .collect()
}

/// What QOR Media reports, on its next `media.next`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Report {
    /// Where playback of a file stopped.
    Position { path: String, position: f64 },
    /// A file's length and picture size, learned as it played.
    Learned {
        path: String,
        duration: Option<f64>,
        width: Option<i64>,
        height: Option<i64>,
    },
    /// The person closed what was open.
    Closed,
}

#[derive(Debug, Default, Deserialize)]
struct Next {
    #[serde(default)]
    reports: Vec<Report>,
}

/// Commands waiting for QOR Media, and the place it last sat.
pub struct Queue {
    commands: Mutex<VecDeque<Command>>,
    ready: tokio::sync::Notify,
    wait: Duration,
}

impl Queue {
    pub fn new(wait: Duration) -> Self {
        Self {
            commands: Mutex::new(VecDeque::new()),
            ready: tokio::sync::Notify::new(),
            wait,
        }
    }

    pub fn push(&self, command: Command) {
        let mut commands = self.commands.lock();
        // A newer place, or theme, makes an older one moot.
        if matches!(command, Command::Place { .. }) {
            commands.retain(|c| !matches!(c, Command::Place { .. }));
        }
        if matches!(command, Command::Theme { .. }) {
            commands.retain(|c| !matches!(c, Command::Theme { .. }));
        }
        commands.push_back(command);
        drop(commands);
        self.ready.notify_one();
    }

    /// The next command, as soon as there is one, or None after the wait.
    pub async fn next(&self) -> Option<Command> {
        let deadline = tokio::time::Instant::now() + self.wait;
        loop {
            if let Some(command) = self.commands.lock().pop_front() {
                return Some(command);
            }
            if tokio::time::timeout_at(deadline, self.ready.notified())
                .await
                .is_err()
            {
                return self.commands.lock().pop_front();
            }
        }
    }
}

/// What answers QOR Media's requests.
pub struct MediaHandler {
    pub queue: Arc<Queue>,
    pub library: Arc<Library>,
    /// Told when QOR Media reports something the Library shows.
    pub changed: Arc<dyn Fn() + Send + Sync>,
}

impl MediaHandler {
    fn apply(&self, report: Report) {
        // A report about a file the Library does not hold is passed over: QOR Media never adds to the index.
        let _ = match report {
            Report::Position { path, position } => self.library.opened(&path, Some(position)),
            Report::Learned {
                path,
                duration,
                width,
                height,
            } => self.library.learned(&path, duration, width, height),
            Report::Closed => Ok(()),
        };
    }
}

impl Handler for MediaHandler {
    fn capabilities(&self) -> &'static [&'static str] {
        &["media.next"]
    }

    fn quiet(&self, method: &str) -> bool {
        method == "media.next"
    }

    async fn handle(&self, method: &str, params: Value) -> Result<Value, Refusal> {
        debug_assert_eq!(method, "media.next");
        let next: Next = if params.is_null() {
            Next::default()
        } else {
            host::session::params(params)?
        };
        if !next.reports.is_empty() {
            for report in next.reports {
                self.apply(report);
            }
            (self.changed)();
        }
        Ok(match self.queue.next().await {
            Some(command) => json!({ "command": command }),
            None => json!({ "command": null }),
        })
    }
}

/// Where QOR Media is: the file `QOR_MEDIA` names, if it exists; else `qor-media\qor-media.exe` in `installed`, the
/// launcher's resource folder, where the installer puts it; else the same under the local app data folder, where its
/// build script deploys it.
pub fn find(
    named: Option<std::ffi::OsString>,
    installed: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
) -> Option<PathBuf> {
    if let Some(named) = named {
        let named = PathBuf::from(named);
        return named.is_file().then_some(named);
    }
    let program = if cfg!(windows) {
        "qor-media.exe"
    } else {
        "qor-media"
    };
    [installed, local_app_data]
        .into_iter()
        .flatten()
        .map(|dir| dir.join("qor-media").join(program))
        .find(|path| path.is_file())
}

/// QOR Media while the launcher runs: its queue, whether it is running, and the rectangle it sits in.
pub struct Media {
    queue: Arc<Queue>,
    running: Arc<Mutex<bool>>,
    rect: Mutex<Option<Rect>>,
    theme: Mutex<std::collections::BTreeMap<String, String>>,
}

impl Default for Media {
    fn default() -> Self {
        Self {
            queue: Arc::new(Queue::new(WAIT)),
            running: Arc::new(Mutex::new(false)),
            rect: Mutex::new(None),
            theme: Mutex::new(std::collections::BTreeMap::new()),
        }
    }
}

/// The launcher's window, as a native handle QOR Media's window becomes a child of (Windows).
fn parent(app: &tauri::AppHandle) -> Option<i64> {
    #[cfg(windows)]
    {
        app.get_webview_window("main")
            .and_then(|w| w.hwnd().ok())
            .map(|h| h.0 as i64)
    }
    #[cfg(not(windows))]
    {
        let _ = app;
        None
    }
}

/// Open a file the Library holds in QOR Media, starting it if it is not running. False when QOR Media is not here.
pub async fn open(app: &tauri::AppHandle, path: &str) -> QorResult<bool> {
    let state = app.state::<crate::AppState>();
    let Some(program) = find(
        std::env::var_os("QOR_MEDIA"),
        app.path().resource_dir().ok(),
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    ) else {
        return Ok(false);
    };
    let item = state.library()?.item(path)?;
    let media = &state.media;
    let start = !*media.running.lock();
    if start {
        let handler = Arc::new(MediaHandler {
            queue: Arc::clone(&media.queue),
            library: state.library()?,
            changed: {
                let told = app.clone();
                Arc::new(move || crate::library::commands::changed(&told))
            },
        });
        let running = Arc::clone(&media.running);
        let told = app.clone();
        let cwd = program
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(std::env::temp_dir);
        host::launch(
            host::Launch {
                product: host::ProductRecord {
                    id: "qor-media",
                    name: "QOR Media",
                    path: program,
                },
                args: vec![],
                cwd,
            },
            handler,
            move |exited| {
                *running.lock() = false;
                let _ = told.emit("media://exited", exited);
            },
        )
        .await
        .map_err(|e| QorError::Internal(format!("QOR Media did not start: {e}")))?;
        *media.running.lock() = true;
        let colors = media.theme.lock().clone();
        if !colors.is_empty() {
            media.queue.push(Command::Theme { colors });
        }
    }
    media.queue.push(Command::Open {
        path: item.path,
        kind: item.kind,
        position: item.position,
        rect: *media.rect.lock(),
        parent: parent(app),
    });
    Ok(true)
}

/// Where QOR Media sits: the Library's content area, measured by the interface, in physical pixels.
#[tauri::command]
pub fn media_place(app: tauri::AppHandle, state: tauri::State<'_, crate::AppState>, rect: Rect) {
    *state.media.rect.lock() = Some(rect);
    if *state.media.running.lock() {
        state.media.queue.push(Command::Place {
            rect,
            parent: parent(&app),
        });
    }
}

/// The launcher's colours, for QOR Media to wear: sent by the interface when the Library shows and when the theme
/// changes.
#[tauri::command]
pub fn media_theme(
    state: tauri::State<'_, crate::AppState>,
    colors: std::collections::BTreeMap<String, String>,
) {
    let colors = theme(colors);
    *state.media.theme.lock() = colors.clone();
    if *state.media.running.lock() {
        state.media.queue.push(Command::Theme { colors });
    }
}

/// Hide QOR Media: the person left the Library.
#[tauri::command]
pub fn media_hide(state: tauri::State<'_, crate::AppState>) {
    if *state.media.running.lock() {
        state.media.queue.push(Command::Hide);
    }
}

#[cfg(test)]
mod tests;
