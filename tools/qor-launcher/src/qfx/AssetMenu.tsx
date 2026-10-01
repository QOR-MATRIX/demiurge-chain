/**
 * The menu an asset raises: trade, sell, share (L4.4).
 *
 * # Reachable without a mouse
 *
 * A menu that only the secondary pointer button can open is unusable by anyone
 * navigating with a keyboard, and `scripts/check-accessibility.mjs` holds this
 * launcher to L1.2. So the card carries a button that opens the same menu, the
 * menu takes focus when it opens, the arrow keys move through it, Escape closes
 * it and focus returns to whatever opened it.
 *
 * # It decides nothing
 *
 * Every item here reports upwards. Trade opens the dialog that the host refuses
 * or signs; Share copies text; Sell opens the listing form (L4.6), which
 * publishes a price on chain through the host's own dialog. An asset already
 * listed offers a new price and a withdrawal in Sell's place. The note under
 * them says what a listing does not do — nobody browses to it, because nothing
 * serves a catalogue yet (M5.4) — before the item is used, because a form that
 * looks like it opened a shop is worse than one that says what it did not do.
 */

import { useEffect, useRef } from 'react';
import { Copy, Send, Store, Undo2 } from 'lucide-react';

/** Where the menu opens: the pointer, or the button that opened it. */
export interface MenuAt {
  x: number;
  y: number;
}

interface Props {
  at: MenuAt;
  /** The asset is listed, and its listing can be bought from. */
  listed: boolean;
  /** The asset carries a listing its previous holder made, which is void. */
  stale: boolean;
  onTrade: () => void;
  onSell: () => void;
  onWithdraw: () => void;
  onShare: () => void;
  onClose: () => void;
}

export function AssetMenu({
  at,
  listed,
  stale,
  onTrade,
  onSell,
  onWithdraw,
  onShare,
  onClose,
}: Props) {
  const box = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    // Open where there is room: a menu that runs off the window cannot be read.
    const node = box.current;
    if (node) {
      const size = node.getBoundingClientRect();
      const x = Math.min(at.x, window.innerWidth - size.width - 8);
      const y = Math.min(at.y, window.innerHeight - size.height - 8);
      node.style.left = `${Math.max(8, x)}px`;
      node.style.top = `${Math.max(8, y)}px`;
      node.querySelector<HTMLButtonElement>('[role="menuitem"]:not(:disabled)')?.focus();
    }
    const away = () => onClose();
    // A click anywhere else closes it, as a menu does.
    window.addEventListener('pointerdown', away);
    window.addEventListener('resize', away);
    return () => {
      window.removeEventListener('pointerdown', away);
      window.removeEventListener('resize', away);
    };
  }, [at, onClose]);

  const keys = (event: React.KeyboardEvent) => {
    if (event.key === 'Escape') {
      event.stopPropagation();
      onClose();
      return;
    }
    if (event.key !== 'ArrowDown' && event.key !== 'ArrowUp') return;
    event.preventDefault();
    const items = Array.from(
      box.current?.querySelectorAll<HTMLButtonElement>('[role="menuitem"]:not(:disabled)') ?? [],
    );
    if (items.length === 0) return;
    const here = items.indexOf(document.activeElement as HTMLButtonElement);
    const step = event.key === 'ArrowDown' ? 1 : -1;
    items[(here + step + items.length) % items.length]?.focus();
  };

  return (
    <div
      ref={box}
      role="menu"
      aria-label="What to do with this asset"
      className="menu cut"
      data-asset-menu
      onKeyDown={keys}
      onPointerDown={(event) => event.stopPropagation()}
    >
      <button type="button" role="menuitem" className="menu-item text-ui" onClick={onTrade}>
        <Send size={13} />
        {/* Named for what it is, always. It read "Send…" for an account holding
            one asset, which hid the feature from the person looking for it. */}
        Trade…
      </button>
      <button
        type="button"
        role="menuitem"
        className="menu-item text-ui"
        onClick={onSell}
        data-asset-sell
      >
        <Store size={13} />
        {listed ? 'Change price' : 'Sell'}
      </button>
      {(listed || stale) && (
        <button
          type="button"
          role="menuitem"
          className="menu-item text-ui"
          onClick={onWithdraw}
          data-asset-withdraw
        >
          <Undo2 size={13} />
          {stale ? 'Clear void listing' : 'Withdraw listing'}
        </button>
      )}
      <p className="menu-note text-micro">
        A listing puts a price on chain. Nobody browses to it: there is no storefront yet (M5.4),
        so a buyer needs this asset&rsquo;s number from you.
      </p>
      <button type="button" role="menuitem" className="menu-item text-ui" onClick={onShare}>
        <Copy size={13} />
        Copy reference
      </button>
    </div>
  );
}
