//! What a client may ask, and what it causes (docs/blueprints/social.md section 2).
//!
//! Each method is a typed intent: its parameters are read into a struct that refuses unknown fields, the store checks
//! the rule, and the events it causes go to exactly the people who should see them. Nothing here logs a parameter, a
//! body or a token.

use std::sync::Arc;

use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::{SocialError, SocialResult};
use crate::hub::event;
use crate::identity::Person;
use crate::state::AppState;
use crate::store::{self, ProfileChange, RoomChange};

fn params<T: DeserializeOwned>(value: Value) -> SocialResult<T> {
    let value = if value.is_null() { json!({}) } else { value };
    serde_json::from_value(value).map_err(|e| {
        // serde's message names the field and the problem, never the value.
        SocialError::bad(format!("the request's parameters are not right: {e}"))
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Nothing {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InRoom {
    room: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateRoom {
    kind: String,
    name: String,
    #[serde(default)]
    topic: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateRoom {
    room: Uuid,
    name: Option<String>,
    topic: Option<String>,
    ambience: Option<Value>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Named {
    qor_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomAndName {
    room: Uuid,
    qor_id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomAndPerson {
    room: Uuid,
    sub: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetRole {
    room: Uuid,
    sub: Uuid,
    role: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct History {
    room: Uuid,
    before: Option<i64>,
    limit: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Send {
    room: Uuid,
    body: String,
    reply_to: Option<Uuid>,
    nonce: Option<String>,
    /// The language the words are in, as the sender's launcher found it (ADR-092 decision 3).
    lang: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Edit {
    message: Uuid,
    body: String,
    lang: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OneMessage {
    message: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct React {
    message: Uuid,
    emoji: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Read {
    room: Uuid,
    seq: i64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SetPresence {
    state: String,
    playing: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GetProfiles {
    qor_id: Option<String>,
    subs: Option<Vec<Uuid>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Someone {
    sub: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    message: Uuid,
    reason: String,
    #[serde(default)]
    note: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Mute {
    sub: Uuid,
    minutes: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoomMute {
    room: Uuid,
    sub: Uuid,
    minutes: i32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NewCircle {
    name: String,
    color: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeCircle {
    circle: Uuid,
    name: Option<String>,
    #[serde(default, deserialize_with = "store::present")]
    color: Option<Option<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct InCircle {
    circle: Uuid,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CircleAndPerson {
    circle: Uuid,
    sub: Uuid,
}

/// Who is asking.
pub struct Caller<'a> {
    pub person: &'a Person,
}

impl Caller<'_> {
    fn sub(&self) -> Uuid {
        self.person.sub
    }

    /// An account on a made name reads, but does not post, create, invite or open a conversation until it has chosen
    /// its name (ADR-091 decision 5).
    fn named(&self) -> SocialResult<()> {
        if self.person.username_chosen {
            Ok(())
        } else {
            Err(SocialError::NameNotChosen)
        }
    }
}

async fn profiles_of(
    state: &AppState,
    subs: impl IntoIterator<Item = Uuid>,
) -> SocialResult<Value> {
    let mut subs: Vec<Uuid> = subs.into_iter().collect();
    subs.sort_unstable();
    subs.dedup();
    let mut map = serde_json::Map::new();
    for p in state.store.profiles(&subs).await? {
        let presence = state.hub.presence(p.sub).await;
        let mut value = json!(p);
        value["presence"] = json!(presence);
        map.insert(p.sub.to_string(), value);
    }
    Ok(Value::Object(map))
}

/// Tell `sub`'s contacts, and `sub`'s own sockets, how they appear now.
pub async fn announce_presence(state: &AppState, sub: Uuid) -> SocialResult<()> {
    let presence = state.hub.presence(sub).await;
    let mut to = state.store.contacts(sub).await?;
    to.push(sub);
    state
        .hub
        .send(
            &to,
            &event("presence", json!({ "sub": sub, "presence": presence })),
        )
        .await;
    Ok(())
}

/// Who hears what `author` does in `room`: its members, less those who block the author. World is everyone, so what
/// happens there reaches whoever is connected, less those who block the author (ADR-092 decision 1).
async fn hearers(state: &AppState, room: Uuid, author: Uuid) -> SocialResult<Vec<Uuid>> {
    if room == state.store.world().await? {
        let blockers = state.store.blockers_of(author).await?;
        let mut to = state.hub.connected().await;
        to.retain(|s| !blockers.contains(s));
        Ok(to)
    } else {
        state.store.recipients(room, author).await
    }
}

async fn to_room(
    state: &AppState,
    room: Uuid,
    author: Uuid,
    name: &str,
    data: Value,
) -> SocialResult<()> {
    let to = hearers(state, room, author).await?;
    state.hub.send(&to, &event(name, data)).await;
    Ok(())
}

/// Tell two people about something between them, on all their sockets.
async fn to_both(state: &AppState, a: Uuid, b: Uuid, name: &str, data: Value) {
    state.hub.send(&[a, b], &event(name, data)).await;
}

/// After a friendship ends, each stops seeing the other's presence, unless another room still joins them.
async fn part(state: &AppState, a: Uuid, b: Uuid) -> SocialResult<()> {
    if state.store.contacts(a).await?.contains(&b) {
        return Ok(());
    }
    let offline = json!({ "state": "offline", "playing": null });
    state
        .hub
        .send(
            &[a],
            &event("presence", json!({ "sub": b, "presence": offline })),
        )
        .await;
    state
        .hub
        .send(
            &[b],
            &event("presence", json!({ "sub": a, "presence": offline })),
        )
        .await;
    Ok(())
}

/// The answer to one request, after its events are sent.
pub async fn handle(
    state: &Arc<AppState>,
    caller: &Caller<'_>,
    method: &str,
    raw: Value,
) -> SocialResult<Value> {
    let me = caller.sub();
    let store = &state.store;
    match method {
        "rooms.list" => {
            let _: Nothing = params(raw)?;
            Ok(json!({
                "rooms": store.rooms_of(me).await?,
                "invites": store.invites_for(me).await?,
            }))
        }
        "rooms.discover" => {
            let _: Nothing = params(raw)?;
            let rooms: Vec<Value> = store
                .public_rooms()
                .await?
                .into_iter()
                .map(|(room, members)| json!({ "room": room, "members": members }))
                .collect();
            Ok(json!({ "rooms": rooms }))
        }
        "rooms.create" => {
            caller.named()?;
            let p: CreateRoom = params(raw)?;
            store.spend(me, &store::CREATE_ROOM).await?;
            let room = store.create_room(me, &p.kind, &p.name, &p.topic).await?;
            state
                .hub
                .send(
                    &[me],
                    &event("room.joined", json!({ "room": room, "role": "owner" })),
                )
                .await;
            Ok(json!({ "room": room }))
        }
        "rooms.join" => {
            let p: InRoom = params(raw)?;
            let room = store.join(me, p.room).await?;
            to_room(
                state,
                p.room,
                me,
                "member.joined",
                json!({ "room": p.room, "sub": me }),
            )
            .await?;
            state
                .hub
                .send(
                    &[me],
                    &event("room.joined", json!({ "room": room, "role": "member" })),
                )
                .await;
            announce_presence(state, me).await?;
            Ok(json!({ "room": room }))
        }
        "rooms.leave" => {
            let p: InRoom = params(raw)?;
            let before = store.recipients(p.room, me).await.unwrap_or_default();
            let closed = store.leave(me, p.room).await?;
            let data = json!({ "room": p.room, "sub": me, "closed": closed });
            state.hub.send(&before, &event("member.left", data)).await;
            Ok(json!({ "closed": closed }))
        }
        "rooms.update" => {
            let p: UpdateRoom = params(raw)?;
            let change = RoomChange {
                name: p.name,
                topic: p.topic,
                ambience: p.ambience,
            };
            let room = store.change_room(me, p.room, change).await?;
            to_room(state, p.room, me, "room.updated", json!({ "room": room })).await?;
            Ok(json!({ "room": room }))
        }
        "rooms.members" => {
            let p: InRoom = params(raw)?;
            let members = if p.room == store.world().await? {
                // World is everyone: it lists who is here and shows it, up to two hundred.
                store.membership(me, p.room).await?;
                let mut here = Vec::new();
                for sub in state.hub.connected().await {
                    if state.hub.presence(sub).await.state != "offline" {
                        here.push(sub);
                    }
                }
                here.truncate(200);
                store.members_among(p.room, &here).await?
            } else {
                store.members(me, p.room).await?
            };
            let profiles = profiles_of(state, members.iter().map(|m| m.sub)).await?;
            Ok(json!({ "members": members, "profiles": profiles }))
        }
        "rooms.invite" => {
            caller.named()?;
            let p: RoomAndName = params(raw)?;
            store.spend(me, &store::INVITE).await?;
            let invitee = store.profile_by_name(&p.qor_id).await?;
            store.invite(me, p.room, invitee.sub).await?;
            let (room, _) = store.membership(me, p.room).await?;
            state
                .hub
                .send(
                    &[invitee.sub],
                    &event(
                        "invite",
                        json!({ "room": room.id, "name": room.name, "inviter": me }),
                    ),
                )
                .await;
            Ok(json!({}))
        }
        "rooms.decline" => {
            let p: InRoom = params(raw)?;
            store.decline(me, p.room).await?;
            Ok(json!({}))
        }
        "rooms.role" => {
            let p: SetRole = params(raw)?;
            store.set_role(me, p.room, p.sub, &p.role).await?;
            to_room(
                state,
                p.room,
                me,
                "member.role",
                json!({ "room": p.room, "sub": p.sub, "role": p.role }),
            )
            .await?;
            Ok(json!({}))
        }
        "rooms.remove" => {
            let p: RoomAndPerson = params(raw)?;
            let before = store.recipients(p.room, me).await?;
            store.remove(me, p.room, p.sub).await?;
            let data = json!({ "room": p.room, "sub": p.sub, "removed_by": me });
            state.hub.send(&before, &event("member.left", data)).await;
            Ok(json!({}))
        }
        "dm.open" => {
            caller.named()?;
            let p: Named = params(raw)?;
            store.spend(me, &store::OPEN_DIRECT).await?;
            let other = store.profile_by_name(&p.qor_id).await?;
            let room = store.open_direct(me, other.sub).await?;
            Ok(
                json!({ "room": room, "other": profiles_of(state, [other.sub]).await?[other.sub.to_string()] }),
            )
        }
        "messages.history" => {
            let p: History = params(raw)?;
            let messages = store
                .history(me, p.room, p.before, p.limit.unwrap_or(50))
                .await?;
            let authors: Vec<Uuid> = messages.iter().map(|m| m.author).collect();
            let profiles = profiles_of(state, authors.iter().copied()).await?;
            // The levels above member its authors hold here, for the badges beside their names (ADR-093).
            let levels: serde_json::Map<String, Value> = store
                .levels_of(p.room, &authors)
                .await?
                .into_iter()
                .map(|(sub, role)| (sub.to_string(), json!(role)))
                .collect();
            Ok(json!({ "messages": messages, "profiles": profiles, "levels": levels }))
        }
        "messages.send" => {
            caller.named()?;
            let p: Send = params(raw)?;
            // Malformed words cost nothing, and are refused for what they are.
            crate::text::message(&p.body)?;
            store.spend(me, &store::SEND).await?;
            // World's pace is for members; a VIP or above talks freely (ADR-093).
            if p.room == store.world().await?
                && store.rank_in(p.room, me).await? < store::rank("vip")
            {
                store.spend(me, &store::WORLD_SEND).await?;
            }
            let (message, created) = store
                .send(
                    me,
                    p.room,
                    &p.body,
                    p.reply_to,
                    p.nonce.as_deref(),
                    p.lang.as_deref(),
                )
                .await?;
            if created {
                let level = store.membership(me, p.room).await?.1;
                to_room(
                    state,
                    p.room,
                    me,
                    "message.created",
                    json!({ "message": message, "level": level }),
                )
                .await?;
                let mentioned = store.mentioned(p.room, me, &p.body).await?;
                if !mentioned.is_empty() {
                    let data = json!({ "room": p.room, "message": message.id, "by": me });
                    state.hub.send(&mentioned, &event("mention", data)).await;
                }
            }
            Ok(json!({ "message": message }))
        }
        "messages.edit" => {
            let p: Edit = params(raw)?;
            crate::text::message(&p.body)?;
            store.spend(me, &store::SEND).await?;
            let message = store
                .edit(me, p.message, &p.body, p.lang.as_deref())
                .await?;
            to_room(
                state,
                message.room,
                me,
                "message.edited",
                json!({ "message": message }),
            )
            .await?;
            Ok(json!({ "message": message }))
        }
        "messages.delete" => {
            let p: OneMessage = params(raw)?;
            let message = store.delete(me, p.message).await?;
            to_room(
                state,
                message.room,
                message.author,
                "message.deleted",
                json!({ "message": message }),
            )
            .await?;
            Ok(json!({ "message": message }))
        }
        "messages.react" => {
            let p: React = params(raw)?;
            store.spend(me, &store::REACT).await?;
            let message = store.react(me, p.message, &p.emoji).await?;
            to_room(
                state,
                message.room,
                me,
                "message.reaction",
                json!({ "message": message }),
            )
            .await?;
            Ok(json!({ "message": message }))
        }
        "messages.read" => {
            let p: Read = params(raw)?;
            let last_read = store.mark_read(me, p.room, p.seq).await?;
            state
                .hub
                .send(
                    &[me],
                    &event("read", json!({ "room": p.room, "last_read": last_read })),
                )
                .await;
            Ok(json!({ "last_read": last_read }))
        }
        "typing" => {
            let p: InRoom = params(raw)?;
            store.membership(me, p.room).await?;
            if p.room == store.world().await? {
                // Everyone typing in a room of everyone is noise, not news.
                return Ok(json!({}));
            }
            let to: Vec<Uuid> = store
                .recipients(p.room, me)
                .await?
                .into_iter()
                .filter(|s| *s != me)
                .collect();
            state
                .hub
                .send(&to, &event("typing", json!({ "room": p.room, "sub": me })))
                .await;
            Ok(json!({}))
        }
        "presence.set" => {
            let p: SetPresence = params(raw)?;
            if !matches!(p.state.as_str(), "online" | "away" | "invisible") {
                return Err(SocialError::bad("presence is online, away or invisible"));
            }
            let playing = p
                .playing
                .as_deref()
                .map(|g| crate::text::line(g, "what you are playing", 1, 48))
                .transpose()?;
            state.hub.set(me, &p.state, playing).await;
            announce_presence(state, me).await?;
            Ok(json!({ "presence": state.hub.presence(me).await }))
        }
        "profile.get" => {
            let p: GetProfiles = params(raw)?;
            let subs = match (p.qor_id, p.subs) {
                (Some(name), None) => vec![store.profile_by_name(&name).await?.sub],
                (None, Some(subs)) if subs.len() <= 200 => subs,
                _ => return Err(SocialError::bad("ask for one qor_id, or up to 200 subs")),
            };
            Ok(json!({ "profiles": profiles_of(state, subs).await? }))
        }
        "profile.update" => {
            caller.named()?;
            let change: ProfileChange = params(raw)?;
            store.spend(me, &store::PROFILE).await?;
            let profile = store.change_profile(me, change).await?;
            let mut to = store.contacts(me).await?;
            to.push(me);
            state
                .hub
                .send(
                    &to,
                    &event("profile.updated", json!({ "profile": profile })),
                )
                .await;
            Ok(json!({ "profile": profile }))
        }
        "block" => {
            let p: Someone = params(raw)?;
            store.block(me, p.sub).await?;
            // They stop seeing this person's presence at once.
            state
                .hub
                .send(
                    &[p.sub],
                    &event(
                        "presence",
                        json!({ "sub": me, "presence": { "state": "offline", "playing": null } }),
                    ),
                )
                .await;
            Ok(json!({ "blocked": store.blocked_by(me).await? }))
        }
        "unblock" => {
            let p: Someone = params(raw)?;
            store.unblock(me, p.sub).await?;
            Ok(json!({ "blocked": store.blocked_by(me).await? }))
        }
        "blocks.list" => {
            let _: Nothing = params(raw)?;
            Ok(json!({ "blocked": store.blocked_by(me).await? }))
        }
        "report" => {
            let p: Report = params(raw)?;
            store.spend(me, &store::REPORT).await?;
            store.report(me, p.message, &p.reason, &p.note).await?;
            Ok(json!({}))
        }
        "friends.list" => {
            let _: Nothing = params(raw)?;
            let friends = store.friends(me).await?;
            let everyone = friends
                .friends
                .iter()
                .copied()
                .chain(friends.incoming.iter().map(|r| r.sub))
                .chain(friends.outgoing.iter().map(|r| r.sub));
            let profiles = profiles_of(state, everyone).await?;
            Ok(json!({
                "friends": friends.friends,
                "incoming": friends.incoming,
                "outgoing": friends.outgoing,
                "profiles": profiles,
            }))
        }
        "friends.request" => {
            caller.named()?;
            let p: Named = params(raw)?;
            store.spend(me, &store::FRIEND).await?;
            let other = store.profile_by_name(&p.qor_id).await?.sub;
            let friends = store.request_friend(me, other).await?;
            let profiles = profiles_of(state, [me, other]).await?;
            if friends {
                // They had asked first: friends at once.
                to_both(
                    state,
                    me,
                    other,
                    "friend.accepted",
                    json!({ "between": [me, other], "profiles": profiles }),
                )
                .await;
                announce_presence(state, me).await?;
                announce_presence(state, other).await?;
            } else {
                to_both(
                    state,
                    me,
                    other,
                    "friend.requested",
                    json!({ "from": me, "to": other, "profiles": profiles }),
                )
                .await;
            }
            Ok(json!({ "friends": friends, "sub": other }))
        }
        "friends.accept" => {
            let p: Someone = params(raw)?;
            store.accept_friend(me, p.sub).await?;
            let profiles = profiles_of(state, [me, p.sub]).await?;
            to_both(
                state,
                me,
                p.sub,
                "friend.accepted",
                json!({ "between": [me, p.sub], "profiles": profiles }),
            )
            .await;
            announce_presence(state, me).await?;
            announce_presence(state, p.sub).await?;
            Ok(json!({}))
        }
        "friends.decline" | "friends.cancel" => {
            let p: Someone = params(raw)?;
            let (from, to) = if method == "friends.decline" {
                (p.sub, me)
            } else {
                (me, p.sub)
            };
            store.drop_request(from, to).await?;
            to_both(
                state,
                from,
                to,
                "friend.declined",
                json!({ "from": from, "to": to }),
            )
            .await;
            Ok(json!({}))
        }
        "friends.remove" => {
            let p: Someone = params(raw)?;
            if !store.end_friendship(me, p.sub).await? {
                return Err(SocialError::not_found("you are not friends"));
            }
            to_both(
                state,
                me,
                p.sub,
                "friend.removed",
                json!({ "between": [me, p.sub] }),
            )
            .await;
            part(state, me, p.sub).await?;
            Ok(json!({}))
        }
        "circles.list" => {
            let _: Nothing = params(raw)?;
            Ok(json!({ "circles": store.circles(me).await? }))
        }
        "circles.create" => {
            caller.named()?;
            let p: NewCircle = params(raw)?;
            store.spend(me, &store::CIRCLE).await?;
            let circle = store.create_circle(me, &p.name, p.color.as_deref()).await?;
            Ok(json!({ "circle": circle }))
        }
        "circles.update" => {
            let p: ChangeCircle = params(raw)?;
            store.spend(me, &store::CIRCLE).await?;
            let color = p.color.as_ref().map(|c| c.as_deref());
            let circle = store
                .change_circle(me, p.circle, p.name.as_deref(), color)
                .await?;
            Ok(json!({ "circle": circle }))
        }
        "circles.delete" => {
            let p: InCircle = params(raw)?;
            store.delete_circle(me, p.circle).await?;
            Ok(json!({}))
        }
        "circles.add" | "circles.remove" => {
            let p: CircleAndPerson = params(raw)?;
            store.spend(me, &store::CIRCLE).await?;
            let circle = if method == "circles.add" {
                store.circle_add(me, p.circle, p.sub).await?
            } else {
                store.circle_remove(me, p.circle, p.sub).await?
            };
            Ok(json!({ "circle": circle }))
        }
        "circles.open" => {
            caller.named()?;
            let p: InCircle = params(raw)?;
            let circle = store.circle(me, p.circle).await?;
            if circle.members.is_empty() {
                return Err(SocialError::bad("add friends to the circle first"));
            }
            store.spend(me, &store::CREATE_ROOM).await?;
            let room = store.create_room(me, "private", &circle.name, "").await?;
            state
                .hub
                .send(
                    &[me],
                    &event("room.joined", json!({ "room": room, "role": "owner" })),
                )
                .await;
            for sub in &circle.members {
                // Someone either of them has blocked since is passed over; the others are asked.
                if store.invite(me, room.id, *sub).await.is_ok() {
                    state
                        .hub
                        .send(
                            &[*sub],
                            &event(
                                "invite",
                                json!({ "room": room.id, "name": room.name, "inviter": me }),
                            ),
                        )
                        .await;
                }
            }
            Ok(json!({ "room": room }))
        }
        "world.mute" => {
            let p: Mute = params(raw)?;
            let world = store.world().await?;
            let until = store.mute(me, world, p.sub, p.minutes).await?;
            state
                .hub
                .send(
                    &[p.sub],
                    &event("world.muted", json!({ "room": world, "until": until })),
                )
                .await;
            Ok(json!({ "until": until }))
        }
        "rooms.mute" => {
            let p: RoomMute = params(raw)?;
            let until = store.mute(me, p.room, p.sub, p.minutes).await?;
            let name = if p.room == store.world().await? {
                "world.muted"
            } else {
                "room.muted"
            };
            state
                .hub
                .send(
                    &[p.sub],
                    &event(name, json!({ "room": p.room, "until": until })),
                )
                .await;
            Ok(json!({ "until": until }))
        }
        "rooms.unmute" => {
            let p: RoomAndPerson = params(raw)?;
            store.unmute(me, p.room, p.sub).await?;
            let name = if p.room == store.world().await? {
                "world.muted"
            } else {
                "room.muted"
            };
            state
                .hub
                .send(
                    &[p.sub],
                    &event(name, json!({ "room": p.room, "until": null })),
                )
                .await;
            Ok(json!({}))
        }
        "elite.grant" | "elite.revoke" => {
            let target = if method == "elite.grant" {
                let p: Named = params(raw)?;
                let sub = store.profile_by_name(&p.qor_id).await?.sub;
                store.grant_elite(me, sub).await?;
                sub
            } else {
                let p: Someone = params(raw)?;
                store.revoke_elite(me, p.sub).await?;
                p.sub
            };
            // Their standing shows wherever their profile does: tell them, their contacts and godmode.
            let profile = profiles_of(state, [target]).await?[target.to_string()].clone();
            let mut to = store.contacts(target).await?;
            to.extend([target, me]);
            state
                .hub
                .send(
                    &to,
                    &event("profile.updated", json!({ "profile": profile })),
                )
                .await;
            Ok(json!({ "profile": profile }))
        }
        "elite.list" => {
            let _: Nothing = params(raw)?;
            if !store.standing(me).await?.0 {
                return Err(SocialError::forbidden(
                    "only godmode keeps the list of Elite accounts",
                ));
            }
            let elites = store.elites().await?;
            Ok(
                json!({ "elites": elites, "profiles": profiles_of(state, elites.iter().copied()).await? }),
            )
        }
        "world.unmute" => {
            let p: Someone = params(raw)?;
            let world = store.world().await?;
            store.unmute(me, world, p.sub).await?;
            state
                .hub
                .send(
                    &[p.sub],
                    &event("world.muted", json!({ "room": world, "until": null })),
                )
                .await;
            Ok(json!({}))
        }
        other => Err(SocialError::bad(format!(
            "Social offers no method {other:?}"
        ))),
    }
}
