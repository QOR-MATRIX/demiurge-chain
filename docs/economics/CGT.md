# CGT: the economic model

**Status:** canonical. Adopted from the owner's direction of 13 September 2026.
**Supersedes:** the fixed 13,000,000,000 supply and everything built on it: the 13B figure that was in
`.cursorrules` and in the retired `framework/primitives/src/denomination.rs`, and decisions D-001, D-002
and D-003 in [`../DECISIONS.md`](../DECISIONS.md). **The figure is out of the code entirely as of 20
September 2026:** the chain's constant went with `framework/` when M3.5 deleted it, and the launcher's
mirror of it went the same day. `chain/` declares no total-supply constant of any kind. Recorded in
[`../audit/RECONCILIATION.md`](../audit/RECONCILIATION.md), "Remediation since this audit".
**Open numbers:** nothing in this document states a rate, a split or a percentage that the owner has
not decided. Those are listed, with what would resolve them, in
[`OPEN_QUESTIONS.md`](OPEN_QUESTIONS.md).
**Decisions:** ADR-002 through ADR-008 in [`../decisions/`](../decisions/README.md).
**What the code does today:** [`../audit/RECONCILIATION.md`](../audit/RECONCILIATION.md) §1 and §3.

---

## 1. The governing principle

CGT derives its value from demand to spend it, not from demand to hold it.

Every mechanic in the system, whether a fee, a reward, a sink or an incentive, has to pass one test:

> If CGT could never be traded for dollars, would it still be worth holding?

If the answer is yes, there is a real economy underneath: people hold CGT because they need it to do
things they already want to do. If the answer is no, CGT is a speculative chip with a creator theme,
and every "earning" it produces is a bet on the next holder. A mechanic that fails the test is a
design defect, not a matter of taste, and is recorded as one (§9).

The failure this rules out is the common one. A creator token whose payouts depend on the token
appreciating pays creators only while the bet is on. When the bet stops, the earnings stop, and the
creators who built on it are left with a currency nobody needs. Demiurge is not built that way.
Creators are paid because someone used, hosted or licensed their work, and the payment is denominated
in a currency that the payer needed for that purpose.

## 2. What this replaces, and why

The previous plan fixed the supply at 13 billion CGT, allocated at genesis, with no issuance ever.
That plan is superseded, for three reasons.

**Unit legibility.** Per-use creator pricing at eighteen decimal places produces prices such as
0.000001 CGT for a single play or a single view. If CGT appreciates, every creator has to re-price
their catalogue, and every user needs a calculator to understand what anything costs. A larger unit
count means that ordinary prices read like ordinary money.

**The security and infrastructure budget.** Demiurge wants near-zero fees so that users never think
about gas. A fixed supply combined with near-zero fees leaves no perpetual income for the people who
run validators and store and serve content. Genesis pools pay them for a while and then run dry, and at
that point the network has no way to fund its own security or its own distribution. Under the previous
plan, the staking reserve was exactly such a pool.

**Deflation fights spending.** A fixed supply with a fee burn shrinks over time. That rewards holding
over spending, which is the opposite of what a creative economy's currency needs. A currency people
are reluctant to spend is a poor medium for paying creators.

The 13 billion figure was in the code when this was written, as `TOTAL_SUPPLY` in the custom chain's
`denomination.rs`, as a mirror of it in the launcher, and in the tests that pinned all three. **None of
that remains.** The chain's constant went with `framework/` at M3.5 and the launcher's mirror went on 20
September 2026; `chain/` declares no total-supply constant, because issuance is OPEN-1 and the genesis
split is OPEN-2, so there is nothing yet to declare. This document states the direction; what the code
does about it is in the reconciliation report and in [`../../HANDOFF.md`](../../HANDOFF.md) §1.

## 3. The supply model

**Base supply: 100,000,000,000,000 CGT, one hundred trillion.** The number is chosen for display
legibility, so that everyday amounts are whole numbers or short decimals, and not as a scarcity
signal. Scarcity is not the source of CGT's value; demand to spend it is (§1).

**The large majority of the base supply is allocated at genesis** and released over a multi-decade
decay curve to four recipients: validators, who secure the chain; seeders, who store and serve content
on the Mesh; the treasury; and the creator and player rewards pool. Release is gradual and declining,
so that the early network, which has the least fee volume, has the most support, and the mature
network, which should be self-funding through fees, has the least. The split between the recipients,
the shape of the curve and its duration are open (OPEN-2, OPEN-3).

**A small perpetual issuance, targeted under one percent a year,** continues after the genesis
release has decayed away. Its designated purpose is to fund the Mesh, meaning storage and bandwidth
provision, and validator security, in perpetuity. It is a paid-infrastructure budget: the network
issues CGT to pay for work it consumed. It is not a reward for holding, and nothing about it should be
described as one. The exact rate is open (OPEN-1).

**Fee burn is retained as the counterweight.** A share of every fee is destroyed. At meaningful
transaction volume, the burn offsets the issuance and net supply trends roughly flat rather than
growing. The burn share per class of fee is open (OPEN-4).

Issuance and burn are one mechanism, not two features. Issuance pays for infrastructure the network
needs regardless of volume; burn returns supply to the network in proportion to how much it is used.
Together they mean that a quiet network is subsidised, a busy network pays its own way, and neither a
runaway inflation nor a hoarding deflation is the resting state. Any document, model or piece of UI
that describes one without the other is describing half a mechanism.

**Precision is unchanged by this direction.** The chain encodes CGT at eighteen decimals with an
atomic unit called the Spark (`chain/runtime/src/denomination.rs`, the single source of truth since
M3.5; eighteen decimals was confirmed as a decision by ADR-035). One hundred trillion
CGT at eighteen decimals is 10^32 atomic units, which fits comfortably in the `u128` balance type
(maximum about 3.4 × 10^38). Eighteen decimals is **decided** (ADR-035), on both conditions the owner attached: the stale headroom
comment was corrected, and scaled arithmetic uses the SDK's helpers (AGENTS.md §5). U-1 is closed and
has left the open questions.

## 4. Useful-work mining: the Mesh

This is the central economic innovation of Demiurge and should be understood as such.

Bitcoin converts electricity into security by having machines do work that is intrinsically useless:
the hashes have no value beyond proving that effort was spent. Demiurge converts storage and bandwidth
that the network actually needs into currency. The work that earns CGT is the work of hosting and
delivering creators' content.

Installing the QOR Launcher makes a machine a potential node. The node contributes storage and
bandwidth to the Mesh. Creators pay CGT to have their work hosted and distributed; seeders earn CGT for
hosting and serving it. Payment is for verified work: what a seeder stored or served is checked against
content fingerprints recorded on chain, so the network pays for bytes it can prove were delivered, not
for bytes a peer claims to have delivered. The verification mechanism is not yet designed (U-6).

Difficulty is not artificially escalated. Bitcoin needs a difficulty adjustment because miners compete
over a fixed reward for useless work, and without the adjustment the reward would be captured by
whoever added hardware fastest. The Mesh does not have that problem, because the work is useful. The
correct limiter is real demand: seeders are paid for storage and bandwidth the network genuinely
consumes, and if nobody is downloading a file, nobody is paid for serving it. The network self-balances
on need.

One consequence follows from that design and should be stated plainly, in the right words. When the
network is small, demand per provider is high, so early seeders naturally earn more CGT per unit of
work than later seeders will. That is an emergent property of paying for consumed work, not a feature.
It must not be described, anywhere in the codebase, the documentation or the interface, as a return, a
guarantee, or an investment opportunity. Seeders are paid for hosting. The amount depends on what is
hosted and how much it is used.

## 5. The demand sinks

A national currency is grounded by an obligation to pay taxes in it: whatever else people think of the
currency, they need some of it. Demiurge has no taxing authority, so it needs its own structural
demand. Five sinks are designed to provide it. Each is load-bearing; each is a case of someone needing
CGT to do something they already want to do, which is the design target.

**Mesh hosting.** Creators pay CGT to store and distribute their work. Seeders earn it. Every
published work that anyone wants delivered generates demand on the creator's side and supply on the
seeder's side.

**Access gating.** Licenses, unlocks, remix rights and entitlements settle only in CGT. A player who
wants the sword, a studio that wants the track, a creator who wants to remix a model: all of them need
CGT at the moment of the transaction, and no other currency will do.

**Staking for distribution.** Creators stake CGT for reach and shelf space in Market and Nexus,
instead of buying advertising. Visibility is a scarce resource, and it is allocated by committing the
network's own currency to it. What happens to the staked CGT is not yet designed (U-7).

**Agent compute.** AI generation and agent operations spend CGT as a metered resource cost. An agent
that generates, tests or transacts on a user's behalf consumes CGT as it works.

**Escrow and reputation bonds.** Collaborations, commissions and licensing deals bond CGT. Trust
between strangers is created by putting the network's currency at stake, so trust itself creates
demand. Adjudication of disputes is not yet designed (U-8).

## 6. The display layer

Users must never be required to read raw CGT amounts to understand value.

The launcher and the web surface display a legible unit, working name "credits", formatted like
ordinary money: "you earned 4.20", not "0.0000042 CGT". The chain settles in CGT underneath. Raw CGT
amounts are visible only in advanced or developer views, where the person looking has asked for them.

Creators may denominate prices in a stable display unit, and the protocol converts to CGT at
settlement, so a catalogue does not need re-pricing when the market moves. How that conversion is
sourced is not yet designed and is one of the harder open questions (U-2), because it must be done
without turning the display unit into a peg.

The display layer is a legibility affordance and nothing more. Neither documentation nor interface
copy may imply that CGT is redeemable for, pegged to, or equivalent to any national currency. A
display that shows "4.20" is showing a number of credits, and credits are a way of reading CGT.

The launcher today formats raw CGT with a two-decimal display floor
(`tools/qor-launcher/src-tauri/src/cgt.rs:120-134`) and has no credits unit; the reconciliation report
records the current state.

## 7. Language discipline

Throughout the codebase, the documentation and the interface, Demiurge is infrastructure for creators
to earn from use. It is not a vehicle for appreciation.

That has concrete consequences for how things are written. There are no projected returns, no yield
framing and no price talk. Earnings language refers to payment for work, for hosting and for licensing,
and to nothing else. Validators are paid for securing the chain. Seeders are paid for storing and
serving content. Creators are paid when their work is used, licensed or remixed. Nobody is paid for
holding CGT, and no copy should suggest that they could be.

Words that do not belong in this project's copy include APY, yield, return, ROI, investment, passive
income and any statement about CGT's price or future value. Where a technical term is unavoidable, such
as a function's return value, it is fine; where the same word would be read as a financial promise, it
is not. Existing violations in code comments, frozen applications and earlier documents are listed in
the reconciliation report so that they can be corrected rather than copied forward.

## 8. What this model requires of the chain

The model above assumes mechanisms the chain does not have yet. Stating them here keeps the model and
the roadmap honest with each other.

Issuance requires a per-block or per-era release from genesis pools and a perpetual issuance path,
neither of which exists. **There is no mint path at all now**, which is a change from when this was
written: the custom chain's development faucet and admin mint went with `framework/` at M3.5, and
`chain/` has none. CGT exists only where a development chain specification endows an account, and
AGENTS.md §5 forbids adding any path that creates CGT outside `--dev` until the issuance mechanism is
designed. Burn requires a CGT fee to burn from; **no fee of any kind is charged**, because a
`WeightToFee` is an OPEN-4 value and the runtime therefore mounts no transaction-payment pallet. The
energy mechanism that took the place of a fee on the custom chain does not exist here.
Mesh payments require on-chain content fingerprints and a way to verify hosting. The fingerprints
exist: since 22 September 2026 `pallet-drc369` records ADR-047's 41-byte content reference (an
algorithm tag, a BLAKE3-256 root and a size) on every asset it mints. A way to verify hosting does not. The
demand sinks require licensing, staking-for-distribution, metered compute and escrow mechanics, none of
which exist. The display layer requires a credits formatting and, for stable pricing, a conversion
source; neither exists. The reconciliation report gives the evidence for each of these.

## 9. Mechanics that fail the spend test

The following mechanics, in code or in earlier documents, fail the test in §1 or the rules in this
document. Each is recorded as a design defect with its disposition.

| Mechanic | Where | Why it fails | Disposition |
| --- | --- | --- | --- |
| Fixed 13 billion supply with fee burn | the custom chain's `denomination.rs:54`, deleted at M3.5; `.cursorrules`; D-000/D-003 | Deflation rewards holding over spending; no perpetual infrastructure budget | Superseded by this document (ADR-003, ADR-004), and **out of the code since 20 September 2026** |
| Staking reserve paying "10% of the remainder per year" | D-002 in `docs/DECISIONS.md` | A pool that runs dry; framed as a rate on a holding | Superseded (ADR-004); the wording violates §7 |
| 0.001 CGT fee split 50% burn / 30% proposer / 20% treasury | D-001 in `docs/DECISIONS.md` | Numbers were chosen without the balance mechanism in §3; burn share per fee class is open | Numbers withdrawn (OPEN-4); sponsorship principle retained |
| Energy-only transactions with no CGT cost | the custom chain's `runtime.rs` and `energy` module, both deleted at M3.5 | Nothing in that chain created any demand to spend CGT | **Gone with the code.** `chain/` charges no fee either, but for a different reason: OPEN-4 is undecided, not a mechanism chosen instead (M6.4) |
| Reward and yield language in frozen applications and module names | the custom chain's `yield-nfts` module, deleted at M3.5; hub staking pages, still present and frozen | Violates §7 | Module gone with the code. The frozen pages remain and are not carried forward; `apps/` is frozen scope (D-011) |

Anything added later that fails the test belongs in this table, with a disposition, before it ships.
