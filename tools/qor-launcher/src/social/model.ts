/**
 * Social, as the interface holds it (ADR-091; docs/blueprints/social.md; roadmap L8.5).
 *
 * The launcher's host holds the one socket to Social and the QOR ID token; this side only asks (`social_request`)
 * and listens (`social://status`, `social://welcome`, `social://event`). Every event is folded into one store here, so
 * a message, a reaction or someone arriving shows up wherever it is shown, at once.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { create } from 'zustand';

/* ───────────────────────────────── shapes ───────────────────────────────── */

export type Status =
  | { state: 'off' }
  | { state: 'connecting' }
  | { state: 'connected' }
  | { state: 'retrying'; in_secs: number; reason: string }
  | { state: 'signed_out' }
  | { state: 'refused'; reason: string };

export interface Backdrop {
  preset?: BackdropName;
  hue?: number;
  intensity?: number;
}

export type BackdropName = 'drift' | 'aurora' | 'nebula' | 'lattice' | 'starfield' | 'liquid';

export interface Presence {
  state: 'online' | 'away' | 'offline';
  playing: string | null;
}

export interface Profile {
  sub: string;
  qor_id: string;
  avatar_url: string | null;
  level: number;
  display: string;
  bio: string;
  accent: string | null;
  banner: Backdrop;
  showcase: string[];
  status_line: string;
  seen_at: string;
  presence?: Presence;
}

export interface Room {
  id: string;
  kind: 'public' | 'private' | 'direct';
  name: string;
  topic: string;
  ambience: Backdrop;
  created_at: string;
}

export interface RoomSummary extends Room {
  role: 'owner' | 'moderator' | 'member';
  unread: number;
  last_seq: number;
  members: number;
  other: string | null;
}

export interface Reaction {
  emoji: string;
  by: string[];
}

export interface Message {
  id: string;
  room: string;
  seq: number;
  author: string;
  body: string | null;
  reply_to: string | null;
  reactions: Reaction[];
  edited_at: string | null;
  deleted: boolean;
  created_at: string;
}

export interface Invite {
  room: string;
  name: string;
  inviter: string;
  created_at: string;
}

export interface Member {
  sub: string;
  role: 'owner' | 'moderator' | 'member';
  joined_at: string;
}

export interface Welcome {
  me: Profile;
  username_chosen: boolean;
  rooms: RoomSummary[];
  invites: Invite[];
  blocked: string[];
  presence: Record<string, Presence>;
}

/** A refusal from Social, with its own code. */
export interface SocialRefusal {
  kind: string;
  code?: string;
  message: string;
}

export function refusalOf(error: unknown): SocialRefusal {
  if (error && typeof error === 'object' && 'message' in error) return error as SocialRefusal;
  return { kind: 'internal', message: String(error) };
}

/* ───────────────────────────────── the host ─────────────────────────────── */

export const host = {
  connect: () => invoke<void>('social_connect'),
  disconnect: () => invoke<void>('social_disconnect'),
  status: () => invoke<{ status: Status; welcome: (Welcome & { type: string }) | null; endpoint: string }>('social_status'),
  request: <T,>(method: string, params: Record<string, unknown> = {}) =>
    invoke<T>('social_request', { method, params }),
  claimUsername: (username: string) => invoke<unknown>('qor_claim_username', { username }),
};

/** Listen to the host's three Social events. Returns how to stop. */
export function listenToSocial(handlers: {
  status: (s: Status) => void;
  welcome: (w: Welcome) => void;
  event: (e: { event: string; data: Record<string, unknown> }) => void;
}): () => void {
  const stops: (() => void)[] = [];
  let stopped = false;
  const add = <T,>(name: string, fn: (payload: T) => void) => {
    listen<T>(name, (e) => fn(e.payload))
      .then((un) => (stopped ? un() : stops.push(un)))
      .catch(() => undefined);
  };
  add<Status>('social://status', handlers.status);
  add<Welcome>('social://welcome', handlers.welcome);
  add<{ event: string; data: Record<string, unknown> }>('social://event', handlers.event);
  return () => {
    stopped = true;
    stops.forEach((s) => s());
  };
}

/* ───────────────────────────────── the store ────────────────────────────── */

interface Thread {
  messages: Message[];
  /** Whether older messages may exist before the first one held. */
  more: boolean;
  loaded: boolean;
}

interface SocialState {
  status: Status;
  me: Profile | null;
  usernameChosen: boolean;
  rooms: Record<string, RoomSummary>;
  invites: Invite[];
  blocked: string[];
  profiles: Record<string, Profile>;
  presence: Record<string, Presence>;
  threads: Record<string, Thread>;
  /** Who is typing where, and until when (ms since the epoch). */
  typing: Record<string, Record<string, number>>;
  mentions: Record<string, number>;
  active: string | null;
  /** Whose card is open, if anyone's. */
  card: string | null;

  setStatus: (s: Status) => void;
  welcome: (w: Welcome) => void;
  apply: (e: { event: string; data: Record<string, unknown> }) => void;
  remember: (profiles: Record<string, Profile>) => void;
  open: (room: string | null) => void;
  showCard: (sub: string | null) => void;
  load: (room: string, older?: boolean) => Promise<void>;
  putRoom: (room: Room, role: RoomSummary['role']) => void;
  dropRoom: (room: string) => void;
  putMessage: (m: Message) => void;
}

const sorted = (messages: Message[]) => [...messages].sort((a, b) => a.seq - b.seq);

export const useSocial = create<SocialState>((set, get) => ({
  status: { state: 'off' },
  me: null,
  usernameChosen: true,
  rooms: {},
  invites: [],
  blocked: [],
  profiles: {},
  presence: {},
  threads: {},
  typing: {},
  mentions: {},
  active: null,
  card: null,

  setStatus: (status) => set({ status }),

  welcome: (w) =>
    set((s) => {
      const rooms: Record<string, RoomSummary> = {};
      for (const r of w.rooms) rooms[r.id] = r;
      // The conversation the person had open, if it is still theirs; never one they did not choose, which would mark
      // it read unseen.
      const active = s.active && rooms[s.active] ? s.active : null;
      return {
        me: w.me,
        usernameChosen: w.username_chosen,
        rooms,
        invites: w.invites,
        blocked: w.blocked,
        presence: { ...s.presence, ...w.presence, [w.me.sub]: { state: 'online', playing: null } },
        profiles: { ...s.profiles, [w.me.sub]: w.me },
        active,
        // What was held may be stale after a reconnect: each room reloads when it is next shown.
        threads: {},
      };
    }),

  apply: ({ event, data }) => {
    const s = get();
    switch (event) {
      case 'message.created': {
        const m = data.message as Message;
        get().putMessage(m);
        const mine = m.author === s.me?.sub;
        const room = s.rooms[m.room];
        if (room) {
          const seen = s.active === m.room && document.hasFocus();
          set({
            rooms: {
              ...get().rooms,
              [m.room]: {
                ...room,
                last_seq: Math.max(room.last_seq, m.seq),
                unread: mine || seen ? room.unread : room.unread + 1,
              },
            },
          });
          if (seen && !mine) void host.request('messages.read', { room: m.room, seq: m.seq }).catch(() => undefined);
        }
        // Whoever spoke has stopped typing.
        const typing = { ...get().typing[m.room] };
        delete typing[m.author];
        set({ typing: { ...get().typing, [m.room]: typing } });
        break;
      }
      case 'message.edited':
      case 'message.deleted':
      case 'message.reaction':
        get().putMessage(data.message as Message);
        break;
      case 'mention': {
        const room = data.room as string;
        if (s.active !== room) set({ mentions: { ...s.mentions, [room]: (s.mentions[room] ?? 0) + 1 } });
        break;
      }
      case 'typing': {
        const room = data.room as string;
        const sub = data.sub as string;
        set({ typing: { ...s.typing, [room]: { ...s.typing[room], [sub]: Date.now() + 6000 } } });
        break;
      }
      case 'presence':
        set({ presence: { ...s.presence, [data.sub as string]: data.presence as Presence } });
        break;
      case 'profile.updated': {
        const p = data.profile as Profile;
        set({ profiles: { ...s.profiles, [p.sub]: p }, me: s.me?.sub === p.sub ? p : s.me });
        break;
      }
      case 'room.joined':
        get().putRoom(data.room as Room, data.role as RoomSummary['role']);
        break;
      case 'room.updated': {
        const r = data.room as Room;
        const held = s.rooms[r.id];
        if (held) set({ rooms: { ...s.rooms, [r.id]: { ...held, ...r } } });
        break;
      }
      case 'member.joined': {
        const held = s.rooms[data.room as string];
        if (held) set({ rooms: { ...s.rooms, [held.id]: { ...held, members: held.members + 1 } } });
        break;
      }
      case 'member.left': {
        const room = data.room as string;
        if (data.sub === s.me?.sub || data.closed) get().dropRoom(room);
        else {
          const held = s.rooms[room];
          if (held) set({ rooms: { ...s.rooms, [room]: { ...held, members: Math.max(1, held.members - 1) } } });
        }
        break;
      }
      case 'member.role': {
        const room = s.rooms[data.room as string];
        if (room && data.sub === s.me?.sub)
          set({ rooms: { ...s.rooms, [room.id]: { ...room, role: data.role as RoomSummary['role'] } } });
        break;
      }
      case 'invite':
        set({
          invites: [
            { room: data.room as string, name: data.name as string, inviter: data.inviter as string, created_at: new Date().toISOString() },
            ...s.invites.filter((i) => i.room !== data.room),
          ],
        });
        break;
      case 'read': {
        const room = s.rooms[data.room as string];
        if (room) set({ rooms: { ...s.rooms, [room.id]: { ...room, unread: 0 } } });
        break;
      }
    }
  },

  remember: (profiles) =>
    set((s) => {
      const presence = { ...s.presence };
      for (const p of Object.values(profiles)) if (p.presence) presence[p.sub] = p.presence;
      return { profiles: { ...s.profiles, ...profiles }, presence };
    }),

  // The conversation it opens loads its own history, once (`Conversation` in SocialView.tsx).
  open: (room) => set((s) => ({ active: room, mentions: room ? { ...s.mentions, [room]: 0 } : s.mentions })),

  showCard: (card) => set({ card }),

  load: async (room, older = false) => {
    const thread = get().threads[room];
    const before = older ? thread?.messages[0]?.seq : undefined;
    const page = await host.request<{ messages: Message[]; profiles: Record<string, Profile> }>('messages.history', {
      room,
      ...(before ? { before } : {}),
      limit: 50,
    });
    get().remember(page.profiles);
    set((s) => {
      const held = older ? (s.threads[room]?.messages ?? []) : [];
      const byId = new Map([...held, ...page.messages].map((m) => [m.id, m]));
      return {
        threads: { ...s.threads, [room]: { messages: sorted([...byId.values()]), more: page.messages.length === 50, loaded: true } },
      };
    });
    const last = page.messages[page.messages.length - 1];
    if (!older && last) {
      await host.request('messages.read', { room, seq: last.seq }).catch(() => undefined);
      set((s) => (s.rooms[room] ? { rooms: { ...s.rooms, [room]: { ...s.rooms[room]!, unread: 0 } } } : {}));
    }
  },

  putRoom: (room, role) =>
    set((s) => ({
      rooms: {
        ...s.rooms,
        [room.id]: { ...room, role, unread: 0, last_seq: s.rooms[room.id]?.last_seq ?? 0, members: s.rooms[room.id]?.members ?? 1, other: null },
      },
      invites: s.invites.filter((i) => i.room !== room.id),
    })),

  dropRoom: (room) =>
    set((s) => {
      const rooms = { ...s.rooms };
      delete rooms[room];
      return { rooms, active: s.active === room ? (Object.keys(rooms)[0] ?? null) : s.active };
    }),

  putMessage: (m) =>
    set((s) => {
      const thread = s.threads[m.room];
      if (!thread) return {};
      const others = thread.messages.filter((x) => x.id !== m.id);
      return { threads: { ...s.threads, [m.room]: { ...thread, messages: sorted([...others, m]) } } };
    }),
}));

/** The name a person shows: their display line, else their QOR ID. */
export function shownName(p: Profile | undefined | null): string {
  if (!p) return 'someone';
  return p.display.trim() || p.qor_id;
}

/** A room's title as one member sees it: a direct room is the other person. */
export function roomTitle(room: RoomSummary, profiles: Record<string, Profile>): string {
  if (room.kind === 'direct') return shownName(room.other ? profiles[room.other] : null);
  return room.name;
}

/** `@name` mentions in words, for highlighting. */
export const MENTION = /(^|[^A-Za-z0-9_])@([A-Za-z0-9_]{3,20})/g;
