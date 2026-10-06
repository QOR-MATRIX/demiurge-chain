# Open questions in the economic model

**Status:** current as of 13 September 2026.
**Rule:** nothing here has a value. An honest gap is worth more than a plausible number that later gets
treated as decided. When an item is resolved, the value goes into [`CGT.md`](CGT.md), an ADR records
the reasoning in [`../decisions/`](../decisions/README.md), and the item is removed from this file.

Two kinds of item are listed. **OPEN** items are those the owner explicitly left undecided in the
direction. **Unaddressed** items are those the direction does not mention but which the model cannot be
implemented without; they are listed so that nobody fills them in by accident.

The foreclosing event for most of these is the genesis of the first network whose CGT is meant to
carry value. Genesis allocation and issuance schedule are in the "stay boring" category of the
innovation budget precisely because they cannot be changed afterwards. The first public testnet is the
last cheap point at which the chosen values can be rehearsed; SDK publication forecloses anything that
touches wire formats.

---

## OPEN items

### OPEN-1: the perpetual issuance rate

The direction targets a rate under one percent a year, designated for Mesh funding and validator
security. The exact rate is undecided.

Deciding it needs: an estimate of what the network must pay per year for the storage and bandwidth it
expects to consume, and for the number of validators it wants to secure it; the relationship between
that budget and the genesis release curve (OPEN-3), since issuance only has to carry what the decaying
genesis release no longer does; and a model of fee volume and burn (OPEN-4) at several adoption levels,
so that the point at which burn offsets issuance is known rather than hoped for.

Foreclosed by: mainnet genesis. Issuance is protocol law after that; changing it is a hard fork or a
governance action, and governance cannot execute anything today.

### OPEN-2: the genesis allocation split

The direction names six recipients: validators, seeders, treasury, creator/player rewards, team and
ecosystem. Their shares are undecided.

Deciding it needs: the treasury's expected operating budget for its first years; the expected number
and cost of validators and seeders at launch, since they are paid from their pools until fees carry
them; the size of the rewards pool needed for the programs that will draw on it; the legal posture on a
team allocation and its vesting, which the earlier plan set at a one-year cliff and four-year vest and
which is withdrawn with that plan; and what "ecosystem" is meant to fund that treasury does not.

Foreclosed by: mainnet genesis. The split is written into the genesis state and hashed into the chain's
identity, so it cannot be changed afterwards without a new chain. On `chain/` the genesis state is built
by `chain/node/src/chain_spec.rs` through the runtime's `GenesisBuilder`; the development specifications
there endow well-known development accounts only, and carry no pools at all.

### OPEN-3: the decay curve

The direction says the genesis majority is released over a multi-decade curve. Its shape (linear,
exponential, stepped) and its duration are undecided, as is whether every pool decays on the same
curve.

Deciding it needs: OPEN-1 and OPEN-2, since the three together determine the network's income in any
given year; a decision on the release cadence (per block, per era) and on whether releases are
automatic or claimed; and a view on how long the early subsidy should last relative to expected fee
growth.

Foreclosed by: mainnet genesis for the curve parameters. The release mechanism itself must exist before
the first public testnet, or the testnet cannot rehearse it.

### OPEN-4: the burn share per fee class

The direction retains fee burn as the counterweight to issuance and says the burn percentage is set
per class of fee. Neither the fee classes nor their burn shares are defined.

Deciding it needs: a list of fee classes (transfers, minting, licensing, hosting payments, agent
operations, and so on); a decision on where the non-burned share of each fee goes; and the same fee
volume model as OPEN-1, because the burn share is what makes net supply roughly flat at meaningful
volume.

Foreclosed by: less firmly than the others. Burn shares are the kind of parameter that governance
could adjust later, once governance can execute. The first public testnet should still carry the
intended values so that the balance mechanism is observed rather than assumed.

---

## Unaddressed by the direction

### Precision, settled and recorded elsewhere

**Decided on 17 September 2026: eighteen decimal places**, on the two conditions this item recommended, both met in
the same change. CGT has eighteen decimals, `1 CGT = 10^18 Sparks`, the Spark is the atomic unit, the stale headroom
argument in `denomination.rs` is corrected, and the rule that scaled arithmetic uses the SDK's helpers rather than a
hand-written `a * b / c` is written into AGENTS.md §5. The record, including the corrected arithmetic and what was
checked against the pinned release, is [ADR-035](../decisions/ADR-035-eighteen-decimals.md). The existential deposit
it blocked is [ADR-036](../decisions/ADR-036-existential-deposit-100-dmrg.md): 100 CGT.

### U-2: the stable display unit and conversion at settlement

The direction lets creators price in a stable display unit with the protocol converting to CGT at
settlement, while forbidding any implication of a peg. Conversion needs a reference rate from
somewhere: an on-chain oracle, a governance-set figure, or a market. None exists, and each has a
different trust and language problem. Unknown.

### U-3: what "credits" is

"Credits" is the working name for the legible display unit. Whether a credit is a fixed multiple of
CGT (a formatting choice) or the stable unit of U-2 (a conversion) is not stated. The two are different
things with different failure modes. Unknown.

### U-4: fee classes and sponsorship mechanics

The chain must let end users transact without holding tokens. **Nothing in `chain/` does this today, and
nothing charges anything either:** the runtime mounts no transaction-payment pallet, because a
`WeightToFee` is an OPEN-4 value. The custom chain's partial answer — an energy module whose `Sponsor`
call consumed the sponsor's energy without making any user transaction free — was deleted with
`framework/` at M3.5 and is not carried forward. How a sponsor is designated, what it pays, how caps are
enforced and which fee classes exist are all still undefined.

**Proposed for the owner's review** in [`../architecture/SPONSORSHIP.md`](../architecture/SPONSORSHIP.md), under
ADR-029 (a sponsor pays the existential deposit, and the user holds it). Nothing is decided until the owner reviews it.

### U-6: verifying seeders' work

The Mesh pays seeders for storage and bandwidth verified against on-chain fingerprints. What a seeder
proves, to whom, and how often (proof of storage, proof of retrieval, challenge-response, client
receipts) is undefined. Without it, the largest demand sink pays for unverifiable claims.

### U-7: staking for distribution

Creators stake CGT for reach in Market and Nexus. Whether staked CGT is locked and returned, spent to
the treasury, or burned, and how stake maps to placement, is undefined. The answer changes whether this
sink is demand or merely a deposit.

### U-8: escrow and reputation bonds

Who adjudicates a disputed commission or licensing deal, and what happens to a bond when the parties
disagree, is undefined.

### U-9: agent compute metering

Who provides the compute that agents spend CGT on, and how it is priced, is undefined. If compute is
provided off chain by the user's own model, the sink is a payment for on-chain operations only.

### U-10: where voting power comes from

On-chain governance is one of the four chain properties. If voting power is based on holding CGT, the
governance design sits uneasily with a currency whose value is meant to come from spending. Unknown.

**Must be decided before OpenGov lands** (ADR-021; a Public Release criterion in `docs/GATES.toml`). It is
not assumed anywhere. The collective of ADR-021 votes one member, one vote, and says nothing about CGT.

**Why holdings are a problem here.**
- OpenGov's standard voting (`pallet-conviction-voting`) weights a vote by the tokens locked behind it,
  multiplied by how long they stay locked.
- Influence then comes from holding and locking CGT, which is a reason to hold rather than to spend. That is
  exactly what ADR-002's spend test rejects.
- Before CGT is widely distributed, whoever holds the genesis pools would control every decision.

**Options.**
- **A. Holdings.** Standard conviction voting on CGT.
  - For: proven, standard, and what tooling expects.
  - Against: fails the spend test; concentrated before distribution; influence can be bought.
- **B. One person, one vote.** Voting power per verified person.
  - For: no link to holdings.
  - Against: needs resistance to one person holding many accounts. QOR ID holds identity off chain
    (ADR-027), and making its attestation something the chain acts on would put QOR ID's say-so behind
    governance: the same trust ADR-017 and R-3 keep out of value.
- **C. Contribution.** Voting power from work the network used: CGT spent in fees and burned, seeding
  verified under U-6, creators paid for use.
  - For: aligned with ADR-002.
  - Against: needs a custom, non-transferable weight that must resist gaming (wash spending costs real
    burn, which is its price), and governance is not where ADR-001 wants novelty.
- **D. Hybrid: tracks with membership ranks.** Standard `pallet-referenda` tracks whose voters are the ranks
  of a `pallet-ranked-collective`. Ranks are non-transferable and one member casts one vote at their rank.
  They are granted by governance for contribution. Different tracks carry different weight: runtime
  upgrades, treasury, parameters. No track counts CGT holdings until distribution is broad and this question
  is re-examined.
  - For: standard pallets throughout, and no influence from holding.
  - Against: whoever grants ranks can capture governance, and it is not token-holder decentralisation.

**Recommendation: D.** It gives OpenGov's machinery (tracks, referenda, scheduling, preimages) without making
holding CGT the source of power, and it needs no custom voting pallet. The rules for granting ranks become the
decision that matters, and are set out in the ADR that adopts it. Holding-weighted tracks are not ruled out
for ever: they are deferred until distribution is broad, and re-decided then.

**Foreclosed by:** OpenGov landing, which must happen before mainnet.

**Known unresolved tension, recorded here and in ADR-020 so that it is not rediscovered.**
- **The move.** ADR-020 defers nominated proof of stake until CGT is widely distributed, and `docs/GATES.toml` requires
  the move before mainnet.
- **The problem.** At mainnet genesis, CGT has barely begun its multi-decade release (ADR-003, OPEN-3). Stake, and any
  voting power drawn from holdings, could still belong mostly to whoever holds the genesis pools.
- **The trap.** ADR-020's own precondition may block its own gate criterion.
- **Where it is settled.** Deciding this question, with OPEN-2 and OPEN-3, decides whose stake and votes count at
  genesis, and so which gives way: the criterion, the precondition, or the timing.

### U-11: validator income under sponsored, near-zero fees

If fees are near zero and often sponsored, validators are paid almost entirely from the genesis
release and perpetual issuance. That is consistent with the direction, but it means OPEN-1 and OPEN-2
together are the whole validator budget, and the split between validators and seeders needs to be
decided as one question.

### U-12: team and ecosystem release terms

The earlier plan's team vesting (one-year cliff, then linear over three years) is withdrawn with that
plan. Nothing replaces it. Whether team and ecosystem allocations decay on the same curve as the
infrastructure pools, or vest separately, is unknown.

### U-14: storage deposits for assets

`pallet-nfts` holds a deposit from whoever stores a collection, an asset, a name or an attribute, and `pallet-drc369`'s
records are priced inside those. ADR-030 says each of these deposits "needs its own sizing", and nothing has sized them:
`../architecture/SPONSORSHIP.md` proposes who pays, not how much. What a deposit is for — pricing permanent state, as
the existential deposit does — is settled; how much each one is, is not. The amounts change what a new creator must hold
before a first mint (F-D7), which is why ADR-047 left the singles-collection question to the owner.

Since 22 September 2026 the runtime carries **placeholders**, each marked as one in `chain/runtime/src/assets.rs` and
derived rather than picked, by ADR-030's arithmetic: a storage entry costs one existential deposit, a byte a
hundred-and-sixtieth of one (ADR-052). That is a number a development chain can run on, not an answer.

**Two more entries per asset since 29 September 2026 (ADR-061).** `pallet-drc369-royalties` stores at most one set of
royalty terms (up to eight recipients) and one listing per asset, and takes no deposit for either: only the asset's
holder can write them, so how many can exist is already bounded by the assets, which hold deposits. The placeholder
`ITEM` deposit still prices four entries; a sizing that answers this question should count the two.
`public-release.economics-decided` counts this item.

### U-15: the platform's share of a sale, and whether it may be taken in fiat

On 22 September 2026 the owner asked for a Sell path that posts an asset to a marketplace every on-chain user can
see, and for a marketplace on demiurge.cloud that settles in USD "with a small fixed 15% fee per asset
transaction". The share and the rail are both open, and they are different questions.

**The share.** Fifteen percent is the owner's proposal, recorded here as a proposal and written nowhere as a
decided value: AGENTS.md §5 forbids that while it is undecided. Two things are unstated besides the number. What
it is a share *of* — the price before or after a creator's remix royalty (M4.2) — gives the creator different
money. And where it goes: no treasury exists (M6.5), and funding one touches OPEN-2.

Since 29 September 2026 the chain settles sales in CGT (ADR-061) and **takes no share**. If one is decided it is one
more payment inside `pallet-drc369-royalties`'s `split`, and the order matters: taken before the royalties, it
reduces what creators receive; taken after, it does not.

**The rail.** Settling in USD means someone holds money, which brings custody, KYC/AML, chargebacks and refunds,
none of which a chain can arbitrate. An off-chain payment against an on-chain delivery means one side moves
first, which is escrow — U-8, also open. It is also the one path that can make a product a money transmitter in
some jurisdictions: a legal question, not an engineering one, and the owner's to take.

Nothing is blocked on an answer today. The royalty pallet (M4.2), the indexer (M5.4) and a listing settled in CGT
(P5.5) are the work in front of it, and each is needed whatever the answer is.

### U-16: paid play, prizes, and who funds a game's payouts

Opened on 4 October 2026 by ADR-069, when the owner named ARQADE as the gaming platform, with CGT as the only cost
and payout currency. A paid game entry is access gating (ADR-006). A prize paid to a player is not: it is neither
payment for work nor licensing (ADR-008), and it fails the spend test (ADR-002) unless whatever funds it is itself
demand to spend. Three things are undecided, and none has a value:

- **Whether a paid entry may lead to a CGT or asset prize at all**, and under which rules — skill only, or chance as
  well. Paid chance with a transferable prize is a legal question in many jurisdictions before it is an economic one.
- **What funds a prize — answered in direction on 4 October 2026.** The owner: a game pays from a dedicated payout
  account of its own, reached through the ARQADE SDK. ADR-070, accepted that day, makes it a keyless **ARQ Wallet** (the owner's name) per
  published game, funded by its developer. Whether entry fees may also flow into it stays here, with the first bullet.
- **The protocol's bounds on an ARQ Wallet's policy**: the shortest delay before a loosening or a withdrawal applies,
  how long a paid outcome id is remembered, and the longest an unpaid accrual waits. A developer's own caps are theirs
  and are not platform values; these bounds are. Any platform share of entries joins U-15's question.

On the devnet ARQADE builds and tests the whole loop with test CGT and values marked as placeholders. In production
nothing is paid out until this item is decided and removed.

### U-17: backing a game in CGT

Opened on 4 October 2026 by ADR-071, when the owner asked for backers to support ARQADE projects in CGT, as on
Kickstarter and Patreon. Backers receive the work (DRC-369 rewards), never its proceeds, which is settled; what is not:

- **The threshold** above which a campaign is reviewed by a person before it opens.
- **A missed milestone**: whether the remaining funds return to backers, and on what terms.
- **How long a pledge may be held** before a campaign must settle.
- **Any platform share** of pledges, memberships or tips (which joins U-15).

Crowdfunding and donations are regulated in many places, so production also waits on the owner's legal review. On the
devnet it is built and tested with test CGT and values marked as placeholders.

### U-18: CGT for tasks, the welcome grant, and what funds them

Opened on 6 October 2026 by ADR-078, when the owner asked for CGT rewards for completing tasks across the ecosystem and a
small welcome grant on finishing the tutorial. Levels and XP are settled there and touch no CGT. What is not:

- **The welcome grant's amount, N.** The owner's, above 100 CGT (the existential deposit), or the account opens with
  nothing to spend. Asked of the owner with ADR-078.
- **Who funds the Welcome account** with real CGT, and whether task rewards beyond the grant pay CGT at all, and how much.
- **The ten-minute boost**: what it multiplies and by how much.
- **Whether a reward is a payment for work, a licence, or neither** (ADR-008), which decides whether it passes the spend
  test (ADR-002) and which demand sink, if any, it belongs to (ADR-006).

Rewards for sign-up tasks attract account farming, and token grants and rewards are regulated in many places, so real
CGT also waits on the owner's legal review. On the devnet the grant runs with test CGT from the faucet.

### Settled, and recorded elsewhere

**U-13, the currency's ticker.** Decided on 17 September 2026 and no longer open: the ticker is `CGT` and the name
stays Creator-God Token. The decision, the collision it escapes, the clearance research behind `CGT`, the candidates
rejected before it, the method and its limits, and the sources are all in
[ADR-034](../decisions/ADR-034-ticker-dmrg.md). Nothing about it is re-litigated here.
