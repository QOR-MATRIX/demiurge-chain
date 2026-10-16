//! Social end to end: a real WebSocket on a loopback port, the real store on Postgres, and people QOR ID would
//! vouch for (docs/blueprints/social.md section 4: the rules that are tested, not hoped for).

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt as _, StreamExt as _};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio_tungstenite::tungstenite::Message as Frame;
use uuid::Uuid;

use crate::hub::Hub;
use crate::identity::{Fake, Person};
use crate::seal::Sealer;
use crate::state::AppState;
use crate::store::Store;

type Socket =
    tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>;

pub struct Harness {
    pub url: String,
    pub fake: Arc<Fake>,
    pub db: PgPool,
}

pub async fn start(db: PgPool) -> Harness {
    let fake = Arc::new(Fake::default());
    let state = Arc::new(AppState {
        store: Store::new(db.clone(), Sealer::random()),
        identity: fake.clone(),
        hub: Hub::default(),
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("a port");
    let address = listener.local_addr().expect("an address");
    tokio::spawn(async move { axum::serve(listener, crate::router(state)).await });
    Harness {
        url: format!("ws://{address}/v1/socket"),
        fake,
        db,
    }
}

impl Harness {
    /// Someone QOR ID vouches for, by token.
    pub fn person(&self, name: &str) -> (String, Person) {
        self.person_with(name, true, 3600)
    }

    pub fn person_with(&self, name: &str, chosen: bool, lives_secs: i64) -> (String, Person) {
        let token = format!("token-of-{name}-{}", Uuid::new_v4().simple());
        let person = Person {
            sub: Uuid::new_v4(),
            qor_id: name.to_string(),
            username_chosen: chosen,
            avatar_url: None,
            level: 3,
            expires_at: chrono::Utc::now().timestamp() + lives_secs,
        };
        self.fake
            .people
            .lock()
            .expect("lock")
            .insert(token.clone(), person.clone());
        (token, person)
    }

    pub async fn connect(&self, token: &str) -> (Client, Value) {
        let (socket, _) = tokio_tungstenite::connect_async(&self.url)
            .await
            .expect("connects");
        let mut client = Client {
            socket,
            next: 1,
            events: VecDeque::new(),
        };
        client
            .send_raw(json!({ "type": "hello", "version": 1, "token": token, "client": "tests" }))
            .await;
        let welcome = client.frame().await.expect("an answer to hello");
        (client, welcome)
    }
}

pub struct Client {
    socket: Socket,
    next: u64,
    events: VecDeque<Value>,
}

/// A refusal: its code and its sentence.
#[derive(Debug)]
pub struct Refused {
    pub code: String,
    pub message: String,
}

impl Client {
    pub async fn send_raw(&mut self, value: Value) {
        self.socket
            .send(Frame::Text(value.to_string().into()))
            .await
            .expect("sent");
    }

    /// The next JSON frame, or None when the socket ends.
    pub async fn frame(&mut self) -> Option<Value> {
        loop {
            match tokio::time::timeout(Duration::from_secs(10), self.socket.next())
                .await
                .ok()??
            {
                Ok(Frame::Text(text)) => return serde_json::from_str(&text).ok(),
                Ok(Frame::Ping(_) | Frame::Pong(_)) => continue,
                _ => return None,
            }
        }
    }

    pub async fn call(&mut self, method: &str, params: Value) -> Result<Value, Refused> {
        let id = self.next;
        self.next += 1;
        self.send_raw(json!({ "type": "req", "id": id, "method": method, "params": params }))
            .await;
        loop {
            let frame = self.frame().await.expect("an answer");
            match frame["type"].as_str() {
                Some("event") => self.events.push_back(frame),
                Some("res") if frame["id"] == id => return Ok(frame["result"].clone()),
                Some("err") if frame["id"] == id => {
                    return Err(Refused {
                        code: frame["code"].as_str().unwrap_or_default().into(),
                        message: frame["message"].as_str().unwrap_or_default().into(),
                    });
                }
                _ => panic!("an unexpected frame: {frame}"),
            }
        }
    }

    pub async fn ok(&mut self, method: &str, params: Value) -> Value {
        self.call(method, params)
            .await
            .unwrap_or_else(|r| panic!("{method}: {r:?}"))
    }

    pub async fn refused(&mut self, method: &str, params: Value) -> String {
        match self.call(method, params).await {
            Ok(result) => panic!("{method} should have been refused: {result}"),
            Err(r) => r.code,
        }
    }

    /// The next event of this name, skipping others.
    pub async fn event(&mut self, name: &str) -> Value {
        if let Some(at) = self.events.iter().position(|e| e["event"] == name) {
            return self.events.remove(at).expect("present")["data"].clone();
        }
        loop {
            let frame = self
                .frame()
                .await
                .unwrap_or_else(|| panic!("no {name} event arrived"));
            if frame["type"] == "event" && frame["event"] == name {
                return frame["data"].clone();
            }
        }
    }

    /// Drop every event of this name already received, so what is awaited next is caused by what follows.
    pub async fn forget(&mut self, name: &str) {
        while let Ok(Some(Ok(Frame::Text(text)))) =
            tokio::time::timeout(Duration::from_millis(200), self.socket.next()).await
        {
            self.events
                .push_back(serde_json::from_str(&text).expect("json"));
        }
        self.events.retain(|e| e["event"] != name);
    }

    /// No event of this name arrives within a moment.
    pub async fn none(&mut self, name: &str) {
        assert!(
            !self.events.iter().any(|e| e["event"] == name),
            "a {name} event arrived"
        );
        while let Ok(Some(Ok(Frame::Text(text)))) =
            tokio::time::timeout(Duration::from_millis(400), self.socket.next()).await
        {
            let frame: Value = serde_json::from_str(&text).expect("json");
            assert!(frame["event"] != name, "a {name} event arrived: {frame}");
        }
    }
}

fn id(value: &Value) -> String {
    value["id"].as_str().expect("an id").to_string()
}

#[sqlx::test]
async fn hello_is_first_and_only_a_live_qor_id_session_gets_in(db: PgPool) {
    let h = start(db).await;
    let (token, me) = h.person("alice");

    let (_, refused) = h.connect("not a live token").await;
    assert_eq!(refused["code"], "unauthorized", "{refused}");

    let (mut wrong, _) = (
        tokio_tungstenite::connect_async(&h.url)
            .await
            .expect("connects")
            .0,
        (),
    );
    wrong
        .send(Frame::Text(
            json!({ "type": "hello", "version": 9, "token": token })
                .to_string()
                .into(),
        ))
        .await
        .expect("sent");
    let answer = wrong.next().await.expect("a frame").expect("ok");
    assert!(
        answer.to_text().expect("text").contains("\"version\""),
        "{answer}"
    );

    let (_, welcome) = h.connect(&token).await;
    assert_eq!(welcome["type"], "welcome", "{welcome}");
    assert_eq!(welcome["version"], 1);
    assert_eq!(welcome["me"]["sub"], me.sub.to_string());
    assert_eq!(welcome["me"]["qor_id"], "alice");
    assert_eq!(welcome["me"]["level"], 3);
    assert_eq!(welcome["username_chosen"], true);
    assert_eq!(welcome["rooms"], json!([]));
}

#[sqlx::test]
async fn a_room_carries_messages_replies_edits_reactions_and_deletes_to_its_members(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;

    let room = a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "  The   Lounge ", "topic": "hi" }),
        )
        .await["room"]
        .clone();
    assert_eq!(room["name"], "The Lounge");
    let room_id = id(&room);
    let found = b.ok("rooms.discover", json!({})).await;
    assert!(
        found["rooms"]
            .as_array()
            .expect("rooms")
            .iter()
            .any(|r| r["room"]["id"] == room_id.as_str())
    );
    b.ok("rooms.join", json!({ "room": room_id })).await;
    assert_eq!(a.event("member.joined").await["sub"], bob.sub.to_string());

    let sent = a
        .ok(
            "messages.send",
            json!({ "room": room_id, "body": "hello, bob" }),
        )
        .await["message"]
        .clone();
    assert_eq!(sent["seq"], 1);
    let heard = b.event("message.created").await["message"].clone();
    assert_eq!(heard["body"], "hello, bob");
    assert_eq!(heard["author"], alice.sub.to_string());
    let listed = b.ok("rooms.list", json!({})).await;
    assert_eq!(listed["rooms"][0]["unread"], 1, "{listed}");
    b.ok("messages.read", json!({ "room": room_id, "seq": 1 }))
        .await;
    assert_eq!(b.ok("rooms.list", json!({})).await["rooms"][0]["unread"], 0);

    let reply = b
        .ok(
            "messages.send",
            json!({ "room": room_id, "body": "hi!", "reply_to": id(&sent) }),
        )
        .await["message"]
        .clone();
    assert_eq!(reply["reply_to"], id(&sent));
    assert_eq!(reply["seq"], 2);

    assert_eq!(
        b.refused(
            "messages.edit",
            json!({ "message": id(&sent), "body": "mine now" })
        )
        .await,
        "forbidden"
    );
    a.ok(
        "messages.edit",
        json!({ "message": id(&sent), "body": "hello again, bob" }),
    )
    .await;
    let edited = b.event("message.edited").await["message"].clone();
    assert_eq!(edited["body"], "hello again, bob");
    assert!(edited["edited_at"].is_string());

    b.ok(
        "messages.react",
        json!({ "message": id(&sent), "emoji": "🔥" }),
    )
    .await;
    let reacted = a.event("message.reaction").await["message"].clone();
    assert_eq!(
        reacted["reactions"],
        json!([{ "emoji": "🔥", "by": [bob.sub] }])
    );
    b.ok(
        "messages.react",
        json!({ "message": id(&sent), "emoji": "🔥" }),
    )
    .await;
    assert_eq!(
        a.event("message.reaction").await["message"]["reactions"],
        json!([]),
        "a second press takes it back"
    );
    assert_eq!(
        b.refused(
            "messages.react",
            json!({ "message": id(&sent), "emoji": "lol" })
        )
        .await,
        "bad_request"
    );

    assert_eq!(
        b.refused("messages.delete", json!({ "message": id(&sent) }))
            .await,
        "forbidden"
    );
    a.ok(
        "rooms.role",
        json!({ "room": room_id, "sub": bob.sub, "role": "moderator" }),
    )
    .await;
    b.ok("messages.delete", json!({ "message": id(&sent) }))
        .await;
    let gone = a.event("message.deleted").await["message"].clone();
    assert_eq!(
        (gone["deleted"].clone(), gone["body"].clone()),
        (json!(true), Value::Null)
    );

    let history = a.ok("messages.history", json!({ "room": room_id })).await;
    let seqs: Vec<i64> = history["messages"]
        .as_array()
        .expect("a page")
        .iter()
        .map(|m| m["seq"].as_i64().expect("seq"))
        .collect();
    assert_eq!(seqs, [1, 2], "a deleted message keeps its place");
    assert_eq!(
        history["messages"][0]["body"],
        Value::Null,
        "and never its words"
    );
    assert!(history["profiles"][bob.sub.to_string()]["qor_id"] == "bob");
    assert_eq!(
        a.refused(
            "messages.edit",
            json!({ "message": id(&sent), "body": "back" })
        )
        .await,
        "not_found"
    );
}

#[sqlx::test]
async fn only_members_read_and_a_private_room_is_joined_by_invitation(db: PgPool) {
    let h = start(db).await;
    let (ta, _) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (tc, carol) = h.person("carol");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (mut c, _) = h.connect(&tc).await;

    let room = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "private", "name": "Council" }),
        )
        .await["room"]);
    a.ok(
        "messages.send",
        json!({ "room": room, "body": "secret plans" }),
    )
    .await;
    for (method, params) in [
        ("messages.history", json!({ "room": room })),
        ("rooms.members", json!({ "room": room })),
        (
            "messages.send",
            json!({ "room": room, "body": "let me in" }),
        ),
        ("rooms.join", json!({ "room": room })),
    ] {
        assert_eq!(c.refused(method, params).await, "not_found", "{method}");
    }
    let discovered = c.ok("rooms.discover", json!({})).await;
    assert_eq!(
        discovered["rooms"],
        json!([]),
        "a private room is not listed"
    );

    a.ok("rooms.invite", json!({ "room": room, "qor_id": "BOB" }))
        .await;
    assert_eq!(b.event("invite").await["room"], room.as_str());
    b.ok("rooms.join", json!({ "room": room })).await;
    assert_eq!(
        b.ok("rooms.list", json!({})).await["invites"],
        json!([]),
        "the invitation is spent"
    );
    assert_eq!(
        b.refused("rooms.invite", json!({ "room": room, "qor_id": "carol" }))
            .await,
        "forbidden",
        "members do not invite"
    );
    assert_eq!(
        b.ok("messages.history", json!({ "room": room })).await["messages"][0]["body"],
        "secret plans"
    );

    // An owner hands the room on before leaving; a moderator removes members, not other moderators.
    assert_eq!(
        a.refused("rooms.leave", json!({ "room": room })).await,
        "forbidden"
    );
    a.ok("rooms.invite", json!({ "room": room, "qor_id": "carol" }))
        .await;
    c.ok("rooms.join", json!({ "room": room })).await;
    a.ok(
        "rooms.role",
        json!({ "room": room, "sub": bob.sub, "role": "owner" }),
    )
    .await;
    let members = b.ok("rooms.members", json!({ "room": room })).await["members"].clone();
    assert_eq!(members[0]["sub"], bob.sub.to_string());
    assert_eq!(members[0]["role"], "owner");
    assert_eq!(members[1]["role"], "moderator", "the old owner moderates");
    assert_eq!(
        a.refused("rooms.remove", json!({ "room": room, "sub": bob.sub }))
            .await,
        "forbidden"
    );
    a.ok("rooms.remove", json!({ "room": room, "sub": carol.sub }))
        .await;
    assert_eq!(c.event("member.left").await["sub"], carol.sub.to_string());
    assert_eq!(
        c.refused("messages.history", json!({ "room": room })).await,
        "not_found"
    );
    a.ok("rooms.leave", json!({ "room": room })).await;
    assert_eq!(
        b.ok("rooms.leave", json!({ "room": room })).await["closed"],
        true,
        "the last owner closes it"
    );
}

#[sqlx::test]
async fn a_direct_conversation_is_two_people_and_a_block_ends_what_reaches_the_blocker(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;

    let opened = a.ok("dm.open", json!({ "qor_id": "bob" })).await;
    let dm = id(&opened["room"]);
    assert_eq!(opened["other"]["qor_id"], "bob");
    assert_eq!(
        id(&b.ok("dm.open", json!({ "qor_id": "alice" })).await["room"]),
        dm,
        "one pair, one room"
    );
    assert_eq!(
        a.refused("dm.open", json!({ "qor_id": "alice" })).await,
        "bad_request"
    );
    assert_eq!(
        a.refused("rooms.invite", json!({ "room": dm, "qor_id": "bob" }))
            .await,
        "forbidden"
    );
    assert_eq!(
        a.refused("rooms.leave", json!({ "room": dm })).await,
        "forbidden"
    );
    a.ok("messages.send", json!({ "room": dm, "body": "psst" }))
        .await;
    assert_eq!(b.event("message.created").await["message"]["body"], "psst");
    let listed = b.ok("rooms.list", json!({})).await;
    assert_eq!(listed["rooms"][0]["other"], alice.sub.to_string());

    // A shared public room too, to see that a block reaches beyond the conversation.
    let lounge = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Lounge" }),
        )
        .await["room"]);
    b.ok("rooms.join", json!({ "room": lounge })).await;
    b.ok("block", json!({ "sub": alice.sub })).await;
    b.forget("message.created").await;
    b.forget("presence").await;
    assert_eq!(
        a.refused("messages.send", json!({ "room": dm, "body": "hello?" }))
            .await,
        "forbidden"
    );
    assert_eq!(
        b.refused("messages.send", json!({ "room": dm, "body": "no" }))
            .await,
        "forbidden",
        "either way"
    );
    a.ok(
        "messages.send",
        json!({ "room": lounge, "body": "anyone?" }),
    )
    .await;
    b.none("message.created").await;
    let history = b.ok("messages.history", json!({ "room": dm })).await;
    assert_eq!(
        history["messages"],
        json!([]),
        "the blocked person's words are not shown"
    );
    assert_eq!(
        b.ok("rooms.list", json!({})).await["rooms"]
            .as_array()
            .expect("rooms")
            .iter()
            .map(|r| r["unread"].as_i64().unwrap_or(0))
            .sum::<i64>(),
        0
    );
    a.ok("presence.set", json!({ "state": "away" })).await;
    b.none("presence").await;
    assert_eq!(
        a.refused("dm.open", json!({ "qor_id": "bob" })).await,
        "forbidden"
    );
    assert_eq!(
        b.ok("blocks.list", json!({})).await["blocked"],
        json!([alice.sub])
    );
    b.ok("unblock", json!({ "sub": alice.sub })).await;
    a.ok("messages.send", json!({ "room": dm, "body": "again" }))
        .await;
    assert_eq!(b.event("message.created").await["message"]["body"], "again");
    let _ = bob;
}

#[sqlx::test]
async fn an_account_on_a_made_name_reads_but_does_not_post_until_it_chooses(db: PgPool) {
    let h = start(db).await;
    let (ta, _) = h.person("alice");
    let (tk, _) = h.person_with("key_0a1b2c3d", false, 3600);
    let (mut a, _) = h.connect(&ta).await;
    let (mut k, welcome) = h.connect(&tk).await;
    assert_eq!(welcome["username_chosen"], false);

    let room = id(&a
        .ok("rooms.create", json!({ "kind": "public", "name": "Open" }))
        .await["room"]);
    a.ok("messages.send", json!({ "room": room, "body": "welcome" }))
        .await;
    k.ok("rooms.join", json!({ "room": room })).await;
    assert_eq!(
        k.ok("messages.history", json!({ "room": room })).await["messages"][0]["body"],
        "welcome"
    );
    for (method, params) in [
        ("messages.send", json!({ "room": room, "body": "hi" })),
        ("rooms.create", json!({ "kind": "public", "name": "Mine" })),
        ("dm.open", json!({ "qor_id": "alice" })),
        ("rooms.invite", json!({ "room": room, "qor_id": "alice" })),
        ("profile.update", json!({ "bio": "me" })),
    ] {
        assert_eq!(
            k.refused(method, params).await,
            "name_not_chosen",
            "{method}"
        );
    }
    let told = k
        .call("messages.send", json!({ "room": room, "body": "hi" }))
        .await
        .expect_err("refused");
    assert_eq!(told.message, "choose your QOR ID name before you post");
}

#[sqlx::test]
async fn limits_hold_across_sockets_and_a_retry_is_not_a_second_message(db: PgPool) {
    let h = start(db).await;
    let (ta, _) = h.person("alice");
    let (mut a1, _) = h.connect(&ta).await;
    let (mut a2, _) = h.connect(&ta).await;
    let room = id(&a1
        .ok("rooms.create", json!({ "kind": "public", "name": "Busy" }))
        .await["room"]);

    let first = a1
        .ok(
            "messages.send",
            json!({ "room": room, "body": "once", "nonce": "n-1" }),
        )
        .await;
    let again = a2
        .ok(
            "messages.send",
            json!({ "room": room, "body": "once", "nonce": "n-1" }),
        )
        .await;
    assert_eq!(
        first["message"]["id"], again["message"]["id"],
        "the same nonce, the same message"
    );
    assert_eq!(a1.event("message.created").await["message"]["seq"], 1);
    a1.none("message.created").await;

    // Ten sends in ten seconds per person, shared by both sockets; the two above were two, the retry included.
    for n in 0..8 {
        let socket = if n % 2 == 0 { &mut a1 } else { &mut a2 };
        socket
            .ok(
                "messages.send",
                json!({ "room": room, "body": format!("m{n}") }),
            )
            .await;
    }
    assert_eq!(
        a2.refused(
            "messages.send",
            json!({ "room": room, "body": "one too many" })
        )
        .await,
        "rate_limited"
    );
    assert_eq!(
        a1.refused(
            "messages.send",
            json!({ "room": room, "body": "x".repeat(4001) })
        )
        .await,
        "bad_request"
    );
    assert_eq!(
        a1.refused(
            "messages.send",
            json!({ "room": room, "body": "\u{202E}  " })
        )
        .await,
        "bad_request"
    );
    assert_eq!(
        a1.refused(
            "messages.send",
            json!({ "room": room, "body": "x", "colour": "red" })
        )
        .await,
        "bad_request",
        "unknown fields"
    );
    assert_eq!(a1.refused("rooms.explode", json!({})).await, "bad_request");
}

#[sqlx::test]
async fn words_are_sealed_at_rest_and_open_only_for_members(db: PgPool) {
    let h = start(db.clone()).await;
    let (ta, _) = h.person("alice");
    let (mut a, _) = h.connect(&ta).await;
    let room = id(&a
        .ok("rooms.create", json!({ "kind": "public", "name": "Vault" }))
        .await["room"]);
    let words = "the treasure is under the third pillar";
    a.ok("messages.send", json!({ "room": room, "body": words }))
        .await;

    let (version, sealed): (i16, Vec<u8>) =
        sqlx::query_as("SELECT key_version, sealed FROM messages")
            .fetch_one(&h.db)
            .await
            .expect("a row");
    assert_eq!(version, 1);
    assert!(
        !sealed.windows(8).any(|w| w == &words.as_bytes()[..8]),
        "no plain words in the database"
    );
    let dumped: String = sqlx::query_scalar("SELECT row_to_json(m)::text FROM messages m")
        .fetch_one(&h.db)
        .await
        .expect("a row");
    assert!(!dumped.contains("treasure"), "{dumped}");
    assert_eq!(
        a.ok("messages.history", json!({ "room": room })).await["messages"][0]["body"],
        words
    );
}

#[sqlx::test]
async fn a_profile_is_shaped_by_its_person_within_the_rules_and_its_contacts_see_it(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, _) = h.person("bob");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let room = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Gallery" }),
        )
        .await["room"]);
    b.ok("rooms.join", json!({ "room": room })).await;

    let profile = a
        .ok(
            "profile.update",
            json!({
                "display": "Alice of the Aurora",
                "bio": "I build worlds.\nAnd break them.",
                "accent": "#7C5CFF",
                "banner": { "preset": "aurora", "hue": 260, "intensity": 0.8 },
                "showcase": ["3/1", "3/1", "0012/7"],
                "status_line": "building a nebula",
            }),
        )
        .await["profile"]
        .clone();
    assert_eq!(profile["accent"], "#7c5cff");
    assert_eq!(
        profile["showcase"],
        json!(["3/1", "12/7"]),
        "normalised, once each"
    );
    assert_eq!(profile["banner"]["preset"], "aurora");
    let seen = b.event("profile.updated").await["profile"].clone();
    assert_eq!(seen["display"], "Alice of the Aurora");
    let card = b.ok("profile.get", json!({ "qor_id": "ALICE" })).await["profiles"]
        [alice.sub.to_string()]
    .clone();
    assert_eq!(card["bio"], "I build worlds.\nAnd break them.");
    assert_eq!(card["presence"]["state"], "online");

    for bad in [
        json!({ "accent": "purple" }),
        json!({ "banner": { "preset": "fire" } }),
        json!({ "showcase": ["1/1", "1/2", "1/3", "1/4", "1/5", "1/6", "1/7"] }),
        json!({ "display": "x".repeat(49) }),
        json!({ "qor_id": "mallory" }),
    ] {
        assert_eq!(
            a.refused("profile.update", bad.clone()).await,
            "bad_request",
            "{bad}"
        );
    }
    let cleared = a.ok("profile.update", json!({ "accent": null })).await["profile"].clone();
    assert_eq!(cleared["accent"], Value::Null);
    assert_eq!(
        cleared["display"], "Alice of the Aurora",
        "what is not named stays"
    );
}

#[sqlx::test]
async fn presence_follows_people_in_and_out_and_reports_are_kept(db: PgPool) {
    let h = start(db.clone()).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (mut a, _) = h.connect(&ta).await;
    let room = id(&a
        .ok("rooms.create", json!({ "kind": "public", "name": "Hall" }))
        .await["room"]);
    a.forget("presence").await;
    let (mut b, _) = h.connect(&tb).await;
    b.ok("rooms.join", json!({ "room": room })).await;
    let p = a.event("presence").await;
    assert_eq!(
        (p["sub"].clone(), p["presence"]["state"].clone()),
        (json!(bob.sub), json!("online"))
    );

    b.ok(
        "presence.set",
        json!({ "state": "away", "playing": "Rift Reversi" }),
    )
    .await;
    let p = a.event("presence").await;
    assert_eq!(
        p["presence"],
        json!({ "state": "away", "playing": "Rift Reversi" })
    );
    b.ok("typing", json!({ "room": room })).await;
    assert_eq!(a.event("typing").await["sub"], bob.sub.to_string());

    let said = id(&a
        .ok(
            "messages.send",
            json!({ "room": room, "body": "buy cheap gold" }),
        )
        .await["message"]);
    b.ok(
        "report",
        json!({ "message": said, "reason": "spam", "note": "obvious" }),
    )
    .await;
    b.ok("report", json!({ "message": said, "reason": "spam" }))
        .await;
    assert_eq!(
        b.refused("report", json!({ "message": said, "reason": "rude" }))
            .await,
        "bad_request"
    );
    assert_eq!(
        a.refused("report", json!({ "message": said, "reason": "spam" }))
            .await,
        "bad_request",
        "not your own"
    );
    let reports: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reports")
        .fetch_one(&h.db)
        .await
        .expect("count");
    assert_eq!(reports, 1, "once per reporter");

    drop(b);
    let p = a.event("presence").await;
    assert_eq!(
        (p["sub"].clone(), p["presence"]["state"].clone()),
        (json!(bob.sub), json!("offline"))
    );
    let _ = alice;
}

#[sqlx::test]
async fn a_socket_closes_when_its_token_expires(db: PgPool) {
    let h = start(db).await;
    let (token, _) = h.person_with("brief", true, 1);
    let (mut c, welcome) = h.connect(&token).await;
    assert_eq!(welcome["type"], "welcome");
    let expired = loop {
        let frame = c.frame().await.expect("a frame before it closes");
        if frame["type"] == "err" {
            break frame;
        }
    };
    assert_eq!(expired["code"], "token_expired", "{expired}");
    assert!(c.frame().await.is_none(), "and then it closes");
}

#[sqlx::test]
async fn a_mention_reaches_the_member_named_and_nobody_outside_the_room(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, _) = h.person("bob");
    let (tc, _) = h.person("carol");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (mut c, _) = h.connect(&tc).await;
    let room = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Callout" }),
        )
        .await["room"]);
    b.ok("rooms.join", json!({ "room": room })).await;
    // Carol is in a room of her own, so a mention that looked beyond this room would find her.
    c.ok(
        "rooms.create",
        json!({ "kind": "public", "name": "Elsewhere" }),
    )
    .await;

    let said = a
        .ok(
            "messages.send",
            json!({ "room": room, "body": "hey @BOB and @carol, mail me at alice@bob.example, and @alice" }),
        )
        .await["message"]
        .clone();
    let mention = b.event("mention").await;
    assert_eq!(mention["message"], said["id"]);
    assert_eq!(mention["by"], alice.sub.to_string());
    c.none("mention").await;
    a.none("mention").await;
}

#[sqlx::test]
async fn ready_answers_only_while_the_database_does(db: PgPool) {
    let h = start(db.clone()).await;
    let base = h.url.replace("ws://", "http://").replace("/v1/socket", "");
    let status = |path: &'static str| {
        let url = format!("{base}{path}");
        async move {
            let (mut socket, _) = tokio::net::TcpStream::connect(
                url.trim_start_matches("http://")
                    .split('/')
                    .next()
                    .expect("host"),
            )
            .await
            .map(|s| (s, ()))
            .expect("connects");
            use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};
            socket
                .write_all(
                    format!(
                        "GET {path} HTTP/1.1
Host: x
Connection: close

"
                    )
                    .as_bytes(),
                )
                .await
                .expect("sent");
            let mut answer = String::new();
            socket.read_to_string(&mut answer).await.expect("read");
            answer.split(' ').nth(1).unwrap_or_default().to_string()
        }
    };
    assert_eq!(status("/ready").await, "200");
    assert_eq!(status("/health").await, "200");
    db.close().await;
    assert_eq!(
        status("/ready").await,
        "503",
        "a closed database is not ready"
    );
    assert_eq!(
        status("/health").await,
        "200",
        "which /health would not say"
    );
}
