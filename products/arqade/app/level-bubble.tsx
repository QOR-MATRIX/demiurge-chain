"use client";
// The player's QOR ID level on the sidebar avatar (ADR-078), read from QOR ID through this site's server.
import { useEffect, useState } from 'react';

type Me = { signedIn: boolean; progress?: { level: number } | null };

export function LevelBubble() {
  const [level, setLevel] = useState<number | null>(null);
  useEffect(() => {
    let live = true;
    const read = () =>
      fetch('/api/auth/me', { cache: 'no-store' })
        .then((r) => r.json() as Promise<Me>)
        .then((me) => live && setLevel(me.signedIn && me.progress ? me.progress.level : null))
        .catch(() => undefined);
    read();
    const timer = setInterval(read, 60_000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, []);
  if (level === null) return null;
  return <span className="level-bubble" aria-label={`Level ${level}`}>{level}</span>;
}
