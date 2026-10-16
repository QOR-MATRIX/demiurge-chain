//! The launcher's Social socket against a stand-in Social on a loopback port, speaking the real protocol over soketto.

use std::sync::Arc;
use std::time::Duration;

use parking_lot::Mutex;
use serde_json::{json, Value};
use tokio::net::TcpListener;
use tokio_util::compat::{Compat, TokioAsyncReadCompatExt as _};

use super::*;

type Stream = Compat<tokio::net::TcpStream>;

/// One client connection, from the server's side.
struct Peer {
    sender: soketto::Sender<Stream>,
    receiver: soketto::Receiver<Stream>,
}

impl Peer {
    async fn accept(listener: &TcpListener) -> Peer {
        let (stream, _) = tokio::time::timeout(Duration::from_secs(10), listener.accept())
            .await
            .expect("a connection in time")
            .expect("accepted");
        let mut server = soketto::handshake::Server::new(stream.compat());
        let key = server.receive_request().await.expect("a handshake").key();
        server
            .send_response(&soketto::handshake::server::Response::Accept {
                key,
                protocol: None,
            })
            .await
            .expect("accepted");
        let (sender, receiver) = server.into_builder().finish();
        Peer { sender, receiver }
    }

    async fn frame(&mut self) -> Value {
        let mut bytes = Vec::new();
        tokio::time::timeout(
            Duration::from_secs(10),
            self.receiver.receive_data(&mut bytes),
        )
        .await
        .expect("a frame in time")
        .expect("a frame");
        serde_json::from_slice(&bytes).expect("json")
    }

    async fn send(&mut self, value: Value) {
        self.sender
            .send_text(value.to_string())
            .await
            .expect("sent");
        self.sender.flush().await.expect("flushed");
    }

    /// Read hello, check it, and return its token.
    async fn hello(&mut self) -> String {
        let hello = self.frame().await;
        assert_eq!(hello["type"], "hello", "{hello}");
        assert_eq!(hello["version"], 1);
        assert!(hello["client"]
            .as_str()
            .is_some_and(|c| c.starts_with("qor-launcher/")));
        hello["token"].as_str().expect("a token").to_string()
    }

    async fn welcome(&mut self) {
        self.send(
            json!({ "type": "welcome", "version": 1, "me": { "qor_id": "alice" }, "rooms": [] }),
        )
        .await;
    }
}

#[derive(Default)]
struct Recorder {
    statuses: Mutex<Vec<Status>>,
    welcomes: Mutex<Vec<Value>>,
    events: Mutex<Vec<Value>>,
}

impl Events for Recorder {
    fn status(&self, status: &Status) {
        self.statuses.lock().push(status.clone());
    }
    fn welcome(&self, welcome: &Value) {
        self.welcomes.lock().push(welcome.clone());
    }
    fn event(&self, event: &Value) {
        self.events.lock().push(event.clone());
    }
}

/// Tokens as QOR ID would hand them out: a fresh one after a refresh; `signed_out` once refreshing cannot work.
#[derive(Default)]
struct FakeTokens {
    asked: Mutex<Vec<bool>>,
    signed_out: Mutex<bool>,
}

impl Tokens for FakeTokens {
    fn token(&self, fresh: bool) -> TokenFuture<'_> {
        self.asked.lock().push(fresh);
        let signed_out = *self.signed_out.lock();
        Box::pin(async move {
            if signed_out {
                return Err(QorError::NotAuthenticated);
            }
            Ok(if fresh { "token-fresh" } else { "token-first" }.to_string())
        })
    }
}

async fn wait_for(recorder: &Recorder, wanted: impl Fn(&Status) -> bool) {
    for _ in 0..200 {
        if recorder.statuses.lock().last().is_some_and(&wanted) {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!(
        "never reached the status wanted: {:?}",
        recorder.statuses.lock()
    );
}

async fn setup() -> (TcpListener, Arc<Social>, Arc<FakeTokens>, Arc<Recorder>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("a port");
    let url = format!(
        "ws://{}/v1/socket",
        listener.local_addr().expect("an address")
    );
    (
        listener,
        Arc::new(Social::new(url)),
        Arc::new(FakeTokens::default()),
        Arc::new(Recorder::default()),
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn hello_carries_the_token_and_events_and_answers_come_back() {
    let (listener, social, tokens, recorder) = setup().await;
    assert!(
        matches!(social.request("rooms.list", json!({})).await, Err(QorError::Social { code, .. }) if code == "unavailable"),
        "not connected yet"
    );
    social
        .connect(tokens.clone(), recorder.clone())
        .expect("connects");

    let mut peer = Peer::accept(&listener).await;
    assert_eq!(peer.hello().await, "token-first");
    peer.welcome().await;
    wait_for(&recorder, |s| *s == Status::Connected).await;
    assert_eq!(recorder.welcomes.lock().len(), 1);
    assert_eq!(social.welcome().expect("kept")["me"]["qor_id"], "alice");

    let asking = {
        let social = Arc::clone(&social);
        tokio::spawn(async move { social.request("rooms.list", json!({ "x": 1 })).await })
    };
    let request = peer.frame().await;
    assert_eq!(
        (request["type"].clone(), request["method"].clone()),
        (json!("req"), json!("rooms.list"))
    );
    assert_eq!(request["params"], json!({ "x": 1 }));
    peer.send(json!({ "type": "event", "event": "presence", "data": { "sub": "x" } }))
        .await;
    peer.send(json!({ "type": "res", "id": request["id"], "result": { "rooms": [1, 2] } }))
        .await;
    assert_eq!(
        asking.await.expect("joined").expect("answered"),
        json!({ "rooms": [1, 2] })
    );
    assert_eq!(recorder.events.lock()[0]["event"], "presence");

    // A refusal keeps Social's own code, for the interface to act on.
    let asking = {
        let social = Arc::clone(&social);
        tokio::spawn(async move { social.request("messages.send", json!({})).await })
    };
    let request = peer.frame().await;
    peer.send(json!({ "type": "err", "id": request["id"], "code": "rate_limited", "message": "slow down" })).await;
    match asking.await.expect("joined") {
        Err(QorError::Social { code, message }) => assert_eq!(
            (code.as_str(), message.as_str()),
            ("rate_limited", "slow down")
        ),
        other => panic!("{other:?}"),
    }
    assert_eq!(*tokens.asked.lock(), [false], "one token, not refreshed");

    social.disconnect(&*recorder);
    assert_eq!(social.status(), Status::Off);
    assert!(social.welcome().is_none());
}

#[tokio::test(flavor = "multi_thread")]
async fn a_session_that_is_not_live_is_refreshed_once_then_reported_signed_out() {
    let (listener, social, tokens, recorder) = setup().await;
    social
        .connect(tokens.clone(), recorder.clone())
        .expect("connects");
    let mut first = Peer::accept(&listener).await;
    assert_eq!(first.hello().await, "token-first");
    first
        .send(json!({ "type": "err", "id": null, "code": "unauthorized", "message": "not live" }))
        .await;
    let mut second = Peer::accept(&listener).await;
    assert_eq!(
        second.hello().await,
        "token-fresh",
        "refreshed, and tried again at once"
    );
    second
        .send(json!({ "type": "err", "id": null, "code": "unauthorized", "message": "not live" }))
        .await;
    wait_for(&recorder, |s| *s == Status::SignedOut).await;
    assert_eq!(*tokens.asked.lock(), [false, true]);
    assert!(
        tokio::time::timeout(Duration::from_millis(500), listener.accept())
            .await
            .is_err(),
        "no third try: signing in again is the person's"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn an_expired_token_is_replaced_and_hello_said_again_at_once() {
    let (listener, social, tokens, recorder) = setup().await;
    social
        .connect(tokens.clone(), recorder.clone())
        .expect("connects");
    let mut first = Peer::accept(&listener).await;
    first.hello().await;
    first.welcome().await;
    wait_for(&recorder, |s| *s == Status::Connected).await;
    first
        .send(json!({ "type": "err", "id": null, "code": "token_expired", "message": "again" }))
        .await;
    let mut second = Peer::accept(&listener).await;
    assert_eq!(second.hello().await, "token-fresh");
    second.welcome().await;
    wait_for(&recorder, |s| *s == Status::Connected).await;
    assert!(
        !recorder
            .statuses
            .lock()
            .iter()
            .any(|s| matches!(s, Status::Retrying { .. })),
        "no wait: {:?}",
        recorder.statuses.lock()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_lost_connection_fails_what_waits_on_it_and_comes_back() {
    let (listener, social, tokens, recorder) = setup().await;
    social
        .connect(tokens.clone(), recorder.clone())
        .expect("connects");
    let mut first = Peer::accept(&listener).await;
    first.hello().await;
    first.welcome().await;
    wait_for(&recorder, |s| *s == Status::Connected).await;

    let asking = {
        let social = Arc::clone(&social);
        tokio::spawn(async move { social.request("rooms.list", json!({})).await })
    };
    let _request = first.frame().await;
    drop(first);
    let started = std::time::Instant::now();
    match asking.await.expect("joined") {
        Err(QorError::Social { code, .. }) => assert_eq!(code, "unavailable"),
        other => panic!("{other:?}"),
    }
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "failed at once, not after the timeout"
    );
    wait_for(&recorder, |s| {
        matches!(s, Status::Retrying { in_secs: 1, .. })
    })
    .await;

    let mut second = Peer::accept(&listener).await;
    assert_eq!(
        second.hello().await,
        "token-first",
        "a lost connection is not a dead token"
    );
    second.welcome().await;
    wait_for(&recorder, |s| *s == Status::Connected).await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_signed_out_launcher_does_not_knock() {
    let (listener, social, tokens, recorder) = setup().await;
    *tokens.signed_out.lock() = true;
    social
        .connect(tokens.clone(), recorder.clone())
        .expect("connects");
    wait_for(&recorder, |s| *s == Status::SignedOut).await;
    assert!(
        tokio::time::timeout(Duration::from_millis(300), listener.accept())
            .await
            .is_err(),
        "nothing sent without a token"
    );
}

#[test]
fn social_is_reached_over_wss_or_on_this_machine() {
    for good in [
        "wss://social.qorsync.dev/v1/socket",
        "ws://127.0.0.1:8090/v1/socket",
        "ws://localhost:8090/v1/socket",
    ] {
        assert!(normalise_endpoint(good).is_ok(), "{good}");
    }
    for bad in [
        "ws://social.qorsync.dev/v1/socket",
        "https://social.qorsync.dev",
        "file:///etc/passwd",
        "not a url",
    ] {
        assert!(normalise_endpoint(bad).is_err(), "{bad}");
    }
}

/// A whole session with every level of the launcher's log on: hello with its token, a request carrying words, an
/// event and an answer carrying words, and a refusal. None of the token or the words appears in the log, as text or as
/// bytes; the log is checked to hold the session, so a silent logger cannot pass.
#[tokio::test(flavor = "multi_thread")]
async fn nothing_secret_from_social_reaches_the_log_at_any_level() {
    let captured = crate::testing::captured_log();
    let from = captured.0.lock().unwrap().len();
    let (listener, social, tokens, recorder) = setup().await;
    let words = format!("words only the room should read {}", std::process::id());
    let answered = format!(
        "an answer only the asker should read {}",
        std::process::id()
    );

    social
        .connect(tokens.clone(), recorder.clone())
        .expect("connects");
    let mut peer = Peer::accept(&listener).await;
    let token = peer.hello().await;
    peer.welcome().await;
    wait_for(&recorder, |s| *s == Status::Connected).await;
    let asking = {
        let social = Arc::clone(&social);
        let words = words.clone();
        tokio::spawn(async move {
            social
                .request("messages.send", json!({ "body": words }))
                .await
        })
    };
    let request = peer.frame().await;
    peer.send(json!({ "type": "event", "event": "message.created", "data": { "body": answered } }))
        .await;
    peer.send(json!({ "type": "res", "id": request["id"], "result": { "body": answered } }))
        .await;
    asking.await.expect("joined").expect("answered");
    let refused = {
        let social = Arc::clone(&social);
        tokio::spawn(async move {
            social
                .request("messages.send", json!({ "body": "x" }))
                .await
        })
    };
    let request = peer.frame().await;
    peer.send(json!({ "type": "err", "id": request["id"], "code": "rate_limited", "message": "slow down" })).await;
    assert!(refused.await.expect("joined").is_err());
    social.disconnect(&*recorder);
    tokio::time::sleep(Duration::from_millis(200)).await;

    let log = String::from_utf8_lossy(&captured.0.lock().unwrap()[from..]).into_owned();
    assert!(
        log.contains("jsonrpsee") || log.contains("soketto"),
        "the session reached the log at all"
    );
    for secret in [token.as_str(), words.as_str(), answered.as_str()] {
        let bytes = format!("{:?}", &secret.as_bytes()[..16.min(secret.len())]);
        let bytes = bytes.trim_start_matches('[').trim_end_matches(']');
        assert!(
            !log.contains(secret) && !log.contains(bytes),
            "a value of {} characters reached the log",
            secret.len()
        );
    }
}

/// The launcher aborted at start-up when a synchronous command connected from the main thread, outside any runtime
/// (16 October 2026). Outside a runtime, connecting is refused, and nothing panics.
#[test]
fn connecting_outside_a_runtime_is_refused_not_a_panic() {
    let social = Arc::new(Social::new("ws://127.0.0.1:9/v1/socket".into()));
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        social.connect(
            Arc::new(FakeTokens::default()),
            Arc::new(Recorder::default()),
        )
    }));
    let refused = outcome.expect("no panic");
    assert!(refused.is_err(), "refused outside a runtime");
    assert_eq!(social.status(), Status::Off, "and nothing started");
}
