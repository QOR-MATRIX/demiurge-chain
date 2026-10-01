/**
 * A DRC-369 asset as a card (QFX layer two, first slice).
 *
 * # Why this file is in `src/qfx/`
 *
 * The card answers the pointer: it tilts towards it, and its registration marks
 * lift. `scripts/check-design.mjs` forbids pointer-following effects everywhere
 * except this directory, which is exempt from exactly two rules — the pointer
 * and the frame loop — and from nothing else. So there is no gradient here and
 * no glow: a holographic sheen is a gradient, and a gradient is still forbidden.
 * The sheen belongs to the next slice, where the one QFX canvas shows through
 * the card rather than a second canvas being started for it (ADR-051: one
 * canvas, owned by the chrome).
 *
 * # What it may not cost
 *
 * - **Readability.** Everything drawn here sits behind text the chrome owns, and
 *   `scripts/check-readability.mjs` measures every run of it as painted, on this
 *   surface, in every theme, over a hostile backdrop.
 * - **Stillness.** Reduced motion means no tilt and no transition — not a slower
 *   one. The setting is read from `data-motion`, which `applyA11y` writes, so
 *   this and the rest of the chrome cannot disagree.
 * - **The pointer.** No frame loop: the tilt is written straight to the element
 *   on the pointer's own events, so nothing schedules work when the pointer is
 *   still.
 *
 * # What is on the face
 *
 * What the chain holds, and nothing invented: the name, the content reference,
 * the commit it pins, whether it is permanent, and — when it is listed — the
 * price it is offered at (L4.6). The sigil is drawn from the fingerprint itself
 * — the same asset always draws the same mark — so an asset is recognisable
 * before it has a preview image. When the manifest names a Preview (ADR-047's
 * role, which nothing sets yet), it takes the sigil's place.
 *
 * # One card, held or found
 *
 * The same card shows an asset this account holds and one it looked up by its
 * number. A held card carries its menu, Make permanent and Withdraw. A found
 * card carries who holds it and Buy — or, where it cannot be bought, the host's
 * reason in words. The card never works out whether a purchase is possible: the
 * host does, from the chain (`src-tauri/src/chain/sales.rs`).
 */

import { useEffect, useRef, useState } from 'react';
import { Lock, Maximize2, MoreHorizontal, ShoppingBag, Undo2, X } from 'lucide-react';

import { shortAddress } from '../lib/ipc';
import type { OwnedAsset, SaleView } from '../lib/ipc';
import { Surface } from '../components/ui/Surface';
import { AssetMenu, type MenuAt } from './AssetMenu';
import { Sigil } from './Sigil';

/** How far the card leans towards the pointer, at the edges. */
const TILT_DEGREES = 5;

function stillnessWanted(): boolean {
  const asked = document.documentElement.dataset.motion;
  if (asked === 'reduced') return true;
  if (asked === 'full') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

function short(hex: string, lead = 6, tail = 4) {
  return hex.length <= lead + tail ? hex : `${hex.slice(0, lead)}…${hex.slice(-tail)}`;
}

/** What a card offers when this account holds the asset. */
export interface HeldActions {
  onMakePermanent: () => void;
  onTrade: () => void;
  onSell: () => void;
  onWithdraw: () => void;
}

/** What a card offers when the asset was looked up by its number. */
export interface FoundActions {
  /** Everything the host read about it: its holder, its terms, what a sale pays. */
  sale: SaleView;
  onBuy: () => void;
}

interface Props {
  asset: OwnedAsset;
  index: number;
  busy: boolean;
  symbol: string;
  /** Exactly one of the two: an asset is drawn as held, or as found. */
  held?: HeldActions;
  found?: FoundActions;
}

export function AssetCard({ asset, index, busy, symbol, held, found }: Props) {
  const face = useRef<HTMLElement | null>(null);
  const opener = useRef<HTMLButtonElement | null>(null);
  const menuButton = useRef<HTMLButtonElement | null>(null);
  const [open, setOpen] = useState(false);
  const [menu, setMenu] = useState<MenuAt | null>(null);
  const [copied, setCopied] = useState(false);

  const id = `${asset.collection}/${asset.item}`;
  const listing = asset.listing ?? null;
  const listed = listing !== null && !listing.void;
  const stale = listing !== null && listing.void;

  const closeMenu = () => {
    setMenu(null);
    menuButton.current?.focus();
  };

  /** The reference is what identifies this asset anywhere, so that is what is copied. */
  const share = async () => {
    const text = `${asset.current.algo} ${asset.current.root} (asset ${id})`;
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      window.setTimeout(() => setCopied(false), 2400);
    } catch {
      // A webview may refuse the clipboard. Showing it beats failing silently.
      window.prompt('Copy this asset\u2019s reference', text);
    }
    closeMenu();
  };


  const lean = (event: React.PointerEvent) => {
    const node = face.current;
    if (!node || stillnessWanted()) return;
    const box = node.getBoundingClientRect();
    const x = (event.clientX - box.left) / box.width - 0.5;
    const y = (event.clientY - box.top) / box.height - 0.5;
    node.style.transform = `perspective(900px) rotateY(${x * TILT_DEGREES * 2}deg) rotateX(${-y * TILT_DEGREES * 2}deg) translateY(-2px)`;
  };

  const settle = () => {
    const node = face.current;
    if (node) node.style.transform = '';
  };

  return (
    <>
      <Surface
        as="article"
        cut
        className="qfx-card stagger flex min-w-0 flex-col p-5"
        style={{ '--i': index } as React.CSSProperties}
      >
        <div
          className="flex min-w-0 flex-1 flex-col"
          ref={(node) => {
            face.current = node?.parentElement ?? null;
          }}
          onPointerMove={lean}
          onPointerLeave={settle}
          onContextMenu={(event) => {
            // The menu is what a holder does with an asset. A found asset has
            // none, so the pointer's secondary button is left alone there.
            if (!held) return;
            event.preventDefault();
            setMenu({ x: event.clientX, y: event.clientY });
          }}
          data-asset={found ? undefined : id}
          data-found-asset={found ? id : undefined}
        >
          <div className="flex items-baseline justify-between gap-3">
            <button
              type="button"
              ref={opener}
              className="qfx-card-open min-w-0 truncate text-left text-body font-semibold text-ink"
              onClick={() => setOpen(true)}
              aria-label={`Open ${asset.name || 'the untitled asset'}`}
              data-asset-name
            >
              {asset.name || 'Untitled'}
            </button>
            <div className="flex flex-none items-center gap-2">
              <span
                className={`eyebrow ${asset.revisable ? 'text-ink-muted' : 'text-ok'}`}
                data-asset-status
              >
                {asset.revisable ? 'Revisable' : 'Permanent'}
              </span>
              {held && (
                <button
                  type="button"
                  ref={menuButton}
                  className="qfx-card-open text-ink-muted"
                  aria-haspopup="menu"
                  aria-expanded={menu !== null}
                  aria-label={`What to do with ${asset.name || 'the untitled asset'}`}
                  data-asset-more
                  onClick={(event) => {
                    const box = event.currentTarget.getBoundingClientRect();
                    setMenu(menu ? null : { x: box.left, y: box.bottom + 4 });
                  }}
                >
                  <MoreHorizontal size={15} />
                </button>
              )}
            </div>
          </div>
          <p className="numeric text-micro text-ink-faint">Asset {id}</p>

          {/* The price it is offered at, as the chain holds it. A void listing
              is said to be one, never drawn as a price somebody could pay. */}
          {listed && listing && (
            <p
              className="mt-2 flex items-baseline justify-between gap-3 border-y border-edge py-1.5"
              data-asset-listing
            >
              <span className="eyebrow text-accent">For sale</span>
              <span className="numeric text-ui text-ink" data-asset-price>
                {listing.price_cgt} <span className="text-micro text-ink-muted">{symbol}</span>
              </span>
            </p>
          )}
          {stale && (
            <p className="mt-2 border-y border-edge py-1.5 text-micro text-ink-muted" data-asset-stale>
              A listing made by an earlier holder is still on chain. It is void: nobody can buy
              from it.
            </p>
          )}

          <Sigil root={asset.current.root} className="mx-auto my-5" />

          <dl className="grid grid-cols-[5rem_1fr] items-baseline gap-x-3 gap-y-1 text-caption">
            <dt className="text-ink-muted">Reference</dt>
            <dd className="numeric min-w-0 truncate text-ink-body" data-asset-root>
              {asset.current.algo} {asset.current.root}
            </dd>
            <dt className="text-ink-muted">Commit</dt>
            <dd className="numeric min-w-0 truncate text-ink-body" data-asset-commit>
              {asset.commit ? `${asset.commit.id} (${asset.commit.kind})` : 'None'}
            </dd>
            {found && (
              <>
                <dt className="text-ink-muted">Holder</dt>
                <dd
                  className="numeric min-w-0 truncate text-ink-body"
                  title={found.sale.holder}
                  data-asset-holder
                >
                  {found.sale.held_by_viewer
                    ? 'This account'
                    : shortAddress(found.sale.holder, 8, 8)}
                </dd>
              </>
            )}
          </dl>

          {found && found.sale.pasted_root_matches === false && (
            <p className="mt-3 text-micro text-bad" data-asset-mismatch>
              This is not the fingerprint you pasted. The asset points at different content now, or
              the reference was for another asset.
            </p>
          )}
          {found && found.sale.pasted_root_matches === true && (
            <p className="mt-3 text-micro text-ok" data-asset-match>
              It carries the fingerprint you pasted.
            </p>
          )}
          {found && found.sale.cannot_buy && (
            <p className="mt-3 text-caption text-ink-muted" data-asset-cannot-buy>
              {found.sale.cannot_buy}
            </p>
          )}

          {copied && (
            <p className="mt-3 text-micro text-ok" data-asset-copied>
              Reference copied
            </p>
          )}

          <div className="mt-auto flex flex-wrap items-center gap-2 pt-5">
            <button
              type="button"
              className="btn btn-ghost whitespace-nowrap"
              onClick={() => setOpen(true)}
            >
              <Maximize2 size={13} />
              Look closer
            </button>
            {held && asset.revisable && (
              <button
                type="button"
                className="btn whitespace-nowrap"
                onClick={held.onMakePermanent}
                disabled={busy}
              >
                <Lock size={13} />
                Make permanent
              </button>
            )}
            {held && listing && (
              <button
                type="button"
                className="btn whitespace-nowrap"
                onClick={held.onWithdraw}
                disabled={busy}
                data-asset-withdraw
              >
                <Undo2 size={13} />
                {stale ? 'Clear void listing' : 'Withdraw'}
              </button>
            )}
            {found && listed && listing && !found.sale.cannot_buy && (
              <button
                type="button"
                className="btn btn-primary whitespace-nowrap"
                onClick={found.onBuy}
                disabled={busy}
                data-asset-buy
              >
                <ShoppingBag size={13} />
                Buy
              </button>
            )}
          </div>
        </div>
      </Surface>

      {menu && held && (
        <AssetMenu
          at={menu}
          listed={listed}
          stale={stale}
          onTrade={() => {
            setMenu(null);
            held.onTrade();
          }}
          onSell={() => {
            setMenu(null);
            held.onSell();
          }}
          onWithdraw={() => {
            setMenu(null);
            held.onWithdraw();
          }}
          onShare={share}
          onClose={closeMenu}
        />
      )}

      {open && (
        <AssetCloseUp
          asset={asset}
          symbol={symbol}
          sale={found?.sale ?? null}
          onClose={() => {
            setOpen(false);
            opener.current?.focus();
          }}
        />
      )}
    </>
  );
}

/**
 * The card, larger, with everything the chain holds about the asset. The place a
 * preview or a model will render when there is one; today it is the sigil at
 * size, so the surface is honest about holding no artwork yet.
 */
function AssetCloseUp({
  asset,
  symbol,
  sale,
  onClose,
}: {
  asset: OwnedAsset;
  symbol: string;
  /** What the host read about a found asset: its holder and its terms. */
  sale: SaleView | null;
  onClose: () => void;
}) {
  const panel = useRef<HTMLDivElement | null>(null);
  const revised = asset.current.root !== asset.origin.root;

  useEffect(() => {
    panel.current?.focus();
    const key = (event: KeyboardEvent) => {
      if (event.key === 'Escape') onClose();
    };
    document.addEventListener('keydown', key);
    return () => document.removeEventListener('keydown', key);
  }, [onClose]);

  return (
    <div className="modal" onClick={onClose} data-asset-closeup>
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={asset.name || 'Untitled asset'}
        tabIndex={-1}
        className="modal-panel surface cut p-8"
        onClick={(event) => event.stopPropagation()}
      >
        <div className="mb-5 flex items-baseline justify-between gap-4">
          <div className="min-w-0">
            <p className="heading truncate text-heading text-ink">{asset.name || 'Untitled'}</p>
            <p className="numeric text-micro text-ink-faint">
              Asset {asset.collection}/{asset.item} ·{' '}
              {asset.revisable ? 'Revisable' : 'Permanent'}
            </p>
          </div>
          <button type="button" className="btn btn-ghost" onClick={onClose} aria-label="Close">
            <X size={14} />
          </button>
        </div>

        <Sigil root={asset.current.root} large className="mb-6" />

        <dl className="grid gap-x-4 gap-y-2 text-ui sm:grid-cols-[9rem_1fr]">
          <dt className="text-caption text-ink-muted">Content reference</dt>
          <dd className="min-w-0">
            <span className="numeric break-all text-ink-body">
              {asset.current.algo} {asset.current.root}
            </span>
            <span className="block text-caption text-ink-faint">
              Manifest of {asset.current.size} bytes
            </span>
          </dd>
          {revised && (
            <>
              <dt className="text-caption text-ink-muted">Minted as</dt>
              <dd className="numeric min-w-0 break-all text-ink-muted">
                {asset.origin.algo} {asset.origin.root}
              </dd>
            </>
          )}
          <dt className="text-caption text-ink-muted">Pinned commit</dt>
          <dd className="numeric min-w-0 break-all text-ink-body">
            {asset.commit ? `${asset.commit.id} (${asset.commit.kind})` : 'None'}
          </dd>
          <dt className="text-caption text-ink-muted">Short form</dt>
          <dd className="numeric min-w-0 text-ink-muted">
            {short(asset.current.root)} · {asset.commit ? short(asset.commit.id) : '—'}
          </dd>
          {asset.listing && (
            <>
              <dt className="text-caption text-ink-muted">Listed at</dt>
              <dd className="min-w-0" data-closeup-listing>
                <span className="numeric text-ink-body">
                  {asset.listing.price_cgt} {symbol}
                </span>
                <span className="block text-caption text-ink-faint">
                  {asset.listing.void
                    ? 'Void: whoever listed it no longer holds it.'
                    : 'On chain. A buyer reaches it by this asset’s number.'}
                </span>
              </dd>
            </>
          )}
          {sale && (
            <>
              <dt className="text-caption text-ink-muted">Held by</dt>
              <dd className="numeric min-w-0 break-all text-ink-body">{sale.holder}</dd>
              {sale.derived_from && (
                <>
                  <dt className="text-caption text-ink-muted">Remixed from</dt>
                  <dd className="numeric min-w-0 text-ink-body">
                    Asset {sale.derived_from.collection}/{sale.derived_from.item}
                  </dd>
                </>
              )}
              <dt className="text-caption text-ink-muted">Royalties</dt>
              <dd className="min-w-0" data-closeup-terms>
                {sale.terms && sale.terms.recipients.length > 0 ? (
                  <>
                    {sale.terms.recipients.map((one) => (
                      <span key={one.address} className="block text-ink-body">
                        {one.share} of every sale to{' '}
                        <span className="numeric break-all">{one.address}</span>
                      </span>
                    ))}
                    <span className="block text-caption text-ink-faint">
                      A sale of a remix of this owes them {sale.terms.remix}.
                    </span>
                  </>
                ) : (
                  <span className="text-ink-body">None set by its creator.</span>
                )}
              </dd>
            </>
          )}
        </dl>

        <p className="mt-6 text-caption text-ink-muted">
          No preview yet: a mint fingerprints a project's files and names none of them a preview. When
          one is named, it is shown here, and a 3D model is rendered in the backdrop's own canvas.
        </p>
      </div>
    </div>
  );
}
