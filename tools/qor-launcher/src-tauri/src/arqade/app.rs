//! ARQADE's Qt app, docked inside the launcher (ADR-102 decision 1, DIRECTION P7.19; `products/arqade/qt/`).
//!
//! ARQADE is a Qt 6 application the launcher starts under the product host (ADR-046), as it starts QOR Media: its own
//! private channel and a one-time token, ended with the launcher. Its window becomes a child of the launcher's and sits
//! over the ARQADE screen. It keeps one request waiting, `arqade.next`, which the launcher answers with its next
//! [`Command`] (show at a rectangle of its window, hide, wear its colours, who is playing and their Sparks, the person's
//! preferences, quit), or with nothing after [`WAIT`].
//!
//! It asks two things besides:
//!
//! - **`arqade.signin`**: sign the person in to ARQADE's server with the launcher's QOR ID, as the ARQADE window does
//!   (`super`): the app hands over QOR ID's sign-in page as ARQADE's server sent it; the launcher checks it is QOR ID's
//!   sign-in page on the QOR ID it uses, asks QOR ID with its own token (`/api/v1/oauth/handoff`), checks the answer
//!   lands on ARQADE over https, and returns it. The app holds no token of the person's, only ARQADE's own session
//!   cookie, in memory. Neither address is logged.
//! - **`arqade.web`**: open ARQADE's website in the launcher's guarded window, where today's versions of the games play
//!   until their rebuilds arrive.
//!
//! The launcher answers one request at a time, so a sign-in waits for the `arqade.next` already waiting: [`WAIT`] is
//! short for that reason, where QOR Media's is 25 seconds.
//!
//! It is found where `ARQADE_APP` points, or in the launcher's install folder (`arqade\arqade.exe`), or where
//! `products/arqade/qt/build.ps1 -Deploy` puts it, under the local app data folder. When it is in none of them, the
//! ARQADE screen offers the website's window as before.

use std::collections::BTreeMap;
use std::future::Future;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{Emitter as _, Manager as _, Url};

use crate::error::{QorError, QorResult};
use crate::host::{self, protocol::code, Handler, Refusal};
use crate::media::{Queue, Rect, Supersedes};

/// How long `arqade.next` waits for a command before answering with nothing: short, so a sign-in the app asks for is
/// not held behind it for long.
pub const WAIT: Duration = Duration::from_secs(3);

/// The methods ARQADE may ask.
pub const CAPABILITIES: &[&str] = &["arqade.next", "arqade.signin", "arqade.web"];

/// Who is playing, as ARQADE shows it: written out by the launcher, so the app does no arithmetic on money.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Player {
    /// Whether the launcher has a QOR ID session. Off, ARQADE forgets its own session with ARQADE's server.
    pub signed_in: bool,
    pub name: Option<String>,
    pub level: Option<u32>,
    /// The wallet's balance in Sparks, as the launcher writes it ("37,000,000 Sparks"), or in CGT when finer.
    pub sparks: Option<String>,
    pub cgt: Option<String>,
}

impl Player {
    /// The same, with every text cut to a sensible length and without control characters.
    pub fn tidy(self) -> Self {
        let tidy = |text: Option<String>| {
            text.map(|t| {
                t.chars()
                    .filter(|c| !c.is_control())
                    .take(64)
                    .collect::<String>()
            })
            .filter(|t| !t.is_empty())
        };
        Self {
            signed_in: self.signed_in,
            name: tidy(self.name),
            level: self.level,
            sparks: tidy(self.sparks),
            cgt: tidy(self.cgt),
        }
    }
}

/// The person's preferences ARQADE follows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct Prefs {
    pub reduced_motion: bool,
}

/// What the launcher tells ARQADE to do.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    /// Show, sitting here: the ARQADE screen showed, or the launcher moved, resized, or its content area changed.
    Place {
        rect: Rect,
        parent: Option<i64>,
    },
    /// Get out of the way: the person left the ARQADE screen.
    Hide,
    /// Wear the launcher's colours (`crate::media::theme`).
    Theme {
        colors: BTreeMap<String, String>,
    },
    Player {
        player: Player,
    },
    Prefs {
        prefs: Prefs,
    },
    /// Close: the person signed out of the launcher, and what ARQADE knew of them goes with it.
    Quit,
}

impl Supersedes for Command {
    fn supersedes(&self, older: &Self) -> bool {
        matches!(
            (self, older),
            (Command::Place { .. }, Command::Place { .. })
                | (Command::Theme { .. }, Command::Theme { .. })
                | (Command::Player { .. }, Command::Player { .. })
                | (Command::Prefs { .. }, Command::Prefs { .. })
        )
    }
}

/// What ARQADE's requests need of the launcher: QOR ID's host, QOR ID's handoff, and the website's window.
pub trait Launcher: Send + Sync + 'static {
    /// The host of the QOR ID the launcher is set to.
    fn qor_id_host(&self) -> Option<String>;
    /// QOR ID's answer to a sign-in page, asked with the launcher's own session.
    fn handoff(&self, authorize_url: String) -> impl Future<Output = QorResult<String>> + Send;
    /// Open ARQADE's website in the launcher's guarded window.
    fn open_web(&self) -> Result<(), String>;
}

impl Launcher for tauri::AppHandle {
    fn qor_id_host(&self) -> Option<String> {
        self.try_state::<crate::AppState>()
            .and_then(|state| super::host_of(&state.identity.endpoint()))
    }

    async fn handoff(&self, authorize_url: String) -> QorResult<String> {
        match self.try_state::<crate::AppState>() {
            Some(state) => state.identity.handoff(&authorize_url).await,
            None => Err(QorError::NotAuthenticated),
        }
    }

    fn open_web(&self) -> Result<(), String> {
        let endpoint = self
            .try_state::<crate::AppState>()
            .map(|state| state.identity.endpoint())
            .unwrap_or_default();
        super::open(self, &endpoint)
    }
}

#[derive(Deserialize)]
struct SignIn {
    authorize_url: String,
}

/// What answers ARQADE's requests.
pub struct ArqadeHandler<L: Launcher> {
    pub queue: Arc<Queue<Command>>,
    pub launcher: L,
}

impl<L: Launcher> ArqadeHandler<L> {
    async fn sign_in(&self, params: Value) -> Result<Value, Refusal> {
        let SignIn { authorize_url } = host::session::params(params)?;
        let page = Url::parse(&authorize_url)
            .map_err(|_| Refusal::new(code::BAD_REQUEST, "That is not an address."))?;
        let qor_id = self.launcher.qor_id_host().ok_or_else(|| {
            Refusal::new(
                code::NOT_AUTHORISED,
                "The launcher has no QOR ID set to sign in with.",
            )
        })?;
        if !super::is_sign_in(&page, &qor_id) {
            return Err(Refusal::new(
                code::NOT_AUTHORISED,
                "ARQADE may ask the launcher only for QOR ID's own sign-in page.",
            ));
        }
        let redirect =
            match self.launcher.handoff(page.to_string()).await {
                Ok(redirect) => redirect,
                Err(QorError::NotAuthenticated) => return Err(Refusal::new(
                    "not_authenticated",
                    "Sign in to the launcher with your QOR ID, and ARQADE signs you in with it.",
                )),
                Err(e) => {
                    tracing::info!("ARQADE's sign-in was not completed by QOR ID: {e}");
                    return Err(Refusal::new(
                        "handoff",
                        "QOR ID did not complete ARQADE's sign-in. Try again shortly.",
                    ));
                }
            };
        match Url::parse(&redirect) {
            Ok(redirect) if super::lands_on_arqade(&redirect) => {
                Ok(json!({ "redirect": redirect.as_str() }))
            }
            _ => Err(Refusal::new(
                code::NOT_AUTHORISED,
                "QOR ID's answer did not lead to ARQADE, so it was not passed on.",
            )),
        }
    }
}

impl<L: Launcher> Handler for ArqadeHandler<L> {
    fn capabilities(&self) -> &'static [&'static str] {
        CAPABILITIES
    }

    fn quiet(&self, method: &str) -> bool {
        method == "arqade.next"
    }

    async fn handle(&self, method: &str, params: Value) -> Result<Value, Refusal> {
        match method {
            "arqade.next" => Ok(json!({ "command": self.queue.next().await })),
            "arqade.signin" => self.sign_in(params).await,
            "arqade.web" => self
                .launcher
                .open_web()
                .map(|()| json!({}))
                .map_err(|e| Refusal::new("web", e)),
            _ => Err(Refusal::new(
                code::UNSUPPORTED_METHOD,
                format!("this launcher does not offer {method:?}"),
            )),
        }
    }
}

/// Where ARQADE is: the file `ARQADE_APP` names; else `arqade\arqade.exe` in the launcher's resource folder; else the
/// same under the local app data folder.
pub fn find(
    named: Option<std::ffi::OsString>,
    installed: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
) -> Option<PathBuf> {
    host::find_product(named, installed, local_app_data, "arqade")
}

/// ARQADE while the launcher runs: its queue, whether it is running, and what it is sent again when it starts.
pub struct App {
    queue: Arc<Queue<Command>>,
    running: Arc<Mutex<bool>>,
    /// Held while ARQADE is being started, so two quick visits to the screen start it once.
    starting: tokio::sync::Mutex<()>,
    rect: Mutex<Option<Rect>>,
    theme: Mutex<BTreeMap<String, String>>,
    player: Mutex<Option<Player>>,
    prefs: Mutex<Option<Prefs>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            queue: Arc::new(Queue::new(WAIT)),
            running: Arc::new(Mutex::new(false)),
            starting: tokio::sync::Mutex::new(()),
            rect: Mutex::new(None),
            theme: Mutex::new(BTreeMap::new()),
            player: Mutex::new(None),
            prefs: Mutex::new(None),
        }
    }
}

impl App {
    fn running(&self) -> bool {
        *self.running.lock()
    }

    /// Send `command` if ARQADE is running.
    fn tell(&self, command: Command) {
        if self.running() {
            self.queue.push(command);
        }
    }

    /// Start ARQADE from `program` if it is not running, answering its requests with `launcher`, and show it where the
    /// ARQADE screen is. `exited` is told when it ends, however it ends.
    pub async fn open<L: Launcher>(
        &self,
        program: PathBuf,
        launcher: L,
        parent: Option<i64>,
        exited: impl FnOnce(host::Exited) + Send + 'static,
    ) -> QorResult<()> {
        let _starting = self.starting.lock().await;
        if !self.running() {
            // What was queued for a run that has ended (a quit, a hide) is not the new run's to obey.
            self.queue.clear();
            // Marked running before it starts, so an exit at once (which marks it stopped) is not overwritten.
            *self.running.lock() = true;
            let handler = Arc::new(ArqadeHandler {
                queue: Arc::clone(&self.queue),
                launcher,
            });
            let running = Arc::clone(&self.running);
            let cwd = program
                .parent()
                .map(PathBuf::from)
                .unwrap_or_else(std::env::temp_dir);
            host::launch(
                host::Launch {
                    product: host::ProductRecord {
                        id: "arqade",
                        name: "ARQADE",
                        path: program,
                    },
                    args: vec![],
                    cwd,
                },
                handler,
                move |e| {
                    *running.lock() = false;
                    exited(e);
                },
            )
            .await
            .map_err(|e| {
                *self.running.lock() = false;
                QorError::Internal(format!("ARQADE did not start: {e}"))
            })?;
            // What it would have been told had it been running: its colours, who is playing, the person's preferences.
            let colors = self.theme.lock().clone();
            if !colors.is_empty() {
                self.queue.push(Command::Theme { colors });
            }
            if let Some(player) = self.player.lock().clone() {
                self.queue.push(Command::Player { player });
            }
            if let Some(prefs) = *self.prefs.lock() {
                self.queue.push(Command::Prefs { prefs });
            }
        }
        // Read after it is marked running, so a place the screen sent meanwhile is not lost.
        if let Some(rect) = *self.rect.lock() {
            self.queue.push(Command::Place { rect, parent });
        }
        Ok(())
    }

    /// The person signed out of the launcher: ARQADE closes, and nothing of them is kept to send it again.
    pub fn quit(&self) {
        *self.player.lock() = None;
        self.tell(Command::Quit);
    }
}

/// Start ARQADE if it is not running, and show it where the ARQADE screen is. False when ARQADE is not here.
#[tauri::command]
pub async fn arqade_app_open(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, QorError> {
    let Some(program) = find(
        std::env::var_os("ARQADE_APP"),
        app.path().resource_dir().ok(),
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from),
    ) else {
        return Ok(false);
    };
    let told = app.clone();
    state
        .arqade_app
        .open(
            program,
            app.clone(),
            crate::media::parent(&app),
            move |exited| {
                let _ = told.emit("arqade://exited", exited);
            },
        )
        .await?;
    Ok(true)
}

/// Where ARQADE sits: the ARQADE screen's content area, measured by the interface, in physical pixels.
#[tauri::command]
pub fn arqade_app_place(
    app: tauri::AppHandle,
    state: tauri::State<'_, crate::AppState>,
    rect: Rect,
) {
    *state.arqade_app.rect.lock() = Some(rect);
    state.arqade_app.tell(Command::Place {
        rect,
        parent: crate::media::parent(&app),
    });
}

/// Hide ARQADE: the person left the ARQADE screen.
#[tauri::command]
pub fn arqade_app_hide(state: tauri::State<'_, crate::AppState>) {
    *state.arqade_app.rect.lock() = None;
    state.arqade_app.tell(Command::Hide);
}

/// The launcher's colours, for ARQADE to wear.
#[tauri::command]
pub fn arqade_app_theme(
    state: tauri::State<'_, crate::AppState>,
    colors: BTreeMap<String, String>,
) {
    let colors = crate::media::theme(colors);
    *state.arqade_app.theme.lock() = colors.clone();
    state.arqade_app.tell(Command::Theme { colors });
}

/// Who is playing, as the ARQADE screen shows them.
#[tauri::command]
pub fn arqade_app_player(state: tauri::State<'_, crate::AppState>, player: Player) {
    let player = player.tidy();
    *state.arqade_app.player.lock() = Some(player.clone());
    state.arqade_app.tell(Command::Player { player });
}

/// The person's preferences ARQADE follows.
#[tauri::command]
pub fn arqade_app_prefs(state: tauri::State<'_, crate::AppState>, prefs: Prefs) {
    *state.arqade_app.prefs.lock() = Some(prefs);
    state.arqade_app.tell(Command::Prefs { prefs });
}

#[cfg(test)]
mod tests;
