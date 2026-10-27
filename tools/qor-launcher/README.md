# QOR Launcher

The sovereign gateway to the Demiurge ecosystem. One desktop application through
which every chain and engine system is reached: QOR ID, the CGT Vault, the game
library, community, peer-to-peer distribution and the node itself.

**Status:** Version 0.1.17 (unsigned; 25 October 2026: ARQADE's app plays its six games rebuilt in QQ, Orbital, Void Runner, Synapse and Rift Survivor, and the two arenas, Flux Four and Rift Reversi, against another player (P7.20, P7.24); a `qor://arqade` link opens ARQADE, at a game if it names one (ADR-118); the themes a level unlocks, Dusk and Nocturne (P7.16); an asset's level and XP on its card (ADR-113); and QOR Media reads PDFs, rearranges their pages, marks them up (a drawn or typed signature among the marks, which certifies nothing) and fills in their forms (L9.5, ADR-114). 0.1.16, 23 October 2026: an agent asks through `qor://agent` and is allowed only by the profile's own key, after the host's dialog (ADR-112, L5.1); the first release offered through the update channel. 0.1.15, 22 October 2026: the first that updates itself (ADR-111, L6.2); Obsidian, Demiurge's one look, with QOR Media and ARQADE's app in it (ADR-108, ADR-109); the wallet in the core shared with Android (L13.2); and amounts read more strictly. 0.1.14, 18 October 2026: the ARQADE screen docks ARQADE's Qt
app, a 3D home with the games, the player's Sparks and the rankings, signed in through the launcher (ADR-102, P7.19);
and amounts are shown and typed in Sparks, 1,000 to a CGT (ADR-101). 0.1.13 the same day made each QOR ID on a PC a
profile of its own, with its own wallet and data, opened with its PIN and closed completely when it signs out
(ADR-099). 0.1.12 the same day labelled the accounts of one wallet (ADR-098, superseded). 0.1.11, 17 October 2026: a file's menu in
the Library that publishes it to the Mesh, and a premium look throughout: a bundled display
face, a grouped rail with a gliding marker, lit buttons and panels. 0.1.10 brought the Mesh's
paid work and QOR Media inside the Windows installer; 0.1.9 avatars, rings, Social and the
Library). Testers download it from the GitHub Release
[`launcher-v0.1.17`](https://github.com/QOR-MATRIX/demiurge-chain/releases/tag/launcher-v0.1.17). Gate, Nexus, Vault, Inventory, Projects, Chain, Gates
and Settings are built and running, and so is the first slice of Market (L7.2):
every listing the chain holds, read through the connected node, with no search
and no indexer behind it. So are payments asked by websites (`qor://pay`), the
level bubble and XP bar, and the avatar with its ring (below). By default it
connects to Demiurge Devnet at `wss://rpc.qorsync.dev` and QOR ID at
`https://id.qorsync.dev`. Social is real (below). The Library holds the person's own
files and opens them in QOR Media (below); its games shelf and Mesh are placeholder pages that
state their own blockers rather than showing mock content. Of the L1 roadmap
items, L1.1, L1.2, L1.3, L1.5, L1.6 and L1.7 are ticked; **L1.4 is implemented but
unticked**: the `qor://pay` approval dialog was used for real on 5 and 6 October 2026,
but a declined request, a transfer and an endpoint change are still untried. L1.6
and L1.7 were ticked on 6 October 2026 against CI's evidence: CI (GitHub Actions on the public repository,
ADR-063) runs and is green, including the launcher's host tests, its browser
checks and the coverage report. L3.1 and L3.2 are done.

---

## Why this exists, and why it is native

`tools/qor-launcher` was a README and nothing else. The ecosystem's client tier
had grown into a web hub, a browser extension, a standalone portal and a CLI,
each with its own session handling and its own idea of what a wallet is. That is
four attack surfaces and four sign-in flows for one person.

The launcher consolidates them, and it is a native process for reasons that are
not cosmetic:

- **Custody.** Keys in a webview share an address space with page script. Keys in
  a Rust host do not. This host will sign on request and will not export.
- **Distribution.** Seeding content peer-to-peer needs a long-lived process, disk
  and sockets. A browser tab cannot do it.
- **Supervision.** Installing, patching and launching games is process control.
- **Offline first run.** The app must open and let you unlock your vault with no
  network at all.

`.cursorrules` already designates the Qor Launcher as "The Hub". This makes that
real.

---

## Architecture

```
tools/qor-launcher
├── src/                     React 19 + Vite + Tailwind 4 (rendering only)
│   ├── components/
│   │   ├── chrome/          Frameless title bar, navigation rail, fault boundary
│   │   ├── gate/            Vault creation, unlock, sign-in, QOR ID claim
│   │   ├── nexus/           First-run orientation
│   │   ├── onboarding/      Choosing a QOR ID name
│   │   └── ui/              The panel surface
│   ├── lib/ipc.ts           The ONLY module that calls into the host
│   ├── qfx/                 The living backdrop (QFX layer one): one WebGL2 canvas behind
│   │                        the interface, under the chrome's scrim
│   ├── state/store.ts       One Zustand store
│   └── views/               Nexus, Vault, Chain, Inventory, Market, Release gates, Projects, Settings, Horizon
├── core/                    The core (ADR-104): everything that holds a key or talks to a service, shared
│   │                        with the Android shell and QOR OS (ADR-120); each module DIRECTION L13.2
│   │                        moved here is behind a feature, and the Windows launcher builds `desktop`
│   ├── src/cgt.rs           Denomination: parsing, formatting, precision
│   ├── src/vault/           BIP-39, Sr25519 (sp-core), Argon2id, XChaCha20-Poly1305
│   ├── src/profiles.rs      A profile for each QOR ID on this PC (ADR-099)
│   ├── src/identity/        QOR ID auth + OS keychain token storage
│   ├── src/social.rs        Social's socket, signed in through QOR ID (ADR-091)
│   ├── src/chain/           subxt client for the Substrate chain in chain/ (L3.1, ADR-040),
│   │                        including DRC-369 mint, make-permanent and enumeration (M4.1),
│   │                        selling and buying settled on chain (chain/sales.rs, L4.6),
│   │                        and the Market's read of every listing (chain/market.rs, L7.2)
│   ├── src/content/         The object model (ADR-047): manifest, BLAKE3 root, 41-byte
│   │                        reference, and the temporary content store
│   ├── src/pay.rs           qor://pay links, checked before any dialog (ADR-076, ADR-077);
│   │                        paid by src/chain/pay.rs
│   ├── src/agents.rs        qor://agent links, checked before any dialog, and the statement
│   │                        the vault signs to authorise an agent (ADR-112)
│   ├── src/library/         The Library's index (ADR-094): SQLite with FTS5, folders scanned and watched
│   ├── src/mesh/            The Mesh's node on iroh, and its paid work under its grant (ADR-095 to ADR-097)
│   ├── src/qontrol/         Qontrol's port: gitoxide for every read, the helper for staging,
│   │                        switch and discard, and the guard before staging
│   └── src/bridge.rs        The C bridge the Qt shell calls (include/qor_core.h, ADR-104)
├── src-tauri/               The Windows launcher's Rust host: the Tauri commands over the core, the window,
│   │                        and the products it starts
│   ├── src/lib.rs           The commands the interface calls (src/lib/ipc.ts is their list)
│   ├── src/gates.rs         Release-gate progress from docs/GATES.toml (L2.2)
│   └── src/host/            The product host (ADR-046, ADR-088)
├── shell/                   The Qt shell: the Android launcher (ADR-104)
└── qontrol-git/             The libgit2 helper: stage, write-tree, switch and discard, in its own process
```

Qontrol's staging runs in `qontrol-git`, a separate binary the host starts for
each call and speaks line-delimited JSON to, so libgit2 — which parses folders
people pick — never links into the process that holds the vault. The host
looks for it in three places, in order: `QONTROL_GIT_BIN`; beside the launcher's
own executable, which is where an installer puts it; and the development build
in `qontrol-git/target/`.

Since 2026-09-26 it is bundled (P1.2). `npm run app:build` first runs
`scripts/build-helper.mjs`, which builds the helper in release, copies it to
`src-tauri/binaries/qontrol-git-<target triple>` and copies libgit2's licence
from the exact vendored source to `src-tauri/licenses/` (both ignored by git),
then bundles with `src-tauri/tauri.bundle.conf.json`, which declares the
sidecar and the licence. The sidecar is declared there rather than in
`tauri.conf.json` because tauri-build refuses to compile when a declared
sidecar is missing, which would break `cargo test` on a clean checkout. For a
development build, `npm run helper` (or
`cargo build --manifest-path qontrol-git/Cargo.toml`) before committing from
the Projects surface.

The boundary is the point. `src/lib/ipc.ts` is the complete list of things the
interface is able to ask for, and it is short enough to read in one sitting.
There is no `getPrivateKey`, no `getSeed`, no `exportAccount`. The UI can ask the
host to **sign**; it cannot ask for the key.

---

## Security

### Key derivation

24-word BIP-39 phrases and **Sr25519 keys derived the way the ecosystem derives
them** (ADR-023, ADR-039), through `sp-core` itself at the version the chain is
pinned to. The first account is the phrase with no derivation path — the account
Polkadot.js, Talisman and Nova open when the phrase is imported — and further
accounts are the hard junctions `//0`, `//1`, …, which is Talisman's
enumeration. Every account is pinned in a test against
`Pair::from_string("<phrase>//n")`.

Addresses are SS58 at the chain's prefix wherever a person sees one; the raw
32-byte account ID appears only in the vault's Derivation panel (ADR-024).

Until 19 September 2026 this was SLIP-0010 Ed25519 at `m/44'/369'/account'/0'/index'`.
No other wallet opens those accounts from the phrase, which is why it changed
while nothing holds value.

This deliberately differs from `apps/wallet-extension`. That keyring documents
the path `m/44'/369'/0'/0/index` in a comment and then computes
`sha256(seed || "Demiurge:" + index)`. Keys created there are recoverable by that
one extension build and nothing else: not a hardware wallet, not a recovery tool,
not this launcher. **Anyone holding value in the extension needs a migration path
before that becomes a real loss.**

### At rest

| Layer | Choice | Why |
| --- | --- | --- |
| Key derivation | Argon2id, 64 MiB, 3 passes, 4 lanes | Memory-hardness is what actually costs a GPU attacker. PBKDF2 only costs cycles. |
| Encryption | XChaCha20-Poly1305 | A 192-bit nonce can be drawn at random with no collision concern, removing the nonce-reuse footgun AES-GCM carries. |
| Integrity | Header as AEAD associated data | The salt and Argon2 parameters are authenticated, so an attacker cannot downgrade the work factor to make cracking cheap. There is a test for exactly this. |
| In memory | `Zeroize` throughout | The seed is wiped when the vault's session is dropped: on lock and on exit. There is no idle timer since ADR-056. |
| In logs | Redacting `Debug` on the decrypted phrase | A derived `Debug` would print the recovery phrase into any panic message or assertion failure that formatted a `Result`. |
| On disk | Write to a temp file, fsync, rename | A crash cannot leave a truncated file where a recovery phrase used to be. |

### No lock screen: the key is in the keychain

Since 2026-09-28 (ADR-056, superseding ADR-055's Windows Hello) nothing is
typed or asked to open the vault. `vault.qor` holds the recovery phrase sealed
under a random 32-byte key, and names the keychain entry that holds the key:
`vault-key:<id>` under the service `cloud.demiurge.qor-launcher`, beside QOR
ID's tokens, in Windows Credential Manager, the macOS Keychain or the Secret
Service. The launcher reads it before its first frame and shows the shell. There
is no idle lock and no Lock button. A copied `vault.qor` opens nothing without
the account's keychain; anyone using the signed-in computer account can open it,
which is the trade the owner chose.

**QOR ID never blocks the launcher.** Opening the vault tries to sign in with its
key (ADR-016's arrival grant); if the service is down the shell shows anyway, and
Settings offers *Sign in to QOR ID* (`qor_sign_in`), and a name to claim when the
key has none. There is no password sign-in. Every transfer, trade, mint and
endpoint change still asks in a host dialog, and so does *Show recovery phrase*.

A first run is one button: *Begin* makes the phrase and the vault. The phrase is
shown only in Settings, on request.

**Older vaults move once.** The locked status says what seals a vault
(`sealed_with`): a Windows Hello vault from ADR-055 asks Hello one last time; a
passphrase vault asks for its passphrase one last time, or moves with Hello alone
if its ADR-054 `vault.hello` copy belongs to it. The old file is set aside.

Tests use an in-memory keychain; `cargo test --lib the_os_keychain -- --ignored`
writes a throwaway entry to the real keychain, reads it back and deletes it
(passed on this Windows PC, 2026-09-28).

### Restoring from the recovery phrase

The recovery phrase is the way back when the keychain loses the key (a new
computer, a reset profile). The Gate offers **Restore from your recovery phrase**
whenever it shows (`Vault::restore`). The host first says which account the
typed phrase opens (`vault_preview_phrase`), so the person can recognise it
before anything changes. Replacing a vault already on the device is approved in
a host dialog, and the old file is **set aside, never deleted**, as
`vault.qor.set-aside-<date>` beside it.

`node scripts/check-vault-gate.mjs` drives the Gate and Settings against a
stand-in host that records every command, and passes **65 checks
(2026-10-18)**: a keychain vault reaches the shell with no lock screen and
nothing typed; QOR ID down stops nothing and Settings signs in with the vault,
never a password; a first run is one button sending the host the phrase alone; a
keychain vault that will not open says to restore from the phrase; a Hello vault
asks once and not again after a cancel; a passphrase vault asks its passphrase
without Hello being asked; a typed phrase goes to the host normalised and its
account is shown before anything is restored; and Settings shows the phrase
through the host.

### Each QOR ID is a profile (ADR-099)

Since 0.1.13 each QOR ID on a PC is a profile with its own folder,
`profiles/<id>/` in the app data directory (and under the local data directory
for its Library index): its wallet (`vault.qor`, one account), avatar, trade
partners, listing drafts, paid requests, published work's bytes, Mesh and
Library. QOR ID's tokens are kept in the keychain under the profile's id.
`profiles.json` lists the profiles by id, name and address and names the one
signed in (`src-tauri/src/profiles.rs`). Device settings stay shared.

The profile signed in last opens at launch with nothing asked, as ADR-056 decided.
**Signing out** (Settings, *This QOR ID on this PC*) closes it completely: Social
and QOR ID sign out, the Mesh stops, the Library closes, the wallet locks and its
key is taken out of the keychain. The launcher then shows **Who's playing?**: the
profiles by name, no balance, *New QOR ID* and *Restore from a recovery phrase*.
A profile opens there with its **PIN**, 6 to 12 digits, which seals its wallet
key (the vault's own Argon2id and XChaCha20-Poly1305 seal) in `profiles.json`.
The PIN is chosen the first time a profile signs out. After five wrong PINs each
try waits, doubling from 30 seconds. A forgotten PIN is replaced by restoring the
profile from its recovery phrase.

A new QOR ID has a new phrase and an empty wallet. The wallet from before 0.1.13
became the first profile at its first launch; Settings' *Other accounts from this
wallet's recovery phrase* looks for the accounts that phrase used on the chain
and makes each a profile of its own (`profile_split`).

`node scripts/check-profiles-view.mjs` holds the screens to this with a stand-in
host and three profiles, and passes **25 checks (2026-10-18)**. Two planted
faults, a PIN shorter than six digits accepted and a split sending no PIN, were
each caught.

### Arriving: the name, the tutorial and the intro

Since 2026-09-28, a person without a QOR ID sees a bubble, *Choose your QOR ID*
(`src/components/onboarding/Onboarding.tsx`). It opens a card that checks the
name's shape at once and asks QOR ID whether it is free 350 ms after typing
stops, keeping only the latest answer. Claiming first opens a sign-in with the
vault (`qor_sign_in`), which reopens ADR-016's arrival grant, so the claim
(`qor_register_with_key`) needs no dialog. Once named, a notification with a
neochrome halo invites them in; clicking it opens a six-chapter tutorial
(`src/qfx/ceremony/Tutorial.tsx`) that begins "Hello <name>!". Its claims about
money stay inside `docs/economics/CGT.md`. Finishing it is remembered per QOR ID
on this computer.

Every open plays a two-second splash (`src/qfx/ceremony/Intro.tsx`); any click
or key skips it. After an install or update, a first-run animation plays once
per version instead: the owner's own file from `src/assets/first-run/` (its
README gives the formats), or a longer splash while there is none. Reduced
motion, in the launcher or the operating system, skips both.

Since ADR-080 (6 October 2026) glows, gradients, canvas and looping animation
are allowed on every surface, not only in `src/qfx/ceremony/`; reduce motion
still stills or skips them, and text must stay readable as painted. Explanatory
text elsewhere sits behind an information icon (`src/components/ui/InfoTip.tsx`)
that opens on hover or keyboard focus.

### Running QOR ID and a chain on this computer

`powershell -File scripts/start-local.ps1` starts a local QOR ID and chain for
the launcher's settings to point at (by default they point at the live ones,
above): QOR ID on `127.0.0.1:8080` with Postgres (`qor-local-pg`, port
54329, volume `qor-local-pgdata`) and Redis (`qor-local-redis`, port 56389) in
Docker, and `demiurge-node --dev --tmp` on `127.0.0.1:9944`, a fresh chain each
start. JWT secrets are made once into `%LOCALAPPDATA%\qor-local\secrets.env`,
outside the repository; logs go beside them. `-NoChain` skips the node, `-Stop`
stops everything. Run it in a console window: if its output is captured, the
servers it starts hold the capturing pipe open and the caller waits.

### Tokens

Access and refresh tokens live in the **OS keychain**, never on disk and never in
the webview. `localStorage` in a webview is readable by anything that achieves
script execution there.

### Signing

The launcher never encodes a transaction itself. `subxt` builds the signer
payload from the metadata the connected node serves (ADR-040), and the vault
signs those bytes verbatim, behind a host dialog. There is no second encoder to
drift from the chain's, which is what `src-tauri/src/chain/payload.rs` used to
be for: it ported the custom devnet's byte layout by hand, and went with it.

### Products the launcher starts (ADR-046, ADR-088)

QQ Studio is the first **product**: a program of its own the launcher starts, which asks the launcher for what needs a
key and never holds one. `src-tauri/src/host/` is the product host:

- **The channel** (`channel.rs`): a named pipe whose DACL grants only this user, created as the first instance of an
  unguessable name with remote clients refused (a `0700` folder and `0600` socket elsewhere). It accepts **one**
  connection, and only from the process the launcher spawned: the operating system's client process id, that process's
  start time (so a reused id cannot pass) and its user, all checked.
- **The Motion setting** (`mod.rs`): every product is started with `QOR_REDUCED_MOTION`, `1` or `0`, the person's
  own setting as the interface last told the host (`launcher_motion`; ADR-108 decision 5).
- **The token** (`mod.rs`, `session.rs`): 32 random bytes, written as hex on the product's **stdin**, which is then
  closed (never an argument or an environment variable); good once, for 30 seconds, compared in constant time.
- **The protocol** (`frame.rs`, `protocol.rs`; schema and golden frames in `tools/qor-launcher/host-protocol/`): a
  4-byte big-endian length and a JSON object of at most 1 MiB; `hello` / `welcome` with a version and capabilities;
  then **typed intents**, never bytes to sign. QQ Studio is offered `qontrol.commit` and `cartridge.publish`
  (`studio.rs`).
- **Who signs a publish** (ADR-090): the vault account holding the project's Cartridge revises it, whichever account
  the rail shows; a first publish signs as the account the rail shows, which the interface tells the host
  (`vault_set_active`, `Vault::active_address`). The dialog names it (`Owner:`).
- **The dialog** (`ProductDialog`): every prompt a product's request raises starts with the product's name and the
  program the launcher started, from the launcher's own record.
- **Lifetime**: each product is in a Windows job that ends it when the launcher ends; its exit code and the end of its
  error output reach the QQ screen if it ends badly.
- **Nothing secret in the log**: a test runs good and refused sessions with every log level on and looks for the token
  and the session id.

The operating-system calls are FFI, and this crate forbids unsafe code, so they are their own crate,
`tools/qor-launcher/host-proof/` (`qor-host-proof`), with a test for each. The tests start a stand-in product
(`src-tauri/examples/host_fake_product.rs`, built by the tests themselves) through the real channel, including a
connection from a process the launcher did not start, which is never answered.

---

### Payments asked by websites: `qor://pay` (ADR-076, ADR-077)

A website can ask the launcher to pay, never make it pay. The `qor` scheme is registered by `tauri-plugin-deep-link`;
a link clicked while the launcher runs reaches it through `tauri-plugin-single-instance`. Added in 0.1.7. `core/src/pay.rs` checks a link
before anything is shown: signed with the Ed25519 key of an app in `KNOWN_APPS` (ARQADE's only, today), Demiurge Devnet
by genesis, a whole number of embers up to 100,000 CGT, a valid account, a short label, unexpired and valid for at most
fifteen minutes, and not already paid (`paid-requests.json` in the app data directory). Only then does the host dialog
show who asks, the amount, the recipient, that there is no fee, and the balance after; approving is that dialog, not
Windows Hello (ADR-077). `core/src/chain/pay.rs` checks the
connected chain's genesis again and pays the transfer and a `remark_with_event` naming the request in one `batch_all`,
from the signed-in profile's account, the only one open (ADR-099). Any other link, or an unknown app, is refused with a message, and nothing is signed.

### Agents asking to act for a person: `qor://agent` (ADR-112, L5.1)

An agent is authorised by the person's own key, never by their sign-in alone. The agent makes its own Sr25519 key, asks
QOR ID for a challenge for its account (`GET /api/v1/auth/challenge?account_id=0x…`), signs
`demiurge:qor-id:challenge:v1:<challenge>` with that key, and opens

```
qor://agent?r=<hex of {"account_id", "challenge", "signature", "name", "capabilities", "autonomy",
                      "spending_limit" (optional, whole CGT), "model" (optional)}>
```

`core/src/agents.rs` checks it before anything is shown: the agent's signature verifies; the challenge has QOR ID's
shape and is at most ten minutes old; every capability is one QOR ID knows, named once; the autonomy is one of four; a
bounded agent names a limit and none is negative; the name makes a QOR ID (`agent_` and 3 to 14 letters, digits, spaces
or underscores); the model is one short line. The host then requires a signed-in profile whose QOR ID has proved a key,
that key being the open wallet's account, and asks in its own dialog: the QOR ID the agent will act for, the agent's
QOR ID and account, what it may do, its autonomy and limit, and that nothing enforces them yet. On approval the vault
signs the authorisation statement QOR ID verifies (ADR-112 decision 2) and the host registers the agent with the
profile's session. Any refusal comes before the dialog, and nothing is signed. Listing, limiting and revoking agents
from the launcher are L5.2.

To see it, with a launcher of 0.1.16 or later installed and signed in: `cargo run --example agent_link` in
`src-tauri/` plays a test agent against the live QOR ID (`--qor-id` for another, `--print` to print the link instead
of opening it). Its key lives only in that process.

### Opening ARQADE: `qor://arqade` (ADR-118)

```
qor://arqade                 the ARQADE screen
qor://arqade?play=<game>     and one of ARQADE's games: orbital, void, synapse, rift, flux or reversi
```

The launcher's window comes forward at the ARQADE screen (before the vault is open, it is the screen shown after), and
ARQADE's app opens the game the link names: at once if it runs, or as it starts. `src-tauri/src/arqade.rs` (`open_link`)
reads the link: only one of the six names goes further, the first `play` decides, and anything else, or anything after
`arqade`, opens only the screen. The link pays nothing, signs nothing and signs nobody in. The interface asks the host
whether a link came (`arqade_app_asked`) as it starts and on each `arqade://asked`, so a link that started the launcher is
not lost. ARQADE's own website window routes the link here too, where any other `qor://` link still goes to the pay path.
A launcher older than this refuses the link as a payment request, so the website offers it only from the release that
answers it.

### Avatars (ADR-079)

The account's avatar, with its level on a ring that glows from level 1 (ADR-080), sits in the rail and, at 64 px, on the
Nexus header, where clicking it opens the host's own file picker (`qor_choose_avatar`; the webview never reads files).
QOR ID cleans and keeps the image; the launcher keeps a copy in its data directory (`avatar.bin`, `qor_avatar`) so it
shows offline. With no picture, or none yet, an avatar is the name's first letter on a colour the name always gets.
A GIF holds its first frame under reduce motion. Avatars and rings reach people from 0.1.9.

### Level and XP (ADR-078)

The rail's avatar carries the account's level in a small bubble, and an XP bar under the name with the next unlock.
Both are read from QOR ID (`GET /api/v1/profile/progress`, `qor_progress`), which is the only place XP is granted;
finishing the tutorial reports it once (`qor_tutorial_done`). Signed out, nothing is shown.

**Themes a level unlocks.** Settings offers every theme, two of them unlocked by level as QOR ID's list has it
(`services/qor-auth/src/handlers/progress.rs`, UNLOCKS): **Dusk** at level 2 and **Nocturne** at level 5
(`src/styles/themes.ts`, each with its `unlock`). Until the profile's progress lists the unlock, the theme shows its
level and cannot be chosen. `scripts/check-themes-view.mjs` holds it: locked at level 1, chosen and worn once QOR ID
lists it, the other still locked.

### Social (ADR-091, roadmap L8.5)

The Social surface is the first client of Social (`services/social/`). **The host holds the one socket and the token**
(`core/src/social.rs`): the QOR ID access token goes from the keychain into the socket's first frame and nowhere
else; the webview asks through `social_request` and hears `social://status`, `social://welcome` and `social://event`.
The socket is jsonrpsee's WebSocket transport, which subxt already brings for the chain, so Social adds no library to the
process that holds the vault. It reconnects by itself (1, 2, 4 … 30 s), swaps an expired token without a pause, refreshes
once when Social says a session is not live and then reports `signed_out`, and never leaves a request hanging. Social's
address is `social_url` in the settings file (`QOR_SOCIAL_URL` overrides it): `wss://`, or `ws://` on this machine only;
the default is the deployment on Railway, `wss://social-production-e05c.up.railway.app/v1/socket` (L8.6, since
16 October 2026).

The interface (`src/social/`) stays connected while the shell is open, so the rail counts unread messages from any
surface. Entering asks for nothing already known: with no QOR ID the name is chosen first; on a name QOR ID made
(`key_…`) a name is chosen once (`qor_claim_username`, ADR-091 decision 5) and Social is told again who this is.
Rooms, direct conversations, replies, reactions, edits, deletes, @mentions with completion, typing, presence, profiles
with a banner and an accent (refused when it would not read on the theme), and a showcase chosen from the account's own assets. Evidence: the host's socket tests and its log check (`social::tests`), and `scripts/check-social-view.mjs`.

**World, friends and translation (ADR-092).** World is pinned first and opens with Social; Chats and Friends are tabs,
with requests, adding by QOR ID, friends online first, and circles that open a group chat. Each author shows their level
and username. World and public rooms are shown in the reader's language, **translated on this PC**:
`src-tauri/src/translate.rs` reads Mozilla's Remote Settings (`translations-models-v2`, `translations-wasm-v2`), plans the
files a pair needs (two through English), downloads each once with `translate://progress`, checks its SHA-256 packed and
unpacked (unpacking capped at the published size), keeps it by hash under the local data folder's `translations/`, and
serves only files it checked this run (`translate_file`, raw bytes). The engine is pinned (`ENGINE_SHA256`) to the build of
Firefox's loader kept in `public/translate/bergamot-translator.js` (v0.6.0, MPL-2.0, its licence beside it); the worker
there (`public/translate/worker.js`, MPL-2.0, after Firefox's) runs it, which is why the CSP allows `'wasm-unsafe-eval'`.
`translate_detect` tells a message's language on the sender's PC with `whatlang`. **Levels (ADR-093)** live in
`src/social/Levels.tsx`: the badges (godmode, Elite, Owner, Admin, Mod, VIP) beside names, and what a reader's standing
lets them give or do, which is all the screen offers; Social decides each one. Evidence: `translate::tests` (a stand-in
for Mozilla), `scripts/check-social-view.mjs` (the worker in its stand-in mode), and by hand, the real engine with
Mozilla's real files:

```bash
QOR_TRANSLATE_DIR=<folder> cargo test --lib real_mozilla -- --ignored   # in src-tauri: downloads about 30 MB
node scripts/check-translate-engine.mjs <folder>                        # after npm run build
```

### The Library and QOR Media (ADR-094, roadmap L9)

Everything a person has in one SteamOS-like view. The core's `core/src/library/` keeps an index of the folders the
person chose (Videos, Music, Pictures and Documents are offered), in SQLite with full-text search in the launcher's data
folder: each file's kind, size and dates, a picture's size and camera, music tags, a text document's first words, and a
thumbnail for pictures. Folders are scanned at start and watched while the launcher runs. Nothing in it leaves the PC.
`src/library/LibraryView.tsx` draws a hero with Resume, shelves by kind, tabs, Folders and search (`/`); the arrow keys
move a focus ring, and a game controller does the same through the Gamepad API (D-pad or left stick, A opens, B goes
back), since WebView2 turns controller input into no keys.

A file opens in **QOR Media** (`tools/qor-media/`, a Qt 6 program with [its own README](../qor-media/README.md)) where
`QOR_MEDIA` names it, or in the launcher's install folder, where the Windows installer puts it since 0.1.10
(`npm run app:build` stages it with `scripts/stage-qor-media.mjs`, which needs the owner's Qt), or where it is deployed at
`%LOCALAPPDATA%\qor-media\qor-media.exe`; otherwise in the program Windows chooses. `src-tauri/src/media.rs` starts it under the product host (above) and answers its one method, `media.next`,
with the next command: open a file, sit at the Library's rectangle in physical pixels (`media_place`), hide when the
person leaves the Library (`media_hide`), or wear the launcher's colours (`media_theme`, `#rrggbb` tokens only). QOR
Media's window becomes a child of the launcher's. Its reports (where playback stopped, a file's length and size) ride
on its next `media.next` and update the Continue shelf.

```bash
node scripts/check-library-view.mjs                                     # after npm run build: 27 checks
cargo test --lib media::tests::qor_media -- --ignored                   # in src-tauri, with QOR Media deployed
```

### The Mesh (ADR-095, roadmap L10)

A peer-to-peer node in the core (`core/src/mesh/`), on iroh, holding files by their BLAKE3 root (ADR-047's
fingerprint) and streaming them to other launchers verified against it in 16 KiB groups. **Seeding is off until the
person turns it on**, with the two limits they give on the Mesh page: disk for the Mesh's store
(`<data>/mesh/store`) and upload in bytes a second, kept by a token bucket on every 16 KiB served. On, it takes in the
temporary content store's blobs (one's own published work), each checked against the root it is filed under and
referenced where it lies, at start and once a minute. A file is fetched by a **ticket** (the file's root and the PC that
has it); its size is proven before any of it is taken, so nothing passes the disk limit, and a peer whose bytes do not
match the root is refused. The node's key is made on this PC (`<data>/mesh/node.key`), is never the vault's, and can move
no CGT. Peers are reached through n0's public relays until Demiurge runs its own (L10.6). No CGT moves yet: the proofs
(L10.3) and the hosting escrow (L10.4) come first.

```bash
cargo test --lib mesh::                 # in src-tauri: two launchers on this PC, over the loopback address
node scripts/check-mesh-view.mjs        # after npm run build: 23 checks
```

## Release gates (development dashboard)

The Gates surface shows Alpha, Beta and Public Release progress, computed by
`src-tauri/src/gates.rs` from [`docs/GATES.toml`](../../docs/GATES.toml) and nothing
else. The counting rules are the file's own. Every unit is met, not met or
unmeasurable. A gate shows raw counts, and a bar only when none of its units is
unmeasurable. A signal the file does not call unmeasurable, but that cannot be
read, counts as not met and says why.

- **Where it looks.** `DEMIURGE_REPO`, or the nearest ancestor of the working
  directory or the executable that contains `docs/GATES.toml`.
- **CI and coverage** are read with `gh`, which must be installed and signed in.
  Without it, those units are not met, with the reason shown.
- **Suites** run from the view, but only after the host's own dialog names the
  directory and command and is approved, because a suite builds and runs
  repository code. Runs are recorded in the launcher data directory under
  `gate-runs/`, with a separate cargo target directory per suite.

---

## Money handling

All amounts are integer embers in `u128`, never floating point, and the `u128`
crosses the IPC boundary as a decimal string because JavaScript's `number` cannot
hold it.

**Typing an amount round-trips to the host.** The Rust side parses it and returns
the exact ember value it would transfer, so the number shown on the confirmation
screen is the number that gets signed rather than a JavaScript approximation of
it. Views show amounts only as the host formats them, in CGT; no ember count and no
JavaScript `number` stands in for an amount anywhere in the interface. Excess precision is **rejected, not truncated** — silently dropping value
from a payment is the worst failure mode a wallet has.

Sending is two-stage: compose, then confirm against a summary with the full
recipient address. A transfer cannot be undone, so one click should not be able
to cause one.

### On CGT precision

CGT has 18 decimal places, defined once for the chain in
`chain/runtime/src/denomination.rs` (ADR-035). The launcher mirrors it in
`src-tauri/src/cgt.rs`, and a test pins the two together so they cannot drift.
It pins only what the chain declares — the precision, the ticker and the
existential deposit — because the chain declares no total supply at all while
issuance is OPEN-1 and the genesis split is OPEN-2. It was 2 decimals until
September 2026; why it changed is recorded in `docs/DECISIONS.md` (D-000).

Every conversion goes through `src-tauri/src/cgt.rs`. Amounts are integer embers
in `u128`, and excess precision is rejected rather than truncated.

**Sparks (ADR-101).** A Spark is a thousandth of a CGT, the unit people see and
type; the ember is the atomic unit underneath (1 CGT = 1,000 Sparks = 10^18
embers). The Vault shows a balance in CGT and in Sparks (`format_sparks`: whole
Sparks as Sparks, anything finer in full as CGT, never rounded). The send box and a
sale's price take CGT or Sparks (`parse_amount`: "12.5", "12.5 CGT", "250 Sparks");
the Mesh's escrow prices are still typed in CGT.

---

## Running it

```bash
cd tools/qor-launcher
npm install
npm run app:dev        # dev, with hot reload
npm run app:build      # release bundle (msi, nsis, deb, appimage, dmg)
npm run icons          # regenerate the icon set from scripts/make_icons.py
npm run check          # design, accessibility, gates view, Projects view, Inventory view, Market view, QQ view, Social view, Library view, Mesh view, updates view, the release script's signature check, the Gate's restore and Hello, QFX contrast, readability (needs npm run build first)
```

### Updates and releases (ADR-111)

The launcher checks one address for a signed manifest,
`https://github.com/QOR-MATRIX/demiurge-chain/releases/download/launcher-channel/latest.json`, fifteen seconds after
start and every six hours (Settings, *Check for updates by itself*), and installs only when the person presses *Update
and restart*. Tauri's updater verifies the installer against the key in `tauri.conf.json`; the host verifies it again
and requires it to be the announced version's installer (`src-tauri/src/update.rs`).

```powershell
pwsh scripts/updater-key.ps1 -Create              # once, on the PC that builds releases; -Backup says what to keep offline
node scripts/release.mjs --notes notes.md         # build, sign, check the signature and the file's name, write latest.json
node scripts/release.mjs --notes notes.md --publish   # and publish launcher-v<version>, then replace the channel's manifest
node scripts/update-e2e.mjs                       # the whole path on this PC, with a test product of its own (minutes)
```

The key is in `%APPDATA%\Demiurge\updater\`, never in the repository; its password is sealed by Windows beside it. A
lost key means testers reinstall by hand once. `QOR_UPDATE_ENDPOINT` names another channel for testing; whatever it
serves must still verify against the launcher's key, and a release build accepts only HTTPS.

Rust tests. Build Qontrol's helper first, or its tests that commit skip:

```bash
cargo build --manifest-path qontrol-git/Cargo.toml
cd src-tauri && cargo test                              # skips say so
cd src-tauri && cargo test --features qontrol-no-skips  # as CI runs it: a skip is a failure
```

Against a local chain, start a node first and point the launcher at it from the
Chain surface. **This is the Substrate chain in `chain/`** (ADR-040), which since
M3.5 is the only chain in the repository.

```bash
chain/target/release/demiurge-node --dev --tmp
```

The Chain surface's **Local node** preset is `ws://127.0.0.1:9944`; its
**Devnet** preset, and the launcher's default (`DEFAULT_RPC` in
`core/src/chain/mod.rs`), is Demiurge Devnet at `wss://rpc.qorsync.dev`.
The chain surface shows the name the node reports, so you can see which chain
answered.

The live test below exercises the whole path against that node, and is the only
part of the client a test cannot cover without one:

```bash
cd src-tauri && cargo test --lib chain::live -- --ignored --nocapture
```

### Verified

- 204 Rust host tests passing with no node running, and 8 ignored that need a node or the OS keychain
  (2026-10-06; 195 on 2026-10-02), 39 of them Qontrol's and 7 the object
  model's (ADR-047: the same files in any order make the same reference, one changed byte changes it, the
  reference is the 41 bytes the chain stores, and the manifest's encoding is pinned). Build the helper
  first: without it the Qontrol tests that commit skip and say so, and under
  `cargo test --features qontrol-no-skips`, which CI uses, they fail instead. The helper's own 16 tests
  pass in `qontrol-git/`.
- **240 Rust host tests passing, 11 ignored** (15 October 2026, under `qontrol-no-skips`; 10 need a node and 1 opens
  QQ Studio). Since 6 October they include **the product host** (P3.4, ADR-046, ADR-088), `src/host/tests.rs`: a
  frame goes and comes back whole, and one over 1 MiB is refused both ways; the product the launcher started is
  welcomed and answered; a wrong token, a token past its 30 seconds, another product's hello with the right token, a
  request before hello and a second hello are each refused; a product needing a later protocol is told which this
  speaks; a product's dialog names the product from the launcher's record; nothing secret reaches the log at any level
  (one process-wide logger, since `tracing` caches interest across threads); the golden frames in
  `tools/qor-launcher/host-protocol/` are what each side reads and writes; and, with the stand-in product
  `examples/host_fake_product.rs` built by the tests themselves, a spawned product proves itself and is answered, a
  spawned product with the wrong token or name is refused, and a process the launcher did not start is never welcomed.
  Also new: the revision dialog names both commits and keeps the asset (`chain/assets.rs`); the published commit
  nearest HEAD is found in a project's history (`qontrol/tests.rs`); QQ Studio is not opened on a folder that is not
  a project (`qq.rs`). **The fifteenth host check is live**, ignored by default:
  `a_project_is_minted_then_revised_then_left_alone_through_the_channel` commits, mints behind a dialog naming the
  product, revises after a new commit, answers `unchanged` with nothing asked, and leaves the chain alone on a decline,
  against `demiurge-node --dev` (passed 10 and 15 October 2026). `qor-host-proof`'s own checks run in its crate
  (`cargo test` in `host-proof/`): a process's start time is stable and is this user's, an ended or absent process has
  none, a pipe client is named by its process id, a second pipe of the same name is refused, and closing the job ends
  the process in it.
- **An account that cannot pay is told so before anyone is asked** (15 October 2026), live and ignored by default:
  `an_account_that_cannot_pay_is_told_so_before_anyone_is_asked`. A mint from an account the chain holds nothing for,
  then from one holding 200 CGT against about 1,100 CGT of deposits, is refused as `unfunded` with the account and the
  chain named; nobody is asked, nothing reaches the temporary store and the nonce stays 0. Without this, the node
  refuses the first as "Inability to pay some fees", on a chain with no fees, and the second fails in its block with
  `Balances::InsufficientBalance`, which is what the test shows when the deposit check is disabled. Every transaction
  through `sign_and_finalise` makes the first check; a mint makes the second.
- One more host test — the live chain test, ignored by
  default — passing against a development node (2026-09-20): a transfer built from the node's own
  runtime metadata, approved in the host dialog, signed
  inside the vault, submitted, and finalised by GRANDPA; a declined transfer that moves nothing and
  leaves the nonce where it was; exact balance arithmetic on both sides; and two transfers back to
  back, which is what the nonce rule in ADR-040 is for.
- A second live test (2026-09-22, M4.1): a mint built from the node's metadata that, declined, moves
  nothing, sends nothing and writes nothing to the temporary content store; approved, finalises, puts its
  bytes in the store, and appears in the owner's enumeration read from chain storage with its reference,
  pinned commit and name; then made permanent through the same dialog and vault, after which a second
  attempt is refused before anyone is asked. Both live tests pass together; Alice's funding is serialised
  because they run in parallel and would otherwise race her nonce.
- A fourth live test (2026-10-01, L4.6), `a_sale_pays_every_part_and_hands_the_asset_over`: a remix with
  royalty terms of its own, derived from an original with terms, is listed, bought and paid out against a
  development node at `spec_version` 4. A declined listing, purchase and withdrawal each move nothing and
  leave the nonce where it was; what the launcher says the sale will pay is what the chain's `Sold` event then
  reports, part for part; and the four balances are asserted to the ember, with amounts written out in the
  test rather than computed (2,000 CGT and seven embers: 100 and 300 CGT to the source's two recipients, 40
  CGT to the remix's own, 1,560 CGT and the seven embers to the seller). It also proves a listing below what
  a recipient could receive is refused before anyone is asked, a price or fingerprint that changed since the
  buyer looked is refused before anyone is asked, a void listing can be cleared by anyone, and the chain's
  own `PriceAboveLimit` is put in words. The test sends `set_terms` and a remix mint itself, because the
  launcher has no surface for either. All four live tests passed together on 2026-10-01 (377 s).
- **A purchase is held to the work the buyer saw, on chain** (2026-10-02). Buy sends
  `Drc369Royalties::buy_exact` with the price and the content reference that were on screen, so a revision
  landing between the dialog and the block is refused by the chain (`ContentChanged`, put in words) and not only
  by the host's check before signing. It needs a node at `spec_version` 6 or later. The five live tests passed
  against one on 2026-10-02 (377 s), the sale among them.
- **What a sale pays is the chain's arithmetic, not the launcher's** (2026-10-02). The launcher carries no
  copy of the pallet's `split` and no `sp-arithmetic`. `core/src/chain/sales.rs` asks the chain through
  the runtime API `Drc369RoyaltiesApi::sale_preview`, called dynamically from the node's metadata at the same
  finalised block as its other reads, and the Sell form, the Buy dialog and both approval dialogs show that
  answer: every part, who receives it, and whether the sale could settle. For a buyer the chain runs the
  purchase and undoes it, so its refusal (a balance that cannot cover the price, a part its recipient cannot
  receive, an asset held in place by nesting) is the one the transaction would get, said in the same words
  `in_words` gives a refused transaction, with the account and the amounts named where the launcher's reads
  at that block show them. The launcher adds only what the chain is not asked: a void listing, and a price
  or fingerprint that changed since the buyer looked. An answer whose parts do not sum to the price, checked
  without overflow, is refused rather than shown. **A node whose runtime predates `spec_version` 6 has no
  such API, and selling and buying are refused in words on it**; nothing is worked out instead. After a
  sale, what each account received is read from the `Sold` event and nothing else. The module's tests
  decode the chain's answer and name its refusals; eight faults planted in them on 2026-10-02 (a share put
  beside the wrong account, the seller taken from storage rather than the answer, the parts' sum allowed to
  wrap, every refusal treated as blocking the sale, a pallet error misread, another buyer's answer used, the
  nesting sentence lost, a source named without terms) each failed at least one. The live sale test asserts
  the chain's preview part for part against the `Sold` event, and that a buyer holding nothing is refused
  by the chain's own run of their purchase; the five live tests passed together against a `spec_version` 6
  node on 2026-10-02 (380 s).
- The accessibility settings take effect in a real rendering engine: after `npm run build`,
  `node scripts/check-accessibility.mjs` runs 41 checks against headless Edge or Chrome (2026-09-22), seven
  of them a precondition that the page is visible. One case, "stored off, then live: the backdrop starts",
  failed on about half the runs until the interface was lifted above the backdrop and its scrim (below);
  since then it has passed 7 runs of 7. Its threshold was never moved.
  21 of them drive the QFX backdrop through the app itself — settings stored where the app keeps them,
  the page reloaded — and count frame callbacks: live draws, still and reduced motion schedule nothing,
  Off hides the canvas and the scrim together, and Off can be turned back on without a restart.
- The Projects surface shows the repository on disk and nothing else: `node scripts/check-projects-view.mjs`
  drives the built surface against host answers whose contents are known and passes **104 checks
  (2026-09-26)**: every class it uses is defined by the stylesheet; branch, switch and discard send the host
  exactly what was chosen, and Discard asks first; only the ticked changes are committed; a file the guard
  holds back is named and nothing is committed until Commit anyway says yes to it by name; choosing a change asks the host for that
  one file's diff and draws every line it returned with its number, added and removed lines marked as
  `<ins>` and `<del>` rather than by colour alone, a binary file by its sizes, and a late answer for a file no
  longer chosen is dropped; and Mint appears only on a clean tree, shows the commit it would pin by its full
  hash, and sends the host a path and an account and nothing else. The diff itself is the host's
  (`core/src/qontrol/diff.rs`): the working-tree side is converted to what git would store under the
  repository's `core.autocrlf` and `.gitattributes` before it is compared, and no diff driver's program is
  ever run.
- The Inventory shows what the chain holds and nothing else: `node scripts/check-inventory-view.mjs` passes
  **238 checks (2026-10-06)** against a fixture host — the whole script: the assets, the trade, the listing
  form, and selling and buying, all below — each asset's name, the reference it carries now, its
  pinned commit and whether it is permanent; "Make permanent" only where it can be; after it, the chain's
  next answer and not the view's guess; and nothing held, or a chain that cannot be read, each said as
  such. Since an asset is drawn as a card (`src/qfx/AssetCard.tsx`), it also opens every card as a person
  would and reads the close-up: the manifest's size, the reference the chain holds now, and what the asset
  was minted as only where it was revised. Proven to fail first against three faults in the view, and
  three more in the card.
- **The trade window has two sides and a lane**: what leaves this account, where it goes, and a mark that
  travels once per change, not in a loop. "Recently traded with"
  is this machine's own memory of trades the chain finalised (`core/src/partners.rs`), written nowhere
  else and sent to nobody; choosing one fills the destination.
- **A trade is driven end to end in the browser**, inside the same check: the menu opened by the keyboard and
  by the pointer's secondary button, its three items named (Trade, Sell, Copy reference — the label used to
  depend on how many assets the account held, which hid Trade from the person looking for it), the asset the
  menu was raised
  from already in the trade, another added from the account's other assets, the address and message typed,
  nothing sent before the warning, and the warning naming the destination, every asset and what cannot be
  undone. What reaches the host is read back: one call, that address, those assets, that message — and
  afterwards the view draws what the host says is held, not what it hoped. Proven to fail first against three
  faults: Review sending immediately, the trade sending everything held rather than what was chosen, and the
  menu button removed so only the right click remains.
- **Sell publishes a listing on chain, and Buy settles one** (L4.6, 2026-10-01), and the same check drives
  both from the view's side, 238 checks in all. Selling: the form says a listing is public and on chain and
  that the Market shows it with no search (before the Market existed, that there was no storefront); a typed price goes to the host as typed and what the host says a sale pays is
  drawn amount for amount, in the chain's order, with the host's total; a price no sale could settle at shows
  the host's reason and cannot be listed; a late answer for another price is dropped; a declined listing
  leaves the form open and says nothing was signed; an approved one sends the host that account, that asset
  and that text once, and the card then shows the price **the chain reports**, which the fixture makes differ
  from the one typed; a listed asset's menu offers Change price and Withdraw listing; and Withdraw asks the
  host once. Buying: the section names the Market and says it has no search; what is not an asset's number is refused in the
  host's words; a found asset is a card with its holder and its price, not counted among those held, with no
  holder's menu; an asset the host says cannot be bought shows the reason and no Buy; the purchase dialog
  shows every part of the price, the warning and the balance; a refused purchase shows the host's reason,
  reads the asset again and draws that; and the host is sent the price and the fingerprint that were on
  screen. **The fixture's amounts are ones no arithmetic on the price reproduces**, so a view that worked a
  split out for itself could not draw them. Proven against 19 faults planted in the view, one at a time, each
  failing the check; a twentieth, removing the re-read of held assets after a purchase, went **unnoticed**,
  because the Inventory already reads again whenever the store replaces the active account, which a click
  causes. That check is true and proves nothing.
- **The Market lists what is for sale on chain** (L7.2's first slice, 2026-10-02), on the rail and as a Nexus
  tile. `src/views/Market.tsx` draws `drc369_market` (`core/src/chain/market.rs`): every listing the
  chain holds, walked from `Drc369Royalties::Listings` through the connected node at its latest finalised
  block, bounded at 500, with details read only for the 24 on screen. **There is no indexer** (ADR-028), so
  it has no search, no history and no "newest"; the information icon beside its title says so, and the page
  names the block, the chain and the node it was read through. Each listing is the Inventory's card
  (`AssetCard`'s third mode, `listed`): name, the host's price string, holder, what it was remixed from, and
  the host's standing. A listing that cannot be bought — void, or held in place by nesting (ADR-065) — is
  drawn where it falls with the host's reason and no Buy; the viewer's own carries Withdraw, and a void one
  the viewer holds carries Clear. Filter (everyone's, others', yours) and order (cheapest, dearest, by
  number) are the host's three each, and both go to the host; pages go by the host's window. When the walk
  stops at its bound, and when a listing's asset cannot be read, the page says so. **Buy is the existing
  purchase**: the host reads the asset again (`drc369_sale`) and `BuyDialog` opens on that, so the price and
  fingerprint sent with `drc369_buy` are the host's fresh ones, and a price that moved since the list was read
  is said. After a purchase or a withdrawal the Market is read again, not adjusted.
  `node scripts/check-market-view.mjs` passes **127 checks** (2026-10-02) against a fixture whose prices do
  not follow from their embers, whose counts do not follow from the cards on the page, and whose purchase
  breakdown is not a split of the card's price. Proven against 24 faults planted in the view one at a time
  (2026-10-02), and **every one failed the check**: the view sorting the listings itself; formatting the
  price from embers; Buy offered on a listing held in place; unbuyable listings hidden; a void listing's
  reason hidden; the truncation notice dropped when anything was unreadable; the matching count and the
  yours count worked out from the page; a new filter keeping the old page; Buy opening the dialog on the card
  without asking the host; an older answer drawn over a newer one; an error shown only in the friendly line;
  Clear offered on a void listing someone else holds; Next and Previous moving by the cards drawn rather than
  the window; no re-read after a withdrawal or after a purchase; the information icon not saying where the
  list comes from; unreadable listings not mentioned; a failed read saying nothing is listed; the viewer not
  sent; the moved price not said; the order asked for not the one sent; the block and node not named. Nine
  of the 24 were caught by a wait timing out (the run stops at the line that is wrong and exits non-zero)
  rather than by a named check. **Not covered:** the jump back to the last page when the window is past the
  end (after the last listing on the last page goes), and the Refresh button's spinner. The host side has
  a live test that signs (`a_listing_made_here_is_read_back_by_the_market`: mint, list, read back as the
  seller's own and as someone else's to buy, then withdraw), **not yet run**, and a read-only one
  (`the_market_reads_whatever_the_node_holds`), run against a `spec_version` 6 development node on
  2026-10-02: 2 listings, read and decoded at block 861. With the Market in place, four of the Inventory
  check's assertions that the menu, the listing form and "Buy an asset" said "no storefront" or "nobody
  browses to it" now hold them to naming the Market and saying it has no search, which is what is true; the
  Inventory check's count is unchanged at 238.
- **The description is a second screen, and publishes nothing**: a title, a kind and that kind's questions,
  taken from the core's table (`core/src/listings.rs`) rather than a second copy in the view, saved on
  this machine only, and it says so before anything is typed. The chain holds a listing's price and nothing
  else, so the Market shows a price and no description, and until an indexer exists (M5.4) there is nowhere
  public for a description to go.
  **These blocks run after the trade block, so they use an asset the trade did not send away**; the first
  version used the traded one and passed silently, because the click that opens the menu used `?.` and a
  missing card made it a no-op. Those clicks do not use `?.`, and the new blocks wait for what they are about
  to read (`until`) rather than for a length of time.
- **Every run of text is readable as it is painted**, on every screen — the Gate with no vault, an older
  vault's last Windows Hello (cancelled) or last passphrase (Hello appears only there, to move an older vault
  to the keychain, ADR-056), a vault that would not open, a checked recovery phrase, and all
  eleven surfaces on the rail (the Market since 2026-10-02, with a listing of each standing and both its notices), plus an asset's menu, a trade, its warning, a file's diff, the discard question
  and what the guard holds back — in every theme,
  with the backdrop off and with a hostile white backdrop in the canvas's exact place:
  `node scripts/check-readability.mjs`, 361 checks over 16,750 runs of text, all AA (2026-10-02; 341 over 14,500 before the Market). It measures each run twice, from two screenshots: its declared colour
  over the background actually behind it, and its glyphs as actually drawn, so an overlay or a stacking
  mistake is caught as well as a weak colour. It was written because the launcher was unreadable with the
  backdrop live — the scrim was painted over the interface — while every other check passed; it failed on
  every theme against that tree, and it fails 15 of 15 backdrop screens when the bug is put back.
- Text is readable in every theme and over any backdrop: `node scripts/check-contrast.mjs` applies each
  theme through the app, checks every ink step down to `--ink-faint` on every background token, and solves
  for the worst colour the canvas could paint under the chrome's scrim. 54 checks (2026-09-22).
- `node scripts/check-design.mjs` passes its three rules (2026-10-06): no colour literals outside the token
  files, and no sizes or tracking off the scales. Since ADR-080 (6 October 2026) effects — glow, gradients,
  canvas, pointer-reactive light, looping animation — are allowed everywhere and the check no longer looks
  for them; reduce motion and the contrast and readability checks still bind.
- **The Gates surface shows the host's numbers and no others**: `node scripts/check-gates-view.mjs` passes
  34 checks (2026-09-17). It answers `gates_report` with a fixture whose numbers are known, opens the
  surface in the rendering engine and reads back what was drawn, so the view cannot round, weight or
  invent a number, drop a not-met unit, or draw a bar while anything is unmeasurable. It never reads
  `docs/GATES.toml`, because a check that read the same file as the host would pass if both were wrong
  together. It was proven to fail before it was trusted (see the script's header).
- Transfers carry the nonce from `system_accountNextIndex`, which counts the pool as well as the
  chain, so two sent in a row are both accepted (ADR-040). A Substrate account has one nonce; the
  custom devnet's separate request nonce (D-004) went with it.
- `tsc --noEmit` clean, production bundle 420 KB (132 KB gzipped).
- Binary builds and launches to a visible window, 38.7 MB resident.

An Electron equivalent would start around 120 MB resident and ship a bundled
Chromium; Tauri uses the system WebView and shares a language and crypto stack
with the chain.

---

## Design

The launcher wears Demiurge's one look (ADR-108) in its tool register: its default theme, **Obsidian**, is made from
`src/styles/tokens.ts`, and its four faces from `src/styles/tokens.css` and `src/styles/fonts/`, all three generated
by `node docs/design/tokens.mjs` from `docs/design/tokens.json`: edit that, never them. Every launcher moved to
Obsidian once (ADR-109); a theme picked after that stays. `node scripts/check-look.mjs` holds it. The design
system is [`docs/design/DESIGN_SYSTEM.md`](../../docs/design/DESIGN_SYSTEM.md). It covers tokens, themes,
the type, tracking and spacing scales, components, motion, the accessibility settings and what is
ruled out. `node scripts/check-design.mjs` fails on colour literals outside the token files and on sizes
or tracking off the scales. Since ADR-080 effects are allowed; reduce motion must still stop them moving, and the
contrast and readability checks still measure the text over them.

No face comes from the system or a font service: the launcher carries its own, so it renders the same offline and on
first run.

---

## What is next

The roadmap lives in exactly one place: **`docs/DIRECTION.md`**. This README
deliberately keeps no list of its own, because two roadmaps drift apart and then
contradict each other.
