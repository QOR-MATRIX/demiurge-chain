# Social

**What it is:** rooms, direct messages, presence and profiles per QOR ID, in a service of its own, with the QOR
Launcher as its first client. **Decided by** [ADR-091](../decisions/ADR-091-social.md) (accepted by the owner,
15 October 2026) and [ADR-092](../decisions/ADR-092-world-friends-and-translation.md) (World, friends and circles,
translation on the reader's PC; 16 October 2026) and [ADR-093](../decisions/ADR-093-room-levels-elite-and-godmode.md)
(room levels, Elite accounts and godmode; 16 October 2026). **Roadmap:** L7.3, whose steps are L8 in [`DIRECTION.md`](../DIRECTION.md). This document is the
design; what exists is in [`SYSTEMS.md`](../SYSTEMS.md), and what is done is ticked in `DIRECTION.md`, nowhere else.

The owner's brief, 15 October 2026: an engaging, full-featured social network and next-generation chat, with complete
personalised profiles per QOR ID username and customisation never seen in a chat, premium in look and feel, and
seamless onboarding on the QOR ID username a person chose, or, without one, choosing it there. And on 16 October 2026,
after the first conversation: circles and friendship; a world chat everyone on the launcher is entered into on opening
Social; real-time translation, so each person reads the chat in their own language, with the language it was written in
shown; avatars in chat with each person's level and username.

## 1. Shape

```
 QOR Launcher (Tauri)                                Social (services/social, Rust)            QOR ID
 ┌──────────────────────┐   one WebSocket, TLS       ┌──────────────────────────────┐          ┌──────────────┐
 │ interface (webview)  │◄─ Tauri events ─┐          │ socket: hello → session      │ introspect│ /oauth/      │
 │  Social surface      │── commands ──┐  │          │ rooms, members, messages     │──────────►│  introspect  │
 └──────────────────────┘              ▼  │          │ profiles, presence, reports  │           │ /api/v1/     │
 ┌──────────────────────┐   frames  ┌─────┴────┐     │ sealing (AEAD, key versions) │           │  profile/... │
 │ Rust host: token in  │──────────►│ social   │────►│ Postgres (its own database)  │           └──────────────┘
 │ OS keychain          │◄──────────│ client   │◄────│ fan-out per room             │
 └──────────────────────┘           └──────────┘     └──────────────────────────────┘
```

- **The launcher's Rust host holds the socket**, as it holds every QOR ID call: the token never reaches the webview,
  whose CSP stays as it is. Events reach the interface as Tauri events; the interface asks through commands.
- **Social never holds a credential of the person's.** A token arrives in the socket's first frame (never in a URL,
  which proxies log), is checked with QOR ID's introspection endpoint under Social's own client credential, and only
  its hash is cached, for at most 30 seconds or the token's own expiry, whichever is sooner. A token that stops being
  live closes the socket with a reason; the launcher refreshes and reconnects without asking anything (ADR-056).
- **One person, one key:** QOR ID's `sub`. Username, avatar and level are QOR ID's and are shown as it returns them.

## 2. Protocol

A frame is one JSON object of at most 64 KiB. The first frame from the client is `hello`; the service answers
`welcome` or closes.

```
→ {"type":"hello","version":1,"token":"<QOR ID access token>","client":"qor-launcher/0.1.9"}
← {"type":"welcome","version":1,"me":{…profile…},"rooms":[…summaries…],"presence":[…]}
→ {"type":"req","id":7,"method":"messages.send","params":{"room":"…","body":"…","reply_to":null,"nonce":"…"}}
← {"type":"res","id":7,"result":{"message":{…}}}            or {"type":"err","id":7,"code":"rate_limited","message":"…"}
← {"type":"event","event":"message.created","data":{…}}      (to every member's open sockets)
```

Methods are typed intents, never raw writes: `rooms.list | create | join | leave | invite | update | members`,
`messages.history | send | edit | delete | react | read`, `dm.open`, `profile.get | update`, `presence.set`,
`typing`, `block | unblock`, `report`; and since ADR-092 `friends.list | request | accept | decline | cancel | remove`,
`circles.list | create | update | delete | add | remove | open` and `world.mute | unmute`; and since ADR-093
`rooms.mute | unmute` and `elite.grant | revoke | list`. Events mirror them:
`message.created | edited | deleted | reaction`, `room.updated | member.joined | member.left`, `presence`, `typing`,
`profile.updated`, `read`, `friend.requested | accepted | declined | removed`, `world.muted` and `room.muted`. A
profile carries its account's `status` (godmode, Elite or none); a history page carries the `levels` its authors hold
there, a new message its author's `level`, and a room summary `muted_until`. A `nonce` on send makes
a retry after a dropped connection idempotent; a `lang` on send or edit says what language the words are in. The
welcome carries `world`: World's id, and until when the reader is muted there. Unknown methods and versions are refused
with a code, never ignored.

## 3. Data (Postgres, Social's own database)

| Table | Holds |
| --- | --- |
| `profiles` | `sub` (key), display line, bio, accent, banner (a QFX preset and its parameters), showcase (DRC-369 asset ids, checked against the chain when shown), status line, language, updated |
| `rooms` | id, kind (`public`, `private`, `direct`, `world`; one `world`), name, topic, ambience (a QFX preset), owner, created |
| `members` | room, `sub`, level (`owner`, `admin`, `moderator`, `vip`, `member`), joined, last read sequence |
| `elites` | an Elite account, who granted it, and when (only godmode does) |
| `messages` | room, sequence (per room, gapless), id, author, **sealed body** and its key version, reply-to, language, edited, deleted, created |
| `reactions` | message, `sub`, emoji (one row each) |
| `invites`, `blocks`, `reports` | who invited whom; who blocks whom; a message, its reporter, a reason, and its resolution |
| `mutes` | a room (World), a person, until when, and the moderator who muted them |
| `friend_requests`, `friendships` | who asked whom; each pair of friends once, the lower id first |
| `circles`, `circle_members` | a person's own named, coloured groups, and the friends in each |

- **Sealed at rest:** a body is encrypted with XChaCha20-Poly1305 under the service's message key of the current
  version, with the room and sequence as associated data, so a sealed body cannot be moved to another message. Keys
  come from the environment, never the database; old versions stay readable until re-sealed. Search is a later item
  (L8.10) and will need its own design, since sealed bodies cannot be searched in the database.
- **Every query is bound** (the rule `sql_hygiene.rs` enforces in QOR ID is adopted, and its check copied).
- **A direct conversation** is a `direct` room of exactly two members, opened by either and refused while either
  blocks the other.

## 4. Rules that are tested, not hoped for

- Only members read a room's messages or see its members; a private room is joined only by invitation; a direct room
  never gains a third member.
- A person edits and deletes only their own messages; moderators and owners delete others' and remove members; an
  owner cannot be removed, only hand the room on.
- Bodies are NFKC-normalised, control and bidirectional-override characters removed, 1 to 4,000 characters.
- Rate limits per person, enforced in the database so several sockets share one budget: sends, reactions, room creation.
- Nothing secret reaches a log: tokens, the client secret, message bodies and keys are never logged at any level; a
  runtime check drives every flow at TRACE and fails on any of them (AGENTS.md §9's rule, adopted from the first commit).
- A blocked person's messages, typing and presence do not reach the person who blocked them.
- World (ADR-092): everyone is in it from their first hello and nobody leaves, renames, invites to it or is removed from
  it; a newcomer starts at its newest message; it makes nobody a contact, so presence and typing do not cross it; its
  moderators come from configuration, never from a name its holder did not choose, and only they delete others' words
  and mute; a muted person reads but does not post there; it has a sending pace of its own.
- Friendship is asked and answered, never assumed; a block ends it and refuses a new one either way; friends see each
  other's presence; a circle is seen and changed only by its owner, holds only their friends, and loses a friend when
  the friendship ends.
- A message's language is kept only in a language tag's shape.
- Levels (ADR-093): each level acts only on those below it and gives only levels below its own; only the Owner, or
  godmode, hands a room on, and the Owner who does becomes an Admin; an Elite account acts as an Admin in every room it
  is in, never above its Owner; godmode stands above every Owner and alone makes accounts Elite; nobody acts on
  godmode; no level or standing reads a room it is not in; a level appointed in World outlasts the configured
  moderators' hello.

## 5. Profiles: what makes them a person's own

Every surface that shows a person opens their **card**: avatar with the level ring (ADR-079, ADR-080), username,
display line, status, and the banner. A full profile adds the bio and the **showcase**.

- **Banner:** one of the QFX backdrops (drift, aurora, nebula, lattice, starfield, liquid) with the person's hue and
  intensity, moving where motion is allowed and still where it is not. **As built:** drawn in CSS after the six presets
  rather than by the QFX shader, which is one full-window canvas; a CSS banner costs nothing per card, and text on it sits
  behind a scrim so it reads in every theme.
- **Accent:** a colour from the person's theme that tints their name, ring and message rail for everyone who reads
  them, checked for contrast against every theme before it is accepted.
- **Showcase:** up to six DRC-369 assets the account holds. **As built (15 October 2026):** a person chooses from the
  assets their own account holds, read from the chain when they choose; another person's showcase is shown by asset
  number and marked unchecked, because a Social profile does not yet carry the chain account to read it against. Checking
  it each time it is shown, so a sold asset leaves by itself, needs that account in the profile (from QOR ID's
  `chain_account`), and is not built.
- **Room ambience:** a room's owner chooses a backdrop that the launcher plays while the room is open.
- **Status and presence:** online, away or invisible; a status line; and "playing …" when a product the launcher
  started says so.

## 6. Onboarding

Entering Social never asks for anything already known.

1. **No QOR ID on this vault:** the launcher's existing claim (`Onboarding`) asks for the name first, then registers
   the key under it. That name is the person's QOR ID and their Social name.
2. **A QOR ID on an automatic `key_…` name:** a one-time claim asks for the name, checks it as it is typed, and calls
   QOR ID's claim; after that the name is fixed (ADR-091 decision 5, amending ADR-075).
3. **A QOR ID with a chosen name:** Social opens on the person's rooms, with a starter profile already made from
   their QOR ID (avatar, level, an accent from their theme), which they can shape at any time.

## 7. World

One room, made by the migration that adds its kind, with everyone in it (ADR-092 decision 1). The launcher pins it above
every other conversation and opens it when Social opens. A message there reaches whoever is connected, less those who
block its author; a member list there is who is here and showing, up to two hundred. World is kept in full (the owner's
choice). Its moderators are QOR ID usernames in `SOCIAL_WORLD_MODERATORS`: they delete any message and mute a person for
ten minutes, an hour, a day or a week. Typing is not shown in World. Its unread messages do not count on the launcher's
rail; a mention there does.

## 8. Friends and circles

Friendship is mutual (ADR-092 decision 2): a request by username, accepted or declined by the one asked, taken back by
the one who asked, ended by either; two people who ask each other are friends at once. Friends see each other's presence
and what they are playing. The launcher shows friends beside conversations, online first, with the requests waiting and
an add-by-name field. A circle is one person's group of their friends, with a name and a colour from the profile accent
palette, seen by nobody else; choosing it shows its friends, and "Group chat" opens a private room named after it with
its friends invited.

## 9. Translation on each reader's PC

World and public rooms are shown in the reader's language (ADR-092 decision 3); direct messages and private rooms are
not translated.

- **What a message carries:** `lang`, the language its sender's launcher found it to be in (`whatlang`, on the sender's
  PC), or the sender's own language when the words are too few to tell. A person's language is in their profile, set
  from the PC's language on first arrival and changed in the profile editor.
- **What the reader sees:** each message in another language translated, marked "Translated from Spanish", with "Show
  original" beside it; one in the reader's language unmarked; one in a language Mozilla has no model for shown as
  written, saying so. A switch in the room's header shows everything as written instead.
- **How:** the launcher's host reads Mozilla's public Remote Settings collections (`translations-models-v2`,
  `translations-wasm-v2`), plans the files a pair of languages needs (between two languages neither of which is
  English, two models through English), downloads each once, checks it against Mozilla's SHA-256 packed and unpacked
  with unpacking capped at the published size, and keeps it by hash with the launcher's local data. The engine is pinned
  by its hash to the build of Firefox's loader kept in `tools/qor-launcher/public/translate/`. The interface hands the
  bytes to a worker there, which translates on the reader's CPU. The reader is told what is being downloaded and how
  large, once per language pair. Translations are held in memory only.
- **Checked:** the host's catalogue, download and verification rules against a stand-in for Mozilla; the screen against a
  stand-in host with the worker in its stand-in mode (`check-social-view.mjs`); and, by hand, the real engine with
  Mozilla's real files (`check-translate-engine.mjs`).

## 10. Levels and standing

Every room has five levels (ADR-093): **Owner** (everything; hands the room on), **Admin** (settings; sets levels below
it), **Mod** (deletes, mutes and removes those below it; invites to a private room), **VIP** (a badge; free of World's
pace) and **Member**. Two standings sit across rooms: **Elite**, an Admin in every room it is in, made and unmade only by
godmode; and **godmode**, the account `SOCIAL_GODMODE` names, above every Owner. The launcher shows the badges beside
names on messages and in member lists, offers each person only the levels and tools Social would allow them (a level
menu, Mute, Remove in the member list; Delete and Mute on a message), and offers godmode Make Elite and Remove Elite on
a person's card.

## 11. Not in release 1

End-to-end encrypted direct messages (release 2, its own record first), voice, feeds, search across rooms, a web
client, ARQADE's chat moved onto Social, push notifications when the launcher is closed, and several instances of the
service (release 1 fans out in one process; several need a shared bus, which is designed when it is needed).
