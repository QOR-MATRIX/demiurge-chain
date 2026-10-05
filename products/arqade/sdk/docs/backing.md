# Backing: support from players, from the first idea

**For game developers.** How people support your project in CGT before and after it ships: campaigns (the
Kickstarter shape), memberships (the Patreon shape) and tips.

> **Status: designed, not built** ([ADR-071](../../../../docs/decisions/ADR-071-arqade-self-publishing.md) decision 6,
> accepted on 4 October 2026). It runs on Demiurge Devnet with test CGT first. **In production it waits on a legal review and on U-17**
> (review threshold, refunds, holding periods). Names are placeholders.

---

## The one rule

**Backers get the work, never its proceeds.** Early access, a backer badge, a credit, an item — each a DRC-369 asset in
the backer's own Vault. **Never** a share of sales, a promise of anything back, or a buy-back. A backer is a supporter.
ARQADE refuses a campaign whose rewards or wording say otherwise, and the SDK has no field that could express a revenue
share.

## Campaigns: all or nothing, released as you deliver

- You set a **goal** and a **deadline**, and **reward tiers** (each a DRC-369 asset granted when the campaign succeeds).
- Pledges go into a **Campaign Vault**: like an ARQ Wallet ([arq-wallet.md](arq-wallet.md)), an account with no key,
  tied to your Cartridge.
- **Goal met by the deadline**: funds become yours. **Missed**: every backer can take their pledge back, at any time,
  without asking anyone.
- **Milestones** (recommended): split the funds into parts, each released when your Cartridge gains the build the
  campaign promised for it. Backers watch their CGT follow your delivery.

## Memberships

A monthly amount a backer chooses, with perks you define (early builds, a channel, a credit). Until the chain supports
delegated spend caps (M5.2), each month is one payment the backer approves; after it, a backer approves a recurring
amount once and can revoke it whenever they like.

## Tips

A plain transfer of CGT to you, at any stage, from your store page. No reward, no obligation.

## What your store page shows

The goal, what has been pledged and by how many backers, the deadline, the milestones and which are delivered — all read
from chain state, not from ARQADE's database.

## Not decided

The threshold above which a campaign is reviewed by a person, what happens to funds after a missed milestone, how long a
pledge may be held, and any platform share (U-17, U-15).
