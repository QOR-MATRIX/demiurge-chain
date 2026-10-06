/**
 * The navigation rail.
 *
 * Icon-first with labels, because a launcher is returned to daily and its
 * navigation should become muscle memory rather than something you read.
 *
 * The Nexus sits at the top and is where you land: it is the map, and everything
 * else on this rail is also reachable from it. The rail exists for the second
 * visit onward, when you already know where you are going.
 */

import {
  Boxes,
  CircleUser,
  FolderGit2,
  LayoutGrid,
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
import { LevelBubble, XpBar, useProgress } from './Level';

interface Item {
  id: SurfaceId;
  label: string;
  icon: typeof LayoutGrid;
  hint: string;
}

const ITEMS: Item[] = [
  { id: 'nexus', label: 'Nexus', icon: LayoutGrid, hint: 'Every system, one place' },
  { id: 'vault', label: 'Vault', icon: Wallet, hint: 'Hold, send and receive CGT' },
  { id: 'inventory', label: 'Inventory', icon: Boxes, hint: 'DRC-369 assets you own' },
  { id: 'market', label: 'Market', icon: Store, hint: 'What is for sale on chain, as your node reads it' },
  { id: 'projects', label: 'Projects', icon: FolderGit2, hint: 'Version your work, on this machine' },
  { id: 'library', label: 'Library', icon: Library, hint: 'Games and applications' },
  { id: 'social', label: 'Social', icon: MessagesSquare, hint: 'Rooms, messages and presence' },
  { id: 'mesh', label: 'Mesh', icon: Share2, hint: 'Peer-to-peer distribution' },
  { id: 'chain', label: 'Chain', icon: Radio, hint: 'Node, network and validators' },
  { id: 'gates', label: 'Gates', icon: ListChecks, hint: 'Release-gate progress from docs/GATES.toml' },
  { id: 'settings', label: 'Settings', icon: Settings, hint: 'Themes, endpoints, security' },
];

export function Rail() {
  const surface = useQor((s) => s.surface);
  const go = useQor((s) => s.go);
  const session = useQor((s) => s.session);
  const account = useQor(selectActiveAccount);
  const balance = useQor(selectActiveBalance);
  const standing = useAscent().standing;
  const progress = useProgress(Boolean(session));

  return (
    <nav
      aria-label="Primary"
      className="flex w-52 flex-none flex-col border-r border-edge bg-void"
    >
      <ul className="flex flex-col gap-0.5 p-3">
        {ITEMS.map(({ id, label, icon: Icon, hint }) => {
          const active = surface === id;
          return (
            <li key={id}>
              <button
                type="button"
                onClick={() => go(id)}
                title={hint}
                aria-current={active ? 'page' : undefined}
                className={`group relative flex w-full items-center gap-3 px-3 py-2.5 text-left transition-colors duration-150 ${
                  active
                    ? 'bg-surface text-ink'
                    : 'text-ink-muted hover:bg-raised hover:text-ink-body'
                }`}
              >
                <span
                  aria-hidden="true"
                  className={`absolute left-0 top-0 h-full w-[2px] transition-opacity duration-150 ${
                    active ? 'bg-accent opacity-100' : 'opacity-0'
                  }`}
                />
                <Icon
                  size={16}
                  strokeWidth={1.75}
                  className={active ? 'text-accent' : 'text-current'}
                />
                <span className="heading text-caption tracking-label">{label}</span>
              </button>
            </li>
          );
        })}
      </ul>

      <div className="mt-auto border-t border-edge p-3">
        <Surface
          as="button"
          interactive
          cut
          onClick={() => go('vault')}
          className="flex w-full items-center gap-3 p-3 text-left"
        >
          <span className="relative flex-none">
            <CircleUser size={22} strokeWidth={1.5} className="text-accent" />
            <LevelBubble progress={progress} />
          </span>
          <span className="min-w-0 flex-1">
            <span className="block truncate text-ui font-semibold text-ink">
              {session?.qor_id ?? 'Not signed in'}
            </span>
            <span className="numeric block truncate text-micro text-ink-muted">
              {balance?.display ?? (account ? shortAddress(account.address) : standing)}
            </span>
            <XpBar progress={progress} />
          </span>
        </Surface>
      </div>
    </nav>
  );
}
