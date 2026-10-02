//! The Market: every asset offered for sale on the chain, read through the node
//! the launcher is connected to (L7.2's first slice).
//!
//! # Where the list comes from, said plainly
//!
//! **There is no indexer** (ADR-028, roadmap M5.4). So the list of what is for
//! sale is read the only way a node can give it: by walking
//! `Drc369Royalties::Listings`, the chain's own listing storage, at the
//! finalised block the node reports when it is asked. That is the same block
//! every other read in this client uses (`at_current_block` is the latest
//! finalised block), and the whole of one look — the walk and every asset's
//! details — is read at that one block, so a look never mixes two states.
//!
//! This is what a node can serve and nothing more. It is the connected node's
//! view of the chain; it has no search, no history and no "newest", because a
//! listing on chain is a seller and a price and carries no time. It is replaced
//! by the indexer when there is one, and **the replacement is one function**:
//! [`ChainClient::listed`], which returns the rows. Everything after it —
//! arranging, the window, classifying — works on rows and does not care where
//! they came from.
//!
//! # Paged, and bounded
//!
//! - The walk is a stream that asks the node for a page of keys at a time (the
//!   client library's page, 64 keys on the legacy RPC methods), and it **stops
//!   at [`SCAN_BOUND`] listings**. A chain with more says so
//!   ([`MarketPage::truncated`]) instead of being read without limit.
//! - A row is a seller and a price: enough to arrange and to count. An asset's
//!   details cost several more reads each, so they are read **only for the
//!   window that was asked for**, [`PAGE`] at a time by default and never more
//!   than [`SCAN_BOUND`] in one look.
//! - It runs in the host's async runtime, never on the interface's thread.
//!
//! # What can be bought, and what is shown anyway
//!
//! A listing on chain is not always one a person can buy. Two kinds cannot be,
//! and **both are shown, in place, without Buy and with the reason in words**,
//! rather than left out: the chain holds them, a count that skipped them would
//! not add up, and the person who can fix each one needs to see it.
//!
//! - **Void**: whoever listed the asset no longer holds it (`ListingStale`).
//! - **Held in place**: the asset is nested inside another, or holds nested
//!   assets, so the chain will not move it (`ItemLocked`, ADR-065).
//!
//! A person's own listings are marked as theirs and offer Withdraw. Buying is
//! not here: Buy opens the same dialog and the same host-side confirmation as
//! everywhere else ([`super::sales`]), which reads the asset again and checks
//! the price, the fingerprint and the balance before anyone is asked.
//!
//! # What is not known, and is not invented
//!
//! The chain holds an asset's name, its content reference, whether it is
//! permanent, and what it was remixed from. It holds no description, category
//! or picture: a seller's drafted description stays on their machine
//! (`listings.rs`). So a Market card says what the chain says.

use scale_decode::DecodeAsType;
use serde::{Deserialize, Serialize};
use subxt::dynamic;
use subxt::utils::AccountId32;

use crate::cgt;
use crate::error::{QorError, QorResult};
use crate::vault::derive::address_of;
use crate::vault::normalise_address;

use super::assets::{
    CommitIdRead, CommitView, ContentRefRead, ContentView, ItemMetadataRead, OwnedAsset, TradeItem,
};
use super::sales::{held_in_place, ListingView};
use super::{ChainClient, Connection};

/// An account, as the chain keys it.
type Account = [u8; 32];

/// How many listings one look shows before a person asks for more.
///
/// Twenty-four fills a grid of two, three or four columns with whole rows.
/// An engineering choice about a screen, not a chain limit and not an economic
/// value.
pub const PAGE: usize = 24;

/// The most listings one look will read from the node, in the walk and in one
/// window.
///
/// The walk is the only read here whose size the chain decides, so it has a
/// ceiling: past this many the Market says the chain holds more than it shows,
/// and that a catalogue of that size needs the indexer (M5.4). Five hundred is
/// a few pages of keys for the node and far more than a development chain
/// holds. An engineering bound; raising it is a decision with a measurement
/// behind it.
pub const SCAN_BOUND: usize = 500;

// ── What the node holds, decoded by name ─────────────────────────────────────

/// `Drc369Royalties::Listings`' value.
#[derive(Debug, DecodeAsType)]
struct ListingRead {
    seller: AccountId32,
    price: u128,
}

/// The part of `Nfts::Item` that says who holds an item.
#[derive(Debug, DecodeAsType)]
struct ItemRead {
    owner: AccountId32,
}

/// `Drc369::Assets`' value.
#[derive(Debug, DecodeAsType)]
struct RecordRead {
    origin: ContentRefRead,
    current: ContentRefRead,
    commit: Option<CommitIdRead>,
    revisable: bool,
    derived_from: Option<(u32, u32)>,
}

// ── Rows, and what is done with them ─────────────────────────────────────────

/// One listing as the chain's storage holds it: an asset, who listed it, and
/// the price in Sparks. What [`ChainClient::listed`] returns, and all an
/// indexer would have to return in its place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Row {
    pub collection: u32,
    pub item: u32,
    pub seller: Account,
    pub price: u128,
}

/// The rows a walk collected, and whether it stopped at its bound.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Scan {
    pub rows: Vec<Row>,
    /// The chain holds more listings than the bound; the rest were not read.
    pub truncated: bool,
    bound: usize,
}

impl Scan {
    pub(crate) fn bounded_at(bound: usize) -> Self {
        Self {
            rows: Vec::new(),
            truncated: false,
            bound,
        }
    }

    /// Take one more row. `false` once the bound is reached: the walk stops
    /// there, and the row that did not fit is not kept.
    pub(crate) fn offer(&mut self, row: Row) -> bool {
        if self.rows.len() >= self.bound {
            self.truncated = true;
            return false;
        }
        self.rows.push(row);
        true
    }
}

/// The order listings are shown in.
///
/// There is no "newest": a listing on chain carries no time, and an asset's
/// number says when its collection was made, not when it was listed. Offering
/// it would be inventing it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    /// Cheapest first.
    #[default]
    PriceLow,
    /// Dearest first.
    PriceHigh,
    /// By asset number, `collection/item`, ascending.
    Number,
}

/// Whose listings are shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Show {
    #[default]
    All,
    /// Listings made by anyone but the account looking.
    Others,
    /// Listings the account looking made.
    Yours,
}

/// Keep the rows `show` asks for and put them in `order`. Equal prices fall
/// back to the asset's number, so the order never depends on how the node's
/// storage happens to hash.
pub(crate) fn arrange(
    rows: &[Row],
    viewer: Option<&Account>,
    show: Show,
    order: Order,
) -> Vec<Row> {
    let mut kept: Vec<Row> = rows
        .iter()
        .filter(|row| match show {
            Show::All => true,
            Show::Others => viewer != Some(&row.seller),
            Show::Yours => viewer == Some(&row.seller),
        })
        .copied()
        .collect();
    let number = |row: &Row| (row.collection, row.item);
    match order {
        Order::PriceLow => kept.sort_unstable_by_key(|row| (row.price, number(row))),
        Order::PriceHigh => {
            kept.sort_unstable_by_key(|row| (std::cmp::Reverse(row.price), number(row)))
        }
        Order::Number => kept.sort_unstable_by_key(number),
    }
    kept
}

/// The part of `total` rows one look reads details for: from `offset`, at most
/// `limit`, never past the end and never more than [`SCAN_BOUND`]. A `limit`
/// of nothing means one [`PAGE`].
pub(crate) fn window(total: usize, offset: usize, limit: usize) -> std::ops::Range<usize> {
    let limit = if limit == 0 { PAGE } else { limit }.min(SCAN_BOUND);
    let start = offset.min(total);
    start..start.saturating_add(limit).min(total)
}

/// Whether a listing can be bought by the account looking, and if not, why.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Standing {
    /// Someone else's live listing of an asset the chain will move.
    Buyable,
    /// The account looking listed it and still holds it. It can be withdrawn.
    Yours,
    /// Whoever listed it no longer holds it. Nobody can buy from it.
    Void,
    /// Nested, or holding nested assets: listed, and the chain will not move
    /// it (ADR-065).
    HeldInPlace,
}

/// What was read about one listed asset that its standing depends on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Facts {
    pub seller: Account,
    pub holder: Account,
    /// The asset it is nested inside, if it is.
    pub parent: Option<(u32, u32)>,
    /// How many assets are nested directly inside it.
    pub children: u32,
}

/// A listing's standing for `viewer`, and the reason it cannot be bought, in
/// words, when it cannot.
///
/// The order matters. Void comes first: a void listing is nobody's to sell,
/// whoever is looking. Then held in place — unless it is the viewer's own, in
/// which case it is theirs first and the reason is kept beside it, so the one
/// person who can take the asset apart is told why nobody is buying it.
pub(crate) fn standing(facts: &Facts, viewer: Option<&Account>) -> (Standing, Option<String>) {
    if facts.seller != facts.holder {
        return (
            Standing::Void,
            Some(if viewer == Some(&facts.holder) {
                "A listing made by an earlier holder of this asset is still on chain. It is void: \
                 nobody can buy from it. You hold the asset, and you can clear it."
                    .into()
            } else {
                "This listing is void: the account that listed the asset no longer holds it, so \
                 nobody can buy it."
                    .into()
            }),
        );
    }
    let locked = held_in_place(facts.parent, facts.children);
    if viewer == Some(&facts.seller) {
        return (
            Standing::Yours,
            locked.map(|reason| {
                format!("Nobody can buy this while it stays as it is. {reason}")
                    .replace("until its holder takes", "until you take")
            }),
        );
    }
    match locked {
        Some(reason) => (Standing::HeldInPlace, Some(reason)),
        None => (Standing::Buyable, None),
    }
}

// ── What the interface is given ─────────────────────────────────────────────

/// One listing, as a person reads it.
#[derive(Debug, Clone, Serialize)]
pub struct MarketListing {
    /// The asset, with its listing: the name, the content reference, whether
    /// it is permanent, the price and the seller, all as the chain holds them.
    pub asset: OwnedAsset,
    /// Who holds it now, as this chain writes an address. The seller, unless
    /// the listing is void.
    pub holder: String,
    /// The account looking holds it.
    pub held_by_viewer: bool,
    /// The asset it was remixed from, if its minter declared one.
    pub derived_from: Option<TradeItem>,
    pub standing: Standing,
    /// Why it cannot be bought, in words. `None` when it can.
    pub reason: Option<String>,
}

/// One look at the Market.
#[derive(Debug, Clone, Serialize)]
pub struct MarketPage {
    /// The window that was asked for, in order.
    pub listings: Vec<MarketListing>,
    /// Where the window starts among the listings that match.
    pub offset: usize,
    /// How many listings match what was asked for (`show`), among those read.
    pub matching: usize,
    /// How many listings the walk read from the chain, whoever made them.
    pub on_chain: usize,
    /// How many of those the account looking made.
    pub yours: usize,
    /// The chain holds more listings than [`SCAN_BOUND`]; the rest were not
    /// read, and are in none of the counts.
    pub truncated: bool,
    /// The bound the walk stops at, so the interface can say it.
    pub bound: usize,
    /// Listings whose asset could not be read as a DRC-369 asset at this
    /// block (burned, or not one), left out of `listings`.
    pub unreadable: usize,
    /// The finalised block all of this was read at.
    pub block_number: u64,
    pub block_hash: String,
    /// The node it was read through, and what that node calls its chain.
    pub endpoint: String,
    pub chain_name: String,
}

fn rpc(what: &str, error: impl std::fmt::Display) -> QorError {
    QorError::Rpc(format!("{what}: {error}"))
}

impl ChainClient {
    /// Every listing the chain holds, up to `bound`, as rows.
    ///
    /// **This is the function an indexer replaces** (ADR-028, M5.4). Today it
    /// walks `Drc369Royalties::Listings` through the connected node at one
    /// finalised block; the node is asked for a page of keys at a time and the
    /// walk stops at the bound. Nothing else in this module reads the list.
    async fn listed(
        &self,
        storage: &subxt::storage::StorageClient<
            '_,
            super::config::DemiurgeConfig,
            impl subxt::client::OnlineClientAtBlockT<super::config::DemiurgeConfig>,
        >,
        bound: usize,
    ) -> QorResult<Scan> {
        let entry = dynamic::storage::<(u32, u32), ListingRead>("Drc369Royalties", "Listings");
        let mut entries = storage.iter(entry, ()).await.map_err(|e| {
            rpc(
                "this node could not list what is for sale (it may be running a runtime older \
                 than selling)",
                e,
            )
        })?;

        let mut scan = Scan::bounded_at(bound);
        while let Some(entry) = entries.next().await {
            let entry = entry.map_err(|e| rpc("could not read the listings", e))?;
            let key = entry
                .key()
                .map_err(|e| rpc("a listing's key could not be read", e))?;
            let part = |i: usize| -> QorResult<u32> {
                key.part(i)
                    .and_then(|p| p.decode_as::<u32>().ok().flatten())
                    .ok_or_else(|| QorError::Rpc("a listing's key did not name an asset".into()))
            };
            let listing = entry
                .value()
                .decode()
                .map_err(|e| rpc("a listing could not be decoded", e))?;
            let row = Row {
                collection: part(0)?,
                item: part(1)?,
                seller: listing.seller.0,
                price: listing.price,
            };
            if !scan.offer(row) {
                break;
            }
        }
        Ok(scan)
    }

    /// One look at the Market: the listings the chain holds, arranged, and the
    /// details of the window that was asked for. `viewer` is the account
    /// looking, which decides what is theirs.
    pub async fn market(
        &self,
        viewer: Option<&str>,
        show: Show,
        order: Order,
        offset: usize,
        limit: usize,
    ) -> QorResult<MarketPage> {
        let viewer = viewer.map(normalise_address).transpose()?;
        let connection: Connection = self.connected().await?;
        let at = match connection.api.at_current_block().await {
            Ok(at) => at,
            Err(e) => return Err(self.lost_the_node(e).await),
        };
        let storage = at.storage();

        let scan = self.listed(&storage, SCAN_BOUND).await?;
        let yours = scan
            .rows
            .iter()
            .filter(|row| viewer == Some(row.seller))
            .count();
        let arranged = arrange(&scan.rows, viewer.as_ref(), show, order);
        let wanted = window(arranged.len(), offset, limit);
        let start = wanted.start;

        let mut listings = Vec::with_capacity(wanted.len());
        let mut unreadable = 0;
        for row in &arranged[wanted] {
            let id = (row.collection, row.item);
            // Five reads an asset, asked for together: who holds it, what it
            // is, its name, and the two halves of whether it is held in place.
            let (held, record, name, parent, children) = tokio::join!(
                storage.try_fetch(dynamic::storage::<(u32, u32), ItemRead>("Nfts", "Item"), id),
                storage.try_fetch(
                    dynamic::storage::<(u32, u32), RecordRead>("Drc369", "Assets"),
                    id
                ),
                storage.try_fetch(
                    dynamic::storage::<(u32, u32), ItemMetadataRead>("Nfts", "ItemMetadataOf"),
                    id
                ),
                storage.try_fetch(
                    dynamic::storage::<(u32, u32), (u32, u32)>("Drc369", "ParentOf"),
                    id
                ),
                storage.try_fetch(
                    dynamic::storage::<(u32, u32), u32>("Drc369", "ChildCount"),
                    id
                ),
            );

            let held = held.map_err(|e| rpc("could not read who holds a listed asset", e))?;
            let record = record.map_err(|e| rpc("could not read a listed asset", e))?;
            // A listing whose asset is gone, or was never a DRC-369 asset, is
            // counted and left out: there is nothing true to draw for it.
            let (Some(held), Some(record)) = (held, record) else {
                unreadable += 1;
                continue;
            };
            let holder = held
                .decode()
                .map_err(|e| rpc("a listed asset's holder could not be decoded", e))?
                .owner
                .0;
            let record = record
                .decode()
                .map_err(|e| rpc("a listed asset could not be decoded", e))?;
            let name = match name.map_err(|e| rpc("could not read a listed asset's name", e))? {
                Some(value) => String::from_utf8_lossy(
                    &value
                        .decode()
                        .map_err(|e| rpc("a listed asset's name could not be decoded", e))?
                        .data,
                )
                .into_owned(),
                None => String::new(),
            };
            let parent = parent
                .map_err(|e| rpc("could not read a listed asset's nesting", e))?
                .map(|value| value.decode())
                .transpose()
                .map_err(|e| rpc("a listed asset's nesting could not be decoded", e))?;
            let children = children
                .map_err(|e| rpc("could not read a listed asset's nesting", e))?
                .map(|value| value.decode())
                .transpose()
                .map_err(|e| rpc("a listed asset's nesting could not be decoded", e))?
                .unwrap_or(0);

            let facts = Facts {
                seller: row.seller,
                holder,
                parent,
                children,
            };
            let (standing, reason) = standing(&facts, viewer.as_ref());
            listings.push(MarketListing {
                asset: OwnedAsset {
                    collection: row.collection,
                    item: row.item,
                    name,
                    origin: ContentView::from(&record.origin),
                    current: ContentView::from(&record.current),
                    commit: record.commit.as_ref().map(CommitView::from),
                    revisable: record.revisable,
                    listing: Some(ListingView {
                        seller: address_of(&row.seller),
                        price_sparks: row.price.to_string(),
                        price_cgt: cgt::format_cgt_grouped(row.price),
                        void: row.seller != holder,
                    }),
                },
                holder: address_of(&holder),
                held_by_viewer: viewer == Some(holder),
                derived_from: record
                    .derived_from
                    .map(|(collection, item)| TradeItem { collection, item }),
                standing,
                reason,
            });
        }

        Ok(MarketPage {
            listings,
            offset: start,
            matching: arranged.len(),
            on_chain: scan.rows.len(),
            yours,
            truncated: scan.truncated,
            bound: SCAN_BOUND,
            unreadable,
            block_number: at.block_number(),
            block_hash: format!("{:?}", at.block_hash()),
            endpoint: connection.endpoint.clone(),
            chain_name: connection.chain_name.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SELLER: Account = [0x11; 32];
    const VIEWER: Account = [0x22; 32];
    const OTHER: Account = [0x33; 32];

    fn row(collection: u32, item: u32, seller: Account, price: u128) -> Row {
        Row {
            collection,
            item,
            seller,
            price,
        }
    }

    fn numbers(rows: &[Row]) -> Vec<(u32, u32)> {
        rows.iter().map(|r| (r.collection, r.item)).collect()
    }

    /// The walk keeps what fits and stops: it never holds more than its bound,
    /// it says when the chain had more, and it does not say so when the chain
    /// had exactly as many as fit.
    #[test]
    fn the_walk_stops_at_its_bound_and_says_so() {
        let mut scan = Scan::bounded_at(3);
        let mut taken = 0;
        for item in 0..10 {
            if !scan.offer(row(1, item, SELLER, 1)) {
                break;
            }
            taken += 1;
        }
        assert_eq!(taken, 3, "the walk stops asking once the bound is reached");
        assert_eq!(numbers(&scan.rows), [(1, 0), (1, 1), (1, 2)]);
        assert!(scan.truncated);

        // Exactly as many as fit is not "more than fit".
        let mut exact = Scan::bounded_at(3);
        for item in 0..3 {
            assert!(exact.offer(row(1, item, SELLER, 1)));
        }
        assert_eq!(exact.rows.len(), 3);
        assert!(!exact.truncated);

        // And the shipped bound is the one the documents name.
        assert_eq!(SCAN_BOUND, 500);
        assert_eq!(PAGE, 24);
    }

    /// A window is a page of what matched: it starts where it was asked to,
    /// holds at most the limit, never runs past the end, and one look never
    /// reads details for more than the bound.
    #[test]
    fn a_window_is_one_page_and_never_more_than_the_bound() {
        assert_eq!(window(100, 0, 24), 0..24);
        assert_eq!(window(100, 24, 24), 24..48);
        assert_eq!(window(100, 96, 24), 96..100, "the last page is short");
        assert_eq!(window(100, 100, 24), 100..100);
        assert_eq!(window(100, 5_000, 24), 100..100, "past the end is empty");
        assert_eq!(window(0, 0, 24), 0..0);
        assert_eq!(window(100, 0, 0), 0..PAGE, "no limit means one page");
        assert_eq!(window(10_000, 0, 9_999), 0..SCAN_BOUND);
        assert_eq!(window(10_000, 700, usize::MAX), 700..700 + SCAN_BOUND);
        assert_eq!(window(usize::MAX, usize::MAX, usize::MAX).len(), 0);
    }

    /// Order is by price, then by asset number, whatever order storage gave;
    /// the filter is by who listed it.
    #[test]
    fn listings_are_arranged_by_price_and_filtered_by_seller() {
        let rows = [
            row(7, 0, SELLER, 300),
            row(2, 5, VIEWER, 100),
            row(2, 4, OTHER, 300),
            row(9, 1, VIEWER, u128::MAX),
            row(1, 0, SELLER, 100),
        ];
        let all = |order| numbers(&arrange(&rows, Some(&VIEWER), Show::All, order));

        assert_eq!(
            all(Order::PriceLow),
            [(1, 0), (2, 5), (2, 4), (7, 0), (9, 1)],
            "cheapest first, equal prices by number"
        );
        assert_eq!(
            all(Order::PriceHigh),
            [(9, 1), (2, 4), (7, 0), (1, 0), (2, 5)],
            "dearest first, equal prices still by number"
        );
        assert_eq!(all(Order::Number), [(1, 0), (2, 4), (2, 5), (7, 0), (9, 1)]);

        assert_eq!(
            numbers(&arrange(&rows, Some(&VIEWER), Show::Yours, Order::Number)),
            [(2, 5), (9, 1)]
        );
        assert_eq!(
            numbers(&arrange(&rows, Some(&VIEWER), Show::Others, Order::Number)),
            [(1, 0), (2, 4), (7, 0)]
        );
        // Nobody looking: nothing is theirs, and everything is someone else's.
        assert!(arrange(&rows, None, Show::Yours, Order::Number).is_empty());
        assert_eq!(arrange(&rows, None, Show::Others, Order::Number).len(), 5);
    }

    fn facts(seller: Account, holder: Account) -> Facts {
        Facts {
            seller,
            holder,
            parent: None,
            children: 0,
        }
    }

    /// A live listing by someone else, of an asset that stands free, is the
    /// only kind that can be bought.
    #[test]
    fn only_a_live_free_standing_listing_by_someone_else_is_buyable() {
        assert_eq!(
            standing(&facts(SELLER, SELLER), Some(&VIEWER)),
            (Standing::Buyable, None)
        );
        // Nobody looking: still buyable by whoever does look.
        assert_eq!(
            standing(&facts(SELLER, SELLER), None),
            (Standing::Buyable, None)
        );
    }

    /// The viewer's own listing is marked as theirs, never as buyable.
    #[test]
    fn a_listing_the_viewer_made_is_marked_as_theirs() {
        assert_eq!(
            standing(&facts(VIEWER, VIEWER), Some(&VIEWER)),
            (Standing::Yours, None)
        );
        // The same listing is buyable to anyone else.
        assert_eq!(
            standing(&facts(VIEWER, VIEWER), Some(&OTHER)).0,
            Standing::Buyable
        );
    }

    /// A listing whose seller no longer holds the asset is void for everyone:
    /// the buyer, the seller who gave it away, and the account that holds it
    /// now, who is told they can clear it.
    #[test]
    fn a_listing_whose_seller_no_longer_holds_the_asset_is_void() {
        let gone = facts(SELLER, OTHER);
        for viewer in [Some(&VIEWER), Some(&SELLER), None] {
            let (standing, reason) = standing(&gone, viewer);
            assert_eq!(standing, Standing::Void);
            assert!(reason.unwrap().contains("void"));
        }
        let (kind, reason) = standing(&gone, Some(&OTHER));
        assert_eq!(kind, Standing::Void);
        assert!(reason.unwrap().contains("you can clear it"));

        // Void wins over nesting: a void listing is nobody's to sell.
        let nested = Facts {
            parent: Some((4, 9)),
            ..gone
        };
        assert_eq!(standing(&nested, Some(&VIEWER)).0, Standing::Void);
    }

    /// A nested asset, and one holding nested assets, are listed and cannot be
    /// bought (`ItemLocked`, ADR-065): never buyable, with the reason in words.
    /// Their own seller is told it is theirs, and why nobody is buying.
    #[test]
    fn an_asset_held_in_place_is_never_offered_as_buyable() {
        let inside = Facts {
            parent: Some((4, 9)),
            ..facts(SELLER, SELLER)
        };
        let (kind, reason) = standing(&inside, Some(&VIEWER));
        assert_eq!(kind, Standing::HeldInPlace);
        assert!(reason.unwrap().contains("nested inside asset 4/9"));

        let holding = Facts {
            children: 2,
            ..facts(SELLER, SELLER)
        };
        let (kind, reason) = standing(&holding, None);
        assert_eq!(kind, Standing::HeldInPlace);
        assert!(reason.unwrap().contains("holds 2 assets"));

        let (kind, reason) = standing(&inside, Some(&SELLER));
        assert_eq!(kind, Standing::Yours);
        let reason = reason.unwrap();
        assert!(reason.contains("Nobody can buy this"), "{reason}");
        assert!(reason.contains("until you take it out"), "{reason}");
    }

    /// What reaches the interface is named the way the interface reads it.
    #[test]
    fn the_wire_names_are_the_ones_the_interface_reads() {
        let json = |value: Standing| serde_json::to_string(&value).unwrap();
        assert_eq!(json(Standing::Buyable), "\"buyable\"");
        assert_eq!(json(Standing::Yours), "\"yours\"");
        assert_eq!(json(Standing::Void), "\"void\"");
        assert_eq!(json(Standing::HeldInPlace), "\"held_in_place\"");
        assert_eq!(
            serde_json::from_str::<Order>("\"price_high\"").unwrap(),
            Order::PriceHigh
        );
        assert_eq!(
            serde_json::from_str::<Show>("\"others\"").unwrap(),
            Show::Others
        );
    }

    /// Refused before any node is needed, and a node that is not there is said
    /// to be not there.
    #[tokio::test]
    async fn a_look_with_a_bad_address_or_no_node_says_so() {
        let client = ChainClient::new("ws://127.0.0.1:1").unwrap();
        assert_eq!(
            client
                .market(Some("not-an-address"), Show::All, Order::PriceLow, 0, PAGE)
                .await
                .unwrap_err()
                .kind(),
            "bad_address"
        );
        let error = client
            .market(None, Show::All, Order::PriceLow, 0, PAGE)
            .await
            .unwrap_err();
        assert!(matches!(error.kind(), "network" | "rpc"), "{error}");
    }
}
