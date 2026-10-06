import { useEffect, useState } from 'react';
import { useQor } from '../../state/store';
import { identity } from '../../lib/ipc';
import { wantsLessMotion } from '../../lib/a11y';

/**
 * An avatar (ADR-079): the account's picture or GIF in a ring, with its level on the ring (ADR-078), or, with no
 * picture or none available, the first letter of the name on a colour that name always gets. A GIF holds its first frame
 * when "reduce motion" is on.
 */

// Pairs from the theme's own tokens: a letter's background and ink. A name always lands on the same pair.
const LETTER_TONES = [
  // Only pairs that read in every theme (`check-readability.mjs` measures them): the theme's ink on its three dark
  // grounds, and its darkest ground on the accent. A dim accent or the counter under ink fell below AA in some themes.
  'bg-accent text-void',
  'bg-raised text-ink',
  'bg-well text-ink',
  'bg-surface text-ink',
] as const;

function toneFor(name: string): string {
  let hash = 0;
  for (const ch of name.toLowerCase()) hash = (hash * 31 + ch.codePointAt(0)!) >>> 0;
  return LETTER_TONES[hash % LETTER_TONES.length]!;
}

/**
 * The ring a level has unlocked (ADR-078; glow allowed by ADR-080): a plain edge at level 0, the accent with a soft
 * glow from Ember (level 1), the bright accent with a wider glow from Meridian (level 6). A static glow, not motion.
 */
function ringFor(level: number | null | undefined): string {
  if (typeof level !== 'number' || level < 1) return 'border-edge';
  if (level < 6) return 'border-accent shadow-[0_0_10px_var(--accent)]';
  return 'border-accent-bright shadow-[0_0_16px_var(--accent-bright)]';
}

export function initialOf(name: string | undefined | null): string {
  const first = (name ?? '').trim().charAt(0);
  return first ? first.toUpperCase() : '?';
}

/** The account's own avatar, from this machine's copy (so it shows offline), refreshed when the session changes. */
export function useOwnAvatar(): string | null {
  const session = useQor((s) => s.session);
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    // The copy first, at once; then a refresh from QOR ID, which keeps the copy if it cannot be reached.
    identity.avatar(false).then((s) => live && setSrc(s)).catch(() => undefined);
    if (session) identity.avatar(true).then((s) => live && setSrc(s)).catch(() => undefined);
    return () => {
      live = false;
    };
  }, [session, session?.avatar_url]);
  return session ? src : null;
}

/** A still image of a GIF's first frame, for "reduce motion". */
function useStill(src: string | null, still: boolean): string | null {
  const [frame, setFrame] = useState<string | null>(null);
  useEffect(() => {
    if (!src || !still || !src.startsWith('data:image/gif')) return;
    let live = true;
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement('canvas');
      canvas.width = img.naturalWidth;
      canvas.height = img.naturalHeight;
      canvas.getContext('2d')?.drawImage(img, 0, 0);
      if (live) setFrame(canvas.toDataURL('image/png'));
    };
    img.src = src;
    return () => {
      live = false;
    };
  }, [src, still]);
  return src && still && src.startsWith('data:image/gif') ? frame : src;
}

export function Avatar({
  name,
  src,
  size,
  level,
}: {
  name: string | undefined | null;
  src: string | null;
  size: number;
  level?: number | null;
}) {
  const a11y = useQor((s) => s.a11y);
  const shown = useStill(src, wantsLessMotion(a11y));
  const [failed, setFailed] = useState(false);
  useEffect(() => setFailed(false), [src]);
  const bubble = Math.max(14, Math.round(size * 0.36));
  return (
    <span className="relative inline-flex flex-none" style={{ width: size, height: size }}>
      <span
        className={`flex h-full w-full items-center justify-center overflow-hidden rounded-full border-2 p-px ${ringFor(level)}`}
        aria-hidden="true"
      >
        {shown && !failed ? (
          <img src={shown} alt="" className="h-full w-full rounded-full object-cover" onError={() => setFailed(true)} />
        ) : (
          <span
            className={`flex h-full w-full items-center justify-center rounded-full font-semibold ${toneFor(name ?? '')}`}
            style={{ fontSize: Math.round(size * 0.42) }}
          >
            {initialOf(name)}
          </span>
        )}
      </span>
      {typeof level === 'number' && (
        <span
          aria-label={`Level ${level}`}
          className="numeric absolute flex items-center justify-center rounded-full border-2 border-void bg-accent px-1 font-semibold leading-none text-void"
          style={{ minWidth: bubble, height: bubble, right: -2, bottom: -2, fontSize: Math.round(bubble * 0.6) }}
        >
          {level}
        </span>
      )}
    </span>
  );
}
