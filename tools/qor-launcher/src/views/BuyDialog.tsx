/**
 * Buying a listed asset (L4.6).
 *
 * # One screen, and it is the warning
 *
 * A purchase has nothing to compose: the asset, the price and who is paid are
 * all the chain's. So this is a single screen that says, before anything is
 * signed, what leaves this account, where each part of it goes, and that it
 * cannot be undone. After it comes the host's own dialog (roadmap L1.4), which
 * repeats all of it outside the webview with every address in full, and only
 * then does the vault sign.
 *
 * # What this view does not decide
 *
 * Whether the purchase is possible, what it pays and to whom are the host's
 * reading of the chain (`src-tauri/src/chain/sales.rs`); this draws them. It
 * sends back the price and the fingerprint that were on screen, and the host
 * refuses — asking nobody — if either has changed on chain since. The price on
 * screen is also the most the chain may take, so a price raised between this
 * screen and the block moves nothing.
 *
 * # After a refusal, the screen is read again
 *
 * A refusal usually means the screen had gone stale: the price moved, the
 * listing was withdrawn, the asset was revised. So the dialog asks the host for
 * the asset again and draws that answer, under the reason it was refused — a
 * person is never left looking at a price the chain no longer holds, with a
 * button that would only be refused again.
 *
 * One transaction pays everyone and hands the asset over, or none of it
 * happens (ADR-061).
 */

import { useEffect, useRef, useState } from 'react';
import { AlertTriangle, ShoppingBag, X } from 'lucide-react';

import { explain, sales, shortAddress } from '../lib/ipc';
import type { SaleReceipt, SaleView } from '../lib/ipc';
import { Sigil } from '../qfx/Sigil';
import { SaleBreakdown } from './SaleBreakdown';

interface Props {
  /** The account that pays, and will sign. */
  from: string;
  /** The asset as the host read it when Buy was pressed, with what a sale pays. */
  sale: SaleView;
  symbol: string;
  onBought: (receipt: SaleReceipt) => void;
  onClose: () => void;
}

export function BuyDialog({ from, sale: opened, symbol, onBought, onClose }: Props) {
  const panel = useRef<HTMLDivElement | null>(null);
  const [sale, setSale] = useState(opened);
  const [busy, setBusy] = useState(false);
  const [refusal, setRefusal] = useState<string | null>(null);
  /** The screen was read again after a refusal, and shows the chain's answer. */
  const [reread, setReread] = useState(false);

  const { asset, breakdown } = sale;
  const id = `${asset.collection}/${asset.item}`;

  useEffect(() => {
    panel.current?.focus();
  }, []);

  const buy = async () => {
    if (!breakdown) return;
    setBusy(true);
    setRefusal(null);
    try {
      onBought(
        await sales.buy(
          from,
          asset.collection,
          asset.item,
          breakdown.price_sparks,
          asset.current.root,
        ),
      );
    } catch (error) {
      setRefusal(explain(error));
      // Draw what the chain holds now. If it cannot be read, the screen stays
      // as it was, and the host still checks everything again on the next try.
      try {
        setSale(await sales.find(id, from));
        setReread(true);
      } catch {
        setReread(false);
      }
      setBusy(false);
    }
  };

  return (
    <div className="modal" data-buy-dialog onClick={busy ? undefined : onClose}>
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={`Buy ${asset.name || 'the untitled asset'}`}
        tabIndex={-1}
        className="modal-panel surface cut p-7"
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === 'Escape' && !busy) onClose();
        }}
      >
        <div className="mb-5 flex items-baseline justify-between gap-4">
          <div className="min-w-0">
            <p className="eyebrow text-accent">Buy</p>
            <p className="heading truncate text-heading text-ink">{asset.name || 'Untitled'}</p>
          </div>
          <button
            type="button"
            className="btn btn-ghost"
            onClick={onClose}
            disabled={busy}
            aria-label="Close"
          >
            <X size={14} />
          </button>
        </div>

        <div className="grid gap-5 sm:grid-cols-[13rem_1fr]">
          <section className="trade-side" data-buy-asset>
            <p className="eyebrow text-ink-muted">What you are buying</p>
            <Sigil root={asset.current.root} className="mx-auto my-4" />
            <p className="truncate text-body font-semibold text-ink">{asset.name || 'Untitled'}</p>
            <p className="numeric text-micro text-ink-faint">
              Asset {id} · {asset.revisable ? 'Revisable' : 'Permanent'}
            </p>
            <p className="numeric mt-2 break-all text-micro text-ink-muted">
              {asset.current.algo} {asset.current.root}
            </p>
            <p className="mt-3 text-micro text-ink-muted">
              Held by{' '}
              <span className="numeric" title={sale.holder}>
                {shortAddress(sale.holder, 6, 6)}
              </span>
            </p>
          </section>

          <section className="min-w-0">
            <p className="eyebrow text-ink-muted">Where your {symbol} goes</p>
            {breakdown ? (
              <div className="mt-2">
                <SaleBreakdown breakdown={breakdown} reader="buyer" symbol={symbol} />
              </div>
            ) : (
              <p className="mt-2 text-caption text-ink-muted">This asset cannot be bought now.</p>
            )}

            {sale.viewer_free_cgt && (
              <p className="mt-3 text-caption text-ink-muted" data-buy-balance>
                This account can spend{' '}
                <span className="numeric text-ink-body">
                  {sale.viewer_free_cgt} {symbol}
                </span>
                .
              </p>
            )}

            {asset.revisable && (
              <p className="mt-3 text-caption text-ink-muted" data-buy-revisable>
                This asset is still revisable: whoever holds it can point it at different content
                until it is made permanent. If that happens before your purchase is sent, the
                launcher stops and asks nobody. Once you hold it, that choice is yours.
              </p>
            )}
            {sale.pasted_root_matches === false && (
              <p className="mt-3 text-caption text-bad" data-buy-mismatch>
                This is not the fingerprint you pasted. Check with the seller before buying.
              </p>
            )}
          </section>
        </div>

        <div className="mt-5 flex items-start gap-2 border border-edge p-3" data-buy-warning>
          <AlertTriangle size={14} className="mt-0.5 flex-none text-bad" />
          <p className="text-caption text-ink-body">
            This cannot be undone. Once it is in a block the {symbol} is theirs and the asset is
            yours, and nobody here can reverse it. You will not pay more than the price shown: if
            it is raised before this settles, nothing moves. All of it happens or none of it does.
          </p>
        </div>

        {sale.cannot_buy && (
          <p className="mt-4 text-caption text-bad" role="alert" data-buy-cannot>
            {sale.cannot_buy}
          </p>
        )}
        {refusal && (
          <p className="mt-4 text-caption text-bad" role="alert" data-buy-refusal>
            {refusal}
            {reread && (
              <span className="block text-ink-body" data-buy-reread>
                This screen has been read from the chain again. Check it before you buy.
              </span>
            )}
          </p>
        )}

        <div className="mt-6 flex flex-wrap items-center gap-2">
          <button
            type="button"
            className="btn btn-primary whitespace-nowrap"
            onClick={() => void buy()}
            disabled={busy || !breakdown || sale.cannot_buy !== null}
            data-buy-confirm
          >
            <ShoppingBag size={13} />
            {busy
              ? 'Waiting for the chain…'
              : breakdown
                ? `Buy for ${breakdown.price_cgt} ${symbol}`
                : 'Buy'}
          </button>
          <button type="button" className="btn btn-ghost" onClick={onClose} disabled={busy}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
