/**
 * The Nexus: how you find and move between every system in the ecosystem.
 *
 * This is the launcher's centre of gravity, not a dashboard. A dashboard shows
 * you numbers; this shows you *doors*. Everything else in the app is reachable
 * from here in one keystroke or one click.
 *
 * # Why it is built this way
 *
 * Three decisions carry the whole surface:
 *
 * **Search is focused on arrival.** Type immediately, from anywhere, and the
 * grid filters. Muscle memory beats navigation for a tool opened every day, and
 * a launcher whose primary verb is "find the thing" should behave like one.
 *
 * **Tiles tell the truth.** A system that is not built says so, and says what it
 * is waiting on. The alternative — everything looking available — trains people
 * to distrust every status indicator in the product.
 *
 * **Progression is the onboarding.** The Ascent strip is not scorekeeping; it is
 * a readout of verifiable standing, and the next incomplete rite is always the
 * next useful thing to do. A newcomer is never left wondering where to start.
 */

import { useEffect, useMemo, useRef, useState } from 'react';
import {
  Bot,
  Box,
  Boxes,
  Brain,
  ChevronRight,
  Code2,
  Globe,
  Layers,
  Library,
  ListChecks,
  MessagesSquare,
  Music4,
  Radio,
  HelpCircle,
  Search,
  Shapes,
  Share2,
  Sparkles,
  Store,
  Target,
  Users,
  Wallet,
  type LucideIcon,
} from 'lucide-react';

import { explain, shortAddress } from '../lib/ipc';
import {
  byDomain,
  DOMAINS,
  searchSystems,
  STATE_LABEL,
  type System,
  type SystemState,
} from '../lib/systems';
import {
  selectActiveAccount,
  selectActiveBalance,
  useAscent,
  useQor,
  type Surface as SurfaceId,
} from '../state/store';
import { Surface } from '../components/ui/Surface';
import { Orientation, useOrientation } from '../components/nexus/Orientation';

const GLYPHS: Record<string, LucideIcon> = {
  wallet: Wallet,
  radio: Radio,
  boxes: Boxes,
  search: Search,
  layers: Layers,
  sparkles: Sparkles,
  brain: Brain,
  shapes: Shapes,
  music: Music4,
  library: Library,
  globe: Globe,
  messages: MessagesSquare,
  users: Users,
  share: Share2,
  bot: Bot,
  code: Code2,
  target: Target,
  checks: ListChecks,
  store: Store,
};

export function Nexus() {
  const [query, setQuery] = useState('');
  const orientation = useOrientation();
  const searchRef = useRef<HTMLInputElement>(null);

  // Type anywhere to search. A launcher's primary verb is "find", so the
  // keyboard should reach the filter without anyone aiming at a box first.
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      const target = event.target as HTMLElement | null;
      const typingElsewhere =
        target instanceof HTMLInputElement || target instanceof HTMLTextAreaElement;

      if (typingElsewhere) return;

      if (event.key === 'Escape') {
        setQuery('');
        searchRef.current?.blur();
        return;
      }
      if (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey) {
        searchRef.current?.focus();
      }
    };

    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  const groups = useMemo(() => {
    if (!query.trim()) return byDomain();
    const matches = searchSystems(query);
    return byDomain()
      .map((g) => ({ ...g, systems: g.systems.filter((s) => matches.includes(s)) }))
      .filter((g) => g.systems.length > 0);
  }, [query]);

  const total = groups.reduce((n, g) => n + g.systems.length, 0);

  return (
    <div className="relative flex h-full flex-col overflow-hidden">
      <Masthead
        query={query}
        setQuery={setQuery}
        searchRef={searchRef}
        found={total}
        orientationHidden={orientation.dismissed}
        onShowOrientation={orientation.restore}
      />

      <div className="relative z-10 min-h-0 flex-1 overflow-y-auto px-8 pb-10">
        {!orientation.dismissed && !query.trim() && (
          <Orientation onDismiss={orientation.dismiss} />
        )}

        {total === 0 ? (
          <p className="py-16 text-center text-body text-ink-muted">
            Nothing matches “{query}”.
          </p>
        ) : (
          groups.map((group, gi) => (
            <section key={group.domain} className="mb-9">
              <header className="mb-3 flex items-baseline gap-3">
                <h2 className="eyebrow text-accent">{DOMAINS[group.domain].name}</h2>
                <span className="text-caption text-ink-faint">
                  {DOMAINS[group.domain].note}
                </span>
              </header>

              <div className="grid grid-cols-[repeat(auto-fill,minmax(290px,1fr))] gap-3">
                {group.systems.map((system, i) => (
                  <SystemTile
                    key={system.id}
                    system={system}
                    index={gi * 4 + i}
                  />
                ))}
              </div>
            </section>
          ))
        )}
      </div>
    </div>
  );
}

/* ─────────────────────────────── masthead ──────────────────────────────── */

function Masthead({
  query,
  setQuery,
  searchRef,
  found,
  orientationHidden,
  onShowOrientation,
}: {
  query: string;
  setQuery: (v: string) => void;
  searchRef: React.RefObject<HTMLInputElement | null>;
  found: number;
  orientationHidden: boolean;
  onShowOrientation: () => void;
}) {
  const session = useQor((s) => s.session);
  const account = useQor(selectActiveAccount);
  const balance = useQor(selectActiveBalance);
  const token = useQor((s) => s.token);
  const ascent = useAscent();

  return (
    <header className="relative z-10 flex-none border-b border-edge px-8 pb-5 pt-7">
      <div className="flex items-start gap-8">
        <div className="min-w-0 flex-1">
          <p
            className="eyebrow mb-1.5 text-accent"
            title="Your standing reflects how much of the ecosystem you have set up. It is not a score."
          >
            {ascent.standing}
          </p>
          <h1 className="heading mb-1 text-display">
            {session ? session.username : 'The Nexus'}
          </h1>
          <p className="numeric text-caption text-ink-muted">
            {session?.qor_id ?? 'Not signed in'}
            {account && <> · {shortAddress(account.address, 8, 6)}</>}
          </p>
        </div>

        {/* Holdings, always in view: this is a wallet as much as a launcher. */}
        <Surface
          className="flex-none px-5 py-3.5"
          cut
          title={`The ${token?.name ?? 'Creator God Token'} you hold. Only moves when you sign a transfer.`}
        >
          <p className="eyebrow mb-1.5">Holdings</p>
          <p className="flex items-baseline gap-1.5">
            <span className="numeric text-display text-ink">{balance?.cgt ?? '—'}</span>
            <span className="eyebrow text-accent">{token?.symbol ?? 'CGT'}</span>
          </p>
        </Surface>

        <AscentStrip />
      </div>

      <div className="mt-5 flex items-center gap-3">
        <div className="relative flex-1">
          <Search
            size={14}
            className="pointer-events-none absolute left-3 top-1/2 -translate-y-1/2 text-ink-faint"
          />
          <input
            ref={searchRef}
            className="field pl-9"
            placeholder="Search systems — just start typing"
            value={query}
            spellCheck={false}
            onChange={(e) => setQuery(e.target.value)}
          />
        </div>
        <span className="numeric flex-none text-caption text-ink-faint">
          {found} {found === 1 ? 'system' : 'systems'}
        </span>

        {orientationHidden && (
          <button
            type="button"
            className="btn btn-ghost flex-none"
            onClick={onShowOrientation}
            title="Explain this screen again"
          >
            <HelpCircle size={13} />
            Guide
          </button>
        )}
      </div>
    </header>
  );
}

function AscentStrip() {
  const ascent = useAscent();
  const percent = Math.round((ascent.completed / ascent.total) * 100);

  return (
    <Surface
      className="w-[290px] flex-none px-5 py-3.5"
      cut
      title="Seven setup steps, each checked against your real account state rather than awarded."
    >
      <div className="mb-2 flex items-baseline justify-between">
        <p className="eyebrow">The Ascent</p>
        <p className="numeric text-caption text-ink-muted">
          {ascent.completed}/{ascent.total}
        </p>
      </div>
      <div className="rail mb-2.5">
        <div className="rail-fill" style={{ width: `${percent}%` }} />
      </div>
      {ascent.next ? (
        <p className="text-caption leading-snug text-ink-muted">
          <span className="text-ink">{ascent.next.name}</span> — {ascent.next.how}
        </p>
      ) : (
        <p className="text-caption text-accent">Every rite complete.</p>
      )}
    </Surface>
  );
}

/* ───────────────────────────────── tiles ────────────────────────────────── */

function SystemTile({ system, index }: { system: System; index: number }) {
  const go = useQor((s) => s.go);
  const notify = useQor((s) => s.notify);
  const chainStatus = useQor((s) => s.chainStatus);
  const Icon = GLYPHS[system.glyph] ?? Box;

  const open = async () => {
    if (system.surface) {
      go(system.surface as SurfaceId);
      return;
    }

    if (system.state === 'forming') {
      notify('bad', `${system.name} is not built yet. ${system.blockedOn ?? ''}`.trim());
      return;
    }

    // Prefer the local instance when the launcher is already pointed at a local
    // node: someone running the stack locally almost certainly wants the local
    // app, not production.
    const local = /(127\.0\.0\.1|localhost)/.test(chainStatus?.endpoint ?? '');
    const target = (local && system.localUrl) || system.url || system.localUrl;

    if (!target) return;

    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener');
      await openUrl(target);
    } catch (e) {
      notify('bad', explain(e));
    }
  };

  const forming = system.state === 'forming';

  return (
    <Surface
      as="button"
      interactive={!forming}
      cut
      onClick={() => void open()}
      className={`stagger flex flex-col gap-3 p-5 text-left ${forming ? 'opacity-60' : ''}`}
      style={{ '--i': index } as React.CSSProperties}
      aria-label={`${system.name}: ${system.tagline}`}
    >
      <div className="flex items-start gap-3">
        <Icon
          size={19}
          strokeWidth={1.5}
          className={forming ? 'mt-0.5 flex-none text-ink-faint' : 'mt-0.5 flex-none text-accent'}
        />
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h3 className="heading truncate text-body">{system.name}</h3>
            <StateBadge state={system.state} />
          </div>
        </div>
        {!forming && (
          <ChevronRight size={15} className="mt-0.5 flex-none text-ink-faint" />
        )}
      </div>

      <p className="text-ui leading-relaxed text-ink-muted">{system.tagline}</p>

      {forming && system.blockedOn && (
        <p className="border-l border-edge pl-2.5 text-caption leading-snug text-ink-faint">
          {system.blockedOn}
        </p>
      )}
    </Surface>
  );
}

function StateBadge({ state }: { state: SystemState }) {
  const tone =
    state === 'live'
      ? 'text-ok'
      : state === 'local'
        ? 'text-accent'
        : 'text-ink-faint';

  return (
    <span className={`eyebrow flex flex-none items-center gap-1.5 text-micro ${tone}`}>
      <span
        className={`dot ${
          state === 'live' ? 'dot-ok' : state === 'local' ? 'dot-warn' : ''
        }`}
        style={state === 'forming' ? { background: 'var(--ink-faint)' } : undefined}
      />
      {STATE_LABEL[state]}
    </span>
  );
}

/* ──────────────────────────── starter grant ────────────────────────────── */

/**
 * Offered when the account holds nothing.
 *
 * The identity service used to mint this on a new user's behalf by calling the
 * faucet unauthenticated. That hole is closed, so the claim now happens here,
 * signed by the key the user actually holds. Exported for the Vault surface.
 */
export function ClaimGrant() {
  const account = useQor(selectActiveAccount);
  const balance = useQor(selectActiveBalance);

  const empty = balance ? BigInt(balance.sparks) === 0n : false;
  if (!account || !empty) return null;

  // The launcher can talk to the chain since ADR-040, and sign for it. What is
  // missing now is on the chain's side: it has no issuance mechanism, because
  // the issuance rate and the genesis split are open economic questions
  // (OPEN-1, OPEN-2) and nothing may create CGT until they are decided.
  // Rather than offer a button that fails, the card says so.
  return (
    <Surface className="mb-6 flex items-center gap-4 p-5" cut>
      <Sparkles size={18} className="flex-none text-accent" />
      <div className="min-w-0 flex-1">
        <p className="mb-0.5 text-body font-semibold text-ink">No CGT in this account</p>
        <p className="text-ui text-ink-muted">
          There is no starter grant yet: the chain has no way to issue CGT, and will not have
          one until how much is issued, and to whom, is decided. Your key and your address are
          ready for it.
        </p>
      </div>
    </Surface>
  );
}
