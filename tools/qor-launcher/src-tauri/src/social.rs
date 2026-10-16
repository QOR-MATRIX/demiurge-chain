//! The launcher's side of Social (ADR-091 decision 3; roadmap L8.5).
//!
//! The host holds the one WebSocket to Social, as it holds every call to QOR ID: the QOR ID token goes from the
//! keychain into the socket's first frame and nowhere else, never into the webview, whose CSP stays as it is. The
//! interface asks through `social_request` and hears everything through Tauri events (`social://status`,
//! `social://welcome`, `social://event`).
//!
//! The socket is jsonrpsee's WebSocket transport, which subxt already brings for the chain (soketto over rustls), so
//! Social adds no library to the process that holds the vault. Its `receive` is not safe to cancel, so a reader task
//! owns it and hands frames to the connection's loop.
//!
//! The connection keeps itself up: lost, it tries again after 1, 2, 4 … 30 seconds; told the token expired, it asks
//! for a fresh one and says hello again at once; told the session is not live, it refreshes once and then reports
//! `signed_out` rather than retrying a sign-in that cannot work. A request waits at most 15 seconds and is never left
//! hanging when the connection ends. Nothing here logs a token or a frame.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use jsonrpsee_client_transport::ws::{Url, WsTransportClientBuilder};
use jsonrpsee_core::client::{ReceivedMessage, TransportReceiverT, TransportSenderT};
use parking_lot::{Mutex, RwLock};
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::{mpsc, oneshot};

use crate::error::{QorError, QorResult};

/// Where Social is until it is deployed (L8.6): one running on this machine.
pub const DEFAULT_SOCIAL: &str = "ws://127.0.0.1:8090/v1/socket";
/// The protocol version spoken (`services/social/src/socket.rs`).
const VERSION: u32 = 1;
/// The largest frame either side sends.
const FRAME_MAX: u32 = 64 * 1024;
const ANSWER_WITHIN: Duration = Duration::from_secs(15);
const RETRY_MAX: u64 = 30;

/// A Social address: `wss://`, or `ws://` on this machine only, so words and a token never cross a network unencrypted.
pub fn normalise_endpoint(value: &str) -> QorResult<String> {
    let url =
        Url::parse(value.trim()).map_err(|_| QorError::Internal("not a Social address".into()))?;
    let local = matches!(
        url.host_str(),
        Some("127.0.0.1") | Some("localhost") | Some("[::1]")
    );
    match url.scheme() {
        "wss" => Ok(url.to_string()),
        "ws" if local => Ok(url.to_string()),
        _ => Err(QorError::Internal(
            "Social is reached over wss://, or ws:// on this machine".into(),
        )),
    }
}

/// Where the connection stands, as the interface is told it.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Status {
    Off,
    Connecting,
    Connected,
    /// Lost; trying again in `in_secs`.
    Retrying {
        in_secs: u64,
        reason: String,
    },
    /// QOR ID says this session is not live: sign in again.
    SignedOut,
    /// Refused for a reason a retry would not change.
    Refused {
        reason: String,
    },
}

pub type TokenFuture<'a> = Pin<Box<dyn Future<Output = QorResult<String>> + Send + 'a>>;

/// QOR ID access tokens.
pub trait Tokens: Send + Sync + 'static {
    /// A token; `fresh` after Social said the last one was not live, so it is refreshed first.
    fn token(&self, fresh: bool) -> TokenFuture<'_>;
}

/// Where the connection reports.
pub trait Events: Send + Sync + 'static {
    fn status(&self, status: &Status);
    fn welcome(&self, welcome: &Value);
    fn event(&self, event: &Value);
}

type Answer = Result<Value, QorError>;
type Pending = Arc<Mutex<HashMap<u64, oneshot::Sender<Answer>>>>;

struct Link {
    outgoing: mpsc::Sender<String>,
    pending: Pending,
    task: tokio::task::JoinHandle<()>,
}

/// How one connection ended.
enum Ended {
    Lost(String),
    TokenExpired,
    NotLive,
    Refused(String),
}

fn unavailable(message: &str) -> QorError {
    QorError::Social {
        code: "unavailable".into(),
        message: message.into(),
    }
}

pub struct Social {
    endpoint: RwLock<String>,
    status: RwLock<Status>,
    welcome: RwLock<Option<Value>>,
    link: Mutex<Option<Link>>,
    next: AtomicU64,
}

impl Social {
    pub fn new(endpoint: String) -> Self {
        Self {
            endpoint: RwLock::new(endpoint),
            status: RwLock::new(Status::Off),
            welcome: RwLock::new(None),
            link: Mutex::new(None),
            next: AtomicU64::new(1),
        }
    }

    pub fn endpoint(&self) -> String {
        self.endpoint.read().clone()
    }

    pub fn status(&self) -> Status {
        self.status.read().clone()
    }

    /// The last `welcome`, while connected.
    pub fn welcome(&self) -> Option<Value> {
        self.welcome.read().clone()
    }

    fn set(&self, events: &dyn Events, status: Status) {
        if status != Status::Connected {
            *self.welcome.write() = None;
        }
        *self.status.write() = status.clone();
        events.status(&status);
    }

    /// Connect, and stay connected until [`Self::disconnect`]. Connecting while connected does nothing.
    pub fn connect(self: &Arc<Self>, tokens: Arc<dyn Tokens>, events: Arc<dyn Events>) {
        let mut link = self.link.lock();
        if link.as_ref().is_some_and(|l| !l.task.is_finished()) {
            return;
        }
        let (outgoing, receiver) = mpsc::channel(256);
        let pending: Pending = Arc::default();
        let task =
            tokio::spawn(Arc::clone(self).run(tokens, events, receiver, Arc::clone(&pending)));
        *link = Some(Link {
            outgoing,
            pending,
            task,
        });
    }

    /// Close the connection, failing whatever was waiting on it.
    pub fn disconnect(&self, events: &dyn Events) {
        if let Some(link) = self.link.lock().take() {
            link.task.abort();
            link.pending.lock().clear();
        }
        self.set(events, Status::Off);
    }

    /// Ask Social something and wait for its answer.
    pub async fn request(&self, method: &str, params: Value) -> QorResult<Value> {
        let (outgoing, pending) = match self.link.lock().as_ref() {
            Some(link) if self.status() == Status::Connected => {
                (link.outgoing.clone(), Arc::clone(&link.pending))
            }
            _ => return Err(unavailable("Social is not connected")),
        };
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        let (answer, waiting) = oneshot::channel();
        pending.lock().insert(id, answer);
        let frame =
            json!({ "type": "req", "id": id, "method": method, "params": params }).to_string();
        if outgoing.send(frame).await.is_err() {
            pending.lock().remove(&id);
            return Err(unavailable("Social's connection was lost; try again"));
        }
        match tokio::time::timeout(ANSWER_WITHIN, waiting).await {
            Ok(Ok(answer)) => answer,
            Ok(Err(_)) => Err(unavailable("Social's connection was lost; try again")),
            Err(_) => {
                pending.lock().remove(&id);
                Err(unavailable("Social did not answer in time"))
            }
        }
    }

    async fn run(
        self: Arc<Self>,
        tokens: Arc<dyn Tokens>,
        events: Arc<dyn Events>,
        mut outgoing: mpsc::Receiver<String>,
        pending: Pending,
    ) {
        let mut fresh = false;
        let mut failures: u32 = 0;
        self.set(&*events, Status::Connecting);
        loop {
            let (ended, welcomed) = self
                .session(&*tokens, &*events, &mut outgoing, &pending, fresh)
                .await;
            pending.lock().clear();
            if welcomed {
                failures = 0;
            }
            match ended {
                Ended::Refused(reason) => {
                    self.set(&*events, Status::Refused { reason });
                    return;
                }
                Ended::NotLive if !fresh => fresh = true,
                Ended::NotLive => {
                    self.set(&*events, Status::SignedOut);
                    return;
                }
                Ended::TokenExpired => fresh = true,
                Ended::Lost(reason) => {
                    fresh = false;
                    failures += 1;
                    let wait = (1u64 << failures.min(5).saturating_sub(1)).min(RETRY_MAX);
                    self.set(
                        &*events,
                        Status::Retrying {
                            in_secs: wait,
                            reason,
                        },
                    );
                    tokio::time::sleep(Duration::from_secs(wait)).await;
                    self.set(&*events, Status::Connecting);
                }
            }
        }
    }

    /// One connection, from hello to its end. Returns how it ended, and whether it was welcomed.
    async fn session(
        &self,
        tokens: &dyn Tokens,
        events: &dyn Events,
        outgoing: &mut mpsc::Receiver<String>,
        pending: &Pending,
        fresh: bool,
    ) -> (Ended, bool) {
        let token = match tokens.token(fresh).await {
            Ok(token) => token,
            Err(QorError::NotAuthenticated) => return (Ended::NotLive, false),
            Err(e) => return (Ended::Lost(e.to_string()), false),
        };
        let Ok(url) = Url::parse(&self.endpoint()) else {
            return (
                Ended::Refused("the Social address is not valid".into()),
                false,
            );
        };
        let (mut sender, mut receiver) = match WsTransportClientBuilder::default()
            .max_request_size(FRAME_MAX)
            .max_response_size(FRAME_MAX)
            .build(url)
            .await
        {
            Ok(pair) => pair,
            Err(_) => return (Ended::Lost("Social could not be reached".into()), false),
        };
        let hello = json!({
            "type": "hello",
            "version": VERSION,
            "token": token,
            "client": concat!("qor-launcher/", env!("CARGO_PKG_VERSION")),
        })
        .to_string();
        drop(token);
        if sender.send(hello).await.is_err() {
            return (Ended::Lost("Social closed the connection".into()), false);
        }

        let (frames, mut incoming) = mpsc::channel::<String>(256);
        let reader = tokio::spawn(async move {
            while let Ok(message) = receiver.receive().await {
                if let ReceivedMessage::Text(text) = message {
                    if frames.send(text).await.is_err() {
                        break;
                    }
                }
            }
        });
        let ended = self
            .converse(events, &mut sender, &mut incoming, outgoing, pending)
            .await;
        reader.abort();
        let _ = sender.close().await;
        ended
    }

    async fn converse<S: TransportSenderT>(
        &self,
        events: &dyn Events,
        sender: &mut S,
        incoming: &mut mpsc::Receiver<String>,
        outgoing: &mut mpsc::Receiver<String>,
        pending: &Pending,
    ) -> (Ended, bool) {
        // The first frame answers hello.
        let first = match tokio::time::timeout(ANSWER_WITHIN, incoming.recv()).await {
            Ok(Some(text)) => serde_json::from_str::<Value>(&text).unwrap_or(Value::Null),
            _ => return (Ended::Lost("Social did not answer hello".into()), false),
        };
        match first["type"].as_str() {
            Some("welcome") => {
                *self.welcome.write() = Some(first.clone());
                self.set(events, Status::Connected);
                events.welcome(&first);
            }
            Some("err") => {
                let code = first["code"].as_str().unwrap_or_default();
                let message = first["message"]
                    .as_str()
                    .unwrap_or("Social refused the connection")
                    .to_string();
                return match code {
                    "unauthorized" => (Ended::NotLive, false),
                    "version" | "hello_expected" => (Ended::Refused(message), false),
                    _ => (Ended::Lost(message), false),
                };
            }
            _ => {
                return (
                    Ended::Lost("Social answered hello with something else".into()),
                    false,
                )
            }
        }

        loop {
            tokio::select! {
                request = outgoing.recv() => {
                    let Some(frame) = request else {
                        return (Ended::Lost("closed".into()), true);
                    };
                    if sender.send(frame).await.is_err() {
                        return (Ended::Lost("Social's connection was lost".into()), true);
                    }
                }
                frame = incoming.recv() => {
                    let Some(text) = frame else {
                        return (Ended::Lost("Social's connection was lost".into()), true);
                    };
                    let value: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                    match value["type"].as_str() {
                        Some("event") => events.event(&value),
                        Some("res") => {
                            if let Some(waiting) = value["id"].as_u64().and_then(|id| pending.lock().remove(&id)) {
                                let _ = waiting.send(Ok(value["result"].clone()));
                            }
                        }
                        Some("err") => {
                            let code = value["code"].as_str().unwrap_or("unavailable").to_string();
                            let message = value["message"].as_str().unwrap_or("Social refused that").to_string();
                            match value["id"].as_u64() {
                                Some(id) => {
                                    if let Some(waiting) = pending.lock().remove(&id) {
                                        let _ = waiting.send(Err(QorError::Social { code, message }));
                                    }
                                }
                                None if code == "token_expired" => return (Ended::TokenExpired, true),
                                None => return (Ended::Lost(message), true),
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// QOR ID's tokens, from the launcher's own identity client.
impl Tokens for crate::identity::IdentityClient {
    fn token(&self, fresh: bool) -> TokenFuture<'_> {
        Box::pin(self.access_token(fresh))
    }
}

#[cfg(test)]
mod tests;
