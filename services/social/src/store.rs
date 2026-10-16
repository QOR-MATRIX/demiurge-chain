//! Social's data and its rules (docs/blueprints/social.md sections 3 and 4).
//!
//! Every rule a person could break is checked here, against the database, in the same transaction as the write it
//! guards: only members read a room; a private room is joined by invitation; a direct room has exactly two members and
//! none while either blocks the other; people edit and delete their own messages, moderators and owners delete others'
//! and remove members; an owner hands a room on rather than leaving it. Every statement is a literal with bound
//! parameters (`sql_hygiene.rs`).

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::error::{SocialError, SocialResult};
use crate::identity::Person;
use crate::seal::Sealer;
use crate::text;

/// The backdrops a banner or a room's ambience may name: the launcher's QFX presets (`tools/qor-launcher/src/qfx`).
pub const BACKDROPS: [&str; 6] = [
    "drift",
    "aurora",
    "nebula",
    "lattice",
    "starfield",
    "liquid",
];
/// How many assets a profile shows.
pub const SHOWCASE_MAX: usize = 6;
/// How many messages one page of history holds, at most.
pub const PAGE_MAX: i64 = 100;

/// How much each person may do, per window, across all their sockets.
pub struct Limit {
    pub action: &'static str,
    pub count: i32,
    pub window_secs: f64,
}

pub const SEND: Limit = Limit {
    action: "send",
    count: 10,
    window_secs: 10.0,
};
pub const REACT: Limit = Limit {
    action: "react",
    count: 30,
    window_secs: 10.0,
};
pub const CREATE_ROOM: Limit = Limit {
    action: "room",
    count: 10,
    window_secs: 3600.0,
};
pub const INVITE: Limit = Limit {
    action: "invite",
    count: 30,
    window_secs: 3600.0,
};
pub const OPEN_DIRECT: Limit = Limit {
    action: "direct",
    count: 30,
    window_secs: 3600.0,
};
pub const REPORT: Limit = Limit {
    action: "report",
    count: 20,
    window_secs: 3600.0,
};
pub const PROFILE: Limit = Limit {
    action: "profile",
    count: 20,
    window_secs: 60.0,
};

/// A room's columns with the caller's role, and with how many are in it.
type RoomAndRole = (Uuid, String, String, String, Value, DateTime<Utc>, String);
type RoomAndCount = (Uuid, String, String, String, Value, DateTime<Utc>, i64);

#[derive(Clone)]
pub struct Store {
    pub db: PgPool,
    pub sealer: Sealer,
}

// ── views, as clients receive them ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Profile {
    pub sub: Uuid,
    pub qor_id: String,
    pub avatar_url: Option<String>,
    pub level: i32,
    pub display: String,
    pub bio: String,
    pub accent: Option<String>,
    pub banner: Value,
    pub showcase: Value,
    pub status_line: String,
    pub seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Room {
    pub id: Uuid,
    pub kind: String,
    pub name: String,
    pub topic: String,
    pub ambience: Value,
    pub created_at: DateTime<Utc>,
}

/// A room as one member sees it in their list.
#[derive(Debug, Clone, Serialize)]
pub struct RoomSummary {
    #[serde(flatten)]
    pub room: Room,
    pub role: String,
    pub unread: i64,
    pub last_seq: i64,
    pub members: i64,
    /// In a direct room, the other person.
    pub other: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Member {
    pub sub: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Reaction {
    pub emoji: String,
    pub by: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: Uuid,
    pub room: Uuid,
    pub seq: i64,
    pub author: Uuid,
    /// The words, or nothing once deleted.
    pub body: Option<String>,
    pub reply_to: Option<Uuid>,
    pub reactions: Vec<Reaction>,
    pub edited_at: Option<DateTime<Utc>>,
    pub deleted: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct StoredMessage {
    id: Uuid,
    room: Uuid,
    seq: i64,
    author: Uuid,
    key_version: i16,
    nonce: Vec<u8>,
    sealed: Option<Vec<u8>>,
    reply_to: Option<Uuid>,
    edited_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Invite {
    pub room: Uuid,
    pub name: String,
    pub inviter: Uuid,
    pub created_at: DateTime<Utc>,
}

/// What a person may change about their profile; anything absent stays as it is.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileChange {
    pub display: Option<String>,
    pub bio: Option<String>,
    #[serde(default, deserialize_with = "present")]
    pub accent: Option<Option<String>>,
    pub banner: Option<Value>,
    pub showcase: Option<Vec<String>>,
    pub status_line: Option<String>,
}

/// A field that was sent, even as `null`: serde reads a `null` `Option<Option<_>>` as absent without this.
fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    <Option<T> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

/// What a room's owner or moderators may change.
#[derive(Debug, Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoomChange {
    pub name: Option<String>,
    pub topic: Option<String>,
    pub ambience: Option<Value>,
}

// ── checks of shape ─────────────────────────────────────────────────────────────────────────────────────────────

/// A backdrop and its parameters: `{}` for none, or `{"preset": <one of BACKDROPS>, "hue": 0..=360, "intensity":
/// 0..=1}`, with nothing else in it.
pub fn backdrop(value: &Value, what: &str) -> SocialResult<Value> {
    let refused = || {
        SocialError::bad(format!(
            "{what} is one of the QFX backdrops, with a hue and an intensity"
        ))
    };
    let object = value.as_object().ok_or_else(refused)?;
    if object.is_empty() {
        return Ok(json!({}));
    }
    if object
        .keys()
        .any(|k| !matches!(k.as_str(), "preset" | "hue" | "intensity"))
    {
        return Err(refused());
    }
    let preset = object
        .get("preset")
        .and_then(Value::as_str)
        .ok_or_else(refused)?;
    if !BACKDROPS.contains(&preset) {
        return Err(refused());
    }
    let hue = object
        .get("hue")
        .map_or(Some(0.0), Value::as_f64)
        .ok_or_else(refused)?;
    let intensity = object
        .get("intensity")
        .map_or(Some(0.6), Value::as_f64)
        .ok_or_else(refused)?;
    if !(0.0..=360.0).contains(&hue) || !(0.0..=1.0).contains(&intensity) {
        return Err(refused());
    }
    Ok(json!({ "preset": preset, "hue": hue, "intensity": intensity }))
}

fn accent(value: &str) -> SocialResult<String> {
    let lower = value.trim().to_ascii_lowercase();
    let ok = lower.len() == 7
        && lower.starts_with('#')
        && lower[1..].chars().all(|c| c.is_ascii_hexdigit());
    ok.then_some(lower)
        .ok_or_else(|| SocialError::bad("an accent is a colour written #rrggbb"))
}

/// A DRC-369 asset, written `collection/item`. Whether the person holds it is read from the chain where it is shown.
fn asset(value: &str) -> SocialResult<String> {
    let parts: Vec<&str> = value.trim().split('/').collect();
    let ok = parts.len() == 2
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.len() <= 10
                && p.chars().all(|c| c.is_ascii_digit())
                && p.parse::<u32>().is_ok()
        });
    ok.then(|| {
        format!(
            "{}/{}",
            parts[0].parse::<u32>().unwrap_or(0),
            parts[1].parse::<u32>().unwrap_or(0)
        )
    })
    .ok_or_else(|| SocialError::bad("a showcased asset is written collection/item"))
}

fn direct_key(a: Uuid, b: Uuid) -> String {
    let (first, second) = if a < b { (a, b) } else { (b, a) };
    format!("{first}:{second}")
}

impl Store {
    pub fn new(db: PgPool, sealer: Sealer) -> Self {
        Self { db, sealer }
    }

    // ── limits ──────────────────────────────────────────────────────────────────────────────────────────────────

    /// Count one `limit.action` for `sub`, refusing past its budget. One statement, so racing sockets share it.
    pub async fn spend(&self, sub: Uuid, limit: &Limit) -> SocialResult<()> {
        let used: i32 = sqlx::query_scalar(
            "INSERT INTO rate_buckets (sub, action, window_start, count) VALUES ($1, $2, NOW(), 1) \
             ON CONFLICT (sub, action) DO UPDATE SET \
               count = CASE WHEN rate_buckets.window_start <= NOW() - make_interval(secs => $3) \
                            THEN 1 ELSE rate_buckets.count + 1 END, \
               window_start = CASE WHEN rate_buckets.window_start <= NOW() - make_interval(secs => $3) \
                                   THEN NOW() ELSE rate_buckets.window_start END \
             RETURNING count",
        )
        .bind(sub)
        .bind(limit.action)
        .bind(limit.window_secs)
        .fetch_one(&self.db)
        .await?;
        if used > limit.count {
            return Err(SocialError::RateLimited(format!(
                "slow down: at most {} of those every {} seconds",
                limit.count, limit.window_secs as i64
            )));
        }
        Ok(())
    }

    // ── profiles ────────────────────────────────────────────────────────────────────────────────────────────────

    /// Record a person as QOR ID describes them now, keeping what they made of their profile here.
    pub async fn arrive(&self, person: &Person) -> SocialResult<Profile> {
        Ok(sqlx::query_as(
            "INSERT INTO profiles (sub, qor_id, avatar_url, level, seen_at) VALUES ($1, $2, $3, $4, NOW()) \
             ON CONFLICT (sub) DO UPDATE SET qor_id = $2, avatar_url = $3, level = $4, seen_at = NOW() \
             RETURNING sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, seen_at",
        )
        .bind(person.sub)
        .bind(&person.qor_id)
        .bind(&person.avatar_url)
        .bind(person.level)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn profiles(&self, subs: &[Uuid]) -> SocialResult<Vec<Profile>> {
        Ok(sqlx::query_as(
            "SELECT sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, seen_at \
             FROM profiles WHERE sub = ANY($1)",
        )
        .bind(subs)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn profile_by_name(&self, qor_id: &str) -> SocialResult<Profile> {
        sqlx::query_as(
            "SELECT sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, seen_at \
             FROM profiles WHERE LOWER(qor_id) = LOWER($1)",
        )
        .bind(qor_id.trim())
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| SocialError::not_found("nobody of that name has come to Social yet"))
    }

    pub async fn change_profile(&self, sub: Uuid, change: ProfileChange) -> SocialResult<Profile> {
        let display = change
            .display
            .as_deref()
            .map(|d| text::line(d, "a display line", 0, 48))
            .transpose()?;
        let bio = change
            .bio
            .as_deref()
            .map(|b| text::paragraph(b, "a bio", 600))
            .transpose()?;
        let status = change
            .status_line
            .as_deref()
            .map(|s| text::line(s, "a status", 0, 80))
            .transpose()?;
        let accent_given = change.accent.is_some();
        let accent = change.accent.flatten().as_deref().map(accent).transpose()?;
        let banner = change
            .banner
            .as_ref()
            .map(|b| backdrop(b, "a banner"))
            .transpose()?;
        let showcase = match change.showcase {
            Some(list) => {
                if list.len() > SHOWCASE_MAX {
                    return Err(SocialError::bad(format!(
                        "a showcase holds at most {SHOWCASE_MAX} assets"
                    )));
                }
                let mut assets = Vec::with_capacity(list.len());
                for a in &list {
                    let a = asset(a)?;
                    if !assets.contains(&a) {
                        assets.push(a);
                    }
                }
                Some(json!(assets))
            }
            None => None,
        };
        Ok(sqlx::query_as(
            "UPDATE profiles SET \
               display = COALESCE($2, display), bio = COALESCE($3, bio), \
               accent = CASE WHEN $4 THEN $5 ELSE accent END, \
               banner = COALESCE($6, banner), showcase = COALESCE($7, showcase), \
               status_line = COALESCE($8, status_line), updated_at = NOW() \
             WHERE sub = $1 \
             RETURNING sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, seen_at",
        )
        .bind(sub)
        .bind(display)
        .bind(bio)
        .bind(accent_given)
        .bind(accent)
        .bind(banner)
        .bind(showcase)
        .bind(status)
        .fetch_one(&self.db)
        .await?)
    }

    // ── rooms ───────────────────────────────────────────────────────────────────────────────────────────────────

    /// The room and the caller's role in it, or not found: a room's existence is not told to someone outside it,
    /// unless it is public.
    pub async fn membership(&self, sub: Uuid, room: Uuid) -> SocialResult<(Room, String)> {
        let found: Option<RoomAndRole> = sqlx::query_as(
            "SELECT r.id, r.kind, r.name, r.topic, r.ambience, r.created_at, m.role \
             FROM rooms r JOIN members m ON m.room = r.id WHERE r.id = $1 AND m.sub = $2",
        )
        .bind(room)
        .bind(sub)
        .fetch_optional(&self.db)
        .await?;
        let (id, kind, name, topic, ambience, created_at, role) =
            found.ok_or_else(|| SocialError::not_found("no such room, or you are not in it"))?;
        Ok((
            Room {
                id,
                kind,
                name,
                topic,
                ambience,
                created_at,
            },
            role,
        ))
    }

    pub async fn create_room(
        &self,
        creator: Uuid,
        kind: &str,
        name: &str,
        topic: &str,
    ) -> SocialResult<Room> {
        if !matches!(kind, "public" | "private") {
            return Err(SocialError::bad("a room is public or private"));
        }
        let name = text::line(name, "a room's name", 1, 48)?;
        let topic = text::line(topic, "a topic", 0, 280)?;
        let mut tx = self.db.begin().await?;
        let room: Room = sqlx::query_as(
            "INSERT INTO rooms (id, kind, name, topic, created_by) VALUES ($1, $2, $3, $4, $5) \
             RETURNING id, kind, name, topic, ambience, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(kind)
        .bind(&name)
        .bind(&topic)
        .bind(creator)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO members (room, sub, role) VALUES ($1, $2, 'owner')")
            .bind(room.id)
            .bind(creator)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(room)
    }

    /// Public rooms, newest first, with how many are in each, for finding one to join.
    pub async fn public_rooms(&self) -> SocialResult<Vec<(Room, i64)>> {
        let rows: Vec<RoomAndCount> = sqlx::query_as(
            "SELECT r.id, r.kind, r.name, r.topic, r.ambience, r.created_at, \
                    (SELECT COUNT(*) FROM members m WHERE m.room = r.id) \
             FROM rooms r WHERE r.kind = 'public' ORDER BY r.created_at DESC LIMIT 200",
        )
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(id, kind, name, topic, ambience, created_at, members)| {
                (
                    Room {
                        id,
                        kind,
                        name,
                        topic,
                        ambience,
                        created_at,
                    },
                    members,
                )
            })
            .collect())
    }

    /// The rooms a person is in, with what they have not read.
    pub async fn rooms_of(&self, sub: Uuid) -> SocialResult<Vec<RoomSummary>> {
        type Row = (
            Uuid,
            String,
            String,
            String,
            Value,
            DateTime<Utc>,
            String,
            i64,
            i64,
            i64,
            Option<Uuid>,
        );
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT r.id, r.kind, r.name, r.topic, r.ambience, r.created_at, m.role, \
                    (SELECT COUNT(*) FROM messages x WHERE x.room = r.id AND x.seq > m.last_read AND x.author <> $1 \
                       AND x.deleted_at IS NULL \
                       AND x.author NOT IN (SELECT blocked FROM blocks WHERE blocker = $1)), \
                    r.next_seq - 1, \
                    (SELECT COUNT(*) FROM members c WHERE c.room = r.id), \
                    (SELECT o.sub FROM members o WHERE o.room = r.id AND o.sub <> $1 AND r.kind = 'direct' LIMIT 1) \
             FROM rooms r JOIN members m ON m.room = r.id WHERE m.sub = $1 \
             ORDER BY r.next_seq DESC, r.created_at DESC",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .into_iter()
            .map(
                |(
                    id,
                    kind,
                    name,
                    topic,
                    ambience,
                    created_at,
                    role,
                    unread,
                    last_seq,
                    members,
                    other,
                )| RoomSummary {
                    room: Room {
                        id,
                        kind,
                        name,
                        topic,
                        ambience,
                        created_at,
                    },
                    role,
                    unread,
                    last_seq,
                    members,
                    other,
                },
            )
            .collect())
    }

    pub async fn join(&self, sub: Uuid, room: Uuid) -> SocialResult<Room> {
        let mut tx = self.db.begin().await?;
        let found: Option<Room> = sqlx::query_as(
            "SELECT id, kind, name, topic, ambience, created_at FROM rooms WHERE id = $1",
        )
        .bind(room)
        .fetch_optional(&mut *tx)
        .await?;
        let room_row = found.ok_or_else(|| SocialError::not_found("no such room"))?;
        let invited: bool = sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM invites WHERE room = $1 AND invitee = $2)",
        )
        .bind(room)
        .bind(sub)
        .fetch_one(&mut *tx)
        .await?;
        match room_row.kind.as_str() {
            "public" => {}
            "private" if invited => {}
            // A private room is not told to someone without an invitation; a direct room is never joined.
            _ => return Err(SocialError::not_found("no such room")),
        }
        sqlx::query("INSERT INTO members (room, sub, role) VALUES ($1, $2, 'member') ON CONFLICT DO NOTHING")
            .bind(room)
            .bind(sub)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM invites WHERE room = $1 AND invitee = $2")
            .bind(room)
            .bind(sub)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(room_row)
    }

    /// Leave a room. An owner hands it on first, unless they are the last one in it, when the room goes with them.
    pub async fn leave(&self, sub: Uuid, room: Uuid) -> SocialResult<bool> {
        let (found, role) = self.membership(sub, room).await?;
        if found.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation is not left; block the person instead",
            ));
        }
        let mut tx = self.db.begin().await?;
        let others: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM members WHERE room = $1 AND sub <> $2")
                .bind(room)
                .bind(sub)
                .fetch_one(&mut *tx)
                .await?;
        if role == "owner" && others > 0 {
            return Err(SocialError::forbidden(
                "hand the room to someone else before you leave it",
            ));
        }
        let closed = role == "owner";
        if closed {
            sqlx::query("DELETE FROM rooms WHERE id = $1")
                .bind(room)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("DELETE FROM members WHERE room = $1 AND sub = $2")
                .bind(room)
                .bind(sub)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(closed)
    }

    pub async fn change_room(
        &self,
        by: Uuid,
        room: Uuid,
        change: RoomChange,
    ) -> SocialResult<Room> {
        let (found, role) = self.membership(by, room).await?;
        if found.kind == "direct" || role == "member" {
            return Err(SocialError::forbidden(
                "only a room's owner and moderators change it",
            ));
        }
        let name = change
            .name
            .as_deref()
            .map(|n| text::line(n, "a room's name", 1, 48))
            .transpose()?;
        let topic = change
            .topic
            .as_deref()
            .map(|t| text::line(t, "a topic", 0, 280))
            .transpose()?;
        let ambience = change
            .ambience
            .as_ref()
            .map(|a| backdrop(a, "a room's ambience"))
            .transpose()?;
        Ok(sqlx::query_as(
            "UPDATE rooms SET name = COALESCE($2, name), topic = COALESCE($3, topic), \
               ambience = COALESCE($4, ambience) \
             WHERE id = $1 RETURNING id, kind, name, topic, ambience, created_at",
        )
        .bind(room)
        .bind(name)
        .bind(topic)
        .bind(ambience)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn members(&self, sub: Uuid, room: Uuid) -> SocialResult<Vec<Member>> {
        self.membership(sub, room).await?;
        Ok(sqlx::query_as(
            "SELECT sub, role, joined_at FROM members WHERE room = $1 \
             ORDER BY CASE role WHEN 'owner' THEN 0 WHEN 'moderator' THEN 1 ELSE 2 END, joined_at",
        )
        .bind(room)
        .fetch_all(&self.db)
        .await?)
    }

    /// Invite someone Social knows to a room the inviter moderates or owns.
    pub async fn invite(&self, by: Uuid, room: Uuid, invitee: Uuid) -> SocialResult<()> {
        let (found, role) = self.membership(by, room).await?;
        if found.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation is between two people",
            ));
        }
        if found.kind == "private" && role == "member" {
            return Err(SocialError::forbidden(
                "only a private room's owner and moderators invite",
            ));
        }
        if self.profiles(&[invitee]).await?.is_empty() {
            return Err(SocialError::not_found(
                "nobody of that name has come to Social yet",
            ));
        }
        if self.blocked_either_way(by, invitee).await? {
            return Err(SocialError::forbidden(
                "you cannot invite someone either of you has blocked",
            ));
        }
        sqlx::query(
            "INSERT INTO invites (room, invitee, inviter) \
             SELECT $1, $2, $3 WHERE NOT EXISTS (SELECT 1 FROM members WHERE room = $1 AND sub = $2) \
             ON CONFLICT DO NOTHING",
        )
        .bind(room)
        .bind(invitee)
        .bind(by)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    pub async fn invites_for(&self, sub: Uuid) -> SocialResult<Vec<Invite>> {
        Ok(sqlx::query_as(
            "SELECT i.room, r.name, i.inviter, i.created_at FROM invites i JOIN rooms r ON r.id = i.room \
             WHERE i.invitee = $1 AND i.inviter NOT IN (SELECT blocked FROM blocks WHERE blocker = $1) \
             ORDER BY i.created_at DESC",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?)
    }

    pub async fn decline(&self, sub: Uuid, room: Uuid) -> SocialResult<()> {
        sqlx::query("DELETE FROM invites WHERE room = $1 AND invitee = $2")
            .bind(room)
            .bind(sub)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Change someone's role. An owner makes moderators and members, and hands the room on (becoming a moderator).
    pub async fn set_role(
        &self,
        by: Uuid,
        room: Uuid,
        target: Uuid,
        role: &str,
    ) -> SocialResult<()> {
        let (found, by_role) = self.membership(by, room).await?;
        if found.kind == "direct" || by_role != "owner" {
            return Err(SocialError::forbidden("only a room's owner changes roles"));
        }
        if !matches!(role, "owner" | "moderator" | "member") {
            return Err(SocialError::bad("a role is owner, moderator or member"));
        }
        if target == by {
            return Err(SocialError::bad(
                "hand the room to someone else to change your own role",
            ));
        }
        let mut tx = self.db.begin().await?;
        let changed = sqlx::query("UPDATE members SET role = $3 WHERE room = $1 AND sub = $2")
            .bind(room)
            .bind(target)
            .bind(role)
            .execute(&mut *tx)
            .await?
            .rows_affected();
        if changed == 0 {
            return Err(SocialError::not_found("they are not in this room"));
        }
        if role == "owner" {
            sqlx::query("UPDATE members SET role = 'moderator' WHERE room = $1 AND sub = $2")
                .bind(room)
                .bind(by)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Remove someone from a room: an owner removes anyone else, a moderator removes members.
    pub async fn remove(&self, by: Uuid, room: Uuid, target: Uuid) -> SocialResult<()> {
        let (found, by_role) = self.membership(by, room).await?;
        if found.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation is between two people",
            ));
        }
        let target_role: Option<String> =
            sqlx::query_scalar("SELECT role FROM members WHERE room = $1 AND sub = $2")
                .bind(room)
                .bind(target)
                .fetch_optional(&self.db)
                .await?;
        let target_role =
            target_role.ok_or_else(|| SocialError::not_found("they are not in this room"))?;
        let allowed = target != by
            && match by_role.as_str() {
                "owner" => true,
                "moderator" => target_role == "member",
                _ => false,
            };
        if !allowed {
            return Err(SocialError::forbidden("you cannot remove them"));
        }
        sqlx::query("DELETE FROM members WHERE room = $1 AND sub = $2")
            .bind(room)
            .bind(target)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// The one direct room between two people, made the first time either opens it.
    pub async fn open_direct(&self, sub: Uuid, other: Uuid) -> SocialResult<Room> {
        if sub == other {
            return Err(SocialError::bad(
                "a direct conversation is with someone else",
            ));
        }
        if self.profiles(&[other]).await?.is_empty() {
            return Err(SocialError::not_found(
                "nobody of that name has come to Social yet",
            ));
        }
        if self.blocked_either_way(sub, other).await? {
            return Err(SocialError::forbidden(
                "you cannot message someone either of you has blocked",
            ));
        }
        let key = direct_key(sub, other);
        let mut tx = self.db.begin().await?;
        let room: Room = sqlx::query_as(
            "INSERT INTO rooms (id, kind, direct_key, created_by) VALUES ($1, 'direct', $2, $3) \
             ON CONFLICT (direct_key) DO UPDATE SET direct_key = EXCLUDED.direct_key \
             RETURNING id, kind, name, topic, ambience, created_at",
        )
        .bind(Uuid::new_v4())
        .bind(&key)
        .bind(sub)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "INSERT INTO members (room, sub, role) VALUES ($1, $2, 'member'), ($1, $3, 'member') ON CONFLICT DO NOTHING",
        )
        .bind(room.id)
        .bind(sub)
        .bind(other)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(room)
    }

    // ── messages ────────────────────────────────────────────────────────────────────────────────────────────────

    fn view(&self, stored: StoredMessage, reactions: Vec<Reaction>) -> SocialResult<Message> {
        let body = match (&stored.sealed, stored.deleted_at) {
            (Some(sealed), None) => Some(self.sealer.open(
                stored.room,
                stored.seq,
                stored.key_version,
                &stored.nonce,
                sealed,
            )?),
            _ => None,
        };
        Ok(Message {
            id: stored.id,
            room: stored.room,
            seq: stored.seq,
            author: stored.author,
            body,
            reply_to: stored.reply_to,
            reactions,
            edited_at: stored.edited_at,
            deleted: stored.deleted_at.is_some(),
            created_at: stored.created_at,
        })
    }

    async fn reactions(&self, messages: &[Uuid]) -> SocialResult<Vec<(Uuid, Reaction)>> {
        let rows: Vec<(Uuid, String, Vec<Uuid>)> = sqlx::query_as(
            "SELECT message, emoji, ARRAY_AGG(sub ORDER BY created_at) FROM reactions \
             WHERE message = ANY($1) GROUP BY message, emoji ORDER BY MIN(created_at)",
        )
        .bind(messages)
        .fetch_all(&self.db)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(m, emoji, by)| (m, Reaction { emoji, by }))
            .collect())
    }

    async fn message_by_id(&self, id: Uuid) -> SocialResult<Message> {
        let stored: StoredMessage = sqlx::query_as(
            "SELECT id, room, seq, author, key_version, nonce, sealed, reply_to, edited_at, deleted_at, created_at \
             FROM messages WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| SocialError::not_found("no such message"))?;
        let reactions = self
            .reactions(&[id])
            .await?
            .into_iter()
            .map(|(_, r)| r)
            .collect();
        self.view(stored, reactions)
    }

    async fn direct_blocked(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        room: Uuid,
        sub: Uuid,
    ) -> SocialResult<bool> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM rooms r JOIN members o ON o.room = r.id AND o.sub <> $2 \
               JOIN blocks b ON (b.blocker = $2 AND b.blocked = o.sub) OR (b.blocker = o.sub AND b.blocked = $2) \
               WHERE r.id = $1 AND r.kind = 'direct')",
        )
        .bind(room)
        .bind(sub)
        .fetch_one(&mut **tx)
        .await?)
    }

    /// Post a message. The same `client_nonce` from the same author returns the message it made the first time.
    pub async fn send(
        &self,
        author: Uuid,
        room: Uuid,
        body: &str,
        reply_to: Option<Uuid>,
        client_nonce: Option<&str>,
    ) -> SocialResult<(Message, bool)> {
        let body = text::message(body)?;
        if let Some(n) = client_nonce
            && (n.is_empty()
                || n.len() > 64
                || !n
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'))
        {
            return Err(SocialError::bad(
                "a nonce is 1 to 64 letters, digits, '-' or '_'",
            ));
        }
        self.membership(author, room).await?;
        if let Some(n) = client_nonce {
            let earlier: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM messages WHERE author = $1 AND client_nonce = $2",
            )
            .bind(author)
            .bind(n)
            .fetch_optional(&self.db)
            .await?;
            if let Some(id) = earlier {
                return Ok((self.message_by_id(id).await?, false));
            }
        }
        let mut tx = self.db.begin().await?;
        if self.direct_blocked(&mut tx, room, author).await? {
            return Err(SocialError::forbidden(
                "you cannot message someone either of you has blocked",
            ));
        }
        if let Some(reply) = reply_to {
            let same_room: bool = sqlx::query_scalar(
                "SELECT EXISTS (SELECT 1 FROM messages WHERE id = $1 AND room = $2)",
            )
            .bind(reply)
            .bind(room)
            .fetch_one(&mut *tx)
            .await?;
            if !same_room {
                return Err(SocialError::bad("a reply is to a message in the same room"));
            }
        }
        // The room's row lock makes the sequence gapless and ordered.
        let seq: i64 = sqlx::query_scalar(
            "UPDATE rooms SET next_seq = next_seq + 1 WHERE id = $1 RETURNING next_seq - 1",
        )
        .bind(room)
        .fetch_one(&mut *tx)
        .await?;
        let sealed = self.sealer.seal(room, seq, &body)?;
        let id = Uuid::new_v4();
        let stored: StoredMessage = sqlx::query_as(
            "INSERT INTO messages (id, room, seq, author, key_version, nonce, sealed, reply_to, client_nonce) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9) \
             RETURNING id, room, seq, author, key_version, nonce, sealed, reply_to, edited_at, deleted_at, created_at",
        )
        .bind(id)
        .bind(room)
        .bind(seq)
        .bind(author)
        .bind(sealed.key_version)
        .bind(&sealed.nonce)
        .bind(&sealed.sealed)
        .bind(reply_to)
        .bind(client_nonce)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            "UPDATE members SET last_read = GREATEST(last_read, $3) WHERE room = $1 AND sub = $2",
        )
        .bind(room)
        .bind(author)
        .bind(seq)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok((self.view(stored, vec![])?, true))
    }

    /// Change your own message's words.
    pub async fn edit(&self, author: Uuid, message: Uuid, body: &str) -> SocialResult<Message> {
        let body = text::message(body)?;
        let found = self.message_by_id(message).await?;
        self.membership(author, found.room).await?;
        if found.author != author {
            return Err(SocialError::forbidden("you edit only your own messages"));
        }
        if found.deleted {
            return Err(SocialError::not_found("that message was deleted"));
        }
        let sealed = self.sealer.seal(found.room, found.seq, &body)?;
        sqlx::query(
            "UPDATE messages SET key_version = $2, nonce = $3, sealed = $4, edited_at = NOW() \
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(message)
        .bind(sealed.key_version)
        .bind(&sealed.nonce)
        .bind(&sealed.sealed)
        .execute(&self.db)
        .await?;
        self.message_by_id(message).await
    }

    /// Delete a message: its author, or the room's owner or moderators. Its words go; its place stays.
    pub async fn delete(&self, by: Uuid, message: Uuid) -> SocialResult<Message> {
        let found = self.message_by_id(message).await?;
        let (_, role) = self.membership(by, found.room).await?;
        if found.author != by && role == "member" {
            return Err(SocialError::forbidden("you delete only your own messages"));
        }
        sqlx::query(
            "UPDATE messages SET sealed = NULL, deleted_at = COALESCE(deleted_at, NOW()) WHERE id = $1",
        )
        .bind(message)
        .execute(&self.db)
        .await?;
        sqlx::query("DELETE FROM reactions WHERE message = $1")
            .bind(message)
            .execute(&self.db)
            .await?;
        self.message_by_id(message).await
    }

    /// Add a reaction, or take it back if it is already yours.
    pub async fn react(&self, sub: Uuid, message: Uuid, emoji: &str) -> SocialResult<Message> {
        let emoji = text::emoji(emoji)?;
        let found = self.message_by_id(message).await?;
        self.membership(sub, found.room).await?;
        if found.deleted {
            return Err(SocialError::not_found("that message was deleted"));
        }
        let removed =
            sqlx::query("DELETE FROM reactions WHERE message = $1 AND sub = $2 AND emoji = $3")
                .bind(message)
                .bind(sub)
                .bind(&emoji)
                .execute(&self.db)
                .await?
                .rows_affected();
        if removed == 0 {
            let kinds: i64 = sqlx::query_scalar(
                "SELECT COUNT(DISTINCT emoji) FROM reactions WHERE message = $1",
            )
            .bind(message)
            .fetch_one(&self.db)
            .await?;
            if kinds >= 20 {
                return Err(SocialError::Conflict(
                    "a message holds at most 20 different reactions".into(),
                ));
            }
            sqlx::query("INSERT INTO reactions (message, sub, emoji) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING")
                .bind(message)
                .bind(sub)
                .bind(&emoji)
                .execute(&self.db)
                .await?;
        }
        self.message_by_id(message).await
    }

    /// A page of a room's history, newest last, before `before` (or the newest), leaving out people the reader blocks.
    pub async fn history(
        &self,
        reader: Uuid,
        room: Uuid,
        before: Option<i64>,
        limit: i64,
    ) -> SocialResult<Vec<Message>> {
        self.membership(reader, room).await?;
        let limit = limit.clamp(1, PAGE_MAX);
        let mut stored: Vec<StoredMessage> = sqlx::query_as(
            "SELECT id, room, seq, author, key_version, nonce, sealed, reply_to, edited_at, deleted_at, created_at \
             FROM messages WHERE room = $1 AND seq < $2 \
               AND author NOT IN (SELECT blocked FROM blocks WHERE blocker = $3) \
             ORDER BY seq DESC LIMIT $4",
        )
        .bind(room)
        .bind(before.unwrap_or(i64::MAX))
        .bind(reader)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;
        stored.reverse();
        let ids: Vec<Uuid> = stored.iter().map(|m| m.id).collect();
        let mut reactions = self.reactions(&ids).await?;
        stored
            .into_iter()
            .map(|m| {
                let mine: Vec<Reaction> = reactions
                    .iter()
                    .filter(|(id, _)| *id == m.id)
                    .map(|(_, r)| r.clone())
                    .collect();
                reactions.retain(|(id, _)| *id != m.id);
                self.view(m, mine)
            })
            .collect()
    }

    pub async fn mark_read(&self, sub: Uuid, room: Uuid, seq: i64) -> SocialResult<i64> {
        self.membership(sub, room).await?;
        Ok(sqlx::query_scalar(
            "UPDATE members SET last_read = GREATEST(last_read, LEAST($3, (SELECT next_seq - 1 FROM rooms WHERE id = $1))) \
             WHERE room = $1 AND sub = $2 RETURNING last_read",
        )
        .bind(room)
        .bind(sub)
        .bind(seq)
        .fetch_one(&self.db)
        .await?)
    }

    // ── blocks and reports ──────────────────────────────────────────────────────────────────────────────────────

    pub async fn blocked_either_way(&self, a: Uuid, b: Uuid) -> SocialResult<bool> {
        Ok(sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM blocks WHERE (blocker = $1 AND blocked = $2) OR (blocker = $2 AND blocked = $1))",
        )
        .bind(a)
        .bind(b)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn block(&self, sub: Uuid, other: Uuid) -> SocialResult<()> {
        if sub == other {
            return Err(SocialError::bad("you cannot block yourself"));
        }
        sqlx::query("INSERT INTO blocks (blocker, blocked) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(sub)
            .bind(other)
            .execute(&self.db)
            .await?;
        // An invitation from the blocked person is withdrawn with it.
        sqlx::query("DELETE FROM invites WHERE invitee = $1 AND inviter = $2")
            .bind(sub)
            .bind(other)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn unblock(&self, sub: Uuid, other: Uuid) -> SocialResult<()> {
        sqlx::query("DELETE FROM blocks WHERE blocker = $1 AND blocked = $2")
            .bind(sub)
            .bind(other)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn blocked_by(&self, sub: Uuid) -> SocialResult<Vec<Uuid>> {
        Ok(
            sqlx::query_scalar("SELECT blocked FROM blocks WHERE blocker = $1 ORDER BY created_at")
                .bind(sub)
                .fetch_all(&self.db)
                .await?,
        )
    }

    /// Report a message in a room the reporter is in, once.
    pub async fn report(
        &self,
        reporter: Uuid,
        message: Uuid,
        reason: &str,
        note: &str,
    ) -> SocialResult<()> {
        if !matches!(
            reason,
            "spam" | "harassment" | "hate" | "violence" | "sexual" | "other"
        ) {
            return Err(SocialError::bad(
                "a report's reason is spam, harassment, hate, violence, sexual or other",
            ));
        }
        let note = text::paragraph(note, "a report's note", 500)?;
        let found = self.message_by_id(message).await?;
        self.membership(reporter, found.room).await?;
        if found.author == reporter {
            return Err(SocialError::bad("you cannot report your own message"));
        }
        sqlx::query(
            "INSERT INTO reports (id, message, reporter, reason, note) VALUES ($1, $2, $3, $4, $5) ON CONFLICT DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(message)
        .bind(reporter)
        .bind(reason)
        .bind(note)
        .execute(&self.db)
        .await?;
        Ok(())
    }

    // ── who hears what ──────────────────────────────────────────────────────────────────────────────────────────

    /// The members of `room` that `body` mentions by `@name`, less the author and anyone who blocks them. A name that
    /// is not in the room mentions nobody: a mention never reaches beyond the room it was written in.
    pub async fn mentioned(&self, room: Uuid, author: Uuid, body: &str) -> SocialResult<Vec<Uuid>> {
        let mut names: Vec<String> = Vec::new();
        for (at, _) in body.match_indices('@') {
            let before = body[..at].chars().next_back();
            if before.is_some_and(|c| c.is_alphanumeric() || c == '_') {
                continue; // an address, not a mention
            }
            let name: String = body[at + 1..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if (3..=20).contains(&name.len()) {
                names.push(name.to_ascii_lowercase());
            }
        }
        names.sort_unstable();
        names.dedup();
        names.truncate(20);
        if names.is_empty() {
            return Ok(vec![]);
        }
        Ok(sqlx::query_scalar(
            "SELECT p.sub FROM profiles p JOIN members m ON m.sub = p.sub AND m.room = $1 \
             WHERE LOWER(p.qor_id) = ANY($2) AND p.sub <> $3 \
               AND p.sub NOT IN (SELECT blocker FROM blocks WHERE blocked = $3)",
        )
        .bind(room)
        .bind(&names)
        .bind(author)
        .fetch_all(&self.db)
        .await?)
    }

    /// A room's members, less those who block `author`.
    pub async fn recipients(&self, room: Uuid, author: Uuid) -> SocialResult<Vec<Uuid>> {
        Ok(sqlx::query_scalar(
            "SELECT sub FROM members WHERE room = $1 \
               AND sub NOT IN (SELECT blocker FROM blocks WHERE blocked = $2)",
        )
        .bind(room)
        .bind(author)
        .fetch_all(&self.db)
        .await?)
    }

    /// Everyone who shares a room with `sub`, less anyone either way blocked: those who see their presence.
    pub async fn contacts(&self, sub: Uuid) -> SocialResult<Vec<Uuid>> {
        Ok(sqlx::query_scalar(
            "SELECT DISTINCT o.sub FROM members m JOIN members o ON o.room = m.room \
             WHERE m.sub = $1 AND o.sub <> $1 \
               AND o.sub NOT IN (SELECT blocker FROM blocks WHERE blocked = $1) \
               AND o.sub NOT IN (SELECT blocked FROM blocks WHERE blocker = $1)",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_backdrop_is_a_qfx_preset_with_a_hue_and_an_intensity() {
        assert_eq!(backdrop(&json!({}), "a banner").unwrap(), json!({}));
        assert_eq!(
            backdrop(
                &json!({ "preset": "aurora", "hue": 200, "intensity": 0.4 }),
                "a banner"
            )
            .unwrap(),
            json!({ "preset": "aurora", "hue": 200.0, "intensity": 0.4 })
        );
        for bad in [
            json!({ "preset": "fire" }),
            json!({ "preset": "aurora", "hue": 400 }),
            json!({ "preset": "aurora", "intensity": 2 }),
            json!({ "preset": "aurora", "script": "x" }),
            json!("aurora"),
        ] {
            assert!(backdrop(&bad, "a banner").is_err(), "{bad}");
        }
    }

    #[test]
    fn an_accent_and_an_asset_have_one_form_each() {
        assert_eq!(accent(" #A1B2C3 ").unwrap(), "#a1b2c3");
        for bad in ["a1b2c3", "#12345", "#gggggg", "red"] {
            assert!(accent(bad).is_err(), "{bad}");
        }
        assert_eq!(asset("0012/7").unwrap(), "12/7");
        for bad in ["12", "12/7/1", "a/1", "-1/2", "99999999999/1", ""] {
            assert!(asset(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn one_pair_has_one_direct_key_whoever_opens_it() {
        let (a, b) = (Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(direct_key(a, b), direct_key(b, a));
        assert_eq!(direct_key(a, b).len(), 73);
    }
}
