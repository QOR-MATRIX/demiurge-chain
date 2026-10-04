# ADR-071: ARQADE as a self-publishing store: project profiles, stages, backing, prices, in-game items and agents

**Status:** **Proposed**, 4 October 2026. **Decision 1 is the owner's direction**, given that day and recorded rather
than proposed, **including the price ceiling of 10,000 CGT**, which is the owner's own number. **Decisions 2 to 9 are
the assistant's recommendations** and are not accepted. The developer-facing descriptions are
[`products/arqade/sdk/docs/publishing.md`](../../products/arqade/sdk/docs/publishing.md),
[`backing.md`](../../products/arqade/sdk/docs/backing.md) and
[`building-with-agents.md`](../../products/arqade/sdk/docs/building-with-agents.md); the steps are P7.12 to P7.15 in
[`../DIRECTION.md`](../DIRECTION.md).

**Relates to:** [ADR-069](ADR-069-arqade-the-gaming-platform.md) (ARQADE), [ADR-070](ADR-070-game-vaults.md) (Game
Vaults and the Cartridge, Proposed); [ADR-010](ADR-010-agent-rails.md), [ADR-014](ADR-014-agent-keys-authorised-not-created.md)
and [ADR-026](ADR-026-agent-delegation-with-pallet-proxy.md) (agents); [ADR-011](ADR-011-web-surface.md) and
[ADR-050](ADR-050-where-the-products-live.md) decision 11 (what a product may not carry);
[ADR-031](ADR-031-no-fungible-game-items-in-the-first-release.md) (no fungible items); [ADR-002](ADR-002-value-from-spending.md),
[ADR-006](ADR-006-demand-sinks.md), [ADR-008](ADR-008-language-discipline.md).

## Context

On 4 October 2026 the owner asked for ARQADE to work like a futuristic app store for developers:

- developers **self-publish** once requirements are met, among them a **completed project profile**, from a template,
  which becomes the game's **store page**;
- an easy **onboarding** that accepts a project **at any stage**, so an idea can build interest before it is built;
- **CGT support from backers**, as on Kickstarter and Patreon;
- a **price per game of up to 10,000 CGT**;
- **in-game purchases as DRC-369 assets**;
- creation as **frictionless and agentic** as possible, with games "vibe coded" through an **external LLM**.

What stands in the way, read against the tree the same day:

- **Bytes have nowhere to live.** A store page needs images, video and builds. The Mesh is M8; the only store is the
  launcher's temporary one on one machine (G-3). ADR-050 decision 11 forbids a product its own storage.
- **There is no primary sale.** `Drc369Royalties::buy` sells an asset that already exists and is listed; nothing mints
  a copy for each buyer, so a game or an item cannot be sold to many people without the developer minting each copy
  first.
- **There is no escrow, no schedule and no recurring payment.** All-or-nothing backing needs funds held until a goal
  and a deadline (U-8 is open); a monthly membership needs either a signature each month or delegated spend caps
  (M5.2, unbuilt).
- **Agents are decided and unbuilt.** ADR-010 chose one MCP server for every LLM and scoped, revocable agent keys
  with caps enforced on chain (M5.2, M5.3).
- **Backing is the riskiest copy Demiurge could write.** Money given to a project in exchange for a share of what it
  earns is a regulated security in most jurisdictions, and ADR-008 bans the language that would describe it.

## Decision

1. **The owner's direction (recorded).** Developers self-publish to ARQADE once a project meets its stage's
   requirements, the completed project profile among them, and the profile is the game's store page. Projects may
   enter at any stage. Backers support projects in CGT. **A game's price is at most 10,000 CGT** (free is allowed).
   In-game purchases are DRC-369 assets. Creation is agentic, and an external LLM can build and submit a game.

2. **The store page is the Cartridge.** A project is minted as its DRC-369 Cartridge (ADR-070) when it is first
   submitted, even as an idea: the content is the project's **profile manifest** (the template in
   `products/arqade/sdk/templates/`), revisable until the developer makes a release permanent. Every revision is a
   Qontrol commit, so **the devlog is the Cartridge's history**, public and in order, and nobody can rewrite it
   quietly. One identity carries the page, the builds, the Game Vault and the sales.

3. **Four stages, each with requirements a machine checks.** **Concept** (name, pitch, genre, art direction, the
   creator's QOR ID), **Prototype** (a playable build, its hash, controls, a content rating), **Early access**
   (rule versions, a price or free, a support contact, sandbox checks passed) and **Released** (a permanent build, all of
   the above, and every declared in-game item minted or mintable). The SDK's `checkReadiness` computes what is
   missing; ARQADE shows it as the page's launch checklist. Meeting a stage's requirements publishes it — **no human
   gate** — except where the risk is someone else's money or safety: Game Vault payouts, backing campaigns over the
   owner's threshold, and anything a report flags. Those go to review.

4. **Pricing.** The price is set in CGT at most 10,000 CGT, checked by the SDK and by ARQADE's listing rules, and shown
   in CGT (ADR-007's display unit when it exists). Buying a game gives the buyer a **licence asset** in DRC-369, and
   ARQADE launches a paid game for whoever holds one (P5.4's ownership gate). Whether a licence can be resold is the
   owner's choice: transferable licences resell with the chain's royalties; account-bound ones wait on G-16. Paying
   for a game is **access gating** (ADR-006).

5. **In-game purchases are non-fungible DRC-369 items**, declared in the profile's item catalogue with a price each.
   ADR-031 keeps fungible items out: no coins, gems or stacks of consumables sold as tokens. A purchase gives the
   buyer an item they own and can resell under the chain's royalties. Selling many copies needs a **primary sale**
   primitive (mint on purchase, with an optional edition cap), recorded as substrate gap **G-17** and decided by its own
   record; until then a developer mints items ahead and lists each one.

6. **Backing, never a stake.**
   - **What a backer gets is the work, never its proceeds**: early access, a backer badge, a credit, items — each a
     DRC-369 asset granted when the backing settles. **No share of revenue, no promise of a return, no buy-back**, in
     copy, code or contract. A backer is a supporter, never a shareholder (ADR-008).
   - **Campaigns** (Kickstarter's shape) are all-or-nothing: pledges are held in a keyless **Campaign Vault**, the same
     construction as ADR-070, until a goal and a deadline. Met, the funds go to the developer; missed, every backer can
     take their pledge back. **Milestone release** is recommended: funds unlock in parts as the Cartridge gains the
     builds the campaign promised, so backers' CGT follows delivery.
   - **Memberships** (Patreon's shape) are one payment per period, approved by the backer each time, until M5.2's
     delegated caps let a backer approve a recurring amount once and revoke it at will.
   - **Tips** are a plain transfer to the developer, at any stage.
   - All of it runs on the devnet first. **In production it waits on a legal review** — crowdfunding and donations
     are regulated in many places — and on U-17.

7. **Agentic creation is ADR-010's MCP server with ARQADE's tools.** An external LLM connects through the one MCP server
   ADR-010 decided, under an agent key the developer authorised in QOR ID (ADR-014), never their Vault key. Its tools:
   scaffold a game from a template, write and commit to the project, run the game's tests and a devnet rehearsal,
   update the profile, and check readiness. It can prepare everything; **what is irreversible or public — submitting a
   stage, setting a price, opening a campaign, funding a Game Vault — is signed by the developer** in their own Vault,
   or by an agent key within caps the chain enforces once M5.2 exists. If ARQADE ever runs the model itself and charges
   for it, that is the agent-compute sink and U-9; a developer's own LLM costs ARQADE nothing.

8. **Generated code is untrusted, whoever wrote it.** Every build, human or agent, runs in ARQADE's isolated origin
   with a content security policy allowing only the network origins its manifest declares, no access to the parent
   page, its session or any wallet, and size and frame-time limits checked before a stage is published. A build is
   identified by its hash; what plays is what was checked.

9. **Storage is the ecosystem's gap, not ARQADE's.** Store-page media and builds need a store before the Mesh exists. The
   recommendation is **one interim content store for the whole ecosystem**, addressed by ADR-047's fingerprints and
   operated beside QOR ID, replaced by the Mesh at M8 — decided by its own record, because ADR-050 decision 11 forbids
   ARQADE a store of its own. Until then, a store page carries text and a profile manifest, and media links point to
   wherever the developer hosts them, shown as external.

## Consequences

- **New substrate gap G-17** (primary sale) in `blueprints/ECOSYSTEM.md`, and **G-3 is now hit by ARQADE** too.
- **New open question U-17**, backing: the review threshold, refunds after a missed milestone, how long a pledge may be
  held, and any platform share (which joins U-15).
- **The price ceiling is a product rule, not a chain constant.** It is enforced where a game is listed on ARQADE. If
  the owner wants it on chain, it is one check in the primary-sale record.
- **What can be built before any of this is accepted**: the profile template, its schema and `checkReadiness`, and the
  price check — none needs the chain. They are in the SDK as of this record.
- **The legal list grows**: paid chance (ADR-069), and now backing. Neither runs in production before review.

## What this record does not decide

- Whether game licences are transferable.
- The review threshold for campaigns, refund terms, pledge holding periods, or any platform share (U-17, U-15).
- The interim content store, the primary-sale primitive, and every pallet, package and tool name.
