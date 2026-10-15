# Social

**What it is:** rooms, direct messages, presence and profiles per QOR ID, in a service of its own, with the QOR
Launcher as its first client. **Decided by** [ADR-091](../decisions/ADR-091-social.md) (accepted by the owner,
15 October 2026). **Roadmap:** L7.3, whose steps are L8 in [`DIRECTION.md`](../DIRECTION.md). This document is the
design; what exists is in [`SYSTEMS.md`](../SYSTEMS.md), and what is done is ticked in `DIRECTION.md`, nowhere else.

The owner's brief, 15 October 2026: an engaging, full-featured social network and next-generation chat, with complete
personalised profiles per QOR ID username and customisation never seen in a chat, premium in look and feel, and
seamless onboarding on the QOR ID username a person chose, or, without one, choosing it there.

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
`typing`, `block | unblock`, `report`. Events mirror them: `message.created | edited | deleted | reaction`,
`room.updated | member.joined | member.left`, `presence`, `typing`, `profile.updated`, `read`. A `nonce` on send makes a
retry after a dropped connection idempotent. Unknown methods and versions are refused with a code, never ignored.

## 3. Data (Postgres, Social's own database)

| Table | Holds |
| --- | --- |
| `profiles` | `sub` (key), display line, bio, accent, banner (a QFX preset and its parameters), showcase (DRC-369 asset ids, checked against the chain when shown), status line, updated |
| `rooms` | id, kind (`public`, `private`, `direct`), name, topic, ambience (a QFX preset), owner, created |
| `members` | room, `sub`, role (`owner`, `moderator`, `member`), joined, last read sequence, muted |
| `messages` | room, sequence (per room, gapless), id, author, **sealed body** and its key version, reply-to, edited, deleted, created |
| `reactions` | message, `sub`, emoji (one row each) |
| `invites`, `blocks`, `reports` | who invited whom; who blocks whom; a message, its reporter, a reason, and its resolution |

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

## 5. Profiles: what makes them a person's own

Every surface that shows a person opens their **card**: avatar with the level ring (ADR-079, ADR-080), username,
display line, status, and the banner. A full profile adds the bio and the **showcase**.

- **Banner:** one of the QFX backdrops (drift, aurora, nebula, lattice, starfield, liquid) with the person's accent and
  parameters, rendered live where motion is allowed and as a still where it is not (the QFX contrast and frame-time
  contracts apply).
- **Accent:** a colour from the person's theme that tints their name, ring and message rail for everyone who reads
  them, checked for contrast against every theme before it is accepted.
- **Showcase:** up to six DRC-369 assets the account holds, read from the chain each time it is shown, so a sold asset
  leaves the showcase by itself.
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

## 7. Not in release 1

End-to-end encrypted direct messages (release 2, its own record first), voice, feeds, search across rooms, a web
client, ARQADE's chat moved onto Social, push notifications when the launcher is closed, and several instances of the
service (release 1 fans out in one process; several need a shared bus, which is designed when it is needed).
