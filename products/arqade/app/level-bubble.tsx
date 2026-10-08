"use client";
// The player's avatar and QOR ID level on the sidebar (ADR-078, ADR-079), read from QOR ID through this site's server.
import { Fingerprint } from 'lucide-react';
import { useEffect, useState } from 'react';
import { PlayerAvatar } from '@/components/player-avatar';

type Me = { signedIn: boolean; username?: string; avatarUrl?: string | null; progress?: { level: number } | null };

export function IdentityAvatar() {
  const [me, setMe] = useState<Me | null>(null);
  useEffect(() => {
    let live = true;
    const read = () =>
      fetch('/api/auth/me', { cache: 'no-store' })
        .then((r) => r.json() as Promise<Me>)
        .then((m) => live && setMe(m))
        .catch(() => undefined);
    read();
    const timer = setInterval(read, 60_000);
    return () => {
      live = false;
      clearInterval(timer);
    };
  }, []);
  if (!me?.signedIn || !me.username) return <span className="avatar"><Fingerprint /></span>;
  return <PlayerAvatar username={me.username} avatarUrl={me.avatarUrl} level={me.progress?.level ?? null} />;
}
