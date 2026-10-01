# Market and Library

**Status:** Blueprint, 21 September 2026. Describes intent, not code that exists. Measured against the tree:
`construct_runtime!` mounts System, Timestamp, Aura, GRANDPA, Balances, Session, ValidatorSet and a
feature-gated Sudo, and `pallet-nfts` is not among them (`chain/runtime/src/lib.rs`); the launcher's host
commands download, verify, install, patch and launch nothing (`src-tauri/src/lib.rs`); and `Surface` has ten
variants, of which `library` renders a blueprint page (`views/Horizon.tsx`) and `market` is not one
(`src/state/store.ts`).

**Since 22 September 2026:** ADR-047, the object model and DRC-369's wire format, is accepted and M2.3 is ticked, and M2.1 was ticked the same day against the owner's review of a ten-line summary of the inventory's DRC-369 section. Wherever this document says something waits on "M2.1 and M2.3", it now waits on neither, and where it calls the content fingerprint undecided, it is a BLAKE3-256 manifest root with its algorithm tagged.

**Since 1 October 2026:** the launcher calls the settled sale the chain gained on 29 September
(`Drc369Royalties::list`, `unlist` and `buy`, ADR-061). In Inventory, Sell publishes a price on chain, and an asset
looked up by its number can be bought; both show what the sale pays before anything is signed
(`tools/qor-launcher/src-tauri/src/chain/sales.rs`). So step 5 of "How a purchase settles" below exists. Steps 1 and
7, the indexer, do not: there is still no catalogue and no Market surface, and a buyer reaches an asset only by a
number its holder gave them. One mismatch is recorded here rather than resolved: the pallet refuses a price of zero
(`ZeroPrice`), and decision 9 at the end of this document allows zero-price listings.

## What it is

Market is where a creator puts finished work up for sale and a buyer pays for it in CGT. Library is where
what you bought lives on your machine: it downloads, checks every byte against the fingerprint the chain
published, installs it, keeps it current, and starts it. The same two surfaces handle a 40 GB game, an
editor plugin for QOR Engine, a standalone tool and a theme that changes how the launcher looks — because
on chain all four are the same thing: one item, one fingerprint, one price.

## Who it is for

- **The creator who has something finished.** A game, a tool, an editor plugin, a scene, a colour theme.
  Today there is nowhere inside Demiurge to put it.
- **The creator who makes small things.** A theme is kilobytes; a preset is a table of numbers. Nothing in
  the games industry lets that person list, and the plugins-and-themes world pays them a tip jar. This is
  the population Market exists for, and the one that decides whether it was worth building.
- **The buyer.** Someone who wants to install a thing, be sure it is the thing, and not have it break next
  month.
- **The person who installs a plugin.** A separate hat from "buyer", because a plugin runs code inside the
  editor's process, and that changes what Market owes them at the install dialog.

## The foundation, and why

**The ownership ledger is `pallet-nfts`, through DRC-369 (ADR-025).** Not a custom marketplace pallet, not
a contract. An ownership record that silently loses an item, or lets the wrong account move one, fails in
exactly the column ADR-001 says to stay boring in: quietly, permanently, in someone else's favour.
`pallet-nfts` at the pinned tag already gives collections, items, metadata namespaces, per-item transfer
locks and a runtime-supplied `Locker` hook.

**Reused, not written:** the `set_price` / `buy_item` pair as a *model* for what a listing is; content-defined
chunking with per-chunk BLAKE3 and `zstd --patch-from` for deltas; the existing host-dialog signing boundary
(ADR-016, L1.4); the token-driven theme engine (`themes.ts`, 258 lines); and the design, accessibility and
contrast check scripts in `tools/qor-launcher/scripts/` (wired into CI, where no job has ever executed).

**Rejected, with reasons:**

| Alternative | Why it lost |
| --- | --- |
| `pallet-nfts::set_price` / `buy_item` as the purchase mechanism | `do_buy_item` transfers the price to the current owner and nothing else. It cannot pay a creator who is not the seller, so it cannot enforce a royalty. Kept as the **model** — a price, an optional whitelisted buyer, a listing cleared on transfer — and discarded as the **mechanism**. |
| Patching `pallet-nfts` so `buy_item` splits | Needs a written reason under ADR-013 §7, and "our storefront is special" is not one. A patched consensus-critical pallet is a permanent maintenance cost paid in the silent-failure column. |
| Making a listing itself a DRC-369 asset | Doubles the mint, doubles the deposits, and puts storefront copy into consensus state. Nothing in ADR-009 asks for it. |
| Small payloads stored on chain (`pallet-transaction-storage` indexes up to the runtime's maximum transaction size) | Technically available for a 2 KB theme. Rejected: it forks Library into two fetch paths, the Mesh into two manifest kinds, and the wire format into an "is it inline" discriminator forever. One rule — **fingerprint on chain, bytes on the Mesh** — for every payload kind. |
| A bespoke installer and patch format | Content-defined chunking and `zstd --patch-from` are mature and documented. Install correctness is a silent-failure problem: a corrupted patch produces a game that runs and is wrong. |
| Off-chain "credits" balances held by a Market service | That is a bank. It means trusting a storefront with other people's money, which is the thing the chain exists to avoid. |

**The catalogue comes from an indexer (ADR-028), not from the chain.** A Substrate node serves no history;
the launcher already refuses its own transaction-history call for that reason (L3.1). Search, ranking,
screenshots and copy are not consensus state.

## What is new

**A listing that carries a royalty policy, so a creator is paid every time their work changes hands —
including when the buyer is buying a remix of it.** No standard pallet does this; `pay_tips` is the nearest
thing in the pinned SDK and it is voluntary.

**One listing model for six payload kinds.** Game, tool, editor plugin, QFX scene, QFX preset and theme are
one item with one fingerprint and one price on chain, differing only in off-chain fields. The honest
finding is that this expansion **needs no new milestone** — it needs a `kind` discriminator and the same
four chain items L7.1 and L7.2 already depend on. Designed once, it is nearly free. Designed per kind, it is
four storefronts and a second wire format later.

**A preset is a mechanically verifiable remix.** A preset is a values-only override of a named parent
scene's parameter surface: the derivation is a diff, not a claim. That makes preset listings the cleanest
demonstration of remix royalties on the roadmap — the chain settles to a parent whose parenthood is
provable rather than asserted.

**A validator that runs at install time, not only in CI.** The same token-schema and contrast rules the
launcher already enforces on its own themes, run against a downloaded one. Without it, the product's visual
identity is crowdsourced by whoever downloads the most themes.

## Architecture

**On chain — all of it in DRC-369's pallets, none of it in Market's:** the item, a `(collection, item)` pair
that ADR-025 fixes as DRC-369's token identity and freezes into the wire format before any SDK ships; the
content fingerprint, a hash of the bytes, which does not exist yet; the price, `BalanceOf`, meaning CGT and
only CGT, which satisfies ADR-006's access-gating sink verbatim; the royalty policy and the settlement call
that honours it; and the distribution commitments under the demand sink.

**Off chain, in the indexer and the storefront:** the kind, the compatibility matrix, screenshots,
description, category, search, ranking, version notes. Committing an engine's version scheme to a chain
field would be a wire-format promise nobody should make.

**In the launcher host, all of it new Rust:** fetch, chunk, hash, verify, install, patch, launch. The webview
may not read the filesystem or spawn processes and must not gain those grants — it has no filesystem grant
at all, and its only process grants are `process:allow-exit` and `process:allow-restart`
(`src-tauri/capabilities/default.json`). Every byte of this
is host-side, behind the same dialog boundary that already guards signing.

### What a listing is

A listing is neither the asset nor the content. It is a third thing: **an offer to transfer or license a
specific item at a specific price, in CGT, to a buyer who is not yet known.** It decomposes into the item
identity, the content fingerprint, the price, the off-chain presentation and the royalty policy. Only the
first three and the last touch the chain.

### Where the payload kinds actually differ

| Kind | Bytes | Versioning | Compatibility matrix | Runs code on the buyer's machine |
| --- | --- | --- | --- | --- |
| Game | GB | Heavy | Platform | Yes, its own process |
| Tool | MB–GB | Yes | Platform | Yes, its own process |
| Editor plugin | MB | Tight, to an engine release | **QOR Engine build major/minor** | **Yes, inside the editor's process** |
| QFX scene | KB | Light | Scene-contract version | **Yes, on the GPU** |
| QFX preset | Bytes | Barely | The parent scene's parameter surface | No |
| Theme | KB | Barely | Token-schema version | No, unless it references a scene |

Every difference is off chain. That is the argument for doing the expansion with L7.1 and L7.2 rather than
after. Two honesty notes: QFX today is layer one's built-in backdrop and nothing more — no theme format, no scene,
no preset — so scene and preset are kinds the model provisions rather than kinds anything can list; and an editor plugin targets **QOR Engine, built on Godot** —
the custom build that tracks upstream — not upstream Godot.

### How a purchase settles, and what the chain still lacks

1. Buyer sees a listing — **indexer, does not exist.**
2. Launcher builds the call from the connected node's metadata — **exists, proven end to end (ADR-040, L3.1).**
3. Host dialog asks for approval before any signature — **exists (ADR-016, L1.4).**
4. Vault signs inside its own lock — **exists.**
5. Chain executes: CGT moves, the royalty policy splits it, the item transfers, events are emitted — **does not exist.**
6. GRANDPA finalises; the launcher reports only then — **exists.**
7. Indexer sees the events; Library sees a new entitlement — **does not exist.**

**Step 5 is the whole gap**, and it is seven things: `pallet-nfts` mounted (M4); `pallet-drc369` with the
fingerprint and owner index (M4.1); `pallet-drc369-royalties` with the priced transfer (M4.2); any
transaction fee at all (M6.4, and `WeightToFee` is **OPEN-4**); sponsorship so a buyer holding no CGT can be
onboarded (U-4, undecided, and if it is funded from the treasury or from perpetual issuance it also touches
**OPEN-2** and **OPEN-1**); the indexer (ADR-028, M5.4); and a treasury if the platform takes any share
(M6.5, **OPEN-2**).

**One trap worth naming in advance.** `pallet-nfts` reserves deposits for the collection, the item and
metadata, and who pays depends on the call: `mint` charges the caller even when minting to someone else,
`force_mint` charges the collection owner, `mint_pre_signed` charges the recipient. ADR-025 records that for
`set_metadata` the documentation and the code disagree — the documentation says the signer pays, the code
charges the collection owner. **Rely on the code, and test it.** Separately, the existential deposit is 100
CGT (`chain/runtime/src/denomination.rs`, ADR-036), so a purchase can leave an account below it: Market
handles that at the call site and **must not invent a minimum price** to paper over it.

### The transfer-lock problem

ADR-025 says royalty-bearing collections lock plain transfers and route priced transfers through the royalty
pallet. But `do_transfer` checks the `Locker` hook, the `TransferDisabled` system attribute and both the
collection and item transferable settings before it does anything — so the lock that stops a royalty-free
transfer also stops the royalty pallet's own. The boring way out is the **`Locker` hook**: it is a
runtime-supplied type, `pallet-drc369` implements it, and the settlement path sets the flag the hook
consults, which is what `pallet-nft-fractionalization` does with the same hook. The alternative — holding
the `Freezer` role and toggling each item around settlement — works and leaves a per-item unlocked window.

### Arithmetic

Every split uses `Perbill` or `Permill` for a fraction of an amount, or `multiply_by_rational_with_rounding`
for a ratio. Never `a * b / c` (ADR-035, AGENTS.md §5). A chain of remixes each taking a fraction of a
fraction is precisely where a hand-written multiply gets written, and `MIGRATION_INVENTORY.md` records that
the retired devnet's royalty library carried three conflicting caps and a double count.

## Substrate consumed

- **QOR ID** — identity, and the account a Library entitlement is keyed to. **A password-only QOR ID has no
  chain identity until it proves a key (ADR-017),** so a buyer who never linked a key cannot own or launch
  anything. Designed for at the Gate, not discovered at first launch.
- **DRC-369** — the item, the fingerprint, nesting (a bundle is a nested item; a preset names its parent),
  and royalties. **M4, unstarted, blocked on M2.1 and M2.3.** Nothing here works without it.
- **Qontrol** — the creator's side. A plugin, a tool, a theme and a scene are all source in a repository;
  Qontrol versions it and the Projects surface is where a creator tags the version they publish.
- **Mesh** — the bytes, as content-addressed chunks verified against the on-chain manifest hash. **M8.1,
  blocked on U-6.** A long-tail catalogue of themes and presets is mostly cold, so origin fallback is the
  normal case for small items — a hosted bill under ADR-015, not a peer-to-peer delivery that costs nothing.
- **CGT** — settlement, and the only currency a listing can be priced in.

No second identity, no second asset format, no second storage rail and no second payment rail appears
anywhere in this design. Where something is missing, it is recorded below rather than filled.

### Substrate gaps: named, not filled here

1. **Content fingerprint on chain.** M4.1. Without it a download has nothing to be verified against and
   Library is a download manager with extra steps.
2. **Ownership enumeration by owner, and its runtime API.** Custom, M4.1. "What do I own" is not a query
   `pallet-nfts` answers.
3. **Royalty-enforcing settlement.** Custom, M4.2. A genuine departure from standard components under
   ADR-013 §7, and it needs its own written reason.
4. **Offline entitlement proof.** An entitlement check asks whether a chain account owns an item. Offline
   launch needs a cached, expiring attestation, and an attestation needs a signer. If QOR ID signs it, QOR
   ID's say-so stands behind access to paid content — the trust ADR-017 and R-3 exist to keep out of value.
   **Named, not designed.** Either the chain provides a proof the launcher checks against a cached finalised
   header, or offline launch is an explicit recorded concession.
5. **A price a buyer can read in a stable unit.** U-2 and U-3. Whether "credits" is a formatting choice or a
   conversion is not stated, and a conversion needs a reference rate that does not exist and must not imply
   a peg. Catalogues get re-priced, which makes this urgent in a way a wallet does not.
6. **Fees.** OPEN-4. A Market with no transaction cost is a Market nobody has paid to spam.
7. **Sponsorship.** U-4. A buyer arriving with no CGT cannot buy and cannot be given a free first
   transaction, because no sponsorship mechanism is decided; its funding source is OPEN-1 or OPEN-2
   depending on the answer.
8. **Seeder payment for serving Library's bytes.** U-6 decides how verified seeding is proven, and the
   perpetual infrastructure issuance that would pay for it is OPEN-1. Market does not pay seeders locally.
9. **Build provenance.** The fingerprint is of the built artefact, not of the tree, so publishing needs a
   reproducible build or a human attesting "this binary came from that tag" — today it is the latter. That
   is a gap in Qontrol, recorded here, and Market must not grow its own signing story to cover it.

## The first usable slice

**A theme, published by someone else, installed and verified.**

The theme registry becomes loadable from disk instead of compiled in; the host fetches a theme, hashes it
against a manifest, refuses on mismatch, runs the token-schema and contrast validator, and writes it into
the registry. A restraint-rule violation is refused with the rule named, never silently clamped.

This is the only part of Market or Library that can ship before M4 and M8, because a theme with no
entitlement and no purchase is just a file. It must say so inside the product: **no CGT moves, nothing is
owned on chain, and the origin is trusted rather than verified against consensus.** That origin and its local
manifest are a dated stand-in for gap 1 and the Mesh, not a second format: the manifest is the hash structure
the chain fingerprint will hold, and it is deleted when M4.1 and M8.1 land. Outliving them turns it into a
substrate gap filled locally, wearing a shipping date.

It is worth opening because it proves the whole spine at kilobyte scale: fetch, hash, refuse, validate,
install, and a creator whose work is on someone else's screen.

## Phases to a full product

**Phase A — Themes, no chain.** Loadable registry, host-side fetch and hash check, install-time validator,
and a creator-facing "publish a theme" path that produces a manifest. No purchase, no ownership, and the
screen says so. *Weeks. Depends on nothing on chain.*

**Phase B — The installer core.** Content-defined chunking, per-chunk BLAKE3, a hashed manifest, delta
patching with `zstd --patch-from`, install-location management across drives, and supervised launch.
Verified against a local manifest until there is a chain fingerprint. **Chunking must be designed for delta
patching from the first line**; bolting it on afterwards is a rewrite. *~3 months.*

**Phase C — Listings and the catalogue.** The indexer, the listing model with its `kind` discriminator, the
Market surface, and the compatibility matrix the plugin case needs. Read-only: the catalogue lists, and
nothing can be bought. *~3 months, with an indexer spike first, because ADR-028's tooling condition is
unmet.*

**Phase D — Ownership and entitlement.** After M4.1: mint, transfer, fingerprint, owner enumeration. Library
gates launch on ownership, online, and the installer verifies against the chain's fingerprint instead of a
trusted origin — the moment verification starts meaning something. *~3 months after M4.1.*

**Phase E — Purchase.** After M4.2: the priced-transfer call that splits payment, the buy flow through the
host dialog and the vault, reported only once GRANDPA finalises, with deposits tested against the code
rather than the documentation. *~3 months, overlapping D.*

**Phase F — Distribution and scale.** After M8: Mesh delivery, the staking-for-distribution ordering, and
Library at game scale.

### Three months, twelve months, beyond

**In three months, one founder and an agent ship:** Phase A complete, and Phase B's installer core against
local manifests. That is a launcher that installs and updates verified content from a trusted origin. No
purchase, no ownership, no CGT.

**In twelve months, and only if M2.1 clears now:** Phases C, D and E — catalogue, ownership-gated launch
online, and purchase with royalty splits. M2.1 is one document and one sitting, it blocks M4, and M4 blocks
everything after Phase B; a month of delay there is a month of delay here, serially, not in parallel.

**Beyond twelve months, or not estimable at all:** Mesh delivery and Library at game scale (M8.1, blocked on
U-6); the staking sink's disposition (U-7, with OPEN-2 and OPEN-4); offline launch (gap 4); anything needing
fees (OPEN-4) or a treasury (M6.5, OPEN-2). These are blocked on decisions, and a date for a decision is a
guess, so none is given.

## The demand sink

Market's sink is **staking for distribution**, the third of ADR-006's five: creators commit CGT for reach and
shelf space instead of buying advertising. Visibility is scarce, and it is allocated by committing the
network's own currency to it. The spend test (ADR-002) is passed at two points and both are spends, not
holds: a buyer spends CGT to be licensed content, and a creator spends CGT to be placed.

**The mechanism can be built now because placement is an ordering, and an ordering is relative.** No absolute
value is needed anywhere — no minimum, no price per slot, no reserve, no clearing price:

- A creator commits an amount to a `(shelf, epoch)` pair.
- Placement within a shelf is by descending share of that shelf's epoch total.
- Ties break deterministically on something that is not money — the item id.
- A shelf has a finite number of slots. **Slot count is an operating parameter set by governance, not an
  economic value.** It is not an issuance rate, a genesis split, a decay curve, a burn share or a fee class,
  so AGENTS.md §5 does not reach it — and wherever it is written down it should say so.
- Epoch length is a block count, on the same footing.

**What must not be guessed: U-7, what happens to the committed CGT.** Locked and returned, spent to the
treasury, or burned. Locked-and-returned is a deposit, not a sink: a creator holding a large balance keeps
the top slot indefinitely at no cost and the shelf never turns over. Spent-to-treasury needs a treasury
(M6.5, OPEN-2). Burned is a burn share (OPEN-4). **These are not three implementations of one mechanism;
they are three different products.** Building the ordering first is safe, because the ordering code is
identical under all three. Shipping it with a placeholder disposition to a network anyone uses is not.

**Language.** Placement is never described as something a creator obtains, and nothing about a listing's
position is a promise. ADR-008 governs every word of a storefront, and a storefront is exactly where that
language creeps in.

## Dependencies

| Needed | Item |
| --- | --- |
| Owner reads the migration inventory | **M2.1** — unticked, and it blocks all of M4 |
| DRC-369 wire format decisions | **M2.3** |
| Ownership, mint, transfer, collections, owner enumeration, content fingerprint | **M4.1** |
| Nesting, state, royalties settled in CGT | **M4.2** |
| Sponsored fees and deposits, so a new creator can mint without holding CGT | **M4.4** (U-4) |
| DRC-369 acceptance tests, including R-2 | **M4.5** |
| Indexer and the public viewer that shares its listing pages | **M5.4**, ADR-028 |
| Proxy types with spend caps, for launch without a signing prompt | **M5.2** (ADR-026) |
| Transaction payment | **M6.4** (OPEN-4) |
| Treasury, if any platform share exists | **M6.5** (OPEN-2) |
| Mesh: seeder verification and payment | **M8.1** (U-6, OPEN-1) |
| Entitlements from DRC-369, for the Library | **M8.2** |
| Staking for distribution, for Market | **M8.3** (U-7) |
| Escrow and reputation bonds, for commissioned work | **M8.4** (U-8) |
| Host dialog before every signature | **L1.4** — implemented, unticked pending a running launcher |
| Chain client on metadata | **L3.1** — done |
| Studio, for the mint that precedes a listing | **L4.1, L4.2** |
| Signed installers and a signature-verifying update channel | **L6.1, L6.2, L6.3** — the same verification spine, and the reason Library must not invent a second one |
| Library: install, delta patch, launch | **L7.1** |
| Market | **L7.2** |
| Mesh seeding | **L7.4** |

One repair belongs with L7.1: the Library blueprint page promises "launch with a short-lived session key" and
claims a session-keys module "exists and its storage works" (`Horizon.tsx`). Both describe the retired
devnet; session keys in Substrate are validator keys, and the mechanism meant here is `pallet-proxy` with
spend caps (ADR-026, M5.2). Rewrite that page when Library is next touched.

## What "full support" means, and what it does not

**It means:** one listing model across games, tools, editor plugins, scenes, presets and themes; a download
whose every chunk is checked against a fingerprint the chain published; delta updates; an entitlement that
belongs to an account rather than to a storefront's database; a creator paid on the first sale and on every
resale their royalty policy covers; and a remix that pays its parent because the derivation is provable.

**It does not mean:**

- **Safe plugins.** An editor plugin is native code loaded into the editor's process. Market can make it
  *attributable* — a named creator, a fingerprint, a version, a revocable listing — and cannot make it
  *safe*. Anyone who installs one is running that author's code. Say so at the install dialog, in those
  words.
- **A sandboxed QFX scene.** A scene is a shader. An unbounded loop in a fragment shader is a GPU denial of
  service, and the industry's answer is recovery rather than prevention: the Windows TDR watchdog at about
  two seconds, Chromium's GPU watchdog at about ten, and a lost context through `GL_EXT_robustness` — which
  means every accelerated window on the machine flickers. A scene compiles in a worker under a wall-clock
  cap, a lost context suspends QFX rather than the shell, and the shell renders identically with QFX absent.
  That is containment, not sandboxing.
- **Any theme you can imagine.** The restraint rule — exactly one accent, one counter (`themes.ts`) — is
  load-bearing and will be attacked by exactly the population a theme marketplace attracts. A theme failing
  the token schema or the contrast floor is refused at install. Removing that gate crowdsources the
  launcher's visual identity, and that is the owner's decision, not an implementation detail.
- **Offline launch.** Not until gap 4 is decided. An entitlement check needs the chain.
- **Buying without CGT.** No fiat rail, no card, no bridge, and no sponsorship mechanism (U-4). A buyer
  arrives with CGT or does not buy. **The owner asked on 22 September 2026 for exactly this to change** — a
  marketplace on demiurge.cloud selling assets for USD, with a fixed platform share proposed at 15% — and it is
  open as **U-15**: what the share is of, where it goes with no treasury (M6.5), and who holds the money, which
  brings custody, KYC/AML, chargebacks and escrow (U-8). Recorded as asked for, not as decided.
- **Anything about what CGT is worth.** A price is a price and a royalty is payment for use; neither surface
  makes a claim about what CGT is worth, now or later, in copy, identifiers or documentation (ADR-008).
- **Curation, moderation or takedown.** There is no review team, no ratings, no reviews, no dispute process
  and no takedown path. A listing is attributable, not vouched for.
- **Refunds, chargebacks, age ratings, regional pricing, wishlists, bundles, sales, achievements, save sync
  or anti-cheat.** None is in scope, and several are products in their own right.
- **DRM.** Verification proves a download is the published bytes. It does not stop anyone copying them.
- **A mirror of upstream Godot's asset library.** Plugins listed here target QOR Engine, built on Godot — a
  custom build that tracks upstream but is not upstream. Whether an upstream plugin may be listed at all is
  open question 5.
- **Published content surviving on its own.** A listing points at content the Mesh holds. If nobody seeds it
  and the origin fallback is not paid for, the listing outlives the bytes.

## Risks

1. **M2.1 is one unticked line and it blocks the entire product.** Every phase past B waits on it.
2. **The chunking format is a one-way door.** Design delta patching in at the start or rewrite the installer.
3. **The royalty split is the hardest correctness work in M4**, and this project has shipped a royalty
   library with conflicting caps and a double count once already.
4. **The compatibility matrix is how a marketplace becomes a support burden.** A plugin declaring the wrong
   engine range produces a broken editor and a ticket for the platform, not for the author.
5. **Third-party themes erode the visual identity by default**, and the validator is the only thing in the way.
6. **The staking sink may ship as a deposit rather than a sink** if U-7 is settled by drift, not decision.
7. **Cold catalogues cost money.** Long-tail items are served by the origin fallback, a hosted bill under
   ADR-015, and seeder payment is blocked on U-6 and OPEN-1.
8. **Re-pricing a catalogue** is where U-2 and U-3 stop being theoretical.
9. **A storefront is the highest-pressure surface for ADR-008 language in the project**, and it gets written
   by whoever is closest to a deadline.
10. **The Phase A stand-in becomes permanent**, and a trusted origin quietly stands in for the Mesh.

## Decided by the owner, 22 September 2026

Each answer is the owner's and is reversible.

1. **Third-party themes, scenes and presets may be listed.** **Safety checks cannot be overridden by a user;
   taste checks can.** The contrast floor and shader validation are safety; the restraint rule's palette
   judgements are taste.
2. **Deferred to M6, not decided: U-7**, what happens to CGT committed for placement. Staking for distribution
   is recorded as a dependency on M6 as well as M8.3.
3. **Deferred to M6, not decided: any platform share of sales.** Recorded as a dependency, with the treasury it
   would need (M6.5, OPEN-2).
4. **Offline launch is required.** That makes gap 4, a cached and verifiable ownership proof, a requirement of
   P5.4 rather than an option, and it must not put QOR ID's attestation behind access to paid content.
5. **Plugins for upstream Godot may be listed.**
6. **No native code without published source** — and, from the QOR Engine decisions, a signature. The details
   belong in Market's ADR, which is not yet written.
7. **Prices display in credits over CGT settlement** (ADR-007): a display unit, not a second currency, and no
   peg.
8. **The creator mints; Market lists** (ADR-009, ADR-014). Market never mints on a creator's behalf.
9. **Zero-price listings are allowed.**
