/**
 * Room levels and account standing (ADR-093).
 *
 * A room's levels, highest first: Owner, Admin, Mod, VIP, Member. An Elite account acts as an Admin in every room it is
 * in; godmode stands above every room's Owner and alone makes accounts Elite. Each level acts only on those below it.
 * Social decides every one of these; the interface only offers what Social would allow, and shows the badges.
 */

import { Crown, Gem, Shield, ShieldCheck, Sparkles, Star } from 'lucide-react';

import type { Profile, RoomSummary } from './model';

export type Level = 'owner' | 'admin' | 'moderator' | 'vip' | 'member';
export type Standing = 'godmode' | 'elite' | null | undefined;

export const LEVELS: Level[] = ['owner', 'admin', 'moderator', 'vip', 'member'];

export const LEVEL_NAMES: Record<Level, string> = {
  owner: 'Owner',
  admin: 'Admin',
  moderator: 'Mod',
  vip: 'VIP',
  member: 'Member',
};

export function rankOf(level: string | undefined | null): number {
  return { owner: 5, admin: 4, moderator: 3, vip: 2 }[level ?? ''] ?? 1;
}

/** Where someone stands in a room: their level there, raised to an Admin's for Elite and above everyone for godmode. */
export function standingRank(standing: Standing, level: string | undefined | null): number {
  if (standing === 'godmode') return 6;
  if (standing === 'elite') return Math.max(rankOf(level), 4);
  return rankOf(level);
}

/** The reader's own standing in a room they are in. */
export function myRank(me: Profile | null, room: RoomSummary | undefined): number {
  return standingRank(me?.status, room?.role);
}

/** The levels `rank` may give in a room: those below it, and the room itself for its Owner or godmode (never World). */
export function givable(rank: number, myLevel: string | undefined, room: RoomSummary): Level[] {
  if (rank < 4 || room.kind === 'direct') return [];
  return LEVELS.filter((l) => (l === 'owner' ? room.kind !== 'world' && (myLevel === 'owner' || rank >= 6) : rankOf(l) < rank));
}

const BADGE: Record<string, { label: string; className: string; Icon: typeof Crown }> = {
  godmode: { label: 'godmode', className: 'social-badge-godmode', Icon: Sparkles },
  elite: { label: 'Elite', className: 'social-badge-elite', Icon: Gem },
  owner: { label: 'Owner', className: 'social-badge-owner', Icon: Crown },
  admin: { label: 'Admin', className: 'social-badge-admin', Icon: ShieldCheck },
  moderator: { label: 'Mod', className: 'social-badge-mod', Icon: Shield },
  vip: { label: 'VIP', className: 'social-badge-vip', Icon: Star },
};

/** The badges beside a name: the account's standing, then the room level, if either is above the ordinary. */
export function Badges({ standing, level }: { standing: Standing; level?: string | null }) {
  const shown = [standing, level && level !== 'member' ? level : null].filter((b): b is string => !!b && b in BADGE);
  if (shown.length === 0) return null;
  return (
    <>
      {shown.map((b) => {
        const { label, className, Icon } = BADGE[b]!;
        return (
          <span key={b} className={`social-badge ${className}`} data-badge={b} title={label}>
            <Icon size={9} aria-hidden="true" />
            {label}
          </span>
        );
      })}
    </>
  );
}
