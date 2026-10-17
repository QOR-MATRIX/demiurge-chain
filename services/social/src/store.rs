//! Social's data and its rules (docs/blueprints/social.md sections 3 and 4).
//!
//! Every rule a person could break is checked here, against the database, in the same transaction as the write it
//! guards: only members read a room; a private room is joined by invitation; a direct room has exactly two members and
//! none while either blocks the other; people edit and delete their own messages, moderators and owners delete others'
//! and remove members; an owner hands a room on rather than leaving it. Every statement is a literal with bound
//! parameters (`sql_hygiene.rs`).

use std::sync::{Arc, OnceLock};

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
/// World's own budget, on top of [`SEND`]: one room everyone is in moves at a slower pace (ADR-092 decision 1).
pub const WORLD_SEND: Limit = Limit {
    action: "world",
    count: 5,
    window_secs: 20.0,
};
pub const FRIEND: Limit = Limit {
    action: "friend",
    count: 30,
    window_secs: 3600.0,
};
pub const CIRCLE: Limit = Limit {
    action: "circle",
    count: 60,
    window_secs: 60.0,
};
/// How many circles a person keeps, and how many friends one holds (ADR-092 decision 2).
pub const CIRCLES_MAX: i64 = 20;
pub const CIRCLE_MEMBERS_MAX: i64 = 100;
/// The lengths of a mute a moderator chooses from, in minutes: ten minutes, an hour, a day, a week.
pub const MUTE_MINUTES: [i32; 4] = [10, 60, 1_440, 10_080];

/// A room's levels, highest first (ADR-093). Each acts only on those below it.
pub const LEVELS: [&str; 5] = ["owner", "admin", "moderator", "vip", "member"];

/// A level's place on the ladder: member 1 up to owner 5.
pub fn rank(level: &str) -> u8 {
    match level {
        "owner" => 5,
        "admin" => 4,
        "moderator" => 3,
        "vip" => 2,
        _ => 1,
    }
}

/// An Elite account acts as an Admin in every room it is in; godmode stands above every room's Owner (ADR-093).
pub const ELITE_RANK: u8 = 4;
pub const GODMODE_RANK: u8 = 6;

/// What someone may do in a room: the room, their level there, and where they stand once their account's standing
/// is counted.
pub struct Authority {
    pub room: Room,
    pub role: String,
    pub rank: u8,
}

/// A room's columns with the caller's role, and with how many are in it.
type RoomAndRole = (Uuid, String, String, String, Value, DateTime<Utc>, String);
type RoomAndCount = (Uuid, String, String, String, Value, DateTime<Utc>, i64);

#[derive(Clone)]
pub struct Store {
    pub db: PgPool,
    pub sealer: Sealer,
    /// World's moderators, by QOR ID username, lower-cased (`SOCIAL_WORLD_MODERATORS`).
    world_moderators: Arc<Vec<String>>,
    /// World's id, read once: the migration that made it is its only maker.
    world: Arc<OnceLock<Uuid>>,
    /// godmode's QOR ID username, lower-cased (`SOCIAL_GODMODE`); empty when none is named.
    godmode: Arc<String>,
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
    /// The language they read in, and write in when their words are too few to tell (ADR-092 decision 3).
    pub language: Option<String>,
    pub seen_at: DateTime<Utc>,
    /// The account's standing: `godmode`, `elite`, or none (ADR-093). Not a column: worked out when read.
    #[sqlx(skip)]
    pub status: Option<String>,
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
    /// Until when the reader may not post here, if a moderator muted them.
    pub muted_until: Option<DateTime<Utc>>,
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
    /// The language it was written in, as its sender's launcher found it.
    pub lang: Option<String>,
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
    lang: Option<String>,
}

/// A friend request, as either side sees it: the other person, and when it was asked.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct FriendRequest {
    pub sub: Uuid,
    pub created_at: DateTime<Utc>,
}

/// A person's own friends, and the requests between them and others.
#[derive(Debug, Clone, Serialize)]
pub struct Friends {
    pub friends: Vec<Uuid>,
    /// Asked of them, waiting for their answer.
    pub incoming: Vec<FriendRequest>,
    /// Asked by them, waiting for the other's.
    pub outgoing: Vec<FriendRequest>,
}

/// One of a person's circles. Only its owner is ever shown it.
#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Circle {
    pub id: Uuid,
    pub name: String,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
    pub members: Vec<Uuid>,
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
    pub language: Option<String>,
}

/// A field that was sent, even as `null`: serde reads a `null` `Option<Option<_>>` as absent without this.
pub(crate) fn present<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
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
        Self {
            db,
            sealer,
            world_moderators: Arc::default(),
            world: Arc::default(),
            godmode: Arc::default(),
        }
    }

    /// Name the account that stands above every room (ADR-093), by its QOR ID username, which is fixed once chosen.
    pub fn with_godmode(mut self, name: &str) -> Self {
        self.godmode = Arc::new(name.trim().to_ascii_lowercase());
        self
    }

    /// Whether `sub` is godmode, and whether it is Elite.
    pub async fn standing(&self, sub: Uuid) -> SocialResult<(bool, bool)> {
        let row: Option<(bool, bool)> = sqlx::query_as(
            "SELECT $2 <> '' AND LOWER(p.qor_id) = $2, EXISTS (SELECT 1 FROM elites e WHERE e.sub = p.sub)              FROM profiles p WHERE p.sub = $1",
        )
        .bind(sub)
        .bind(self.godmode.as_str())
        .fetch_optional(&self.db)
        .await?;
        Ok(row.unwrap_or((false, false)))
    }

    /// Where someone holding `level` in a room stands there, their account's standing counted.
    async fn rank_with(&self, sub: Uuid, level: &str) -> SocialResult<u8> {
        let (godmode, elite) = self.standing(sub).await?;
        Ok(if godmode {
            GODMODE_RANK
        } else if elite {
            rank(level).max(ELITE_RANK)
        } else {
            rank(level)
        })
    }

    /// The caller's authority in a room they are in.
    pub async fn authority(&self, sub: Uuid, room: Uuid) -> SocialResult<Authority> {
        let (room, role) = self.membership(sub, room).await?;
        let rank = self.rank_with(sub, &role).await?;
        Ok(Authority { room, role, rank })
    }

    /// Where someone stands in a room: as its member, or as a member would if they have left it.
    pub async fn rank_in(&self, room: Uuid, sub: Uuid) -> SocialResult<u8> {
        let role: Option<String> =
            sqlx::query_scalar("SELECT role FROM members WHERE room = $1 AND sub = $2")
                .bind(room)
                .bind(sub)
                .fetch_optional(&self.db)
                .await?;
        self.rank_with(sub, role.as_deref().unwrap_or("member"))
            .await
    }

    /// A member's level and standing, or not found when they are not in the room.
    async fn target(&self, room: Uuid, sub: Uuid) -> SocialResult<u8> {
        let role: Option<String> =
            sqlx::query_scalar("SELECT role FROM members WHERE room = $1 AND sub = $2")
                .bind(room)
                .bind(sub)
                .fetch_optional(&self.db)
                .await?;
        let role = role.ok_or_else(|| SocialError::not_found("they are not in this room"))?;
        self.rank_with(sub, &role).await
    }

    /// The levels above member that `subs` hold in `room`, for the badges beside their names.
    pub async fn levels_of(&self, room: Uuid, subs: &[Uuid]) -> SocialResult<Vec<(Uuid, String)>> {
        Ok(sqlx::query_as(
            "SELECT sub, role FROM members WHERE room = $1 AND sub = ANY($2) AND role <> 'member'",
        )
        .bind(room)
        .bind(subs)
        .fetch_all(&self.db)
        .await?)
    }

    /// Mark each profile with its account's standing: `godmode`, `elite`, or nothing.
    async fn stamped(&self, mut profiles: Vec<Profile>) -> SocialResult<Vec<Profile>> {
        let subs: Vec<Uuid> = profiles.iter().map(|p| p.sub).collect();
        let elites: Vec<Uuid> = sqlx::query_scalar("SELECT sub FROM elites WHERE sub = ANY($1)")
            .bind(&subs)
            .fetch_all(&self.db)
            .await?;
        for p in &mut profiles {
            p.status = if !self.godmode.is_empty() && p.qor_id.to_ascii_lowercase() == *self.godmode
            {
                Some("godmode".into())
            } else if elites.contains(&p.sub) {
                Some("elite".into())
            } else {
                None
            };
        }
        Ok(profiles)
    }

    async fn stamped_one(&self, profile: Profile) -> SocialResult<Profile> {
        Ok(self
            .stamped(vec![profile])
            .await?
            .pop()
            .expect("one profile in, one out"))
    }

    // ── Elite (ADR-093) ──────────────────────────────────────────────────────────────────────────────────────────

    /// Make an account Elite. Only godmode does.
    pub async fn grant_elite(&self, by: Uuid, target: Uuid) -> SocialResult<()> {
        if !self.standing(by).await?.0 {
            return Err(SocialError::forbidden(
                "only godmode makes an account Elite",
            ));
        }
        if target == by {
            return Err(SocialError::bad("godmode is above Elite already"));
        }
        if self.profiles(&[target]).await?.is_empty() {
            return Err(SocialError::not_found(
                "nobody of that name has come to Social yet",
            ));
        }
        sqlx::query("INSERT INTO elites (sub, granted_by) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(target)
            .bind(by)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    /// Take Elite away. Only godmode does.
    pub async fn revoke_elite(&self, by: Uuid, target: Uuid) -> SocialResult<()> {
        if !self.standing(by).await?.0 {
            return Err(SocialError::forbidden(
                "only godmode makes an account Elite",
            ));
        }
        sqlx::query("DELETE FROM elites WHERE sub = $1")
            .bind(target)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn elites(&self) -> SocialResult<Vec<Uuid>> {
        Ok(
            sqlx::query_scalar("SELECT sub FROM elites ORDER BY granted_at")
                .fetch_all(&self.db)
                .await?,
        )
    }

    /// Name World's moderators by QOR ID username. A name is matched without regard to case, and only once its holder
    /// has chosen it, so an automatic `key_…` name never moderates.
    pub fn with_world_moderators(mut self, names: impl IntoIterator<Item = String>) -> Self {
        self.world_moderators = Arc::new(
            names
                .into_iter()
                .map(|n| n.trim().to_ascii_lowercase())
                .filter(|n| !n.is_empty())
                .collect(),
        );
        self
    }

    /// World's id.
    pub async fn world(&self) -> SocialResult<Uuid> {
        if let Some(id) = self.world.get() {
            return Ok(*id);
        }
        let id: Uuid = sqlx::query_scalar("SELECT id FROM rooms WHERE kind = 'world'")
            .fetch_one(&self.db)
            .await?;
        Ok(*self.world.get_or_init(|| id))
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

    /// Record a person as QOR ID describes them now, keeping what they made of their profile here, and put them in
    /// World: a newcomer starts at its newest message, and a name in `SOCIAL_WORLD_MODERATORS` is at least a moderator
    /// there; a level appointed in World since is kept (ADR-093).
    pub async fn arrive(&self, person: &Person) -> SocialResult<Profile> {
        let moderates = person.username_chosen
            && self
                .world_moderators
                .contains(&person.qor_id.to_ascii_lowercase());
        sqlx::query(
            "INSERT INTO members (room, sub, role, last_read) \
             SELECT id, $1, $2, next_seq - 1 FROM rooms WHERE kind = 'world' \
             ON CONFLICT (room, sub) DO UPDATE SET role = CASE \
               WHEN EXCLUDED.role = 'moderator' AND members.role IN ('member', 'vip') THEN 'moderator' \
               ELSE members.role END",
        )
        .bind(person.sub)
        .bind(if moderates { "moderator" } else { "member" })
        .execute(&self.db)
        .await?;
        let profile = sqlx::query_as(
            "INSERT INTO profiles (sub, qor_id, avatar_url, level, seen_at) VALUES ($1, $2, $3, $4, NOW()) \
             ON CONFLICT (sub) DO UPDATE SET qor_id = $2, avatar_url = $3, level = $4, seen_at = NOW() \
             RETURNING sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, language, seen_at",
        )
        .bind(person.sub)
        .bind(&person.qor_id)
        .bind(&person.avatar_url)
        .bind(person.level)
        .fetch_one(&self.db)
        .await?;
        self.stamped_one(profile).await
    }

    pub async fn profiles(&self, subs: &[Uuid]) -> SocialResult<Vec<Profile>> {
        let found = sqlx::query_as(
            "SELECT sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, language, seen_at \
             FROM profiles WHERE sub = ANY($1)",
        )
        .bind(subs)
        .fetch_all(&self.db)
        .await?;
        self.stamped(found).await
    }

    pub async fn profile_by_name(&self, qor_id: &str) -> SocialResult<Profile> {
        let found = sqlx::query_as(
            "SELECT sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, language, seen_at \
             FROM profiles WHERE LOWER(qor_id) = LOWER($1)",
        )
        .bind(qor_id.trim())
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| SocialError::not_found("nobody of that name has come to Social yet"))?;
        self.stamped_one(found).await
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
        let language = change.language.as_deref().map(text::language).transpose()?;
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
        let changed = sqlx::query_as(
            "UPDATE profiles SET \
               display = COALESCE($2, display), bio = COALESCE($3, bio), \
               accent = CASE WHEN $4 THEN $5 ELSE accent END, \
               banner = COALESCE($6, banner), showcase = COALESCE($7, showcase), \
               status_line = COALESCE($8, status_line), language = COALESCE($9, language), updated_at = NOW() \
             WHERE sub = $1 \
             RETURNING sub, qor_id, avatar_url, level, display, bio, accent, banner, showcase, status_line, language, seen_at",
        )
        .bind(sub)
        .bind(display)
        .bind(bio)
        .bind(accent_given)
        .bind(accent)
        .bind(banner)
        .bind(showcase)
        .bind(status)
        .bind(language)
        .fetch_one(&self.db)
        .await?;
        self.stamped_one(changed).await
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
            Option<DateTime<Utc>>,
        );
        let rows: Vec<Row> = sqlx::query_as(
            "SELECT r.id, r.kind, r.name, r.topic, r.ambience, r.created_at, m.role, \
                    (SELECT COUNT(*) FROM messages x WHERE x.room = r.id AND x.seq > m.last_read AND x.author <> $1 \
                       AND x.deleted_at IS NULL \
                       AND x.author NOT IN (SELECT blocked FROM blocks WHERE blocker = $1)), \
                    r.next_seq - 1, \
                    (SELECT COUNT(*) FROM members c WHERE c.room = r.id), \
                    (SELECT o.sub FROM members o WHERE o.room = r.id AND o.sub <> $1 AND r.kind = 'direct' LIMIT 1), \
                    (SELECT u.until FROM mutes u WHERE u.room = r.id AND u.sub = $1 AND u.until > NOW()) \
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
                    muted_until,
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
                    muted_until,
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
            // Everyone is in World from their first hello.
            "world" => return Ok(room_row),
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
        if found.kind == "world" {
            return Err(SocialError::forbidden(
                "World is everyone's, and is not left",
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
        let a = self.authority(by, room).await?;
        if a.room.kind == "world" {
            return Err(SocialError::forbidden("World's name and topic are fixed"));
        }
        if a.room.kind == "direct" || a.rank < rank("admin") {
            return Err(SocialError::forbidden(
                "only a room's owner and admins change it",
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
             ORDER BY CASE role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 WHEN 'moderator' THEN 2 WHEN 'vip' THEN 3 \
               ELSE 4 END, joined_at",
        )
        .bind(room)
        .fetch_all(&self.db)
        .await?)
    }

    /// Invite someone Social knows to a room the inviter moderates or owns.
    pub async fn invite(&self, by: Uuid, room: Uuid, invitee: Uuid) -> SocialResult<()> {
        let a = self.authority(by, room).await?;
        if a.room.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation is between two people",
            ));
        }
        if a.room.kind == "world" {
            return Err(SocialError::forbidden("everyone is in World already"));
        }
        if a.room.kind == "private" && a.rank < rank("moderator") {
            return Err(SocialError::forbidden(
                "only a private room's owner, admins and moderators invite",
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

    /// Set someone's level (ADR-093). An Admin or above sets the level of someone below them, to a level below their
    /// own; only the Owner, or godmode, hands the room on, and the Owner who hands it on becomes an Admin. World has no
    /// Owner; its levels are set by Elite accounts and godmode.
    pub async fn set_role(
        &self,
        by: Uuid,
        room: Uuid,
        target: Uuid,
        role: &str,
    ) -> SocialResult<()> {
        let a = self.authority(by, room).await?;
        if a.room.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation has no levels",
            ));
        }
        if !LEVELS.contains(&role) {
            return Err(SocialError::bad(
                "a level is owner, admin, moderator, vip or member",
            ));
        }
        if target == by {
            return Err(SocialError::bad(
                "hand the room to someone else to change your own level",
            ));
        }
        if a.rank < rank("admin") {
            return Err(SocialError::forbidden(
                "only a room's owner and admins set levels",
            ));
        }
        if self.target(room, target).await? >= a.rank {
            return Err(SocialError::forbidden(
                "you set the level only of someone below your own",
            ));
        }
        if role == "owner" {
            if a.room.kind == "world" {
                return Err(SocialError::forbidden("World has no owner"));
            }
            if a.role != "owner" && a.rank < GODMODE_RANK {
                return Err(SocialError::forbidden("only the room's owner hands it on"));
            }
        } else if rank(role) >= a.rank {
            return Err(SocialError::forbidden(
                "you give only a level below your own",
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
            // One owner: whoever held it (the giver, or the owner godmode replaced) is an Admin now.
            sqlx::query("UPDATE members SET role = 'admin' WHERE room = $1 AND role = 'owner' AND sub <> $2")
                .bind(room)
                .bind(target)
                .execute(&mut *tx)
                .await?;
        }
        tx.commit().await?;
        Ok(())
    }

    /// Remove someone from a room: a Moderator or above removes someone below them (ADR-093).
    pub async fn remove(&self, by: Uuid, room: Uuid, target: Uuid) -> SocialResult<()> {
        let a = self.authority(by, room).await?;
        let found = &a.room;
        if found.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation is between two people",
            ));
        }
        if found.kind == "world" {
            return Err(SocialError::forbidden(
                "nobody is removed from World; a moderator mutes them instead",
            ));
        }
        let below = self.target(room, target).await? < a.rank;
        if target == by || a.rank < rank("moderator") || !below {
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
            lang: stored.lang,
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
            "SELECT id, room, seq, author, key_version, nonce, sealed, reply_to, edited_at, deleted_at, created_at, lang \
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
        lang: Option<&str>,
    ) -> SocialResult<(Message, bool)> {
        let body = text::message(body)?;
        let lang = lang.map(text::language).transpose()?;
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
        if let Some(until) = self.muted_until(room, author).await? {
            return Err(SocialError::forbidden(format!(
                "a moderator muted you here until {} UTC",
                until.format("%H:%M on %-d %B")
            )));
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
            "INSERT INTO messages (id, room, seq, author, key_version, nonce, sealed, reply_to, client_nonce, lang) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) \
             RETURNING id, room, seq, author, key_version, nonce, sealed, reply_to, edited_at, deleted_at, created_at, lang",
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
        .bind(&lang)
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

    /// Change your own message's words, and the language they are in when the launcher says so.
    pub async fn edit(
        &self,
        author: Uuid,
        message: Uuid,
        body: &str,
        lang: Option<&str>,
    ) -> SocialResult<Message> {
        let body = text::message(body)?;
        let lang = lang.map(text::language).transpose()?;
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
            "UPDATE messages SET key_version = $2, nonce = $3, sealed = $4, lang = COALESCE($5, lang), \
               edited_at = NOW() \
             WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(message)
        .bind(sealed.key_version)
        .bind(&sealed.nonce)
        .bind(&sealed.sealed)
        .bind(&lang)
        .execute(&self.db)
        .await?;
        self.message_by_id(message).await
    }

    /// Delete a message: its author, or a Moderator or above whose level is above the author's. Its words go; its place
    /// stays.
    pub async fn delete(&self, by: Uuid, message: Uuid) -> SocialResult<Message> {
        let found = self.message_by_id(message).await?;
        let a = self.authority(by, found.room).await?;
        if found.author != by
            && (a.rank < rank("moderator")
                || self.rank_in(found.room, found.author).await? >= a.rank)
        {
            return Err(SocialError::forbidden(
                "you delete only your own messages, and those of people below you",
            ));
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
            "SELECT id, room, seq, author, key_version, nonce, sealed, reply_to, edited_at, deleted_at, created_at, lang \
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
        // A block ends a friendship, any request between the two, and each one's place in the other's circles.
        self.end_friendship(sub, other).await?;
        sqlx::query(
            "DELETE FROM friend_requests WHERE (requester = $1 AND addressee = $2) OR (requester = $2 AND addressee = $1)",
        )
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

    // ── friends (ADR-092 decision 2) ────────────────────────────────────────────────────────────────────────────

    pub async fn friends(&self, sub: Uuid) -> SocialResult<Friends> {
        let friends = sqlx::query_scalar(
            "SELECT CASE WHEN low = $1 THEN high ELSE low END FROM friendships \
             WHERE low = $1 OR high = $1 ORDER BY since",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?;
        let incoming = sqlx::query_as(
            "SELECT requester AS sub, created_at FROM friend_requests WHERE addressee = $1 \
               AND requester NOT IN (SELECT blocked FROM blocks WHERE blocker = $1) ORDER BY created_at DESC",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?;
        let outgoing = sqlx::query_as(
            "SELECT addressee AS sub, created_at FROM friend_requests WHERE requester = $1 ORDER BY created_at DESC",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?;
        Ok(Friends {
            friends,
            incoming,
            outgoing,
        })
    }

    pub async fn are_friends(&self, a: Uuid, b: Uuid) -> SocialResult<bool> {
        let (low, high) = if a < b { (a, b) } else { (b, a) };
        Ok(sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM friendships WHERE low = $1 AND high = $2)",
        )
        .bind(low)
        .bind(high)
        .fetch_one(&self.db)
        .await?)
    }

    /// Ask `to` to be friends. If they had already asked `from`, the two are friends at once, and this answers true.
    pub async fn request_friend(&self, from: Uuid, to: Uuid) -> SocialResult<bool> {
        if from == to {
            return Err(SocialError::bad("a friend is someone else"));
        }
        if self.blocked_either_way(from, to).await? {
            return Err(SocialError::forbidden(
                "you cannot befriend someone either of you has blocked",
            ));
        }
        if self.are_friends(from, to).await? {
            return Err(SocialError::Conflict("you are friends already".into()));
        }
        let mut tx = self.db.begin().await?;
        let they_asked =
            sqlx::query("DELETE FROM friend_requests WHERE requester = $1 AND addressee = $2")
                .bind(to)
                .bind(from)
                .execute(&mut *tx)
                .await?
                .rows_affected()
                > 0;
        if they_asked {
            let (low, high) = if from < to { (from, to) } else { (to, from) };
            sqlx::query(
                "INSERT INTO friendships (low, high) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(low)
            .bind(high)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO friend_requests (requester, addressee) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            )
            .bind(from)
            .bind(to)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;
        Ok(they_asked)
    }

    /// Accept `from`'s request.
    pub async fn accept_friend(&self, me: Uuid, from: Uuid) -> SocialResult<()> {
        let mut tx = self.db.begin().await?;
        let asked =
            sqlx::query("DELETE FROM friend_requests WHERE requester = $1 AND addressee = $2")
                .bind(from)
                .bind(me)
                .execute(&mut *tx)
                .await?
                .rows_affected();
        if asked == 0 {
            return Err(SocialError::not_found("they have not asked to be friends"));
        }
        sqlx::query("DELETE FROM friend_requests WHERE requester = $1 AND addressee = $2")
            .bind(me)
            .bind(from)
            .execute(&mut *tx)
            .await?;
        let (low, high) = if me < from { (me, from) } else { (from, me) };
        sqlx::query("INSERT INTO friendships (low, high) VALUES ($1, $2) ON CONFLICT DO NOTHING")
            .bind(low)
            .bind(high)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    /// End the request from `requester` to `addressee`, unanswered: declined by one, or taken back by the other.
    pub async fn drop_request(&self, requester: Uuid, addressee: Uuid) -> SocialResult<()> {
        let gone =
            sqlx::query("DELETE FROM friend_requests WHERE requester = $1 AND addressee = $2")
                .bind(requester)
                .bind(addressee)
                .execute(&self.db)
                .await?
                .rows_affected();
        if gone == 0 {
            return Err(SocialError::not_found("there is no such request"));
        }
        Ok(())
    }

    /// End a friendship, and take each out of the other's circles. True when there was one.
    pub async fn end_friendship(&self, a: Uuid, b: Uuid) -> SocialResult<bool> {
        let (low, high) = if a < b { (a, b) } else { (b, a) };
        let mut tx = self.db.begin().await?;
        let ended = sqlx::query("DELETE FROM friendships WHERE low = $1 AND high = $2")
            .bind(low)
            .bind(high)
            .execute(&mut *tx)
            .await?
            .rows_affected()
            > 0;
        sqlx::query(
            "DELETE FROM circle_members m USING circles c WHERE m.circle = c.id \
               AND ((c.owner = $1 AND m.sub = $2) OR (c.owner = $2 AND m.sub = $1))",
        )
        .bind(a)
        .bind(b)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(ended)
    }

    // ── circles: a person's own groups of friends ───────────────────────────────────────────────────────────────

    pub async fn circles(&self, owner: Uuid) -> SocialResult<Vec<Circle>> {
        Ok(sqlx::query_as(
            "SELECT c.id, c.name, c.color, c.created_at, \
                    COALESCE(ARRAY_AGG(m.sub ORDER BY m.added_at) FILTER (WHERE m.sub IS NOT NULL), '{}') AS members \
             FROM circles c LEFT JOIN circle_members m ON m.circle = c.id \
             WHERE c.owner = $1 GROUP BY c.id ORDER BY c.created_at",
        )
        .bind(owner)
        .fetch_all(&self.db)
        .await?)
    }

    /// One of `owner`'s circles, or not found: another's circle is never told of.
    pub async fn circle(&self, owner: Uuid, circle: Uuid) -> SocialResult<Circle> {
        sqlx::query_as(
            "SELECT c.id, c.name, c.color, c.created_at, \
                    COALESCE(ARRAY_AGG(m.sub ORDER BY m.added_at) FILTER (WHERE m.sub IS NOT NULL), '{}') AS members \
             FROM circles c LEFT JOIN circle_members m ON m.circle = c.id \
             WHERE c.id = $1 AND c.owner = $2 GROUP BY c.id",
        )
        .bind(circle)
        .bind(owner)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| SocialError::not_found("no such circle"))
    }

    pub async fn create_circle(
        &self,
        owner: Uuid,
        name: &str,
        color: Option<&str>,
    ) -> SocialResult<Circle> {
        let name = text::line(name, "a circle's name", 1, 32)?;
        let color = color.map(accent).transpose()?;
        let held: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM circles WHERE owner = $1")
            .bind(owner)
            .fetch_one(&self.db)
            .await?;
        if held >= CIRCLES_MAX {
            return Err(SocialError::Conflict(format!(
                "a person keeps at most {CIRCLES_MAX} circles"
            )));
        }
        let id = Uuid::new_v4();
        sqlx::query("INSERT INTO circles (id, owner, name, color) VALUES ($1, $2, $3, $4)")
            .bind(id)
            .bind(owner)
            .bind(&name)
            .bind(&color)
            .execute(&self.db)
            .await?;
        self.circle(owner, id).await
    }

    pub async fn change_circle(
        &self,
        owner: Uuid,
        circle: Uuid,
        name: Option<&str>,
        color: Option<Option<&str>>,
    ) -> SocialResult<Circle> {
        self.circle(owner, circle).await?;
        let name = name
            .map(|n| text::line(n, "a circle's name", 1, 32))
            .transpose()?;
        let color_given = color.is_some();
        let color = color.flatten().map(accent).transpose()?;
        sqlx::query(
            "UPDATE circles SET name = COALESCE($3, name), color = CASE WHEN $4 THEN $5 ELSE color END \
             WHERE id = $1 AND owner = $2",
        )
        .bind(circle)
        .bind(owner)
        .bind(name)
        .bind(color_given)
        .bind(color)
        .execute(&self.db)
        .await?;
        self.circle(owner, circle).await
    }

    pub async fn delete_circle(&self, owner: Uuid, circle: Uuid) -> SocialResult<()> {
        let gone = sqlx::query("DELETE FROM circles WHERE id = $1 AND owner = $2")
            .bind(circle)
            .bind(owner)
            .execute(&self.db)
            .await?
            .rows_affected();
        if gone == 0 {
            return Err(SocialError::not_found("no such circle"));
        }
        Ok(())
    }

    /// Put a friend in one of your circles.
    pub async fn circle_add(&self, owner: Uuid, circle: Uuid, sub: Uuid) -> SocialResult<Circle> {
        let found = self.circle(owner, circle).await?;
        if !self.are_friends(owner, sub).await? {
            return Err(SocialError::forbidden(
                "only your friends join your circles",
            ));
        }
        if !found.members.contains(&sub) && found.members.len() as i64 >= CIRCLE_MEMBERS_MAX {
            return Err(SocialError::Conflict(format!(
                "a circle holds at most {CIRCLE_MEMBERS_MAX} friends"
            )));
        }
        sqlx::query(
            "INSERT INTO circle_members (circle, sub) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(circle)
        .bind(sub)
        .execute(&self.db)
        .await?;
        self.circle(owner, circle).await
    }

    pub async fn circle_remove(
        &self,
        owner: Uuid,
        circle: Uuid,
        sub: Uuid,
    ) -> SocialResult<Circle> {
        self.circle(owner, circle).await?;
        sqlx::query("DELETE FROM circle_members WHERE circle = $1 AND sub = $2")
            .bind(circle)
            .bind(sub)
            .execute(&self.db)
            .await?;
        self.circle(owner, circle).await
    }

    // ── mutes (ADR-092 decision 1) ──────────────────────────────────────────────────────────────────────────────

    /// Until when `sub` is muted in `room`, if they are now.
    pub async fn muted_until(&self, room: Uuid, sub: Uuid) -> SocialResult<Option<DateTime<Utc>>> {
        Ok(sqlx::query_scalar(
            "SELECT until FROM mutes WHERE room = $1 AND sub = $2 AND until > NOW()",
        )
        .bind(room)
        .bind(sub)
        .fetch_optional(&self.db)
        .await?)
    }

    /// A Moderator or above stops someone below them posting in a room for one of [`MUTE_MINUTES`] (ADR-092, ADR-093).
    pub async fn mute(
        &self,
        by: Uuid,
        room: Uuid,
        target: Uuid,
        minutes: i32,
    ) -> SocialResult<DateTime<Utc>> {
        let a = self.authority(by, room).await?;
        if a.room.kind == "direct" {
            return Err(SocialError::forbidden(
                "a direct conversation is ended by a block, not a mute",
            ));
        }
        if a.rank < rank("moderator") {
            return Err(SocialError::forbidden(
                "only a room's moderators and those above them mute",
            ));
        }
        if !MUTE_MINUTES.contains(&minutes) {
            return Err(SocialError::bad(
                "a mute lasts ten minutes, an hour, a day or a week",
            ));
        }
        if target == by || self.target(room, target).await? >= a.rank {
            return Err(SocialError::forbidden(
                "you mute only someone below your own level",
            ));
        }
        Ok(sqlx::query_scalar(
            "INSERT INTO mutes (room, sub, until, muted_by) VALUES ($1, $2, NOW() + make_interval(mins => $3), $4) \
             ON CONFLICT (room, sub) DO UPDATE SET until = EXCLUDED.until, muted_by = EXCLUDED.muted_by, \
               created_at = NOW() \
             RETURNING until",
        )
        .bind(room)
        .bind(target)
        .bind(minutes)
        .bind(by)
        .fetch_one(&self.db)
        .await?)
    }

    pub async fn unmute(&self, by: Uuid, room: Uuid, target: Uuid) -> SocialResult<()> {
        let a = self.authority(by, room).await?;
        if a.room.kind == "direct" || a.rank < rank("moderator") {
            return Err(SocialError::forbidden(
                "only a room's moderators and those above them mute",
            ));
        }
        sqlx::query("DELETE FROM mutes WHERE room = $1 AND sub = $2")
            .bind(room)
            .bind(target)
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

    /// `sub`'s friends, and everyone who shares a room other than World with them, less anyone either way blocked: those
    /// who see their presence. World makes nobody a contact (ADR-092 decision 1).
    pub async fn contacts(&self, sub: Uuid) -> SocialResult<Vec<Uuid>> {
        Ok(sqlx::query_scalar(
            "SELECT c.sub FROM ( \
               SELECT o.sub FROM members m JOIN members o ON o.room = m.room JOIN rooms r ON r.id = m.room \
                 WHERE m.sub = $1 AND o.sub <> $1 AND r.kind <> 'world' \
               UNION \
               SELECT CASE WHEN f.low = $1 THEN f.high ELSE f.low END FROM friendships f \
                 WHERE f.low = $1 OR f.high = $1 \
             ) c \
             WHERE c.sub NOT IN (SELECT blocker FROM blocks WHERE blocked = $1) \
               AND c.sub NOT IN (SELECT blocked FROM blocks WHERE blocker = $1)",
        )
        .bind(sub)
        .fetch_all(&self.db)
        .await?)
    }

    /// Everyone who blocks `sub`: who does not hear from them, in a room as large as World.
    pub async fn blockers_of(&self, sub: Uuid) -> SocialResult<Vec<Uuid>> {
        Ok(
            sqlx::query_scalar("SELECT blocker FROM blocks WHERE blocked = $1")
                .bind(sub)
                .fetch_all(&self.db)
                .await?,
        )
    }

    /// Those of `subs` who are in `room`, with their roles: a room as large as World lists who is here, not everyone.
    pub async fn members_among(&self, room: Uuid, subs: &[Uuid]) -> SocialResult<Vec<Member>> {
        Ok(sqlx::query_as(
            "SELECT sub, role, joined_at FROM members WHERE room = $1 AND sub = ANY($2) \
             ORDER BY CASE role WHEN 'owner' THEN 0 WHEN 'admin' THEN 1 WHEN 'moderator' THEN 2 WHEN 'vip' THEN 3 \
               ELSE 4 END, joined_at",
        )
        .bind(room)
        .bind(subs)
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
