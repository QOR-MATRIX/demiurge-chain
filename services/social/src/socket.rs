//! One WebSocket per client (ADR-091 decision 3; docs/blueprints/social.md section 2).
//!
//! The client's first frame is `hello`, within ten seconds, carrying its QOR ID access token, which is checked with QOR
//! ID and then forgotten. The service answers `welcome`, or an error and closes. After that the client sends typed
//! requests and the service answers each by its id, and pushes events. When the token expires the socket is closed
//! with `token_expired`, and the client says hello again with a fresh one. Frames are at most 64 KiB.

use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::extract::ws::{CloseFrame, Message as Frame, Utf8Bytes, WebSocket, WebSocketUpgrade};
use axum::response::Response;
use futures_util::{SinkExt as _, StreamExt as _};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::api::{self, Caller};
use crate::error::SocialError;
use crate::state::AppState;

/// The protocol's version.
pub const VERSION: u32 = 1;
/// The largest frame either side sends.
pub const FRAME_MAX: usize = 64 * 1024;
const HELLO_WITHIN: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Hello {
    r#type: String,
    version: u32,
    token: String,
    #[serde(default)]
    client: String,
}

#[derive(Deserialize)]
struct Request {
    r#type: String,
    id: u64,
    method: String,
    #[serde(default)]
    params: Value,
}

pub async fn upgrade(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> Response {
    ws.max_message_size(FRAME_MAX)
        .max_frame_size(FRAME_MAX)
        .on_upgrade(move |socket| session(socket, state))
}

fn error_frame(id: Option<u64>, code: &str, message: &str) -> String {
    json!({ "type": "err", "id": id, "code": code, "message": message }).to_string()
}

async fn refuse(mut socket: WebSocket, code: &str, message: &str) {
    let _ = socket
        .send(Frame::Text(error_frame(None, code, message).into()))
        .await;
    let _ = socket
        .send(Frame::Close(Some(CloseFrame {
            code: 4000,
            reason: Utf8Bytes::from(code.to_string()),
        })))
        .await;
}

async fn first_text(socket: &mut WebSocket) -> Option<String> {
    while let Some(Ok(frame)) = socket.recv().await {
        match frame {
            Frame::Text(text) => return Some(text.to_string()),
            Frame::Ping(_) | Frame::Pong(_) => continue,
            _ => return None,
        }
    }
    None
}

async fn session(mut socket: WebSocket, state: Arc<AppState>) {
    let hello = match tokio::time::timeout(HELLO_WITHIN, first_text(&mut socket)).await {
        Ok(Some(text)) => serde_json::from_str::<Hello>(&text).ok(),
        _ => None,
    };
    let Some(hello) = hello.filter(|h| h.r#type == "hello") else {
        return refuse(
            socket,
            "hello_expected",
            "the first frame is hello, with a version and a QOR ID token",
        )
        .await;
    };
    if hello.version != VERSION {
        return refuse(
            socket,
            "version",
            &format!("this service speaks version {VERSION}"),
        )
        .await;
    }
    let person = match state.identity.check(&hello.token).await {
        Ok(Some(person)) => person,
        Ok(None) => {
            return refuse(
                socket,
                "unauthorized",
                "that QOR ID session is not live; sign in again",
            )
            .await;
        }
        Err(_) => {
            return refuse(
                socket,
                "unavailable",
                "QOR ID could not be asked just now; try again shortly",
            )
            .await;
        }
    };
    drop(hello.token);
    tracing::info!(client = %hello.client.chars().take(40).collect::<String>(), "a person connected");

    let me = match state.store.arrive(&person).await {
        Ok(me) => me,
        Err(_) => {
            return refuse(
                socket,
                "unavailable",
                "Social could not open your session just now",
            )
            .await;
        }
    };
    let (connection, mut outbox) = state.hub.connect(person.sub).await;
    let welcome = async {
        let rooms = state.store.rooms_of(person.sub).await?;
        let invites = state.store.invites_for(person.sub).await?;
        let blocked = state.store.blocked_by(person.sub).await?;
        let contacts = state.store.contacts(person.sub).await?;
        let mut presence = serde_json::Map::new();
        for sub in contacts {
            presence.insert(sub.to_string(), json!(state.hub.presence(sub).await));
        }
        Ok::<_, SocialError>(json!({
            "type": "welcome",
            "version": VERSION,
            "me": me,
            "username_chosen": person.username_chosen,
            "rooms": rooms,
            "invites": invites,
            "blocked": blocked,
            "presence": presence,
        }))
    }
    .await;
    let Ok(welcome) = welcome else {
        state.hub.disconnect(person.sub, connection).await;
        return refuse(
            socket,
            "unavailable",
            "Social could not open your session just now",
        )
        .await;
    };
    let (mut sink, mut stream) = socket.split();
    if sink
        .send(Frame::Text(welcome.to_string().into()))
        .await
        .is_err()
    {
        state.hub.disconnect(person.sub, connection).await;
        return;
    }
    let _ = api::announce_presence(&state, person.sub).await;

    let expires_in = (person.expires_at - chrono::Utc::now().timestamp()).max(0) as u64;
    let expiry = tokio::time::sleep(Duration::from_secs(expires_in));
    tokio::pin!(expiry);
    let caller = Caller { person: &person };

    loop {
        tokio::select! {
            incoming = stream.next() => {
                let text = match incoming {
                    Some(Ok(Frame::Text(text))) => text,
                    Some(Ok(Frame::Ping(_) | Frame::Pong(_))) => continue,
                    _ => break,
                };
                let reply = match serde_json::from_str::<Request>(&text) {
                    Ok(request) if request.r#type == "req" => {
                        match api::handle(&state, &caller, &request.method, request.params).await {
                            Ok(result) => json!({ "type": "res", "id": request.id, "result": result }).to_string(),
                            Err(refusal) => {
                                tracing::info!(method = %request.method, code = refusal.code(), "a request was refused");
                                error_frame(Some(request.id), refusal.code(), &refusal.to_string())
                            }
                        }
                    }
                    _ => error_frame(None, "bad_request", "a frame is a request: type, id, method and params"),
                };
                if sink.send(Frame::Text(reply.into())).await.is_err() {
                    break;
                }
            }
            outgoing = outbox.recv() => {
                // None: the hub dropped this socket for falling behind; the client reconnects and catches up.
                let Some(frame) = outgoing else { break };
                if sink.send(Frame::Text(frame.into())).await.is_err() {
                    break;
                }
            }
            () = &mut expiry => {
                let _ = sink.send(Frame::Text(error_frame(None, "token_expired", "say hello again with a fresh QOR ID token").into())).await;
                let _ = sink.send(Frame::Close(Some(CloseFrame { code: 4001, reason: Utf8Bytes::from_static("token_expired") }))).await;
                break;
            }
        }
    }

    if state.hub.disconnect(person.sub, connection).await {
        let _ = api::announce_presence(&state, person.sub).await;
    }
    tracing::info!("a person disconnected");
}
