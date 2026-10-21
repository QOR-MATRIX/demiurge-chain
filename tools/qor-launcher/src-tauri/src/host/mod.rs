//! The product host (ADR-046, built for its first product by ADR-088): the launcher starts a product (QQ Studio) as a
//! process of its own and answers what it asks over a channel only that process can use. The product never holds a
//! key and never hands the launcher bytes to sign: it asks for a typed intent, and the launcher builds the
//! transaction, shows it in its own dialog naming the product, and signs it.
//!
//! - [`launch`] opens a channel only this user can open ([`channel`]), starts the product with the channel's name as an
//!   argument and a one-time token on its stdin (never its arguments or environment), puts it in a job that ends it
//!   with the launcher (Windows), and supervises it: when it exits its exit code and the last of its error output are
//!   handed to `on_exit`.
//! - [`channel`] accepts one connection, only from the process the launcher started (its id and start time, as the
//!   operating system reports them, and its user).
//! - [`session`] checks the token once, in constant time, within [`session::TOKEN_LIFETIME`]; agrees a protocol
//!   version; then answers requests through a [`Handler`] until the product closes the channel.
//! - [`ProductDialog`] puts the product's name, from the launcher's own record, at the top of every dialog its
//!   requests raise (ADR-046 decision 8).
//!
//! The token and the session id are never logged (ADR-046 decision 10): the tests run a session with every log level
//! on and look for both.

pub mod channel;
pub mod frame;
pub mod protocol;
pub mod session;
pub mod studio;

#[cfg(test)]
mod tests;

use std::ffi::OsString;
use std::future::Future;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Instant;

use rand::RngCore;
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

use crate::error::QorError;
use crate::{Answer, Confirm, Prompt};

/// A product the launcher starts, as the launcher's own records describe it: never as the product describes itself.
#[derive(Debug, Clone)]
pub struct ProductRecord {
    /// Its id in the protocol's `hello` (`qq-studio`).
    pub id: &'static str,
    /// The name a person reads in a dialog (`QQ Studio`).
    pub name: &'static str,
    /// The program the launcher starts.
    pub path: PathBuf,
}

/// Where a product the launcher carries is: the file `named` (its environment variable) points to, if it exists; else
/// `<id>\<id>.exe` in `installed`, the launcher's resource folder, where the installer puts it; else the same under the
/// local app data folder, where its build script deploys it.
pub fn find_product(
    named: Option<OsString>,
    installed: Option<PathBuf>,
    local_app_data: Option<PathBuf>,
    id: &str,
) -> Option<PathBuf> {
    if let Some(named) = named {
        let named = PathBuf::from(named);
        return named.is_file().then_some(named);
    }
    let program = if cfg!(windows) {
        format!("{id}.exe")
    } else {
        id.to_owned()
    };
    [installed, local_app_data]
        .into_iter()
        .flatten()
        .map(|dir| dir.join(id).join(&program))
        .find(|path| path.is_file())
}

/// Random bytes as lowercase hex.
pub(crate) fn random_hex(bytes: usize) -> String {
    let mut buffer = Zeroizing::new(vec![0u8; bytes]);
    rand::rngs::OsRng.fill_bytes(&mut buffer);
    hex::encode(&*buffer)
}

/// The one-time token a spawn gives its product (ADR-046 decision 5): 32 random bytes, sent as hex on the product's
/// stdin. It prints as `SpawnToken(…)`, so a careless log line shows nothing.
pub struct SpawnToken(Zeroizing<String>);

impl SpawnToken {
    pub fn new() -> Self {
        Self(Zeroizing::new(random_hex(32)))
    }

    pub(crate) fn hex(&self) -> &str {
        &self.0
    }
}

impl Default for SpawnToken {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SpawnToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SpawnToken(…)")
    }
}

/// A request refused, as the product reads it: a stable code and a sentence for the person.
#[derive(Debug)]
pub struct Refusal {
    pub code: String,
    pub message: String,
}

impl Refusal {
    pub fn new(code: &str, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<QorError> for Refusal {
    fn from(error: QorError) -> Self {
        Self::new(error.kind(), error.to_string())
    }
}

/// What answers a product's requests: the methods it offers (announced as capabilities) and how each is done.
pub trait Handler: Send + Sync + 'static {
    fn capabilities(&self) -> &'static [&'static str];
    /// A method asked so often its answers are logged at debug, not info: QOR Media's `media.next`, which repeats every
    /// [`crate::media::WAIT`]. A refusal is still logged at info.
    fn quiet(&self, _method: &str) -> bool {
        false
    }
    fn handle(
        &self,
        method: &str,
        params: Value,
    ) -> impl Future<Output = Result<Value, Refusal>> + Send;
}

/// Why a session or a spawn ended badly. For the launcher's log: it never carries the token or the session id.
#[derive(Debug, thiserror::Error)]
pub enum HostError {
    #[error("the channel failed: {0}")]
    Io(std::io::Error),
    #[error("the product did not connect, or say hello, in time")]
    Expired,
    #[error("the product closed the channel before saying hello")]
    Closed,
    #[error("a connection that was not the product the launcher started: {0}")]
    NotSpawned(String),
    #[error("the token was wrong, or used too late")]
    BadToken,
    #[error("the handshake was refused: {0}")]
    BadHandshake(String),
    #[error("internal error: {0}")]
    Internal(String),
}

/// The launcher's dialog for a product's request: the product's name and where it was started from, from the
/// launcher's record, at the top of the prompt the request would have shown anyway (ADR-046 decision 8).
pub struct ProductDialog<'a> {
    pub inner: &'a dyn Confirm,
    pub product: &'a ProductRecord,
}

impl Confirm for ProductDialog<'_> {
    fn confirm<'a>(&'a self, prompt: &'a Prompt) -> Answer<'a> {
        let asked = Prompt {
            title: format!("{} asks: {}", self.product.name, prompt.title),
            body: format!(
                "Asked by {name}, which the launcher started from\n{path}\n\n{body}",
                name = self.product.name,
                path = self.product.path.display(),
                body = prompt.body,
            ),
            approve: prompt.approve.clone(),
        };
        let inner = self.inner;
        Box::pin(async move { inner.confirm(&asked).await })
    }
}

/// The person's own Motion setting, as the launcher's interface last said it, passed on to every product the launcher
/// starts as `QOR_REDUCED_MOTION`: `1` for less motion, `0` for all of it (ADR-108 decision 5). Until the interface has
/// said, nothing is passed and a product follows the system's own setting. A product already running keeps what it was
/// started with; ARQADE's app hears changes over its channel as well (`arqade_app_prefs`).
static REDUCED_MOTION: std::sync::atomic::AtomicU8 =
    std::sync::atomic::AtomicU8::new(MOTION_UNSAID);
const MOTION_UNSAID: u8 = 0;
const MOTION_LESS: u8 = 1;
const MOTION_FULL: u8 = 2;

/// Remember the person's Motion setting for the products started from now on.
pub fn set_reduced_motion(reduced: bool) {
    REDUCED_MOTION.store(
        if reduced { MOTION_LESS } else { MOTION_FULL },
        std::sync::atomic::Ordering::Relaxed,
    );
}

/// The value `QOR_REDUCED_MOTION` takes for a product started now, if the interface has said.
fn reduced_motion_word() -> Option<&'static str> {
    match REDUCED_MOTION.load(std::sync::atomic::Ordering::Relaxed) {
        MOTION_LESS => Some("1"),
        MOTION_FULL => Some("0"),
        _ => None,
    }
}

/// The person's Motion setting, from the launcher's interface: less motion, or all of it.
#[tauri::command]
pub fn launcher_motion(reduced: bool) {
    set_reduced_motion(reduced);
}

/// A product to start.
pub struct Launch {
    pub product: ProductRecord,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
}

/// How a product ended: its exit code, if it had one, and the last of its error output, for the person to read.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Exited {
    pub product: &'static str,
    pub code: Option<i32>,
    pub stderr_tail: String,
}

/// How much of a product's error output is kept for [`Exited`].
const STDERR_TAIL: usize = 4096;

/// The job every product is put in, so each ends when the launcher does, however the launcher ends (ADR-046
/// decision 2). One for the launcher's lifetime: closing it is what ends them.
#[cfg(windows)]
fn job() -> std::io::Result<&'static qor_host_proof::KillOnCloseJob> {
    static JOB: std::sync::OnceLock<qor_host_proof::KillOnCloseJob> = std::sync::OnceLock::new();
    if let Some(job) = JOB.get() {
        return Ok(job);
    }
    let job = qor_host_proof::KillOnCloseJob::new()?;
    Ok(JOB.get_or_init(|| job))
}

/// Start a product and serve its channel. Returns once it has started; the session and the supervision run on.
/// Returns the channel's name, which is not a secret.
pub async fn launch<H: Handler>(
    launch: Launch,
    handler: Arc<H>,
    on_exit: impl FnOnce(Exited) + Send + 'static,
) -> Result<String, HostError> {
    let token = SpawnToken::new();
    let channel = channel::Channel::open().map_err(HostError::Io)?;
    let name = channel.name().to_owned();

    let mut command = tokio::process::Command::new(&launch.product.path);
    command
        .args(&launch.args)
        .arg("--launcher-channel")
        .arg(&name)
        .current_dir(&launch.cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    if let Some(word) = reduced_motion_word() {
        command.env("QOR_REDUCED_MOTION", word);
    }
    let mut child = command.spawn().map_err(HostError::Io)?;
    let issued = Instant::now();
    let pid = child
        .id()
        .ok_or_else(|| HostError::Internal("the product ended as it started".into()))?;

    #[cfg(windows)]
    {
        let handle = child
            .raw_handle()
            .ok_or_else(|| HostError::Internal("the product ended as it started".into()))?;
        job()
            .and_then(|job| job.assign(&BorrowedRaw(handle)))
            .map_err(HostError::Io)?;
    }
    let expected = channel::Expected {
        pid,
        started: qor_host_proof::process_start_time(pid).ok(),
    };

    // The token on stdin, which is then closed: never an argument or a variable another process could read.
    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| HostError::Internal("the product has no stdin".into()))?;
    let mut line = Zeroizing::new(token.hex().as_bytes().to_vec());
    line.push(b'\n');
    stdin.write_all(&line).await.map_err(HostError::Io)?;
    drop(stdin);

    let product = launch.product.clone();
    let mut stderr = child.stderr.take();
    tokio::spawn(async move {
        let mut tail = Vec::new();
        if let Some(stderr) = stderr.as_mut() {
            let mut chunk = [0u8; 4096];
            while let Ok(read) = stderr.read(&mut chunk).await {
                if read == 0 {
                    break;
                }
                tail.extend_from_slice(&chunk[..read]);
                if tail.len() > STDERR_TAIL {
                    tail.drain(..tail.len() - STDERR_TAIL);
                }
            }
        }
        let code = child.wait().await.ok().and_then(|status| status.code());
        tracing::info!(
            product = product.id,
            ?code,
            "a product the launcher started has exited"
        );
        on_exit(Exited {
            product: product.id,
            code,
            stderr_tail: String::from_utf8_lossy(&tail).into_owned(),
        });
    });

    let ticket = session::Ticket {
        product: launch.product,
        token,
        issued,
    };
    tokio::spawn(async move {
        let product = ticket.product.id;
        let outcome = match channel.accept(expected).await {
            Ok(stream) => session::serve(stream, ticket, &*handler).await,
            Err(error) => Err(error),
        };
        if let Err(error) = outcome {
            tracing::warn!(product, %error, "a product's channel ended badly");
        }
    });

    Ok(name)
}

/// A raw process handle lent to the job for one call.
#[cfg(windows)]
struct BorrowedRaw(std::os::windows::io::RawHandle);

#[cfg(windows)]
impl std::os::windows::io::AsRawHandle for BorrowedRaw {
    fn as_raw_handle(&self) -> std::os::windows::io::RawHandle {
        self.0
    }
}
