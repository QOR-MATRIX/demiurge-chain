# ADR-075: A QOR ID is a unique name, with no `#0001`

**Status:** Accepted, 5 October 2026, by the project owner: "I do not want usernames to contain '#xxxx' after them at
all. There should NOT be multiple names per QOR ID." The owner also chose how existing shared names are resolved
(decision 3). It changes the identity form that [ADR-027](ADR-027-identity-stays-off-chain.md),
[ADR-049](ADR-049-data-ownership-and-the-user-directory.md) and [ADR-074](ADR-074-arqade-on-vercel.md) describe as
`name#0001`; each says so. It decides no economic value.

## Context

QOR ID was written with a Battle.net-style identity: a username plus a four-digit discriminator, `name#0001`, so that
several accounts could share a name. In practice every sign-up path for people already refused a taken name; only agent
accounts (`agent_…`) could share one. But the number was shown everywhere, and signing in by username picked the
account with the lowest number among those sharing it, which is not an identity a person can rely on. On 5 October the
owner, signing in to ARQADE, saw the number and ruled it out.

## Decision

1. **A QOR ID is the username alone**, lower-cased: `godmode`, never `godmode#0001`. Every response, page and token
   QOR ID issues carries it in that form; the QOR Launcher and ARQADE show it as sent.
2. **A name belongs to one account**, whatever its letter case. The database holds the rule (a unique index on
   `LOWER(username)`, migration 019), so a path that skipped the check is refused too; sign-up for people, for keys and
   for agents all refuse a taken name.
3. **Names already shared are resolved oldest-first** (the owner's choice): the earliest-created account keeps the
   name; each later one becomes `name_2`, `name_3`… (shortened to fit 20 characters, skipping names in use). The name it
   had is kept in `renamed_from`, which `GET /api/v1/profile` returns so the account can be told. Nothing is deleted.
   QOR ID logs, at start, how many accounts were renamed: a count, never a name.
4. **The discriminator is retired, not removed.** The column stays, always 1, and responses keep a `discriminator` of 1,
   so the QOR Launcher 0.1.6 already installed keeps working. An old `name#0001` string is still read, and its number
   ignored.
5. **Username rules are one rule:** 3 to 20 characters, ASCII letters, digits and underscore, in QOR ID and in the
   launcher (which had allowed a hyphen QOR ID refused).

## Consequences

- Evidence, 5 October 2026: QOR ID 145 tests (a name refused in any letter case and by the database itself; the
  profile's name and rename); the launcher 196; ARQADE 41. Migration 019 run on a database holding shared names: the
  oldest kept each name, later ones became `_3`, `_4` around an existing `_2`, a 20-character name was shortened, and a
  new `GODMODE` was refused beside `godmode`. A planted fault (the `#0001` form restored) failed a test.
- **A way to choose a new name** after a rename is not built. Only agent accounts could have shared a name, and an
  agent's name is its controller's; if the count logged at deploy shows a person was renamed, renaming is built then.
- A tutorial finished in the launcher under `name#0001` stays finished under `name`.
- Tokens issued before the change carry the old form until they expire (an app session within 8 hours).
