//! Nothing secret reaches a log, at any level (AGENTS.md section 9, adopted from Social's first commit; ADR-091).
//!
//! The real path end to end: the real QOR ID client asks a stand-in QOR ID over HTTP, with Social's own secret, about
//! the token in a real socket's hello; then every kind of request runs, carrying words, a bio and a status. Every log
//! record at TRACE is captured, and the check fails if a token, the client secret, a message key, a message's words,
//! a bio or a report's note appears in it. It first proves the capture worked.

use std::collections::HashMap;
use std::io::Write;
use std::sync::{Arc, Mutex};

use axum::Form;
use axum::routing::post;
use base64::Engine as _;
use serde_json::{Value, json};
use sqlx::PgPool;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::util::SubscriberInitExt as _;
use uuid::Uuid;

use crate::hub::Hub;
use crate::identity::QorId;
use crate::seal::Sealer;
use crate::state::AppState;
use crate::store::Store;

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("lock").extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> MakeWriter<'a> for Captured {
    type Writer = Captured;
    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

/// A stand-in QOR ID: answers introspection for the tokens it was given, to the one service with the right secret.
async fn qor_id(secret: String, people: HashMap<String, Value>) -> String {
    let app = axum::Router::new().route(
        "/oauth/introspect",
        post(move |Form(form): Form<HashMap<String, String>>| {
            let (secret, people) = (secret.clone(), people.clone());
            async move {
                if form.get("client_id").map(String::as_str) != Some("social")
                    || form.get("client_secret") != Some(&secret)
                {
                    return (
                        axum::http::StatusCode::UNAUTHORIZED,
                        axum::Json(json!({ "error": "invalid_client" })),
                    );
                }
                let answer = form
                    .get("token")
                    .and_then(|t| people.get(t))
                    .cloned()
                    .unwrap_or_else(|| json!({ "active": false }));
                (axum::http::StatusCode::OK, axum::Json(answer))
            }
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a port");
    let address = listener.local_addr().expect("an address");
    tokio::spawn(async move { axum::serve(listener, app).await });
    format!("http://{address}")
}

#[sqlx::test]
async fn nothing_secret_reaches_a_log_at_any_level(db: PgPool) {
    let captured = Captured::default();
    use tracing_subscriber::Layer as _;
    use tracing_subscriber::layer::SubscriberExt as _;
    // Everything at trace, through the filter the service itself runs under.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(captured.clone())
                .with_filter(tracing_subscriber::filter::LevelFilter::TRACE)
                .with_filter(tracing_subscriber::filter::filter_fn(crate::may_log)),
        )
        .try_init()
        .expect("the log check is the only test that installs a subscriber");
    tracing::trace!("log check: capturing at trace level");

    let client_secret = format!("social-secret-{}", Uuid::new_v4().simple());
    let key = base64::engine::general_purpose::STANDARD.encode(rand::random::<[u8; 32]>());
    let tokens: Vec<String> = (0..2)
        .map(|n| format!("qor-token-{n}-{}", Uuid::new_v4().simple()))
        .collect();
    let subs: Vec<Uuid> = (0..2).map(|_| Uuid::new_v4()).collect();
    let expires = chrono::Utc::now().timestamp() + 3600;
    let people: HashMap<String, Value> = [("alice", 0usize), ("bob", 1)]
        .into_iter()
        .map(|(name, n)| {
            (
                tokens[n].clone(),
                json!({ "active": true, "sub": subs[n], "qor_id": name, "username_chosen": true,
                        "avatar_url": null, "exp": expires, "progress": { "level": 2 } }),
            )
        })
        .collect();
    let base = qor_id(client_secret.clone(), people).await;

    let state = Arc::new(AppState {
        store: Store::new(db, Sealer::from_spec(&format!("1:{key}")).expect("a key")),
        identity: Arc::new(
            QorId::new(&base, "social".into(), client_secret.clone()).expect("QOR ID"),
        ),
        hub: Hub::default(),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a port");
    let url = format!(
        "ws://{}/v1/socket",
        listener.local_addr().expect("an address")
    );
    tokio::spawn(async move { axum::serve(listener, crate::router(state)).await });
    let harness = crate::tests::Harness {
        url,
        fake: Default::default(),
        db: sqlx::PgPool::connect_lazy("postgres://unused").expect("lazy"),
    };

    let words = format!("the words of a message {}", Uuid::new_v4().simple());
    let edited = format!("the edited words {}", Uuid::new_v4().simple());
    let private = format!("a direct word {}", Uuid::new_v4().simple());
    let bio = format!("a private bio {}", Uuid::new_v4().simple());
    let note = format!("a report's note {}", Uuid::new_v4().simple());
    let status = format!("a status {}", &Uuid::new_v4().simple().to_string()[..8]);

    let (mut a, welcome) = harness.connect(&tokens[0]).await;
    assert_eq!(welcome["type"], "welcome", "{welcome}");
    let (mut b, _) = harness.connect(&tokens[1]).await;
    let (refused, _) = harness.connect("a token QOR ID never issued").await;
    drop(refused);

    let room = a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Log room" }),
        )
        .await["room"]["id"]
        .as_str()
        .expect("an id")
        .to_string();
    b.ok("rooms.join", json!({ "room": room })).await;
    let message = a
        .ok("messages.send", json!({ "room": room, "body": words }))
        .await["message"]["id"]
        .as_str()
        .expect("an id")
        .to_string();
    a.ok(
        "messages.edit",
        json!({ "message": message, "body": edited }),
    )
    .await;
    b.ok(
        "messages.react",
        json!({ "message": message, "emoji": "👍" }),
    )
    .await;
    b.ok("messages.history", json!({ "room": room })).await;
    b.ok(
        "report",
        json!({ "message": message, "reason": "other", "note": note }),
    )
    .await;
    let dm = a.ok("dm.open", json!({ "qor_id": "bob" })).await["room"]["id"]
        .as_str()
        .expect("an id")
        .to_string();
    a.ok("messages.send", json!({ "room": dm, "body": private }))
        .await;
    a.ok(
        "profile.update",
        json!({ "bio": bio, "status_line": status }),
    )
    .await;
    a.ok(
        "presence.set",
        json!({ "state": "away", "playing": "Flux Four" }),
    )
    .await;
    let _ = a
        .call(
            "messages.send",
            json!({ "room": room, "body": "", "secret_field": words }),
        )
        .await;
    a.ok("messages.delete", json!({ "message": message })).await;
    drop((a, b));
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let log = String::from_utf8_lossy(&captured.0.lock().expect("lock")).into_owned();
    for expected in [
        "log check: capturing at trace level",
        "a person connected",
        "a request was refused",
        "a person disconnected",
    ] {
        assert!(
            log.contains(expected),
            "the log should contain {expected:?}"
        );
    }

    let mut plain: Vec<String> = vec![
        client_secret.clone(),
        key.clone(),
        words.clone(),
        edited.clone(),
        private.clone(),
        bio.clone(),
        note.clone(),
        status.clone(),
        "a token QOR ID never issued".into(),
    ];
    plain.extend(tokens.iter().cloned());
    // A payload printed as bytes rather than text: the first sixteen, as `Debug` writes a byte slice.
    let as_bytes: Vec<String> = plain
        .iter()
        .map(|s| {
            let shown = format!("{:?}", &s.as_bytes()[..16.min(s.len())]);
            shown
                .trim_start_matches('[')
                .trim_end_matches(']')
                .to_string()
        })
        .collect();
    let secrets: Vec<&str> = plain
        .iter()
        .chain(as_bytes.iter())
        .map(String::as_str)
        .collect();
    // A failure message is itself output: every line it quotes has every known secret masked, longest first.
    let mut masks = secrets.clone();
    masks.sort_by_key(|s| std::cmp::Reverse(s.len()));
    let masked = |line: &str| {
        masks
            .iter()
            .fold(line.to_string(), |line, s| line.replace(*s, "<SECRET>"))
    };
    let leaks: Vec<String> = secrets
        .iter()
        .filter(|s| log.contains(**s))
        .map(|s| {
            let lines: Vec<String> = log
                .lines()
                .filter(|l| l.contains(*s))
                .take(2)
                .map(masked)
                .collect();
            format!(
                "a value of {} characters, in:\n  {}",
                s.len(),
                lines.join("\n  ")
            )
        })
        .collect();
    assert!(leaks.is_empty(), "secret values reached the log: {leaks:?}");
}
