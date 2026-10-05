/**
 * The awakening: a short tutorial told as a story, opened from the notification
 * that follows claiming a QOR ID.
 *
 * Its claims stay inside docs/economics/CGT.md: creators are paid when their
 * work is used, licensed or remixed; seeders for storing and serving; a pool
 * for creators and players is being designed and promises no amount. No price,
 * no returns, nothing about holding CGT (ADR-008).
 */

import { useCallback, useEffect, useRef, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { ArrowLeft, ArrowRight, Sparkles, X } from 'lucide-react';

import { wantsLessMotion } from '../../lib/a11y';
import './ceremony.css';
import { useQor } from '../../state/store';

const KEY = (qorId: string) => `qor.tutorial.done:${qorId}`;

/** Whether this QOR ID has finished the tutorial on this computer. */
export function tutorialDone(qorId: string): boolean {
  try {
    // Before ADR-075 a QOR ID was the name, '#' and the number 0001; a tutorial finished under that
    // form stays finished. (Joined, not written out, so it is not read as a colour literal.)
    const retired = [qorId, '0001'].join('#');
    return localStorage.getItem(KEY(qorId)) === '1' || localStorage.getItem(KEY(retired)) === '1';
  } catch {
    return false;
  }
}

interface Chapter {
  mark: string;
  eyebrow: string;
  title: (name: string) => string;
  body: (name: string) => string[];
}

const CHAPTERS: Chapter[] = [
  {
    mark: 'I',
    eyebrow: 'The signal',
    title: (name) => `Hello ${name}!`,
    body: (name) => [
      `Your signal has reached the Nexus. From this moment the name ${name} is yours, carried into every Demiurge world and anchored to a key only this launcher holds.`,
      'Few arrive this early. The city is still being dreamt into being — and you are one of the ones dreaming it.',
    ],
  },
  {
    mark: 'II',
    eyebrow: 'The gateway',
    title: () => 'What QOR is',
    body: () => [
      'QOR is your threshold: one launcher, one identity, one vault. Everything you make, trade or play here carries your name with it.',
      'No platform stands between you and the people who use your work. The keys are yours; this interface can ask them to sign, but it can never read them.',
    ],
  },
  {
    mark: 'III',
    eyebrow: 'The ledger beneath',
    title: () => 'The chain that remembers',
    body: () => [
      'Beneath QOR runs Demiurge, a living ledger kept by many independent validators who are paid to keep it honest.',
      'When you mint a creation, its fingerprint is written there for good. The proof that you made it cannot be edited, hidden or taken back — by anyone.',
    ],
  },
  {
    mark: 'IV',
    eyebrow: 'The current',
    title: () => 'Creation that pays',
    body: () => [
      'The currency here is CGT, and it exists to be spent — it flows toward whatever people make and use.',
      'When someone uses, licenses or remixes your work, you are paid. Those who store and serve the network’s files are paid for that work. And a pool for creators and players is being shaped, for the ones who bring the worlds to life.',
    ],
  },
  {
    mark: 'V',
    eyebrow: 'The city of makers',
    title: () => 'Nobody builds alone',
    body: () => [
      'Collaborate, commission, remix and trade. Every exchange is a thread between creators, and the chain keeps each promise in plain sight: who made what, who may use it, and on what terms.',
      'Social is forming now — rooms, messages and presence under the name you just took.',
    ],
  },
  {
    mark: 'VI',
    eyebrow: 'Your first steps',
    title: (name) => `Go forth, ${name}`,
    body: () => [
      'Open Projects to save your work and mint it into the chain. Visit Inventory to hold and trade what you have made. The Nexus shows your path from here.',
      'The dream is unfinished on purpose. The rest of it is yours to build.',
    ],
  },
];

export function Tutorial({
  name,
  qorId,
  onClose,
}: {
  name: string;
  qorId: string;
  /** `finished`: the last chapter was reached and closed. */
  onClose: (finished: boolean) => void;
}) {
  const a11y = useQor((s) => s.a11y);
  const still = wantsLessMotion(a11y);
  const [at, setAt] = useState(0);
  const dialog = useRef<HTMLDivElement>(null);
  const chapter = CHAPTERS[at]!;
  const last = at === CHAPTERS.length - 1;

  const finish = useCallback(() => {
    try {
      localStorage.setItem(KEY(qorId), '1');
    } catch {
      /* the tutorial offers itself again next time; nothing else depends on it */
    }
    onClose(true);
  }, [onClose, qorId]);

  useEffect(() => {
    dialog.current?.focus();
  }, []);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose(false);
      else if (e.key === 'ArrowRight' && !last) setAt((i) => i + 1);
      else if (e.key === 'ArrowLeft' && at > 0) setAt((i) => i - 1);
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [at, last, onClose]);

  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      exit={{ opacity: 0 }}
      transition={{ duration: still ? 0.1 : 0.45 }}
      className="fixed inset-0 z-[60] flex items-center justify-center bg-void/80 backdrop-blur-md"
    >
      {!still && <Starfield />}

      <div
        ref={dialog}
        tabIndex={-1}
        role="dialog"
        aria-modal="true"
        aria-labelledby="tutorial-title"
        data-tutorial
        className="neochrome relative w-[560px] max-w-[calc(100vw-48px)] rounded-2xl p-10 outline-none"
      >
        <button
          type="button"
          aria-label="Close the tutorial"
          onClick={() => onClose(false)}
          className="absolute right-4 top-4 text-ink-muted transition-colors hover:text-ink"
        >
          <X size={16} />
        </button>

        <AnimatePresence mode="wait">
          <motion.div
            key={at}
            initial={{ opacity: 0, y: still ? 0 : 14, filter: still ? 'none' : 'blur(6px)' }}
            animate={{ opacity: 1, y: 0, filter: 'blur(0px)' }}
            exit={{ opacity: 0, y: still ? 0 : -10, filter: still ? 'none' : 'blur(4px)' }}
            transition={{ duration: still ? 0.1 : 0.5, ease: [0.16, 1, 0.3, 1] }}
          >
            <p className="numeric mb-6 text-caption tracking-eyebrow text-accent">
              {chapter.mark} <span className="text-ink-faint">/ VI</span>
            </p>
            <p className="eyebrow mb-2 text-accent-bright">{chapter.eyebrow}</p>
            <h2 id="tutorial-title" className="heading mb-5 text-display text-ink">
              {chapter.title(name)}
            </h2>
            {chapter.body(name).map((paragraph) => (
              <p key={paragraph} className="mb-3 text-body leading-relaxed text-ink-muted">
                {paragraph}
              </p>
            ))}
          </motion.div>
        </AnimatePresence>

        <div className="mt-8 flex items-center gap-3">
          <div className="flex flex-1 gap-1.5" aria-hidden="true">
            {CHAPTERS.map((c, i) => (
              <span
                key={c.mark}
                className={`h-1 flex-1 rounded-full transition-colors duration-300 ${
                  i <= at ? 'bg-accent' : 'bg-edge'
                }`}
              />
            ))}
          </div>
          {at > 0 && (
            <button type="button" className="btn btn-ghost" onClick={() => setAt((i) => i - 1)}>
              <ArrowLeft size={13} />
              Back
            </button>
          )}
          {last ? (
            <button type="button" className="btn btn-primary" onClick={finish}>
              <Sparkles size={13} />
              Enter the Nexus
            </button>
          ) : (
            <button type="button" className="btn btn-primary" onClick={() => setAt((i) => i + 1)}>
              Continue
              <ArrowRight size={13} />
            </button>
          )}
        </div>
      </div>
    </motion.div>
  );
}

/** Slow motes of light behind the tutorial. Decorative; hidden from assistive tech. */
function Starfield() {
  const motes = Array.from({ length: 36 }, (_, i) => {
    const seed = (i * 9301 + 49297) % 233280;
    const r = seed / 233280;
    return {
      left: `${(r * 997) % 100}%`,
      top: `${(r * 613 + i * 7) % 100}%`,
      size: 1 + ((i * 7) % 3),
      delay: (i % 12) * 0.4,
      duration: 6 + (i % 5),
    };
  });
  return (
    <div className="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
      {motes.map((m, i) => (
        <motion.span
          key={i}
          className="absolute rounded-full bg-accent-bright"
          style={{ left: m.left, top: m.top, width: m.size, height: m.size }}
          animate={{ opacity: [0, 0.9, 0], y: [0, -40] }}
          transition={{ duration: m.duration, delay: m.delay, repeat: Infinity, ease: 'easeInOut' }}
        />
      ))}
    </div>
  );
}
