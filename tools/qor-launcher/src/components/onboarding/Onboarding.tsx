/**
 * Onboarding: the two notifications that bring a new person in.
 *
 * 1. Without a QOR ID, a bubble offers one. Clicking it opens a small card that
 *    checks the name as it is typed and claims it with the vault's key.
 * 2. With a QOR ID and the tutorial not yet taken, a notification with a
 *    neochrome aura invites them in. Clicking it opens the tutorial.
 *
 * Neither blocks anything: the launcher works around them, and each can wait.
 */

import { useEffect, useRef, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { Check, Loader2, Sparkles, X } from 'lucide-react';

import { explain, identity } from '../../lib/ipc';
import { useQor } from '../../state/store';
import { AwakenNotice } from '../../qfx/ceremony/Awaken';
import { Tutorial, tutorialDone } from '../../qfx/ceremony/Tutorial';

// QOR ID's rule exactly (ADR-075): letters, digits and underscore. A hyphen was allowed here and refused there.
const NAME = /^[A-Za-z0-9_]{3,20}$/;

export function Onboarding() {
  const session = useQor((s) => s.session);
  const [tutorialOpen, setTutorialOpen] = useState(false);
  const [done, setDone] = useState(() => (session ? tutorialDone(session.qor_id) : false));

  useEffect(() => {
    setDone(session ? tutorialDone(session.qor_id) : false);
  }, [session]);

  return (
    <>
      <div className="pointer-events-none absolute bottom-6 right-6 z-40 flex flex-col items-end gap-3">
        <AnimatePresence>
          {!session && <ClaimBubble key="claim" />}
          {session && !done && !tutorialOpen && (
            <AwakenNotice key="awaken" name={session.username} onOpen={() => setTutorialOpen(true)} />
          )}
        </AnimatePresence>
      </div>

      <AnimatePresence>
        {tutorialOpen && session && (
          <Tutorial
            name={session.username}
            onClose={(finished) => {
              setTutorialOpen(false);
              if (finished) setDone(true);
            }}
            qorId={session.qor_id}
          />
        )}
      </AnimatePresence>
    </>
  );
}

/* ────────────────────────────── the name ─────────────────────────────── */

type Availability = 'idle' | 'invalid' | 'checking' | 'free' | 'taken' | 'offline';

function ClaimBubble() {
  const [open, setOpen] = useState(false);

  return (
    <motion.div
      initial={{ opacity: 0, y: 16, scale: 0.96 }}
      animate={{ opacity: 1, y: 0, scale: 1 }}
      exit={{ opacity: 0, y: 12, scale: 0.96 }}
      transition={{ duration: 0.3, ease: [0.16, 1, 0.3, 1] }}
      className="pointer-events-auto"
    >
      {open ? (
        <ClaimCard onClose={() => setOpen(false)} />
      ) : (
        <button
          type="button"
          onClick={() => setOpen(true)}
          data-claim-bubble
          className="glass-solid flex items-center gap-3 rounded-full border border-accent-dim py-2.5 pl-3 pr-5 text-left transition-colors hover:border-accent"
        >
          <span className="flex h-8 w-8 items-center justify-center rounded-full bg-accent/15 text-accent">
            <Sparkles size={15} />
          </span>
          <span>
            <span className="block text-ui font-semibold text-ink">Choose your QOR ID</span>
            <span className="block text-caption text-ink-muted">Your name across every world</span>
          </span>
        </button>
      )}
    </motion.div>
  );
}

function ClaimCard({ onClose }: { onClose: () => void }) {
  const accounts = useQor((s) => s.accounts);
  const setSession = useQor((s) => s.setSession);

  const [name, setName] = useState('');
  const [availability, setAvailability] = useState<Availability>('idle');
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const asked = useRef(0);

  // Checked as it is typed: the shape at once, the service after a short pause,
  // and only the answer to the latest name is kept.
  useEffect(() => {
    setProblem(null);
    if (!name) return setAvailability('idle');
    if (!NAME.test(name)) return setAvailability('invalid');
    setAvailability('checking');
    const ticket = ++asked.current;
    const id = setTimeout(async () => {
      try {
        const free = await identity.usernameAvailable(name);
        if (ticket === asked.current) setAvailability(free ? 'free' : 'taken');
      } catch {
        if (ticket === asked.current) setAvailability('offline');
      }
    }, 350);
    return () => clearTimeout(id);
  }, [name]);

  const claim = async () => {
    setBusy(true);
    setProblem(null);
    try {
      // Opening a sign-in first reopens the permission to answer QOR ID's
      // challenges with the vault's key, so no dialog interrupts the claim.
      const arrival = await identity.signIn();
      if (arrival.session) {
        setSession(arrival.session); // this key already has a QOR ID
        return;
      }
      if (!arrival.needs_name) {
        throw new Error(arrival.sign_in_problem ?? 'QOR ID could not be reached.');
      }
      const address = accounts[0]?.address;
      if (!address) throw new Error('No account is available to claim a name with.');
      setSession(await identity.registerWithKey(address, name));
    } catch (e) {
      setProblem(explain(e));
    } finally {
      setBusy(false);
    }
  };

  const status: Record<Availability, { text: string; tone: string }> = {
    idle: { text: '3 to 20 letters, numbers or _. It is your name everywhere: no number is added.', tone: 'text-ink-muted' },
    invalid: { text: 'Only letters, numbers and _, from 3 to 20 of them.', tone: 'text-bad' },
    checking: { text: 'Checking…', tone: 'text-ink-muted' },
    free: { text: `${name} is yours to take.`, tone: 'text-ok' },
    taken: { text: `${name} is taken. Try another.`, tone: 'text-bad' },
    offline: { text: 'QOR ID is not reachable right now.', tone: 'text-bad' },
  };

  return (
    <div
      role="dialog"
      aria-label="Choose your QOR ID"
      className="glass-solid cut w-[340px] border border-accent-dim p-5"
      data-claim-card
    >
      <div className="mb-4 flex items-start justify-between gap-3">
        <div>
          <p className="eyebrow mb-1 text-accent">QOR ID</p>
          <p className="heading text-body">Choose your name</p>
        </div>
        <button
          type="button"
          aria-label="Close"
          onClick={onClose}
          className="text-ink-muted transition-colors hover:text-ink"
        >
          <X size={15} />
        </button>
      </div>

      <input
        className={`field mb-2 ${availability === 'taken' || availability === 'invalid' ? 'field-invalid' : ''}`}
        placeholder="architect"
        aria-label="QOR ID name"
        autoFocus
        autoComplete="off"
        spellCheck={false}
        value={name}
        onChange={(e) => setName(e.target.value.trim())}
        onKeyDown={(e) => e.key === 'Enter' && availability === 'free' && !busy && void claim()}
      />
      <p className={`mb-4 min-h-[1.25rem] text-caption ${status[availability].tone}`} aria-live="polite">
        {status[availability].text}
      </p>

      <button
        type="button"
        className="btn btn-primary w-full"
        disabled={availability !== 'free' || busy}
        onClick={() => void claim()}
      >
        {busy ? <Loader2 size={14} className="animate-spin" /> : <Check size={14} />}
        Claim {name || 'my name'}
      </button>
      {problem && (
        <p role="alert" className="mt-3 text-caption text-bad">
          {problem}
        </p>
      )}
    </div>
  );
}
