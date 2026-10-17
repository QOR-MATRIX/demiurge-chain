/**
 * The Library, as the interface holds it (ADR-094; docs/blueprints/library.md).
 *
 * The launcher's host keeps the index of the person's chosen folders; this side asks it for shelves and search results,
 * and listens for `library://changed` to read them again. Thumbnails arrive as raw bytes and are shown through object
 * URLs, kept for as long as the launcher runs.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

export type Kind = 'video' | 'audio' | 'picture' | 'document' | 'other';

export interface Item {
  path: string;
  folder: string;
  name: string;
  kind: Kind;
  size: number;
  modified: number;
  added: number;
  opened: number | null;
  position: number | null;
  duration: number | null;
  width: number | null;
  height: number | null;
  taken: string | null;
  camera: string | null;
  title: string | null;
  artist: string | null;
  album: string | null;
}

export const library = {
  folders: () => invoke<{ folders: string[]; suggested: string[] }>('library_folders'),
  addFolder: (path?: string) => invoke<string | null>('library_add_folder', { path: path ?? null }),
  removeFolder: (path: string) => invoke<void>('library_remove_folder', { path }),
  scan: () => invoke<{ seen: number; added: number; changed: number; removed: number }>('library_scan'),
  search: (query: string, kind?: Kind | null, limit = 60) => invoke<Item[]>('library_search', { query, kind: kind ?? null, limit }),
  shelf: (kind: Kind | null, order: 'added' | 'opened' | 'name' = 'added', limit = 40) =>
    invoke<Item[]>('library_shelf', { kind, order, limit }),
  counts: () => invoke<[string, number][]>('library_counts'),
  thumbnail: (path: string) => invoke<ArrayBuffer>('library_thumbnail', { path }),
  reveal: (path: string) => invoke<void>('library_reveal', { path }),
  open: (path: string) => invoke<void>('library_open', { path }),
};

/** Where QOR Media sits, docked over the Library: a rectangle of the launcher's window in physical pixels. */
export const media = {
  place: (rect: { x: number; y: number; w: number; h: number }) => invoke<void>('media_place', { rect }),
  hide: () => invoke<void>('media_hide'),
  theme: (colors: Record<string, string>) => invoke<void>('media_theme', { colors }),
};

const THEME_TOKENS = ['void', 'base', 'surface', 'raised', 'well', 'edge', 'accent', 'accent-bright', 'accent-dim', 'ink', 'ink-body', 'ink-muted', 'ink-faint'];

/** The launcher's theme as `#rrggbb` tokens, read from the page, for QOR Media to wear. */
function themeTokens(): Record<string, string> {
  const style = getComputedStyle(document.documentElement);
  return Object.fromEntries(
    THEME_TOKENS.map((t): [string, string] => [t, style.getPropertyValue(`--${t}`).trim().toLowerCase()]).filter(([, v]) => /^#[0-9a-f]{6}$/.test(v)),
  );
}

/** Keep QOR Media's place in step with `el` while the Library is showing, and hide it when the Library goes. */
export function dockOver(el: HTMLElement): () => void {
  let last = '';
  const send = () => {
    const r = el.getBoundingClientRect();
    const d = window.devicePixelRatio || 1;
    const rect = { x: Math.round(r.left * d), y: Math.round(r.top * d), w: Math.round(r.width * d), h: Math.round(r.height * d) };
    const key = JSON.stringify(rect);
    if (key === last || rect.w === 0 || rect.h === 0) return;
    last = key;
    void media.place(rect).catch(() => undefined);
  };
  const observer = new ResizeObserver(send);
  observer.observe(el);
  window.addEventListener('resize', send);
  send();
  void media.theme(themeTokens()).catch(() => undefined);
  // A theme change restyles the page's root: QOR Media follows it.
  const themed = new MutationObserver(() => void media.theme(themeTokens()).catch(() => undefined));
  themed.observe(document.documentElement, { attributes: true, attributeFilter: ['style', 'class', 'data-theme'] });
  return () => {
    observer.disconnect();
    themed.disconnect();
    window.removeEventListener('resize', send);
    void media.hide().catch(() => undefined);
  };
}

/** Call `fn` whenever the host says the index changed. Returns how to stop. */
export function onLibraryChanged(fn: () => void): () => void {
  let stop: (() => void) | null = null;
  let stopped = false;
  listen('library://changed', () => fn())
    .then((un) => (stopped ? un() : (stop = un)))
    .catch(() => undefined);
  return () => {
    stopped = true;
    stop?.();
  };
}

const thumbnails = new Map<string, Promise<string | null>>();

/** A picture's thumbnail as an object URL, asked for once per path. */
export function thumbnailOf(path: string): Promise<string | null> {
  let held = thumbnails.get(path);
  if (!held) {
    held = library
      .thumbnail(path)
      .then((bytes) => URL.createObjectURL(new Blob([bytes], { type: 'image/jpeg' })))
      .catch(() => null);
    thumbnails.set(path, held);
  }
  return held;
}

/** The title a tile shows: a song's title, else the file's name without its extension. */
export function titleOf(item: Item): string {
  if (item.title) return item.title;
  const dot = item.name.lastIndexOf('.');
  return dot > 0 ? item.name.slice(0, dot) : item.name;
}

export function duration(seconds: number | null | undefined): string | null {
  if (!seconds || !Number.isFinite(seconds)) return null;
  const s = Math.round(seconds);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = String(s % 60).padStart(2, '0');
  return h ? `${h}:${String(m).padStart(2, '0')}:${sec}` : `${m}:${sec}`;
}

export function bytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ['KB', 'MB', 'GB', 'TB'];
  let v = n / 1024;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u++;
  }
  return `${v.toFixed(v < 10 ? 1 : 0)} ${units[u]}`;
}

/** The one fact a tile shows beneath its title. */
export function factOf(item: Item): string {
  switch (item.kind) {
    case 'video':
      return [duration(item.duration), item.height ? `${item.height}p` : null, bytes(item.size)].filter(Boolean).join(' · ');
    case 'audio':
      return [item.artist, duration(item.duration)].filter(Boolean).join(' · ') || bytes(item.size);
    case 'picture':
      return item.width && item.height ? `${item.width} × ${item.height}` : bytes(item.size);
    default:
      return `${(item.name.split('.').pop() ?? '').toUpperCase()} · ${bytes(item.size)}`;
  }
}

export const SHELVES: { id: string; title: string; kind: Kind | null; order: 'added' | 'opened' }[] = [
  { id: 'continue', title: 'Continue', kind: null, order: 'opened' },
  { id: 'recent', title: 'Recently added', kind: null, order: 'added' },
  { id: 'video', title: 'Videos', kind: 'video', order: 'added' },
  { id: 'audio', title: 'Music', kind: 'audio', order: 'added' },
  { id: 'picture', title: 'Pictures', kind: 'picture', order: 'added' },
  { id: 'document', title: 'Documents', kind: 'document', order: 'added' },
];
