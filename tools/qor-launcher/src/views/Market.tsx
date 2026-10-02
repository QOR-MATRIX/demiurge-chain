/**
 * Market: every asset offered for sale on the chain (L7.2's first slice).
 *
 * # Where the list comes from, and what it is not
 *
 * **There is no indexer** (ADR-028, M5.4). The host reads the list the only way
 * a node can give it: it walks the chain's own listing storage
 * (`Drc369Royalties::Listings`) through the node the launcher is connected to,
 * at that node's latest finalised block, and reads every asset's details at
 * the same block (`src-tauri/src/chain/market.rs`). So this is the connected
 * node's view of the chain. It has **no search, no history and no "newest"**:
 * a listing on chain is a seller and a price, and carries no time. The screen
 * says so behind its information icon, and names the block and the node.
 *
 * The walk is **bounded**. Past the host's bound the chain holds more than one
 * look reads, and the screen says that plainly instead of drawing a list that
 * looks complete. A listing whose asset cannot be read is counted and said,
 * not silently dropped.
 *
 * # What this view decides, and what it does not
 *
 * Nothing about money. The order, the filter, the counts, the window, whether
 * a listing can be bought and why not are all the host's; this draws them in
 * the order they came. A listing that cannot be bought — void, or held in
 * place by nesting (ADR-065) — **is shown where it falls, with the host's
 * reason and no Buy**, never hidden.
 *
 * Buying is not here. Buy asks the host for the asset again (`sales.find`,
 * the same read the Inventory's "Buy an asset" makes) and opens the same
 * `BuyDialog`, which shows what the sale pays and sends the price and the
 * fingerprint that were on screen. Withdraw is the Inventory's `sales.unlist`.
 * After either, the Market is read from the chain again rather than adjusted
 * here.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { ChevronLeft, ChevronRight, Info, RefreshCw } from 'lucide-react';

import {
  asQorError,
  explain,
  market,
  MARKET_PAGE,
  sales,
  type MarketListing,
  type MarketOrder,
  type MarketPage,
  type MarketShow,
  type SaleView,
} from '../lib/ipc';
import { selectActiveAccount, useQor } from '../state/store';
import { AssetCard } from '../qfx/AssetCard';
import { BuyDialog } from './BuyDialog';
import { Surface } from '../components/ui/Surface';
import { ViewHeader } from './parts';

type Look =
  | { state: 'reading' }
  | { state: 'read'; page: MarketPage }
  | { state: 'failed'; reason: string };

const SHOWS: { id: MarketShow; label: string }[] = [
  { id: 'all', label: 'Everyone’s' },
  { id: 'others', label: 'Others’' },
  { id: 'yours', label: 'Yours' },
];

const ORDERS: { id: MarketOrder; label: string }[] = [
  { id: 'price_low', label: 'Cheapest first' },
  { id: 'price_high', label: 'Dearest first' },
  { id: 'number', label: 'By asset number' },
];

const WHERE_FROM =
  'Every listing the chain holds, read through the node you are connected to, at its latest ' +
  'finalised block. There is no indexer yet (ADR-028), so this is that node’s view of the ' +
  'chain: there is no search and no history, and nothing can be sorted by newest, because a ' +
  'listing on chain is a seller and a price and carries no time. Each look reads a bounded ' +
  'number of listings, and says so when the chain holds more.';

/** A host error in the host's own words, with the friendlier line first when there is one. */
function inWords(error: unknown): string {
  const said = asQorError(error).message;
  const friendly = explain(error);
  return friendly === said ? said : `${friendly} ${said}`;
}

export function Market() {
  const account = useQor(selectActiveAccount);
  const token = useQor((s) => s.token);
  const notify = useQor((s) => s.notify);
  const symbol = token?.symbol ?? 'CGT';
  const viewer = account?.address ?? null;

  const [show, setShow] = useState<MarketShow>('all');
  const [order, setOrder] = useState<MarketOrder>('price_low');
  const [offset, setOffset] = useState(0);
  const [look, setLook] = useState<Look>({ state: 'reading' });
  const [reading, setReading] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [buying, setBuying] = useState<SaleView | null>(null);
  const [problem, setProblem] = useState<string | null>(null);
  /** Only the latest look is drawn: an answer to an older one is dropped. */
  const asked = useRef(0);

  const read = useCallback(async () => {
    const mine = ++asked.current;
    setReading(true);
    try {
      const page = await market.page(viewer, show, order, offset, MARKET_PAGE);
      if (mine !== asked.current) return;
      // A window past the end (the last listing on the last page was just
      // withdrawn or bought): go to the last page that has anything on it.
      if (page.matching > 0 && offset >= page.matching) {
        setOffset(Math.floor((page.matching - 1) / MARKET_PAGE) * MARKET_PAGE);
        return;
      }
      setLook({ state: 'read', page });
    } catch (error) {
      if (mine !== asked.current) return;
      setLook({ state: 'failed', reason: inWords(error) });
    } finally {
      if (mine === asked.current) setReading(false);
    }
  }, [viewer, show, order, offset]);

  useEffect(() => {
    void read();
  }, [read]);

  const choose = (next: { show?: MarketShow; order?: MarketOrder }) => {
    if (next.show) setShow(next.show);
    if (next.order) setOrder(next.order);
    setOffset(0);
  };

  /** Buy: the host reads the asset again, and the one purchase dialog opens on that. */
  const openBuy = async (listing: MarketListing) => {
    const { asset } = listing;
    const id = `${asset.collection}/${asset.item}`;
    setBusy(id);
    setProblem(null);
    try {
      const sale = await sales.find(id, viewer);
      const was = asset.listing?.price_sparks;
      const now = sale.asset.listing?.price_sparks;
      if (was && now && was !== now) {
        notify(
          'bad',
          `The price changed since this list was read: it is ${sale.asset.listing?.price_cgt} ${symbol} now. The purchase screen shows the chain’s price.`,
        );
      }
      setBuying(sale);
    } catch (error) {
      setProblem(`Could not open the purchase of asset ${id}: ${inWords(error)}`);
      void read();
    } finally {
      setBusy(null);
    }
  };

  /** Withdraw this account's listing, or clear a void one on what it holds. The host asks first. */
  const withdraw = async (listing: MarketListing) => {
    if (!account) return;
    const { asset } = listing;
    setBusy(`${asset.collection}/${asset.item}`);
    setProblem(null);
    try {
      await sales.unlist(account.address, asset.collection, asset.item);
      notify(
        'ok',
        listing.standing === 'void'
          ? 'The void listing is cleared.'
          : `"${asset.name || 'The asset'}" is no longer for sale.`,
      );
    } catch (error) {
      notify('bad', explain(error));
    } finally {
      // Whatever happened, draw what the chain now says.
      await read();
      setBusy(null);
    }
  };

  const page = look.state === 'read' ? look.page : null;
  const last = page ? Math.min(page.offset + MARKET_PAGE, page.matching) : 0;

  return (
    <div className="relative flex h-full flex-col overflow-y-auto">
      <ViewHeader
        eyebrow="Exchange"
        title="Market"
        body={WHERE_FROM}
        action={
          <button type="button" className="btn" onClick={() => void read()} data-market-refresh>
            <RefreshCw size={13} className={reading ? 'animate-spin' : ''} />
            Refresh
          </button>
        }
      />

      <div className="p-8">
        <div className="mb-5 flex flex-wrap items-center gap-x-6 gap-y-3">
          <div className="flex flex-wrap items-center gap-2">
            <span className="eyebrow text-ink-muted" id="market-show">
              Listed by
            </span>
            <div className="flex flex-wrap gap-2" role="group" aria-labelledby="market-show">
              {SHOWS.map((one) => (
                <button
                  key={one.id}
                  type="button"
                  className={`btn whitespace-nowrap ${show === one.id ? 'btn-primary' : ''}`}
                  aria-pressed={show === one.id}
                  onClick={() => choose({ show: one.id })}
                  data-market-show={one.id}
                >
                  {one.label}
                </button>
              ))}
            </div>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <span className="eyebrow text-ink-muted" id="market-order">
              Order
            </span>
            <div className="flex flex-wrap gap-2" role="group" aria-labelledby="market-order">
              {ORDERS.map((one) => (
                <button
                  key={one.id}
                  type="button"
                  className={`btn whitespace-nowrap ${order === one.id ? 'btn-primary' : ''}`}
                  aria-pressed={order === one.id}
                  onClick={() => choose({ order: one.id })}
                  data-market-order={one.id}
                >
                  {one.label}
                </button>
              ))}
            </div>
          </div>
        </div>

        <div aria-live="polite">
          {problem && (
            <p className="mb-4 text-caption text-bad" role="alert" data-market-problem>
              {problem}
            </p>
          )}
        </div>

        {look.state === 'reading' ? (
          <Empty>Reading the chain…</Empty>
        ) : look.state === 'failed' ? (
          <Empty data="failed">Could not read what is for sale. {look.reason}</Empty>
        ) : (
          page && (
            <>
              <p className="mb-1 text-caption text-ink-body" data-market-counts>
                {page.matching === 0
                  ? 'No listings match.'
                  : `Listings ${page.offset + 1}–${last} of ${page.matching}`}
                <span className="text-ink-muted">
                  {' '}
                  · {page.on_chain} read from the chain, {page.yours} of them yours
                </span>
              </p>
              <p className="numeric mb-4 text-micro text-ink-faint" data-market-source>
                Finalised block {page.block_number} on {page.chain_name}, through{' '}
                {page.endpoint}
              </p>

              {page.truncated && (
                <div className="mb-4" data-market-truncated>
                  <Surface className="flex items-start gap-3 border-l-2 border-l-bad px-5 py-4">
                    <Info size={15} className="mt-0.5 flex-none text-bad" />
                    <p className="text-caption text-ink-body">
                      The chain holds more than {page.bound} listings, and one look reads no more
                      than that. Only the first {page.bound}, in the order the node stores them —
                      not by price and not by time — were read, so the counts, the order and the
                      pages cover those alone. A catalogue this size needs the indexer, which is
                      not built yet (M5.4).
                    </p>
                  </Surface>
                </div>
              )}

              {page.unreadable > 0 && (
                <p className="mb-4 text-caption text-ink-muted" data-market-unreadable>
                  {page.unreadable === 1
                    ? '1 listing on this page names an asset'
                    : `${page.unreadable} listings on this page name assets`}{' '}
                  that could not be read as DRC-369 assets at this block (burned, or never one),
                  so there is nothing true to draw for{' '}
                  {page.unreadable === 1 ? 'it' : 'them'}.
                </p>
              )}

              {page.matching === 0 ? (
                <Empty data="empty">
                  {show === 'yours'
                    ? 'You have nothing listed. List an asset from its menu in Inventory.'
                    : show === 'others'
                      ? 'Nobody else has anything listed on this chain.'
                      : 'Nothing is listed for sale on this chain.'}
                </Empty>
              ) : (
                <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3" data-market-grid>
                  {page.listings.map((listing, i) => {
                    const id = `${listing.asset.collection}/${listing.asset.item}`;
                    return (
                      <AssetCard
                        key={id}
                        asset={listing.asset}
                        index={i}
                        busy={busy === id}
                        symbol={symbol}
                        listed={{
                          listing,
                          onBuy: () => void openBuy(listing),
                          onWithdraw: () => void withdraw(listing),
                        }}
                      />
                    );
                  })}
                </div>
              )}

              {page.matching > MARKET_PAGE && (
                <nav
                  aria-label="Pages of listings"
                  className="mt-6 flex flex-wrap items-center gap-3"
                  data-market-pages
                >
                  <button
                    type="button"
                    className="btn whitespace-nowrap"
                    disabled={page.offset === 0 || reading}
                    onClick={() => setOffset(Math.max(0, page.offset - MARKET_PAGE))}
                    data-market-previous
                  >
                    <ChevronLeft size={13} />
                    Previous
                  </button>
                  <button
                    type="button"
                    className="btn whitespace-nowrap"
                    disabled={page.offset + MARKET_PAGE >= page.matching || reading}
                    onClick={() => setOffset(page.offset + MARKET_PAGE)}
                    data-market-next
                  >
                    Next
                    <ChevronRight size={13} />
                  </button>
                </nav>
              )}
            </>
          )
        )}
      </div>

      {buying && account && (
        <BuyDialog
          from={account.address}
          sale={buying}
          symbol={symbol}
          onBought={(receipt) => {
            setBuying(null);
            notify(
              'ok',
              `Bought "${receipt.name || 'the asset'}" for ${receipt.price_cgt} ${symbol}. It is in your Inventory.`,
            );
            void read();
          }}
          onClose={() => {
            // The dialog may have read the asset again after a refusal, so the
            // list under it is read again too rather than left showing less.
            setBuying(null);
            void read();
          }}
        />
      )}
    </div>
  );
}

function Empty({ children, data }: { children: React.ReactNode; data?: string }) {
  return (
    <div data-market-empty={data ?? 'reading'}>
      <Surface className="flex items-center gap-3 px-5 py-8">
        <Info size={15} className="flex-none text-ink-faint" />
        <p className="text-ui text-ink-muted">{children}</p>
      </Surface>
    </div>
  );
}
