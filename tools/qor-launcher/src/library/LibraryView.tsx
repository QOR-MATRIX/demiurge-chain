/**
 * The Library (ADR-094; docs/blueprints/library.md section 3): everything a person has, in a SteamOS-like view.
 *
 * A hero for what was last opened, shelves of large tiles by kind, a folders view, and search one key away (`/`). Arrow
 * keys move a focus ring from tile to tile and shelf to shelf, Enter opens, Escape goes back; a game controller does the
 * same with its D-pad or left stick, A and B (`followController`). Files open in QOR Media, docked inside the launcher.
 *
 * Each file has a menu (`FileMenu`): right-click a tile, press the Menu key or Shift+F10 on it, or Y on a controller. It
 * opens, shows the file in its folder, or publishes it to the Mesh (L10.2), which the host asks the person to approve.
 */

import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useLayoutEffect,
  useMemo,
  useRef,
  useState,
  type KeyboardEvent,
  type MouseEvent,
  type ReactNode,
} from 'react';
import {
  Check,
  Clapperboard,
  FileText,
  Folder,
  FolderOpen,
  FolderPlus,
  Gamepad2,
  Image as ImageIcon,
  Library as LibraryIcon,
  Loader2,
  Music2,
  Play,
  RefreshCw,
  Search,
  Share2,
  Trash2,
  X,
} from 'lucide-react';

import { useQor } from '../state/store';
import { dockOver, factOf, library, onLibraryChanged, SHELVES, thumbnailOf, titleOf, type Item, type Kind } from './model';

type Tab = 'all' | 'games' | Kind | 'folders';

/** Where a file's menu opens: beside the pointer, or under the tile it was asked for from. */
type MenuAt = { item: Item; x: number; y: number };

/** How any tile asks for its file's menu, without every shelf passing it down. */
const MenuContext = createContext<(item: Item, x: number, y: number) => void>(() => {});

const TABS: { id: Tab; label: string }[] = [
  { id: 'all', label: 'Home' },
  { id: 'games', label: 'Games' },
  { id: 'video', label: 'Videos' },
  { id: 'audio', label: 'Music' },
  { id: 'picture', label: 'Pictures' },
  { id: 'document', label: 'Documents' },
  { id: 'folders', label: 'Folders' },
];

const KIND_ICON: Record<Kind, typeof Clapperboard> = {
  video: Clapperboard,
  audio: Music2,
  picture: ImageIcon,
  document: FileText,
  other: Folder,
};

function explain(e: unknown): string {
  return e && typeof e === 'object' && 'message' in e ? String((e as { message: unknown }).message) : String(e);
}

/* ─────────────────────────────── the view ──────────────────────────────── */

export function LibraryView() {
  const notify = useQor((s) => s.notify);
  const [folders, setFolders] = useState<string[] | null>(null);
  const [suggested, setSuggested] = useState<string[]>([]);
  const [shelves, setShelves] = useState<Record<string, Item[]>>({});
  const [counts, setCounts] = useState<Record<string, number>>({});
  const [tab, setTab] = useState<Tab>('all');
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<Item[] | null>(null);
  const [scanning, setScanning] = useState(false);
  const search = useRef<HTMLInputElement>(null);
  const root = useRef<HTMLDivElement>(null);
  const [menu, setMenu] = useState<MenuAt | null>(null);
  const [publishing, setPublishing] = useState<string | null>(null);
  const openMenu = useCallback((item: Item, x: number, y: number) => setMenu({ item, x, y }), []);

  // QOR Media docks over the whole Library while it shows, and is hidden when the person goes elsewhere.
  useEffect(() => (root.current ? dockOver(root.current) : undefined), []);
  useEffect(() => (root.current ? followController(root.current) : undefined), []);

  const readFolders = useCallback(
    () =>
      library
        .folders()
        .then((f) => {
          setFolders(f.folders);
          setSuggested(f.suggested);
        })
        .catch((e) => notify('bad', explain(e))),
    [notify],
  );

  const readShelves = useCallback(async () => {
    const next: Record<string, Item[]> = {};
    await Promise.all(
      SHELVES.map(async (s) => {
        next[s.id] = await library.shelf(s.kind, s.order, s.id === 'continue' ? 12 : 30).catch(() => []);
      }),
    );
    setShelves(next);
    const c = await library.counts().catch(() => []);
    setCounts(Object.fromEntries(c));
  }, []);

  useEffect(() => {
    void readFolders();
    void readShelves();
    return onLibraryChanged(() => {
      void readShelves();
    });
  }, [readFolders, readShelves]);

  // Search as the person types, a beat after they stop.
  useEffect(() => {
    const q = query.trim();
    if (!q) {
      setResults(null);
      return;
    }
    const id = window.setTimeout(() => {
      library
        .search(q, tab === 'all' || tab === 'games' || tab === 'folders' ? null : tab)
        .then(setResults)
        .catch(() => setResults([]));
    }, 120);
    return () => window.clearTimeout(id);
  }, [query, tab]);

  const open = useCallback(
    (item: Item) => {
      library.open(item.path).catch((e) => notify('bad', explain(e)));
    },
    [notify],
  );

  const publish = async (item: Item) => {
    setPublishing(item.path);
    try {
      const receipt = await library.publish(item.path);
      notify(
        'ok',
        `Published “${receipt.name}” as asset ${receipt.collection}/${receipt.item}. The Mesh serves it while seeding is on: find it on the Mesh page.`,
      );
    } catch (e) {
      const why = explain(e);
      if (!/declined/i.test(why)) notify('bad', why);
    } finally {
      setPublishing(null);
    }
  };

  const rescan = async () => {
    setScanning(true);
    try {
      const s = await library.scan();
      notify('ok', `Library up to date: ${s.added} new, ${s.changed} changed, ${s.removed} gone.`);
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setScanning(false);
    }
  };

  // `/` searches from anywhere in the Library; Escape clears it.
  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === '/' && document.activeElement !== search.current) {
      e.preventDefault();
      search.current?.focus();
    } else if (e.key === 'Escape' && query) {
      setQuery('');
    } else {
      moveFocus(e);
    }
  };

  const empty = folders !== null && folders.length === 0;
  const total = Object.values(counts).reduce((n, c) => n + c, 0);

  return (
    <MenuContext.Provider value={openMenu}>
    <div ref={root} className="library flex h-full min-h-0 flex-col" onKeyDown={onKey} data-library>
      <header className="library-header flex flex-none items-center gap-4 px-8 pb-3 pt-6">
        <span className="library-mark" aria-hidden="true">
          <LibraryIcon size={18} />
        </span>
        <h1 className="heading text-heading">Library</h1>
        <nav className="library-tabs" aria-label="Library sections" role="tablist">
          {TABS.map((t) => (
            <button
              key={t.id}
              type="button"
              role="tab"
              aria-selected={tab === t.id}
              className={tab === t.id ? 'library-tab library-tab-on' : 'library-tab'}
              onClick={() => setTab(t.id)}
              data-library-tab={t.id}
            >
              {t.label}
              {t.id !== 'all' && t.id !== 'games' && t.id !== 'folders' && counts[t.id] ? (
                <span className="numeric library-tab-count">{counts[t.id]}</span>
              ) : null}
            </button>
          ))}
        </nav>
        <label className="library-search ml-auto">
          <Search size={14} aria-hidden="true" />
          <input
            ref={search}
            type="search"
            placeholder={total ? `Search ${total.toLocaleString()} files` : 'Search'}
            aria-label="Search the Library"
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            data-library-search
          />
          <kbd className="library-kbd">/</kbd>
        </label>
        <button type="button" className="btn btn-ghost px-2" title="Look again" aria-label="Look again" disabled={scanning} onClick={() => void rescan()}>
          {scanning ? <Loader2 size={15} className="animate-spin" /> : <RefreshCw size={15} />}
        </button>
      </header>

      <div className="min-h-0 flex-1 overflow-y-auto px-8 pb-10" data-library-body>
        {folders === null ? (
          <p className="pt-10 text-caption text-ink-muted">
            <Loader2 size={12} className="mr-1 inline animate-spin" /> Opening your Library…
          </p>
        ) : empty && tab !== 'games' ? (
          <Welcome suggested={suggested} onDone={() => void readFolders().then(readShelves)} />
        ) : results ? (
          <Results items={results} query={query} onOpen={open} />
        ) : tab === 'all' ? (
          <Home shelves={shelves} onOpen={open} />
        ) : tab === 'games' ? (
          <Games />
        ) : tab === 'folders' ? (
          <Folders folders={folders} suggested={suggested} onChange={() => void readFolders().then(readShelves)} />
        ) : (
          <KindGrid kind={tab} onOpen={open} />
        )}
      </div>
      {publishing && (
        <p className="library-publishing" role="status" data-library-publishing>
          <Loader2 size={13} className="animate-spin" /> Publishing to the Mesh. Approve it in the window that opened.
        </p>
      )}
      {menu && (
        <FileMenu
          at={menu}
          busy={publishing !== null}
          onClose={() => setMenu(null)}
          onOpen={() => open(menu.item)}
          onReveal={() => void library.reveal(menu.item.path).catch((e) => notify('bad', explain(e)))}
          onPublish={() => void publish(menu.item)}
        />
      )}
    </div>
    </MenuContext.Provider>
  );
}

/* ─────────────────────────────── a file's menu ─────────────────────────── */

/**
 * A file's menu: open it, show it in its folder, or publish it to the Mesh. Keyboard and controller reach every item:
 * it takes focus when it opens, the arrow keys move within it, Enter chooses, and Escape or a click elsewhere closes it,
 * giving focus back to the tile it came from.
 */
function FileMenu({
  at,
  busy,
  onClose,
  onOpen,
  onReveal,
  onPublish,
}: {
  at: MenuAt;
  busy: boolean;
  onClose: () => void;
  onOpen: () => void;
  onReveal: () => void;
  onPublish: () => void;
}) {
  const ref = useRef<HTMLDivElement>(null);
  const [place, setPlace] = useState({ left: at.x, top: at.y });
  const playable = at.item.kind === 'video' || at.item.kind === 'audio';

  // Kept inside the window, then the first item focused.
  useLayoutEffect(() => {
    const menu = ref.current;
    if (!menu) return;
    const { width, height } = menu.getBoundingClientRect();
    setPlace({
      left: Math.max(8, Math.min(at.x, window.innerWidth - width - 8)),
      top: Math.max(8, Math.min(at.y, window.innerHeight - height - 8)),
    });
    menu.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
  }, [at.x, at.y]);

  useEffect(() => {
    const away = (e: globalThis.MouseEvent) => {
      if (!ref.current?.contains(e.target as Node)) onClose();
    };
    window.addEventListener('mousedown', away);
    window.addEventListener('blur', onClose);
    return () => {
      window.removeEventListener('mousedown', away);
      window.removeEventListener('blur', onClose);
    };
  }, [onClose]);

  // Focus goes back to the file's own tile, however the menu was opened: a right-click does not focus it.
  const close = () => {
    onClose();
    const tiles = document.querySelectorAll<HTMLElement>('[data-library] [data-tile]');
    [...tiles].find((t) => t.dataset.tile === at.item.path)?.focus();
  };
  const choose = (action: () => void) => () => {
    close();
    action();
  };
  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    const items = [...(ref.current?.querySelectorAll<HTMLElement>('[role="menuitem"]:not(:disabled)') ?? [])];
    const at = items.indexOf(document.activeElement as HTMLElement);
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      const step = e.key === 'ArrowDown' ? 1 : -1;
      items[(at + step + items.length) % items.length]?.focus();
    } else if (e.key === 'Escape' || e.key === 'Tab') {
      e.preventDefault();
      close();
    }
    // Arrow keys and Escape belong to the menu while it is open, not to the shelves behind it.
    e.stopPropagation();
  };

  return (
    <div
      ref={ref}
      className="file-menu"
      role="menu"
      aria-label={`${titleOf(at.item)}: what to do`}
      style={{ left: place.left, top: place.top }}
      onKeyDown={onKey}
      onContextMenu={(e) => e.preventDefault()}
      data-file-menu={at.item.path}
    >
      <p className="file-menu-head">
        <span className="file-menu-title">{titleOf(at.item)}</span>
        <span className="numeric file-menu-fact">{factOf(at.item)}</span>
      </p>
      <button type="button" role="menuitem" className="file-menu-item" onClick={choose(onOpen)} data-file-menu-open>
        <Play size={14} aria-hidden="true" /> {playable ? 'Play' : 'Open'}
      </button>
      <button type="button" role="menuitem" className="file-menu-item" onClick={choose(onReveal)} data-file-menu-reveal>
        <FolderOpen size={14} aria-hidden="true" /> Show in folder
      </button>
      <span className="file-menu-rule" role="separator" />
      <button
        type="button"
        role="menuitem"
        className="file-menu-item file-menu-item-mesh"
        disabled={busy}
        onClick={choose(onPublish)}
        data-file-menu-publish
      >
        <Share2 size={14} aria-hidden="true" />
        <span className="file-menu-item-text">
          Publish to the Mesh…
          <span className="file-menu-item-note">Makes it public, so other launchers can seed it</span>
        </span>
      </button>
    </div>
  );
}

/* ─────────────────────────────── focus ─────────────────────────────────── */

/** Arrow keys move between tiles in a row, and between rows at the nearest tile (a controller's D-pad arrives here too). */
function moveFocus(e: KeyboardEvent<HTMLDivElement>) {
  const here = document.activeElement as HTMLElement | null;
  if (!here?.dataset.tile) return;
  const row = here.closest<HTMLElement>('[data-row]');
  if (!row) return;
  const tiles = (r: Element) => [...r.querySelectorAll<HTMLElement>('[data-tile]')];
  const inRow = tiles(row);
  const at = inRow.indexOf(here);
  let next: HTMLElement | undefined;
  if (e.key === 'ArrowRight') next = inRow[at + 1];
  else if (e.key === 'ArrowLeft') next = inRow[at - 1];
  else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
    const rows = [...document.querySelectorAll<HTMLElement>('[data-library] [data-row]')];
    const target = rows[rows.indexOf(row) + (e.key === 'ArrowDown' ? 1 : -1)];
    if (target) {
      const x = here.getBoundingClientRect().left;
      next = tiles(target).reduce<HTMLElement | undefined>(
        (best, t) => (!best || Math.abs(t.getBoundingClientRect().left - x) < Math.abs(best.getBoundingClientRect().left - x) ? t : best),
        undefined,
      );
    }
  }
  if (next) {
    e.preventDefault();
    next.focus();
    next.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' });
  }
}

/** What a controller in the standard layout means here: D-pad or left stick as the arrow keys, A as Enter, B as Escape. */
const PAD: { key: string; on: (p: Gamepad) => boolean; repeats: boolean }[] = [
  { key: 'ArrowUp', on: (p) => !!p.buttons[12]?.pressed || (p.axes[1] ?? 0) < -0.5, repeats: true },
  { key: 'ArrowDown', on: (p) => !!p.buttons[13]?.pressed || (p.axes[1] ?? 0) > 0.5, repeats: true },
  { key: 'ArrowLeft', on: (p) => !!p.buttons[14]?.pressed || (p.axes[0] ?? 0) < -0.5, repeats: true },
  { key: 'ArrowRight', on: (p) => !!p.buttons[15]?.pressed || (p.axes[0] ?? 0) > 0.5, repeats: true },
  { key: 'Enter', on: (p) => !!p.buttons[0]?.pressed, repeats: false },
  { key: 'Escape', on: (p) => !!p.buttons[1]?.pressed, repeats: false },
  { key: 'ContextMenu', on: (p) => !!p.buttons[3]?.pressed, repeats: false },
];

/**
 * A game controller drives the Library as the keyboard does. WebView2 turns no controller input into keys, so the
 * Gamepad API is read each frame while a controller is connected: a direction moves the focus ring (the first tile, if
 * none has it yet) and repeats while held; A opens the focused tile; B goes back. Nothing is read while the launcher's
 * window is not the one in use, so a press meant for QOR Media, docked over the Library, does nothing here.
 */
function followController(root: HTMLElement): () => void {
  let frame = 0;
  const next = new Map<string, number>();
  const press = (key: string) => {
    const here = document.activeElement as HTMLElement | null;
    // Inside an open file menu the controller drives the menu: up and down, A to choose, B to close.
    if (here?.closest('[role="menu"]')) {
      if (key === 'Enter') here.click();
      else here.dispatchEvent(new globalThis.KeyboardEvent('keydown', { key, bubbles: true }));
      return;
    }
    const onTile = !!here?.dataset.tile && root.contains(here);
    if (key === 'Enter') {
      if (onTile) here!.click();
      return;
    }
    if (key === 'ContextMenu') {
      if (onTile) here!.dispatchEvent(new globalThis.KeyboardEvent('keydown', { key, bubbles: true }));
      return;
    }
    if (!onTile && key !== 'Escape') {
      const first = root.querySelector<HTMLElement>('[data-tile]');
      first?.focus();
      first?.scrollIntoView({ block: 'nearest', inline: 'nearest' });
      return;
    }
    (onTile ? here! : root).dispatchEvent(new globalThis.KeyboardEvent('keydown', { key, bubbles: true }));
  };
  const poll = (now: number) => {
    const pad = [...(navigator.getGamepads?.() ?? [])].find((p): p is Gamepad => !!p?.connected);
    if (!pad) {
      frame = 0;
      next.clear();
      return;
    }
    for (const { key, on, repeats } of PAD) {
      if (!on(pad) || !document.hasFocus()) {
        next.delete(key);
        continue;
      }
      const due = next.get(key);
      if (due === undefined) {
        press(key);
        next.set(key, repeats ? now + 380 : Infinity);
      } else if (now >= due) {
        press(key);
        next.set(key, now + 110);
      }
    }
    frame = requestAnimationFrame(poll);
  };
  const start = () => {
    if (!frame) frame = requestAnimationFrame(poll);
  };
  window.addEventListener('gamepadconnected', start);
  start();
  return () => {
    window.removeEventListener('gamepadconnected', start);
    cancelAnimationFrame(frame);
    frame = 0;
  };
}

/* ─────────────────────────────── tiles ─────────────────────────────────── */

function Art({ item, large = false }: { item: Item; large?: boolean }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    let live = true;
    if (item.kind === 'picture') void thumbnailOf(item.path).then((s) => live && setSrc(s));
    return () => {
      live = false;
    };
  }, [item.path, item.kind]);
  const Icon = KIND_ICON[item.kind] ?? Folder;
  return (
    <span className={`library-art library-art-${item.kind}`} aria-hidden="true">
      {src ? <img src={src} alt="" loading="lazy" /> : <Icon size={large ? 56 : 34} strokeWidth={1.4} />}
    </span>
  );
}

/** What opens a file's menu from a tile: a right-click at the pointer, or the Menu key or Shift+F10 under the tile. */
function useFileMenu(item: Item) {
  const openMenu = useContext(MenuContext);
  return {
    onContextMenu: (e: MouseEvent<HTMLElement>) => {
      e.preventDefault();
      openMenu(item, e.clientX, e.clientY);
    },
    onKeyDown: (e: KeyboardEvent<HTMLElement>) => {
      if (e.key === 'ContextMenu' || (e.key === 'F10' && e.shiftKey)) {
        e.preventDefault();
        e.stopPropagation();
        const r = e.currentTarget.getBoundingClientRect();
        openMenu(item, r.left + 12, r.bottom - 12);
      }
    },
  };
}

function Tile({ item, onOpen, wide = false }: { item: Item; onOpen: (i: Item) => void; wide?: boolean }) {
  const played = item.position && item.duration ? Math.min(1, item.position / item.duration) : 0;
  const menu = useFileMenu(item);
  return (
    <button
      type="button"
      className={`library-tile ${wide ? 'library-tile-wide' : ''}`}
      onClick={() => onOpen(item)}
      {...menu}
      aria-haspopup="menu"
      data-tile={item.path}
      title={item.path}
    >
      <Art item={item} />
      {played > 0 && played < 0.98 && (
        <span className="library-progress" aria-label={`${Math.round(played * 100)}% played`}>
          <span style={{ width: `${played * 100}%` }} />
        </span>
      )}
      <span className="library-tile-text">
        <span className="library-tile-title">{titleOf(item)}</span>
        <span className="numeric library-tile-fact">{factOf(item)}</span>
      </span>
    </button>
  );
}

function Shelf({ title, items, onOpen, wide = false }: { title: string; items: Item[]; onOpen: (i: Item) => void; wide?: boolean }) {
  if (items.length === 0) return null;
  return (
    <section className="library-shelf" aria-label={title} data-shelf={title}>
      <h2 className="eyebrow mb-3">{title}</h2>
      <div className="library-row" data-row>
        {items.map((i) => (
          <Tile key={i.path} item={i} onOpen={onOpen} wide={wide} />
        ))}
      </div>
    </section>
  );
}

/* ─────────────────────────────── sections ──────────────────────────────── */

function Home({ shelves, onOpen }: { shelves: Record<string, Item[]>; onOpen: (i: Item) => void }) {
  const hero = shelves.continue?.[0] ?? shelves.recent?.[0];
  return (
    <>
      {hero && <Hero item={hero} resumed={!!shelves.continue?.[0]} onOpen={onOpen} />}
      {SHELVES.map((s) => (
        <Shelf key={s.id} title={s.title} items={(shelves[s.id] ?? []).filter((i) => s.id !== 'continue' || i !== hero)} onOpen={onOpen} wide={s.kind === 'video' || s.id === 'continue'} />
      ))}
    </>
  );
}

function Hero({ item, resumed, onOpen }: { item: Item; resumed: boolean; onOpen: (i: Item) => void }) {
  return (
    <section className="library-hero" aria-label="Last opened" data-library-hero>
      <Art item={item} large />
      <div className="library-hero-text">
        <p className="eyebrow text-accent">{resumed ? 'Pick up where you left off' : 'Newest in your Library'}</p>
        <h2 className="heading library-hero-title">{titleOf(item)}</h2>
        <p className="numeric text-caption text-ink-body">{factOf(item)}</p>
        <div className="library-row mt-5" data-row>
          <button type="button" className="btn btn-primary library-hero-play" onClick={() => onOpen(item)} data-tile={item.path}>
            <Play size={15} /> {item.kind === 'video' || item.kind === 'audio' ? (resumed && item.position ? 'Resume' : 'Play') : 'Open'}
          </button>
          <button type="button" className="btn btn-ghost" onClick={() => void library.reveal(item.path)} data-tile={`${item.path}#reveal`}>
            <FolderOpen size={14} /> Show in folder
          </button>
        </div>
      </div>
    </section>
  );
}

function KindGrid({ kind, onOpen }: { kind: Kind; onOpen: (i: Item) => void }) {
  const [items, setItems] = useState<Item[] | null>(null);
  useEffect(() => {
    const read = () => void library.shelf(kind, 'added', 500).then(setItems).catch(() => setItems([]));
    read();
    return onLibraryChanged(read);
  }, [kind]);
  if (!items) return <p className="text-caption text-ink-muted">Reading…</p>;
  if (items.length === 0) return <p className="pt-6 text-ui text-ink-muted">Nothing of this kind in your folders yet.</p>;
  return <Grid items={items} onOpen={onOpen} />;
}

function Grid({ items, onOpen }: { items: Item[]; onOpen: (i: Item) => void }) {
  // Rows of six, so the arrow keys move as the eye does.
  const rows = useMemo(() => {
    const out: Item[][] = [];
    for (let i = 0; i < items.length; i += 6) out.push(items.slice(i, i + 6));
    return out;
  }, [items]);
  return (
    <div className="library-grid">
      {rows.map((row, i) => (
        <div key={i} className="library-grid-row" data-row>
          {row.map((item) => (
            <Tile key={item.path} item={item} onOpen={onOpen} />
          ))}
        </div>
      ))}
    </div>
  );
}

function Results({ items, query, onOpen }: { items: Item[]; query: string; onOpen: (i: Item) => void }) {
  return (
    <section aria-label="Search results" data-library-results>
      <p className="eyebrow mb-3">
        {items.length === 0 ? `Nothing matches “${query.trim()}”` : `${items.length} for “${query.trim()}”`}
      </p>
      <Grid items={items} onOpen={onOpen} />
    </section>
  );
}

function Games() {
  return (
    <Panel icon={<Gamepad2 size={22} />} title="Your games and work">
      Games and work you own on the chain appear here, ready to install and play, once the Library can fetch them from the
      Mesh (L7.1). Everything you buy is already yours in your Inventory.
    </Panel>
  );
}

function Panel({ icon, title, children }: { icon: ReactNode; title: string; children: ReactNode }) {
  return (
    <div className="library-panel" data-library-panel>
      <span className="text-accent">{icon}</span>
      <h2 className="heading text-title">{title}</h2>
      <p className="max-w-xl text-ui text-ink-body">{children}</p>
    </div>
  );
}

/* ─────────────────────────────── folders ───────────────────────────────── */

function Welcome({ suggested, onDone }: { suggested: string[]; onDone: () => void }) {
  const notify = useQor((s) => s.notify);
  const [chosen, setChosen] = useState<string[]>(suggested);
  const [busy, setBusy] = useState(false);
  useEffect(() => setChosen(suggested), [suggested]);
  const add = async (paths: (string | undefined)[]) => {
    setBusy(true);
    try {
      for (const p of paths) await library.addFolder(p);
      onDone();
    } catch (e) {
      notify('bad', explain(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <div className="library-welcome" data-library-welcome>
      <span className="library-mark library-mark-lg" aria-hidden="true">
        <LibraryIcon size={28} />
      </span>
      <h2 className="heading text-heading">Bring your files in</h2>
      <p className="max-w-lg text-ui text-ink-body">
        Choose the folders your Library keeps. It reads them on this PC only, follows them as they change, and makes every
        video, song, picture and document one search away.
      </p>
      {suggested.length > 0 && (
        <ul className="library-choices" aria-label="Folders to add">
          {suggested.map((s) => (
            <li key={s}>
              <label className="library-choice">
                <input
                  type="checkbox"
                  checked={chosen.includes(s)}
                  onChange={(e) => setChosen((c) => (e.target.checked ? [...c, s] : c.filter((x) => x !== s)))}
                />
                <Folder size={14} aria-hidden="true" />
                <span className="truncate">{s}</span>
              </label>
            </li>
          ))}
        </ul>
      )}
      <div className="flex flex-wrap justify-center gap-2">
        {suggested.length > 0 && (
          <button type="button" className="btn btn-primary" disabled={busy || chosen.length === 0} onClick={() => void add(chosen)}>
            {busy ? <Loader2 size={14} className="animate-spin" /> : <Check size={14} />} Add {chosen.length === 1 ? 'this folder' : `these ${chosen.length}`}
          </button>
        )}
        <button type="button" className="btn" disabled={busy} onClick={() => void add([undefined])}>
          <FolderPlus size={14} /> Choose a folder
        </button>
      </div>
    </div>
  );
}

function Folders({ folders, suggested, onChange }: { folders: string[]; suggested: string[]; onChange: () => void }) {
  const notify = useQor((s) => s.notify);
  const act = async (fn: () => Promise<unknown>) => {
    try {
      await fn();
      onChange();
    } catch (e) {
      notify('bad', explain(e));
    }
  };
  return (
    <section className="max-w-3xl" aria-label="Folders" data-library-folders>
      <p className="mb-4 text-ui text-ink-body">The folders your Library keeps, read on this PC only. Taking one out leaves its files where they are.</p>
      <ul className="flex flex-col gap-2">
        {folders.map((f) => (
          <li key={f} className="library-folder">
            <Folder size={16} className="text-accent" aria-hidden="true" />
            <span className="min-w-0 flex-1 truncate text-ui text-ink">{f}</span>
            <button type="button" className="btn btn-ghost btn-danger px-2" aria-label={`Take ${f} out of the Library`} onClick={() => void act(() => library.removeFolder(f))}>
              <Trash2 size={14} />
            </button>
          </li>
        ))}
        {suggested.map((s) => (
          <li key={s} className="library-folder library-folder-suggested">
            <Folder size={16} aria-hidden="true" />
            <span className="min-w-0 flex-1 truncate text-ui text-ink-muted">{s}</span>
            <button type="button" className="btn btn-ghost px-2" onClick={() => void act(() => library.addFolder(s))}>
              <FolderPlus size={14} /> Add
            </button>
          </li>
        ))}
      </ul>
      <button type="button" className="btn mt-4" onClick={() => void act(() => library.addFolder())}>
        <FolderPlus size={14} /> Choose a folder
      </button>
      {folders.length === 0 && suggested.length === 0 && (
        <p className="mt-3 text-caption text-ink-muted">
          <X size={11} className="mr-1 inline" /> No folders yet.
        </p>
      )}
    </section>
  );
}
