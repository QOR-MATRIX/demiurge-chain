# Stream (placeholder name)

**"Stream" is a placeholder name for the music product.** Nothing in the repository uses it, no scope has ever been invented for it, and naming it is the owner's decision — the owner has not named it at all. Since 21 September 2026 it has a track under the placeholder, P6, whose first item is a decision rather than code, and `docs/SYSTEMS.md` lists it as planned. Every use of "Stream" below is a handle for discussion, not a proposed name.

**Status:** Blueprint, 21 September 2026. Describes intent, not code that exists.

**Since 22 September 2026:** ADR-047, the object model and DRC-369's wire format, is accepted and M2.3 is ticked, and M2.1 was ticked the same day against the owner's review of a ten-line summary of the inventory's DRC-369 section. Wherever this document says something waits on "M2.1 and M2.3", it now waits on neither, and where it calls the content fingerprint undecided, it is a BLAKE3-256 manifest root with its algorithm tagged.

**It does not exist before M8, and M8 is not scheduled.** This is the furthest out of the six products by a wide margin, and not a close margin. Read this document as a set of constraints on other people's work, not as a plan to start.

## What it is

A place where a musician publishes a track, an album or a pack of stems, and is paid when someone uses it — plays it, licenses it for a film, or takes the stems and builds something new. The audio is stored and served by the same network that serves everything else on Demiurge, so the people whose machines carry it are paid for carrying it. Settlement is in CGT, and who gets what share — writer, performer, the artist whose loop was sampled — is decided by the work's own royalty policy and applied when the money moves, rather than months later by somebody's accounting department.

## Who it is for

- **Independent musicians and producers** who want their work distributed and to be paid for its use without a label in between.
- **Sample and stem makers**, who today sell through storefronts that take a cut and give the buyer no provenance and the creator no claim on what is built from the pack.
- **Remixers and beatmakers**, for whom the interesting part is not distribution but that a remix can carry a settled, enforced share back to the original.
- **Music supervisors and game developers** buying a sync or in-game licence, who currently need a lawyer to find out who they have to ask.
- **Listeners**, last and deliberately last. A listener has no reason to prefer this over what they already pay for, and this document should not pretend otherwise until the catalogue is itself the reason.

## The foundation, and why

ADR-001: build on proven foundations where failure is silent; invent at the creator-economy layer. Audio delivery and payout aggregation are both columns where failure is silent — a codec artefact, a swarm that starves, a settlement run that pays the wrong split. None of those throw.

**Codec: Opus (RFC 6716) — reused, not written.** IETF-standardised in 2012, royalty-free, 6 kbps to 510 kbps, 8 kHz to 48 kHz, frames from 2.5 ms to 60 ms, and near-transparent stereo music around 96 kbps where AAC needs roughly 128 kbps. The choice is not close.

- **AAC** loses on licensing. A codec whose licensing implies a per-stream obligation to a third party is the wrong foundation for a platform whose entire argument is that creators are paid for use.
- **MP3** loses on quality per bit and has no advantage left.
- **FLAC** is not rejected — it is the archive and lossless-delivery path, not the streaming path. Both exist from the start, or neither does cleanly (see Risks).
- **Anything proprietary or novel** loses under ADR-001 outright.

**Distribution: the Mesh (M8.1), with standard swarm engineering on top.** Streaming from a torrent-style swarm is a solved-enough problem with a known failure mode: naive sequential piece picking causes piece extinction, because every sequential leecher asks for the same early pieces and the tail is neglected. The documented fix is a hybrid — sequential inside a near-term deadline window, rarest-first beyond it, time-critical pieces given the best request-queue slots. That is real engineering rather than configuration, but it is written down and it is not ours to invent.

- **A dedicated streaming CDN as the primary path** loses because it is a second storage rail, which is a substrate gap papered over. The Mesh is the storage rail.
- **A per-track swarm** loses on arithmetic: swarm overhead that amortises over a 40 GB game does not amortise over a 4 MB track. Per-album or per-catalogue-shard swarms are the shape the constraints point at. Offered as a direction, not a verified design.

**Settlement: netting per period, then one ordinary payment per creator per period.** Rejected alternatives, each for its own reason:

- **Per-play on-chain settlement** — infeasible on this chain. The arithmetic is under Architecture and it is not a close call either.
- **`pallet-utility::batch` as the mechanism** — it saves the per-extrinsic base weight and one signature check, not the per-transfer storage work: roughly a 1.66× improvement, not an order of magnitude. It is also one origin paying many recipients, which does nothing for many listeners paying many creators. Useful as the last step of a settlement run; useless as the design.
- **Payment channels** — classic, well understood, and requiring dispute windows, timeouts and a watchtower story. The pinned SDK has no payment-channel pallet. This is squarely an ADR-001 "stay boring" concern with no boring option available, and a channel that fails, fails silently and in someone's favour. **Do not.**

**Identity, assets, versioning, currency: QOR ID, DRC-369, Qontrol, CGT.** Not chosen here, inherited. A music product that needed its own login, its own file format, its own storage or its own payment rail would be a substrate gap, not a feature.

## What is new

Everything above is borrowed. The part nobody else has is this:

1. **A royalty policy applied at settlement, not asserted in a contract.** Writer, performer, publisher, sample source — the split lives with the work as a DRC-369 policy and the chain applies it when the payment moves. Remix chains are the hard case and the interesting one: a remix of a remix carries a share back along the chain, arithmetic checked once, then permanent.
2. **The seeder who serves the audio and the creator who made it are paid by the same event.** Distribution cost and creator payment stop being two unrelated systems with a platform in the middle.
3. **Stems and remix rights are first-class items, not a download link.** A buyer of a stem pack gets a DRC-369 item with a content fingerprint and provenance; what they build carries a verifiable relationship to what they bought.
4. **Licensing a buyer can complete alone.** A game developer who wants a track reads the policy on chain, pays it, and holds an entitlement. No email thread, no clearance house.
5. **The creator's project history is versioned with the same tool as everything else.** Qontrol's foundation research already covers the DAW formats: `.als` is gzip-compressed XML that references samples by path, `.rpp` is plain text and natively diffable and mergeable, `.flp` is binary, `.logicx` is a directory package the DAW may rewrite wholesale on save. Stems and masters live in the content-addressed side store, not in history as whole-file revisions.

## Architecture

Six layers, of which exactly one is Stream's own.

| Layer | What it does | Whose it is |
| --- | --- | --- |
| Authoring | DAW project, stems, masters, versioned; large audio deduplicated by content-defined chunking into `.qontrol/cas/` | **Qontrol** |
| Publishing | Mint the release as DRC-369 items, record the content fingerprint, attach the royalty policy | **DRC-369 (M4.1, M4.2)** |
| Distribution | Content-addressed chunks verified against the on-chain fingerprint, per-album swarms, origin fallback when a swarm is cold | **Mesh (M8.1)** |
| Playback | Opus decode, deadline-aware piece picking, and **consuming whatever delivery receipt U-6 defines. Stream does not define one** | **Stream** |
| Settlement | Net a period's obligations, pay once per creator per period in CGT | **DRC-369 royalties (M4.2) + a netting mechanism that is chain work, not music work** |
| Discovery | Catalogue, search, playlists, shelf placement | **The indexer (ADR-028, M5.4)** — a Substrate node serves no history, which is why `cgt_history` already refuses. Not a music-only index |

### Why per-play on-chain settlement is not on the table

Two independent arguments. The second is fatal in a way the first is not.

**Throughput, measured against this runtime rather than guessed.** The normal-class budget is 1.5 s of `ref_time` per block (2 s maximum block weight × 75% normal dispatch ratio), less block execution weight. One signed `transfer_keep_alive` costs 272,208 ns all-in at the pinned weights (`ExtrinsicBaseWeight` 108,157 ns, the call at 39,051 ns, one `RocksDbWeight` read and one write). That is **≈ 5,509 transfers per block, ≈ 918 per second, ≈ 79.3 million per day — with nothing else in any block, ever.** A royalty-splitting settlement is much heavier: read the item, read the policy, read and write each recipient. At four reads and four writes that is an **estimated** ~700 µs, or **≈ 2,100 per block, ≈ 30 million per day**. As an illustration of scale and not as a forecast: a service with 100,000 daily listeners averaging 50 plays would consume about 5 million settlements a day — roughly **17% of the entire chain's capacity, permanently, for one product**. That says "expensive and rude". It does not say "impossible".

**The existential deposit, which does say impossible.** `EXISTENTIAL_DEPOSIT = 100 * CGT` (`chain/runtime/src/denomination.rs`, ADR-036, derived from the owner's target under ADR-030 — a decided value, not one invented here). `transfer_keep_alive` fails if the transfer would leave the recipient below it. A per-play payment is by construction the smallest payment the system makes. **A chain whose minimum viable account balance is 100 units cannot be the settlement layer for the smallest payment it makes** — unless one play is worth a meaningful multiple of 100 CGT, in which case it is not a micropayment and the entire framing was wrong. Split that play across a writer, a performer and a publisher and it is three payments, each smaller still.

Other platforms publish per-stream rates. **They are not repeated here**, because a monetary figure standing next to a per-play argument reads as a target for what CGT is worth, and this project states nothing about CGT's value (ADR-008). The structural argument needs no such figure: it is about a minimum balance against the smallest payment, in units, and it holds whatever anything is worth.

Audius is not a counterexample. Its enabling conditions are a fee floor near zero, no meaningful existential deposit, and a multi-recipient splitter primitive. Demiurge has a 100 CGT existential deposit by deliberate decision and no splitter. The design does not port.

**One thing the record should say plainly.** ADR-009 calls music "the cleanest per-play micropayment story". That is true as an accounting observation — music has the cleanest per-unit accounting of any creative medium — and is **not** a claim that each play is an extrinsic. It would be easy to read it that way, and nobody should.

### What batching and aggregation would actually require

**Netting is the approach.** N plays of creator C within a period become one number. Settle `pot → C` once per period. The payment per creator per period is then large enough to clear the existential deposit, the settlement count collapses from *plays* to *creators × periods*, and the arithmetic above stops binding. Concretely, this requires:

1. **A netting and payout mechanism that does not exist.** Nothing standard does this. It holds a period's obligations, closes the period, and pays. ADR-035 and AGENTS.md §5 govern its arithmetic absolutely: `Perbill`/`Permill` for a fraction, `multiply_by_rational_with_rounding` or `FixedU128` for a ratio, never a hand-written `a * b / c`. At eighteen decimals two amount-sized `u128` values multiplied together overflow long before the result would, and the deleted chain's royalty library already shipped this failure once — three conflicting caps and a double count. **This is chain work, not music work** (see Substrate gaps).
2. **A dust rule.** A creator owed less than the existential deposit at period close cannot be paid. Carry forward, or forfeit to somewhere. Carry-forward is the only option that needs no decided destination; a forfeiture destination is **OPEN-2** (treasury) or **OPEN-4** (burn share), and both are undecided. **Do not pick one.**
3. **An unclaimed-funds rule**, if the payout is a claim rather than a push. A Merkle-root claim — accumulate a period's obligations off chain, write one root, let each creator claim with a proof of `(account, amount)` — is the standard mass-payout pattern and costs **one on-chain write per period regardless of how many creators are owed**. Its costs are a custom claim pallet, that unclaimed-funds rule, and shifting a transaction fee onto the creator at precisely the moment they are being paid. What that fee is, and what share of it burns, is **OPEN-4**.
4. **A play count the chain can rely on.** This is the whole problem, and it is not Stream's.

The period length itself is a block count — an operating parameter set by governance, like Market's shelf slot count. It is not an issuance rate, a genesis split, a decay curve, a burn share or a fee class, so AGENTS.md §5 does not reach it. Say that explicitly wherever it is written down, or someone will assume it does.

### The counting gap, which is the real one

**Nothing on this chain knows a play happened.** A play is an event on a listener's machine, reported by software the listener controls, about bytes served by a seeder who is paid more if there are more plays. Every batching scheme above assumes something counted correctly, and nothing does.

That is **U-6** verbatim: "What a seeder proves, to whom, and how often (proof of storage, proof of retrieval, challenge-response, client receipts) is undefined. Without it, the largest demand sink pays for unverifiable claims."

A streaming product does not get to invent its own play-counting oracle. That would be a second attestation authority sitting next to QOR ID, off chain, deciding who gets paid — which is exactly what ADR-017 and requirement R-3 exist to prevent.

- If **U-6 is answered with signed client receipts a seeder redeems**, Stream gets play counting *for free*, as the same primitive.
- If **U-6 is answered with a storage proof that says nothing about delivery**, Stream has no counting mechanism at all and would need its own — which it must not build.

**U-6's answer determines whether this is a product or a research project.** It is the single most important thing in this document, and the only thing on this page worth acting on today.

## Substrate consumed

| Substrate | How Stream uses it |
| --- | --- |
| **QOR ID** | The creator's and the listener's identity. Entitlements key to a **proven chain key**, not a username — a password-only QOR ID has no chain identity (ADR-017), so a listener who bought a licence with one cannot use it. Design for that; do not discover it. |
| **DRC-369** | A release, a track, an album, a stem pack and a remix right are all DRC-369 items with content fingerprints and royalty policies. **M4, started and not blocked:** M2.1 and M2.3 were ticked on 22 September 2026, M4.1 (mint and content fingerprint) is done, and M4.2's royalties, remix royalties, settled sale and nesting are built. |
| **Qontrol** | The creator's project history, stems and masters. Large audio goes to the content-addressed side store, chunked and deduplicated, never into history as whole-file revisions. `.gitattributes` marks `*.wav`, `*.aiff`, `*.als` and `*.flp` as binary and unmergeable; `*.rpp` stays text, because it diffs and merges. |
| **Mesh** | The only storage and delivery rail. **M8.1, blocked on U-6.** Qontrol's remote is also the Mesh; Stream does not get a second one. |
| **CGT** | Every payment: licences, stem sales, hosting, seeder payment, and the listener's subscription if there is one. Prices display in the legible unit (ADR-007) with raw CGT behind an advanced view. |

## Substrate gaps

Recorded here, not filled locally. Each belongs to something that is not this product.

1. **Delivery receipts (U-6).** Stream must not build a play counter. It consumes whatever the Mesh's answer produces. See "The counting gap" above. **The one thing worth deciding now.**
2. **Cached, offline entitlement proof.** Playing a licensed track on a plane needs a cached, expiring attestation, and an attestation needs a signer. If QOR ID signs it, QOR ID's say-so now stands behind access to paid content, which ADR-017 and R-3 were written to keep out of value. Either the chain provides a verifiable ownership proof checkable against a cached finalised header, or offline playback is an explicit, recorded concession. **Named, not designed** — and it is the Library's gap first, not music's.
3. **Periodic netting and payout.** Nothing standard nets obligations over a period and pays once. Seeder payment (M8.1) needs the same shape, so **if it is built it is built once, as platform economics work, and never as a music pallet.** Whether it lives with royalties (M4.2) or with the economic mechanisms (M6) is a chain decision, not this document's.
4. **A stable price a buyer can read (U-2, U-3).** Whether "credits" is a formatting choice or a conversion is not stated, and a conversion needs a reference rate that does not exist and must not imply a peg (ADR-007). A catalogue makes this urgent in a way a wallet does not, because catalogues get re-priced.
5. **Catalogue indexing.** Search, playlists and shelves need history a Substrate node does not serve. That is ADR-028's indexer at M5.4, whose condition 1 is unmet. **Stream does not get its own index.**
6. **Any platform share, and any forfeited dust, needs a destination.** Treasury is **OPEN-2**; burn share is **OPEN-4**. Neither is available, so neither is assumed anywhere in this document.

**Not a gap, but a bill:** a cold swarm needs an origin. For a 40 GB game, waiting is acceptable and the fallback is the exception. **For a track nobody has played this week, the fallback is the normal case, and an origin server is a hosted CDN with an invoice.** A long-tail music catalogue is mostly cold. "The network gets faster as it grows rather than more expensive" is true of a popular game and false of a long tail, and whatever record proposes this product must say so in those words. The bill belongs in ADR-015's infrastructure map.

## The first usable slice

**Not listening. Licensing.**

A musician opens the launcher, points it at a finished track and its stems, and publishes. The release mints as DRC-369 items with a content fingerprint and a royalty policy naming the splits. A game developer finds it, reads the licence terms and the price, pays in CGT, and holds an entitlement they can prove. The payment splits across the policy's recipients in one settlement, and every recipient's share clears the existential deposit because a licence is a real payment, not a fraction of a cent.

That is worth opening the first time it works, and it is honestly described as **Market with an audio payload and a good royalty policy editor** — which is the point. It reuses L7.2 wholesale and needs no netting mechanism, no play counting, no swarm tuning and no answer to U-6.

The listening product is Phase 3 and it is a different product.

## Phases to a full product

**Phase 0 — now, and it is not code.** Get one sentence into whatever record governs the Mesh: *whatever U-6 decides must produce a delivery receipt, not only a storage proof.* Decided on purpose now, a streaming product stays possible. Discovered at M8, it does not. This phase costs one decision and is the entire value of this document today.

**Phase 1 — catalogue and licensing.** Audio as a DRC-369 payload; royalty policy authoring; the licence and stem-pack purchase path; Qontrol handling DAW projects and masters. Depends on M4. Ships as part of Market, not beside it.

**Phase 2 — delivery.** Opus encode in the publish path, decode in the launcher, per-album swarms on the Mesh with deadline-aware piece picking and origin fallback. Depends on M8.1 existing at all.

**Phase 3 — metered listening.** Netting, period close, one payment per creator per period, dust carry-forward. **Gated on U-6 having produced a delivery receipt.** This is where it becomes a streaming service and where it stops being a certainty.

**Phase 4 — discovery.** Indexer, catalogue search, playlists, shelf placement. Placement ordering is parameter-free and buildable — ranking N listings by committed CGT needs no absolute value anywhere — but its **disposition** (locked and returned, spent, or burned) is U-7, and the three options are three different products, not three implementations of one.

**Phase 5 — the remix economy.** Sample clearance by policy, remix chains that settle back along the chain, collaboration escrow (U-8). The part with no precedent, and correctly the last part.

## Dependencies on chain and launcher milestones

**Chain:**

- **M2.1** — the owner reads `MIGRATION_INVENTORY.md`. **Ticked on 22 September 2026**; nothing below is blocked behind it any longer.
- **M2.3** — the wire format decision. **Ticked the same day** (ADR-047).
- **M4.1** — DRC-369 ownership, mint, transfer, collections, owner enumeration, **content fingerprint**. Nothing here works without the fingerprint; there is nothing to verify a download against.
- **M4.2** — nesting, state and XP, **royalties with remix royalties settled in CGT**. The core of Phase 1.
- **M4.4** — sponsored fees and deposits, so a musician can publish without holding CGT first. Mechanics undecided (U-4, `SPONSORSHIP.md`).
- **M5.4** — the public viewer is the first thing to need an indexer (ADR-028), whose condition 1 is unmet: its type-generation tools were last published in 2024 and support for metadata versions 15 and 16 is unverified.
- **M6.4** — transaction payment. A `WeightToFee` is an **OPEN-4** value, which is why the runtime mounts no transaction-payment pallet today.
- **M6.5** — treasury, if any platform share exists. **OPEN-2**.
- **M7** — public testnet, external security review.
- **M8.1** — **the Mesh: seeder work verification (U-6) and seeder payment.** The gate.
- **M8.2** — entitlements from DRC-369.
- **M8.3** — staking for distribution (U-7), for playlist and shelf placement.
- **M8.4** — escrow and reputation bonds (U-8), for commissions and collaborations.

**Launcher:**

- **L4.1–L4.3** — minting from any file, owned assets from on-chain enumeration, amounts in the display unit. Phase 1 sits directly on these.
- **L6.1–L6.3** — signed installers and a verifying update channel. A listener is a consumer install, not a developer checkout.
- **L7.1** — Library install, delta patch and launch. The verify-before-write machinery is shared; a track is a small, cold instance of it.
- **L7.2** — Market. Phase 1 *is* this, with an audio payload.
- **L7.3** — Social (named Agora until 28 September 2026, with VYB merged into it), for anything social.
- **L7.4** — Mesh seeding. Phase 2 needs it.

**All of L7 depends on M8.** There is no path that shortens this.

**What exists today, and what does not.** The launcher's `Surface` type has eleven variants — `nexus`, `vault`, `inventory`, `market`, `projects`, `library`, `social`, `mesh`, `chain`, `gates`, `settings` (`src/state/store.ts`, checked 6 October 2026). `projects`, Qontrol's first surface, landed on 21 September 2026, and `market`, a screen that reads every listing from chain storage with no search and no indexer, on 2 October 2026; those two are the ones this product would build on directly. There is no installer crate (`tools/` holds the launcher and an MCP server), no `fs` plugin and no updater plugin, and the webview's capability grants it no filesystem access, no process spawning and no arbitrary hosts. And `systems.ts` advertises **Resonance** — "Release, stream and earn on sound" — which is frozen `apps/` copy with no roadmap item and no decision behind it. **None of this is a head start.**

## Feasibility

One founder and an agent. Honest, and the honest answer is mostly "not this".

**In three months — only the decision, and one small piece of borrowed work.** Phase 0 costs a sentence in the Mesh's record and is the only deliverable available. Little else in this document can start: M4 is no longer blocked (M2.1 was ticked on 22 September 2026), and the mint, sale and royalties Phase 1 would sit on exist, but Phase 1 is Market's work, M8 is not scheduled and U-6 is unanswered. The nearest real code is Qontrol's DAW-format handling — `.als` gunzip plus a structural XML diff is roughly two weeks for a useful first version — and that is Qontrol's item, not this product's, and it is worth doing whether or not this product ever exists.

**In twelve months — the licensing and stem shop.** Every hour of it sits in somebody else's milestone, and much of that has landed: `pallet-nfts` plus DRC-369 mint, transfer, fingerprint and owner index (M4.1, 22 September 2026), and the priced transfer with royalty splits (M4.2's royalty half, 29 September 2026). The indexer is about three months with a spike first and has not started. Market's first screen exists since 2 October 2026, reading listings straight from the chain. Twelve months is the realistic read for L7.1 and L7.2 shipping together, and that read assumes no slippage anywhere. The audio-specific part on top — Opus in the publish path, and a royalty policy editor a musician can actually use — is weeks, because it is a payload and an editor, not a new system.

**Beyond twelve months — the listening product, if U-6 allows it at all.** It needs the Mesh to exist and to pay a seeder for verified work on a real network; it needs streaming-aware piece picking, about three months once there is a Mesh to pick pieces from; it needs netting and payout, about three months after M4.2 and only once there is a play count worth netting. Its blockers are M4, M8.1, U-6, U-7, OPEN-1, OPEN-2 and OPEN-4. **Not estimable** is the correct answer for the whole of it, and it stays the correct answer for play counting specifically until the owner answers U-6.

**What this means in one line.** Nothing about this product should be built, and nothing about it should be promised, until the Mesh pays a seeder for verified work on a real network.

## What "full support" means, and what it does not

**Means:** a musician can publish a release, have it stored and served by the network, license it, sell stems, be paid in CGT under an applied split, and see a verifiable chain of provenance through every remix of it.

**Does not mean — and this half matters more:**

- **Not a Spotify replacement.** No catalogue of major-label music will be here. Anyone comparing the two is comparing a shop to a library.
- **Not per-play on-chain payment, ever.** The existential deposit forecloses it. Any copy implying "every play is a transaction on the blockchain" is false and must not be written.
- **Not a play-count oracle.** Stream counts nothing on its own authority. If U-6 produces no delivery receipt, the listening half does not exist; it is not replaced by something Stream signs.
- **Not free distribution.** Cold long-tail catalogue is served from an origin with a bill. Peer-to-peer saves money on the popular fraction only.
- **Not lossless by default.** A content-addressed Mesh fingerprints bytes; re-encoding a catalogue later changes every fingerprint for every re-encoded work. Ship the lossless path from day one or accept never having one cleanly.
- **Not label back-office.** No ISRC or ISWC registration, no PRO or mechanical-rights registration, no territorial windowing, no neighbouring-rights collection, no publisher administration. Each is a specialist system with statutory obligations, and none is in scope.
- **Not rights clearance.** The chain applies a policy someone declared. It does not verify that the declarer had the right to declare it. An uncleared sample settles just as smoothly as a cleared one.
- **No takedown machinery is designed.** Who can remove what, on whose say-so, from a peer-to-peer network, is undecided, and it is a decision before launch rather than after the first notice.
- **Not offline listening**, until the cached-entitlement gap has a decided answer.
- **Not a second rail of anything.** No second identity, no second asset format, no second storage network, no second payment path, no second index. If this product turns out to need one, that is a substrate gap to record, not to fill here.
- **Not priced in dollars.** Prices are CGT, displayed in the legible unit, with no peg implied (ADR-007) and the conversion question open (U-2, U-3).
- **No statement about what anything will be worth.** ADR-008 governs every word of this product's copy, and a storefront is exactly where such language creeps in.

## The spend test, argued

ADR-002: *if CGT could never be traded for dollars, would it still be worth holding?* A mechanic that passes creates a reason to need CGT.

**Where this passes cleanly.**

- **Access gating (ADR-006, sink 2).** Licences, unlocks, remix rights and entitlements settle only in CGT. A supervisor who wants that track for a film needs CGT at the moment of the transaction and no other currency will do. Each is a one-off payment at a scale that clears the existential deposit comfortably. **This passes as plainly as any mechanic in the project.**
- **Mesh hosting (sink 1).** A creator pays to have a catalogue stored and served; seeders are paid for the storage and bandwidth the network actually consumed (ADR-005). Passes by construction.
- **Staking for distribution (sink 3).** A playlist or a front shelf is the same scarce-visibility resource as Market's, with the same U-7 question about disposition.
- **Escrow and reputation bonds (sink 5).** A producer commits CGT against delivering stems. Passes; blocked on U-8.

**Where it does not pass, and this is the half that decides the product.**

Per-play listening — the thing that makes it a *streaming service* rather than a shop — is the weakest fit for the spend test of anything examined, for a reason that has nothing to do with fees.

- **A listener does not want to spend anything per play.** The entire consumer market is subscription. A subscription is one payment per period, which is fine under the spend test — but then **the per-play layer is distribution of a pot, not demand for CGT.** Distributing a pot is an accounting operation. It creates no new reason for anyone to need CGT.
- So the demand is created by **the subscription**, and the plays only decide who receives. That is a defensible design and it must be described that way. "Per-play royalties" is the **payout mechanism**; "the subscription" is the **sink**. Conflating them produces a document claiming a demand sink it does not have.
- **If the pot is funded by issuance rather than by listeners, the mechanic fails the spend test outright.** It would pay creators from new supply, the exact pattern ADR-002 was written against: payouts that are attractive while new holders keep arriving and stop when they do not. Issuance is **OPEN-1**, the genesis split is **OPEN-2**, and the fee class and burn share a subscription would touch are **OPEN-4**. None of the three is decided, and none may be quietly assumed into a music subsidy. No subscription price appears in this document for the same reason.

**Verdict.** This passes the spend test **as a licensing and hosting surface**, and **only conditionally** as a listening surface. The condition: **listener payment is real, in CGT, and funds the pot, with plays deciding distribution rather than creating demand.** If this product is ever proposed formally, that sentence is the first line of its ADR.

## Risks

1. **U-6 is answered without a delivery receipt.** Then there is no trustworthy play count, and the listening product cannot be built on this substrate at all. **Highest-impact risk on the page, and the only one that is cheap to prevent today.**
2. **The long tail is cold and the origin bill is the real cost structure.** Mitigation: per-album or per-shard swarms, honest cost modelling in ADR-015's infrastructure map, and never describing peer-to-peer as free.
3. **Play-count fraud.** A seeder is paid more when there are more plays, and a listener's machine reports the plays. Whatever U-6 produces must be unforgeable by both parties, or the largest demand sink pays for fabricated claims.
4. **Royalty arithmetic on remix chains.** A chain of N remixes each taking a fraction of a fraction is exactly where somebody writes `a * b / c`. This project has already shipped that failure once. Mitigation: ADR-035's helpers, and a test pinning the largest intermediate every money path forms (AGENTS.md §5).
5. **Dust.** Many creators owed less than the existential deposit at period close. Carry-forward is the only rule available without a decided destination, and it needs a bound or it becomes an unbounded map.
6. **The codec argument.** 96 kbps Opus is transparent *to most listeners*, and a creator-facing platform attracts precisely the people who will argue about it. Mitigation: a higher-rate tier and a lossless archive path from the start, because retro-fitting a second encoding across a content-addressed Mesh changes every fingerprint.
7. **Rights disputes with no adjudicator.** U-8 is undecided, and music has more of these per capita than any other medium.
8. **Language drift.** A music storefront is the single most likely place in this project for ADR-008 violations to appear, because the entire surrounding industry speaks in them. Mitigation: the copy review is a gate, not a courtesy.
9. **Scope drift by sympathy.** This product is easy to want and the furthest from possible. Under `docs/DIRECTION.md`'s rule that anything proposed later must justify itself against one of the four primitives, **this contributes no primitive of its own** — its distribution is the Mesh's, its settlement is the royalty pallet's, its identity is QOR ID's, its counting is U-6's. That is an argument for building it last, and an argument against writing a roadmap item for it now. The owner's instruction gave it a track anyway, P6, which is honest only because its first item is the U-6 decision and nothing after it can start before M4 and M8.

## Decided by the owner, 22 September 2026

Each answer is the owner's and is reversible.

1. **The name is the owner's to give later.** "Stream" stays a placeholder.
2. **Per-play payment on chain is ruled out for now, not permanently.**
3. **Deferred to M6, not decided: P6.1**, whether U-6's answer must produce a delivery receipt.
4. **It is a listening service first.** This reverses this blueprint's recommendation of a licensing shop first.
   The track keeps dependency order — licensing needs only M4, listening needs M8 — so the owner's priority is
   a statement of what the product is, not a change of sequence the dependencies would allow.
5. **Deferred to M6, not decided:** what funds a subscription pot, and how small balances are handled.
6. **Creators are paid automatically**, with the mechanism designed in M6 — push rather than claim, in this
   blueprint's terms.
7. **Deferred to M6** with 5: what happens to a balance below the existential deposit.
8. **Lossless masters are stored; lossy tiers are streamed.**
9. **The creator can always take their work down; the platform only on legal notice**, and that notice is
   recorded.
10. **A listener needs no chain key; payments do.** Listening works with a QOR ID alone, and anything that moves
    CGT needs a proven key (ADR-017).
