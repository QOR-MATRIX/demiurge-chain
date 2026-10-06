import { useEffect, useState } from 'react';
import { identity, type Progress } from '../../lib/ipc';

/**
 * The signed-in account's level and XP, read from QOR ID (ADR-078), which is the only place XP is granted. Read when
 * the session changes, when the window regains focus, and once a minute; nothing is shown while signed out or unread.
 */
export function useProgress(signedIn: boolean): Progress | null {
  const [progress, setProgress] = useState<Progress | null>(null);
  useEffect(() => {
    if (!signedIn) return;
    let live = true;
    const read = () => {
      identity
        .progress()
        .then((p) => live && setProgress(p))
        .catch(() => undefined);
    };
    read();
    const timer = window.setInterval(read, 60_000);
    window.addEventListener('focus', read);
    return () => {
      live = false;
      window.clearInterval(timer);
      window.removeEventListener('focus', read);
    };
  }, [signedIn]);
  return signedIn ? progress : null;
}

/** The small level bubble on the avatar. */
export function LevelBubble({ progress }: { progress: Progress | null }) {
  if (!progress) return null;
  return (
    <span
      aria-label={`Level ${progress.level}`}
      className="numeric absolute -bottom-1 -right-1 flex h-4 min-w-4 items-center justify-center rounded-full border border-void bg-accent px-1 text-micro font-semibold leading-none text-void"
    >
      {progress.level}
    </span>
  );
}

/** XP towards the next level, and what it unlocks. */
export function XpBar({ progress }: { progress: Progress | null }) {
  if (!progress) return null;
  const span = Math.max(1, progress.next_level_xp - progress.level_xp);
  const into = Math.min(span, Math.max(0, progress.xp - progress.level_xp));
  const percent = Math.round((into / span) * 100);
  const hint = progress.next_unlock ? `Next: ${progress.next_unlock}` : `Level ${progress.level + 1}`;
  return (
    <span className="mt-1 block" title={`${progress.xp} XP · ${progress.next_level_xp - progress.xp} to level ${progress.level + 1}`}>
      <span
        role="progressbar"
        aria-label={`Experience towards level ${progress.level + 1}`}
        aria-valuemin={0}
        aria-valuemax={span}
        aria-valuenow={into}
        className="block h-1 w-full overflow-hidden rounded-full bg-edge"
      >
        <span className="block h-full bg-accent" style={{ width: `${percent}%` }} />
      </span>
      <span className="numeric mt-0.5 block truncate text-micro text-ink-muted">
        Lv {progress.level} · {into}/{span} XP · {hint}
      </span>
    </span>
  );
}
