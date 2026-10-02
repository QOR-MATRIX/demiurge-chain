/**
 * Listing an asset for sale (L4.6).
 *
 * # What it publishes, and what it does not
 *
 * **A listing is a price on chain.** `Drc369Royalties::list` records this asset,
 * its seller and a price in CGT, and a buyer's one transaction then pays the
 * work it was remixed from, its royalties and the seller, and hands the asset
 * over (ADR-061). That is real, public and settled the moment it is in a block.
 *
 * **Nobody browses to it.** A catalogue is an indexer over the chain's events
 * (ADR-028, M5.4), which is not built, so a buyer reaches an asset by its
 * number. The banner at the top of the form says that in the product, not only
 * in a comment, and says what to send a buyer — because a form that looks like
 * it opened a shop is worse than one that says what it did.
 *
 * # What a sale pays is shown before anything is signed
 *
 * As a price is typed, the host works out who a sale at that price pays — with
 * the chain's own arithmetic, from the royalty terms it reads from the chain —
 * and this draws it (`SaleBreakdown.tsx`). The view adds nothing up and parses
 * no amount: a typed price goes to the host as text, where excess precision is
 * refused rather than rounded and no float touches it (AGENTS.md §5).
 *
 * Listing itself is approved in the host's own dialog (roadmap L1.4), which
 * repeats the price and the parts outside the webview. Declined, nothing is
 * signed and nothing is sent.
 *
 * # Two screens, because they are two different acts
 *
 * The first is the listing: a price, and what a sale at it pays. The second is
 * the description — a title, a kind, that kind's questions — which the chain
 * has nowhere to hold, so it is drafted into this machine's data directory
 * (`listings.rs`) and published nowhere. It says so before anything is typed,
 * and it takes its questions from the host's table rather than keeping a second
 * vocabulary here. Keeping them apart means the screen that moves an asset onto
 * the market is never confused with the one that only takes notes.
 */

import { useEffect, useMemo, useRef, useState } from 'react';
import { ArrowLeft, Info, NotebookPen, Store, Trash2, Undo2, X } from 'lucide-react';

import { explain, listings as listingsApi, sales } from '../lib/ipc';
import type {
  Breakdown,
  ListReceipt,
  Listing,
  ListingCategory,
  OwnedAsset,
} from '../lib/ipc';
import { Sigil } from '../qfx/Sigil';
import { SaleBreakdown } from './SaleBreakdown';

/** The host's limits, repeated only to count characters while typing. */
const TITLE_LIMIT = 30;
const NOTES_LIMIT = 600;

/** How long typing rests before the host is asked what a price would pay. */
const PREVIEW_AFTER_MS = 300;

type Preview =
  | { state: 'none' }
  | { state: 'reading' }
  | { state: 'ready'; price: string; breakdown: Breakdown }
  | { state: 'refused'; reason: string };

interface Props {
  asset: OwnedAsset;
  /** The account that holds it, and will sign. */
  from: string;
  symbol: string;
  onClose: () => void;
  /** The chain finalised the listing. */
  onListed: (receipt: ListReceipt) => void;
  /** The chain finalised its withdrawal. */
  onWithdrawn: () => void;
  onSaved: (listing: Listing) => void;
}

export function SellDialog({
  asset,
  from,
  symbol,
  onClose,
  onListed,
  onWithdrawn,
  onSaved,
}: Props) {
  const id = `${asset.collection}/${asset.item}`;
  const live = asset.listing && !asset.listing.void ? asset.listing : null;

  const [describing, setDescribing] = useState(false);
  const [price, setPrice] = useState(live?.price_cgt ?? '');
  const [preview, setPreview] = useState<Preview>({ state: 'none' });
  const [busy, setBusy] = useState<'listing' | 'withdrawing' | 'draft' | null>(null);
  const [refusal, setRefusal] = useState<string | null>(null);

  const [categories, setCategories] = useState<ListingCategory[]>([]);
  const [title, setTitle] = useState(asset.name || '');
  const [category, setCategory] = useState('');
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [notes, setNotes] = useState('');
  const [existing, setExisting] = useState<Listing | null>(null);
  const [saved, setSaved] = useState(false);
  const [draftRefusal, setDraftRefusal] = useState<string | null>(null);

  const priceTouched = useRef(false);
  const asked = useRef(0);
  const priceField = useRef<HTMLInputElement | null>(null);
  const titleField = useRef<HTMLInputElement | null>(null);

  const chosen = useMemo(
    () => categories.find((one) => one.id === category) ?? null,
    [categories, category],
  );

  // The keyboard starts where each screen starts: the price, or the title.
  useEffect(() => {
    (describing ? titleField : priceField).current?.focus();
  }, [describing]);

  useEffect(() => {
    void listingsApi
      .vocabulary()
      .then(setCategories)
      .catch(() => setCategories([]));
    void listingsApi
      .drafts()
      .then((drafts) => {
        const mine = drafts.find(
          (draft) => draft.collection === asset.collection && draft.item === asset.item,
        );
        if (!mine) return;
        setExisting(mine);
        setTitle(mine.title);
        setCategory(mine.category);
        setAnswers(Object.fromEntries(mine.details.map((detail) => [detail.field, detail.value])));
        setNotes(mine.notes);
        // The price on chain wins; a draft's price only fills an empty field.
        if (!priceTouched.current) setPrice((was) => (was === '' ? mine.price_cgt : was));
      })
      .catch(() => undefined);
  }, [asset.collection, asset.item]);

  // What a sale at the typed price would pay, asked of the host once typing
  // rests. A late answer for a price no longer in the field is dropped.
  useEffect(() => {
    const typed = price.trim();
    const mine = ++asked.current;
    if (typed === '') {
      setPreview({ state: 'none' });
      return;
    }
    setPreview({ state: 'reading' });
    const timer = window.setTimeout(() => {
      sales
        .preview(asset.collection, asset.item, typed)
        .then((breakdown) => {
          if (asked.current === mine) setPreview({ state: 'ready', price: typed, breakdown });
        })
        .catch((error) => {
          if (asked.current === mine) setPreview({ state: 'refused', reason: explain(error) });
        });
    }, PREVIEW_AFTER_MS);
    return () => window.clearTimeout(timer);
  }, [price, asset.collection, asset.item]);

  const answer = (field: string, value: string) =>
    setAnswers((was) => ({ ...was, [field]: was[field] === value ? '' : value }));

  const samePrice =
    live !== null && preview.state === 'ready' && preview.breakdown.price_sparks === live.price_sparks;
  const canList =
    preview.state === 'ready' &&
    preview.price === price.trim() &&
    preview.breakdown.blocked === null &&
    !samePrice &&
    busy === null;

  const list = async () => {
    setBusy('listing');
    setRefusal(null);
    try {
      onListed(await sales.list(from, asset.collection, asset.item, price.trim()));
    } catch (error) {
      setRefusal(explain(error));
      setBusy(null);
    }
  };

  const withdraw = async () => {
    setBusy('withdrawing');
    setRefusal(null);
    try {
      await sales.unlist(from, asset.collection, asset.item);
      onWithdrawn();
    } catch (error) {
      setRefusal(explain(error));
      setBusy(null);
    }
  };

  const save = async () => {
    setBusy('draft');
    setDraftRefusal(null);
    try {
      const listing = await listingsApi.save({
        collection: asset.collection,
        item: asset.item,
        title,
        category,
        details: Object.entries(answers)
          .filter(([, value]) => value.trim().length > 0)
          .map(([field, value]) => ({ field, value })),
        price_cgt: price,
        notes,
      });
      setExisting(listing);
      setSaved(true);
      onSaved(listing);
    } catch (error) {
      setDraftRefusal(explain(error));
    } finally {
      setBusy(null);
    }
  };

  const discard = async () => {
    setBusy('draft');
    setDraftRefusal(null);
    try {
      await listingsApi.discard(asset.collection, asset.item);
      setExisting(null);
      setSaved(false);
      setCategory('');
      setAnswers({});
      setNotes('');
      setTitle(asset.name || '');
    } catch (error) {
      setDraftRefusal(explain(error));
    } finally {
      setBusy(null);
    }
  };

  const draftReady = title.trim().length > 0 && category !== '' && price.trim().length > 0;
  const waiting = busy === 'listing' || busy === 'withdrawing';
  const heading = describing ? 'Describe it' : live ? 'Your listing' : 'List for sale';

  return (
    <div className="modal" data-sell-dialog onClick={waiting ? undefined : onClose}>
      <div
        role="dialog"
        aria-modal="true"
        aria-label={heading}
        className="modal-panel surface cut p-7"
        data-sell-step={describing ? 'describe' : 'price'}
        onClick={(event) => event.stopPropagation()}
        onKeyDown={(event) => {
          if (event.key === 'Escape' && !waiting) onClose();
        }}
      >
        <div className="mb-4 flex items-baseline justify-between gap-4">
          <div>
            <p className="eyebrow text-accent">Sell</p>
            <p className="heading text-heading text-ink">{heading}</p>
          </div>
          <button
            type="button"
            className="btn btn-ghost"
            onClick={onClose}
            disabled={waiting}
            aria-label="Close"
          >
            <X size={14} />
          </button>
        </div>

        {/* Keyed, so each screen mounts fresh rather than one turning into the
            other in place (the trade dialog records why). */}
        {describing ? (
          <div key="describe" data-sell-description>
            {/* Said first, in the product, and not only in a comment. */}
            <div className="mb-4 flex items-start gap-2 border border-edge p-3" data-sell-unpublished>
              <Info size={14} className="mt-0.5 flex-none text-ink-faint" />
              <p className="text-caption text-ink-body">
                <span className="text-ink">A description is not published.</span> The chain holds a
                listing&rsquo;s price and nothing else, so the Market shows no title or description
                (M5.4). What you write here is saved on this machine only.
              </p>
            </div>

            <label className="block text-caption text-ink-muted" htmlFor="sell-title">
              Title
            </label>
            <input
              id="sell-title"
              ref={titleField}
              className="field mt-1"
              value={title}
              maxLength={TITLE_LIMIT}
              onChange={(event) => setTitle(event.target.value)}
              data-sell-title
            />
            <p className="mt-1 text-micro text-ink-faint">
              {title.length} of {TITLE_LIMIT} characters
            </p>

            <p className="mt-4 text-caption text-ink-muted" id="sell-kind">
              What kind of thing is it?
            </p>
            <div
              className="mt-2 flex flex-wrap gap-2"
              role="group"
              aria-labelledby="sell-kind"
              data-sell-categories
            >
              {categories.map((one) => (
                <button
                  key={one.id}
                  type="button"
                  className={`btn whitespace-nowrap ${category === one.id ? 'btn-primary' : ''}`}
                  aria-pressed={category === one.id}
                  onClick={() => {
                    setCategory(one.id);
                    setAnswers({});
                  }}
                  data-sell-category={one.id}
                >
                  {one.name}
                </button>
              ))}
            </div>

            {chosen?.note && (
              <p className="mt-3 border border-edge p-3 text-caption text-ink-body" data-sell-note>
                {chosen.note}
              </p>
            )}

            {/* Whatever the chosen category asks, and nothing this view invents. */}
            {chosen && (
              <div className="mt-4 space-y-4" data-sell-fields>
                {chosen.fields.map((field) => (
                  <div key={field.id} data-sell-field={field.id}>
                    {field.options.length > 0 ? (
                      <>
                        <p className="text-caption text-ink-muted" id={`sell-field-${field.id}`}>
                          {field.label}
                        </p>
                        <div
                          className="mt-2 flex flex-wrap gap-2"
                          role="group"
                          aria-labelledby={`sell-field-${field.id}`}
                        >
                          {field.options.map((option) => (
                            <button
                              key={option}
                              type="button"
                              className={`btn whitespace-nowrap ${
                                answers[field.id] === option ? 'btn-primary' : 'btn-ghost'
                              }`}
                              aria-pressed={answers[field.id] === option}
                              onClick={() => answer(field.id, option)}
                              data-sell-option={`${field.id}:${option}`}
                            >
                              {option}
                            </button>
                          ))}
                        </div>
                      </>
                    ) : (
                      <>
                        <label
                          className="block text-caption text-ink-muted"
                          htmlFor={`sell-answer-${field.id}`}
                        >
                          {field.label}
                        </label>
                        <input
                          id={`sell-answer-${field.id}`}
                          className="field mt-1"
                          value={answers[field.id] ?? ''}
                          onChange={(event) =>
                            setAnswers((was) => ({ ...was, [field.id]: event.target.value }))
                          }
                          data-sell-answer={field.id}
                        />
                      </>
                    )}
                  </div>
                ))}
              </div>
            )}

            <label className="mt-5 block text-caption text-ink-muted" htmlFor="sell-notes">
              Description (optional)
            </label>
            <textarea
              id="sell-notes"
              className="field mt-1"
              rows={3}
              value={notes}
              maxLength={NOTES_LIMIT}
              onChange={(event) => setNotes(event.target.value)}
              data-sell-notes
            />
            <p className="mt-1 text-micro text-ink-faint">
              {notes.length} of {NOTES_LIMIT} characters
            </p>

            <p className="mt-4 text-caption text-ink-muted" data-sell-draft-price>
              {price.trim() === '' ? (
                'A draft keeps the price typed on the listing screen. None is typed yet.'
              ) : (
                <>
                  Price kept with this draft: <span className="numeric text-ink-body">{price}</span>{' '}
                  {symbol}, as typed on the listing screen.
                </>
              )}
            </p>

            {draftRefusal && (
              <p className="mt-4 text-caption text-bad" role="alert" data-sell-draft-refusal>
                {draftRefusal}
              </p>
            )}
            {saved && !draftRefusal && (
              <p className="mt-4 text-caption text-ok" data-sell-saved>
                Saved on this machine. It is not published anywhere.
              </p>
            )}

            <div className="mt-6 flex flex-wrap items-center gap-2">
              <button
                type="button"
                className="btn btn-primary whitespace-nowrap"
                disabled={!draftReady || busy !== null}
                onClick={() => void save()}
                data-sell-save
              >
                <NotebookPen size={13} />
                {existing ? 'Save changes' : 'Save this draft'}
              </button>
              {existing && (
                <button
                  type="button"
                  className="btn btn-ghost whitespace-nowrap"
                  onClick={() => void discard()}
                  disabled={busy !== null}
                  data-sell-discard
                >
                  <Trash2 size={13} />
                  Delete draft
                </button>
              )}
              <button
                type="button"
                className="btn btn-ghost whitespace-nowrap"
                onClick={() => setDescribing(false)}
                data-sell-back
              >
                <ArrowLeft size={13} />
                Back to the listing
              </button>
            </div>
          </div>
        ) : (
          <div key="price">
            {/* Said first, in the product, and not only in a comment. */}
            <div className="mb-4 flex items-start gap-2 border border-edge p-3" data-sell-reach>
              <Info size={14} className="mt-0.5 flex-none text-ink-faint" />
              <p className="text-caption text-ink-body">
                <span className="text-ink">A listing is public and on chain:</span> this asset, your
                address and the price. It appears in the Market, which reads listings straight from
                the chain and has no search yet (M5.4). To point a buyer at it, send them its
                number, <span className="numeric text-ink">{id}</span>: use Copy reference in the
                asset&rsquo;s menu.
              </p>
            </div>

            <div className="grid gap-5 sm:grid-cols-[13rem_1fr]">
              {/* The asset, as a container beside its details. */}
              <section className="trade-side" data-sell-asset>
                <p className="eyebrow text-ink-muted">What you are listing</p>
                <Sigil root={asset.current.root} className="mx-auto my-4" />
                <p className="truncate text-body font-semibold text-ink">
                  {asset.name || 'Untitled'}
                </p>
                <p className="numeric text-micro text-ink-faint">
                  Asset {id} · {asset.revisable ? 'Revisable' : 'Permanent'}
                </p>
                <p className="numeric mt-2 break-all text-micro text-ink-muted">
                  {asset.current.algo} {asset.current.root.slice(0, 16)}…
                </p>
                {asset.revisable && (
                  <p className="mt-3 text-micro text-ink-faint">
                    This asset can still be revised. A buyer gets what it points at when they buy,
                    and the launcher stops a purchase if that changed since they looked.
                  </p>
                )}
              </section>

              <section className="min-w-0">
                <label className="block text-caption text-ink-muted" htmlFor="sell-price">
                  Price, in {symbol}
                </label>
                <input
                  id="sell-price"
                  ref={priceField}
                  className="field numeric mt-1"
                  value={price}
                  onChange={(event) => {
                    priceTouched.current = true;
                    setPrice(event.target.value);
                  }}
                  onKeyDown={(event) => {
                    // Enter lists, exactly when the button would.
                    if (event.key === 'Enter' && canList) void list();
                  }}
                  placeholder="0.00"
                  inputMode="decimal"
                  spellCheck={false}
                  autoComplete="off"
                  aria-describedby="sell-price-help"
                  data-sell-price
                />
                <p id="sell-price-help" className="mt-1 text-micro text-ink-faint">
                  {live ? (
                    <span data-sell-current>
                      Listed now at{' '}
                      <span className="numeric text-ink-body">
                        {live.price_cgt} {symbol}
                      </span>
                      . Type a different price to change it.
                    </span>
                  ) : (
                    'Your own number. Nothing here suggests one.'
                  )}
                </p>

                <div className="mt-4" aria-live="polite" data-sell-preview={preview.state}>
                  <p className="eyebrow text-ink-muted">What a sale pays</p>
                  {preview.state === 'none' && (
                    <p className="mt-2 text-caption text-ink-muted">
                      Type a price to see who a sale at that price pays.
                    </p>
                  )}
                  {preview.state === 'reading' && (
                    <p className="mt-2 text-caption text-ink-muted">Reading the royalty terms…</p>
                  )}
                  {preview.state === 'refused' && (
                    <p className="mt-2 text-caption text-bad" data-sell-price-refusal>
                      {preview.reason}
                    </p>
                  )}
                  {preview.state === 'ready' && (
                    <div className="mt-2">
                      <SaleBreakdown
                        breakdown={preview.breakdown}
                        reader="seller"
                        symbol={symbol}
                      />
                    </div>
                  )}
                </div>
              </section>
            </div>

            {refusal && (
              <p className="mt-4 text-caption text-bad" role="alert" data-sell-refusal>
                {refusal}
              </p>
            )}

            <div className="mt-6 flex flex-wrap items-center gap-2">
              <button
                type="button"
                className="btn btn-primary whitespace-nowrap"
                disabled={!canList}
                onClick={() => void list()}
                data-sell-list
              >
                <Store size={13} />
                {busy === 'listing'
                  ? 'Waiting for the chain…'
                  : live
                    ? 'Change the price'
                    : 'List for sale'}
              </button>
              {asset.listing && (
                <button
                  type="button"
                  className="btn whitespace-nowrap"
                  onClick={() => void withdraw()}
                  disabled={busy !== null}
                  data-sell-withdraw
                >
                  <Undo2 size={13} />
                  {busy === 'withdrawing'
                    ? 'Waiting for the chain…'
                    : asset.listing.void
                      ? 'Clear void listing'
                      : 'Withdraw listing'}
                </button>
              )}
              <button
                type="button"
                className="btn btn-ghost whitespace-nowrap"
                onClick={() => setDescribing(true)}
                disabled={waiting}
                data-sell-describe
              >
                <NotebookPen size={13} />
                {existing ? 'Your description' : 'Describe it'}
              </button>
              <button type="button" className="btn btn-ghost" onClick={onClose} disabled={waiting}>
                Close
              </button>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
