# ADR-079: Avatars live in QOR ID

**Status:** **Accepted**, 6 October 2026, by the project owner, who replied "I accept ADR-07." to this record, the one
just proposed to them. Proposed the same day. Decisions 1, 2 and 5 are the owner's direction of 6 October 2026 (an
avatar image or GIF per account, kept by QOR ID in its own database, shown to others when online, a letter when offline);
the rest were recommendations, accepted as written. It decides no economic value. Not yet built.

## Context

The owner asked for "a more modern avatar look and feel": a user picks an image or a GIF in the client, it synchronises
with QOR ID, and anyone else online sees it; offline, an avatar shows the first letter of the username. ADR-078's level
bubble and ring styles sit on the avatar, so it is where a person's progress is seen.

Today there is no avatar. QOR ID has an `avatar_url` column nothing writes and a `POST /api/v1/profile/avatar` that
answers "not implemented"; the launcher draws a person icon; ARQADE draws a fingerprint icon.

An image anyone can upload and everyone can see brings three risks of its own: a file can carry data its owner did not
mean to publish (a photo's GPS position), a crafted file can attack whatever decodes it, and a picture can be abusive or
illegal, which creates obligations for whoever hosts it.

## Decision

1. **One avatar per QOR ID, kept by QOR ID in its own Postgres database** (the owner's choice over a separate storage
   bucket), so the launcher, ARQADE and every later app show the same one.
2. **The client uploads; QOR ID serves.** The launcher uploads with the signed-in account's token. Anyone fetches an
   avatar from QOR ID's own address by a URL that names the image's content (`/avatars/<hash>`), so it can be cached for
   good and changes URL when the picture changes. Userinfo and the profile return the account's current avatar URL.
3. **QOR ID never stores what was sent.** It accepts PNG, JPEG, WebP and GIF up to 4 MB, decodes them, and stores a
   clean re-encoding: square, 256 x 256, every piece of metadata dropped. A GIF stays animated, at most 2 MB stored and
   at most 120 frames. Anything else, or anything that does not decode, is refused. QOR ID decodes in a library written
   in a memory-safe language, never by shelling out.
4. **Abuse.** Every avatar shown to others carries a **Report** action. A reported avatar is queued for the owner, who can
   remove it; a removed avatar falls back to the letter, and the account is told. ADR-078's abuse watcher is asked to scan
   new avatars too and **flag** any it suspects, never remove one on its own until the owner says it may. Illegal imagery
   is a matter for the owner's legal review: what must be reported and to whom.
5. **The letter.** With no avatar, or offline before a cached one is available, an avatar is a circle with the first
   letter of the username, its colour derived from the name, so it is the same everywhere. The launcher keeps a copy of
   its own account's avatar to show offline.
6. **Motion follows the person's setting.** With "reduce motion" on, a GIF shows its first frame only, as the launcher's
   other animation does.
7. **The look.** A circular avatar inside a thin ring in the unlocked ring style (colour and pattern; glow is ADR-078's
   open question), the level bubble on the ring, and on the launcher's Nexus the XP bar beside it. ARQADE shows avatars
   in lobbies, chat, the leaderboard and matches.

## Consequences

- **First build, if accepted:** QOR ID's upload, clean-up, serving, Report and removal, with the log check over the new
  routes; launcher 0.1.9 with the avatar, ring, bubble and XP bar, upload, and the offline copy; then ARQADE's avatars.
- A 256 x 256 image is tens of kilobytes; a GIF up to 2 MB. Thousands of accounts fit in Postgres comfortably. If
  avatars ever outgrow it, moving them to a bucket is a storage change behind the same URLs.
- Avatars are public: anyone who can see an account's QOR ID can see its picture. The upload screen says so.

## Not decided here

Whether ring styles may glow (ADR-078). The limits in decision 3 are engineering defaults the owner may change.
