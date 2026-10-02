# QOR Launcher

The sovereign gateway to the Demiurge ecosystem. One desktop application through
which every chain and engine system is reached: QOR ID, the CGT Vault, the game
library, community, peer-to-peer distribution and the node itself.

**Status:** Gate, Nexus, Vault, Inventory, Projects, Chain, Gates and Settings
are built and running, and so is the first slice of Market (L7.2): every listing
the chain holds, read through the connected node, with no search and no indexer
behind it. Library, Social and Mesh are placeholder pages that state their own
blockers rather than showing mock content. Of the L1
roadmap items, L1.1, L1.2, L1.3 and L1.5 are ticked; **L1.4 is implemented but
unticked** until its native dialogs are exercised in a running launcher, and
L1.6 and L1.7 wait on CI running at all. L3.1 and L3.2 are done.

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
│   │   └── ui/              The panel surface
│   ├── lib/ipc.ts           The ONLY module that calls into the host
│   ├── qfx/                 The living backdrop (QFX layer one): one WebGL2 canvas behind
│   │                        the interface, under the chrome's scrim
│   ├── state/store.ts       One Zustand store
│   └── views/               Overview, Vault, Chain, Inventory, Market, Release gates, Projects, Settings, Horizon
├── src-tauri/               Rust host (everything security-relevant)
│   ├── src/cgt.rs           Denomination: parsing, formatting, precision
│   ├── src/chain/           subxt client for the Substrate chain in chain/ (L3.1, ADR-040),
│   │                        including DRC-369 mint, make-permanent and enumeration (M4.1),
│   │                        selling and buying settled on chain (chain/sales.rs, L4.6),
│   │                        and the Market's read of every listing (chain/market.rs, L7.2)
│   ├── src/content/         The object model (ADR-047): manifest, BLAKE3 root, 41-byte
│   │                        reference, and the temporary content store
│   ├── src/gates.rs         Release-gate progress from docs/GATES.toml (L2.2)
│   ├── src/identity/        QOR ID auth + OS keychain token storage
│   ├── src/qontrol/         Qontrol's port: gitoxide for every read, the helper for staging,
│   │                        switch and discard, and the guard before staging
│   └── src/vault/           BIP-39, Sr25519 (sp-core), Argon2id, XChaCha20-Poly1305
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
| In memory | `Zeroize` throughout | The seed is wiped on lock, on exit, and on the idle timer. |
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
(2026-09-28)**: a keychain vault reaches the shell with no lock screen and
nothing typed; QOR ID down stops nothing and Settings signs in with the vault,
never a password; a first run is one button sending the host the phrase alone; a
keychain vault that will not open says to restore from the phrase; a Hello vault
asks once and not again after a cancel; a passphrase vault asks its passphrase
without Hello being asked; a typed phrase goes to the host normalised and its
account is shown before anything is restored; and Settings shows the phrase
through the host.

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

`src/qfx/ceremony/` is the one folder where the design check allows glows,
gradients and looping animation (the owner's decision, 2026-09-28,
`docs/design/DESIGN_SYSTEM.md`). Explanatory text elsewhere sits behind an
information icon (`src/components/ui/InfoTip.tsx`) that opens on hover or
keyboard focus.

### Running QOR ID and a chain on this computer

`powershell -File scripts/start-local.ps1` starts what the launcher's settings
point at: QOR ID on `127.0.0.1:8080` with Postgres (`qor-local-pg`, port
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

---

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

All amounts are integer Sparks in `u128`, never floating point, and the `u128`
crosses the IPC boundary as a decimal string because JavaScript's `number` cannot
hold it.

**Typing an amount round-trips to the host.** The Rust side parses it and returns
the exact Spark value it would transfer, so the number shown on the confirmation
screen is the number that gets signed rather than a JavaScript approximation of
it. Views show amounts only as the host formats them, in CGT; no Spark count and no
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

Every conversion goes through `src-tauri/src/cgt.rs`. Amounts are integer Sparks
in `u128`, and excess precision is rejected rather than truncated.

---

## Running it

```bash
cd tools/qor-launcher
npm install
npm run app:dev        # dev, with hot reload
npm run app:build      # release bundle (msi, nsis, deb, appimage, dmg)
npm run icons          # regenerate the icon set from scripts/make_icons.py
npm run check          # design, accessibility, gates view, Projects view, Inventory view, Market view, the Gate's restore and Hello, QFX contrast, readability (needs npm run build first)
```

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

The launcher points at `ws://127.0.0.1:9944`. The chain surface shows the name
the node reports, so you can see which chain answered.

The live test below exercises the whole path against that node, and is the only
part of the client a test cannot cover without one:

```bash
cd src-tauri && cargo test --lib chain::live -- --ignored --nocapture
```

### Verified

- 193 Rust host tests passing with no node running (2026-10-02; 182 on 2026-10-01), 39 of them Qontrol's and 7 the object
  model's (ADR-047: the same files in any order make the same reference, one changed byte changes it, the
  reference is the 41 bytes the chain stores, and the manifest's encoding is pinned). Build the helper
  first: without it the Qontrol tests that commit skip and say so, and under
  `cargo test --features qontrol-no-skips`, which CI uses, they fail instead. The helper's own 16 tests
  pass in `qontrol-git/`.
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
  reports, part for part; and the four balances are asserted to the Spark, with amounts written out in the
  test rather than computed (2,000 CGT and seven Sparks: 100 and 300 CGT to the source's two recipients, 40
  CGT to the remix's own, 1,560 CGT and the seven Sparks to the seller). It also proves a listing below what
  a recipient could receive is refused before anyone is asked, a price or fingerprint that changed since the
  buyer looked is refused before anyone is asked, a void listing can be cleared by anyone, and the chain's
  own `PriceAboveLimit` is put in words. The test sends `set_terms` and a remix mint itself, because the
  launcher has no surface for either. All four live tests passed together on 2026-10-01 (377 s).
- **A purchase is held to the work the buyer saw, on chain** (2026-10-02). Buy sends
  `Drc369Royalties::buy_exact` with the price and the content reference that were on screen, so a revision
  landing between the dialog and the block is refused by the chain (`ContentChanged`, put in words) and not only
  by the host's check before signing. It needs a node at `spec_version` 6 or later. The five live tests passed
  against one on 2026-10-02 (377 s), the sale among them.
- **What a sale pays is the chain's arithmetic, not the launcher's.** `src-tauri/src/chain/sales.rs` carries
  the pallet's `split`, line for line, on `sp-arithmetic` at the version the pinned SDK release uses
  (`=28.0.1`, ADR-033 rule 1): `Permill::mul_floor` and `multiply_by_rational_with_rounding`, no float and no
  hand-written `a * b / c`. Its four arithmetic tests are the pallet's own vectors, the largest intermediate
  included. After a sale, what each account received is read from the `Sold` event and nothing else.
  Eleven faults planted in that module on 2026-10-01 each failed at least one of its 19 tests.
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
  (`src-tauri/src/qontrol/diff.rs`): the working-tree side is converted to what git would store under the
  repository's `core.autocrlf` and `.gitattributes` before it is compared, and no diff driver's program is
  ever run.
- The Inventory shows what the chain holds and nothing else: `node scripts/check-inventory-view.mjs` passes
  **127 checks (2026-09-26)** against a fixture host — the whole script, of which the assets half below is
  72, the trade 29 and the listing form 16 — each asset's name, the reference it carries now, its
  pinned commit and whether it is permanent; "Make permanent" only where it can be; after it, the chain's
  next answer and not the view's guess; and nothing held, or a chain that cannot be read, each said as
  such. Since an asset is drawn as a card (`src/qfx/AssetCard.tsx`), it also opens every card as a person
  would and reads the close-up: the manifest's size, the reference the chain holds now, and what the asset
  was minted as only where it was revised. Proven to fail first against three faults in the view, and
  three more in the card.
- **The trade window has two sides and a lane**: what leaves this account, where it goes, and a mark that
  travels once per change (never a loop — `check-design.mjs` forbids that everywhere). "Recently traded with"
  is this machine's own memory of trades the chain finalised (`src-tauri/src/partners.rs`), written nowhere
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
  tile. `src/views/Market.tsx` draws `drc369_market` (`src-tauri/src/chain/market.rs`): every listing the
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
  not follow from their Sparks, whose counts do not follow from the cards on the page, and whose purchase
  breakdown is not a split of the card's price. Proven against 24 faults planted in the view one at a time
  (2026-10-02), and **every one failed the check**: the view sorting the listings itself; formatting the
  price from Sparks; Buy offered on a listing held in place; unbuyable listings hidden; a void listing's
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
  taken from the host's table (`src-tauri/src/listings.rs`) rather than a second copy in the view, saved on
  this machine only, and it says so before anything is typed. The chain holds a listing's price and nothing
  else, so the Market shows a price and no description, and until an indexer exists (M5.4) there is nowhere
  public for a description to go.
  **These blocks run after the trade block, so they use an asset the trade did not send away**; the first
  version used the traded one and passed silently, because the click that opens the menu used `?.` and a
  missing card made it a no-op. Those clicks do not use `?.`, and the new blocks wait for what they are about
  to read (`until`) rather than for a length of time.
- **Every run of text is readable as it is painted**, on every screen — the Gate's three states, its unlock
  screen with Windows Hello on and just cancelled, a checked recovery phrase, and all
  eleven surfaces on the rail (the Market since 2026-10-02, with a listing of each standing and both its notices; Settings with its Windows Hello panel), plus an asset's menu, a trade, its warning, a file's diff, the discard question
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
- `node scripts/check-design.mjs` passes: no colour literals outside the token files, no sizes or tracking
  off the scales, and no glows, gradients, pointer-following light or canvas animation — except that
  `src/qfx/` is exempt from the canvas and pointer rules, and from nothing else (2026-09-21).
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

The launcher is the reference implementation of the Demiurge design system:
[`docs/design/DESIGN_SYSTEM.md`](../../docs/design/DESIGN_SYSTEM.md). It covers tokens, themes,
the type, tracking and spacing scales, components, motion, the accessibility settings and what is
ruled out. `node scripts/check-design.mjs` fails on anything the document rules out.

System fonts only: a launcher must render correctly offline and on first run.

---

## What is next

The roadmap lives in exactly one place: **`docs/DIRECTION.md`**. This README
deliberately keeps no list of its own, because two roadmaps drift apart and then
contradict each other.
