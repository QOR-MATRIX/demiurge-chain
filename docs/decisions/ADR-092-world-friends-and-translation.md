# ADR-092: World chat for everyone, friends and circles, and translation on each person's own PC

**Status:** **Accepted**, 16 October 2026, by the owner, after the first conversation on Social ("Successful. Looking
great so far"). The owner asked for "a circle / friendship system", "a general world chat page that everyone logged in to
QOR Launcher would be automatically entered into upon opening Social", "a real time translation mechanic … users can
enter text in their language and see the chat in that same language even if it was posted by another user using another
language, the originating language could be displayed in the chat", and "avatars in chat … displaying user level and
username", "carefully implemented with high quality premium look and feel". Asked four questions, the owner chose
**"Friends + circles"**, translation for **"World and public rooms"**, and to **"Keep everything"** said in World. For
the translator, the owner chose none of the four offered and wrote: "The best possible option with zero costs associated
with its use would be best." Decision 3 is how that is met; the engine and its source were chosen under the owner's
standing delegation of engineering choices. **Extends** [ADR-091](ADR-091-social.md); **decides part of L8.7** (how long
World keeps what is said in it), and leaves the rest of L8.7 open. It decides no economic value.

**Relates to:** [ADR-078](ADR-078-levels-tasks-and-the-welcome-grant.md) and [ADR-079](ADR-079-avatars-in-qor-id.md)
(level and avatar, shown in chat), [ADR-080](ADR-080-effects-are-allowed.md) (effects), [ADR-089](ADR-089-an-advisory-in-a-crate-the-launcher-never-builds.md)
(why the launcher does not build `ruzstd`).

## Context

Checked on 16 October 2026, against `services/social/` and `tools/qor-launcher/`:

- Social has public rooms (joined by anyone who finds them), private rooms by invitation, and direct rooms. **Who sees a
  person's presence is everyone who shares any room with them** (`Store::contacts`). A room everyone is in would make
  everyone everyone's contact.
- **A message already shows its author's avatar with the level ring** (`SocialView.tsx`, from QOR ID's avatar and level,
  which Social copies on each hello). It does not show the level as a number.
- **Nothing in Social knows what language anything is in.**
- **Translation that costs nothing to use is possible only on the reader's machine.** Every hosted translator (Claude,
  DeepL, Google, Azure) bills per message or per character past a free allowance. A self-hosted one (LibreTranslate) has
  no per-message price but costs Railway compute every hour it runs, and is the weakest of them. Firefox has translated
  on the user's own computer since 2023 with Mozilla's Bergamot engine (Marian, compiled to WebAssembly) and small
  per-language models. Mozilla publishes both, with their SHA-256, in public Remote Settings collections that Firefox
  release reads: `translations-wasm-v2` (the engine, `bergamot-translator` 4.0, release v0.6.0, 1.2 MB compressed, 5.0 MB
  unpacked) and `translations-models-v2` (version 3.x models; for a desktop, 57 languages to and from English: 114 pairs,
  a median of 24 MB per pair to download, at most 50 MB). The JavaScript that loads the engine is in Firefox's source
  (`toolkit/components/translations/bergamot-translator/bergamot-translator.js`, v0.6.0, MPL-2.0). Everything is
  compressed with zstd.
- The launcher's webview CSP allows scripts only from its own origin, and no WebAssembly.

## Decision

1. **World: one room everyone is in.** Social gains a room of a fourth kind, `world`, of which there is exactly one,
   named World, made by the migration that adds the kind. Every person is a member from their first hello on, and
   cannot leave it; nobody is invited to it, and it is not listed among public rooms to join. A newcomer starts at its
   newest message, with nothing unread. **World keeps everything said in it** (the owner's choice; part of L8.7).
   - **World does not make people contacts.** Presence goes to friends and to people who share another room; a message
     in World reaches whoever is connected at the time, less anyone who blocks its author.
   - **Its moderators are named in Social's configuration** (`SOCIAL_WORLD_MODERATORS`, QOR ID usernames, which are
     fixed once chosen). A moderator deletes any message in World, and **mutes** a person there for a set time (ten
     minutes, an hour, a day or a week); someone muted reads World but does not post in it until the time passes. World
     has its own, slower, sending limit.
   - What reports are judged against is still the owner's content policy (L8.7). **A launcher release that includes
     Social waits for that policy**, since World puts everyone in one room.
2. **Friends and circles.** Friendship is mutual: a person asks by username, the other accepts or declines, and either
   may end it. A request to or from someone either of them blocks is refused, and blocking ends a friendship and any
   request between the two. Friends see each other's presence. **A circle** is a person's own named, coloured group of
   their friends ("Squad", "Studio crew"); nobody else sees another's circles, or which circles they are in. Ending a
   friendship takes each out of the other's circles. A circle opens a private room named after it, with its friends
   invited, in one step. Limits: 30 friend requests an hour, 20 circles of up to 100 friends each.
3. **Translation runs on each reader's PC, with Mozilla's Bergamot engine and models** (decision 3 meets the owner's
   "zero costs associated with its use": nothing is paid per message, no service runs for it, and no message text
   leaves the reader's computer to be translated).
   - **What is translated:** World and public rooms (the owner's choice). Direct messages and private rooms are not.
   - **Every message carries the language it was written in** (`lang`, a language tag, which Social checks the shape of
     and stores beside the sealed body). The sender's launcher detects it on the sender's PC, and uses the sender's own
     language when the text is too short to tell. Each person has **a language** in their Social profile, which follows
     them to other devices, set from the PC's language on first use and changeable.
   - **A reader sees World and public rooms in their language**, each translated message marked with the language it
     came from, and its original one click away. A message in the reader's language, or in a language Mozilla has no
     model for, is shown as written. Translations are kept on the reader's PC only; Social never sees or stores one.
   - **Where the engine comes from:** the launcher's Rust host downloads the engine and a language's models from
     Mozilla's public collections on first use, with the reader told what is downloading and how large it is. It
     checks each file against the SHA-256 Mozilla publishes, before and after unpacking, and caps unpacking at the size
     published, so a corrupt or oversized file is refused, never used. The engine is pinned in the launcher's code by
     its hash, beside the loader it was built with (Firefox release's `bergamot-translator.js`, v0.6.0, kept in the
     launcher with its MPL-2.0 licence); a new engine is taken only by a launcher update. Files are cached by hash.
     Between two languages neither of which is English, the engine translates through English, as Firefox does.
   - **The webview's CSP gains `'wasm-unsafe-eval'`**, which lets it compile WebAssembly and nothing else (string
     `eval` stays forbidden). The engine runs in a worker from the launcher's own origin. Unpacking uses the `zstd`
     crate (the C library), not `ruzstd` (ADR-089).
4. **Avatars in chat show the level and the username.** Each run of messages from one author starts with their avatar
   inside the level ring, a level badge with the number, their username in their accent, and their display line; any
   of these opens their card.
5. **Names** (plain, under ADR-032): the room kind `world`; the methods `friends.list | request | accept | decline |
   cancel | remove`, `circles.list | create | update | delete | add | remove | open`, `world.mute | unmute`; the events
   `friend.requested | accepted | declined | removed` and `world.muted`; the message field `lang`, the profile field
   `language`, and `world` in the welcome; the launcher module `translate`, its commands `translate_languages | plan |
   prepare | file | detect` and its event `translate://progress`. The protocol stays at version 1: everything added is
   optional to an older client.

## Consequences

- **Mozilla's service is a dependency.** If its collections move, change shape, or Mozilla asks third parties not to
  read them, translation stops until the launcher reads a mirror of our own (for example a release on the public
  repository, which is free); chat itself is unaffected. Records are read over TLS and every file is checked against
  the record's hash, but the record list itself is trusted as Mozilla serves it; Firefox also checks a signature over
  the collection, which the launcher does not.
- A reader downloads 24 MB on average per language pair the first time, and twice that when translating through
  English. Translating a chat line takes milliseconds on a current PC once a model is loaded.
- Machine translation is sometimes wrong. The original is always one click away, and the language it came from is
  always shown.
- World is the first place everyone meets, so its moderators, mutes, reports and the owner's content policy carry
  more weight than in any other room. Message volume in World grows without limit, since it keeps everything.
- `SOCIAL_WORLD_MODERATORS` is a new variable on Railway's `social` service; until it is set, World has no
  moderators, and only authors delete their own messages there.
- Nothing here touches CGT, the chain or DRC-369 state.
