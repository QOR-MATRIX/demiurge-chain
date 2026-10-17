/**
 * Translation on this PC, for Social (ADR-092 decision 3).
 *
 * World and public rooms are shown in the reader's language. Each message carries the language it was written in; when
 * that is not the reader's, it is translated here, by Mozilla's Bergamot engine in a worker (`public/translate/`), with
 * models the launcher's host downloaded and checked against Mozilla's hashes. No message leaves this PC to be
 * translated, nothing is paid per message, and translations are kept only in memory, for as long as the launcher runs.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { create } from 'zustand';

import type { Message, RoomSummary } from './model';

/* ───────────────────────────────── the host ─────────────────────────────── */

export interface ModelFile {
  kind: string;
  name: string;
  hash: string;
  size: number;
}

export interface Plan {
  from: string;
  to: string;
  pairs: { from: string; to: string; files: ModelFile[] }[];
  engine: ModelFile;
  to_download: number;
}

export const translateHost = {
  languages: () => invoke<{ languages: string[]; system: string | null }>('translate_languages'),
  plan: (from: string, to: string) => invoke<Plan | null>('translate_plan', { from, to }),
  prepare: (from: string, to: string) => invoke<Plan | null>('translate_prepare', { from, to }),
  file: (hash: string) => invoke<ArrayBuffer>('translate_file', { hash }),
  detect: (text: string) => invoke<string | null>('translate_detect', { text }),
};

/* ───────────────────────────────── languages ────────────────────────────── */

/** A language tag's name, in the reader's own language where the PC knows it: `es` is "Spanish", or "español". */
export function languageName(tag: string | null | undefined, inLanguage?: string | null): string {
  if (!tag) return 'another language';
  try {
    const names = new Intl.DisplayNames(inLanguage ? [inLanguage, 'en'] : undefined, { type: 'language' });
    return names.of(tag) ?? tag;
  } catch {
    return tag;
  }
}

/** The part of a tag that decides whether two people read the same language: `pt-BR` and `pt` do; `zh-Hans` and
 *  `zh-Hant` do not. */
function base(tag: string): string {
  const parts = tag.toLowerCase().split(/[-_]/);
  if (parts[0] === 'zh') return parts.some((p) => ['hant', 'tw', 'hk', 'mo'].includes(p)) ? 'zh-hant' : 'zh-hans';
  if (parts[0] === 'no') return 'nb';
  return parts[0] ?? tag;
}

export function sameLanguage(a: string, b: string): boolean {
  return base(a) === base(b);
}

/** The rooms translated: World and public rooms (the owner's choice, ADR-092). */
export function translatable(room: RoomSummary | undefined): boolean {
  return room?.kind === 'world' || room?.kind === 'public';
}

/* ───────────────────────────────── the store ────────────────────────────── */

type RouteState = 'preparing' | 'ready' | 'unavailable' | 'failed';

interface Downloading {
  from: string;
  to: string;
  done: number;
  total: number;
}

interface TranslationState {
  /** The reader's switch, remembered on this PC. */
  on: boolean;
  /** Per written language and reader's language (`es>en`), how far its route is. */
  routes: Record<string, RouteState>;
  downloading: Downloading | null;
  /** Translated words, by message, its last edit and the reader's language. */
  done: Record<string, string>;
  failed: Record<string, true>;
  /** Messages the reader asked to see as written. */
  original: Record<string, true>;
  problem: string | null;
  setOn: (on: boolean) => void;
  toggleOriginal: (id: string) => void;
}

const SWITCH = 'qor.social.translate';

function remembered(): boolean {
  try {
    return localStorage.getItem(SWITCH) !== 'off';
  } catch {
    return true;
  }
}

export const useTranslation = create<TranslationState>((set) => ({
  on: remembered(),
  routes: {},
  downloading: null,
  done: {},
  failed: {},
  original: {},
  problem: null,
  setOn: (on) => {
    try {
      localStorage.setItem(SWITCH, on ? 'on' : 'off');
    } catch {
      /* the switch still holds for this run */
    }
    set({ on });
  },
  toggleOriginal: (id) =>
    set((s) => {
      const original = { ...s.original };
      if (original[id]) delete original[id];
      else original[id] = true;
      return { original };
    }),
}));

export const keyOf = (m: Message, reader: string) => `${m.id}:${m.edited_at ?? ''}:${base(reader)}`;

/* ───────────────────────────────── engines ──────────────────────────────── */

interface Engine {
  worker: Worker;
  ready: Promise<void>;
  waiting: Map<number, { resolve: (text: string) => void; reject: (e: Error) => void }>;
  used: number;
}

/** Engines by the route they run, model languages and all (`es>ja`); a few at once, the least used let go. */
const engines = new Map<string, Engine>();
const ENGINES_HELD = 3;
let nextId = 1;

function engineFor(plan: Plan, files: Map<string, ArrayBuffer>): Engine {
  const key = `${plan.from}>${plan.to}`;
  const held = engines.get(key);
  if (held) {
    held.used = Date.now();
    return held;
  }
  if (engines.size >= ENGINES_HELD) {
    const oldest = [...engines.entries()].sort((a, b) => a[1].used - b[1].used)[0];
    if (oldest) {
      oldest[1].worker.terminate();
      engines.delete(oldest[0]);
    }
  }
  const worker = new Worker('/translate/worker.js');
  const waiting: Engine['waiting'] = new Map();
  const ready = new Promise<void>((resolve, reject) => {
    worker.addEventListener('message', ({ data }) => {
      if (data.type === 'ready') resolve();
      else if (data.type === 'error') reject(new Error(data.message));
      else if (data.type === 'done' || data.type === 'failed') {
        const w = waiting.get(data.id);
        waiting.delete(data.id);
        if (data.type === 'done') w?.resolve(data.text);
        else w?.reject(new Error(data.message));
      }
    });
    worker.addEventListener('error', () => reject(new Error('The translation engine stopped.')));
  });
  if (plan.engine.kind === 'mock') {
    worker.postMessage({ type: 'init', mock: true, pairs: plan.pairs.map(({ from, to }) => ({ from, to })) });
  } else {
    const transfer: ArrayBuffer[] = [];
    const take = (hash: string) => {
      const bytes = files.get(hash)!;
      transfer.push(bytes);
      return bytes;
    };
    const engine = take(plan.engine.hash);
    const pairs = plan.pairs.map((p) => ({
      from: p.from,
      to: p.to,
      files: Object.fromEntries(p.files.map((f) => [f.kind, { name: f.name, bytes: take(f.hash) }])),
    }));
    worker.postMessage({ type: 'init', engine, pairs }, transfer);
  }
  const made: Engine = { worker, ready, waiting, used: Date.now() };
  engines.set(key, made);
  return made;
}

function run(engine: Engine, text: string): Promise<string> {
  const id = nextId++;
  return new Promise((resolve, reject) => {
    engine.waiting.set(id, { resolve, reject });
    engine.worker.postMessage({ type: 'translate', id, text });
  });
}

/* ───────────────────────────────── routes ───────────────────────────────── */

const plans = new Map<string, Promise<Plan | null>>();
let progress: Record<string, number> = {};

listen<{ hash: string; done: number; total: number }>('translate://progress', (e) => {
  progress = { ...progress, [e.payload.hash]: e.payload.done };
  const d = useTranslation.getState().downloading;
  if (d) {
    const done = Object.values(progress).reduce((n, x) => n + x, 0);
    useTranslation.setState({ downloading: { ...d, done: Math.min(done, d.total) } });
  }
}).catch(() => undefined);

/** What turns `from` into `to`, downloaded and loaded into an engine; null when Mozilla has nothing for it. */
function routeFor(from: string, to: string): Promise<{ plan: Plan; engine: Engine } | null> {
  const key = `${base(from)}>${base(to)}`;
  let planned = plans.get(key);
  if (!planned) {
    planned = (async () => {
      const set = (state: RouteState) =>
        useTranslation.setState((s) => ({ routes: { ...s.routes, [key]: state } }));
      set('preparing');
      try {
        const plan = await translateHost.plan(from, to);
        if (!plan) {
          set('unavailable');
          return null;
        }
        if (plan.to_download > 0) {
          progress = {};
          useTranslation.setState({ downloading: { from: plan.from, to: plan.to, done: 0, total: plan.to_download } });
          try {
            await translateHost.prepare(from, to);
          } finally {
            useTranslation.setState({ downloading: null });
          }
        }
        set('ready');
        return plan;
      } catch (e) {
        set('failed');
        plans.delete(key);
        const message = e && typeof e === 'object' && 'message' in e ? String((e as { message: unknown }).message) : String(e);
        useTranslation.setState({ problem: message });
        return null;
      }
    })();
    plans.set(key, planned);
  }
  return planned.then(async (plan) => {
    if (!plan) return null;
    const files = new Map<string, ArrayBuffer>();
    const routeKey = `${plan.from}>${plan.to}`;
    if (!engines.has(routeKey) && plan.engine.kind !== 'mock') {
      for (const f of [plan.engine, ...plan.pairs.flatMap((p) => p.files)]) files.set(f.hash, await translateHost.file(f.hash));
    }
    const engine = engineFor(plan, files);
    await engine.ready;
    return { plan, engine };
  });
}

/* ───────────────────────────────── the queue ────────────────────────────── */

interface Job {
  message: Message;
  reader: string;
}

/** Messages waiting, by key; the newest asked for is translated first, since it is the one on screen. */
const queue = new Map<string, Job>();
const asked = new Set<string>();
let working = false;

async function work() {
  if (working) return;
  working = true;
  try {
    while (queue.size > 0) {
      const [key, job] = [...queue.entries()].pop()!;
      queue.delete(key);
      const { message, reader } = job;
      try {
        const route = await routeFor(message.lang!, reader);
        if (!route) {
          // Mozilla has nothing for this language: it is shown as written, and says so, rather than waiting forever.
          useTranslation.setState((s) => ({ failed: { ...s.failed, [key]: true } }));
          continue;
        }
        const text = await run(route.engine, message.body ?? '');
        useTranslation.setState((s) => ({ done: { ...s.done, [key]: text } }));
      } catch {
        useTranslation.setState((s) => ({ failed: { ...s.failed, [key]: true } }));
      }
    }
  } finally {
    working = false;
  }
}

/** Whether `m` is shown translated for `reader` in `room`: written in another language, in a room that is translated. */
export function wanted(m: Message, room: RoomSummary | undefined, reader: string | null): reader is string {
  return (
    !!reader &&
    translatable(room) &&
    !m.deleted &&
    !!m.body &&
    !!m.lang &&
    !sameLanguage(m.lang, reader) &&
    useTranslation.getState().on
  );
}

/** Ask for `m` in `reader`'s language. Each message, edit and language is asked for once. */
export function request(m: Message, reader: string) {
  const key = keyOf(m, reader);
  if (asked.has(key)) return;
  asked.add(key);
  queue.set(key, { message: m, reader });
  void work();
}
