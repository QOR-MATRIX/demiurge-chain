# ADR-101: The Spark is a thousandth of a CGT, and the ember is the chain's atomic unit

**Status:** **Accepted**, 18 October 2026, by the owner: "I'd also like to rework Sparks overall to be more along the
lines of 1000 Sparks = 1 CGT." Asked how, the owner chose a Spark of 0.001 CGT over an unchanged chain, rather than a
chain of three decimals, and named the atomic unit the **ember**. **Keeps** [ADR-035](ADR-035-eighteen-decimals.md):
CGT has eighteen decimals. **Replaces** the definition "1 CGT = 10^18 Sparks; a Spark is the atomic unit" in
[`../DECISIONS.md`](../DECISIONS.md) and [`../economics/CGT.md`](../economics/CGT.md). **Answers part of** U-2 (the
legible display unit). It decides **no economic value**: it names units, and changes no amount.

## Context

Since September 2026 the Spark was the chain's atomic unit, 10^-18 CGT. Nobody can picture a Spark that small, so every
screen showed CGT with up to eighteen decimals, and a reward of a few thousandths of a CGT read as a long string of
zeros. The owner wants the Spark to be the everyday small unit: a thousandth of a CGT, the unit players are rewarded in.

Eighteen decimals stays right underneath. A royalty split, a remix share and the Mesh's price per MiB routinely come to
less than a thousandth of a CGT, and three decimals could not carry them.

## Decision

1. **1 CGT = 1,000 Sparks.** A Spark is 10^15 embers. It is a unit of display and entry, not of the chain.
2. **The ember is the atomic unit: 1 CGT = 10^18 embers.** Every balance, price and transfer is a whole number of embers
   in `u128`, exactly as the Spark was. The chain does not change: its balances, deposits and pallets are the same
   numbers, renamed in documentation.
3. **How amounts are shown.** An amount of a whole number of Sparks shows as CGT with up to three decimals ("1.250 CGT")
   or, where the amount is small or a reward, as Sparks ("250 Sparks"). An amount with more precision than a Spark shows
   in full ("0.0004 CGT"), never rounded, so what is shown is what moves. People type CGT or Sparks; more precision than
   an ember is refused, never truncated (AGENTS.md §5).
4. **Names in code and on the wire.** The atomic unit is `embers` in identifiers (`embers`, `amount_embers`,
   `EMBERS_PER_CGT`), and `SPARK` is the constant of 10^15 embers. A wire field that carried atomic units as `sparks`
   is renamed `embers`, and a reader accepts the old name for one release so launchers already installed keep working.

## Consequences

- AGENTS.md §5 reads "All amounts are integer embers in `u128`", and every current document is brought into line.
- The chain's pallet documentation, which is part of its metadata, says embers from the next runtime upgrade; nothing
  else on chain changes, so no migration and no reset.
- U-2 keeps its other half: whether a stable display unit ("credits") is converted to CGT at settlement.
