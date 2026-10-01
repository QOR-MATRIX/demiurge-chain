//! Selling a DRC-369 asset, and buying one, settled on chain in CGT (L4.6).
//!
//! # What the chain does, and what this module adds
//!
//! `pallet-drc369-royalties` holds a listing — an asset, its seller and a price
//! in CGT — and settles a sale in one transaction: the remix share to the work
//! it was derived from, then the asset's own royalties, then the seller, then
//! the asset to the buyer (ADR-061, ADR-062). All of it happens or none of it.
//!
//! This module calls `list`, `unlist` and `buy`, each built from the connected
//! node's metadata, approved in the host dialog and signed in the vault like
//! every other transaction (ADR-040, roadmap L1.4), and it reads back what an
//! asset's listing and royalty terms are. **Everything it shows is read from
//! chain storage at the moment it is asked**; the launcher keeps no list of
//! listings of its own.
//!
//! # There is no storefront, and this does not pretend to be one
//!
//! A listing is public the moment it is in a block, but nothing serves a
//! catalogue: that is an indexer over the chain's events (ADR-028, M5.4), which
//! is not built. So a buyer reaches an asset by its number, `collection/item`,
//! which its holder gives them — [`parse_asset_id`] reads it out of whatever
//! was pasted, including the line an asset's menu copies.
//!
//! # The arithmetic is the chain's, and so is the last word
//!
//! Before anyone signs, the dialog says who a sale pays and how much. Those
//! amounts come from [`split`], which is the pallet's own `split` run on the
//! terms read from the chain, with the SDK's own helpers at the pinned version:
//! `Permill::mul_floor` and `multiply_by_rational_with_rounding`. No float, and
//! no hand-written `a * b / c` (AGENTS.md §5, ADR-035). The tests below pin it
//! to the pallet's own vectors, and the live test holds it to the `Sold` event
//! of a real sale, part for part.
//!
//! What is shown **before** a sale is this module's reading of the chain; what
//! is reported **after** one is the chain's `Sold` event and nothing else.
//!
//! # What is refused before anyone is asked
//!
//! A person should not be asked to approve what the chain would refuse. So a
//! listing that is void, a price that changed since it was looked at, a
//! fingerprint that changed since it was looked at, a balance that cannot cover
//! the price and still keep the account open, and a part that its recipient
//! could not receive are each said in words, before the dialog. The chain still
//! checks every one of them again, and [`in_words`] turns its refusal into the
//! same sentences when something changes in between.
//!
//! # Nothing is taken that the chain does not take
//!
//! No platform share (U-15 is undecided) and no fee (OPEN-4): the parts of a
//! sale are the remix share, the royalties and the seller, and they sum to the
//! price.

use scale_decode::DecodeAsType;
use serde::Serialize;
use sp_arithmetic::{
    helpers_128bit::multiply_by_rational_with_rounding, per_things::Rounding, Permill,
};
use subxt::dynamic;
use subxt::utils::AccountId32;

use crate::cgt;
use crate::error::{QorError, QorResult};
use crate::vault::derive::address_of;
use crate::vault::{normalise_address, Vault};
use crate::{Confirm, Prompt};

use super::assets::{OwnedAsset, TradeItem};
use super::{ChainClient, Connection};

/// An account, as the chain keys it.
type Account = [u8; 32];

// ── The arithmetic ───────────────────────────────────────────────────────────

/// How one sale's price is divided: `pallet-drc369-royalties`'s `Split`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Split<A> {
    /// What the source's recipients receive, if the asset is a remix.
    pub remix: Vec<(A, u128)>,
    /// What the asset's own recipients receive.
    pub royalties: Vec<(A, u128)>,
    /// What the seller receives: everything else.
    pub seller: u128,
}

/// Divide `price` between a remix's source, the asset's own recipients and the
/// seller.
///
/// **This is `pallet_drc369_royalties::split`, line for line**, so that what the
/// dialog shows before a sale is what the chain pays in it. The pool is the
/// source's remix share of the price, rounded down, divided between the source's
/// recipients in proportion to their shares; each of the asset's own recipients
/// receives their share of what is left; the seller receives the rest, rounding
/// included. The parts always sum to `price`.
pub(crate) fn split<A: Clone>(
    price: u128,
    upstream: Option<(Permill, &[(A, Permill)])>,
    own: &[(A, Permill)],
) -> Split<A> {
    let mut remix = Vec::new();
    let mut paid_upstream: u128 = 0;
    if let Some((share, recipients)) = upstream {
        let pool = share.mul_floor(price);
        let weights: u128 = recipients
            .iter()
            .map(|(_, part)| u128::from(part.deconstruct()))
            .sum();
        if weights > 0 {
            for (who, part) in recipients {
                // `part <= weights`, so the result is at most `pool` and the
                // helper cannot report an overflow.
                let amount = multiply_by_rational_with_rounding(
                    pool,
                    u128::from(part.deconstruct()),
                    weights,
                    Rounding::Down,
                )
                .unwrap_or(0);
                paid_upstream = paid_upstream.saturating_add(amount);
                remix.push((who.clone(), amount));
            }
        }
    }

    // `paid_upstream <= pool <= price`.
    let rest = price.saturating_sub(paid_upstream);
    let mut paid_own: u128 = 0;
    let royalties = own
        .iter()
        .map(|(who, share)| {
            let amount = share.mul_floor(rest);
            paid_own = paid_own.saturating_add(amount);
            (who.clone(), amount)
        })
        .collect();

    // Stored shares sum to at most one whole, so `paid_own <= rest`.
    Split {
        remix,
        royalties,
        seller: rest.saturating_sub(paid_own),
    }
}

// ── What the launcher reads, decoded by name ─────────────────────────────────

/// `Drc369Royalties::RoyaltyTerms`. A share is a `Permill`: parts per million.
#[derive(Debug, Clone, DecodeAsType)]
struct TermsRead {
    recipients: Vec<(AccountId32, u32)>,
    remix: u32,
}

/// `Drc369Royalties::Listings`.
#[derive(Debug, Clone, DecodeAsType)]
struct ListingRead {
    seller: AccountId32,
    price: u128,
}

/// The part of `Nfts::Item` that says who holds an item.
#[derive(Debug, DecodeAsType)]
struct ItemRead {
    owner: AccountId32,
}

/// The part of `Drc369::Assets` that says what an asset was derived from.
#[derive(Debug, DecodeAsType)]
struct SourceRead {
    derived_from: Option<(u32, u32)>,
}

#[derive(Debug, DecodeAsType)]
struct AccountRead {
    data: FundsRead,
}

#[derive(Debug, DecodeAsType)]
struct FundsRead {
    free: u128,
    reserved: u128,
    frozen: u128,
}

/// `Drc369Royalties::Sold`.
#[derive(Debug, DecodeAsType)]
struct SoldRead {
    collection: u32,
    item: u32,
    from: AccountId32,
    to: AccountId32,
    price: u128,
    remix: Vec<(AccountId32, u128)>,
    royalties: Vec<(AccountId32, u128)>,
    seller_received: u128,
}

// ── What the interface is given ─────────────────────────────────────────────

/// A listing, as a person reads it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ListingView {
    /// Who listed it, as this chain writes an address.
    pub seller: String,
    /// The price in Sparks, as a decimal string: `u128` does not survive
    /// JavaScript's `number`.
    pub price_sparks: String,
    /// The price in CGT, grouped for reading.
    pub price_cgt: String,
    /// Whoever listed it no longer holds the asset, so nobody can buy it. Its
    /// holder, or anyone, may clear it.
    pub void: bool,
}

/// One recipient of an asset's royalty terms.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ShareView {
    pub address: String,
    /// Their share, as a person reads it: `10%`, `2.5%`.
    pub share: String,
}

/// An asset's royalty terms: who is paid from each sale of it, and what a sale
/// of a remix of it owes them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TermsView {
    pub recipients: Vec<ShareView>,
    /// What a sale of any remix of this asset owes its recipients.
    pub remix: String,
}

/// Why a part of a sale is paid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutKind {
    /// The remix share: to a recipient of the asset this one was derived from.
    Source,
    /// A royalty: to a recipient of the asset's own terms.
    Royalty,
    /// The seller: everything else.
    Seller,
}

/// One part of a sale's price, and who receives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Payout {
    pub kind: PayoutKind,
    pub address: String,
    /// A royalty recipient's share of every sale. `None` for the seller, who
    /// receives what is left, and for a source's recipients, who divide the
    /// remix share between them in proportion to their own shares.
    pub share: Option<String>,
    pub amount_sparks: String,
    pub amount_cgt: String,
    /// The recipient is the buyer, so this part never leaves their account.
    pub to_buyer: bool,
}

/// What a sale at one price pays, part by part, in the order the chain pays it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Breakdown {
    pub price_sparks: String,
    pub price_cgt: String,
    /// The asset this one was derived from, when a sale owes it a remix share.
    pub source: Option<TradeItem>,
    /// That source's remix share, as a person reads it.
    pub source_share: Option<String>,
    /// The source's recipients, then the asset's own, then the seller. Their
    /// amounts sum to the price.
    pub payouts: Vec<Payout>,
    /// Why the chain would refuse a sale at this price as things stand, in
    /// words, if it would.
    pub blocked: Option<String>,
}

/// One asset, whoever holds it, and everything about selling or buying it.
#[derive(Debug, Clone, Serialize)]
pub struct SaleView {
    pub asset: OwnedAsset,
    /// Who holds it, as this chain writes an address.
    pub holder: String,
    /// The account asking holds it.
    pub held_by_viewer: bool,
    /// The asset it was derived from, if its minter declared one.
    pub derived_from: Option<TradeItem>,
    /// Its own royalty terms, if its creator set any.
    pub terms: Option<TermsView>,
    /// The terms of the asset it was derived from, if that has any.
    pub source_terms: Option<TermsView>,
    /// What a sale at the listed price pays. `None` when it cannot be bought:
    /// not listed, or the listing is void.
    pub breakdown: Option<Breakdown>,
    /// What the account asking can spend, in CGT, grouped for reading.
    pub viewer_free_cgt: Option<String>,
    /// Why the account asking cannot buy it, in words. `None` when it can.
    pub cannot_buy: Option<String>,
    /// When a fingerprint was pasted with the asset's number: whether it is
    /// the one the asset carries now.
    pub pasted_root_matches: Option<bool>,
}

/// A finalised listing.
#[derive(Debug, Clone, Serialize)]
pub struct ListReceipt {
    pub collection: u32,
    pub item: u32,
    pub price_sparks: String,
    pub price_cgt: String,
    pub tx_hash: String,
    pub block_hash: String,
}

/// A finalised withdrawal of a listing.
#[derive(Debug, Clone, Serialize)]
pub struct UnlistReceipt {
    pub collection: u32,
    pub item: u32,
    pub tx_hash: String,
    pub block_hash: String,
}

/// A finalised sale, as the chain's own `Sold` event reports it.
#[derive(Debug, Clone, Serialize)]
pub struct SaleReceipt {
    pub collection: u32,
    pub item: u32,
    pub name: String,
    pub seller: String,
    pub buyer: String,
    pub price_sparks: String,
    pub price_cgt: String,
    /// What each account received, from the event and not from the launcher's
    /// arithmetic.
    pub payouts: Vec<Payout>,
    pub tx_hash: String,
    pub block_hash: String,
}

/// What a buyer was shown when they pressed Buy: the price and the fingerprint
/// on their screen. A purchase is held to both, so nobody approves a price or
/// bytes they did not see.
#[derive(Debug, Clone, Copy)]
pub struct Seen<'a> {
    pub price_sparks: u128,
    /// The root the asset carried, as 64 hex characters.
    pub root: &'a str,
}

// ── Reading an asset's number ───────────────────────────────────────────────

/// An asset's number, read out of whatever a person pasted.
///
/// A buyer is given an asset's number by its holder, and what they are given is
/// usually the line the asset's own menu copies: `BLAKE3-256 <root> (asset
/// 4/0)`. So this takes `4/0`, `4 / 0`, `asset 4/0` and that whole line. When a
/// fingerprint comes with it, it is handed back too, so the launcher can say
/// whether the asset still carries it.
pub fn parse_asset_id(text: &str) -> QorResult<(TradeItem, Option<String>)> {
    let not_a_number = || {
        QorError::Qontrol(
            "That is not an asset's number. It is the two numbers on an asset's card, like 4/0, \
             or the reference its holder copied from the asset's menu."
                .into(),
        )
    };

    let mut tidy = text.trim().to_string();
    // `4 / 0` is the same number as `4/0`.
    while tidy.contains(" /") || tidy.contains("/ ") {
        tidy = tidy.replace(" /", "/").replace("/ ", "/");
    }

    let mut found: Vec<TradeItem> = Vec::new();
    let mut root = None;
    for token in tidy.split(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | ',' | ';')) {
        if token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit()) {
            root = Some(token.to_ascii_lowercase());
            continue;
        }
        let Some((collection, item)) = token.split_once('/') else {
            continue;
        };
        let (Ok(collection), Ok(item)) = (collection.parse::<u32>(), item.parse::<u32>()) else {
            continue;
        };
        let one = TradeItem { collection, item };
        if !found.contains(&one) {
            found.push(one);
        }
    }

    match found.as_slice() {
        [one] => Ok((*one, root)),
        [] => Err(not_a_number()),
        _ => Err(QorError::Qontrol(
            "That names more than one asset. Paste one asset's number, like 4/0.".into(),
        )),
    }
}

// ── What was read, and what follows from it ──────────────────────────────────

/// What an account can spend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Funds {
    free: u128,
    reserved: u128,
    frozen: u128,
}

impl Funds {
    /// What can leave the account while it stays open: `pallet-balances`'s
    /// reducible balance under `Preservation::Preserve`, which is how a sale
    /// takes the price from its buyer.
    fn spendable(&self, existential_deposit: u128) -> u128 {
        let untouchable = self
            .frozen
            .saturating_sub(self.reserved)
            .max(existential_deposit);
        self.free.saturating_sub(untouchable)
    }
}

/// Everything about one asset that a sale depends on, read at one block.
#[derive(Debug, Clone)]
struct SaleState {
    holder: Account,
    asset: OwnedAsset,
    derived_from: Option<(u32, u32)>,
    own_terms: Option<TermsRead>,
    source_terms: Option<TermsRead>,
    listing: Option<ListingRead>,
    existential_deposit: u128,
    /// The free balance of every account a sale of this asset could pay.
    free: Vec<(Account, u128)>,
    /// The account asking, and what it holds.
    viewer: Option<(Account, Funds)>,
}

fn shares(terms: &TermsRead) -> Vec<(Account, Permill)> {
    terms
        .recipients
        .iter()
        .map(|(who, parts)| (who.0, Permill::from_parts(*parts)))
        .collect()
}

/// A `Permill` as a person reads it, exactly: `100000` is `10%`, `25000` is
/// `2.5%`, `1` is `0.0001%`. Integer arithmetic only.
fn percent(parts_per_million: u32) -> String {
    let whole = parts_per_million / 10_000;
    let fraction = parts_per_million % 10_000;
    if fraction == 0 {
        return format!("{whole}%");
    }
    let digits = format!("{fraction:04}");
    format!("{whole}.{}%", digits.trim_end_matches('0'))
}

fn cgt_of(sparks: u128) -> String {
    cgt::format_cgt_grouped(sparks)
}

fn with_symbol(sparks: u128) -> String {
    format!("{} {}", cgt_of(sparks), cgt::SYMBOL)
}

impl SaleState {
    /// The listing, if someone could buy it: it exists and its seller still
    /// holds the asset.
    fn live_listing(&self) -> Option<&ListingRead> {
        self.listing
            .as_ref()
            .filter(|listing| listing.seller.0 == self.holder)
    }

    fn free_of(&self, account: &Account) -> u128 {
        self.free
            .iter()
            .find(|(known, _)| known == account)
            .map(|(_, free)| *free)
            .unwrap_or(0)
    }

    /// What a sale at `price` pays, to whom, with `buyer` buying.
    fn breakdown(&self, price: u128, buyer: Option<&Account>) -> Breakdown {
        let upstream = self.source_terms.as_ref().map(shares);
        let own = self.own_terms.as_ref().map(shares).unwrap_or_default();
        let source_share = self
            .source_terms
            .as_ref()
            .map(|terms| Permill::from_parts(terms.remix));
        let parts = split(price, source_share.zip(upstream.as_deref()), &own);

        let payout = |kind, who: &Account, share: Option<String>, amount: u128| Payout {
            kind,
            address: address_of(who),
            share,
            amount_sparks: amount.to_string(),
            amount_cgt: cgt_of(amount),
            to_buyer: buyer == Some(who),
        };

        let mut payouts = Vec::new();
        for (who, amount) in &parts.remix {
            payouts.push(payout(PayoutKind::Source, who, None, *amount));
        }
        for ((who, amount), (_, share)) in parts.royalties.iter().zip(own.iter()) {
            payouts.push(payout(
                PayoutKind::Royalty,
                who,
                Some(percent(share.deconstruct())),
                *amount,
            ));
        }
        payouts.push(payout(PayoutKind::Seller, &self.holder, None, parts.seller));

        // The chain pays each part in turn and refuses the whole sale at the
        // first its recipient cannot receive (`PaymentCannotBeReceived`): an
        // account that would be left holding less than the existential deposit.
        // A part of nothing, and a part to the buyer, move nothing. Walked in
        // the chain's order, because an account paid twice has its first part
        // by the time its second arrives.
        let mut held: Vec<(Account, u128)> = Vec::new();
        let mut blocked = None;
        for (who, amount) in parts
            .remix
            .iter()
            .chain(parts.royalties.iter())
            .chain(std::iter::once(&(self.holder, parts.seller)))
        {
            if *amount == 0 || buyer == Some(who) {
                continue;
            }
            let at = match held.iter().position(|(known, _)| known == who) {
                Some(at) => at,
                None => {
                    held.push((*who, self.free_of(who)));
                    held.len() - 1
                }
            };
            let after = held[at].1.saturating_add(*amount);
            if after < self.existential_deposit {
                blocked = Some(format!(
                    "A sale at this price cannot settle as things stand. {} is owed {} from it, \
                     and that account holds too little to stay open on that: an account needs \
                     {} to exist. The chain refuses the whole sale rather than paying everyone \
                     else. A higher price clears it, or that account receiving CGT first.",
                    address_of(who),
                    with_symbol(*amount),
                    with_symbol(self.existential_deposit),
                ));
                break;
            }
            held[at].1 = after;
        }

        Breakdown {
            price_sparks: price.to_string(),
            price_cgt: cgt_of(price),
            source: source_share
                .and(self.derived_from)
                .map(|(collection, item)| TradeItem { collection, item }),
            source_share: source_share.map(|share| percent(share.deconstruct())),
            payouts,
            blocked,
        }
    }

    /// Why `buyer` cannot buy this asset now, in words. `None` when they can.
    fn cannot_buy(&self, buyer: &Account, funds: &Funds) -> Option<String> {
        let Some(listing) = self.listing.as_ref() else {
            return Some("This asset is not listed for sale.".into());
        };
        if *buyer == self.holder {
            return Some(if listing.seller.0 == self.holder {
                "You hold this asset, and this is your own listing. To stop selling it, withdraw \
                 the listing."
                    .into()
            } else {
                "You hold this asset already.".into()
            });
        }
        if listing.seller.0 != self.holder {
            return Some(
                "This listing is void: the account that listed the asset no longer holds it, so \
                 nobody can buy it."
                    .into(),
            );
        }

        let breakdown = self.breakdown(listing.price, Some(buyer));
        if let Some(blocked) = breakdown.blocked {
            return Some(blocked);
        }

        // A part owed to the buyer themselves never leaves their account.
        let stays: u128 = breakdown
            .payouts
            .iter()
            .filter(|payout| payout.to_buyer)
            .filter_map(|payout| payout.amount_sparks.parse::<u128>().ok())
            .fold(0u128, u128::saturating_add);
        let leaves = listing.price.saturating_sub(stays);
        let spendable = funds.spendable(self.existential_deposit);
        if leaves > spendable {
            return Some(format!(
                "This account cannot cover it. Buying takes {} and the account has {} it can \
                 spend: it holds {}, and {} of that has to stay for the account to remain open.",
                with_symbol(leaves),
                with_symbol(spendable),
                with_symbol(funds.free),
                with_symbol(funds.free.saturating_sub(spendable)),
            ));
        }
        None
    }

    /// Why this purchase must not be put to `buyer`, given what they were
    /// shown. Everything in [`Self::cannot_buy`], and then the two things only a
    /// purchase can get wrong: a price, or a fingerprint, that is no longer the
    /// one that was on their screen.
    fn refuses(&self, buyer: &Account, funds: &Funds, seen: Seen<'_>) -> Option<String> {
        if let Some(reason) = self.cannot_buy(buyer, funds) {
            return Some(reason);
        }
        let price = self.live_listing().map(|listing| listing.price)?;
        if price != seen.price_sparks {
            return Some(format!(
                "The price changed since you looked: it was {} and it is {} now. Nothing was \
                 sent.",
                with_symbol(seen.price_sparks),
                with_symbol(price)
            ));
        }
        if !self.asset.current.root.eq_ignore_ascii_case(seen.root) {
            return Some(
                "This asset was revised since you looked: it points at different content now. \
                 Nothing was sent."
                    .into(),
            );
        }
        None
    }

    fn view(self, pasted_root: Option<&str>) -> SaleView {
        let terms_view = |terms: &TermsRead| TermsView {
            recipients: terms
                .recipients
                .iter()
                .map(|(who, parts)| ShareView {
                    address: address_of(&who.0),
                    share: percent(*parts),
                })
                .collect(),
            remix: percent(terms.remix),
        };

        let buyer = self.viewer.as_ref().map(|(account, _)| account);
        let breakdown = self
            .live_listing()
            .map(|listing| self.breakdown(listing.price, buyer));
        let cannot_buy = self
            .viewer
            .as_ref()
            .and_then(|(account, funds)| self.cannot_buy(account, funds));

        SaleView {
            holder: address_of(&self.holder),
            held_by_viewer: buyer == Some(&self.holder),
            derived_from: self
                .derived_from
                .map(|(collection, item)| TradeItem { collection, item }),
            terms: self.own_terms.as_ref().map(terms_view),
            source_terms: self.source_terms.as_ref().map(terms_view),
            breakdown,
            viewer_free_cgt: self
                .viewer
                .as_ref()
                .map(|(_, funds)| cgt_of(funds.spendable(self.existential_deposit))),
            cannot_buy,
            pasted_root_matches: pasted_root
                .map(|root| root.eq_ignore_ascii_case(&self.asset.current.root)),
            asset: self.asset,
        }
    }
}

fn no_such_asset(collection: u32, item: u32) -> QorError {
    QorError::Qontrol(format!(
        "There is no DRC-369 asset {collection}/{item} on this chain. Check the number with \
         whoever gave it to you."
    ))
}

/// The chain's own refusal of a sale, a listing or a withdrawal, in the words a
/// person was owed.
///
/// Everything here is also checked before anyone is asked, so reaching the
/// chain's refusal means something changed in between: a price raised, a
/// listing withdrawn, an asset sent away. The transaction's hash is kept, since
/// it is the one thing a person can look up.
pub(super) fn in_words(error: QorError) -> QorError {
    let QorError::Rpc(text) = &error else {
        return error;
    };
    let reason = if text.contains("PriceAboveLimit") {
        "The price was raised before your purchase settled, so nothing moved. Look the asset up \
         again to see what it costs now."
    } else if text.contains("NotListed") {
        "The listing was withdrawn before this settled, so nothing moved."
    } else if text.contains("ListingStale") {
        "The seller no longer holds this asset, so the listing is void and nothing moved."
    } else if text.contains("OwnListing") {
        "This is your own listing. The seller cannot buy their own asset."
    } else if text.contains("PaymentCannotBeReceived") {
        "One of the accounts this sale pays cannot receive its part, so the chain refused the \
         whole sale and nothing moved."
    } else if text.contains("NotOwner") {
        "This account does not hold that asset any more, so nothing changed."
    } else if text.contains("ZeroPrice") {
        "A price of nothing is a gift, and a gift is a trade, not a sale."
    } else if text.contains("FundsUnavailable") || text.contains("Funds are unavailable") {
        "This account could not cover the price and stay open, so nothing moved."
    } else {
        return error;
    };
    // "the purchase 0x… was finalised in 0x… but failed: …" names the
    // transaction before its first colon.
    let which = text
        .split_once(" was finalised")
        .map(|(what, _)| format!(" ({what})"))
        .unwrap_or_default();
    QorError::Qontrol(format!("{reason}{which}"))
}

impl ChainClient {
    /// An asset's listing, if it has one, for whoever holds it.
    pub(super) async fn listing(
        &self,
        connection: &Connection,
        holder: Account,
        collection: u32,
        item: u32,
    ) -> QorResult<Option<ListingView>> {
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let Some(listing) = at
            .storage()
            .try_fetch(
                dynamic::storage::<(u32, u32), ListingRead>("Drc369Royalties", "Listings"),
                (collection, item),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the asset's listing: {e}")))?
        else {
            return Ok(None);
        };
        let listing = listing
            .decode()
            .map_err(|e| QorError::Rpc(format!("the listing could not be decoded: {e}")))?;
        Ok(Some(ListingView {
            seller: address_of(&listing.seller.0),
            price_sparks: listing.price.to_string(),
            price_cgt: cgt_of(listing.price),
            void: listing.seller.0 != holder,
        }))
    }

    /// Everything a sale of one asset depends on, read from chain storage.
    /// `None` when no DRC-369 asset has that number.
    async fn sale_state(
        &self,
        connection: &Connection,
        collection: u32,
        item: u32,
        viewer: Option<Account>,
    ) -> QorResult<Option<SaleState>> {
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let storage = at.storage();

        let Some(held) = storage
            .try_fetch(
                dynamic::storage::<(u32, u32), ItemRead>("Nfts", "Item"),
                (collection, item),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read who holds the asset: {e}")))?
        else {
            return Ok(None);
        };
        let holder = held
            .decode()
            .map_err(|e| QorError::Rpc(format!("the asset's holder could not be decoded: {e}")))?
            .owner
            .0;

        // An item without a DRC-369 record is not a DRC-369 asset.
        let Some(asset) = self.asset(connection, holder, collection, item).await? else {
            return Ok(None);
        };

        let derived_from = match storage
            .try_fetch(
                dynamic::storage::<(u32, u32), SourceRead>("Drc369", "Assets"),
                (collection, item),
            )
            .await
            .map_err(|e| QorError::Rpc(format!("could not read the asset: {e}")))?
        {
            Some(record) => {
                record
                    .decode()
                    .map_err(|e| QorError::Rpc(format!("the asset could not be decoded: {e}")))?
                    .derived_from
            }
            None => None,
        };

        let terms_entry =
            || dynamic::storage::<(u32, u32), TermsRead>("Drc369Royalties", "RoyaltyTerms");
        // The asset's own terms, then its source's: one level, as the chain
        // reads them (ADR-061 decision 4).
        let mut terms: [Option<TermsRead>; 2] = [None, None];
        for (slot, wanted) in [Some((collection, item)), derived_from]
            .into_iter()
            .enumerate()
        {
            let Some(key) = wanted else { continue };
            let Some(value) = storage
                .try_fetch(terms_entry(), key)
                .await
                .map_err(|e| QorError::Rpc(format!("could not read the royalty terms: {e}")))?
            else {
                continue;
            };
            terms[slot] = Some(value.decode().map_err(|e| {
                QorError::Rpc(format!("the royalty terms could not be decoded: {e}"))
            })?);
        }
        let [own_terms, source_terms] = terms;

        let listing =
            match storage
                .try_fetch(
                    dynamic::storage::<(u32, u32), ListingRead>("Drc369Royalties", "Listings"),
                    (collection, item),
                )
                .await
                .map_err(|e| QorError::Rpc(format!("could not read the asset's listing: {e}")))?
            {
                Some(value) => Some(value.decode().map_err(|e| {
                    QorError::Rpc(format!("the listing could not be decoded: {e}"))
                })?),
                None => None,
            };

        let existential_deposit = at
            .constants()
            .entry(dynamic::constant::<u128>("Balances", "ExistentialDeposit"))
            .map_err(|e| QorError::Rpc(format!("could not read the existential deposit: {e}")))?;

        // Every account a sale could pay, once each, and the account asking.
        let mut payees: Vec<Account> = vec![holder];
        for terms in [&own_terms, &source_terms].into_iter().flatten() {
            for (who, _) in &terms.recipients {
                if !payees.contains(&who.0) {
                    payees.push(who.0);
                }
            }
        }
        let account_entry = || dynamic::storage::<(Account,), AccountRead>("System", "Account");
        let mut free = Vec::with_capacity(payees.len());
        for payee in payees {
            let funds = storage
                .fetch(account_entry(), (payee,))
                .await
                .map_err(|e| QorError::Rpc(format!("could not read an account: {e}")))?
                .decode()
                .map_err(|e| QorError::Rpc(format!("an account could not be decoded: {e}")))?;
            free.push((payee, funds.data.free));
        }
        let viewer = match viewer {
            Some(account) => {
                let funds = storage
                    .fetch(account_entry(), (account,))
                    .await
                    .map_err(|e| QorError::Rpc(format!("could not read the account: {e}")))?
                    .decode()
                    .map_err(|e| QorError::Rpc(format!("the account could not be decoded: {e}")))?
                    .data;
                Some((
                    account,
                    Funds {
                        free: funds.free,
                        reserved: funds.reserved,
                        frozen: funds.frozen,
                    },
                ))
            }
            None => None,
        };

        Ok(Some(SaleState {
            holder,
            asset,
            derived_from,
            own_terms,
            source_terms,
            listing,
            existential_deposit,
            free,
            viewer,
        }))
    }

    /// One asset, whoever holds it: what it is, whether it is for sale, and
    /// what a sale would pay. `viewer` is the account that might buy it.
    ///
    /// `asset` is whatever was pasted: see [`parse_asset_id`].
    pub async fn sale(&self, asset: &str, viewer: Option<&str>) -> QorResult<SaleView> {
        let (id, pasted_root) = parse_asset_id(asset)?;
        let viewer = viewer.map(normalise_address).transpose()?;
        let connection = self.connected().await?;
        let state = self
            .sale_state(&connection, id.collection, id.item, viewer)
            .await?
            .ok_or_else(|| no_such_asset(id.collection, id.item))?;
        Ok(state.view(pasted_root.as_deref()))
    }

    /// What a sale of an asset at `price_sparks` would pay, before it is listed.
    pub async fn sale_preview(
        &self,
        collection: u32,
        item: u32,
        price_sparks: u128,
    ) -> QorResult<Breakdown> {
        if price_sparks == 0 {
            return Err(zero_price());
        }
        let connection = self.connected().await?;
        let state = self
            .sale_state(&connection, collection, item, None)
            .await?
            .ok_or_else(|| no_such_asset(collection, item))?;
        Ok(state.breakdown(price_sparks, None))
    }

    /// Offer an asset for sale at a price in CGT, or change the price it is
    /// offered at.
    ///
    /// **The listing is public once it is in a block**, and anyone who knows the
    /// asset's number can buy it at that price until it is withdrawn. The dialog
    /// says so, and says what a sale at that price would pay.
    pub async fn list(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        collection: u32,
        item: u32,
        price_sparks: u128,
    ) -> QorResult<ListReceipt> {
        let seller = normalise_address(from)?;
        if price_sparks == 0 {
            return Err(zero_price());
        }

        let connection = self.connected().await?;
        let state = self
            .sale_state(&connection, collection, item, None)
            .await?
            .filter(|state| state.holder == seller)
            .ok_or_else(|| {
                QorError::Qontrol(format!(
                    "this account holds no DRC-369 asset {collection}/{item}"
                ))
            })?;
        if state
            .live_listing()
            .is_some_and(|listing| listing.price == price_sparks)
        {
            return Err(QorError::Qontrol(format!(
                "{} is already listed at {}. Nothing was sent.",
                named(&state.asset),
                with_symbol(price_sparks)
            )));
        }
        let breakdown = state.breakdown(price_sparks, None);
        if let Some(blocked) = &breakdown.blocked {
            return Err(QorError::Qontrol(blocked.clone()));
        }

        let call =
            dynamic::transaction("Drc369Royalties", "list", (collection, item, price_sparks));
        let was = state.live_listing().map(|listing| listing.price);
        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    list_prompt(&state.asset, &breakdown, was, chain_name, endpoint)
                },
                || Ok(()),
                "listing",
            )
            .await
            .map_err(in_words)?;

        // The chain's answer, not the launcher's.
        if !finalised
            .events
            .iter()
            .filter_map(Result::ok)
            .any(|event| event.pallet_name() == "Drc369Royalties" && event.event_name() == "Listed")
        {
            return Err(QorError::Rpc(format!(
                "the listing {} was finalised but the chain reported no Listed event",
                finalised.tx_hash
            )));
        }

        Ok(ListReceipt {
            collection,
            item,
            price_sparks: price_sparks.to_string(),
            price_cgt: cgt_of(price_sparks),
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }

    /// Withdraw an asset's listing: its seller or the asset's holder, or anyone
    /// once the listing is void.
    pub async fn unlist(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        collection: u32,
        item: u32,
    ) -> QorResult<UnlistReceipt> {
        let who = normalise_address(from)?;
        let connection = self.connected().await?;
        let state = self
            .sale_state(&connection, collection, item, None)
            .await?
            .ok_or_else(|| no_such_asset(collection, item))?;
        let Some(listing) = state.listing.as_ref() else {
            return Err(QorError::Qontrol(format!(
                "{} is not listed, so there is nothing to withdraw.",
                named(&state.asset)
            )));
        };
        let void = listing.seller.0 != state.holder;
        if !void && who != state.holder {
            return Err(QorError::Qontrol(
                "Only the account that holds this asset can withdraw its listing.".into(),
            ));
        }

        let call = dynamic::transaction("Drc369Royalties", "unlist", (collection, item));
        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    unlist_prompt(&state.asset, listing.price, void, chain_name, endpoint)
                },
                || Ok(()),
                "withdrawal",
            )
            .await
            .map_err(in_words)?;

        if !finalised.events.iter().filter_map(Result::ok).any(|event| {
            event.pallet_name() == "Drc369Royalties" && event.event_name() == "Unlisted"
        }) {
            return Err(QorError::Rpc(format!(
                "the withdrawal {} was finalised but the chain reported no Unlisted event",
                finalised.tx_hash
            )));
        }

        Ok(UnlistReceipt {
            collection,
            item,
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }

    /// Buy a listed asset.
    ///
    /// `seen` is what the buyer was shown. If the price or the fingerprint has
    /// changed, nobody is asked: a price is not approved by someone who looked
    /// at a different one, and a revisable asset is not bought by someone who
    /// looked at different bytes. The price they saw is also the most the chain
    /// may take (`max_price`), so a price raised between the dialog and the
    /// block moves nothing.
    pub async fn buy(
        &self,
        vault: &Vault,
        confirm: &dyn Confirm,
        from: &str,
        collection: u32,
        item: u32,
        seen: Seen<'_>,
    ) -> QorResult<SaleReceipt> {
        let buyer = normalise_address(from)?;
        let connection = self.connected().await?;
        let state = self
            .sale_state(&connection, collection, item, Some(buyer))
            .await?
            .ok_or_else(|| no_such_asset(collection, item))?;

        let funds = state
            .viewer
            .as_ref()
            .map(|(_, funds)| *funds)
            .ok_or_else(|| QorError::Internal("the buyer's balance was not read".into()))?;
        if let Some(reason) = state.refuses(&buyer, &funds, seen) {
            return Err(QorError::Qontrol(reason));
        }
        let listing = state
            .live_listing()
            .ok_or_else(|| QorError::Internal("a sale without a listing".into()))?;

        let breakdown = state.breakdown(listing.price, Some(&buyer));
        let call =
            dynamic::transaction("Drc369Royalties", "buy", (collection, item, listing.price));
        let finalised = self
            .sign_and_finalise(
                &connection,
                vault,
                confirm,
                from,
                &call,
                |chain_name, endpoint| {
                    buy_prompt(
                        &state.asset,
                        &breakdown,
                        &address_of(&buyer),
                        chain_name,
                        endpoint,
                    )
                },
                || Ok(()),
                "purchase",
            )
            .await
            .map_err(in_words)?;

        // What each account received is the chain's `Sold` event, never the
        // launcher's own arithmetic.
        let sold = finalised
            .events
            .iter()
            .filter_map(Result::ok)
            .find(|event| event.pallet_name() == "Drc369Royalties" && event.event_name() == "Sold")
            .ok_or_else(|| {
                QorError::Rpc(format!(
                    "the purchase {} was finalised but the chain reported no Sold event",
                    finalised.tx_hash
                ))
            })?
            .decode_fields_unchecked_as::<SoldRead>()
            .map_err(|e| QorError::Rpc(format!("the Sold event could not be read: {e}")))?;
        if (sold.collection, sold.item, sold.to.0) != (collection, item, buyer) {
            return Err(QorError::Rpc(format!(
                "the purchase {} was finalised, but the chain's Sold event is for another sale",
                finalised.tx_hash
            )));
        }

        let paid = |kind, who: &AccountId32, amount: u128| Payout {
            kind,
            address: address_of(&who.0),
            share: None,
            amount_sparks: amount.to_string(),
            amount_cgt: cgt_of(amount),
            to_buyer: who.0 == buyer,
        };
        let mut payouts: Vec<Payout> = sold
            .remix
            .iter()
            .map(|(who, amount)| paid(PayoutKind::Source, who, *amount))
            .chain(
                sold.royalties
                    .iter()
                    .map(|(who, amount)| paid(PayoutKind::Royalty, who, *amount)),
            )
            .collect();
        payouts.push(paid(PayoutKind::Seller, &sold.from, sold.seller_received));

        Ok(SaleReceipt {
            collection,
            item,
            name: state.asset.name.clone(),
            seller: address_of(&sold.from.0),
            buyer: address_of(&sold.to.0),
            price_sparks: sold.price.to_string(),
            price_cgt: cgt_of(sold.price),
            payouts,
            tx_hash: finalised.tx_hash,
            block_hash: finalised.block_hash,
        })
    }
}

fn zero_price() -> QorError {
    QorError::BadAmount(
        "a listing needs a price above zero. To give an asset away, use Trade.".into(),
    )
}

/// An asset, as a sentence names it.
fn named(asset: &OwnedAsset) -> String {
    if asset.name.is_empty() {
        format!("Asset {}/{}", asset.collection, asset.item)
    } else {
        format!("\"{}\"", asset.name)
    }
}

/// The parts of a sale, as lines of a dialog: every recipient by their whole
/// address, because this is the one place a person can check them.
fn payout_lines(breakdown: &Breakdown, seller: &str) -> String {
    let mut lines = Vec::new();
    let part = |payout: &Payout| {
        let share = payout
            .share
            .as_deref()
            .map(|share| format!(" ({share})"))
            .unwrap_or_default();
        let stays = if payout.to_buyer {
            " — this is your own account, so it stays with you"
        } else {
            ""
        };
        format!(
            "    {}{share}: {} {}{stays}",
            payout.address,
            payout.amount_cgt,
            cgt::SYMBOL
        )
    };

    let of = |kind| breakdown.payouts.iter().filter(move |p| p.kind == kind);
    if let (Some(source), Some(share)) = (breakdown.source, breakdown.source_share.as_deref()) {
        lines.push(format!(
            "  To the work it was remixed from (asset {}/{}), {share} of the price:",
            source.collection, source.item
        ));
        lines.extend(of(PayoutKind::Source).map(part));
    }
    if of(PayoutKind::Royalty).next().is_some() {
        lines.push(if breakdown.source.is_some() {
            "  Royalties, each a share of what is left:".to_string()
        } else {
            "  Royalties:".to_string()
        });
        lines.extend(of(PayoutKind::Royalty).map(part));
    }
    for payout in of(PayoutKind::Seller) {
        lines.push(format!("  {seller}: {} {}", payout.amount_cgt, cgt::SYMBOL));
    }
    if breakdown.payouts.len() == 1 {
        lines.push("  No royalty is owed on this asset, so the seller receives all of it.".into());
    }
    lines.join("\n")
}

/// What the person listing an asset is shown.
fn list_prompt(
    asset: &OwnedAsset,
    breakdown: &Breakdown,
    was: Option<u128>,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    let changing = match was {
        Some(was) => format!("It is listed at {} now.\n", with_symbol(was)),
        None => String::new(),
    };
    Prompt {
        title: if was.is_some() {
            "Change this asset's price".into()
        } else {
            "List this asset for sale".into()
        },
        body: format!(
            "List {name} for {price} {symbol}\n\n\
             Asset: {collection}/{item}\n\
             Fingerprint: {algo} {root}\n\
             {changing}\n\
             What a sale at this price pays, as the royalty terms stand now:\n\
             {parts}\n\n\
             The listing is public. Anyone who knows this asset's number can buy it at this \
             price until you withdraw the listing, and a sale hands the asset to the buyer in the \
             same transaction. A sale cannot be undone.\n\n\
             Seller: {seller}\nChain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this listing with your key and sends it.",
            name = named(asset),
            price = breakdown.price_cgt,
            symbol = cgt::SYMBOL,
            collection = asset.collection,
            item = asset.item,
            algo = asset.current.algo,
            root = asset.current.root,
            parts = payout_lines(breakdown, "You, the seller"),
            seller = seller_of(breakdown),
        ),
        approve: if was.is_some() {
            "Change the price".into()
        } else {
            "List for sale".into()
        },
    }
}

/// The seller's address, from the one payout every breakdown ends with.
fn seller_of(breakdown: &Breakdown) -> &str {
    breakdown
        .payouts
        .iter()
        .find(|payout| payout.kind == PayoutKind::Seller)
        .map(|payout| payout.address.as_str())
        .unwrap_or_default()
}

/// What the person withdrawing a listing is shown.
fn unlist_prompt(
    asset: &OwnedAsset,
    price: u128,
    void: bool,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    let what = if void {
        "This listing is void: the account that made it no longer holds the asset, and nobody \
         can buy from it. Clearing it removes it from the chain."
    } else {
        "Once this is in a block nobody can buy the asset. It stays in your Inventory, and you \
         can list it again at any time."
    };
    Prompt {
        title: "Withdraw this listing".into(),
        body: format!(
            "Withdraw the listing of {name}\n\n\
             Asset: {collection}/{item}\n\
             Listed at: {price}\n\n\
             {what}\n\n\
             Chain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this with your key and sends it.",
            name = named(asset),
            collection = asset.collection,
            item = asset.item,
            price = with_symbol(price),
        ),
        approve: "Withdraw".into(),
    }
}

/// What the person buying an asset is shown. It says what they pay, who is paid
/// from it, that no more can be taken, and that it cannot be undone.
fn buy_prompt(
    asset: &OwnedAsset,
    breakdown: &Breakdown,
    buyer: &str,
    chain_name: &str,
    endpoint: &str,
) -> Prompt {
    let revisable = if asset.revisable {
        "\nThis asset is still revisable: whoever holds it can point it at different content \
         until it is made permanent. Once you hold it, that is you.\n"
    } else {
        ""
    };
    Prompt {
        title: "Buy this asset".into(),
        body: format!(
            "Buy {name} for {price} {symbol}\n\n\
             Asset: {collection}/{item}\n\
             Fingerprint: {algo} {root}\n\
             {revisable}\n\
             You pay {price} {symbol}, and the chain pays it out as:\n\
             {parts}\n\n\
             You will not pay more than {price} {symbol}: if the price is raised before this \
             settles, nothing moves.\n\n\
             This cannot be undone. Once it is in a block the CGT is theirs and the asset is \
             yours. All of it happens or none of it does.\n\n\
             Buyer: {buyer}\nChain: {chain_name}\nNode: {endpoint}\n\n\
             Approving signs this purchase with your key and sends it.",
            name = named(asset),
            price = breakdown.price_cgt,
            symbol = cgt::SYMBOL,
            collection = asset.collection,
            item = asset.item,
            algo = asset.current.algo,
            root = asset.current.root,
            parts = payout_lines(breakdown, &format!("The seller, {}", seller_of(breakdown))),
        ),
        approve: "Buy".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::assets::ContentView;
    use super::*;
    use crate::cgt::SPARKS_PER_CGT;
    use crate::content::ContentRef;
    use crate::testing::Scripted;

    const NOWHERE: &str = "ws://127.0.0.1:1";
    const ED: u128 = 100 * SPARKS_PER_CGT;

    fn ppm(parts: u32) -> Permill {
        Permill::from_parts(parts)
    }

    fn percent_of(value: u32) -> Permill {
        Permill::from_percent(value)
    }

    // ── The arithmetic, on the pallet's own vectors ─────────────────────────
    //
    // These four are `chain/pallets/drc369-royalties/src/tests.rs`'s tests of
    // `split`, with the same inputs and the same expected parts. If the pallet's
    // rule changes, its tests change, and these must change with them.

    #[test]
    fn a_split_always_sums_to_the_price() {
        let source = [(1u8, ppm(333_333)), (2, ppm(1)), (3, ppm(250_000))];
        let own = [(4u8, ppm(123_457)), (5, ppm(876_543))];
        for price in [0u128, 1, 7, 999, 1_000_001, 10u128.pow(20), 10u128.pow(32)] {
            for remix in [Permill::zero(), ppm(1), percent_of(17), Permill::one()] {
                let parts = split(price, Some((remix, &source[..])), &own);
                let total: u128 = parts
                    .remix
                    .iter()
                    .chain(parts.royalties.iter())
                    .map(|(_, amount)| *amount)
                    .sum::<u128>()
                    + parts.seller;
                assert_eq!(total, price, "price {price}, remix {remix:?}");
            }
        }
    }

    #[test]
    fn the_remix_pool_is_divided_in_proportion_to_the_sources_shares() {
        // A 20% remix share of 10,000 is a pool of 2,000; shares of 30% and 10%
        // divide it three to one. The remix's own 50% is of the 8,000 left.
        let parts = split(
            10_000,
            Some((
                percent_of(20),
                &[(1u8, percent_of(30)), (2, percent_of(10))][..],
            )),
            &[(3u8, percent_of(50))],
        );
        assert_eq!(parts.remix, vec![(1, 1_500), (2, 500)]);
        assert_eq!(parts.royalties, vec![(3, 4_000)]);
        assert_eq!(parts.seller, 4_000);
    }

    #[test]
    fn rounding_goes_to_the_seller_and_never_above_a_share() {
        // 1% of 99 is 0.99: rounded down to nothing, and the seller keeps it.
        let parts = split(99, None, &[(1u8, percent_of(1))]);
        assert_eq!(parts.royalties, vec![(1, 0)]);
        assert_eq!(parts.seller, 99);

        // A pool of 10 divided three ways gets 3 each; the 1 left is the seller's.
        let thirds = [(1u8, ppm(1)), (2, ppm(1)), (3, ppm(1))];
        let parts = split(100, Some((percent_of(10), &thirds[..])), &[]);
        assert_eq!(parts.remix, vec![(1, 3), (2, 3), (3, 3)]);
        assert_eq!(parts.seller, 91);
    }

    /// AGENTS.md §5: every path that forms a money intermediate pins the largest
    /// one. `split` forms `share × price` inside `Permill::mul_floor` and
    /// `pool × part` inside `multiply_by_rational_with_rounding`, both of which
    /// widen internally; here both run at the largest price a `u128` can carry,
    /// with every share at its most, and nothing overflows or panics.
    #[test]
    fn the_largest_intermediate_cannot_overflow() {
        let full_supply = 10u128.pow(32);
        for price in [full_supply, u128::MAX, u128::MAX - 1] {
            let parts = split(
                price,
                Some((Permill::one(), &[(1u8, ppm(999_999)), (2, ppm(1))][..])),
                &[(3u8, Permill::one())],
            );
            let upstream: u128 = parts.remix.iter().map(|(_, a)| *a).sum();
            assert!(upstream <= price);
            // A 100% remix share leaves at most the rounding for the rest.
            assert!(price - upstream < 2);
            let total =
                upstream + parts.royalties.iter().map(|(_, a)| *a).sum::<u128>() + parts.seller;
            assert_eq!(total, price);
        }

        // And a whole-price royalty with no remix pays the whole price.
        let parts = split(u128::MAX, None, &[(1u8, Permill::one())]);
        assert_eq!(parts.royalties, vec![(1, u128::MAX)]);
        assert_eq!(parts.seller, 0);
    }

    // ── What follows from what was read ─────────────────────────────────────

    const SELLER: Account = [0x11; 32];
    const BUYER: Account = [0x22; 32];
    const CREATOR: Account = [0x33; 32];
    const SOURCE: Account = [0x44; 32];

    fn asset(collection: u32, item: u32, name: &str) -> OwnedAsset {
        OwnedAsset {
            collection,
            item,
            name: name.into(),
            origin: ContentView::from(&ContentRef::of(b"one")),
            current: ContentView::from(&ContentRef::of(b"two")),
            commit: None,
            revisable: true,
            listing: None,
        }
    }

    fn terms(recipients: &[(Account, u32)], remix: u32) -> TermsRead {
        TermsRead {
            recipients: recipients
                .iter()
                .map(|(who, parts)| (AccountId32(*who), *parts))
                .collect(),
            remix,
        }
    }

    fn rich() -> Funds {
        Funds {
            free: 10_000 * SPARKS_PER_CGT,
            reserved: 0,
            frozen: 0,
        }
    }

    /// A remix held by SELLER, listed at `price`: its source owes SOURCE a 5%
    /// remix share, and its own terms pay CREATOR 10%.
    fn remix_listed_at(price: u128) -> SaleState {
        SaleState {
            holder: SELLER,
            asset: asset(7, 2, "a-remix"),
            derived_from: Some((3, 1)),
            own_terms: Some(terms(&[(CREATOR, 100_000)], 0)),
            source_terms: Some(terms(&[(SOURCE, 200_000)], 50_000)),
            listing: Some(ListingRead {
                seller: AccountId32(SELLER),
                price,
            }),
            existential_deposit: ED,
            free: vec![(SELLER, 500 * SPARKS_PER_CGT), (CREATOR, ED), (SOURCE, ED)],
            viewer: Some((BUYER, rich())),
        }
    }

    fn amounts(breakdown: &Breakdown) -> Vec<(PayoutKind, String, String)> {
        breakdown
            .payouts
            .iter()
            .map(|p| (p.kind, p.address.clone(), p.amount_cgt.clone()))
            .collect()
    }

    /// The breakdown is the chain's order — source, royalties, seller — in CGT a
    /// person can read, and its parts are the price.
    #[test]
    fn a_breakdown_names_every_part_of_the_price_in_the_chains_order() {
        let state = remix_listed_at(3_000 * SPARKS_PER_CGT);
        let breakdown = state.breakdown(3_000 * SPARKS_PER_CGT, Some(&BUYER));

        assert_eq!(breakdown.price_cgt, "3,000.00");
        assert_eq!(
            breakdown.source,
            Some(TradeItem {
                collection: 3,
                item: 1
            })
        );
        assert_eq!(breakdown.source_share.as_deref(), Some("5%"));
        // 5% of 3,000 is 150 upstream; 10% of the 2,850 left is 285; the rest.
        assert_eq!(
            amounts(&breakdown),
            vec![
                (PayoutKind::Source, address_of(&SOURCE), "150.00".into()),
                (PayoutKind::Royalty, address_of(&CREATOR), "285.00".into()),
                (PayoutKind::Seller, address_of(&SELLER), "2,565.00".into()),
            ]
        );
        assert_eq!(breakdown.payouts[1].share.as_deref(), Some("10%"));
        assert_eq!(breakdown.payouts[2].share, None);
        let total: u128 = breakdown
            .payouts
            .iter()
            .map(|p| p.amount_sparks.parse::<u128>().unwrap())
            .sum();
        assert_eq!(total, 3_000 * SPARKS_PER_CGT);
        assert_eq!(breakdown.blocked, None);

        // An asset with no terms and no source pays its seller everything.
        let plain = SaleState {
            derived_from: None,
            own_terms: None,
            source_terms: None,
            ..state
        };
        let breakdown = plain.breakdown(SPARKS_PER_CGT / 2, None);
        assert_eq!(
            amounts(&breakdown),
            vec![(PayoutKind::Seller, address_of(&SELLER), "0.50".into())]
        );
        assert_eq!(breakdown.source, None);
    }

    /// A source with no terms is owed nothing, and is not named as if it were.
    #[test]
    fn a_remix_of_a_source_without_terms_names_no_source() {
        let state = SaleState {
            source_terms: None,
            ..remix_listed_at(1_000 * SPARKS_PER_CGT)
        };
        let breakdown = state.breakdown(1_000 * SPARKS_PER_CGT, None);
        assert_eq!(breakdown.source, None);
        assert_eq!(breakdown.source_share, None);
        assert_eq!(breakdown.payouts.len(), 2);
        assert_eq!(breakdown.payouts[0].amount_cgt, "100.00");
        assert_eq!(breakdown.payouts[1].amount_cgt, "900.00");
    }

    /// The chain refuses a sale whose part would leave its recipient below the
    /// existential deposit; the launcher says so before anyone is asked, and
    /// names the account and the amount.
    #[test]
    fn a_part_its_recipient_cannot_receive_blocks_the_sale_in_words() {
        // CREATOR does not exist, and 10% of 500 is 50, below 100.
        let mut state = remix_listed_at(500 * SPARKS_PER_CGT);
        state.source_terms = None;
        state.free = vec![(SELLER, 500 * SPARKS_PER_CGT)];

        let blocked = state
            .breakdown(500 * SPARKS_PER_CGT, Some(&BUYER))
            .blocked
            .expect("a part that cannot be received");
        assert!(blocked.contains(&address_of(&CREATOR)), "{blocked}");
        assert!(blocked.contains("50.00 CGT"), "{blocked}");
        assert!(blocked.contains("100.00 CGT"), "{blocked}");
        assert_eq!(state.cannot_buy(&BUYER, &rich()), Some(blocked));

        // At a price where the part reaches the existential deposit, it clears.
        assert_eq!(
            state
                .breakdown(1_000 * SPARKS_PER_CGT, Some(&BUYER))
                .blocked,
            None
        );
        // A part of nothing moves nothing, so it blocks nothing.
        let dust = state.breakdown(9, Some(&BUYER));
        assert_eq!(dust.payouts[0].amount_sparks, "0");
        assert_eq!(dust.blocked, None);

        // An account paid twice has its first part when its second arrives, as
        // on chain: CREATOR, who does not exist, is opened by 150 from the
        // source's terms, and the 28.5 its own royalty adds is then fine.
        state.source_terms = Some(terms(&[(CREATOR, 200_000)], 500_000));
        state.own_terms = Some(terms(&[(CREATOR, 190_000)], 0));
        let twice = state.breakdown(300 * SPARKS_PER_CGT, Some(&BUYER));
        assert_eq!(twice.payouts[0].amount_cgt, "150.00");
        assert_eq!(twice.payouts[1].amount_cgt, "28.50");
        assert_eq!(twice.blocked, None);
    }

    /// A part owed to the buyer themselves never leaves their account, so it is
    /// marked, it cannot block the sale, and it is not counted against them.
    #[test]
    fn a_part_owed_to_the_buyer_stays_with_them() {
        let mut state = remix_listed_at(1_000 * SPARKS_PER_CGT);
        state.source_terms = None;
        let breakdown = state.breakdown(1_000 * SPARKS_PER_CGT, Some(&CREATOR));
        assert!(breakdown.payouts[0].to_buyer);
        assert!(!breakdown.payouts[1].to_buyer);

        // 900 leaves the account, not 1,000: with 1,000 free and 100 to keep,
        // the buyer who is owed the royalty can pay and a stranger cannot.
        let exactly = Funds {
            free: 1_000 * SPARKS_PER_CGT,
            reserved: 0,
            frozen: 0,
        };
        assert_eq!(state.cannot_buy(&CREATOR, &exactly), None);
        let refused = state.cannot_buy(&BUYER, &exactly).expect("too little");
        assert!(refused.contains("1,000.00 CGT"), "{refused}");
        assert!(refused.contains("900.00 CGT"), "{refused}");
    }

    /// Every reason a purchase would be refused is said in words, and the first
    /// one that applies is the one given.
    #[test]
    fn what_stops_a_purchase_is_said_in_words() {
        let listed = remix_listed_at(1_000 * SPARKS_PER_CGT);

        assert_eq!(listed.cannot_buy(&BUYER, &rich()), None);

        let unlisted = SaleState {
            listing: None,
            ..listed.clone()
        };
        assert!(unlisted
            .cannot_buy(&BUYER, &rich())
            .unwrap()
            .contains("not listed"));

        assert!(listed
            .cannot_buy(&SELLER, &rich())
            .unwrap()
            .contains("your own listing"));

        // Listed by SELLER, since sent to CREATOR: void for everyone.
        let void = SaleState {
            holder: CREATOR,
            ..listed.clone()
        };
        assert!(void.live_listing().is_none());
        assert!(void.cannot_buy(&BUYER, &rich()).unwrap().contains("void"));
        assert!(void
            .cannot_buy(&CREATOR, &rich())
            .unwrap()
            .contains("hold this asset already"));

        // The price, and the existential deposit on top of it.
        let short = Funds {
            free: 1_050 * SPARKS_PER_CGT,
            reserved: 0,
            frozen: 0,
        };
        let refused = listed.cannot_buy(&BUYER, &short).unwrap();
        assert!(refused.contains("1,000.00 CGT"), "{refused}");
        assert!(refused.contains("950.00 CGT"), "{refused}");
        assert!(refused.contains("100.00 CGT"), "{refused}");
        let enough = Funds {
            free: 1_100 * SPARKS_PER_CGT,
            ..short
        };
        assert_eq!(listed.cannot_buy(&BUYER, &enough), None);
    }

    /// Nobody is asked to approve a price they did not see, or bytes they did not
    /// see: what was on the buyer's screen is held against the chain.
    #[test]
    fn a_purchase_is_refused_when_what_was_seen_has_changed() {
        let price = 1_000 * SPARKS_PER_CGT;
        let state = remix_listed_at(price);
        let root = state.asset.current.root.clone();
        fn seen(price_sparks: u128, root: &str) -> Seen<'_> {
            Seen { price_sparks, root }
        }

        assert_eq!(state.refuses(&BUYER, &rich(), seen(price, &root)), None);
        assert_eq!(
            state.refuses(&BUYER, &rich(), seen(price, &root.to_uppercase())),
            None,
            "a fingerprint is the same in either case"
        );

        // Looked while it cost half as much, and while it cost twice as much:
        // neither is the price they would be approving.
        for looked_at in [price / 2, price * 2] {
            let refused = state
                .refuses(&BUYER, &rich(), seen(looked_at, &root))
                .unwrap();
            assert!(refused.contains("The price changed since you looked"));
            assert!(refused.contains("1,000.00 CGT"), "{refused}");
            assert!(refused.contains(&with_symbol(looked_at)), "{refused}");
        }

        let refused = state
            .refuses(&BUYER, &rich(), seen(price, &"0".repeat(64)))
            .unwrap();
        assert!(refused.contains("revised since you looked"), "{refused}");

        // What stops any purchase is said first.
        assert!(state
            .refuses(&SELLER, &rich(), seen(price / 2, &root))
            .unwrap()
            .contains("your own listing"));
    }

    /// What an account can spend is what `pallet-balances` lets a sale take:
    /// free, less the existential deposit or what is frozen beyond the reserve.
    #[test]
    fn spendable_keeps_the_account_open() {
        let funds = |free, reserved, frozen| Funds {
            free,
            reserved,
            frozen,
        };
        assert_eq!(funds(1_000, 0, 0).spendable(100), 900);
        assert_eq!(funds(50, 0, 0).spendable(100), 0);
        assert_eq!(funds(1_000, 0, 400).spendable(100), 600);
        assert_eq!(funds(1_000, 300, 400).spendable(100), 900);
    }

    #[test]
    fn a_share_reads_as_a_percentage_exactly() {
        for (parts, expected) in [
            (0, "0%"),
            (1, "0.0001%"),
            (25_000, "2.5%"),
            (100_000, "10%"),
            (123_457, "12.3457%"),
            (333_333, "33.3333%"),
            (1_000_000, "100%"),
        ] {
            assert_eq!(percent(parts), expected);
        }
    }

    /// A view carries the terms, whether the asker holds it, what they can
    /// spend, and whether a pasted fingerprint is the one the asset carries.
    #[test]
    fn a_view_says_what_the_chain_holds_and_nothing_else() {
        let state = remix_listed_at(3_000 * SPARKS_PER_CGT);
        let root = state.asset.current.root.clone();

        let view = state.clone().view(Some(&root.to_uppercase()));
        assert_eq!(view.holder, address_of(&SELLER));
        assert!(!view.held_by_viewer);
        assert_eq!(view.cannot_buy, None);
        assert_eq!(view.pasted_root_matches, Some(true));
        assert_eq!(view.viewer_free_cgt.as_deref(), Some("9,900.00"));
        assert_eq!(view.terms.as_ref().unwrap().recipients[0].share, "10%");
        assert_eq!(view.terms.as_ref().unwrap().remix, "0%");
        assert_eq!(view.source_terms.as_ref().unwrap().remix, "5%");
        assert_eq!(
            view.breakdown.as_ref().unwrap().payouts.len(),
            3,
            "a live listing carries what it would pay"
        );

        let other = state.clone().view(Some(&"0".repeat(64)));
        assert_eq!(other.pasted_root_matches, Some(false));

        // Nobody asking: nothing is said about buying.
        let anonymous = SaleState {
            viewer: None,
            ..state.clone()
        }
        .view(None);
        assert_eq!(anonymous.cannot_buy, None);
        assert_eq!(anonymous.viewer_free_cgt, None);
        assert_eq!(anonymous.pasted_root_matches, None);

        // A void listing has no breakdown: there is no sale to describe.
        let void = SaleState {
            holder: CREATOR,
            ..state
        }
        .view(None);
        assert!(void.breakdown.is_none());
    }

    #[test]
    fn an_assets_number_is_read_out_of_what_was_pasted() {
        let root = "ab".repeat(32);
        let one = TradeItem {
            collection: 4,
            item: 0,
        };
        for (text, expected_root) in [
            ("4/0".to_string(), None),
            ("  4 / 0 ".to_string(), None),
            ("asset 4/0".to_string(), None),
            ("Asset 4/0,".to_string(), None),
            (format!("BLAKE3-256 {root} (asset 4/0)"), Some(root.clone())),
            (
                format!("BLAKE3-256 {} (asset 4/0)", root.to_uppercase()),
                Some(root.clone()),
            ),
        ] {
            assert_eq!(
                parse_asset_id(&text).unwrap(),
                (one, expected_root),
                "{text:?}"
            );
        }

        for bad in [
            "",
            "four",
            "4",
            "4/",
            "/0",
            "4/x",
            "-4/0",
            "4/0/1",
            "99999999999/0",
        ] {
            let error = parse_asset_id(bad).unwrap_err();
            assert!(
                error.to_string().contains("not an asset's number"),
                "{bad:?}: {error}"
            );
        }
        assert!(parse_asset_id("4/0 and 5/1")
            .unwrap_err()
            .to_string()
            .contains("more than one asset"));
        // The same asset named twice is one asset.
        assert_eq!(parse_asset_id("4/0 4/0").unwrap().0, one);
    }

    // ── The dialogs ─────────────────────────────────────────────────────────

    /// The dialog before a listing says the price, who a sale would pay and how
    /// much, that it is public, and that a sale cannot be undone.
    #[test]
    fn the_list_prompt_says_what_a_sale_would_pay() {
        let state = remix_listed_at(1);
        let breakdown = state.breakdown(3_000 * SPARKS_PER_CGT, None);
        let prompt = list_prompt(
            &state.asset,
            &breakdown,
            None,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );

        assert_eq!(prompt.title, "List this asset for sale");
        assert_eq!(prompt.approve, "List for sale");
        for expected in [
            "List \"a-remix\" for 3,000.00 CGT",
            "Asset: 7/2",
            state.asset.current.root.as_str(),
            "To the work it was remixed from (asset 3/1), 5% of the price:",
            &format!("{}: 150.00 CGT", address_of(&SOURCE)),
            "Royalties, each a share of what is left:",
            &format!("{} (10%): 285.00 CGT", address_of(&CREATOR)),
            "You, the seller: 2,565.00 CGT",
            "The listing is public",
            "cannot be undone",
            &format!("Seller: {}", address_of(&SELLER)),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        ] {
            assert!(
                prompt.body.contains(expected),
                "missing {expected:?} in\n{}",
                prompt.body
            );
        }

        // Changing a price says what it was, and is named for what it is.
        let change = list_prompt(
            &state.asset,
            &breakdown,
            Some(2_000 * SPARKS_PER_CGT),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert_eq!(change.approve, "Change the price");
        assert!(change.body.contains("It is listed at 2,000.00 CGT now."));

        // No terms: the dialog says nothing else is owed, rather than nothing.
        let plain = SaleState {
            derived_from: None,
            own_terms: None,
            source_terms: None,
            ..state
        };
        let prompt = list_prompt(
            &plain.asset,
            &plain.breakdown(SPARKS_PER_CGT, None),
            None,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert!(prompt.body.contains("You, the seller: 1.00 CGT"));
        assert!(prompt.body.contains("No royalty is owed on this asset"));
        assert!(!prompt.body.contains("remixed from"));
    }

    /// The dialog before a purchase says what is paid, to whom, that no more can
    /// be taken, and that it cannot be undone.
    #[test]
    fn the_buy_prompt_says_what_is_paid_and_that_it_cannot_be_undone() {
        let state = remix_listed_at(3_000 * SPARKS_PER_CGT);
        let breakdown = state.breakdown(3_000 * SPARKS_PER_CGT, Some(&BUYER));
        let prompt = buy_prompt(
            &state.asset,
            &breakdown,
            &address_of(&BUYER),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );

        assert_eq!(prompt.approve, "Buy");
        for expected in [
            "Buy \"a-remix\" for 3,000.00 CGT",
            "Asset: 7/2",
            state.asset.current.root.as_str(),
            "You pay 3,000.00 CGT",
            &format!("{}: 150.00 CGT", address_of(&SOURCE)),
            &format!("{} (10%): 285.00 CGT", address_of(&CREATOR)),
            &format!("The seller, {}: 2,565.00 CGT", address_of(&SELLER)),
            "You will not pay more than 3,000.00 CGT",
            "cannot be undone",
            "All of it happens or none of it does",
            "still revisable",
            &format!("Buyer: {}", address_of(&BUYER)),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        ] {
            assert!(
                prompt.body.contains(expected),
                "missing {expected:?} in\n{}",
                prompt.body
            );
        }

        // A part owed to the buyer is said to stay with them.
        let own = buy_prompt(
            &state.asset,
            &state.breakdown(3_000 * SPARKS_PER_CGT, Some(&CREATOR)),
            &address_of(&CREATOR),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert!(own.body.contains("stays with you"));

        // A permanent asset carries no warning about revision.
        let permanent = OwnedAsset {
            revisable: false,
            ..state.asset.clone()
        };
        let prompt = buy_prompt(
            &permanent,
            &breakdown,
            &address_of(&BUYER),
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert!(!prompt.body.contains("revisable"));
    }

    #[test]
    fn the_unlist_prompt_says_what_withdrawing_does() {
        let listed = asset(7, 2, "a-remix");
        let prompt = unlist_prompt(
            &listed,
            3_000 * SPARKS_PER_CGT,
            false,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert_eq!(prompt.approve, "Withdraw");
        for expected in [
            "Withdraw the listing of \"a-remix\"",
            "Asset: 7/2",
            "Listed at: 3,000.00 CGT",
            "nobody can buy the asset",
            "stays in your Inventory",
        ] {
            assert!(
                prompt.body.contains(expected),
                "missing {expected:?} in\n{}",
                prompt.body
            );
        }

        let void = unlist_prompt(
            &asset(7, 2, ""),
            1,
            true,
            "Demiurge Development",
            super::super::LOCAL_RPC,
        );
        assert!(void.body.contains("This listing is void"));
        assert!(
            void.body.contains("Withdraw the listing of Asset 7/2"),
            "an unnamed asset is named by its number"
        );
    }

    /// The chain's own refusals are turned into the sentences a person was
    /// owed, and the transaction is still named so it can be looked up.
    #[test]
    fn the_chains_refusal_is_put_in_words() {
        let failed = |error: &str| {
            in_words(QorError::Rpc(format!(
                "the purchase 0xabc was finalised in 0xdef but failed: Pallet error: \
                 Drc369Royalties::{error}"
            )))
            .to_string()
        };
        for (error, expected) in [
            ("PriceAboveLimit", "price was raised"),
            ("NotListed", "listing was withdrawn"),
            ("ListingStale", "listing is void"),
            ("OwnListing", "your own listing"),
            ("PaymentCannotBeReceived", "cannot receive its part"),
            ("NotOwner", "does not hold that asset"),
        ] {
            let words = failed(error);
            assert!(words.contains(expected), "{error}: {words}");
            assert!(words.contains("the purchase 0xabc"), "{error}: {words}");
            assert!(!words.contains("Pallet error"), "{error}: {words}");
        }

        // Anything it does not recognise is passed on untouched, as is anything
        // that is not the chain's.
        let unknown = in_words(QorError::Rpc("the node did not answer".into()));
        assert_eq!(unknown.kind(), "rpc");
        assert_eq!(in_words(QorError::Declined).kind(), "declined");
    }

    // ── Refused before anyone is asked, and before any node is needed ───────

    #[tokio::test]
    async fn a_sale_nobody_could_finish_asks_nothing() {
        let client = ChainClient::new(NOWHERE).unwrap();
        let dir = std::env::temp_dir().join(format!("qor-sale-nowhere-{}", std::process::id()));
        let vault = crate::vault::testing::vault_in(&dir);
        let confirm = Scripted::approving();
        let from = address_of(&SELLER);
        let root = "ab".repeat(32);
        let seen = Seen {
            price_sparks: 1,
            root: &root,
        };

        // A price of nothing is refused where it is typed, by every path.
        for error in [
            client
                .list(&vault, &confirm, &from, 1, 0, 0)
                .await
                .unwrap_err(),
            client.sale_preview(1, 0, 0).await.unwrap_err(),
        ] {
            assert_eq!(error.kind(), "bad_amount");
            assert!(error.to_string().contains("above zero"), "{error}");
        }

        // An address that is not one, and a number that is not one.
        assert_eq!(
            client
                .list(&vault, &confirm, "not-an-address", 1, 0, 1)
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );
        assert_eq!(
            client
                .buy(&vault, &confirm, "not-an-address", 1, 0, seen)
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );
        assert_eq!(
            client
                .unlist(&vault, &confirm, "not-an-address", 1, 0)
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );
        assert!(client
            .sale("nonsense", None)
            .await
            .unwrap_err()
            .to_string()
            .contains("not an asset's number"));
        assert_eq!(
            client
                .sale("4/0", Some("not-an-address"))
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );

        // What IS well formed gets as far as the node, and no further, because
        // there is no node. Still nothing signed and nobody asked.
        for error in [
            client
                .list(&vault, &confirm, &from, 1, 0, 1)
                .await
                .unwrap_err(),
            client
                .unlist(&vault, &confirm, &from, 1, 0)
                .await
                .unwrap_err(),
            client
                .buy(&vault, &confirm, &from, 1, 0, seen)
                .await
                .unwrap_err(),
            client.sale("1/0", Some(&from)).await.unwrap_err(),
            client.sale_preview(1, 0, 1).await.unwrap_err(),
        ] {
            assert!(matches!(error.kind(), "network" | "rpc"), "{error}");
        }
        assert_eq!(confirm.times_asked(), 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
