/**
 * Inventory: the DRC-369 assets an account holds, and its settled history.
 *
 * # What is shown, and where it comes from
 *
 * Each asset is a card (`src/qfx/AssetCard.tsx`, QFX layer two): it leans
 * towards the pointer, carries a mark drawn from its own fingerprint, and opens
 * at size. What it says is still only what the chain holds.
 *
 * **Assets are read from chain storage, every time this view asks** (M4.1): the
 * host walks `pallet-nfts`'s owner index for the account and reads each item's
 * DRC-369 record and name (ADR-052). For each asset the view shows the project
 * name, the content reference the chain holds, the commit it pins and whether
 * it is permanent. It keeps no list of its own: after making an asset permanent
 * it asks the chain again and draws that answer, not what it expected.
 *
 * **A trade leaves this view holding nothing of its own** (L4.5). The dialog
 * sends the whole trade as one transaction; whatever it returns, the assets are
 * read from the chain again, so what disappears from here disappeared on chain.
 *
 * **Selling and buying are on chain, and so is what is drawn of them** (L4.6).
 * A card's price is the listing the chain holds, read with the asset. Listing,
 * withdrawing and buying each go through the host's own dialog, and afterwards
 * the assets are read again rather than adjusted here.
 *
 * **There is no storefront, and this does not draw one.** A catalogue of what
 * is for sale is an indexer over the chain's events (ADR-028, M5.4), which is
 * not built. So buying starts from an asset's number, which its holder gives
 * the buyer: the "Buy an asset" section looks one asset up and shows it as a
 * card with Buy on it. It lists nothing nobody asked for.
 *
 * **History still has no source.** A Substrate node serves no history RPC; it
 * comes from an indexer reading the chain's events (ADR-028), which is not
 * built. The host refuses `cgt_history` for exactly that reason, and this view
 * shows the host's reason rather than an empty list that looks like nothing
 * happened.
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { ArrowDownLeft, ArrowUpRight, Info, RefreshCw, Search } from 'lucide-react';

import {
  assets,
  chain,
  explain,
  sales,
  shortAddress,
  type HistoryEntry,
  type OwnedAsset,
  type SaleView,
} from '../lib/ipc';
import { selectActiveAccount, useQor } from '../state/store';
import { AssetCard } from '../qfx/AssetCard';
import { BuyDialog } from './BuyDialog';
import { TradeDialog } from './TradeDialog';
import { SellDialog } from './SellDialog';
import { Surface } from '../components/ui/Surface';
import { ViewHeader } from './parts';

type Held =
  | { state: 'reading' }
  | { state: 'read'; assets: OwnedAsset[] }
  | { state: 'failed'; reason: string };

/** One asset looked up by its number, to buy. */
type Found =
  | { state: 'idle' }
  | { state: 'looking' }
  | { state: 'found'; sale: SaleView }
  | { state: 'failed'; reason: string };

export function Inventory() {
  const account = useQor(selectActiveAccount);
  const token = useQor((s) => s.token);
  const notify = useQor((s) => s.notify);

  const [held, setHeld] = useState<Held>({ state: 'reading' });
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [historyNote, setHistoryNote] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);
  const [trading, setTrading] = useState<OwnedAsset | null>(null);
  const [selling, setSelling] = useState<OwnedAsset | null>(null);
  const [wanted, setWanted] = useState('');
  const [found, setFound] = useState<Found>({ state: 'idle' });
  const [buying, setBuying] = useState<SaleView | null>(null);
  const foundCard = useRef<HTMLDivElement | null>(null);
  const symbol = token?.symbol ?? 'CGT';

  const readAssets = useCallback(async () => {
    if (!account) return;
    try {
      setHeld({ state: 'read', assets: await assets.of(account.address) });
    } catch (e) {
      setHeld({ state: 'failed', reason: explain(e) });
    }
  }, [account]);

  const load = useCallback(async () => {
    if (!account) return;
    setLoading(true);
    setHeld({ state: 'reading' });
    await readAssets();
    try {
      setHistory(await chain.history(account.address, 50));
      setHistoryNote(null);
    } catch (e) {
      setHistory([]);
      setHistoryNote(explain(e));
    } finally {
      setLoading(false);
    }
  }, [account, readAssets]);

  useEffect(() => {
    void load();
  }, [load]);

  const makePermanent = async (asset: OwnedAsset) => {
    if (!account) return;
    setBusy(`${asset.collection}/${asset.item}`);
    try {
      await assets.makePermanent(account.address, asset.collection, asset.item);
      notify('ok', `"${asset.name || 'The asset'}" is permanent now.`);
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      // Whatever happened, draw what the chain now says.
      await readAssets();
      setBusy(null);
    }
  };

  /** Withdraw a listing, or clear a void one. The host asks first. */
  const withdraw = async (asset: OwnedAsset) => {
    if (!account) return;
    setBusy(`${asset.collection}/${asset.item}`);
    try {
      await sales.unlist(account.address, asset.collection, asset.item);
      notify(
        'ok',
        asset.listing?.void
          ? 'The void listing is cleared.'
          : `"${asset.name || 'The asset'}" is no longer for sale.`,
      );
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      // Whatever happened, draw what the chain now says.
      await readAssets();
      setBusy(null);
    }
  };

  /** Look one asset up by its number. `text` is whatever was pasted. */
  const lookUp = useCallback(
    async (text: string) => {
      if (text.trim() === '') return;
      setFound({ state: 'looking' });
      try {
        setFound({ state: 'found', sale: await sales.find(text, account?.address ?? null) });
      } catch (e) {
        setFound({ state: 'failed', reason: explain(e) });
      }
    },
    [account],
  );

  // The card is drawn under the lookup, usually below the fold: bring it into
  // view, or pressing "Look it up" looks as if it did nothing.
  const foundId =
    found.state === 'found' ? `${found.sale.asset.collection}/${found.sale.asset.item}` : null;
  useEffect(() => {
    if (foundId) foundCard.current?.scrollIntoView({ block: 'nearest' });
  }, [foundId]);

  const heldAssets = held.state === 'read' ? held.assets : [];

  return (
    <div className="relative flex h-full flex-col overflow-y-auto">
      <ViewHeader
        eyebrow="Web3"
        title="Inventory"
        body="What you hold on chain, read from the chain every time you look."
        action={
          <button type="button" className="btn" onClick={() => void load()}>
            <RefreshCw size={13} className={loading ? 'animate-spin' : ''} />
            Refresh
          </button>
        }
      />

      <div className="p-8">
        <section className="mb-8">
          <div className="mb-3 flex items-baseline gap-3">
            <h2 className="eyebrow text-accent">DRC-369 assets</h2>
            <span className="text-caption text-ink-faint">
              Read from chain storage for this account
            </span>
          </div>

          {!account ? (
            <Empty>No account selected.</Empty>
          ) : held.state === 'reading' ? (
            <Empty>Reading the chain…</Empty>
          ) : held.state === 'failed' ? (
            <Empty>Could not read this account's assets: {held.reason}</Empty>
          ) : held.assets.length === 0 ? (
            <Empty>No assets yet. Mint one from a project in Projects.</Empty>
          ) : (
            <div className="grid gap-4 sm:grid-cols-2 xl:grid-cols-3">
              {held.assets.map((asset, i) => (
                <AssetCard
                  key={`${asset.collection}/${asset.item}`}
                  asset={asset}
                  index={i}
                  busy={busy === `${asset.collection}/${asset.item}`}
                  symbol={symbol}
                  held={{
                    onTrade: () => setTrading(asset),
                    onSell: () => setSelling(asset),
                    onMakePermanent: () => void makePermanent(asset),
                    onWithdraw: () => void withdraw(asset),
                  }}
                />
              ))}
            </div>
          )}
        </section>

        <section className="mb-8" data-find>
          <div className="mb-3 flex items-baseline gap-3">
            <h2 className="eyebrow text-accent">Buy an asset</h2>
            <span className="text-caption text-ink-faint">By its number, from whoever holds it</span>
          </div>

          <Surface className="p-5">
            <p className="text-caption text-ink-body" data-find-honest>
              There is no storefront to browse yet. To buy an asset, ask its holder for its number
              — the two numbers on its card, like 4/0 — or for the reference they copy from its
              menu, and paste it here. The launcher reads that one asset from the chain.
            </p>
            <form
              className="mt-3 flex flex-wrap items-end gap-2"
              onSubmit={(event) => {
                event.preventDefault();
                void lookUp(wanted);
              }}
            >
              <div className="min-w-0 flex-1">
                <label className="block text-caption text-ink-muted" htmlFor="find-asset">
                  Asset number or reference
                </label>
                <input
                  id="find-asset"
                  className="field numeric mt-1"
                  value={wanted}
                  onChange={(event) => setWanted(event.target.value)}
                  placeholder="4/0"
                  spellCheck={false}
                  autoComplete="off"
                  data-find-input
                />
              </div>
              <button
                type="submit"
                className="btn whitespace-nowrap"
                disabled={!account || wanted.trim() === '' || found.state === 'looking'}
                data-find-submit
              >
                <Search size={13} />
                {found.state === 'looking' ? 'Reading the chain…' : 'Look it up'}
              </button>
            </form>

            <div aria-live="polite">
              {found.state === 'failed' && (
                <p className="mt-3 text-caption text-bad" data-find-refusal>
                  {found.reason}
                </p>
              )}
            </div>
          </Surface>

          {found.state === 'found' && (
            <div
              ref={foundCard}
              className="mt-4 grid gap-4 sm:grid-cols-2 xl:grid-cols-3"
              data-found
            >
              <AssetCard
                asset={found.sale.asset}
                index={0}
                busy={false}
                symbol={symbol}
                found={{ sale: found.sale, onBuy: () => setBuying(found.sale) }}
              />
            </div>
          )}
        </section>

        <section>
          <div className="mb-3 flex items-baseline gap-3">
            <h2 className="eyebrow text-accent">Settled history</h2>
            <span className="text-caption text-ink-faint">
              Transactions committed in blocks, newest first
            </span>
          </div>

          {!account ? (
            <Empty>No account selected.</Empty>
          ) : historyNote ? (
            <Empty>{historyNote}</Empty>
          ) : history.length === 0 ? (
            <Empty>
              {loading
                ? 'Reading the chain…'
                : 'Nothing yet. Transfers appear here once they are in a block.'}
            </Empty>
          ) : (
            <ul className="flex flex-col gap-1.5">
              {history.map((entry, i) => (
                <HistoryRow
                  key={`${entry.hash}-${i}`}
                  entry={entry}
                  index={i}
                  symbol={symbol}
                />
              ))}
            </ul>
          )}
        </section>
      </div>

      {selling && account && (
        <SellDialog
          asset={selling}
          from={account.address}
          symbol={symbol}
          onListed={(receipt) => {
            setSelling(null);
            notify(
              'ok',
              `Listed at ${receipt.price_cgt} ${symbol}. A buyer needs this asset\u2019s number: ${receipt.collection}/${receipt.item}.`,
            );
            void readAssets();
          }}
          onWithdrawn={() => {
            const name = selling.name || 'The asset';
            setSelling(null);
            notify('ok', `"${name}" is no longer for sale.`);
            void readAssets();
          }}
          onSaved={() =>
            notify('ok', 'Description saved on this machine. It is not published anywhere.')
          }
          onClose={() => setSelling(null)}
        />
      )}

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
            void readAssets();
            // The card that offered it now says who holds it: this account.
            void lookUp(`${receipt.collection}/${receipt.item}`);
          }}
          onClose={() => {
            // The dialog may have read the asset again after a refusal, so the
            // card under it is read again too rather than left showing less.
            const id = `${buying.asset.collection}/${buying.asset.item}`;
            setBuying(null);
            void lookUp(id);
          }}
        />
      )}

      {trading && account && (
        <TradeDialog
          from={account.address}
          held={heldAssets}
          start={trading}
          onSent={(receipt) => {
            setTrading(null);
            notify(
              'ok',
              receipt.moved.length === 1
                ? `Sent to ${shortAddress(receipt.to, 6, 6)}. It is theirs now.`
                : `Sent ${receipt.moved.length} assets to ${shortAddress(receipt.to, 6, 6)}. They are theirs now.`,
            );
            void readAssets();
          }}
          onClose={() => setTrading(null)}
        />
      )}
    </div>
  );
}

function HistoryRow({
  entry,
  index,
  symbol,
}: {
  entry: HistoryEntry;
  index: number;
  symbol: string;
}) {
  const incoming = entry.direction === 'in';
  const Arrow = incoming ? ArrowDownLeft : ArrowUpRight;
  const counterparty = incoming ? entry.from : (entry.to ?? '—');

  return (
    <Surface
      as="li"
      className="stagger flex items-center gap-4 px-4 py-3"
      style={{ '--i': index } as React.CSSProperties}
    >
      <Arrow
        size={15}
        className={`flex-none ${incoming ? 'text-ok' : 'text-ink-muted'}`}
      />

      <div className="min-w-0 flex-1">
        <p className="text-ui text-ink">
          {incoming ? 'Received from' : 'Sent to'}{' '}
          <span className="numeric text-ink-body">
            {counterparty === '—' ? '—' : shortAddress(counterparty, 10, 6)}
          </span>
        </p>
        <p className="numeric text-micro text-ink-faint">
          {entry.block_number != null ? `Block ${entry.block_number}` : 'Pending'} ·{' '}
          {shortAddress(entry.hash, 8, 6)}
        </p>
      </div>

      <p className="numeric flex-none text-body text-ink">
        {entry.amount_cgt ? (
          <>
            {incoming ? '+' : '−'}
            {entry.amount_cgt} <span className="text-micro text-ink-muted">{symbol}</span>
          </>
        ) : (
          '—'
        )}
      </p>
    </Surface>
  );
}

function Empty({ children }: { children: React.ReactNode }) {
  return (
    <Surface className="flex items-center gap-3 px-5 py-8">
      <Info size={15} className="flex-none text-ink-faint" />
      <p className="text-ui text-ink-muted">{children}</p>
    </Surface>
  );
}
