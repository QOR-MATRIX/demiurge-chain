/**
 * The Gate: what shows before a wallet is open.
 *
 * Since ADR-056 a returning person never sees it: the signed-in profile's key is
 * in the operating system's keychain and the launcher opens it before its first
 * frame. The Gate is left for these cases:
 *
 *   signed out (ADR-099)  →  Who's playing?: this PC's QOR IDs, each opened with its PIN,
 *                            a new QOR ID, or one restored from its recovery phrase
 *   no QOR ID on this PC  →  make one, or restore one from a recovery phrase
 *   a Windows Hello vault (ADR-055)  →  Hello one last time, and it moves
 *   a passphrase vault (before ADR-055)  →  the passphrase one last time
 *   a keychain vault that would not open  →  why, and the recovery phrase
 *
 * QOR ID is not part of it. Opening the vault tries to sign in; a service that
 * is down stops nothing, and Settings offers to try again.
 */

import { useEffect, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import {
  ArrowLeft,
  ArrowRight,
  Fingerprint,
  KeyRound,
  Loader2,
  Lock,
  RotateCw,
  TriangleAlert,
  UserPlus,
} from 'lucide-react';

import {
  asQorError,
  explain,
  profiles as profileCalls,
  shortAddress,
  validPin,
  vault,
  type Arrival,
  type ProfileView,
} from '../../lib/ipc';
import { PinField } from '../ui/PinField';
import { useQor } from '../../state/store';
import { Connection } from './Connection';

type Step = 'checking' | 'who' | 'pin' | 'welcome' | 'restore' | 'unlock' | 'move';

export function Gate() {
  const vaultStatus = useQor((s) => s.vaultStatus);
  const refreshVault = useQor((s) => s.refreshVault);
  const refreshProfiles = useQor((s) => s.refreshProfiles);
  const setSession = useQor((s) => s.setSession);
  const profiles = useQor((s) => s.profiles);
  // The QOR ID chosen on Who's playing?, whose PIN is asked.
  const [chosen, setChosen] = useState<ProfileView | null>(null);
  // The person asked for a new QOR ID from Who's playing?, so the list arriving does not take them back to it.
  const [creating, setCreating] = useState(false);

  const [step, setStep] = useState<Step>('checking');
  const [showConnection, setShowConnection] = useState(false);
  const chainStatus = useQor((s) => s.chainStatus);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // A phrase the person already holds, typed to restore, and the address it
  // opens. Held only until it is sealed or abandoned.
  const [restoring, setRestoring] = useState<{ phrase: string; address: string } | null>(null);
  const [asked, setAsked] = useState(false);

  const sealedWith = vaultStatus.state === 'locked' ? vaultStatus.sealed_with : null;

  // Pick the step from the vault, but never pull the person out of restoring, a PIN or a new QOR ID.
  useEffect(() => {
    if (step === 'restore' || step === 'pin' || creating) return;
    if (vaultStatus.state === 'absent') setStep(profiles.length > 0 ? 'who' : 'welcome');
    else if (sealedWith === 'passphrase') setStep('move');
    else if (vaultStatus.state === 'locked') setStep('unlock');
  }, [vaultStatus.state, sealedWith, step, profiles.length, creating]);

  const run = async (action: () => Promise<void>) => {
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (e) {
      if (asQorError(e).kind === 'passphrase_vault') setStep('move');
      else setError(explain(e));
    } finally {
      setBusy(false);
    }
  };

  /** The vault is open: in, signed in to QOR ID or not. */
  const arrive = async (arrival: Arrival) => {
    if (arrival.session) setSession(arrival.session);
    await refreshVault();
    await refreshProfiles();
  };

  const choose = (profile: ProfileView) => {
    setError(null);
    setChosen(profile);
    setStep('pin');
  };

  const open = (pin: string) =>
    run(async () => {
      if (!chosen) return;
      try {
        await arrive(await profileCalls.open(chosen.id, pin));
        setChosen(null);
      } finally {
        await refreshProfiles();
      }
    });

  const toWho = () => {
    setError(null);
    setChosen(null);
    setCreating(false);
    setStep(profiles.length > 0 ? 'who' : 'welcome');
  };

  const begin = () =>
    run(async () => {
      const phrase = await vault.generatePhrase();
      await arrive(await vault.create(phrase));
    });

  const startRestore = () => {
    setError(null);
    setRestoring(null);
    setStep('restore');
  };

  const leaveRestore = () => {
    setError(null);
    setRestoring(null);
    setAsked(true); // back from restoring, Windows Hello is not asked again on its own
    setStep(
      vaultStatus.state === 'absent'
        ? profiles.length > 0
          ? 'who'
          : 'welcome'
        : sealedWith === 'passphrase'
          ? 'move'
          : 'unlock',
    );
  };

  const checkPhrase = (typed: string) =>
    run(async () => {
      const phrase = typed.trim().toLowerCase().split(/\s+/).join(' ');
      setRestoring({ phrase, address: await vault.previewPhrase(phrase) });
    });

  const restoreVault = () =>
    run(async () => {
      if (!restoring) throw new Error('The recovery phrase was lost. Start again.');
      const arrival = await vault.restore(restoring.phrase);
      setRestoring(null);
      await arrive(arrival);
    });

  // A vault sealed under Windows Hello asks it once on arrival, one last time;
  // a cancel is not asked again on its own, and the button stays.
  const unlock = () => run(async () => arrive(await vault.unlock()));

  useEffect(() => {
    if (step === 'unlock' && sealedWith === 'hello' && !asked && !busy) {
      setAsked(true);
      void unlock();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step, sealedWith, asked, busy]);

  const move = (passphrase: string) =>
    run(async () => arrive(await vault.moveToKeychain(passphrase)));

  return (
    <div className="relative flex h-full w-full items-center justify-center overflow-hidden bg-base">
      <AnimatePresence mode="wait">
        <motion.div
          key={step}
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -8 }}
          transition={{ duration: 0.28, ease: [0.16, 1, 0.3, 1] }}
          className="glass-solid cut relative z-10 w-[460px] p-8"
        >
          {showConnection ? (
            <Connection onClose={() => setShowConnection(false)} />
          ) : (
            <>
              {step === 'checking' && <Centered>Opening…</Centered>}

              {step === 'who' && (
                <Who
                  profiles={profiles}
                  onChoose={choose}
                  onNew={() => {
                    setError(null);
                    setCreating(true);
                    setStep('welcome');
                  }}
                  onRestore={startRestore}
                />
              )}

              {step === 'pin' && chosen && (
                <PinStep busy={busy} profile={chosen} onOpen={open} onBack={toWho} />
              )}

              {step === 'welcome' && (
                <Welcome
                  busy={busy}
                  onBegin={begin}
                  onRestore={startRestore}
                  onBack={profiles.length > 0 ? toWho : undefined}
                />
              )}

              {step === 'restore' && (
                <RestoreStep
                  busy={busy}
                  replacing={vaultStatus.state !== 'absent'}
                  checked={restoring}
                  onCheck={checkPhrase}
                  onEdit={() => setRestoring(null)}
                  onContinue={restoreVault}
                  onBack={leaveRestore}
                />
              )}

              {step === 'unlock' && (
                <UnlockStep
                  busy={busy}
                  hello={sealedWith === 'hello'}
                  onUnlock={unlock}
                  onRestore={startRestore}
                />
              )}

              {step === 'move' && <MoveStep busy={busy} onSubmit={move} onRestore={startRestore} />}
            </>
          )}

          {!showConnection && (
            <button
              type="button"
              onClick={() => setShowConnection(true)}
              className={`mt-6 flex w-full items-center justify-center gap-2 border-t border-edge pt-4 text-caption transition-colors ${
                chainStatus && !chainStatus.reachable
                  ? 'text-bad hover:text-ink'
                  : 'text-ink-faint hover:text-ink-muted'
              }`}
            >
              <span className={`dot ${chainStatus?.reachable ? 'dot-ok' : 'dot-bad'}`} />
              {chainStatus?.reachable
                ? 'Connected. Change network'
                : 'Cannot reach the network. Choose one'}
            </button>
          )}

          {error && (
            <p
              role="alert"
              className="mt-5 flex items-start gap-2 border-l-2 border-bad bg-bad/5 px-3 py-2 text-ui text-bad"
            >
              <TriangleAlert size={13} className="mt-[3px] flex-none" />
              <span>{error}</span>
            </p>
          )}
        </motion.div>
      </AnimatePresence>
    </div>
  );
}

/* ──────────────────────────────── steps ─────────────────────────────────── */

/**
 * Who's playing? (ADR-099): this PC's QOR IDs by name, nothing else. No balance is shown until a QOR ID is opened with
 * its PIN.
 */
function Who({
  profiles,
  onChoose,
  onNew,
  onRestore,
}: {
  profiles: ProfileView[];
  onChoose: (profile: ProfileView) => void;
  onNew: () => void;
  onRestore: () => void;
}) {
  return (
    <div data-gate-who>
      <Header
        eyebrow="Demiurge"
        title="Who&apos;s playing?"
        body="Each QOR ID on this PC has its own wallet. Choose yours."
      />
      <ul className="mb-4 grid gap-2" aria-label="QOR IDs on this PC">
        {profiles.map((p) => (
          <li key={p.id}>
            <button
              type="button"
              className="flex w-full items-center gap-3 border border-edge bg-well px-4 py-3 text-left transition-colors hover:border-accent"
              onClick={() => onChoose(p)}
              data-gate-profile={p.id}
            >
              <Lock size={14} className="flex-none text-ink-muted" aria-hidden="true" />
              <span className="min-w-0 flex-1">
                <span className="block truncate text-ui font-semibold text-ink">
                  {p.name ?? 'QOR ID without a name'}
                </span>
                {!p.name && p.address && (
                  <span className="numeric block truncate text-micro text-ink-faint">
                    {shortAddress(p.address, 10, 6)}
                  </span>
                )}
              </span>
              <ArrowRight size={13} className="flex-none text-ink-muted" aria-hidden="true" />
            </button>
          </li>
        ))}
      </ul>
      <button type="button" className="btn w-full" onClick={onNew} data-gate-new>
        <UserPlus size={14} />
        New QOR ID
      </button>
      <button type="button" className="btn btn-ghost mt-2 w-full" onClick={onRestore}>
        Restore from a recovery phrase
      </button>
    </div>
  );
}

/** A QOR ID's PIN, which opens its wallet. A forgotten one is replaced by restoring from the recovery phrase. */
function PinStep({
  busy,
  profile,
  onOpen,
  onBack,
}: {
  busy: boolean;
  profile: ProfileView;
  onOpen: (pin: string) => void;
  onBack: () => void;
}) {
  const [pin, setPin] = useState('');
  const ready = validPin(pin) && !busy;
  return (
    <div data-gate-pin>
      <Header
        eyebrow="Who&apos;s playing?"
        title={profile.name ?? 'QOR ID without a name'}
        body={
          profile.has_pin
            ? 'Enter this QOR ID\u2019s PIN to open its wallet. Forgotten it? Restore this QOR ID from its recovery phrase.'
            : 'This QOR ID has no PIN on this PC. Restore it from its recovery phrase to open it.'
        }
      />
      {profile.wait_seconds > 0 && (
        <p className="mb-4 text-caption text-warn">Too many wrong PINs. Wait a little before trying again.</p>
      )}
      {profile.has_pin && (
        <div className="mb-5">
          <PinField value={pin} onChange={setPin} autoFocus onEnter={() => ready && onOpen(pin)} />
        </div>
      )}
      <div className="flex gap-2">
        <button type="button" className="btn btn-ghost" onClick={onBack}>
          <ArrowLeft size={13} />
          Back
        </button>
        {profile.has_pin && (
          <button
            type="button"
            className="btn btn-primary flex-1"
            disabled={!ready}
            onClick={() => onOpen(pin)}
            data-gate-open
          >
            {busy ? <Loader2 size={14} className="animate-spin" /> : <KeyRound size={14} />}
            Open
          </button>
        )}
      </div>
    </div>
  );
}

function Welcome({
  busy,
  onBegin,
  onRestore,
  onBack,
}: {
  busy: boolean;
  onBegin: () => void;
  onRestore: () => void;
  onBack?: () => void;
}) {
  return (
    <>
      <Header
        eyebrow="Demiurge"
        title={onBack ? 'A new QOR ID' : 'Welcome'}
        body={
          onBack
            ? 'A new QOR ID gets a wallet of its own, with a new recovery phrase and an empty balance. It shares nothing with the others on this PC.'
            : 'The launcher makes your keys and keeps them on this computer. There is nothing to type and nothing to remember. You can see your 24-word recovery phrase in Settings whenever you want to write down a backup.'
        }
      />

      <button type="button" className="btn btn-primary w-full" disabled={busy} onClick={onBegin}>
        {busy ? <Loader2 size={14} className="animate-spin" /> : <KeyRound size={14} />}
        Begin
      </button>
      <button type="button" className="btn btn-ghost mt-2 w-full" onClick={onRestore}>
        I already have a recovery phrase
      </button>
      {onBack && (
        <button type="button" className="btn btn-ghost mt-2 w-full" onClick={onBack} data-gate-back>
          <ArrowLeft size={13} />
          Back to Who&apos;s playing?
        </button>
      )}
    </>
  );
}

function UnlockStep({
  busy,
  hello,
  onUnlock,
  onRestore,
}: {
  busy: boolean;
  hello: boolean;
  onUnlock: () => void;
  onRestore: () => void;
}) {
  return (
    <>
      <Header
        eyebrow={hello ? 'One last time' : 'Your vault'}
        title={hello ? 'Windows Hello, one last time' : 'Open your vault'}
        body={
          hello
            ? 'Your vault is sealed with Windows Hello. Approve it once more and it moves to this computer’s keychain. After this, nothing is asked.'
            : 'Your vault did not open on its own. Try again, or restore it from your recovery phrase.'
        }
      />

      <button
        type="button"
        className="btn btn-primary w-full"
        disabled={busy}
        onClick={onUnlock}
        data-unlock
      >
        {busy ? (
          <Loader2 size={14} className="animate-spin" />
        ) : hello ? (
          <Fingerprint size={14} />
        ) : (
          <RotateCw size={14} />
        )}
        {hello ? 'Use Windows Hello' : 'Try again'}
      </button>
      {hello && (
        <p className="mt-3 text-caption text-ink-muted">
          If nothing appears, the Windows Hello prompt may be behind this window. Look for it in
          the taskbar.
        </p>
      )}
      <button type="button" className="btn btn-ghost mt-2 w-full" onClick={onRestore}>
        Restore from your recovery phrase
      </button>
    </>
  );
}

/**
 * A vault sealed with a passphrase before ADR-055. Its passphrase is typed once
 * more and it moves to the keychain; the old file is set aside, never deleted.
 */
function MoveStep({
  busy,
  onSubmit,
  onRestore,
}: {
  busy: boolean;
  onSubmit: (passphrase: string) => void;
  onRestore: () => void;
}) {
  const [value, setValue] = useState('');

  return (
    <>
      <Header
        eyebrow="One last time"
        title="Your old passphrase, one last time"
        body="This vault was sealed with a passphrase. Type it once more and it moves to this
              computer’s keychain. After this, nothing is asked."
      />

      <input
        className="field mb-5"
        type="password"
        placeholder="Your old passphrase"
        aria-label="Your old passphrase"
        autoFocus
        value={value}
        onChange={(e) => setValue(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && value && !busy && onSubmit(value)}
      />

      <button
        type="button"
        className="btn btn-primary w-full"
        disabled={!value || busy}
        onClick={() => onSubmit(value)}
      >
        {busy ? <Loader2 size={14} className="animate-spin" /> : <KeyRound size={14} />}
        Open and keep it open
      </button>
      <button type="button" className="btn btn-ghost mt-2 w-full" onClick={onRestore}>
        Forgot it? Restore from your recovery phrase
      </button>
    </>
  );
}

/**
 * Restoring from a phrase the person already holds. The host is asked which
 * account the phrase opens before anything changes, so the person can
 * recognise it; replacing a vault already here is approved again in a host
 * dialog, and that vault is set aside, never deleted.
 */
function RestoreStep({
  busy,
  replacing,
  checked,
  onCheck,
  onEdit,
  onContinue,
  onBack,
}: {
  busy: boolean;
  replacing: boolean;
  checked: { phrase: string; address: string } | null;
  onCheck: (phrase: string) => void;
  onEdit: () => void;
  onContinue: () => void;
  onBack: () => void;
}) {
  const [typed, setTyped] = useState(checked?.phrase ?? '');
  const count = typed.trim() ? typed.trim().split(/\s+/).length : 0;
  const plausible = [12, 15, 18, 21, 24].includes(count);

  return (
    <>
      <Header
        eyebrow="Restore"
        title="Enter your recovery phrase"
        body={
          replacing
            ? 'The vault on this device will be set aside, not deleted, and a new one made from this phrase.'
            : 'The words you wrote down, in order. Type them here only, never into a website.'
        }
      />

      <textarea
        className="field mb-2 resize-none"
        rows={4}
        aria-label="Recovery phrase"
        autoFocus
        autoComplete="off"
        spellCheck={false}
        value={typed}
        onChange={(e) => {
          setTyped(e.target.value);
          if (checked) onEdit();
        }}
      />
      <p className="mb-5 text-caption text-ink-muted">
        {count === 0 ? 'Separate the words with spaces.' : `${count} words`}
      </p>

      {checked && (
        <div className="mb-5 border border-edge bg-well p-3">
          <p className="text-caption text-ink-muted">This phrase opens the account</p>
          <p className="numeric selectable mt-1 break-all text-ui text-ink">{checked.address}</p>
        </div>
      )}

      <div className="flex gap-2">
        <button type="button" className="btn btn-ghost" onClick={onBack}>
          Back
        </button>
        {checked ? (
          <button
            type="button"
            className="btn btn-primary flex-1"
            disabled={busy}
            onClick={onContinue}
          >
            {busy ? <Loader2 size={14} className="animate-spin" /> : null}
            That is my account
            <ArrowRight size={13} />
          </button>
        ) : (
          <button
            type="button"
            className="btn btn-primary flex-1"
            disabled={!plausible || busy}
            onClick={() => onCheck(typed)}
          >
            {busy ? <Loader2 size={14} className="animate-spin" /> : null}
            Check the phrase
          </button>
        )}
      </div>
    </>
  );
}

/* ─────────────────────────────── fragments ─────────────────────────────── */

function Header({
  eyebrow,
  title,
  body,
}: {
  eyebrow: string;
  title: string;
  body?: string;
}) {
  return (
    <div className="mb-6">
      <p className="eyebrow mb-2 text-accent">{eyebrow}</p>
      <h1 className="heading mb-2 text-heading">{title}</h1>
      {body && <p className="text-body leading-relaxed text-ink-muted">{body}</p>}
    </div>
  );
}

function Centered({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex items-center justify-center gap-3 py-10 text-ink-muted">
      <Loader2 size={15} className="animate-spin" />
      <span className="text-body">{children}</span>
    </div>
  );
}
