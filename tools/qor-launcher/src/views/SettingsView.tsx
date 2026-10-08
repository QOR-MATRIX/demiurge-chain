/**
 * Settings: identity endpoint, session, and recovery-phrase export.
 */

import { useState } from 'react';
import { Check, Eye, KeyRound, Link2, Loader2, LogOut, ShieldAlert } from 'lucide-react';

import { explain, identity, vault } from '../lib/ipc';
import { selectActiveAccount, useQor } from '../state/store';
import { THEMES } from '../styles/themes';
import { InfoTip } from '../components/ui/InfoTip';
import { Surface } from '../components/ui/Surface';
import { useProgress } from '../components/chrome/Level';
import { BACKDROP_PRESETS, applyBackdrop, currentBackdrop, type BackdropId } from '../qfx/backdrops';
import { AccessibilityPanel } from './AccessibilityPanel';
import { Field, Panel, ViewHeader } from './parts';

export function SettingsView() {
  const session = useQor((s) => s.session);
  const token = useQor((s) => s.token);
  const version = useQor((s) => s.version);
  const signOut = useQor((s) => s.signOut);
  const notify = useQor((s) => s.notify);

  const [authEndpoint, setAuthEndpoint] = useState('');

  const applyAuth = async () => {
    try {
      await identity.setEndpoint(authEndpoint);
      notify('ok', 'Identity endpoint updated.');
    } catch (e) {
      notify('bad', explain(e));
    }
  };

  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <ViewHeader
        eyebrow="Configuration"
        title="Settings"
        body="Endpoints, session and recovery."
      />

      <div className="space-y-4 p-8">
        <AccessibilityPanel />

        <ThemePicker />

        <BackdropPicker />

        <Panel className="p-6">
          <p className="eyebrow mb-5">Identity</p>

          {session ? (
            <dl className="mb-5 grid grid-cols-[110px_1fr] gap-y-2 text-ui">
              <dt className="text-ink-muted">QOR ID</dt>
              <dd className="numeric selectable text-ink">{session.qor_id}</dd>
              <dt className="text-ink-muted">Role</dt>
              <dd className="text-ink">{session.role}</dd>
              <dt className="text-ink-muted">Linked address</dt>
              <dd className="numeric selectable break-all text-ink">
                {session.address ?? 'None linked'}
              </dd>
            </dl>
          ) : (
            <SignIn />
          )}

          <Field label="Identity service" hint="Where QOR ID sign-in is handled">
            <div className="flex gap-2">
              <input
                className="field numeric"
                placeholder="https://id.qorsync.dev/api/v1"
                spellCheck={false}
                value={authEndpoint}
                onChange={(e) => setAuthEndpoint(e.target.value)}
              />
              <button
                type="button"
                className="btn flex-none"
                disabled={!authEndpoint.trim()}
                onClick={() => void applyAuth()}
              >
                Apply
              </button>
            </div>
          </Field>

          {session && (
            <div className="mt-5 flex items-center gap-2">
              <BindWallet />
              <button
                type="button"
                className="btn btn-danger"
                onClick={() => void signOut()}
              >
                <LogOut size={13} />
                Sign out
              </button>
            </div>
          )}
        </Panel>

        <RecoveryPanel />

        <Panel className="p-6">
          <p className="eyebrow mb-4">About</p>
          <dl className="grid grid-cols-[110px_1fr] gap-y-2 text-ui">
            <dt className="text-ink-muted">Launcher</dt>
            <dd className="numeric text-ink">{version}</dd>
            <dt className="text-ink-muted">Token</dt>
            <dd className="text-ink">
              {token?.name} ({token?.symbol}), {token?.decimals} decimals
            </dd>
            <dt className="text-ink-muted">Sub-unit</dt>
            <dd className="text-ink">
              {token
                ? `${(10n ** BigInt(token.decimals)).toString()} ${token.sub_unit}s = 1 ${token.symbol}`
                : '—'}
            </dd>
          </dl>
        </Panel>
      </div>
    </div>
  );
}

/**
 * Signing in to QOR ID when it did not happen at launch, usually because the
 * service was down (ADR-056). The vault's key signs a challenge; there is no
 * password. A key with no QOR ID yet is offered a name.
 */
function SignIn() {
  const setSession = useQor((s) => s.setSession);
  const accounts = useQor((s) => s.accounts);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const [needsName, setNeedsName] = useState(false);
  const [name, setName] = useState('');

  const attempt = async (work: () => Promise<void>) => {
    setBusy(true);
    setProblem(null);
    try {
      await work();
    } catch (e) {
      setProblem(explain(e));
    } finally {
      setBusy(false);
    }
  };

  const signIn = () =>
    attempt(async () => {
      const arrival = await identity.signIn();
      if (arrival.session) setSession(arrival.session);
      else if (arrival.needs_name) setNeedsName(true);
      else setProblem(arrival.sign_in_problem ?? 'QOR ID could not be reached.');
    });

  const claim = () =>
    attempt(async () => {
      const address = accounts[0]?.address;
      if (!address) throw new Error('No account is available to claim a QOR ID with.');
      setSession(await identity.registerWithKey(address, name));
    });

  return (
    <div className="mb-5 space-y-3">
      <p className="text-ui text-ink-muted">
        Not signed in to QOR ID.
      </p>
      {needsName ? (
        <div className="flex gap-2">
          <input
            className="field"
            placeholder="Choose a name"
            aria-label="QOR ID name"
            autoComplete="off"
            spellCheck={false}
            value={name}
            onChange={(e) => setName(e.target.value.trim())}
          />
          <button
            type="button"
            className="btn btn-primary flex-none"
            disabled={!/^[A-Za-z0-9_-]{3,20}$/.test(name) || busy}
            onClick={() => void claim()}
          >
            Claim
          </button>
        </div>
      ) : (
        <button type="button" className="btn" disabled={busy} onClick={() => void signIn()}>
          {busy ? <Loader2 size={13} className="animate-spin" /> : <KeyRound size={13} />}
          Sign in to QOR ID
        </button>
      )}
      {problem && (
        <p role="alert" className="text-ui text-bad">
          {problem}
        </p>
      )}
    </div>
  );
}

/**
 * Recovery-phrase export.
 *
 * Asked for in a host dialog even though the vault is already open (ADR-056).
 * An unlocked session means "this person is using their computer"; it does not
 * mean "it is fine to print the recovery phrase on screen right now". Those are
 * different claims and the second one deserves its own proof.
 */
function RecoveryPanel() {
  const notify = useQor((s) => s.notify);

  const [phrase, setPhrase] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const reveal = async () => {
    setBusy(true);
    try {
      setPhrase(await vault.exportPhrase());
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Panel className="p-6">
      <p className="eyebrow mb-2">Recovery phrase</p>
      <p className="mb-5 flex items-start gap-2 text-ui leading-relaxed text-ink-muted">
        <ShieldAlert size={14} className="mt-[2px] flex-none text-warn" />
        <span>Anyone who reads these words can spend your CGT. Write them down once, alone.</span>
      </p>

      {phrase ? (
        <>
          <ol className="mb-4 grid grid-cols-4 gap-x-3 gap-y-2 border border-edge bg-well p-4">
            {phrase.split(/\s+/).map((word, i) => (
              <li key={`${i}-${word}`} className="flex items-baseline gap-1.5">
                <span className="numeric w-4 flex-none text-right text-micro text-ink-faint">
                  {i + 1}
                </span>
                <span className="selectable text-ui text-ink">{word}</span>
              </li>
            ))}
          </ol>
          <button type="button" className="btn" onClick={() => setPhrase(null)}>
            Hide
          </button>
        </>
      ) : (
        <button
          type="button"
          className="btn"
          disabled={busy}
          onClick={() => void reveal()}
        >
          <Eye size={13} />
          Show recovery phrase
        </button>
      )}
    </Panel>
  );
}

/**
 * The theme picker.
 *
 * Each swatch previews its own palette rather than showing a name and a dot, so
 * the choice is made by looking rather than by reading. Switching is immediate:
 * themes are CSS custom properties, so the whole interface re-colours in one
 * repaint with no reload and no flash.
 *
 * Level unlocks (ADR-078): cosmetic themes are unlocked at level 2 and level 5.
 * The `unlocked` array in progress lists them by name ("Theme: Dusk", etc.).
 * Until the Dusk and Nocturne palettes are built, those entries are shown as
 * locked cosmetics so the player can see what is coming.
 */
function ThemePicker() {
  const theme = useQor((s) => s.theme);
  const setTheme = useQor((s) => s.setTheme);
  const session = useQor((s) => s.session);
  const progress = useProgress(Boolean(session));

  // Which cosmetic names the player has unlocked. Format: "Theme: Foo".
  const unlockedNames = new Set(progress?.unlocked ?? []);

  return (
    <Panel className="p-6">
      <p className="eyebrow mb-5 flex items-center gap-1.5">
        Appearance
        <InfoTip text="One accent carries meaning across the whole interface. Pick the one you want to live inside." />
      </p>

      <div className="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-3">
        {THEMES.map((t, i) => {
          const active = t.id === theme;
          return (
            <Surface
              key={t.id}
              as="button"
              interactive
              cut={active}
              onClick={() => setTheme(t.id)}
              className="stagger flex flex-col gap-3 p-4 text-left"
              style={{ '--i': i } as React.CSSProperties}
              aria-label={`${t.name} theme`}
            >
              <div className="flex items-center gap-2">
                <span className="heading flex-1 text-ui tracking-label">{t.name}</span>
                {active && <Check size={13} className="flex-none text-accent" />}
              </div>

              {/* A real preview: the theme's own ground, panel and accent. */}
              <div
                className="flex h-12 items-end gap-1.5 rounded-qor p-2"
                style={{ background: t.tokens.base, border: `1px solid ${t.tokens.edge}` }}
              >
                <span
                  className="h-full flex-1 rounded-[1px]"
                  style={{ background: t.tokens.surface }}
                />
                <span
                  className="h-full w-3 rounded-[1px]"
                  style={{ background: t.tokens.accent }}
                />
                <span
                  className="h-1/2 w-3 rounded-[1px]"
                  style={{ background: t.tokens.counter }}
                />
              </div>

              <p className="text-caption leading-snug text-ink-muted">{t.note}</p>
            </Surface>
          );
        })}
      </div>

      {/* Upcoming cosmetic theme unlocks (ADR-078). Shown when the player has a
          level and at least one theme unlock is not yet reached. Once the Dusk
          and Nocturne palettes are built they move into the grid above and this
          section shrinks to nothing. */}
      <UpcomingThemeUnlocks unlockedNames={unlockedNames} />
    </Panel>
  );
}

/**
 * The QFX backdrop behind the interface: six shaders, each moving with the
 * pointer and growing richer with the level. Choosing one applies it at once.
 * Whether it moves at all is the Ambience setting's, above, and reduced motion
 * still holds it to one still frame.
 */
function BackdropPicker() {
  const [chosen, setChosen] = useState<BackdropId>(() => currentBackdrop());
  const choose = (id: BackdropId) => {
    applyBackdrop(id);
    setChosen(id);
  };

  return (
    <Panel className="p-6">
      <p className="eyebrow mb-5 flex items-center gap-1.5">
        Backdrop
        <InfoTip text="The living layer behind the interface. Each one reacts to the pointer, and grows richer as your level rises. Ambience, under Accessibility, says whether it moves." />
      </p>
      <div
        role="group"
        aria-label="Backdrop"
        className="grid grid-cols-[repeat(auto-fill,minmax(190px,1fr))] gap-3"
      >
        {BACKDROP_PRESETS.map((b, i) => {
          const active = b.id === chosen;
          return (
            <Surface
              key={b.id}
              as="button"
              interactive
              cut={active}
              aria-pressed={active}
              onClick={() => choose(b.id)}
              className="stagger flex flex-col gap-2 p-4 text-left"
              style={{ '--i': i } as React.CSSProperties}
            >
              <div className="flex items-center gap-2">
                <span className="heading flex-1 text-ui tracking-label">{b.name}</span>
                {active && <Check size={13} className="flex-none text-accent" />}
              </div>
              <p className="text-caption leading-snug text-ink-muted">{b.description}</p>
            </Surface>
          );
        })}
      </div>
    </Panel>
  );
}

/** Cosmetic theme unlocks from the level system that are not yet available as
 *  selectable themes. Shown with their unlock level so a player can see what
 *  earning XP unlocks. */
function UpcomingThemeUnlocks({ unlockedNames }: { unlockedNames: Set<string> }) {
  // Theme entries from the server-side UNLOCKS list that are not yet built into
  // the launcher's THEMES array. Each entry is "Theme: <Name>".
  const pending = [
    { name: 'Dusk', level: 2 },
    { name: 'Nocturne', level: 5 },
  ].filter((t) => !THEMES.some((lt) => lt.name === t.name));

  if (pending.length === 0) return null;

  return (
    <div className="mt-5 border-t border-edge pt-5">
      <p className="eyebrow mb-3 text-ink-muted">Coming with levels</p>
      <div className="flex flex-wrap gap-3">
        {pending.map((t) => {
          const earned = unlockedNames.has(`Theme: ${t.name}`);
          return (
            <div
              key={t.name}
              className="flex items-center gap-2 rounded-qor border border-edge bg-well px-3 py-2 text-ui"
              aria-label={earned ? `${t.name} theme — unlocked` : `${t.name} theme — unlocks at level ${t.level}`}
            >
              <span className={earned ? 'text-accent' : 'text-ink-faint'}>{t.name}</span>
              {earned ? (
                <span className="text-caption text-ok">Unlocked</span>
              ) : (
                <span className="numeric text-caption text-ink-faint">Lv {t.level}</span>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

/**
 * Bind the active vault address to the signed-in QOR ID.
 *
 * Signs a server-issued challenge, so the account cannot claim an address it
 * does not hold. Until this is done the identity and the money are two unrelated
 * facts about the same person.
 */
function BindWallet() {
  const session = useQor((s) => s.session);
  const account = useQor(selectActiveAccount);
  const setSession = useQor((s) => s.setSession);
  const notify = useQor((s) => s.notify);
  const [busy, setBusy] = useState(false);

  if (!account) return null;

  const bound =
    !!session?.address && session.address.toLowerCase() === account.address.toLowerCase();

  if (bound) {
    return (
      <span className="flex items-center gap-2 text-ui text-ink-muted">
        <Check size={13} className="text-ok" />
        Wallet bound to this QOR ID
      </span>
    );
  }

  const bind = async () => {
    setBusy(true);
    try {
      setSession(await identity.linkWallet(account.address));
      notify('ok', 'Your wallet is now bound to your QOR ID.');
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <button type="button" className="btn" disabled={busy} onClick={() => void bind()}>
      <Link2 size={13} />
      {busy ? 'Binding…' : 'Bind wallet'}
    </button>
  );
}
