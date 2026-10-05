# ADR-049: A creator's work lives in their QOR ID directory, keeps working offline, and leaves in open formats

**Status:** **Accepted**, 28 September 2026, by the project owner. Proposed 21 September 2026.

**Identity form changed, 5 October 2026, by [ADR-075](ADR-075-one-name-per-qor-id.md):** a QOR ID is the username
alone, unique on its own; the `#0001` discriminator below is retired. The text is kept as it was written.

**Depends on:** [ADR-001](ADR-001-innovation-budget.md) (build boring where failure is silent),
[ADR-016](ADR-016-sign-in-with-unlock.md) (sign-in is an unlock),
[ADR-017](ADR-017-password-accounts-no-chain-identity-without-a-key.md) (an account may have no key at all),
[ADR-048](ADR-048-interchange-formats.md) (which formats an export is written in; **Proposed the same day**).

**Relates to:** the app-host record proposed the same day, which puts each product in its own OS process, gives it a
product id and keeps the vault in the launcher process. This record says where that process writes.

## Context

The launcher already has exactly one place on disk and one identifier. `tauri.conf.json:5` sets the bundle
identifier `cloud.demiurge.qor-launcher`; `src-tauri/src/lib.rs:867-878` resolves `app_data_dir()` once at
startup, creates it, and hands it to `AppState`. Everything the launcher persists is a file in that one directory:
`vault.qor` (`vault/mod.rs:42`, `:124`), `settings.json` (`config.rs:39`) and `gate-runs/` (`gates.rs:45`). There
is no per-account subdivision, because nothing per-account has needed storing.

Three facts force the question now.

1. **Products are arriving that own creator work:** a Projects surface, GNOSIS documents, QOR Engine projects,
   QFX scenes. The Projects surface landed the same day (`5a44207`) and opens whatever folder the person picks,
   for which decision 3's `work/` would be the default rather than a requirement; none of the rest exists —
   `docs/SYSTEMS.md:51` records Library as a placeholder, `:55` records Market as not present in the launcher at
   all — so the layout can be decided once, not six ways.
2. **The launcher holds no stable identifier for a person, and has not been promised one.** QOR ID's profile
   payload does carry an account UUID — the launcher's own test pins the real response, `"id": "f67c563e…"`
   (`identity/mod.rs:695`) — but `parse_session` (`identity/mod.rs:560-612`) never reads it, and the response
   shape documented above it (`identity/mod.rs:544-551`) does not list it. `Session` (`identity/mod.rs:77-86`)
   is the whole of what the launcher keeps: `qor_id` (`handle#0001`), `username`, `discriminator`, `role`, an
   optional `address`, an avatar URL — and every one of those can change.
3. **The chain cannot be on the critical path.** There is no fee mechanism (OPEN-4), DRC-369 does not exist and
   the wire format it needs is undecided (`docs/DIRECTION.md:174`, M4 and M2.3), and the Mesh is M8. A product that
   blocks on an RPC today blocks forever; one that blocks on an RPC in 2027 blocks on somebody's Wi-Fi.

Keys are already settled and must stay settled: the sealed vault is the only place a secret is at rest
(`vault/mod.rs:42`), tokens are in the OS keychain under the same identifier (`identity/mod.rs:37`, `:447-448`),
and `Vault::create` refuses to overwrite an existing vault rather than silently replace it (`vault/mod.rs:158-160`).

## Decision

1. **There is one data root per device, and it is the launcher's.** Durable launcher files stay where they are,
   in `app_data_dir()`. Creator work goes under `app_local_data_dir()`, which on Windows is
   `%LOCALAPPDATA%\cloud.demiurge.qor-launcher` and on macOS and Linux resolves to the same directory the
   launcher already uses (`~/Library/Application Support/cloud.demiurge.qor-launcher`,
   `${XDG_DATA_HOME:-~/.local/share}/cloud.demiurge.qor-launcher`).

2. **Every signed-in QOR ID gets a profile directory, named by a local profile id.** The id is 16 bytes from the
   OS random source, written as 32 lowercase hex characters, minted once on first sign-in on that device. The
   handle, the discriminator and the linked address are recorded *inside* `profile.json`, never in the path.
   Where the profile response carries an account id, `profile.json` records that too, so a later record can link
   one person's profiles without renaming a directory.

3. **The layout is fixed, and the launcher creates it — not the product.**

   ```
   <local data root>/profiles/index.json          profile id -> handle last seen, created, last opened
   <local data root>/profiles/<profile-id>/profile.json
   <local data root>/profiles/<profile-id>/outbox/          unsubmitted intents, one file each
   <local data root>/profiles/<profile-id>/<product-id>/product.json   id, schema version, last written by
   <local data root>/profiles/<profile-id>/<product-id>/state/         durable product data
   <local data root>/profiles/<profile-id>/<product-id>/work/          default home for documents it creates
   <local data root>/profiles/<profile-id>/<product-id>/cache/         deletable at any time, without loss
   <local data root>/profiles/<profile-id>/<product-id>/logs/
   ```

4. **`<product-id>` is one id used in three places**: this directory, the `product` field of the local SDK handshake,
   and the `demiurge:product:<product-id>:v1:` domain tag a product statement is signed under — the shape the vault
   already signs QOR ID challenges under (`identity/mod.rs:44`). It matches `[a-z][a-z0-9-]{0,31}`, is never a Windows
   reserved device name (`con`, `prn`, `aux`, `nul`, `com1`…, `lpt1`…), and two products never differ only by case.

5. **A product is handed its own directory at spawn and derives nothing above it.** The absolute path arrives in
   the handshake reply. A product that needs another product's data asks through the launcher; it never walks the
   tree.

6. **Offline-first is four obligations, each with a test.**
   - **Start and work with no network interface and no node.** Launch, open existing work, create new work, save,
     and quit, with the machine offline. No startup path makes an HTTP or WebSocket call.
   - **No UI blocks on the chain.** A chain-dependent surface renders from cache, marked with the block height it
     was read at, or says it does not know. It never spins.
   - **Identity works from the last unlock.** The vault unlock is local and always was; an offline launch enters
     the profile the device last signed into and marks it offline. **No server session is minted offline and no
     check is skipped** — what needs a live session stays unavailable and says why.
   - **Writes never wait for an acknowledgement.** Work saved offline is saved.

7. **Anything that would settle on chain is queued as a draft intent in `outbox/`, and is not pre-approved.**
   When a node is reachable the launcher offers each intent to the person, one intent, one approval, with the
   same host-drawn dialog that already guards signing. A queued intent is a note that something was *wanted*, not
   permission to spend CGT later.

8. **Export is one command, it runs offline, and it produces a plain directory.**
   `qor-launcher export --profile <id> [--product <id>] --out <dir>`, and one button that calls the same function.
   It writes to `<dir>.partial` and renames on success, so a failed export is never mistaken for a complete one.

9. **What an export contains.** Per product: the finished artifacts in the open formats
   [ADR-048](ADR-048-interchange-formats.md) names; the editable sources in an open format wherever ADR-048 names
   one, and the product's own files unchanged where it does not; `MANIFEST.json` listing every file with its size
   and SHA-256; and `NOT-PORTABLE.md` naming, file by file, what could not be written losslessly and what was
   lost. SHA-256 rather than the chain's Blake2, because the creator checks it with `sha256sum` or `Get-FileHash`.

10. **No secret is in an export or in a product directory, and this record moves no key.** An export carries no
    key material, recovery phrase, passphrase, access or refresh token, session id or spawn token — an export the
    creator cannot open with ordinary tools is not an exit. `vault.qor` stays in the launcher's own directory,
    tokens stay in the OS keychain, a connection's spawn token and session id stay in memory, and no product ever
    receives a path to the vault.

11. **Removing an account and removing data are three separate events.**
    - **Sign out** removes the session. Nothing on disk changes.
    - **Remove this profile from this device** is a typed confirmation that names the exact directory and its
      size on disk, offers an export first, deletes the directory and its index entry, and leaves the vault
      untouched.
    - **The QOR ID account is deleted at QOR ID.** The server deletes the server's records; it does not reach the
      disk. The launcher marks the profile detached, keeps the directory, and offers export and then deletion — on
      the person's instruction, never on a response from the network.

12. **Every write of durable state is write-temp-then-rename inside the same directory, flushed first.** One
    process holds a `<product-id>/state/.lock` naming its pid and process creation time; a lock whose process is
    gone is stale and is taken, and a lock whose process is alive is refused with the owner named.

13. **What is logged is the profile id and counts.** Not the handle, not an email address, not an address, not a
    token, not a path inside another profile. If any part of these flows reaches QOR ID, it joins
    `services/qor-auth/src/log_hygiene.rs` in the same change, under AGENTS.md §9.

## Consequences

**What it costs.** A per-profile path resolver, an index file, a lock file, an atomic-write helper, an exporter and a
deletion flow — a fortnight of launcher work, plus one acceptance test per obligation in decision 6, which must run
with the machine's network genuinely down and is the part that is easy to fake and worthless when faked. Every product
then carries the export half of its own formats: a per-product cost that never goes away, and what makes decision 8
true rather than aspirational.

**What it makes possible, and what it forecloses.** Two QOR IDs on one machine with no shared state. A product that
crashes without taking the vault. A creator who leaves with a directory rather than a support ticket. And a boundary
an OS sandbox can later be attached to, because the product writes in exactly one place. Foreclosed: a product keeping
a global store outside a profile, and a sync service bolted on without a decision.

**Substrate gaps this record names and does not fill.**
- **No subject id the launcher keeps or QOR ID guarantees.** The account UUID is on the wire and discarded
  (`identity/mod.rs:695`), nothing states that it is stable, and the documented response shape omits it. So the
  profile id is local: the same person on two devices has two profile ids and no link between them. The nearest
  adjacent work, [ADR-043](ADR-043-qor-id-as-an-identity-provider.md) §3, gives a *session* a client identity
  rather than a person a durable subject id, and it is Proposed. Closing this needs a record that makes the account
  id part of QOR ID's contract.
- **No portable provenance.** Royalty splits, licence terms and remix lineage have no format: DRC-369 is M4 and
  unstarted, and its wire format is undecided at M2.3. An export carries files and hashes, not a graph. A creator
  is told before publishing that what is written on a chain is not deleted by asking.
- **No rail to move an export anywhere** (Mesh is M8), so export writes to a local path and that is the whole of
  it; and **no entitlements** (M8.2), so a profile directory is not proof of anything and is not used as proof.

**What "full support" does not mean.** Not that a project opens elsewhere with nothing lost: where an open editable
format exists the export is faithful, and where none does it is the artifact plus the product's own files, with
`NOT-PORTABLE.md` saying so file by file. Not that the directory is a sandbox: a process running as the user can read
the whole disk, and this boundary is an organising rule, not a security control.

**Alternatives rejected.**
- **Name the profile directory by the QOR ID handle** (`architect#0001`). Rejected: handle and discriminator are
  display data that can change, `#` is awkward in shells, scripts and URLs, and it writes a person's identity into
  a path anyone who can list the disk can read. The handle goes in `profile.json` instead.
- **Name it by the QOR ID account UUID**, which the profile response already carries. Rejected on three counts:
  the directory must resolve before and without a network response, and an offline first launch has none; the
  field is in no documented response shape and is read by no code, so it is a value observed once, not a
  contract; and a server-supplied string that becomes a filesystem path on the vault's machine is untrusted
  input. It is recorded inside `profile.json` (decision 2) for the day a record promises it.
- **Name it by the SS58 address.** Rejected: ADR-017 allows an account with no key at all, an address may be linked
  long after the directory exists, and an address is a prefix and a derivation away from renaming (ADR-024, ADR-039).
- **One shared store with an account column.** Rejected: a single database with a `user_id` column is the layout in
  which one wrong `WHERE` clause shows one creator another's work — the silent, cross-account failure ADR-001 keeps
  out of this layer. A directory boundary is cheap and fails loudly.
- **Roaming `app_data_dir()` for bulk work.** Rejected: on Windows that root is roamed by profile synchronisation
  in managed environments, which turns a multi-gigabyte project into a logon-time copy. Small durable files stay
  there because they already are.
- **The platform cache directory for `cache/`.** Rejected for v1: the contract that matters is "deletable at any
  time without loss", which a test enforces and a path does not, and one layout is testable on three platforms.
  The cost is acknowledged — this departs from the macOS `~/Library/Caches` convention.
- **Each product choosing its own location.** Rejected: six conventions means no export, no deletion story and no
  way to answer "where is my work" in one sentence.
- **A single-vendor archive (`.qorpack`), or an encrypted export by default.** Rejected: a container only Demiurge
  reads is the lock-in this record exists to prevent, and an archive the creator cannot open is not an exit — the
  export holds no secrets by construction. A directory and a manifest open everywhere, including in ten years.
- **Deleting local data when QOR ID reports the account gone.** Rejected: the files are the creator's, the server
  has no authority over the disk, and a destructive act triggered by a network response is the silent,
  unrecoverable failure ADR-001 keeps out of this layer.
- **Cloud backup or sync now.** Rejected: there is no rail (Mesh is M8), and standing up storage locally would be
  a substrate gap filled in the wrong place.

## What this record does not decide

- **Where the vault lives.** `vault.qor` stays in `app_data_dir()`. Whether the sealed vault should move off a
  roaming root on Windows is a separate question and a migration.
- **Which formats an export is written in.** That is ADR-048, which is Proposed.
- **Any product's internal state format.** `state/` is the product's; this record fixes only where it is, that it
  is written atomically and that it can be exported.
- **Sync, backup, or the same profile on two devices**, until there is a rail; and **entitlements, ownership or
  gating** — a profile directory is not proof of a purchase and is never used as one.
- **What QOR ID deletes server-side**, its retention or any legal erasure obligation. This record governs the disk
  in front of the person.
- **The product id list.** Naming a product, like naming a pallet, is the owner's (AGENTS.md §8); decision 4
  fixes the shape of an id, not its contents.
- **Any economic value.** Nothing here names an issuance rate (OPEN-1), a genesis split (OPEN-2), or a fee or
  burn share (OPEN-4), and nothing here moves CGT.
