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
        store: Store::new(db.clone(), Sealer::random())
            .with_world_moderators(["Warden".into()])
            .with_godmode("godmode"),
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
    // A newcomer is in World, and nothing else, with nothing unread there.
    let rooms = welcome["rooms"].as_array().expect("rooms");
    assert_eq!(rooms.len(), 1, "{welcome}");
    assert_eq!(rooms[0]["kind"], "world");
    assert_eq!(rooms[0]["name"], "World");
    assert_eq!(rooms[0]["unread"], 0);
    assert_eq!(welcome["world"]["id"], rooms[0]["id"]);
    assert_eq!(welcome["world"]["muted_until"], Value::Null);
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
    // A moderator acts only below their own level (ADR-093): the owner's words are the owner's to delete.
    assert_eq!(
        b.refused("messages.delete", json!({ "message": id(&sent) }))
            .await,
        "forbidden"
    );
    a.ok("messages.delete", json!({ "message": id(&sent) }))
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
    assert_eq!(
        members[1]["role"], "admin",
        "the old owner is an admin (ADR-093)"
    );
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

// ── ADR-092: World, friends and circles, and the language of what is said ───────────────────────────────────────

fn world_of(welcome: &Value) -> String {
    welcome["world"]["id"]
        .as_str()
        .expect("World's id")
        .to_string()
}

#[sqlx::test]
async fn world_is_everyones_and_a_newcomer_starts_at_its_newest_message(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (mut a, welcome) = h.connect(&ta).await;
    let world = world_of(&welcome);

    for n in 0..3 {
        a.ok(
            "messages.send",
            json!({ "room": world, "body": format!("before bob {n}"), "lang": "en" }),
        )
        .await;
    }
    let (mut b, welcome) = h.connect(&tb).await;
    assert_eq!(world_of(&welcome), world, "one World");
    let mine = &welcome["rooms"][0];
    assert_eq!(mine["kind"], "world");
    assert_eq!(mine["unread"], 0, "a newcomer starts at the newest message");
    assert_eq!(mine["last_seq"], 3);
    let history = b.ok("messages.history", json!({ "room": world })).await;
    assert_eq!(
        history["messages"].as_array().expect("messages").len(),
        3,
        "and reads what came before"
    );
    assert_eq!(history["messages"][0]["lang"], "en");

    // What is said there reaches whoever is connected (alice heard her own three; those are not what is awaited).
    a.forget("message.created").await;
    b.ok(
        "messages.send",
        json!({ "room": world, "body": "hello world" }),
    )
    .await;
    let heard = a.event("message.created").await;
    assert_eq!(heard["message"]["body"], "hello world");
    assert_eq!(
        heard["message"]["lang"],
        Value::Null,
        "no language given, none kept"
    );

    // It is not left, changed, invited to, or found among public rooms; joining it again changes nothing.
    assert_eq!(
        a.refused("rooms.leave", json!({ "room": world })).await,
        "forbidden"
    );
    a.ok("rooms.join", json!({ "room": world })).await;
    assert_eq!(
        a.refused("rooms.update", json!({ "room": world, "topic": "mine" }))
            .await,
        "forbidden"
    );
    assert_eq!(
        a.refused("rooms.invite", json!({ "room": world, "qor_id": "bob" }))
            .await,
        "forbidden"
    );
    assert_eq!(
        a.refused("rooms.remove", json!({ "room": world, "sub": bob.sub }))
            .await,
        "forbidden"
    );
    assert_eq!(
        a.refused(
            "rooms.role",
            json!({ "room": world, "sub": bob.sub, "role": "moderator" })
        )
        .await,
        "forbidden"
    );
    let found = b.ok("rooms.discover", json!({})).await;
    assert!(
        !found["rooms"].to_string().contains(&world),
        "World is not a public room to find"
    );

    // World makes nobody a contact: presence and typing do not cross it.
    a.forget("presence").await;
    b.ok("presence.set", json!({ "state": "away" })).await;
    a.none("presence").await;
    b.ok("typing", json!({ "room": world })).await;
    a.none("typing").await;

    // Who is here: the people connected and showing, not the invisible.
    let here = a.ok("rooms.members", json!({ "room": world })).await;
    let listed = here["members"].to_string();
    assert!(listed.contains(&alice.sub.to_string()) && listed.contains(&bob.sub.to_string()));
    b.ok("presence.set", json!({ "state": "invisible" })).await;
    let here = a.ok("rooms.members", json!({ "room": world })).await;
    assert!(
        !here.to_string().contains(&bob.sub.to_string()),
        "invisible is not listed"
    );

    // A block holds in World as everywhere: the blocker does not hear the blocked.
    a.ok("block", json!({ "sub": bob.sub })).await;
    a.forget("message.created").await;
    b.ok(
        "messages.send",
        json!({ "room": world, "body": "can you hear me" }),
    )
    .await;
    a.none("message.created").await;
}

#[sqlx::test]
async fn world_moves_at_its_own_pace(db: PgPool) {
    let h = start(db).await;
    let (ta, _) = h.person("alice");
    let (mut a, welcome) = h.connect(&ta).await;
    let world = world_of(&welcome);
    // Five in twenty seconds, on top of the ordinary limit.
    for n in 0..5 {
        a.ok(
            "messages.send",
            json!({ "room": world, "body": format!("w{n}") }),
        )
        .await;
    }
    assert_eq!(
        a.refused(
            "messages.send",
            json!({ "room": world, "body": "one too many" })
        )
        .await,
        "rate_limited"
    );
    // A room of one's own keeps the ordinary pace.
    let room = id(&a
        .ok("rooms.create", json!({ "kind": "public", "name": "Mine" }))
        .await["room"]);
    a.ok(
        "messages.send",
        json!({ "room": room, "body": "still fine" }),
    )
    .await;
}

#[sqlx::test]
async fn worlds_moderators_delete_and_mute_and_nobody_else_does(db: PgPool) {
    let h = start(db).await;
    let (tw, warden) = h.person("warden");
    let (ta, alice) = h.person("alice");
    let (tb, _) = h.person("bob");
    let (tk, _) = h.person_with("warden", false, 3600);
    let (mut w, welcome) = h.connect(&tw).await;
    let world = world_of(&welcome);
    assert_eq!(
        welcome["rooms"][0]["role"], "moderator",
        "named in the configuration, in any case"
    );
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (_k, welcome) = h.connect(&tk).await;
    assert_eq!(
        welcome["rooms"][0]["role"], "member",
        "a name its holder has not chosen never moderates"
    );

    let said = id(&a
        .ok(
            "messages.send",
            json!({ "room": world, "body": "rude words" }),
        )
        .await["message"]);
    let warden_said = id(&w
        .ok("messages.send", json!({ "room": world, "body": "be kind" }))
        .await["message"]);
    assert_eq!(
        b.refused("messages.delete", json!({ "message": said }))
            .await,
        "forbidden",
        "a member deletes only their own"
    );
    assert_eq!(
        a.refused("messages.delete", json!({ "message": warden_said }))
            .await,
        "forbidden"
    );
    w.ok("messages.delete", json!({ "message": said })).await;

    // Only a moderator mutes; a moderator is not muted; a mute lasts one of four lengths.
    assert_eq!(
        b.refused("world.mute", json!({ "sub": alice.sub, "minutes": 10 }))
            .await,
        "forbidden"
    );
    assert_eq!(
        a.refused("world.mute", json!({ "sub": warden.sub, "minutes": 10 }))
            .await,
        "forbidden"
    );
    assert_eq!(
        w.refused("world.mute", json!({ "sub": alice.sub, "minutes": 7 }))
            .await,
        "bad_request"
    );
    assert_eq!(
        w.refused("world.mute", json!({ "sub": warden.sub, "minutes": 10 }))
            .await,
        "forbidden"
    );
    let until = w
        .ok("world.mute", json!({ "sub": alice.sub, "minutes": 60 }))
        .await["until"]
        .clone();
    assert!(until.is_string(), "{until}");
    assert_eq!(a.event("world.muted").await["until"], until);
    let told = a
        .call(
            "messages.send",
            json!({ "room": world, "body": "let me speak" }),
        )
        .await
        .expect_err("muted");
    assert_eq!(told.code, "forbidden");
    assert!(
        told.message.contains("muted you here until"),
        "{}",
        told.message
    );
    // Muted in World only: a room of their own still carries their words.
    let room = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Elsewhere" }),
        )
        .await["room"]);
    a.ok(
        "messages.send",
        json!({ "room": room, "body": "here I may" }),
    )
    .await;
    // Saying hello again shows the mute.
    let (_again, welcome) = h.connect(&ta).await;
    assert!(welcome["world"]["muted_until"].is_string(), "{welcome}");

    w.ok("world.unmute", json!({ "sub": alice.sub })).await;
    assert_eq!(a.event("world.muted").await["until"], Value::Null);
    a.ok(
        "messages.send",
        json!({ "room": world, "body": "thank you" }),
    )
    .await;
}

#[sqlx::test]
async fn friends_ask_and_answer_and_friends_see_each_others_presence(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (tc, _) = h.person("carol");
    let (td, dave) = h.person("dave");
    let (tk, _) = h.person_with("key_00aa11bb", false, 3600);
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (mut c, _) = h.connect(&tc).await;
    let (mut d, _) = h.connect(&td).await;
    let (mut k, _) = h.connect(&tk).await;

    assert_eq!(
        a.refused("friends.request", json!({ "qor_id": "alice" }))
            .await,
        "bad_request"
    );
    assert_eq!(
        a.refused("friends.request", json!({ "qor_id": "nobody" }))
            .await,
        "not_found"
    );
    assert_eq!(
        k.refused("friends.request", json!({ "qor_id": "alice" }))
            .await,
        "name_not_chosen"
    );

    // A request, seen by both; answered by the one asked.
    let asked = a.ok("friends.request", json!({ "qor_id": "BOB" })).await;
    assert_eq!(asked["friends"], false);
    let told = b.event("friend.requested").await;
    assert_eq!(told["from"], alice.sub.to_string());
    assert_eq!(told["profiles"][alice.sub.to_string()]["qor_id"], "alice");
    a.event("friend.requested").await;
    a.ok("friends.request", json!({ "qor_id": "bob" })).await;
    let list = b.ok("friends.list", json!({})).await;
    assert_eq!(
        list["incoming"].as_array().expect("incoming").len(),
        1,
        "asking twice is one request"
    );
    assert_eq!(list["incoming"][0]["sub"], alice.sub.to_string());
    assert_eq!(list["profiles"][alice.sub.to_string()]["qor_id"], "alice");
    assert_eq!(
        a.ok("friends.list", json!({})).await["outgoing"][0]["sub"],
        bob.sub.to_string()
    );
    assert_eq!(
        a.refused("friends.accept", json!({ "sub": bob.sub })).await,
        "not_found",
        "the one who asked does not answer"
    );
    b.ok("friends.accept", json!({ "sub": alice.sub })).await;
    let accepted = a.event("friend.accepted").await;
    assert!(
        accepted["between"]
            .to_string()
            .contains(&bob.sub.to_string()),
        "{accepted}"
    );
    assert_eq!(
        a.ok("friends.list", json!({})).await["friends"],
        json!([bob.sub])
    );
    assert_eq!(
        a.refused("friends.request", json!({ "qor_id": "bob" }))
            .await,
        "conflict",
        "friends already"
    );

    // Friends see each other's presence, though they share no room but World.
    a.forget("presence").await;
    b.ok(
        "presence.set",
        json!({ "state": "away", "playing": "Flux Four" }),
    )
    .await;
    let seen = a.event("presence").await;
    assert_eq!(seen["sub"], bob.sub.to_string());
    assert_eq!(seen["presence"]["state"], "away");

    // Two who ask each other are friends at once.
    c.ok("friends.request", json!({ "qor_id": "alice" })).await;
    assert_eq!(
        a.ok("friends.request", json!({ "qor_id": "carol" })).await["friends"],
        true
    );
    c.event("friend.accepted").await;

    // Declined by the one asked, or taken back by the one who asked: both hear it ended.
    d.ok("friends.request", json!({ "qor_id": "alice" })).await;
    a.ok("friends.decline", json!({ "sub": dave.sub })).await;
    assert_eq!(
        d.event("friend.declined").await["to"],
        alice.sub.to_string()
    );
    d.ok("friends.request", json!({ "qor_id": "alice" })).await;
    d.ok("friends.cancel", json!({ "sub": alice.sub })).await;
    assert_eq!(a.ok("friends.list", json!({})).await["incoming"], json!([]));
    assert_eq!(
        a.refused("friends.decline", json!({ "sub": dave.sub }))
            .await,
        "not_found"
    );

    // Ending a friendship ends the presence that came with it.
    a.forget("presence").await;
    a.ok("friends.remove", json!({ "sub": bob.sub })).await;
    b.event("friend.removed").await;
    let gone = a.event("presence").await;
    assert_eq!(gone["sub"], bob.sub.to_string());
    assert_eq!(gone["presence"]["state"], "offline");
    b.ok("presence.set", json!({ "state": "online" })).await;
    a.none("presence").await;
    assert_eq!(
        a.refused("friends.remove", json!({ "sub": bob.sub })).await,
        "not_found"
    );

    // A block ends a friendship and refuses a new one, either way.
    let carol = h.fake.people.lock().expect("lock")[&tc].sub;
    c.ok("block", json!({ "sub": alice.sub })).await;
    assert!(
        !a.ok("friends.list", json!({})).await["friends"]
            .to_string()
            .contains(&carol.to_string())
    );
    assert_eq!(
        a.refused("friends.request", json!({ "qor_id": "carol" }))
            .await,
        "forbidden"
    );
    assert_eq!(
        c.refused("friends.request", json!({ "qor_id": "alice" }))
            .await,
        "forbidden"
    );
}

#[sqlx::test]
async fn circles_are_their_owners_alone_and_open_a_private_room(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (tc, carol) = h.person("carol");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (mut c, _) = h.connect(&tc).await;
    a.ok("friends.request", json!({ "qor_id": "bob" })).await;
    b.ok("friends.accept", json!({ "sub": alice.sub })).await;

    assert_eq!(
        a.refused("circles.create", json!({ "name": "  " })).await,
        "bad_request"
    );
    assert_eq!(
        a.refused(
            "circles.create",
            json!({ "name": "Squad", "color": "teal" })
        )
        .await,
        "bad_request"
    );
    let circle = a
        .ok(
            "circles.create",
            json!({ "name": "  Squad ", "color": "#AABBCC" }),
        )
        .await["circle"]
        .clone();
    assert_eq!(circle["name"], "Squad");
    assert_eq!(circle["color"], "#aabbcc");
    assert_eq!(circle["members"], json!([]));
    let circle_id = id(&circle);

    assert_eq!(
        a.refused("circles.open", json!({ "circle": circle_id }))
            .await,
        "bad_request",
        "an empty circle opens nothing"
    );
    let with_bob = a
        .ok(
            "circles.add",
            json!({ "circle": circle_id, "sub": bob.sub }),
        )
        .await;
    assert_eq!(with_bob["circle"]["members"], json!([bob.sub]));
    assert_eq!(
        a.refused(
            "circles.add",
            json!({ "circle": circle_id, "sub": carol.sub })
        )
        .await,
        "forbidden",
        "only friends"
    );

    // Nobody else sees it, or touches it.
    assert_eq!(b.ok("circles.list", json!({})).await["circles"], json!([]));
    assert_eq!(
        b.refused(
            "circles.update",
            json!({ "circle": circle_id, "name": "Mine" })
        )
        .await,
        "not_found"
    );
    assert_eq!(
        c.refused("circles.delete", json!({ "circle": circle_id }))
            .await,
        "not_found"
    );
    let renamed = a
        .ok(
            "circles.update",
            json!({ "circle": circle_id, "name": "Crew", "color": null }),
        )
        .await;
    assert_eq!(renamed["circle"]["name"], "Crew");
    assert_eq!(renamed["circle"]["color"], Value::Null);

    // One step from a circle to a private room, its friends invited.
    let room = a.ok("circles.open", json!({ "circle": circle_id })).await["room"].clone();
    assert_eq!(room["kind"], "private");
    assert_eq!(room["name"], "Crew");
    let invite = b.event("invite").await;
    assert_eq!(invite["room"], room["id"]);
    b.ok("rooms.join", json!({ "room": room["id"] })).await;

    // Ending the friendship takes them out of the circle.
    b.ok("friends.remove", json!({ "sub": alice.sub })).await;
    assert_eq!(
        a.ok("circles.list", json!({})).await["circles"][0]["members"],
        json!([])
    );

    // Twenty circles at most.
    for n in 1..20 {
        a.ok("circles.create", json!({ "name": format!("c{n}") }))
            .await;
    }
    assert_eq!(
        a.refused("circles.create", json!({ "name": "one more" }))
            .await,
        "conflict"
    );
    a.ok("circles.delete", json!({ "circle": circle_id })).await;
    a.ok("circles.create", json!({ "name": "room again" }))
        .await;
}

#[sqlx::test]
async fn a_message_keeps_its_language_and_a_profile_its_readers(db: PgPool) {
    let h = start(db).await;
    let (ta, _) = h.person("alice");
    let (mut a, welcome) = h.connect(&ta).await;
    let world = world_of(&welcome);
    assert_eq!(welcome["me"]["language"], Value::Null);

    let sent = a
        .ok(
            "messages.send",
            json!({ "room": world, "body": "olá a todos", "lang": "pt-br" }),
        )
        .await["message"]
        .clone();
    assert_eq!(sent["lang"], "pt-BR", "kept in its usual case");
    assert_eq!(
        a.refused(
            "messages.send",
            json!({ "room": world, "body": "hi", "lang": "english" })
        )
        .await,
        "bad_request"
    );
    let edited = a
        .ok(
            "messages.edit",
            json!({ "message": sent["id"], "body": "hello everyone", "lang": "en" }),
        )
        .await;
    assert_eq!(edited["message"]["lang"], "en");
    let kept = a
        .ok(
            "messages.edit",
            json!({ "message": sent["id"], "body": "hello all" }),
        )
        .await;
    assert_eq!(
        kept["message"]["lang"], "en",
        "an edit without a language keeps the one it had"
    );

    let profile = a.ok("profile.update", json!({ "language": "JA" })).await;
    assert_eq!(profile["profile"]["language"], "ja");
    assert_eq!(
        a.refused("profile.update", json!({ "language": "klingon!" }))
            .await,
        "bad_request"
    );
    let (_, again) = h.connect(&ta).await;
    assert_eq!(again["me"]["language"], "ja", "it follows them");
}

// ── ADR-093: room levels, Elite accounts and godmode ───────────────────────────────────────────────────────────

#[sqlx::test]
async fn levels_climb_from_member_to_owner_and_each_acts_only_below_itself(db: PgPool) {
    let h = start(db).await;
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (tc, carol) = h.person("carol");
    let (td, dave) = h.person("dave");
    let (te, erin) = h.person("erin");
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (mut c, _) = h.connect(&tc).await;
    let (mut d, _) = h.connect(&td).await;
    let (mut e, _) = h.connect(&te).await;
    let room = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Ladder" }),
        )
        .await["room"]);
    for client in [&mut b, &mut c, &mut d, &mut e] {
        client.ok("rooms.join", json!({ "room": room })).await;
    }
    let set = |sub: Uuid, role: &str| json!({ "room": room, "sub": sub, "role": role });

    assert_eq!(
        a.refused("rooms.role", set(bob.sub, "captain")).await,
        "bad_request"
    );
    assert_eq!(
        d.refused("rooms.role", set(erin.sub, "vip")).await,
        "forbidden",
        "a member sets no levels"
    );
    a.ok("rooms.role", set(bob.sub, "admin")).await;
    b.ok("rooms.role", set(carol.sub, "moderator")).await;
    b.ok("rooms.role", set(dave.sub, "vip")).await;
    assert_eq!(
        b.refused("rooms.role", set(erin.sub, "admin")).await,
        "forbidden",
        "an admin gives only levels below their own"
    );
    assert_eq!(
        b.refused("rooms.role", set(erin.sub, "owner")).await,
        "forbidden",
        "only the owner hands the room on"
    );
    assert_eq!(
        b.refused("rooms.role", set(alice.sub, "member")).await,
        "forbidden",
        "nobody acts on someone at or above them"
    );
    assert_eq!(
        c.refused("rooms.role", set(erin.sub, "vip")).await,
        "forbidden",
        "a moderator sets no levels"
    );

    // Settings are an admin's; a moderator's tools are removing, muting and deleting below them.
    b.ok("rooms.update", json!({ "room": room, "topic": "climb" }))
        .await;
    assert_eq!(
        c.refused("rooms.update", json!({ "room": room, "topic": "mine" }))
            .await,
        "forbidden"
    );
    let vip_said = id(&d
        .ok(
            "messages.send",
            json!({ "room": room, "body": "vip words" }),
        )
        .await["message"]);
    let admin_said = id(&b
        .ok(
            "messages.send",
            json!({ "room": room, "body": "admin words" }),
        )
        .await["message"]);
    c.ok("messages.delete", json!({ "message": vip_said }))
        .await;
    assert_eq!(
        c.refused("messages.delete", json!({ "message": admin_said }))
            .await,
        "forbidden"
    );
    c.ok(
        "rooms.mute",
        json!({ "room": room, "sub": dave.sub, "minutes": 10 }),
    )
    .await;
    assert_eq!(d.event("room.muted").await["room"], room);
    assert_eq!(
        d.refused("messages.send", json!({ "room": room, "body": "hello?" }))
            .await,
        "forbidden"
    );
    let (_again, welcome) = h.connect(&td).await;
    let listed = welcome["rooms"]
        .as_array()
        .expect("rooms")
        .iter()
        .find(|r| r["id"] == room.as_str())
        .expect("the room");
    assert!(listed["muted_until"].is_string(), "the room list says so");
    assert_eq!(
        c.refused(
            "rooms.mute",
            json!({ "room": room, "sub": bob.sub, "minutes": 10 })
        )
        .await,
        "forbidden"
    );
    assert_eq!(
        c.refused("rooms.remove", json!({ "room": room, "sub": bob.sub }))
            .await,
        "forbidden"
    );
    c.ok("rooms.remove", json!({ "room": room, "sub": erin.sub }))
        .await;

    // Badges: history says who holds which level, and a new message carries its author's.
    let history = a.ok("messages.history", json!({ "room": room })).await;
    assert_eq!(history["levels"][bob.sub.to_string()], "admin");
    assert!(
        history["levels"].get(dave.sub.to_string()).is_none()
            || history["levels"][dave.sub.to_string()] == "vip"
    );
    a.forget("message.created").await;
    b.ok("messages.send", json!({ "room": room, "body": "badge" }))
        .await;
    assert_eq!(a.event("message.created").await["level"], "admin");

    // The owner hands on and becomes an admin.
    a.ok("rooms.role", set(bob.sub, "owner")).await;
    let members = a.ok("rooms.members", json!({ "room": room })).await["members"].clone();
    assert_eq!(members[0]["sub"], bob.sub.to_string());
    assert_eq!(members[1]["role"], "admin");
    let _ = (carol, erin);
}

#[sqlx::test]
async fn godmode_stands_above_every_room_and_alone_makes_elite(db: PgPool) {
    let h = start(db).await;
    let (tg, god) = h.person("godmode");
    let (ta, alice) = h.person("alice");
    let (tb, bob) = h.person("bob");
    let (tv, eve) = h.person("eve");
    let (mut g, welcome) = h.connect(&tg).await;
    assert_eq!(welcome["me"]["status"], "godmode");
    let world = world_of(&welcome);
    let (mut a, _) = h.connect(&ta).await;
    let (mut b, _) = h.connect(&tb).await;
    let (mut v, _) = h.connect(&tv).await;
    let room = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "public", "name": "Alice's" }),
        )
        .await["room"]);
    for client in [&mut g, &mut b, &mut v] {
        client.ok("rooms.join", json!({ "room": room })).await;
    }
    let secret = id(&a
        .ok(
            "rooms.create",
            json!({ "kind": "private", "name": "Secret" }),
        )
        .await["room"]);

    // Only godmode makes an account Elite.
    assert_eq!(
        a.refused("elite.grant", json!({ "qor_id": "eve" })).await,
        "forbidden"
    );
    let granted = g.ok("elite.grant", json!({ "qor_id": "eve" })).await;
    assert_eq!(granted["profile"]["status"], "elite");
    assert_eq!(
        v.event("profile.updated").await["profile"]["status"],
        "elite"
    );
    assert_eq!(
        v.refused("elite.grant", json!({ "qor_id": "bob" })).await,
        "forbidden",
        "an Elite makes no Elite"
    );
    assert_eq!(a.refused("elite.list", json!({})).await, "forbidden");
    assert_eq!(
        g.ok("elite.list", json!({})).await["elites"],
        json!([eve.sub])
    );

    // An Elite acts as an admin in a room it is in, never on its owner; the owner outranks it.
    v.ok(
        "rooms.role",
        json!({ "room": room, "sub": bob.sub, "role": "moderator" }),
    )
    .await;
    v.ok(
        "rooms.mute",
        json!({ "room": room, "sub": bob.sub, "minutes": 10 }),
    )
    .await;
    assert_eq!(
        v.refused(
            "rooms.role",
            json!({ "room": room, "sub": alice.sub, "role": "member" })
        )
        .await,
        "forbidden"
    );
    assert_eq!(
        v.refused("rooms.remove", json!({ "room": room, "sub": alice.sub }))
            .await,
        "forbidden"
    );
    a.ok(
        "rooms.mute",
        json!({ "room": room, "sub": eve.sub, "minutes": 10 }),
    )
    .await;
    a.ok("rooms.unmute", json!({ "room": room, "sub": eve.sub }))
        .await;
    // Nor does any level reach into a room it is not in.
    assert_eq!(
        v.refused("messages.history", json!({ "room": secret }))
            .await,
        "not_found"
    );
    assert_eq!(
        g.refused("messages.history", json!({ "room": secret }))
            .await,
        "not_found",
        "not godmode either: a private room's words are its members'"
    );

    // Nobody acts on godmode; godmode acts on anyone, and hands any room on.
    for (method, params) in [
        (
            "rooms.mute",
            json!({ "room": room, "sub": god.sub, "minutes": 10 }),
        ),
        ("rooms.remove", json!({ "room": room, "sub": god.sub })),
        (
            "rooms.role",
            json!({ "room": room, "sub": god.sub, "role": "member" }),
        ),
    ] {
        assert_eq!(a.refused(method, params).await, "forbidden", "{method}");
    }
    g.ok(
        "rooms.role",
        json!({ "room": room, "sub": bob.sub, "role": "owner" }),
    )
    .await;
    let members = a.ok("rooms.members", json!({ "room": room })).await["members"].clone();
    assert_eq!(members[0]["sub"], bob.sub.to_string());
    assert!(
        members
            .as_array()
            .expect("members")
            .iter()
            .any(|m| m["sub"] == alice.sub.to_string() && m["role"] == "admin"),
        "the owner godmode replaced is an admin: {members}"
    );

    // In World, an Elite sets levels; a VIP talks without World's pace.
    v.ok(
        "rooms.role",
        json!({ "room": world, "sub": alice.sub, "role": "vip" }),
    )
    .await;
    for n in 0..7 {
        a.ok(
            "messages.send",
            json!({ "room": world, "body": format!("vip {n}") }),
        )
        .await;
    }
    assert_eq!(
        v.refused(
            "rooms.role",
            json!({ "room": world, "sub": bob.sub, "role": "owner" })
        )
        .await,
        "forbidden",
        "World has no owner"
    );

    // Taken away, Elite is gone with its powers.
    g.ok("elite.revoke", json!({ "sub": eve.sub })).await;
    let shown = a
        .ok("profile.get", json!({ "subs": [eve.sub, god.sub] }))
        .await;
    assert_eq!(
        shown["profiles"][eve.sub.to_string()]["status"],
        Value::Null
    );
    assert_eq!(shown["profiles"][god.sub.to_string()]["status"], "godmode");
    assert_eq!(
        v.refused(
            "rooms.role",
            json!({ "room": world, "sub": alice.sub, "role": "member" })
        )
        .await,
        "forbidden"
    );
}

#[sqlx::test]
async fn a_level_appointed_in_world_outlasts_the_configured_moderators(db: PgPool) {
    let h = start(db).await;
    let (tg, _) = h.person("godmode");
    let (tw, warden) = h.person("warden");
    let (mut g, welcome) = h.connect(&tg).await;
    let world = world_of(&welcome);
    let (_w, welcome) = h.connect(&tw).await;
    assert_eq!(welcome["rooms"][0]["role"], "moderator");
    g.ok(
        "rooms.role",
        json!({ "room": world, "sub": warden.sub, "role": "admin" }),
    )
    .await;
    let (_again, welcome) = h.connect(&tw).await;
    assert_eq!(
        welcome["rooms"][0]["role"], "admin",
        "saying hello again does not undo it"
    );
}
