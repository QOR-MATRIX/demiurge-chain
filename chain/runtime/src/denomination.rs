//! What one CGT is worth, and how the chain describes it.
//!
//! The single source of truth for this chain, and now the only one: the custom
//! devnet's own denomination module went with that directory at M3.5. Every
//! value here is decided in a record, and none of them is picked.

/// Decimal places (ADR-035).
///
/// `1 CGT = 10^18 embers`, and the ember is the atomic, indivisible unit.
///
/// Headroom: at the base supply of 100,000,000,000,000 CGT (ADR-003) the full
/// supply is `10^32` embers against `u128`'s ceiling of about `3.4 × 10^38`, so
/// about six orders of magnitude spare. That is enough for the SDK's own
/// arithmetic, which was checked rather than assumed: `PerThing` divides before
/// it multiplies, and `multiply_by_rational_with_rounding` uses a 256-bit
/// intermediate. It is **not** enough for a hand-written `a * b / c`, where
/// `a * b` overflows long before the result would, which is why AGENTS.md §5
/// requires the SDK's helpers for scaled arithmetic.
pub const DECIMALS: u8 = 18;

/// One CGT, in embers.
pub const CGT: u128 = 10u128.pow(DECIMALS as u32);

/// One Spark, in embers: a thousandth of a CGT (ADR-101). The unit people see and type, and the unit players are
/// rewarded in. Nothing on chain is denominated in it; every balance stays a whole number of embers.
pub const SPARK: u128 = CGT / 1_000;

/// The minimum balance an account must hold to be kept in state: 100 CGT
/// (ADR-036), which is `10^20` embers.
///
/// Derived from the owner's target, never picked (ADR-030): sponsoring a
/// million accounts costs a negligible share of the sponsorship budget, while
/// dust spam stays uneconomic. Revisitable before mainnet only, and only if
/// OPEN-2's sponsorship budget changes that arithmetic.
pub const EXISTENTIAL_DEPOSIT: u128 = 100 * CGT;

/// The deposit must be above zero, or `pallet-balances`' integrity test panics
/// at the pinned release. Asserted at compile time rather than in a test, so a
/// future edit to the constant cannot produce a runtime that only fails when it
/// is started.
const _: () = assert!(EXISTENTIAL_DEPOSIT > 0);

/// The address prefix for development and test networks (ADR-024).
///
/// Prefix 42 is the shared Substrate prefix. The registry that once allocated
/// prefixes is archived, and the mainnet prefix is a Public Release criterion,
/// so no prefix is claimed here.
pub const SS58_PREFIX: u16 = 42;

/// The ticker (ADR-045, superseding ADR-034).
///
/// It was `DMRG` for four days. ADR-034 moved it off `CGT` because that symbol
/// is Curio's in the SS58 registry and in shipped wallet data; ADR-045 moves it
/// back, accepting that collision in the owner's name, because `DMRG` inherits
/// the `DEMIURGE` trademark exposure that `CGT` does not. Both records stand.
pub const TOKEN_SYMBOL: &str = "CGT";

/// The currency's name, which neither ticker change altered.
pub const TOKEN_NAME: &str = "Creator God Token";

#[cfg(test)]
mod tests {
    use super::*;

    /// The unit is what ADR-035 decided, and the deposit is what ADR-036
    /// decided, expressed in it.
    #[test]
    fn the_unit_and_the_deposit_are_what_was_decided() {
        assert_eq!(DECIMALS, 18);
        assert_eq!(CGT, 1_000_000_000_000_000_000);
        assert_eq!(EXISTENTIAL_DEPOSIT, 100 * CGT);
        assert_eq!(EXISTENTIAL_DEPOSIT, 100_000_000_000_000_000_000);
    }

    /// A Spark is a thousandth of a CGT, a whole number of embers (ADR-101).
    #[test]
    fn a_spark_is_a_thousandth_of_a_cgt() {
        assert_eq!(SPARK * 1_000, CGT);
        assert_eq!(SPARK, 1_000_000_000_000_000);
    }

    /// ADR-030 derived 100 CGT from two constraints. Asserting the constant
    /// equals itself would prove only that nobody mistyped it; these assert the
    /// constraints it came from still hold, so that changing the budget without
    /// re-deriving the deposit fails here (ADR-036).
    ///
    /// The budget is a **placeholder**: OPEN-2, the genesis allocation split, is
    /// undecided, and AGENTS.md §5 forbids presenting such a number as decided.
    /// It is the 0.1% of supply that ADR-030 assumed, marked as an assumption.
    #[test]
    fn the_deposit_still_satisfies_the_target_it_was_derived_from() {
        // Whole CGT throughout: these are the units ADR-030 reasons in, and
        // the deposit's atomic encoding is checked separately above.
        const BASE_SUPPLY: u128 = 100_000_000_000_000;
        // PLACEHOLDER, not a decided value. OPEN-2 decides the real budget.
        const ASSUMED_SPONSORSHIP_BUDGET: u128 = BASE_SUPPLY / 1_000; // 0.1% of supply
        const ONE_MILLION: u128 = 1_000_000;
        // ADR-030 read "negligible" as at most a thousandth of the budget.
        const NEGLIGIBLE_DENOMINATOR: u128 = 1_000;

        let ed = EXISTENTIAL_DEPOSIT / CGT;

        // Constraint 1: sponsoring a million accounts is negligible against the
        // budget. Written as a multiplication so it never divides to zero.
        let cost_of_a_million = ed * ONE_MILLION;
        assert!(
            cost_of_a_million * NEGLIGIBLE_DENOMINATOR <= ASSUMED_SPONSORSHIP_BUDGET,
            "sponsoring a million accounts costs {cost_of_a_million} CGT, which is more than              1/{NEGLIGIBLE_DENOMINATOR} of the assumed budget of {ASSUMED_SPONSORSHIP_BUDGET} CGT.              Re-derive the deposit from ADR-030's method before changing this test."
        );

        // Constraint 2: dust spam stays uneconomic. A holder of 0.01% of supply
        // can keep at most this many accounts alive, which ADR-030 put at 10^8,
        // roughly 16 GB of state.
        let one_hoard = BASE_SUPPLY / 10_000;
        let accounts_one_hoard_keeps = one_hoard / ed;
        assert!(
            accounts_one_hoard_keeps <= 100_000_000,
            "a 0.01% holder could keep {accounts_one_hoard_keeps} accounts alive, above the              10^8 ADR-030 accepted"
        );
    }
}
