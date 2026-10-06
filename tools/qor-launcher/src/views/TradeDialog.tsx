/**
 * Sending assets to another account (L4.5).
 *
 * # Two sides, and what moves between them
 *
 * The owner asked for a window with two asset containers and something moving
 * between them. What moves is a mark that travels the lane **once**, each time
 * the trade changes: an asset added or removed, or a destination chosen. It does
 * not loop (by choice; ADR-080 allows looping animation), and it is not drawn at
 * all under reduced motion. Motion that means something, rather than motion that fills a gap.
 *
 * # Two steps, and the second one is the warning
 *
 * Composing a trade and agreeing to it are different acts, so they are different
 * screens. The first gathers the assets, a destination and an optional message;
 * the second says, before anything is signed, that the assets leave this account
 * for good. After that comes the host's own dialog (L1.4), which repeats it
 * outside the webview, and only then does the vault sign.
 *
 * # What this view does not decide
 *
 * It refuses nothing on its own that matters. The address is checked by the host
 * against this chain's format, ownership is re-read from chain storage at the
 * latest block, and the whole trade is one `Utility::batch_all` (ADR-053), so a
 * trade that cannot finish moves nothing. What is checked here is only what
 * saves a person a round trip: an empty destination, nothing chosen, more than
 * the host's limits.
 *
 * **"Recently traded with" is this machine's own memory** (`partners.rs`),
 * written only after the chain finalised a trade, kept nowhere else and sent to
 * nobody. Reaching someone by their QOR ID instead of an address needs a
 * directory lookup, which does not exist yet; until it does, this takes an
 * address.
 */

import { useEffect, useMemo, useRef, useState } from 'react';
import { AlertTriangle, Plus, Send, X } from 'lucide-react';

import {
  assets as assetsApi,
  explain,
  shortAddress,
  MESSAGE_LIMIT,
  TRADE_LIMIT,
} from '../lib/ipc';
import type { OwnedAsset, TradePartner, TradeReceipt } from '../lib/ipc';

const idOf = (asset: OwnedAsset) => `${asset.collection}/${asset.item}`;

/** Bytes, not characters: the host's limit is in bytes and so is the chain's. */
const bytes = (text: string) => new TextEncoder().encode(text).length;

/** The setting `applyA11y` writes, so this and the chrome cannot disagree. */
function stillnessWanted(): boolean {
  const asked = document.documentElement.dataset.motion;
  if (asked === 'reduced') return true;
  if (asked === 'full') return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}

interface Props {
  from: string;
  held: OwnedAsset[];
  start: OwnedAsset;
  onSent: (receipt: TradeReceipt) => void;
  onClose: () => void;
}

export function TradeDialog({ from, held, start, onSent, onClose }: Props) {
  const panel = useRef<HTMLDivElement | null>(null);
  const [chosen, setChosen] = useState<string[]>([idOf(start)]);
  const [to, setTo] = useState('');
  const [message, setMessage] = useState('');
  const [partners, setPartners] = useState<TradePartner[]>([]);
  const [confirming, setConfirming] = useState(false);
  const [busy, setBusy] = useState(false);
  const [refusal, setRefusal] = useState<string | null>(null);

  const picked = useMemo(
    () =>
      chosen.map((id) => held.find((asset) => idOf(asset) === id)).filter(Boolean) as OwnedAsset[],
    [chosen, held],
  );
  const others = held.filter((asset) => !chosen.includes(idOf(asset)));
  const tooMany = chosen.length > TRADE_LIMIT;
  const tooLong = bytes(message) > MESSAGE_LIMIT;
  const ready = to.trim().length > 0 && chosen.length > 0 && !tooMany && !tooLong;

  // The mark travels once whenever the trade changes. Keyed, so React mounts a
  // new element and the one-shot animation runs again from its start.
  const travel = `${chosen.join(',')}|${to.trim()}`;

  useEffect(() => {
    // A convenience: a list that cannot be read is simply no list.
    void assetsApi
      .partners()
      .then(setPartners)
      .catch(() => setPartners([]));
  }, []);

  const toggle = (asset: OwnedAsset) => {
    const id = idOf(asset);
    setChosen((was) => (was.includes(id) ? was.filter((one) => one !== id) : [...was, id]));
  };

  const send = async () => {
    setBusy(true);
    setRefusal(null);
    try {
      const receipt = await assetsApi.trade(
        from,
        to.trim(),
        picked.map((asset) => ({ collection: asset.collection, item: asset.item })),
        message.trim() ? message.trim() : null,
        null,
      );
      onSent(receipt);
    } catch (error) {
      // Back to composing: the destination or an asset is usually what is wrong.
      setConfirming(false);
      // The host refuses with `{ kind, message }`, not an `Error`.
      setRefusal(explain(error));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="modal" data-trade-dialog onClick={busy ? undefined : onClose}>
      <div
        ref={panel}
        role="dialog"
        aria-modal="true"
        aria-label={confirming ? 'Confirm this trade' : 'Send assets'}
        className={`modal-panel surface cut p-7 ${confirming ? 'modal-narrow' : ''}`}
        data-trade-step={confirming ? 'confirm' : 'compose'}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === 'Escape' && !busy) onClose();
        }}
      >
        <div className="mb-5 flex items-baseline justify-between gap-4">
          <div>
            <p className="eyebrow text-accent">{confirming ? 'Last check' : 'Trade'}</p>
            <p className="heading text-heading text-ink">
              {confirming ? 'This cannot be undone' : 'Send assets'}
            </p>
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

        {/* Keyed, so each step mounts fresh: unkeyed, React turned Review's button
            into Send's in place and it cross-faded out of the primary look. */}
        {confirming ? (
          <div key="warning" data-trade-warning>
            <p className="text-ui text-ink-body">
              {picked.length === 1 ? 'This asset moves to' : `These ${picked.length} assets move to`}
            </p>
            <p className="numeric mt-1 break-all text-ui text-ink">{to.trim()}</p>

            <ul className="mt-4 space-y-1">
              {picked.map((asset) => (
                <li key={idOf(asset)} className="flex items-baseline justify-between gap-3">
                  <span className="min-w-0 truncate text-ui text-ink-body">
                    {asset.name || 'Untitled'}
                  </span>
                  <span className="numeric flex-none text-caption text-ink-faint">
                    {idOf(asset)}
                  </span>
                </li>
              ))}
            </ul>

            {message.trim() && (
              <p className="mt-4 text-caption text-ink-muted">
                With a message anyone can read, for ever: “{message.trim()}”
              </p>
            )}

            <div className="mt-5 flex items-start gap-2 border border-edge p-3">
              <AlertTriangle size={14} className="mt-0.5 flex-none text-bad" />
              <p className="text-caption text-ink-body">
                Ownership transfers to that account and this cannot be undone. The assets leave this
                Inventory, and only that account can send them back — nobody here can reverse it, and
                no key you hold changes it. All of it happens or none of it does.
              </p>
            </div>

            <div className="mt-6 flex flex-wrap items-center gap-2">
              <button
                type="button"
                className="btn btn-danger whitespace-nowrap"
                onClick={send}
                disabled={busy}
                data-trade-send
              >
                <Send size={13} />
                {busy ? 'Waiting for the chain…' : 'Send them for good'}
              </button>
              <button
                type="button"
                className="btn btn-ghost"
                onClick={() => setConfirming(false)}
                disabled={busy}
              >
                Go back
              </button>
            </div>
          </div>
        ) : (
          <div key="compose">
            <div className="grid items-stretch gap-3 sm:grid-cols-[1fr_auto_1fr]">
              {/* What leaves this account. */}
              <section className="trade-side" data-trade-from>
                <p className="eyebrow text-ink-muted">Leaving you</p>
                <p className="mt-1 text-caption text-ink-faint">
                  {chosen.length} of {held.length}
                  {tooMany && ` — at most ${TRADE_LIMIT} in one trade`}
                </p>

                <div className="mt-3 flex flex-wrap gap-2" data-trade-picked>
                  {picked.map((asset) => (
                    <button
                      key={idOf(asset)}
                      type="button"
                      className="btn whitespace-nowrap"
                      aria-pressed="true"
                      onClick={() => toggle(asset)}
                      data-trade-pick={idOf(asset)}
                    >
                      {asset.name || 'Untitled'}
                      <X size={12} />
                    </button>
                  ))}
                </div>

                {others.length > 0 && (
                  <>
                    <p className="mt-4 text-caption text-ink-muted">Add another</p>
                    <div className="mt-2 flex gap-2 overflow-x-auto pb-1" data-trade-others>
                      {others.map((asset) => (
                        <button
                          key={idOf(asset)}
                          type="button"
                          className="btn whitespace-nowrap"
                          aria-pressed="false"
                          onClick={() => toggle(asset)}
                          data-trade-pick={idOf(asset)}
                        >
                          <Plus size={12} />
                          {asset.name || 'Untitled'}
                        </button>
                      ))}
                    </div>
                  </>
                )}
              </section>

              {/* The lane. It carries no meaning a reader needs — what it says
                  in motion, the two headings say in words — so it is hidden
                  from assistive technology. */}
              <div className="trade-lane" aria-hidden="true" data-trade-lane>
                {!stillnessWanted() && <span key={travel} className="trade-spark" />}
              </div>

              {/* Where they go. */}
              <section className="trade-side" data-trade-to-side>
                <p className="eyebrow text-ink-muted">Going to</p>
                <label className="mt-1 block text-caption text-ink-faint" htmlFor="trade-to">
                  Destination address
                </label>
                <input
                  id="trade-to"
                  className="field numeric mt-1"
                  value={to}
                  onChange={(event) => setTo(event.target.value)}
                  placeholder="5…"
                  spellCheck={false}
                  autoComplete="off"
                  data-trade-to
                />

                {partners.length > 0 && (
                  <>
                    <p className="mt-4 text-caption text-ink-muted">Recently traded with</p>
                    <div className="mt-2 flex flex-wrap gap-2" data-trade-partners>
                      {partners.map((partner) => (
                        <button
                          key={partner.address}
                          type="button"
                          className="btn btn-ghost whitespace-nowrap"
                          onClick={() => setTo(partner.address)}
                          data-trade-partner={partner.address}
                        >
                          {partner.label || shortAddress(partner.address, 6, 6)}
                          <span className="text-micro text-ink-faint">×{partner.trades}</span>
                        </button>
                      ))}
                    </div>
                  </>
                )}

                <p className="mt-4 text-caption text-ink-muted">
                  {picked.length === 1
                    ? 'One asset becomes theirs'
                    : `${picked.length} assets become theirs`}
                </p>
              </section>
            </div>

            <label className="mt-5 block text-caption text-ink-muted" htmlFor="trade-message">
              Message (optional)
            </label>
            <input
              id="trade-message"
              className="field mt-1"
              value={message}
              onChange={(event) => setMessage(event.target.value)}
              placeholder="Anyone can read this, for ever"
              data-trade-message
            />
            <p className="mt-1 text-micro text-ink-faint">
              A message is stored in the block with the trade. It is public and permanent:{' '}
              {bytes(message)} of {MESSAGE_LIMIT} bytes.
            </p>

            {refusal && (
              <p className="mt-4 text-caption text-bad" data-trade-refusal>
                {refusal}
              </p>
            )}

            <div className="mt-6 flex flex-wrap items-center gap-2">
              <button
                type="button"
                className="btn btn-primary whitespace-nowrap"
                disabled={!ready}
                onClick={() => setConfirming(true)}
                data-trade-review
              >
                Review this trade
              </button>
              <button type="button" className="btn btn-ghost" onClick={onClose}>
                Cancel
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
