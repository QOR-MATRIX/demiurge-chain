/**
 * What a sale pays, part by part (L4.6).
 *
 * # It does no arithmetic
 *
 * Every amount here was worked out by the host, with the chain's own helpers,
 * from the royalty terms it read from the chain (`src-tauri/src/chain/sales.rs`).
 * This draws them in the order the chain pays them — the work a remix came
 * from, then the asset's own royalties, then the seller — and adds nothing up.
 * The total it shows is the price the host returned, not a sum made here.
 *
 * The same list is shown to a seller before they list and to a buyer before
 * they buy, so both read one account of where the price goes. The host's own
 * dialog then repeats it outside the webview, with every address in full.
 */

import { AlertTriangle } from 'lucide-react';

import { shortAddress } from '../lib/ipc';
import type { Breakdown, Payout } from '../lib/ipc';

interface Props {
  breakdown: Breakdown;
  /** Who is reading it: the last line is "You keep" or "The seller". */
  reader: 'seller' | 'buyer';
  symbol: string;
}

function Row({
  payout,
  label,
  named = true,
}: {
  payout: Payout;
  label: string;
  /** Whether the account is named. The reader's own line does not need it. */
  named?: boolean;
}) {
  return (
    <li
      className="flex items-baseline justify-between gap-4 py-1"
      data-payout={payout.kind}
      data-payout-amount={payout.amount_sparks}
    >
      <span className="min-w-0 text-caption text-ink-body">
        {label}
        {named && (
          <>
            {' '}
            <span className="numeric text-ink-muted" title={payout.address}>
              {shortAddress(payout.address, 6, 6)}
            </span>
          </>
        )}
        {payout.to_buyer && <span className="text-ink-muted"> (your own account)</span>}
      </span>
      <span className="numeric flex-none text-ui text-ink">{payout.amount_cgt}</span>
    </li>
  );
}

export function SaleBreakdown({ breakdown, reader, symbol }: Props) {
  const source = breakdown.payouts.filter((payout) => payout.kind === 'source');
  const royalties = breakdown.payouts.filter((payout) => payout.kind === 'royalty');
  const seller = breakdown.payouts.filter((payout) => payout.kind === 'seller');
  const nothingOwed = source.length === 0 && royalties.length === 0;

  return (
    <div data-breakdown>
      <ul aria-label={`What a sale at ${breakdown.price_cgt} ${symbol} pays`}>
        {breakdown.source && source.length > 0 && (
          <li className="pt-1 text-micro text-ink-muted" data-breakdown-source>
            The work it was remixed from, asset {breakdown.source.collection}/
            {breakdown.source.item}: {breakdown.source_share} of the price
          </li>
        )}
        {source.map((payout) => (
          <Row key={`source-${payout.address}`} payout={payout} label="To" />
        ))}

        {royalties.length > 0 && (
          <li className="pt-1 text-micro text-ink-muted">
            {source.length > 0 ? 'Royalties, each a share of what is left' : 'Royalties'}
          </li>
        )}
        {royalties.map((payout) => (
          <Row key={`royalty-${payout.address}`} payout={payout} label={`${payout.share} to`} />
        ))}

        {seller.map((payout) => (
          <Row
            key={`seller-${payout.address}`}
            payout={payout}
            label={reader === 'seller' ? 'You keep' : 'The seller'}
            named={reader === 'buyer'}
          />
        ))}

        <li
          className="mt-1 flex items-baseline justify-between gap-4 border-t border-edge pt-2"
          data-breakdown-total
        >
          <span className="text-caption text-ink-muted">
            {reader === 'seller' ? 'A buyer pays' : 'You pay'}
          </span>
          <span className="numeric flex-none text-body font-semibold text-ink">
            {breakdown.price_cgt} <span className="text-micro text-ink-muted">{symbol}</span>
          </span>
        </li>
      </ul>

      <p className="mt-2 text-micro text-ink-faint" data-breakdown-note>
        {nothingOwed
          ? 'No royalty is owed on this asset, so the seller receives the whole price. '
          : 'Read from the royalty terms on chain as they stand now. '}
        Nothing else is taken: today this chain charges no fee and takes no share of a sale.
      </p>

      {breakdown.blocked && (
        <div className="mt-3 flex items-start gap-2 border border-edge p-3" data-breakdown-blocked>
          <AlertTriangle size={14} className="mt-0.5 flex-none text-bad" />
          <p className="text-caption text-ink-body">{breakdown.blocked}</p>
        </div>
      )}
    </div>
  );
}
