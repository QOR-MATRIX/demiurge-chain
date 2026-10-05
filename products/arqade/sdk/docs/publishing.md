# Publishing your game on ARQADE

**For game developers.** How a project goes from an idea to a game people buy and play, publishing itself at each
stage once it meets that stage's requirements. No approval queue, except where someone else's money or safety is at
stake.

> **Status: partly built.** The profile template, its checks (`checkReadiness`, `readyStage`) and the price check
> (`priceProblem`) exist in this SDK and are tested. Minting the Cartridge, store pages, listing and sales are
> **proposed** in [ADR-071](../../../../docs/decisions/ADR-071-arqade-self-publishing.md), accepted on 4 October 2026, and not built. Names are
> placeholders. On Demiurge Devnet, CGT is test CGT.

---

## The shape of it

1. **Start a project** from the template, [`../templates/arqade-project.json`](../templates/arqade-project.json), by
   hand or with your LLM ([building-with-agents.md](building-with-agents.md)).
2. **Submit it at any stage.** Your project becomes a DRC-369 asset, its **Cartridge**, whose content is your profile.
   ARQADE renders the profile as your **store page**.
3. **Every update is a revision** of the Cartridge and a Qontrol commit. Your **devlog is that history**: public, in
   order, and impossible to rewrite quietly.
4. **The page's launch checklist** shows what the next stage still needs. Meet it and the stage publishes.
5. **People can back you from the first day** ([backing.md](backing.md)), and buy the game when you set a price.

## The four stages

| Stage | What people see | What it needs (on top of every earlier stage) |
| --- | --- | --- |
| **Concept** | A store page: name, pitch, description, genres, art direction, you. Followers and backers can join | Name (2–60 characters), slug, pitch (10–140), description (80+), one to three genres, your QOR ID, art direction |
| **Prototype** | A playable build, free to try | A build with its BLAKE3-256 hash, entry file, size and controls; a content rating; a cover image with alt text |
| **Early access** | The game, at your price or free, with your rule versions | Rule versions (`name@number`); a price or free; a support page; **sandbox checks passed for this exact build**; three screenshots |
| **Released** | The finished game and its item store | A **permanent** build; every in-game item with a unique id, a name, a description and a price |

Check where you stand:

```ts
import { checkReadiness, readyStage } from 'arqade-sdk'; // placeholder name

readyStage(profile);                    // 'prototype'
checkReadiness(profile, 'early-access'); // [{ stage: 'early-access', missing: 'a support page (https)' }, …]
```

**What goes to a person, not a machine:** turning on ARQ Wallet payouts ([arq-wallet.md](arq-wallet.md)), a backing
campaign above the platform's review threshold, and anything a player reports.

## Prices

- **Free, or a fixed price up to 10,000 CGT.** The ceiling is the project owner's rule. Prices are whole Sparks
  (`1 CGT = 10^18 Sparks`); `priceProblem` refuses anything above the ceiling, a fixed price of zero ("use free"), and
  any amount with a fraction of a Spark.
- **Buying gives the player a licence**, a DRC-369 asset in their own Vault. ARQADE launches a paid game for whoever
  holds one. Whether a licence can be resold is not decided yet.
- Paying for a game is a payment for your work. ARQADE's copy never presents CGT as something that gains value.

## In-game purchases

Items are **DRC-369 assets the player owns**: skins, ships, cards, equipment. Each is declared in your profile's
`items`, with a price in Sparks and an optional edition cap. A player can resell an item, and your royalty terms apply
to the resale.

- **No fungible items**: no coins, gems or stacks of consumables sold as tokens (ADR-031). Sell the thing itself.
- **Until the chain can mint on purchase** (gap G-17), each copy must be minted ahead and listed. When the primary
  sale exists, a purchase mints the buyer's copy and the edition cap is enforced on chain.

## What every build must pass

Whoever wrote it — you or your LLM — a build runs in ARQADE's isolated origin: no access to the page around it, the
player's session or any wallet; network only to the origins your manifest declares; size and frame-time limits. The
checks run against the build's hash, and what plays is exactly what was checked.

## Storage, for now

Until the ecosystem has a content store (gap G-3, M8), a store page carries its text and profile, and images link to
`https://` addresses you host, shown as external. Builds wait for the interim store ADR-071 recommends.
