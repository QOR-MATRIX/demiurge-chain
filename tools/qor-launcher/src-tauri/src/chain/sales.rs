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
//! Before anyone signs, the dialog says who a sale pays and how much. The
//! launcher does not work that out. It asks the chain, through the runtime API
//! `Drc369RoyaltiesApi::sale_preview` (spec_version 6 onwards), at the same
//! finalised block as every other read here, and shows the chain's answer: the
//! parts, who receives each, and whether the sale could settle. There is no
//! copy of the pallet's arithmetic in the launcher, so there is nothing to keep
//! in step with it. A node whose runtime does not serve that API is refused in
//! words rather than guessed at.
//!
//! What is shown **before** a sale is the chain's preview; what is reported
//! **after** one is the chain's `Sold` event and nothing else.
//!
//! # What is refused before anyone is asked
//!
//! A person should not be asked to approve what the chain would refuse. With a
//! buyer, the chain's preview runs the sale itself and undoes it, so its
//! refusal is the one the transaction would get: a balance that cannot cover
//! the price and keep the account open, a part its recipient could not
//! receive, an asset held in place by nesting. Each is said in words, with the
//! same sentences [`in_words`] gives a refused transaction. The launcher adds
//! only what the chain is not asked: a listing that is void, and a price or a
//! fingerprint that changed since the buyer looked. The chain still checks
//! everything again when the transaction arrives.
//!
//! # Nothing is taken that the chain does not take
//!
//! No platform share (U-15 is undecided) and no fee (OPEN-4): the parts of a
//! sale are the remix share, the royalties and the seller, and they sum to the
//! price.

use scale_decode::DecodeAsType;
use serde::Serialize;
use subxt::dynamic::{self, Value};
use subxt::ext::scale_value::{Composite, ValueDef};
use subxt::utils::AccountId32;

use crate::cgt;
use crate::error::{QorError, QorResult};
use crate::vault::derive::address_of;
use crate::vault::{normalise_address, Vault};
use crate::{Confirm, Prompt};

use super::assets::{ContentRefArg, OwnedAsset, TradeItem};
use super::{ChainClient, Connection};

/// An account, as the chain keys it.
type Account = [u8; 32];

// ── What the launcher reads, decoded by name ─────────────────────────────────

/// The runtime API that says what a sale would pay (spec_version 6 onwards).
const PREVIEW_API: &str = "Drc369RoyaltiesApi";
const PREVIEW_METHOD: &str = "sale_preview";

/// `pallet_drc369_royalties::SalePreview`: the chain's own answer to what a
/// sale of one asset at one price would pay. The refusal is a `DispatchError`,
/// read as a value and named through the node's metadata ([`refusal_of`]).
#[derive(Debug, Clone, DecodeAsType)]
struct PreviewRead {
    source: Option<(u32, u32)>,
    remix: Vec<(AccountId32, u128)>,
    royalties: Vec<(AccountId32, u128)>,
    seller: AccountId32,
    seller_receives: u128,
    refusal: Option<Value>,
}

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
    /// Why the chain will not move this asset, when it is nested or holds
    /// nested assets (ADR-065). A sale of it fails with `ItemLocked`.
    held_in_place: Option<String>,
    existential_deposit: u128,
    /// The free balance of every account a sale of this asset could pay.
    free: Vec<(Account, u128)>,
    /// The account asking, and what it holds.
    viewer: Option<(Account, Funds)>,
    /// The chain's answer to what a sale would pay, read at the same block:
    /// at the price asked, or at the listed price. `None` when nothing was
    /// asked.
    preview: Option<Preview>,
}

/// What [`ChainClient::sale_state`] asks the chain a sale would pay at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ask {
    /// Nothing: the caller needs no sale's parts.
    Nothing,
    /// The listed price, if the listing is live.
    Listed,
    /// A price someone is thinking of listing at.
    At(u128),
}

/// Why the chain would refuse a sale, as its metadata names it: a pallet's
/// error (`Drc369Royalties`, `PaymentCannotBeReceived`), or one of the
/// runtime's own (`Token`, `FundsUnavailable`; or `BadOrigin`, with no name).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Refusal {
    within: String,
    name: String,
}

impl Refusal {
    fn is(&self, within: &str, name: &str) -> bool {
        self.within == within && self.name == name
    }

    /// The buyer's balance could not cover the price and keep the account open.
    fn is_funds(&self) -> bool {
        matches!(
            self.name.as_str(),
            "FundsUnavailable" | "InsufficientBalance" | "NotExpendable"
        )
    }

    /// A part of the sale could not be received: this stops every buyer.
    fn is_unreceivable(&self) -> bool {
        self.is("Drc369Royalties", "PaymentCannotBeReceived")
    }
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.name.is_empty() {
            write!(f, "{}", self.within)
        } else {
            write!(f, "{}::{}", self.within, self.name)
        }
    }
}

/// A named field of a struct-like value.
fn field<'a>(value: &'a Value, name: &str) -> Option<&'a Value> {
    match &value.value {
        ValueDef::Composite(Composite::Named(fields)) => fields
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, value)| value),
        _ => None,
    }
}

/// The first element of a tuple- or array-like value.
fn first(value: &Value) -> Option<&Value> {
    match &value.value {
        ValueDef::Composite(composite) => composite.values().next(),
        _ => None,
    }
}

/// Name a `DispatchError`. A pallet's error carries only the pallet's index
/// and the error's; `module` turns those into names through the node's
/// metadata, so a refusal is named as the runtime that gave it names it.
fn refusal_of(value: &Value, module: impl Fn(u8, u8) -> Option<(String, String)>) -> Refusal {
    let ValueDef::Variant(variant) = &value.value else {
        return Refusal {
            within: "an error the launcher cannot read".into(),
            name: String::new(),
        };
    };
    let fields: Vec<&Value> = variant.values.values().collect();
    if variant.name == "Module" {
        // `Module(ModuleError { index, error })`: the error's own index is the
        // first byte of `error`.
        let inner = fields.first().copied();
        let index = inner
            .and_then(|error| field(error, "index"))
            .and_then(Value::as_u128)
            .and_then(|index| u8::try_from(index).ok());
        let error = inner
            .and_then(|error| field(error, "error"))
            .and_then(first)
            .and_then(Value::as_u128)
            .and_then(|error| u8::try_from(error).ok());
        return match (index, error) {
            (Some(index), Some(error)) => match module(index, error) {
                Some((within, name)) => Refusal { within, name },
                None => Refusal {
                    within: format!("pallet {index}"),
                    name: format!("error {error}"),
                },
            },
            _ => Refusal {
                within: "Module".into(),
                name: String::new(),
            },
        };
    }
    // `Token(FundsUnavailable)`, `Arithmetic(Underflow)`, or a bare `BadOrigin`.
    let name = match fields.as_slice() {
        [inner] => match &inner.value {
            ValueDef::Variant(detail) => detail.name.clone(),
            _ => String::new(),
        },
        _ => String::new(),
    };
    Refusal {
        within: variant.name.clone(),
        name,
    }
}

/// The chain's answer to what a sale of one asset at one price would pay,
/// as the launcher holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Preview {
    /// The price the chain was asked about.
    price: u128,
    /// The buyer it was asked about, if any. With one, the chain ran the sale
    /// for that account and undid it.
    buyer: Option<Account>,
    source: Option<(u32, u32)>,
    remix: Vec<(Account, u128)>,
    royalties: Vec<(Account, u128)>,
    seller: Account,
    seller_receives: u128,
    refusal: Option<Refusal>,
}

impl Preview {
    /// The chain's answer, decoded, with its refusal named through `module`.
    /// The parts are the price, or the answer is not used: a part that does
    /// not add up is a misreading, and nobody is shown one.
    fn from_read(
        read: PreviewRead,
        price: u128,
        buyer: Option<Account>,
        module: impl Fn(u8, u8) -> Option<(String, String)>,
    ) -> QorResult<Self> {
        let preview = Preview {
            price,
            buyer,
            source: read.source,
            remix: read.remix.into_iter().map(|(who, a)| (who.0, a)).collect(),
            royalties: read
                .royalties
                .into_iter()
                .map(|(who, a)| (who.0, a))
                .collect(),
            seller: read.seller.0,
            seller_receives: read.seller_receives,
            refusal: read.refusal.as_ref().map(|value| refusal_of(value, module)),
        };
        let total = preview
            .parts()
            .try_fold(0u128, |sum, (_, amount)| sum.checked_add(amount));
        if total != Some(price) {
            return Err(QorError::Rpc(format!(
                "the chain's answer to what a sale at {} pays does not add up to the price",
                with_symbol(price)
            )));
        }
        Ok(preview)
    }

    /// Every part, in the order the chain pays it: the source's recipients,
    /// the asset's own, then the seller.
    fn parts(&self) -> impl Iterator<Item = (Account, u128)> + '_ {
        self.remix
            .iter()
            .chain(self.royalties.iter())
            .copied()
            .chain(std::iter::once((self.seller, self.seller_receives)))
    }
}

/// What is said when the connected node's runtime cannot answer what a sale
/// would pay. The launcher has no arithmetic of its own to fall back on.
fn cannot_preview(spec_version: u32) -> QorError {
    QorError::Qontrol(format!(
        "This node runs spec version {spec_version} of the chain, which cannot say what a sale \
         would pay: that needs spec version 6 or later. The launcher does not work it out on its \
         own, so selling and buying wait until the node is upgraded."
    ))
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

    /// What a sale pays, to whom, as the chain answered it. `None` when the
    /// chain was not asked.
    ///
    /// The amounts, their order, the seller and the source are the chain's.
    /// What the launcher adds is only what a person reads alongside them: each
    /// royalty recipient's share, from the terms read at the same block, and
    /// which part is the viewer's own.
    fn breakdown(&self) -> Option<Breakdown> {
        let preview = self.preview.as_ref()?;
        let viewer = self.viewer.as_ref().map(|(account, _)| account);
        let payout = |kind, who: &Account, share: Option<String>, amount: u128| Payout {
            kind,
            address: address_of(who),
            share,
            amount_sparks: amount.to_string(),
            amount_cgt: cgt_of(amount),
            to_buyer: viewer == Some(who),
        };

        let mut payouts = Vec::new();
        for (who, amount) in &preview.remix {
            payouts.push(payout(PayoutKind::Source, who, None, *amount));
        }
        // The chain pays the asset's recipients in the order its terms name
        // them. A share is said only where the terms name the same account in
        // the same place, so a share is never put beside someone else's part.
        let named = self
            .own_terms
            .as_ref()
            .map(|terms| terms.recipients.as_slice())
            .unwrap_or_default();
        for (at, (who, amount)) in preview.royalties.iter().enumerate() {
            let share = named
                .get(at)
                .filter(|(recipient, _)| recipient.0 == *who)
                .map(|(_, parts)| percent(*parts));
            payouts.push(payout(PayoutKind::Royalty, who, share, *amount));
        }
        payouts.push(payout(
            PayoutKind::Seller,
            &preview.seller,
            None,
            preview.seller_receives,
        ));

        // A source is named when a sale owes it something: it is a remix, and
        // the work it was remixed from has terms.
        let source_terms = self
            .source_terms
            .as_ref()
            .filter(|_| preview.source.is_some());
        Some(Breakdown {
            price_sparks: preview.price.to_string(),
            price_cgt: cgt_of(preview.price),
            source: source_terms
                .and(preview.source)
                .map(|(collection, item)| TradeItem { collection, item }),
            source_share: source_terms.map(|terms| percent(terms.remix)),
            payouts,
            // Only what stops every buyer: what stops one buyer is theirs to
            // read in `cannot_buy`, and a seller may list an asset that is
            // nested, as the chain lets them.
            blocked: preview
                .refusal
                .as_ref()
                .filter(|refusal| refusal.is_unreceivable())
                .map(|refusal| self.refusal_in_words(preview, refusal)),
        })
    }

    /// The chain's refusal of a sale, in the words a person is owed.
    ///
    /// The chain decides; these sentences only explain. Where the launcher can
    /// say more from what it read at the same block — which account cannot
    /// receive its part, how far the buyer is short, why the asset is held in
    /// place — it does. Otherwise it is the sentence [`in_words`] gives the
    /// same refusal of a transaction.
    fn refusal_in_words(&self, preview: &Preview, refusal: &Refusal) -> String {
        let explained = if refusal.is_unreceivable() {
            self.unreceivable(preview)
        } else if refusal.name == "ItemLocked" {
            self.held_in_place.clone()
        } else if refusal.is_funds() {
            self.short_of_funds(preview)
        } else {
            None
        };
        explained
            .or_else(|| reason(&refusal.to_string()).map(str::to_string))
            .unwrap_or_else(|| format!("The chain would refuse this sale: {refusal}."))
    }

    /// Which part of a sale its recipient cannot receive, from the balances
    /// read at the same block, once the chain has said one cannot.
    ///
    /// The chain refuses the whole sale at the first part its recipient cannot
    /// receive: an account that would be left holding less than the existential
    /// deposit. A part of nothing, and a part to the buyer, move nothing. An
    /// account paid twice has its first part by the time its second arrives.
    fn unreceivable(&self, preview: &Preview) -> Option<String> {
        let mut held: Vec<(Account, u128)> = Vec::new();
        for (who, amount) in preview.parts() {
            if amount == 0 || preview.buyer == Some(who) {
                continue;
            }
            let at = match held.iter().position(|(known, _)| *known == who) {
                Some(at) => at,
                None => {
                    held.push((who, self.free_of(&who)));
                    held.len() - 1
                }
            };
            let after = held[at].1.saturating_add(amount);
            if after < self.existential_deposit {
                return Some(format!(
                    "A sale at this price cannot settle as things stand. {} is owed {} from it, \
                     and that account holds too little to stay open on that: an account needs \
                     {} to exist. The chain refuses the whole sale rather than paying everyone \
                     else. A higher price clears it, or that account receiving CGT first.",
                    address_of(&who),
                    with_symbol(amount),
                    with_symbol(self.existential_deposit),
                ));
            }
            held[at].1 = after;
        }
        None
    }

    /// How far the buyer is short, once the chain has said they cannot pay.
    /// A part owed to the buyer themselves never leaves their account.
    fn short_of_funds(&self, preview: &Preview) -> Option<String> {
        let (buyer, funds) = self.viewer.as_ref()?;
        let stays = preview
            .parts()
            .filter(|(who, _)| who == buyer)
            .fold(0u128, |sum, (_, amount)| sum.saturating_add(amount));
        let leaves = preview.price.saturating_sub(stays);
        let spendable = funds.spendable(self.existential_deposit);
        (leaves > spendable).then(|| {
            format!(
                "This account cannot cover it. Buying takes {} and the account has {} it can \
                 spend: it holds {}, and {} of that has to stay for the account to remain open.",
                with_symbol(leaves),
                with_symbol(spendable),
                with_symbol(funds.free),
                with_symbol(funds.free.saturating_sub(spendable)),
            )
        })
    }

    /// Why the account asking cannot buy this asset now, in words. `None` when
    /// it can, or when nobody is asking.
    fn cannot_buy(&self) -> Option<String> {
        let (buyer, _) = self.viewer.as_ref()?;
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

        // Everything else is the chain's to say: it ran this purchase for this
        // buyer at this price, and undid it.
        let Some(preview) = self
            .preview
            .as_ref()
            .filter(|preview| preview.buyer == Some(*buyer) && preview.price == listing.price)
        else {
            return Some(
                "The launcher could not ask the chain what this purchase would do, so it is not \
                 offered. Look the asset up again."
                    .into(),
            );
        };
        preview
            .refusal
            .as_ref()
            .map(|refusal| self.refusal_in_words(preview, refusal))
    }

    /// Why this purchase must not be put to the account asking, given what
    /// they were shown. Everything in [`Self::cannot_buy`], and then the two
    /// things only a purchase can get wrong: a price, or a fingerprint, that is
    /// no longer the one that was on their screen.
    fn refuses(&self, seen: Seen<'_>) -> Option<String> {
        if let Some(reason) = self.cannot_buy() {
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
        let breakdown = self.live_listing().and_then(|_| self.breakdown());
        let cannot_buy = self.cannot_buy();

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

/// Why the chain will not move an asset, in words: it is nested inside another,
/// or it holds nested assets (`pallet-drc369`'s `is_held_in_place`, ADR-065).
/// `None` when it stands free. One sentence for the Market and for a purchase,
/// so the two cannot say different things about the same asset.
pub(super) fn held_in_place(parent: Option<(u32, u32)>, children: u32) -> Option<String> {
    match (parent, children) {
        (Some((collection, item)), _) => Some(format!(
            "This asset is nested inside asset {collection}/{item}, and the chain does not move \
             a nested asset. It cannot be bought until its holder takes it out."
        )),
        (None, 0) => None,
        (None, 1) => Some(
            "This asset holds another asset nested inside it, and the chain does not move an \
             asset that holds others. It cannot be bought until its holder takes that out."
                .into(),
        ),
        (None, count) => Some(format!(
            "This asset holds {count} assets nested inside it, and the chain does not move an \
             asset that holds others. It cannot be bought until its holder takes them out."
        )),
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
    let Some(reason) = reason(text) else {
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

/// The sentence for one of the chain's refusals named in `text`, if it is one a
/// person is owed words for. One table for a refused transaction and for a
/// refusal the chain's preview foresees, so the two cannot say different
/// things about the same refusal.
fn reason(text: &str) -> Option<&'static str> {
    Some(if text.contains("PriceAboveLimit") {
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
    } else if text.contains("ContentChanged") {
        "The seller revised this asset before your purchase settled, so it no longer holds \
         the content you looked at. Nothing moved. Look the asset up again."
    } else if text.contains("ItemLocked") {
        "This asset is nested inside another, or holds nested assets, so the chain would not \
         move it and nothing moved."
    } else if text.contains("NotOwner") {
        "This account does not hold that asset any more, so nothing changed."
    } else if text.contains("ZeroPrice") {
        "A price of nothing is a gift, and a gift is a trade, not a sale."
    } else if text.contains("FundsUnavailable")
        || text.contains("Funds are unavailable")
        || text.contains("InsufficientBalance")
        || text.contains("NotExpendable")
    {
        "This account could not cover the price and stay open, so nothing moved."
    } else {
        return None;
    })
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

    /// Everything a sale of one asset depends on, read from chain storage, and
    /// the chain's own answer to what a sale would pay at the price `ask`
    /// names, all at one finalised block. `None` when no DRC-369 asset has
    /// that number.
    ///
    /// The chain is asked for `viewer` as the buyer unless they hold the
    /// asset, so its refusal is the one their purchase would get.
    async fn sale_state(
        &self,
        connection: &Connection,
        collection: u32,
        item: u32,
        viewer: Option<Account>,
        ask: Ask,
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

        let held_in_place = {
            let parent = storage
                .try_fetch(
                    dynamic::storage::<(u32, u32), (u32, u32)>("Drc369", "ParentOf"),
                    (collection, item),
                )
                .await
                .map_err(|e| QorError::Rpc(format!("could not read the asset's nesting: {e}")))?
                .map(|value| value.decode())
                .transpose()
                .map_err(|e| {
                    QorError::Rpc(format!("the asset's nesting could not be decoded: {e}"))
                })?;
            let children = storage
                .try_fetch(
                    dynamic::storage::<(u32, u32), u32>("Drc369", "ChildCount"),
                    (collection, item),
                )
                .await
                .map_err(|e| QorError::Rpc(format!("could not read the asset's nesting: {e}")))?
                .map(|value| value.decode())
                .transpose()
                .map_err(|e| {
                    QorError::Rpc(format!("the asset's nesting could not be decoded: {e}"))
                })?
                .unwrap_or(0);
            held_in_place(parent, children)
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

        let price = match ask {
            Ask::Nothing => None,
            Ask::Listed => listing
                .as_ref()
                .filter(|listing| listing.seller.0 == holder)
                .map(|listing| listing.price),
            Ask::At(price) => Some(price),
        };
        let preview = match price {
            None => None,
            Some(price) => {
                let metadata = at.metadata_ref();
                // A runtime before spec_version 6 has no such API. Its absence
                // is said in words; nothing is worked out instead.
                if metadata
                    .runtime_api_trait_by_name(PREVIEW_API)
                    .and_then(|api| api.method_by_name(PREVIEW_METHOD))
                    .is_none()
                {
                    return Err(cannot_preview(at.spec_version()));
                }
                let buyer = viewer
                    .map(|(account, _)| account)
                    .filter(|account| *account != holder);
                let read = at
                    .runtime_apis()
                    .call(dynamic::runtime_api_call::<_, Option<PreviewRead>>(
                        PREVIEW_API,
                        PREVIEW_METHOD,
                        (collection, item, price, buyer.map(AccountId32)),
                    ))
                    .await
                    .map_err(|e| {
                        QorError::Rpc(format!(
                            "could not ask the chain what a sale would pay: {e}"
                        ))
                    })?
                    .ok_or_else(|| {
                        QorError::Rpc(format!(
                            "the chain's preview says {collection}/{item} is not a DRC-369 asset, \
                             though its storage holds one"
                        ))
                    })?;
                Some(Preview::from_read(read, price, buyer, |pallet, error| {
                    let pallet = metadata.pallet_by_error_index(pallet)?;
                    let error = pallet.error_variant_by_index(error)?;
                    Some((pallet.name().to_string(), error.name.clone()))
                })?)
            }
        };

        Ok(Some(SaleState {
            holder,
            asset,
            derived_from,
            own_terms,
            source_terms,
            listing,
            held_in_place,
            existential_deposit,
            free,
            viewer,
            preview,
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
            .sale_state(&connection, id.collection, id.item, viewer, Ask::Listed)
            .await?
            .ok_or_else(|| no_such_asset(id.collection, id.item))?;
        Ok(state.view(pasted_root.as_deref()))
    }

    /// What a sale of an asset at `price_sparks` would pay, before it is listed:
    /// the chain's answer, for no buyer in particular.
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
            .sale_state(&connection, collection, item, None, Ask::At(price_sparks))
            .await?
            .ok_or_else(|| no_such_asset(collection, item))?;
        state.breakdown().ok_or_else(not_asked)
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
            .sale_state(&connection, collection, item, None, Ask::At(price_sparks))
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
        let breakdown = state.breakdown().ok_or_else(not_asked)?;
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
            .sale_state(&connection, collection, item, None, Ask::Nothing)
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
            .sale_state(&connection, collection, item, Some(buyer), Ask::Listed)
            .await?
            .ok_or_else(|| no_such_asset(collection, item))?;

        if let Some(reason) = state.refuses(seen) {
            return Err(QorError::Qontrol(reason));
        }
        let listing = state
            .live_listing()
            .ok_or_else(|| QorError::Internal("a sale without a listing".into()))?;

        // What the chain said this purchase would pay, for this buyer.
        let breakdown = state.breakdown().ok_or_else(not_asked)?;
        // `buy_exact`, not `buy`: the content the buyer was shown goes with the
        // price, so a revision that lands after this point is refused by the
        // chain (`ContentChanged`) and not only by the check above.
        let content = ContentRefArg::try_from(&state.asset.current)?;
        let call = dynamic::transaction(
            "Drc369Royalties",
            "buy_exact",
            (collection, item, listing.price, content),
        );
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

/// A sale's parts were wanted and the chain was not asked for them: a fault in
/// the launcher, never something a person did.
fn not_asked() -> QorError {
    QorError::Internal("the chain was not asked what this sale would pay".into())
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

    /// An amount in whole CGT.
    fn c(whole: u128) -> u128 {
        whole * SPARKS_PER_CGT
    }

    // ── What follows from what was read ─────────────────────────────────────
    //
    // Every amount below is written out as the chain's answer, never worked out
    // here: the launcher has no arithmetic of its own to check it against, and
    // these tests hold it to showing what the chain said and nothing else.

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
            free: c(10_000),
            reserved: 0,
            frozen: 0,
        }
    }

    fn refused(within: &str, name: &str) -> Option<Refusal> {
        Some(Refusal {
            within: within.into(),
            name: name.into(),
        })
    }

    /// What the chain answers about a sale of the remix 7/2, derived from 3/1,
    /// held by SELLER.
    fn chain_says(
        price: u128,
        buyer: Option<Account>,
        remix: &[(Account, u128)],
        royalties: &[(Account, u128)],
        seller_receives: u128,
        refusal: Option<Refusal>,
    ) -> Preview {
        Preview {
            price,
            buyer,
            source: Some((3, 1)),
            remix: remix.to_vec(),
            royalties: royalties.to_vec(),
            seller: SELLER,
            seller_receives,
            refusal,
        }
    }

    /// A remix held by SELLER and listed at 3,000 CGT: its source owes SOURCE a
    /// 5% remix share, and its own terms pay CREATOR 10%. BUYER is looking, and
    /// the chain was asked what BUYER's purchase would pay.
    fn listed() -> SaleState {
        SaleState {
            holder: SELLER,
            asset: asset(7, 2, "a-remix"),
            derived_from: Some((3, 1)),
            own_terms: Some(terms(&[(CREATOR, 100_000)], 0)),
            source_terms: Some(terms(&[(SOURCE, 200_000)], 50_000)),
            listing: Some(ListingRead {
                seller: AccountId32(SELLER),
                price: c(3_000),
            }),
            held_in_place: None,
            existential_deposit: ED,
            free: vec![(SELLER, c(500)), (CREATOR, ED), (SOURCE, ED)],
            viewer: Some((BUYER, rich())),
            preview: Some(chain_says(
                c(3_000),
                Some(BUYER),
                &[(SOURCE, c(150))],
                &[(CREATOR, c(285))],
                c(2_565),
                None,
            )),
        }
    }

    /// The same remix, listed at `price`, with the chain's answer for BUYER.
    fn listed_at(price: u128, answer: Preview) -> SaleState {
        SaleState {
            listing: Some(ListingRead {
                seller: AccountId32(SELLER),
                price,
            }),
            preview: Some(answer),
            ..listed()
        }
    }

    fn amounts(breakdown: &Breakdown) -> Vec<(PayoutKind, String, String)> {
        breakdown
            .payouts
            .iter()
            .map(|p| (p.kind, p.address.clone(), p.amount_cgt.clone()))
            .collect()
    }

    /// The breakdown is the chain's answer, in the chain's order — source,
    /// royalties, seller — in CGT a person can read.
    #[test]
    fn a_breakdown_is_the_chains_answer_in_the_chains_order() {
        let state = listed();
        let breakdown = state.breakdown().expect("the chain was asked");

        assert_eq!(breakdown.price_cgt, "3,000.00");
        assert_eq!(breakdown.price_sparks, c(3_000).to_string());
        assert_eq!(
            breakdown.source,
            Some(TradeItem {
                collection: 3,
                item: 1
            })
        );
        assert_eq!(breakdown.source_share.as_deref(), Some("5%"));
        assert_eq!(
            amounts(&breakdown),
            vec![
                (PayoutKind::Source, address_of(&SOURCE), "150.00".into()),
                (PayoutKind::Royalty, address_of(&CREATOR), "285.00".into()),
                (PayoutKind::Seller, address_of(&SELLER), "2,565.00".into()),
            ]
        );
        assert_eq!(breakdown.payouts[0].share, None);
        assert_eq!(breakdown.payouts[1].share.as_deref(), Some("10%"));
        assert_eq!(breakdown.payouts[2].share, None);
        assert!(breakdown.payouts.iter().all(|p| !p.to_buyer));
        assert_eq!(breakdown.blocked, None);

        // Whatever the chain says is what is shown, even where it is not what
        // the terms alone would suggest: the launcher does not check the
        // chain's arithmetic against arithmetic of its own, because it has none.
        let odd = listed_at(
            c(3_000),
            chain_says(
                c(3_000),
                Some(BUYER),
                &[(SOURCE, c(151))],
                &[(CREATOR, c(284))],
                c(2_565),
                None,
            ),
        );
        assert_eq!(
            amounts(&odd.breakdown().unwrap()),
            vec![
                (PayoutKind::Source, address_of(&SOURCE), "151.00".into()),
                (PayoutKind::Royalty, address_of(&CREATOR), "284.00".into()),
                (PayoutKind::Seller, address_of(&SELLER), "2,565.00".into()),
            ]
        );

        // The seller is whoever the chain names.
        let named = listed_at(
            c(3_000),
            Preview {
                seller: CREATOR,
                ..state.preview.clone().unwrap()
            },
        );
        assert_eq!(
            named.breakdown().unwrap().payouts[2].address,
            address_of(&CREATOR)
        );

        // An asset with no terms and no source pays its seller everything.
        let plain = SaleState {
            derived_from: None,
            own_terms: None,
            source_terms: None,
            preview: Some(Preview {
                source: None,
                ..chain_says(SPARKS_PER_CGT / 2, None, &[], &[], SPARKS_PER_CGT / 2, None)
            }),
            ..state
        };
        let breakdown = plain.breakdown().unwrap();
        assert_eq!(
            amounts(&breakdown),
            vec![(PayoutKind::Seller, address_of(&SELLER), "0.50".into())]
        );
        assert_eq!(breakdown.source, None);
        assert_eq!(breakdown.source_share, None);

        // Nothing asked, nothing to show.
        let unasked = SaleState {
            preview: None,
            ..listed()
        };
        assert!(unasked.breakdown().is_none());
    }

    /// A source with no terms is owed nothing, and is not named as if it were.
    #[test]
    fn a_remix_of_a_source_without_terms_names_no_source() {
        let state = SaleState {
            source_terms: None,
            ..listed_at(
                c(1_000),
                chain_says(
                    c(1_000),
                    Some(BUYER),
                    &[],
                    &[(CREATOR, c(100))],
                    c(900),
                    None,
                ),
            )
        };
        let breakdown = state.breakdown().unwrap();
        assert_eq!(breakdown.source, None);
        assert_eq!(breakdown.source_share, None);
        assert_eq!(breakdown.payouts.len(), 2);
        assert_eq!(breakdown.payouts[0].amount_cgt, "100.00");
        assert_eq!(breakdown.payouts[1].amount_cgt, "900.00");
    }

    /// A royalty recipient's share is read from the terms, and put beside a
    /// part only where the terms name the same account in the same place, so
    /// nobody's share is shown beside somebody else's part.
    #[test]
    fn a_share_is_said_only_beside_the_account_the_terms_name() {
        let both = terms(&[(CREATOR, 100_000), (SOURCE, 50_000)], 0);
        let answer = |royalties: &[(Account, u128)]| {
            let paid: u128 = royalties.iter().map(|(_, amount)| amount).sum();
            chain_says(c(1_000), Some(BUYER), &[], royalties, c(1_000) - paid, None)
        };
        let shares = |royalties: &[(Account, u128)]| -> Vec<Option<String>> {
            SaleState {
                own_terms: Some(both.clone()),
                source_terms: None,
                ..listed_at(c(1_000), answer(royalties))
            }
            .breakdown()
            .unwrap()
            .payouts
            .into_iter()
            .filter(|p| p.kind == PayoutKind::Royalty)
            .map(|p| p.share)
            .collect()
        };

        assert_eq!(
            shares(&[(CREATOR, c(100)), (SOURCE, c(45))]),
            vec![Some("10%".to_string()), Some("5%".to_string())]
        );
        // The chain's order is not the terms' order: no share is guessed.
        assert_eq!(
            shares(&[(SOURCE, c(45)), (CREATOR, c(100))]),
            vec![None, None]
        );
        // A part the terms read at this block do not name at all.
        assert_eq!(
            shares(&[(CREATOR, c(100)), (SOURCE, c(45)), (BUYER, c(1))]),
            vec![Some("10%".to_string()), Some("5%".to_string()), None]
        );
    }

    /// A refusal is named as the runtime names it: a pallet's error through the
    /// node's metadata, and the runtime's own errors by their variants.
    #[test]
    fn a_refusal_is_named_as_the_runtime_names_it() {
        let module = |index: u128, error: u128| {
            Value::unnamed_variant(
                "Module",
                [Value::named_composite([
                    ("index", Value::u128(index)),
                    (
                        "error",
                        Value::unnamed_composite([
                            Value::u128(error),
                            Value::u128(0),
                            Value::u128(0),
                            Value::u128(0),
                        ]),
                    ),
                ])],
            )
        };
        // Pallet 9's error 3, and nothing else, is the one this metadata names.
        let metadata = |pallet: u8, error: u8| {
            (pallet == 9 && error == 3).then(|| {
                (
                    "Drc369Royalties".to_string(),
                    "PaymentCannotBeReceived".to_string(),
                )
            })
        };

        let named = refusal_of(&module(9, 3), metadata);
        assert_eq!(
            named,
            refused("Drc369Royalties", "PaymentCannotBeReceived").unwrap()
        );
        assert!(named.is_unreceivable());
        assert_eq!(
            named.to_string(),
            "Drc369Royalties::PaymentCannotBeReceived"
        );

        // An error this metadata does not name is still said, by its numbers,
        // and is not mistaken for one it does.
        let unknown = refusal_of(&module(9, 4), metadata);
        assert_eq!(unknown.to_string(), "pallet 9::error 4");
        assert!(!unknown.is_unreceivable());
        assert_eq!(
            refusal_of(&module(3, 9), metadata).to_string(),
            "pallet 3::error 9"
        );
        // An index that is not a byte is not one.
        assert_eq!(refusal_of(&module(265, 3), metadata).to_string(), "Module");

        let funds = refusal_of(
            &Value::unnamed_variant("Token", [Value::unnamed_variant("FundsUnavailable", [])]),
            metadata,
        );
        assert_eq!(funds.to_string(), "Token::FundsUnavailable");
        assert!(funds.is_funds());
        assert!(!funds.is_unreceivable());

        assert_eq!(
            refusal_of(&Value::unnamed_variant("BadOrigin", []), metadata).to_string(),
            "BadOrigin"
        );
        assert_eq!(
            refusal_of(&Value::u128(1), metadata).to_string(),
            "an error the launcher cannot read"
        );
    }

    /// The chain's answer is decoded by name, and used only when its parts are
    /// the price. AGENTS.md §5: the one money intermediate the launcher forms
    /// here is that sum, and it is checked, so the largest answer a `u128` can
    /// carry is read and one past it is refused, never wrapped or panicked on.
    #[test]
    fn the_chains_answer_is_used_only_when_its_parts_are_the_price() {
        let read = |remix: u128, royalty: u128, seller: u128| PreviewRead {
            source: Some((3, 1)),
            remix: vec![(AccountId32(SOURCE), remix)],
            royalties: vec![(AccountId32(CREATOR), royalty)],
            seller: AccountId32(SELLER),
            seller_receives: seller,
            refusal: Some(Value::unnamed_variant(
                "Token",
                [Value::unnamed_variant("FundsUnavailable", [])],
            )),
        };
        let nothing_named = |_: u8, _: u8| None;

        let preview = Preview::from_read(
            read(c(150), c(285), c(2_565)),
            c(3_000),
            Some(BUYER),
            nothing_named,
        )
        .unwrap();
        assert_eq!(
            preview,
            Preview {
                price: c(3_000),
                buyer: Some(BUYER),
                source: Some((3, 1)),
                remix: vec![(SOURCE, c(150))],
                royalties: vec![(CREATOR, c(285))],
                seller: SELLER,
                seller_receives: c(2_565),
                refusal: refused("Token", "FundsUnavailable"),
            }
        );
        assert_eq!(
            preview.parts().collect::<Vec<_>>(),
            vec![(SOURCE, c(150)), (CREATOR, c(285)), (SELLER, c(2_565))],
            "the chain's order"
        );

        // A Spark out either way is a misreading, and nobody is shown it.
        for price in [c(3_000) - 1, c(3_000) + 1] {
            let error =
                Preview::from_read(read(c(150), c(285), c(2_565)), price, None, nothing_named)
                    .unwrap_err();
            assert_eq!(error.kind(), "rpc");
            assert!(error.to_string().contains("does not add up"), "{error}");
        }

        // The largest price there is, all of it to one part, reads; parts that
        // sum past it are refused rather than wrapped.
        assert!(Preview::from_read(read(u128::MAX, 0, 0), u128::MAX, None, nothing_named).is_ok());
        assert!(Preview::from_read(read(0, 0, u128::MAX), u128::MAX, None, nothing_named).is_ok());
        let past = Preview::from_read(read(u128::MAX, 1, 0), u128::MAX, None, nothing_named);
        assert_eq!(past.unwrap_err().kind(), "rpc");
        let wrapped = Preview::from_read(read(u128::MAX, 1, 0), 0, None, nothing_named);
        assert_eq!(wrapped.unwrap_err().kind(), "rpc");
    }

    /// A part its recipient cannot receive stops every buyer. The chain says
    /// so; the launcher names the account and the amount from the balances it
    /// read at the same block, and blocks the sale before anyone is asked.
    #[test]
    fn a_part_its_recipient_cannot_receive_blocks_the_sale_in_words() {
        // CREATOR does not exist, and is owed 50 from a sale at 500.
        let unreceivable = || refused("Drc369Royalties", "PaymentCannotBeReceived");
        let mut state = SaleState {
            source_terms: None,
            free: vec![(SELLER, c(500))],
            ..listed_at(
                c(500),
                chain_says(
                    c(500),
                    Some(BUYER),
                    &[],
                    &[(CREATOR, c(50))],
                    c(450),
                    unreceivable(),
                ),
            )
        };

        let blocked = state
            .breakdown()
            .unwrap()
            .blocked
            .expect("a part that cannot be received");
        assert!(blocked.contains(&address_of(&CREATOR)), "{blocked}");
        assert!(blocked.contains("50.00 CGT"), "{blocked}");
        assert!(blocked.contains("100.00 CGT"), "{blocked}");
        assert_eq!(state.cannot_buy(), Some(blocked));

        // The chain says so and the balances read do not show who: the chain
        // is still believed, in the words a refused transaction gets.
        state.free = vec![(SELLER, c(500)), (CREATOR, ED)];
        let blocked = state.breakdown().unwrap().blocked.unwrap();
        assert!(blocked.contains("cannot receive its part"), "{blocked}");

        // The chain sees nothing wrong: nothing is blocked, whatever the
        // balances read suggest.
        state.free = vec![(SELLER, c(500))];
        state.preview.as_mut().unwrap().refusal = None;
        assert_eq!(state.breakdown().unwrap().blocked, None);
        assert_eq!(state.cannot_buy(), None);

        // An account paid twice has its first part when its second arrives, as
        // on chain: CREATOR, who does not exist, is opened by 150 from the
        // source's terms, and the 28.5 its own royalty adds is then fine.
        let twice = chain_says(
            c(300),
            Some(BUYER),
            &[(CREATOR, c(150))],
            &[(CREATOR, c(28) + SPARKS_PER_CGT / 2)],
            c(121) + SPARKS_PER_CGT / 2,
            unreceivable(),
        );
        assert_eq!(state.unreceivable(&twice), None);
        // A part of nothing moves nothing, and a part to the buyer stays put.
        let dust = chain_says(9, Some(BUYER), &[], &[(CREATOR, 0)], 9, unreceivable());
        assert_eq!(
            SaleState {
                free: vec![(SELLER, c(500))],
                ..state.clone()
            }
            .unreceivable(&dust),
            None
        );
        let to_buyer = chain_says(
            c(500),
            Some(CREATOR),
            &[],
            &[(CREATOR, c(50))],
            c(450),
            unreceivable(),
        );
        assert_eq!(state.unreceivable(&to_buyer), None);
    }

    /// The chain's other refusals are the buyer's to read, not the sale's: a
    /// seller may list an asset that is nested, as the chain lets them, and a
    /// buyer who cannot pay does not stop anyone else.
    #[test]
    fn only_what_stops_every_buyer_blocks_the_sale() {
        for refusal in [
            refused("Nfts", "ItemLocked"),
            refused("Token", "FundsUnavailable"),
            refused("BadOrigin", ""),
        ] {
            let state = listed_at(
                c(3_000),
                Preview {
                    refusal: refusal.clone(),
                    ..listed().preview.unwrap()
                },
            );
            assert_eq!(state.breakdown().unwrap().blocked, None, "{refusal:?}");
            assert!(state.cannot_buy().is_some(), "{refusal:?}");
        }
    }

    /// What the chain refuses a buyer is said in words: the launcher's fuller
    /// sentence where what it read explains it, the sentence a refused
    /// transaction gets where it does not, and the chain's own name for
    /// anything else.
    #[test]
    fn the_chains_refusal_of_a_buyer_is_said_in_words() {
        let refusing = |refusal: Option<Refusal>, funds: Funds| SaleState {
            viewer: Some((BUYER, funds)),
            ..listed_at(
                c(1_000),
                chain_says(
                    c(1_000),
                    Some(BUYER),
                    &[(SOURCE, c(50))],
                    &[(CREATOR, c(95))],
                    c(855),
                    refusal,
                ),
            )
        };

        // Short of the price, and the existential deposit on top of it.
        let short = Funds {
            free: c(1_050),
            reserved: 0,
            frozen: 0,
        };
        for name in ["FundsUnavailable", "InsufficientBalance", "NotExpendable"] {
            let words = refusing(refused("Token", name), short)
                .cannot_buy()
                .unwrap();
            assert!(words.contains("cannot cover it"), "{name}: {words}");
            assert!(words.contains("1,000.00 CGT"), "{words}");
            assert!(words.contains("950.00 CGT"), "{words}");
            assert!(words.contains("100.00 CGT"), "{words}");
        }
        // The chain says the buyer cannot pay, and the balance read says they
        // can: the chain is believed, in a refused transaction's words.
        let words = refusing(refused("Token", "FundsUnavailable"), rich())
            .cannot_buy()
            .unwrap();
        assert!(words.contains("could not cover the price"), "{words}");
        // And the balance read says they cannot, but the chain says they can:
        // the chain is believed again.
        assert_eq!(refusing(None, short).cannot_buy(), None);

        // Held in place by nesting: the launcher's sentence names the parent.
        let mut nested = refusing(refused("Nfts", "ItemLocked"), rich());
        nested.held_in_place = held_in_place(Some((4, 9)), 0);
        let words = nested.cannot_buy().unwrap();
        assert!(words.contains("nested inside asset 4/9"), "{words}");
        nested.held_in_place = None;
        let words = nested.cannot_buy().unwrap();
        assert!(words.contains("nested inside another"), "{words}");

        // Anything the launcher has no words for is said by the chain's name.
        let words = refusing(refused("BadOrigin", ""), rich())
            .cannot_buy()
            .unwrap();
        assert_eq!(words, "The chain would refuse this sale: BadOrigin.");
        let words = refusing(refused("pallet 9", "error 4"), rich())
            .cannot_buy()
            .unwrap();
        assert!(words.contains("pallet 9::error 4"), "{words}");
    }

    /// A part owed to the buyer themselves never leaves their account, so it is
    /// marked, and it is not counted against them when the chain says they are
    /// short.
    #[test]
    fn a_part_owed_to_the_buyer_stays_with_them() {
        let owed = |buyer: Account, funds: Funds| SaleState {
            source_terms: None,
            viewer: Some((buyer, funds)),
            ..listed_at(
                c(1_000),
                chain_says(
                    c(1_000),
                    Some(buyer),
                    &[],
                    &[(CREATOR, c(100))],
                    c(900),
                    refused("Token", "FundsUnavailable"),
                ),
            )
        };
        let exactly = Funds {
            free: c(1_000),
            reserved: 0,
            frozen: 0,
        };

        let creator = owed(CREATOR, exactly);
        let breakdown = creator.breakdown().unwrap();
        assert!(breakdown.payouts[0].to_buyer);
        assert!(!breakdown.payouts[1].to_buyer);

        // 900 leaves the account, not 1,000: with 1,000 free and 100 to keep,
        // the buyer who is owed the royalty is not short by the launcher's
        // reading, and a stranger is, by 1,000.
        assert!(creator
            .cannot_buy()
            .unwrap()
            .contains("could not cover the price"));
        let refused = owed(BUYER, exactly).cannot_buy().unwrap();
        assert!(refused.contains("1,000.00 CGT"), "{refused}");
        assert!(refused.contains("900.00 CGT"), "{refused}");
    }

    /// Everything the chain is not asked — whether it is listed, by whom, and
    /// whether the listing is live — is said by the launcher, first.
    #[test]
    fn what_stops_a_purchase_is_said_in_words() {
        let listed = listed();
        assert_eq!(listed.cannot_buy(), None);

        let unlisted = SaleState {
            listing: None,
            ..listed.clone()
        };
        assert!(unlisted.cannot_buy().unwrap().contains("not listed"));

        let own = SaleState {
            viewer: Some((SELLER, rich())),
            ..listed.clone()
        };
        assert!(own.cannot_buy().unwrap().contains("your own listing"));

        // Listed by SELLER, since sent to CREATOR: void for everyone.
        let void = SaleState {
            holder: CREATOR,
            ..listed.clone()
        };
        assert!(void.live_listing().is_none());
        assert!(void.cannot_buy().unwrap().contains("void"));
        assert!(SaleState {
            viewer: Some((CREATOR, rich())),
            ..void
        }
        .cannot_buy()
        .unwrap()
        .contains("hold this asset already"));

        // An answer that is not about this purchase is not used as if it were:
        // none at all, another buyer's, or another price's.
        let answer = listed.preview.clone().unwrap();
        for preview in [
            None,
            Some(Preview {
                buyer: Some(CREATOR),
                ..answer.clone()
            }),
            Some(Preview {
                buyer: None,
                ..answer.clone()
            }),
            Some(Preview {
                price: c(2_000),
                ..answer
            }),
        ] {
            let state = SaleState {
                preview,
                ..listed.clone()
            };
            assert!(state
                .cannot_buy()
                .unwrap()
                .contains("could not ask the chain"));
        }

        // Nobody asking: nothing is said about buying.
        let anonymous = SaleState {
            viewer: None,
            ..listed
        };
        assert_eq!(anonymous.cannot_buy(), None);
    }

    /// A nested asset, or one holding nested assets, can be listed and cannot be
    /// sold (`ItemLocked`, ADR-065): the chain says so, the buyer is told why
    /// before anyone is asked, and its own seller is still told it is their
    /// listing.
    #[test]
    fn an_asset_held_in_place_cannot_be_bought_and_says_why() {
        assert_eq!(held_in_place(None, 0), None);
        let mut state = listed_at(
            c(1_000),
            chain_says(
                c(1_000),
                Some(BUYER),
                &[],
                &[],
                c(1_000),
                refused("Nfts", "ItemLocked"),
            ),
        );
        state.held_in_place = held_in_place(Some((4, 9)), 0);
        let refused = state.cannot_buy().unwrap();
        assert!(refused.contains("nested inside asset 4/9"), "{refused}");
        let root = state.asset.current.root.clone();
        assert_eq!(
            state.refuses(Seen {
                price_sparks: c(1_000),
                root: &root
            }),
            Some(refused)
        );
        assert!(SaleState {
            viewer: Some((SELLER, rich())),
            ..state.clone()
        }
        .cannot_buy()
        .unwrap()
        .contains("your own listing"));

        state.held_in_place = held_in_place(None, 1);
        assert!(state.cannot_buy().unwrap().contains("holds another asset"));
        assert!(held_in_place(None, 3).unwrap().contains("holds 3 assets"));
    }

    /// Nobody is asked to approve a price they did not see, or bytes they did not
    /// see: what was on the buyer's screen is held against the chain.
    #[test]
    fn a_purchase_is_refused_when_what_was_seen_has_changed() {
        let price = c(3_000);
        let state = listed();
        let root = state.asset.current.root.clone();
        fn seen(price_sparks: u128, root: &str) -> Seen<'_> {
            Seen { price_sparks, root }
        }

        assert_eq!(state.refuses(seen(price, &root)), None);
        assert_eq!(
            state.refuses(seen(price, &root.to_uppercase())),
            None,
            "a fingerprint is the same in either case"
        );

        // Looked while it cost half as much, and while it cost twice as much:
        // neither is the price they would be approving.
        for looked_at in [price / 2, price * 2] {
            let words = state.refuses(seen(looked_at, &root)).unwrap();
            assert!(words.contains("The price changed since you looked"));
            assert!(words.contains("3,000.00 CGT"), "{words}");
            assert!(words.contains(&with_symbol(looked_at)), "{words}");
        }

        let words = state.refuses(seen(price, &"0".repeat(64))).unwrap();
        assert!(words.contains("revised since you looked"), "{words}");

        // What stops any purchase is said first, and so is the chain's refusal.
        assert!(SaleState {
            viewer: Some((SELLER, rich())),
            ..state.clone()
        }
        .refuses(seen(price / 2, &root))
        .unwrap()
        .contains("your own listing"));
        let mut locked = state.clone();
        locked.preview.as_mut().unwrap().refusal = refused("Nfts", "ItemLocked");
        assert!(locked
            .refuses(seen(price / 2, &root))
            .unwrap()
            .contains("nested inside another"));
    }

    /// A node that cannot answer what a sale pays is refused in words that say
    /// which runtime it runs and which it needs.
    #[test]
    fn a_node_without_the_preview_is_refused_in_words() {
        let error = cannot_preview(5);
        assert_eq!(error.kind(), "qontrol");
        let words = error.to_string();
        assert!(words.contains("spec version 5"), "{words}");
        assert!(words.contains("spec version 6 or later"), "{words}");
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
        let state = listed();
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
        // Not listed yet: the chain was asked about 3,000 CGT for no buyer.
        let state = SaleState {
            listing: None,
            viewer: None,
            preview: Some(chain_says(
                c(3_000),
                None,
                &[(SOURCE, c(150))],
                &[(CREATOR, c(285))],
                c(2_565),
                None,
            )),
            ..listed()
        };
        let breakdown = state.breakdown().unwrap();
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
            preview: Some(Preview {
                source: None,
                ..chain_says(c(1), None, &[], &[], c(1), None)
            }),
            ..state
        };
        let prompt = list_prompt(
            &plain.asset,
            &plain.breakdown().unwrap(),
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
        let state = listed();
        let breakdown = state.breakdown().unwrap();
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
            &SaleState {
                viewer: Some((CREATOR, rich())),
                ..state.clone()
            }
            .breakdown()
            .unwrap(),
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
            ("ItemLocked", "nested inside another"),
            ("FundsUnavailable", "could not cover the price"),
            ("InsufficientBalance", "could not cover the price"),
            ("NotExpendable", "could not cover the price"),
            (
                "ContentChanged",
                "no longer holds the content you looked at",
            ),
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
