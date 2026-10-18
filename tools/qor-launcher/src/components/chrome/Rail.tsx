/**
 * The navigation rail.
 *
 * Icon-first with labels, because a launcher is returned to daily and its
 * navigation should become muscle memory rather than something you read.
 *
 * The Nexus sits at the top and is where you land: it is the map, and everything
 * else on this rail is also reachable from it. The rail exists for the second
 * visit onward, when you already know where you are going.
 *
 * Below the Nexus, destinations are grouped by what a person is doing: playing,
 * owning, creating, connecting, and the network itself. The current one is marked
 * by one lit bar (`.nav-glider`) that glides from item to item.
 */

import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useUnread } from '../../social/link';
import {
  Boxes,
  FolderGit2,
  Gamepad2,
  LayoutGrid,
  Shapes,
  ListChecks,
  MessagesSquare,
  Radio,
  Settings,
  Share2,
  Library,
  Store,
  Wallet,
} from 'lucide-react';

import { shortAddress } from '../../lib/ipc';
import {
  selectActiveAccount,
  selectActiveBalance,
  useAscent,
  useQor,
  type Surface as SurfaceId,
} from '../../state/store';
import { Surface } from '../ui/Surface';
import { XpBar, useProgress } from './Level';
import { Avatar, useOwnAvatar } from './Avatar';

interface Item {
  id: SurfaceId;
  label: string;
  icon: typeof LayoutGrid;
  hint: string;
}

/** The rail's groups, in order: a group with no title is the Nexus, on its own at the top. */
const GROUPS: { title: string | null; items: Item[] }[] = [
  { title: null, items: [{ id: 'nexus', label: 'Nexus', icon: LayoutGrid, hint: 'Every system, one place' }] },
  {
    title: 'Play',
    items: [
      { id: 'library', label: 'Library', icon: Library, hint: 'Your games, videos, music, pictures and documents' },
      { id: 'arqade', label: 'ARQADE', icon: Gamepad2, hint: 'Games and collectibles on the Demiurge devnet' },
      { id: 'qq', label: 'QQ', icon: Shapes, hint: 'The QOR Engine: build and play a game here' },
    ],
  },
  {
    title: 'Own',
    items: [
      { id: 'vault', label: 'Vault', icon: Wallet, hint: 'Hold, send and receive CGT' },
      { id: 'inventory', label: 'Inventory', icon: Boxes, hint: 'DRC-369 assets you own' },
      { id: 'market', label: 'Market', icon: Store, hint: 'What is for sale on chain, as your node reads it' },
    ],
  },
  {
    title: 'Create',
    items: [
      { id: 'projects', label: 'Projects', icon: FolderGit2, hint: 'Version your work, on this machine' },
      { id: 'mesh', label: 'Mesh', icon: Share2, hint: 'Seed published work for others, and be paid for it' },
    ],
  },
  {
    title: 'Connect',
    items: [{ id: 'social', label: 'Social', icon: MessagesSquare, hint: 'Rooms, messages, presence and profiles' }],
  },
  {
    title: 'Network',
    items: [
      { id: 'chain', label: 'Chain', icon: Radio, hint: 'Node, network and validators' },
      { id: 'gates', label: 'Gates', icon: ListChecks, hint: 'Release-gate progress from docs/GATES.toml' },
      { id: 'settings', label: 'Settings', icon: Settings, hint: 'Themes, endpoints, security' },
    ],
  },
];

export function Rail() {
  const unread = useUnread();
  const surface = useQor((s) => s.surface);
  const go = useQor((s) => s.go);
  const session = useQor((s) => s.session);
  const account = useQor(selectActiveAccount);
  const balance = useQor(selectActiveBalance);
  const standing = useAscent().standing;
  const progress = useProgress(Boolean(session));
  const avatar = useOwnAvatar();
  const list = useRef<HTMLDivElement>(null);
  const [glider, setGlider] = useState<{ top: number; height: number } | null>(null);

  // The lit bar sits behind the current item, measured from the item itself, so it follows any change of density or
  // scale; it glides because only its transform and height change.
  useLayoutEffect(() => {
    const place = () => {
      const current = list.current?.querySelector<HTMLElement>('[aria-current="page"]');
      setGlider(current ? { top: current.offsetTop, height: current.offsetHeight } : null);
    };
    place();
    window.addEventListener('resize', place);
    return () => window.removeEventListener('resize', place);
  }, [surface]);

  // The backdrops grow richer with the level (QFX), and read it from here. The
  // rail is mounted whenever the launcher is in use, so this is the one writer.
  const level = progress?.level;
  useEffect(() => {
    const root = document.documentElement;
    if (level == null) root.removeAttribute('data-level');
    else root.setAttribute('data-level', String(level));
  }, [level]);

  return (
    <nav aria-label="Primary" className="nav flex w-60 flex-none flex-col">
      <div ref={list} className="relative min-h-0 flex-1 overflow-y-auto px-3 pb-3 pt-2">
        <span
          aria-hidden="true"
          className="nav-glider"
          style={
            glider
              ? { transform: `translateY(${glider.top}px)`, height: glider.height, opacity: 1 }
              : { opacity: 0 }
          }
        />
        {GROUPS.map((group) => (
          <div key={group.title ?? 'home'} role="group" aria-label={group.title ?? undefined}>
            {group.title && <p className="nav-group">{group.title}</p>}
            <ul className="flex flex-col gap-0.5">
              {group.items.map(({ id, label, icon: Icon, hint }) => {
                const active = surface === id;
                return (
                  <li key={id}>
                    <button
                      type="button"
                      onClick={() => go(id)}
                      title={hint}
                      aria-current={active ? 'page' : undefined}
                      className="nav-item"
                    >
                      <span className="nav-icon" aria-hidden="true">
                        <Icon size={16} strokeWidth={1.9} />
                      </span>
                      <span>{label}</span>
                      {id === 'social' && unread > 0 && (
                        <span className="numeric nav-badge" aria-label={`${unread} unread`} data-social-unread>
                          {unread > 99 ? '99+' : unread}
                        </span>
                      )}
                    </button>
                  </li>
                );
              })}
            </ul>
          </div>
        ))}
      </div>

      <div className="border-t border-edge-soft p-3">
        <Surface
          as="button"
          interactive
          cut
          onClick={() => go('vault')}
          className="flex w-full items-center gap-3 p-3 text-left"
        >
          <Avatar name={session?.username} src={avatar} size={34} level={progress?.level ?? null} />
          <span className="min-w-0 flex-1">
            <span className="block truncate text-ui font-semibold text-ink">
              {session?.qor_id ?? 'Not signed in'}
            </span>
            <span className="numeric block truncate text-micro text-ink-muted">
              {balance?.display ?? (account ? shortAddress(account.address) : standing)}
            </span>
            {session && account && session.address !== account.address && (
              <span className="block truncate text-micro text-warn" data-rail-mismatch>
                Not {session.qor_id}&apos;s wallet
              </span>
            )}
            <XpBar progress={progress} />
          </span>
        </Surface>
      </div>
    </nav>
  );
}
