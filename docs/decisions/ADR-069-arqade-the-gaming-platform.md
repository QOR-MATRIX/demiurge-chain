# ADR-069: ARQADE, the gaming platform: a product under ADR-050, a third web surface, and the decisions it needs

**Status:** **Accepted**, 4 October 2026, by the project owner, who wrote "I accept ADR-069 and ADR-043 as written".
Proposed the same day. Decision 1 is the owner's direction; decisions 2 to 9 were the assistant's recommendations and
are accepted as written. **The three further choices listed after decision 9 (design, hosting, name) are not decided
by this acceptance**: the record says so itself. **Amends:** [ADR-011](ADR-011-web-surface.md) (decision 3). **Accepted
with:** [ADR-043](ADR-043-qor-id-as-an-identity-provider.md). The design is [`../blueprints/arqade.md`](../blueprints/arqade.md); the steps are
P7 in [`../DIRECTION.md`](../DIRECTION.md).

**Hosting settled, 5 October 2026:** the owner chose Vercel; [ADR-074](ADR-074-arqade-on-vercel.md) records it. The
hosting option below is kept as it was written.

**Relates to:** [ADR-011](ADR-011-web-surface.md) (two web surfaces), which decision 3 amends;
[ADR-043](ADR-043-qor-id-as-an-identity-provider.md) (browser sign-in), accepted with this record, which decision 4 needs;
[ADR-046](ADR-046-the-launcher-is-an-app-host.md) and [ADR-010](ADR-010-agent-rails.md) /
[ADR-026](ADR-026-agent-delegation-with-pallet-proxy.md), the two signing paths decision 5 chooses between;
[ADR-050](ADR-050-where-the-products-live.md), whose form decision 2 follows; [ADR-002](ADR-002-value-from-spending.md),
[ADR-006](ADR-006-demand-sinks.md), [ADR-008](ADR-008-language-discipline.md) and
[ADR-031](ADR-031-no-fungible-game-items-in-the-first-release.md), which bound decision 7.

## Context

On 4 October 2026 the owner named **ARQADE** as the ecosystem's future gaming platform and handed over a master
implementation prompt for it. The starting point is an existing website, "DEMIURGE · The Sovereign Arcade", built
outside this repository. Read on 4 October 2026 (files only):

- **Where it runs.** OpenAI Sites (`demiurge-arcade.quick-crab-0610.chatgpt.site`), built with vinext for
  Cloudflare Workers, with Cloudflare D1 (SQLite) through drizzle. Identity is the Sites host's
  `oai-authenticated-user-*` headers, turned into an `Explorer-xxxxxxxx` alias by hashing the user id
  (`lib/arena-store.ts:6`).
- **What it does.** Four solo games (Void Runner, Synapse, Orbital, Rift Survivor) and two server-authoritative
  multiplayer arenas (Flux Four, Rift Reversi) with revision-guarded writes, polling (2 s lobby, 1.2 s match), chat,
  a template "creator" and an in-browser terminal.
- **What is not real.** Solo scores, "Energy" and tokens live in `localStorage` (`app/page.tsx:15-21`); a finished
  game grants 0 to 3 tokens from browser randomness. A `qor_id` column exists (`db/schema.ts:8`) and nothing writes
  it. `app/api/chain/route.ts:4` calls `chain_getBlockNumber` at `https://rpc.demiurge.cloud` — the deleted custom
  chain's vocabulary at an address that never served this chain. The storefront shows **USD prices** for Energy
  packs ($5, $20, $60, `app/systems.tsx:11`, marked not connected) and names **CRGT** and **"Energy Tokens → DMRG"**
  (`systems.tsx:10-13`). It has no `test` script; `node --test tests/arena.test.mjs` is run by hand.

Read against this repository the same day, the handoff's model of the ecosystem is mostly right — one chain,
`chain/`; CGT at 18 decimals with a 100 CGT existential deposit; no fees, no issuance, no treasury; DRC-369 without
state or XP — and wrong or silent on the following, each of which changes what can be built:

1. **QOR ID has no browser sign-in.** No `/authorize`, no PKCE, no client registry (ADR-043, Proposed). Tokens are
   HS256 JWTs signed with a **shared secret** (`services/qor-auth/src/services/session_service.rs:104`), carried as
   Bearer headers, and **no endpoint verifies or introspects them** for another service. A server outside QOR ID can
   only check a token by holding QOR ID's signing secret, which must never happen. CORS is an allowlist
   (`main.rs:201-230`), so ARQADE's origin is refused today.
2. **Nothing connects a browser to the launcher's vault.** No deep link, no custom scheme, no localhost listener
   (`tools/qor-launcher/src-tauri/Cargo.toml:22-27`). ADR-046's app-host protocol exists only on paper; there is no
   `products/` or `platform/`. ADR-011 already says how a web surface signs: **with delegated agent keys** (ADR-010,
   ADR-026), whose spend caps are M5.2 and unbuilt. `pallet-proxy` is not in the runtime.
3. **ADR-011 allows two web surfaces**, a public viewer and a thin console, "never merged", and keeps creation and
   publishing in the launcher. A game platform with a developer release pipeline is neither.
4. **The design system rules ARQADE's look out.** `docs/design/DESIGN_SYSTEM.md:15-21` forbids, *on every surface*,
   glows, particle fields and animated backgrounds, neon and cyberpunk styling, and decorative looping animation.
   The handoff asks to preserve a cinematic portal and a terminal aesthetic.
5. **The chain lacks three primitives the handoff treats as product details.** There is **no randomness source**
   (Aura has no VRF; no randomness pallet is mounted), so a pack's allocation cannot be verifiable on chain. There is
   **no account-bound asset**: the runtime's call filter lets `Nfts::transfer` through and nothing locks an item's
   transfer. And **editions are not enforced**: `Drc369::mint` mints to the signer, into the signer's one singles
   collection, with no supply cap a creator can set.
6. **Economic limits.** Mechanics that touch CGT must pass the spend test (ADR-002) and name their demand sink
   (ADR-006). A paid entry is access gating. A prize paid to a player is neither payment for work nor licensing, so
   ADR-008's language applies to it with force, and **who funds prizes in production has no answer**: no treasury
   (OPEN-2), no issuance (OPEN-1), and no CGT may be created outside `--dev` (AGENTS.md §5). Paid chance with a
   transferable prize is also a legal question before it is an engineering one. ADR-031 keeps fungible game items
   out of the first release, which rules out "Energy" as a token.
7. **Hosting.** The site runs on a third-party host outside ADR-042's two domains and outside Railway (ADR-063,
   ADR-068), with its source outside this repository.

## Decision

1. **The owner's direction (recorded, not proposed).** ARQADE becomes the ecosystem's gaming platform: games
   played with a QOR ID, **every owned collectible a DRC-369 asset** and **CGT the only currency** for costs and
   payouts, a player's own Vault account as the custody and signing authority, and a developer devnet and SDK so
   that independent creators can publish games on it. The existing arcade is the starting point and its experience
   is preserved.

2. **ARQADE is a product under ADR-050**, with product track **P7** in `DIRECTION.md` and a gate `arqade` in
   `GATES.toml`. When it enters active scope (ADR-050 decision 10) its source lives in **`products/arqade/`**, imported
   from the arcade checkout. **It is TypeScript, not a Cargo workspace**, so its CI job is the TypeScript equivalent of
   ADR-050's: type check, lint, `node --test`, and a production build. That is the one departure from ADR-050's form.
   No frozen path (`apps/games` included) is reused.

3. **ARQADE is a third web surface, amending ADR-011.** It is not the public viewer and not the console, and it does
   not absorb either. It reads assets and provenance from the same sources the viewer will (M5.4); it links an
   asset's detail to the viewer once the viewer exists rather than growing a second asset inspector. **Creation and
   publishing of DRC-369 assets stay in the launcher** (ADR-011): ARQADE mints only through the signing path of
   decision 5, never with a key of its own for a player.

4. **Sign-in is ADR-043, and ARQADE is what makes it needed.** The owner is asked to accept ADR-043. Until it is
   built ARQADE has **no verified QOR identity**: the Sites alias stays a practice identity, labelled as one, and no
   ranking, payout or asset award is bound to it. ARQADE's servers never hold QOR ID's signing secret. Because
   ADR-043 decides how a first-party frontend gets tokens but not how a separate server checks them, its build adds
   a way for a registered client to verify a token (introspection, or asymmetric signing with a published key). A
   third-party game signing players in is ADR-043's excluded case and needs its own record.

5. **Signing: delegated keys are the destination; the bridge needs its own record.** Three ways a browser game can
   get a Vault signature:
   - **A. A request handed to the launcher.** ARQADE presents a signing request; the launcher shows it in its host
     dialog (L1.4) and signs in the vault. Needs a browser-to-launcher transport (a custom URL scheme is the narrowest;
     a localhost listener is refused, because any web page can reach one). New, security-critical code.
   - **B. Delegated session keys with protocol-enforced caps** (ADR-010, ADR-026, M5.2). What ADR-011 already says web
     surfaces use. Needs `pallet-proxy`, `pallet-agent-caps` and M5.2: unbuilt.
   - **C. ARQADE as a launcher-hosted product** (ADR-046). Signs over the host protocol; does not reach a browser.

   **Recommendation: B for production, A for devnet play before M5.2, each in its own record before code.** C is
   right for games built with QOR Engine (P3) and does not serve the web. Until one exists, ARQADE reads and does not
   sign, and every chain action a player takes happens in the launcher's own screens.

6. **The devnet is Demiurge Devnet, and there is one faucet.** ARQADE's developer environment is the existing devnet
   (`wss://rpc.qorsync.dev`, genesis `0x934e2caa36fba548ee5f51195c2d029097fbba0400e6e805ca8f3e07947a254a`, ADR-068) and
   a local node run as `scripts/run-local-stack.md` describes. No second chain. Test CGT for developers comes from the
   ecosystem's faucet page (`OWNER.md`'s next item 3), not from an ARQADE-run faucet, and never from a key embedded in
   ARQADE.

7. **Economics: test values on the devnet, nothing in production until decided.**
   - A paid entry, a pack and a card sale are **access gating** (ADR-006, sink 2). A card sale settles through
     `Drc369Royalties::buy_exact`, so royalties are the chain's, not ARQADE's.
   - Payouts **move existing CGT** from a funded account under recorded caps; nothing creates CGT. **Who funds them in
     production, and whether paid entry may lead to a CGT or asset prize at all, is U-16**, opened by this record.
   - **Paid chance-based prizes stay off** until U-16 is decided and the owner has a legal review; free practice and
     unpaid skill play are not affected.
   - **"Energy" is retired as a currency**: no Energy packs, no USD prices, no CRGT, no DMRG, no conversion. If a
     prepaid session entitlement is ever wanted it is an off-chain record of a finalised CGT payment, never a token
     (ADR-031), and never called Energy, a name that belonged to the deleted chain's fee model (D-001).
   - Copy follows ADR-008: no cash value, no "earn" framing for play, no statement about what CGT is worth.

8. **Chain gaps are chain work, on the chain's roadmap, not ARQADE's.** Evolving assets are **M4.2's state and XP**;
   ARQADE does not keep evolution in its database and call it on-chain. The three new gaps — **randomness**,
   **account-bound assets**, **creator-set collections with enforced editions** — are recorded as substrate gaps G-14
   to G-16 in `blueprints/ECOSYSTEM.md`, each to be decided by its own ADR, and any new pallet is named by the owner
   (AGENTS.md §8). Until then: trophies are minted by an issuer account and transferred (one `batch_all`), and are
   labelled transferable; packs are not sold for CGT; cards declare their set in their manifest, and nothing claims an
   edition cap the chain does not enforce.

9. **The developer SDK sits on M5.1 and is published after the wire-format freeze.** ARQADE's game SDK (sessions,
   outcomes, leaderboards, manifests) is a layer over the ecosystem SDK of M5.1, not a second chain client. Its home
   is `products/arqade/` and the chain-facing half goes to `platform/` under ADR-050 decision 2. Its package names are
   the owner's (AGENTS.md §8). It is not published before `beta.wire-format-frozen` (ADR-009). **Third-party games are
   listed through Market's one listing model** (P5.3, kind "game") rather than a second catalogue, and run in an
   isolated origin with a validated message bridge.

**The owner's further choices, not proposed here, and blocking nothing before P7.3:**
- **Design.** Either ARQADE gets a design record of its own that admits its cinematic look on its own surface, as
  ADR-051 admitted QFX's canvas, or it is restyled to `DESIGN_SYSTEM.md`. Recommendation: its own record, with the
  launcher's rules on reduced motion, contrast and focus kept.
- **Hosting.** Stay on OpenAI Sites as a prototype host, or move to `demiurge.cloud` (ADR-042) on Railway or
  Cloudflare. Recommendation: keep Sites until P7.3; serve production from `demiurge.cloud`, where QOR ID's
  allowlist and ADR-043's redirect URIs can name it.
- **Name.** ARQADE is a public name; it is cleared as `public-release.name-clearance` clears the others before a
  public launch.

## Consequences

- **The handoff is corrected, not followed literally.** `blueprints/arqade.md` is its corrected form and is what an
  implementing agent reads. The original's mandate stands; its assumptions about sign-in, signing, randomness, editions,
  hosting, faucet and design are replaced by this record's.
- **What can start today, with nothing accepted:** P7.2, live read-only chain reads from the existing site (genesis,
  finalised head, a balance and an account's DRC-369 inventory, in integer Sparks), and the removal of the site's
  false claims (CRGT, DMRG, USD prices, `chain_getBlockNumber`). Neither needs sign-in, signing or an economic value.
- **What this costs the chain's weeks.** ARQADE's later steps wait on M4.2, M5.1, M5.2, M5.4 and three new gap
  records; it adds demand on all of them and builds none. Its gate counts them (ADR-050 decision 8).
- **If decision 3 is rejected**, ARQADE can only be a launcher-hosted product (5C), and the website stays a
  practice arcade without chain value.
- **If decision 5 is rejected in favour of a browser-held key**, an XSS on ARQADE's origin spends a player's CGT.
  That is the outcome ADR-011 was written to prevent, and this record does not offer it.

## What this record does not decide

- Any economic value: entry prices, prize sizes, caps, a platform share (U-15) or prize funding (U-16).
- ADR-043's token lifetimes, the bridge of option A, or the three gap records.
- Whether ARQADE's arenas move to WebSockets; that is a product choice inside P7, made with a measurement.
- Any pallet or package name.
