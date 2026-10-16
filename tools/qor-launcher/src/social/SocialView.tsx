/**
 * Social, the launcher's surface (ADR-091, ADR-092; docs/blueprints/social.md; roadmap L8.5).
 *
 * Three columns: the conversations a person is in (or their friends), the one open, and whoever or whatever they are
 * looking at. Entering never asks for anything already known: no QOR ID, and the name is chosen first; a QOR ID on a name
 * QOR ID made, and a name is chosen once; otherwise World is open, everyone is there, and what they say arrives in the
 * reader's language.
 */

import { Fragment, useEffect, useLayoutEffect, useMemo, useRef, useState, type CSSProperties, type ReactNode } from 'react';
import {
  AtSign,
  Check,
  CornerUpLeft,
  Flag,
  Globe2,
  Hash,
  Languages,
  Loader2,
  Lock,
  LogIn,
  MessageCirclePlus,
  MessagesSquare,
  Pencil,
  Plus,
  Search,
  SendHorizontal,
  Settings2,
  SmilePlus,
  Trash2,
  UserPlus,
  Users,
  VolumeX,
  WifiOff,
  X,
} from 'lucide-react';

import { Avatar } from '../components/chrome/Avatar';
import { identity } from '../lib/ipc';
import { useQor } from '../state/store';
import { Banner, BACKDROPS } from './Banner';
import { FriendsPanel } from './Friends';
import {
  host,
  MENTION,
  readerLanguage,
  refusalOf,
  roomTitle,
  shownName,
  useSocial,
  type Backdrop,
  type Member,
  type Message,
  type Profile,
  type Room,
  type RoomSummary,
} from './model';
import { accentStyle, PresenceDot, ProfileCard, ProfileEditor } from './Profile';
import { keyOf, languageName, request as requestTranslation, translatable, translateHost, useTranslation, wanted } from './translate';

const QUICK_REACTIONS = ['👍', '❤️', '😂', '🔥', '🎉', '👀'];
const NAME = /^[A-Za-z0-9_]{3,20}$/;

/* ─────────────────────────────────── entry ─────────────────────────────── */

export function SocialView() {
  const session = useQor((s) => s.session);
  const status = useSocial((s) => s.status);
  const chosen = useSocial((s) => s.usernameChosen);

  if (!session) return <ClaimName kind="new" />;
  if (status.state !== 'connected') return <Connection />;
  if (!chosen || session.username_chosen === false) return <ClaimName kind="made" current={session.qor_id} />;
  return <Social />;
}

function Shell({ children }: { children: ReactNode }) {
  return <div className="grid h-full place-items-center p-8">{children}</div>;
}

function Connection() {
  const status = useSocial((s) => s.status);
  const go = useQor((s) => s.go);
  const [endpoint, setEndpoint] = useState('');
  useEffect(() => {
    host.status().then((s) => setEndpoint(s.endpoint)).catch(() => undefined);
  }, []);
  const said: Record<string, { title: string; body: string }> = {
    off: { title: 'Connecting…', body: 'Opening your connection to Social.' },
    connecting: { title: 'Connecting…', body: 'Opening your connection to Social.' },
    retrying: {
      title: 'Social is not reachable',
      body: `Trying again in ${'in_secs' in status ? status.in_secs : 1} s. ${'reason' in status ? status.reason : ''}`,
    },
    signed_out: { title: 'Sign in again', body: 'QOR ID says this session has ended. Sign in from Settings, and Social reconnects.' },
    refused: { title: 'Social refused this launcher', body: 'reason' in status ? status.reason : '' },
  };
  const now = said[status.state] ?? said.off!;
  return (
    <Shell>
      <div className="glass-solid cut w-[26rem] p-7 text-center" role="status" aria-live="polite" data-social-connection={status.state}>
        {status.state === 'connecting' || status.state === 'off' ? (
          <Loader2 size={22} className="mx-auto mb-4 animate-spin text-accent" />
        ) : (
          <WifiOff size={22} className="mx-auto mb-4 text-ink-muted" />
        )}
        <p className="heading mb-2 text-title">{now.title}</p>
        <p className="text-ui text-ink-body">{now.body}</p>
        {endpoint && <p className="numeric mt-4 truncate text-micro text-ink-faint">{endpoint}</p>}
        {status.state === 'signed_out' && (
          <button type="button" className="btn mt-5" onClick={() => go('settings')}>
            <LogIn size={13} /> Settings
          </button>
        )}
      </div>
    </Shell>
  );
}

/** Choosing the QOR ID name: before there is a QOR ID (`new`), or once, instead of a made one (`made`). */
function ClaimName({ kind, current }: { kind: 'new' | 'made'; current?: string }) {
  const accounts = useQor((s) => s.accounts);
  const setSession = useQor((s) => s.setSession);
  const notify = useQor((s) => s.notify);
  const [name, setName] = useState('');
  const [state, setState] = useState<'idle' | 'invalid' | 'checking' | 'free' | 'taken' | 'offline'>('idle');
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const asked = useRef(0);

  useEffect(() => {
    setProblem(null);
    if (!name) return setState('idle');
    if (!NAME.test(name)) return setState('invalid');
    setState('checking');
    const ticket = ++asked.current;
    const id = setTimeout(() => {
      identity
        .usernameAvailable(name)
        .then((free) => ticket === asked.current && setState(free ? 'free' : 'taken'))
        .catch(() => ticket === asked.current && setState('offline'));
    }, 350);
    return () => clearTimeout(id);
  }, [name]);

  const claim = async () => {
    setBusy(true);
    setProblem(null);
    try {
      if (kind === 'made') {
        const session = (await host.claimUsername(name)) as Parameters<typeof setSession>[0];
        setSession(session);
        // Social knew this person by their made name: say hello again under the chosen one.
        await host.disconnect();
        await host.connect();
        notify('ok', `You are ${name} now, everywhere your QOR ID goes.`);
      } else {
        const arrival = await identity.signIn();
        if (arrival.session) return setSession(arrival.session);
        if (!arrival.needs_name) throw new Error(arrival.sign_in_problem ?? 'QOR ID could not be reached.');
        const address = accounts[0]?.address;
        if (!address) throw new Error('No account is available to claim a name with.');
        setSession(await identity.registerWithKey(address, name));
      }
    } catch (e) {
      setProblem(refusalOf(e).message);
    } finally {
      setBusy(false);
    }
  };

  const line: Record<typeof state, { text: string; tone: string }> = {
    idle: { text: '3 to 20 letters, numbers or _. It is your name everywhere: no number is added.', tone: 'text-ink-muted' },
    invalid: { text: 'Only letters, numbers and _, from 3 to 20 of them.', tone: 'text-bad' },
    checking: { text: 'Checking…', tone: 'text-ink-muted' },
    free: { text: `${name} is yours to take.`, tone: 'text-ok' },
    taken: { text: `${name} is taken. Try another.`, tone: 'text-bad' },
    offline: { text: 'QOR ID is not reachable right now.', tone: 'text-bad' },
  };

  return (
    <Shell>
      <div className="glass-solid cut w-[28rem] overflow-hidden p-0" data-social-claim={kind}>
        <Banner backdrop={{ preset: 'aurora', hue: 255, intensity: 0.7 }} className="h-24" />
        <div className="p-7">
          <p className="eyebrow mb-1 text-accent">Social · QOR ID</p>
          <h1 className="heading mb-2 text-heading">{kind === 'new' ? 'Choose your name' : 'Make your name your own'}</h1>
          <p className="mb-5 text-ui text-ink-body">
            {kind === 'new'
              ? 'Your QOR ID is the name people find you by, here and in every Demiurge product.'
              : `Your account was given the name ${current ?? 'it has'} automatically. Choose the one you want, once: after that it is yours to keep.`}
          </p>
          <input
            className={`field mb-2 ${state === 'taken' || state === 'invalid' ? 'field-invalid' : ''}`}
            placeholder="your_name"
            aria-label="QOR ID name"
            autoFocus
            autoComplete="off"
            spellCheck={false}
            value={name}
            onChange={(e) => setName(e.target.value.trim())}
            onKeyDown={(e) => e.key === 'Enter' && state === 'free' && !busy && void claim()}
          />
          <p className={`mb-5 min-h-[1.25rem] text-caption ${line[state].tone}`} aria-live="polite">
            {line[state].text}
          </p>
          <button type="button" className="btn btn-primary w-full" disabled={state !== 'free' || busy} onClick={() => void claim()}>
            {busy ? <Loader2 size={14} className="animate-spin" /> : <Check size={14} />} Claim {name || 'my name'}
          </button>
          {problem && (
            <p role="alert" className="mt-3 text-caption text-bad">
              {problem}
            </p>
          )}
        </div>
      </div>
    </Shell>
  );
}

/* ─────────────────────────────────── layout ───────────────────────────── */

type Dialog = 'new-room' | 'discover' | 'new-dm' | 'invite' | 'room-settings' | 'edit-profile' | null;

function Social() {
  const active = useSocial((s) => s.active);
  const card = useSocial((s) => s.card);
  const world = useSocial((s) => s.world);
  const [dialog, setDialog] = useState<Dialog>(null);
  const [panel, setPanel] = useState<'members' | null>(null);

  // Opening Social with nothing chosen opens World: everyone is entered into it (ADR-092 decision 1).
  useEffect(() => {
    if (!active && world) useSocial.getState().open(world);
  }, [active, world]);

  return (
    <div className="flex h-full min-h-0" data-social>
      <RoomList onDialog={setDialog} />
      <section className="flex min-w-0 flex-1 flex-col border-l border-edge" aria-label="Conversation">
        {active ? (
          <Conversation key={active} room={active} onDialog={setDialog} panel={panel} onPanel={setPanel} />
        ) : (
          <Empty onDialog={setDialog} />
        )}
      </section>
      {(card || (panel && active)) && (
        <aside className="w-80 flex-none overflow-y-auto border-l border-edge bg-void/30" aria-label={card ? 'Profile' : 'Members'}>
          <div className="flex justify-end p-2">
            <button
              type="button"
              aria-label="Close panel"
              className="p-1 text-ink-muted hover:text-ink"
              onClick={() => (card ? useSocial.getState().showCard(null) : setPanel(null))}
            >
              <X size={15} />
            </button>
          </div>
          {card ? <ProfileCard sub={card} onEdit={() => setDialog('edit-profile')} /> : active && <Members room={active} />}
        </aside>
      )}
      {dialog === 'new-room' && <NewRoom onClose={() => setDialog(null)} />}
      {dialog === 'discover' && <Discover onClose={() => setDialog(null)} />}
      {dialog === 'new-dm' && <NewDirect onClose={() => setDialog(null)} />}
      {dialog === 'invite' && active && <Invite room={active} onClose={() => setDialog(null)} />}
      {dialog === 'room-settings' && active && <RoomSettings room={active} onClose={() => setDialog(null)} />}
      {dialog === 'edit-profile' && <ProfileEditor onClose={() => setDialog(null)} />}
    </div>
  );
}

function Empty({ onDialog }: { onDialog: (d: Dialog) => void }) {
  const me = useSocial((s) => s.me);
  return (
    <div className="grid flex-1 place-items-center p-10 text-center">
      <div className="max-w-sm">
        <MessagesSquare size={28} className="mx-auto mb-4 text-accent" />
        <p className="heading mb-2 text-title">Welcome, {shownName(me)}</p>
        <p className="mb-6 text-ui text-ink-body">Find a room to join, start your own, or message someone by their QOR ID.</p>
        <div className="flex flex-wrap justify-center gap-2">
          <button type="button" className="btn btn-primary" onClick={() => onDialog('discover')}>
            <Search size={13} /> Find a room
          </button>
          <button type="button" className="btn" onClick={() => onDialog('new-room')}>
            <Plus size={13} /> New room
          </button>
          <button type="button" className="btn" onClick={() => onDialog('new-dm')}>
            <MessageCirclePlus size={13} /> Message someone
          </button>
        </div>
      </div>
    </div>
  );
}

/* ──────────────────────────────── room list ───────────────────────────── */

function RoomList({ onDialog }: { onDialog: (d: Dialog) => void }) {
  const me = useSocial((s) => s.me);
  const rooms = useSocial((s) => s.rooms);
  const invites = useSocial((s) => s.invites);
  const profiles = useSocial((s) => s.profiles);
  const presence = useSocial((s) => s.presence);
  const active = useSocial((s) => s.active);
  const mentions = useSocial((s) => s.mentions);
  const open = useSocial((s) => s.open);
  const showCard = useSocial((s) => s.showCard);
  const asking = useSocial((s) => s.incoming.length);
  const [filter, setFilter] = useState('');
  const [tab, setTab] = useState<'chats' | 'friends'>('chats');

  const all = Object.values(rooms).filter((r) => roomTitle(r, profiles).toLowerCase().includes(filter.toLowerCase()));
  const byActivity = (a: RoomSummary, b: RoomSummary) => b.last_seq - a.last_seq || a.created_at.localeCompare(b.created_at);
  const world = Object.values(rooms).find((r) => r.kind === 'world');
  const direct = all.filter((r) => r.kind === 'direct').sort(byActivity);
  const groups = all.filter((r) => r.kind !== 'direct' && r.kind !== 'world').sort(byActivity);

  return (
    <nav className="flex w-72 flex-none flex-col" aria-label="Conversations">
      <button
        type="button"
        className="flex items-center gap-3 border-b border-edge px-4 py-4 text-left hover:bg-raised"
        onClick={() => me && showCard(me.sub)}
        style={accentStyle(me)}
        aria-label="Your profile"
      >
        <Avatar name={me?.qor_id} src={me?.avatar_url ?? null} size={36} level={me?.level ?? null} />
        <span className="min-w-0 flex-1">
          <span className="social-accent block truncate text-ui font-semibold">{shownName(me)}</span>
          <span className="block truncate text-micro text-ink-muted">{me?.status_line || `@${me?.qor_id ?? ''}`}</span>
        </span>
        <PresenceDot presence={me ? presence[me.sub] : undefined} />
      </button>

      <div className="social-tabs mx-3 mt-3" role="tablist" aria-label="Conversations or friends">
        {(['chats', 'friends'] as const).map((t) => (
          <button key={t} type="button" role="tab" aria-selected={tab === t} className={tab === t ? 'social-tab social-tab-on' : 'social-tab'} onClick={() => setTab(t)}>
            {t === 'chats' ? <MessagesSquare size={13} /> : <Users size={13} />}
            {t === 'chats' ? 'Chats' : 'Friends'}
            {t === 'friends' && asking > 0 && (
              <span className="numeric social-tab-count" aria-label={`${asking} asking to be friends`}>
                {asking}
              </span>
            )}
          </button>
        ))}
      </div>

      {tab === 'friends' ? (
        <FriendsPanel />
      ) : (
      <>
      <div className="flex gap-1 px-3 pt-3">
        <label className="relative min-w-0 flex-1">
          <Search size={12} className="pointer-events-none absolute left-2.5 top-1/2 -translate-y-1/2 text-ink-faint" />
          <input className="field py-1.5 pl-7 text-ui" placeholder="Filter" aria-label="Filter conversations" value={filter} onChange={(e) => setFilter(e.target.value)} />
        </label>
        <button type="button" className="btn btn-ghost px-2" title="Find a room" aria-label="Find a room" onClick={() => onDialog('discover')}>
          <Globe2 size={14} />
        </button>
        <button type="button" className="btn btn-ghost px-2" title="New room" aria-label="New room" onClick={() => onDialog('new-room')}>
          <Plus size={14} />
        </button>
        <button type="button" className="btn btn-ghost px-2" title="Message someone" aria-label="Message someone" onClick={() => onDialog('new-dm')}>
          <MessageCirclePlus size={14} />
        </button>
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-4 pt-2">
        {world && <WorldRow room={world} active={active === world.id} mentions={mentions[world.id] ?? 0} onOpen={() => open(world.id)} />}
        {invites.length > 0 && (
          <Section title="Invitations">
            {invites.map((i) => (
              <InviteRow key={i.room} room={i.room} name={i.name} inviter={profiles[i.inviter]} />
            ))}
          </Section>
        )}
        <Section title="Direct">
          {direct.length === 0 && <p className="px-2 py-1 text-micro text-ink-faint">No conversations yet.</p>}
          {direct.map((r) => (
            <RoomRow
              key={r.id}
              room={r}
              title={roomTitle(r, profiles)}
              active={active === r.id}
              mentions={mentions[r.id] ?? 0}
              onOpen={() => open(r.id)}
              leading={
                <span className="relative">
                  <Avatar name={r.other ? profiles[r.other]?.qor_id : null} src={r.other ? (profiles[r.other]?.avatar_url ?? null) : null} size={26} />
                  <PresenceDot presence={r.other ? presence[r.other] : undefined} className="absolute -bottom-0.5 -right-0.5 ring-2 ring-void" />
                </span>
              }
            />
          ))}
        </Section>
        <Section title="Rooms">
          {groups.length === 0 && <p className="px-2 py-1 text-micro text-ink-faint">Not in any room yet.</p>}
          {groups.map((r) => (
            <RoomRow
              key={r.id}
              room={r}
              title={r.name}
              active={active === r.id}
              mentions={mentions[r.id] ?? 0}
              onOpen={() => open(r.id)}
              leading={
                <span className="grid h-[26px] w-[26px] place-items-center border border-edge bg-well text-ink-muted">
                  {r.kind === 'private' ? <Lock size={12} /> : <Hash size={12} />}
                </span>
              }
            />
          ))}
        </Section>
      </div>
      </>
      )}
    </nav>
  );
}

/** World, pinned above everything: one room everyone is in (ADR-092 decision 1). */
function WorldRow({ room, active, mentions, onOpen }: { room: RoomSummary; active: boolean; mentions: number; onOpen: () => void }) {
  return (
    <button
      type="button"
      onClick={onOpen}
      aria-current={active ? 'true' : undefined}
      data-room-row={room.id}
      data-world-row
      className={`social-world-row mb-2 flex w-full items-center gap-3 overflow-hidden px-3 py-2.5 text-left ${active ? 'social-world-row-on' : ''}`}
    >
      <Banner backdrop={{ preset: 'aurora', hue: 200, intensity: 0.55 }} className="absolute inset-0 opacity-60" />
      <span className="social-world-globe relative">
        <Globe2 size={16} />
      </span>
      <span className="relative min-w-0 flex-1">
        <span className="block text-ui font-semibold text-ink">World</span>
        <span className="block truncate text-micro text-ink-body">Everyone on QOR, in every language</span>
      </span>
      {mentions > 0 && (
        <span className="numeric relative flex items-center gap-0.5 bg-accent-bright px-1.5 text-micro font-semibold text-void" aria-label={`${mentions} mentions`}>
          <AtSign size={9} />
          {mentions}
        </span>
      )}
      {room.unread > 0 && mentions === 0 && <span className="social-world-pulse relative" aria-label="New messages" />}
    </button>
  );
}

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="mb-3">
      <p className="eyebrow px-2 pb-1 pt-2">{title}</p>
      <ul className="flex flex-col gap-0.5">{children}</ul>
    </section>
  );
}

function RoomRow({
  room,
  title,
  active,
  mentions,
  leading,
  onOpen,
}: {
  room: RoomSummary;
  title: string;
  active: boolean;
  mentions: number;
  leading: ReactNode;
  onOpen: () => void;
}) {
  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        aria-current={active ? 'true' : undefined}
        data-room-row={room.id}
        className={`flex w-full items-center gap-2.5 px-2 py-1.5 text-left transition-colors ${active ? 'bg-surface text-ink' : 'text-ink-body hover:bg-raised'}`}
      >
        {leading}
        <span className={`min-w-0 flex-1 truncate text-ui ${room.unread > 0 ? 'font-semibold text-ink' : ''}`}>{title}</span>
        {mentions > 0 && (
          <span className="numeric flex items-center gap-0.5 bg-accent-bright px-1.5 text-micro font-semibold text-void" aria-label={`${mentions} mentions`}>
            <AtSign size={9} />
            {mentions}
          </span>
        )}
        {room.unread > 0 && (
          <span className="numeric min-w-5 bg-accent px-1.5 text-center text-micro font-semibold text-void" aria-label={`${room.unread} unread`}>
            {room.unread > 99 ? '99+' : room.unread}
          </span>
        )}
      </button>
    </li>
  );
}

function InviteRow({ room, name, inviter }: { room: string; name: string; inviter: Profile | undefined }) {
  const putRoom = useSocial((s) => s.putRoom);
  const open = useSocial((s) => s.open);
  const notify = useQor((s) => s.notify);
  const answer = async (join: boolean) => {
    try {
      if (join) {
        const r = await host.request<{ room: Room }>('rooms.join', { room });
        putRoom(r.room, 'member');
        open(r.room.id);
      } else {
        await host.request('rooms.decline', { room });
        useSocial.setState((s) => ({ invites: s.invites.filter((i) => i.room !== room) }));
      }
    } catch (e) {
      notify('bad', refusalOf(e).message);
    }
  };
  return (
    <li className="border border-accent-dim bg-raised/60 p-2.5" data-invite={room}>
      <p className="truncate text-ui text-ink">
        <Lock size={11} className="mr-1 inline text-accent" />
        {name}
      </p>
      <p className="mb-2 truncate text-micro text-ink-muted">from {shownName(inviter)}</p>
      <div className="flex gap-1.5">
        <button type="button" className="btn btn-primary px-3 py-1" onClick={() => void answer(true)}>
          Join
        </button>
        <button type="button" className="btn btn-ghost px-3 py-1" onClick={() => void answer(false)}>
          Decline
        </button>
      </div>
    </li>
  );
}

/* ──────────────────────────────── conversation ────────────────────────── */

function Conversation({
  room: roomId,
  onDialog,
  panel,
  onPanel,
}: {
  room: string;
  onDialog: (d: Dialog) => void;
  panel: 'members' | null;
  onPanel: (p: 'members' | null) => void;
}) {
  const room = useSocial((s) => s.rooms[roomId]);
  const thread = useSocial((s) => s.threads[roomId]);
  const profiles = useSocial((s) => s.profiles);
  const me = useSocial((s) => s.me);
  const load = useSocial((s) => s.load);
  const scroller = useRef<HTMLDivElement>(null);
  const [reply, setReply] = useState<Message | null>(null);
  const [editing, setEditing] = useState<Message | null>(null);
  const [loadingOlder, setLoadingOlder] = useState(false);
  const stuck = useRef(true);

  useEffect(() => {
    if (!thread?.loaded) void load(roomId).catch(() => undefined);
  }, [roomId, thread?.loaded, load]);

  // Follow the newest message while the reader is at the bottom; leave them be when they have scrolled up.
  useLayoutEffect(() => {
    const el = scroller.current;
    if (el && stuck.current) el.scrollTop = el.scrollHeight;
  }, [thread?.messages.length]);

  if (!room) return null;
  const title = roomTitle(room, profiles);
  const other = room.kind === 'direct' && room.other ? profiles[room.other] : null;
  const isWorld = room.kind === 'world';
  const canModerate = room.role !== 'member' && room.kind !== 'direct' && !isWorld;

  const older = async () => {
    const el = scroller.current;
    const before = el?.scrollHeight ?? 0;
    setLoadingOlder(true);
    await load(roomId, true).catch(() => undefined);
    setLoadingOlder(false);
    if (el) el.scrollTop = el.scrollHeight - before;
  };

  return (
    <>
      <header className="relative flex-none overflow-hidden border-b border-edge">
        <Banner
          backdrop={room.kind === 'direct' ? other?.banner : isWorld ? { preset: 'aurora', hue: 200, intensity: 0.7 } : room.ambience}
          className="absolute inset-0 opacity-70"
        />
        <div className="relative flex items-center gap-3 px-6 py-4">
          {isWorld ? (
            <span className="social-world-globe social-world-globe-lg">
              <Globe2 size={18} />
            </span>
          ) : room.kind === 'direct' ? (
            <button type="button" onClick={() => other && useSocial.getState().showCard(other.sub)} aria-label={`${title}'s profile`}>
              <Avatar name={other?.qor_id} src={other?.avatar_url ?? null} size={34} level={other?.level ?? null} />
            </button>
          ) : (
            <span className="grid h-9 w-9 place-items-center border border-edge bg-void/60 text-accent">
              {room.kind === 'private' ? <Lock size={15} /> : <Hash size={15} />}
            </span>
          )}
          <div className="min-w-0 flex-1">
            <h1 className="truncate text-title font-semibold text-ink" style={accentStyle(other)}>
              <span className={other ? 'social-accent' : ''}>{title}</span>
            </h1>
            <p className="truncate text-caption text-ink-body">
              {room.kind === 'direct' ? `@${other?.qor_id ?? ''}${other?.status_line ? ` · ${other.status_line}` : ''}` : room.topic || `${room.members} in this room`}
            </p>
          </div>
          {translatable(room) && <TranslateSwitch />}
          {room.kind !== 'direct' && (
            <>
              {room.kind === 'public' || canModerate ? (
                <button type="button" className="btn btn-ghost px-2" title="Invite someone" aria-label="Invite someone" onClick={() => onDialog('invite')}>
                  <UserPlus size={15} />
                </button>
              ) : null}
              {canModerate && (
                <button type="button" className="btn btn-ghost px-2" title="Room settings" aria-label="Room settings" onClick={() => onDialog('room-settings')}>
                  <Settings2 size={15} />
                </button>
              )}
              <button
                type="button"
                className={`btn btn-ghost px-2 ${panel === 'members' ? 'text-accent' : ''}`}
                title={isWorld ? 'Who is here' : 'Members'}
                aria-label={isWorld ? 'Who is here' : 'Members'}
                aria-pressed={panel === 'members'}
                onClick={() => {
                  useSocial.getState().showCard(null);
                  onPanel(panel === 'members' ? null : 'members');
                }}
              >
                <Users size={15} />
              </button>
            </>
          )}
        </div>
      </header>
      {translatable(room) && <TranslationBar />}

      <div
        ref={scroller}
        className="min-h-0 flex-1 overflow-y-auto px-6 py-4"
        role="log"
        aria-label={`Messages in ${title}`}
        aria-live="polite"
        onScroll={(e) => {
          const el = e.currentTarget;
          stuck.current = el.scrollHeight - el.scrollTop - el.clientHeight < 40;
        }}
      >
        {!thread?.loaded ? (
          <p className="text-caption text-ink-muted">
            <Loader2 size={12} className="mr-1 inline animate-spin" /> Reading the conversation…
          </p>
        ) : (
          <>
            {thread.more && (
              <button type="button" className="btn btn-ghost mx-auto mb-4 flex" disabled={loadingOlder} onClick={() => void older()}>
                {loadingOlder ? <Loader2 size={13} className="animate-spin" /> : null} Earlier messages
              </button>
            )}
            {thread.messages.length === 0 && (
              <p className="mt-10 text-center text-ui text-ink-muted">
                {room.kind === 'direct'
                  ? `This is the start of your conversation with ${title}.`
                  : isWorld
                    ? 'This is the start of World. Say hello to everyone, in any language.'
                    : `This is the start of ${title}. Say hello.`}
              </p>
            )}
            {thread.messages.map((m, i) => {
              const prev = thread.messages[i - 1];
              const grouped =
                !!prev && prev.author === m.author && !prev.deleted && new Date(m.created_at).getTime() - new Date(prev.created_at).getTime() < 5 * 60_000;
              const day = new Date(m.created_at).toDateString();
              const newDay = !prev || new Date(prev.created_at).toDateString() !== day;
              return (
                <Fragment key={m.id}>
                  {newDay && <DayRule date={m.created_at} />}
                  <MessageItem
                    message={m}
                    grouped={grouped && !newDay}
                    room={room}
                    me={me}
                    replyTo={m.reply_to ? thread.messages.find((x) => x.id === m.reply_to) : undefined}
                    onReply={() => setReply(m)}
                    onEdit={() => setEditing(m)}
                  />
                </Fragment>
              );
            })}
          </>
        )}
      </div>

      <Typing room={roomId} />
      <Composer
        room={room}
        title={title}
        reply={reply}
        editing={editing}
        onDone={() => {
          setReply(null);
          setEditing(null);
          stuck.current = true;
        }}
      />
    </>
  );
}

function DayRule({ date }: { date: string }) {
  const d = new Date(date);
  const today = new Date().toDateString() === d.toDateString();
  return (
    <div className="my-4 flex items-center gap-3" role="separator">
      <span className="h-px flex-1 bg-edge" />
      <span className="eyebrow">{today ? 'Today' : d.toLocaleDateString(undefined, { weekday: 'long', month: 'long', day: 'numeric' })}</span>
      <span className="h-px flex-1 bg-edge" />
    </div>
  );
}

/** Words with `@mentions` marked, and line breaks kept. */
function Words({ body, me }: { body: string; me: string | undefined }) {
  const parts: ReactNode[] = [];
  let last = 0;
  for (const match of body.matchAll(MENTION)) {
    const at = (match.index ?? 0) + (match[1]?.length ?? 0);
    parts.push(body.slice(last, at));
    const name = match[2] ?? '';
    const mine = me !== undefined && name.toLowerCase() === me.toLowerCase();
    parts.push(
      <span key={at} className={`social-mention ${mine ? 'social-mention-me' : ''}`}>
        @{name}
      </span>,
    );
    last = at + name.length + 1;
  }
  parts.push(body.slice(last));
  return <span className="whitespace-pre-wrap break-words">{parts}</span>;
}

function MessageItem({
  message: m,
  grouped,
  room,
  me,
  replyTo,
  onReply,
  onEdit,
}: {
  message: Message;
  grouped: boolean;
  room: RoomSummary;
  me: Profile | null;
  replyTo: Message | undefined;
  onReply: () => void;
  onEdit: () => void;
}) {
  const author = useSocial((s) => s.profiles[m.author]);
  const replyAuthor = useSocial((s) => (replyTo ? s.profiles[replyTo.author] : undefined));
  const notify = useQor((s) => s.notify);
  const [picking, setPicking] = useState(false);
  const [muting, setMuting] = useState(false);
  const mine = me?.sub === m.author;
  const canDelete = mine || (room.role !== 'member' && room.kind !== 'direct');
  const canMute = room.kind === 'world' && room.role !== 'member' && !mine;
  const reader = readerLanguage(me);
  const on = useTranslation((s) => s.on);
  const translating = on && wanted(m, room, reader);
  const key = translating && reader ? keyOf(m, reader) : '';
  const translated = useTranslation((s) => (key ? s.done[key] : undefined));
  const failed = useTranslation((s) => (key ? !!s.failed[key] : false));
  const original = useTranslation((s) => !!s.original[m.id]);
  useEffect(() => {
    if (translating && reader) requestTranslation(m, reader);
  }, [translating, reader, m]);
  const showTranslated = translating && translated !== undefined && !original;
  const time = new Date(m.created_at).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });

  const act = async (method: string, params: Record<string, unknown>, ok?: string) => {
    try {
      const r = await host.request<{ message?: Message }>(method, params);
      if (r.message) useSocial.getState().putMessage(r.message);
      if (ok) notify('ok', ok);
    } catch (e) {
      notify('bad', refusalOf(e).message);
    }
  };

  return (
    <div
      className={`social-message group relative -mx-3 flex gap-3 px-3 ${grouped ? 'py-0.5' : 'mt-3 py-1'} hover:bg-raised/40`}
      style={accentStyle(author)}
      data-message={m.id}
    >
      <div className="w-9 flex-none">
        {!grouped && (
          <button type="button" onClick={() => useSocial.getState().showCard(m.author)} aria-label={`${shownName(author)}'s profile`}>
            <Avatar name={author?.qor_id} src={author?.avatar_url ?? null} size={36} level={author?.level ?? null} />
          </button>
        )}
      </div>
      <div className="min-w-0 flex-1">
        {!grouped && (
          <p className="flex min-w-0 items-baseline gap-2">
            <button type="button" className="social-accent truncate text-ui font-semibold hover:underline" onClick={() => useSocial.getState().showCard(m.author)}>
              {shownName(author)}
            </button>
            {author && (author.display.trim() ? <span className="numeric truncate text-micro text-ink-muted">@{author.qor_id}</span> : null)}
            {author && (
              <span className="social-level numeric" title={`Level ${author.level}`} aria-label={`Level ${author.level}`}>
                Lv {author.level}
              </span>
            )}
            <span className="numeric text-micro text-ink-faint">{time}</span>
          </p>
        )}
        {replyTo && (
          <p className="mb-0.5 flex items-center gap-1.5 truncate text-caption text-ink-muted">
            <CornerUpLeft size={11} />
            <span className="font-semibold">{shownName(replyAuthor)}</span>
            <span className="truncate">{replyTo.body ?? 'a deleted message'}</span>
          </p>
        )}
        <div className={`social-rail pl-3 text-body ${m.deleted ? 'italic text-ink-faint' : 'text-ink-body'} selectable`} lang={showTranslated ? (reader ?? undefined) : (m.lang ?? undefined)}>
          {m.deleted ? 'This message was deleted.' : <Words body={showTranslated ? translated! : (m.body ?? '')} me={me?.qor_id} />}
          {m.edited_at && !m.deleted && <span className="ml-1.5 text-micro text-ink-faint">(edited)</span>}
        </div>
        {translating && (
          <p className="social-translated mt-0.5 pl-3" data-translated={showTranslated ? 'yes' : 'no'}>
            <Languages size={11} aria-hidden="true" />
            {translated === undefined && !failed ? (
              <span className="social-translating">Translating from {languageName(m.lang, reader)}…</span>
            ) : failed ? (
              <span>Written in {languageName(m.lang, reader)}; it could not be translated here.</span>
            ) : (
              <>
                <span>{original ? `Written in ${languageName(m.lang, reader)}` : `Translated from ${languageName(m.lang, reader)}`}</span>
                <button type="button" className="social-translated-toggle" onClick={() => useTranslation.getState().toggleOriginal(m.id)}>
                  {original ? 'Show translation' : 'Show original'}
                </button>
              </>
            )}
          </p>
        )}
        {m.reactions.length > 0 && (
          <div className="mt-1 flex flex-wrap gap-1 pl-3">
            {m.reactions.map((r) => {
              const pressed = !!me && r.by.includes(me.sub);
              return (
                <button
                  key={r.emoji}
                  type="button"
                  aria-pressed={pressed}
                  aria-label={`${r.emoji} ${r.by.length}`}
                  className={`flex items-center gap-1 border px-1.5 text-caption ${pressed ? 'border-accent bg-accent/15 text-ink' : 'border-edge text-ink-body hover:border-accent-dim'}`}
                  onClick={() => void act('messages.react', { message: m.id, emoji: r.emoji })}
                >
                  <span>{r.emoji}</span>
                  <span className="numeric">{r.by.length}</span>
                </button>
              );
            })}
          </div>
        )}
      </div>

      {!m.deleted && (
        <div className="social-actions absolute -top-3 right-3 flex border border-edge bg-raised shadow-lg">
          <button type="button" className="p-1.5 text-ink-muted hover:text-ink" title="React" aria-label="React" onClick={() => setPicking((p) => !p)}>
            <SmilePlus size={14} />
          </button>
          <button type="button" className="p-1.5 text-ink-muted hover:text-ink" title="Reply" aria-label="Reply" onClick={onReply}>
            <CornerUpLeft size={14} />
          </button>
          {mine && (
            <button type="button" className="p-1.5 text-ink-muted hover:text-ink" title="Edit" aria-label="Edit" onClick={onEdit}>
              <Pencil size={14} />
            </button>
          )}
          {canDelete && (
            <button type="button" className="p-1.5 text-ink-muted hover:text-bad" title="Delete" aria-label="Delete" onClick={() => void act('messages.delete', { message: m.id })}>
              <Trash2 size={14} />
            </button>
          )}
          {canMute && (
            <button type="button" className="p-1.5 text-ink-muted hover:text-bad" title="Mute in World" aria-label="Mute in World" onClick={() => setMuting((x) => !x)}>
              <VolumeX size={14} />
            </button>
          )}
          {!mine && (
            <button
              type="button"
              className="p-1.5 text-ink-muted hover:text-bad"
              title="Report"
              aria-label="Report"
              onClick={() => void act('report', { message: m.id, reason: 'other' }, 'Reported. The room’s moderators and Social’s operators can see it.')}
            >
              <Flag size={14} />
            </button>
          )}
        </div>
      )}
      {muting && (
        <div className="absolute right-3 top-7 z-10 flex items-center gap-1 border border-edge bg-raised p-1 shadow-lg" role="menu" aria-label="Mute in World">
          <span className="eyebrow px-1.5">Mute {shownName(author)} for</span>
          {(
            [
              [10, '10 min'],
              [60, '1 hour'],
              [1440, '1 day'],
              [10080, '1 week'],
            ] as const
          ).map(([minutes, label]) => (
            <button
              key={minutes}
              type="button"
              role="menuitem"
              className="px-2 py-1 text-ui text-ink-body hover:bg-base hover:text-ink"
              onClick={() => {
                setMuting(false);
                void act('world.mute', { sub: m.author, minutes }, `${shownName(author)} is muted in World for ${label}.`);
              }}
            >
              {label}
            </button>
          ))}
        </div>
      )}
      {picking && (
        <div className="absolute right-3 top-7 z-10 flex gap-0.5 border border-edge bg-raised p-1 shadow-lg" role="menu" aria-label="Reactions">
          {QUICK_REACTIONS.map((e) => (
            <button
              key={e}
              type="button"
              role="menuitem"
              className="p-1 text-title hover:bg-base"
              onClick={() => {
                setPicking(false);
                void act('messages.react', { message: m.id, emoji: e });
              }}
            >
              {e}
            </button>
          ))}
        </div>
      )}
    </div>
  );
}

/** Translation on or off for the reader, and the language they read in (ADR-092 decision 3). */
function TranslateSwitch() {
  const on = useTranslation((s) => s.on);
  const me = useSocial((s) => s.me);
  const reader = readerLanguage(me);
  return (
    <button
      type="button"
      className={`social-translate-switch ${on ? 'social-translate-switch-on' : ''}`}
      aria-pressed={on}
      title={on ? `Shown in ${languageName(reader, reader)}. Click to show messages as written.` : 'Click to show messages in your language'}
      onClick={() => useTranslation.getState().setOn(!on)}
      data-translate-switch
    >
      <Languages size={13} />
      <span className="truncate">{on ? languageName(reader, reader) : 'As written'}</span>
    </button>
  );
}

/** What the reader's PC is fetching to translate, and anything that went wrong. */
function TranslationBar() {
  const downloading = useTranslation((s) => s.downloading);
  const problem = useTranslation((s) => s.problem);
  const me = useSocial((s) => s.me);
  const reader = readerLanguage(me);
  if (!downloading && !problem) return null;
  if (downloading) {
    const mb = (n: number) => (n / 1_000_000).toFixed(n < 10_000_000 ? 1 : 0);
    const share = downloading.total ? Math.round((downloading.done / downloading.total) * 100) : 0;
    return (
      <div className="social-translate-bar" role="status" aria-live="polite" data-translate-download>
        <Loader2 size={12} className="animate-spin" />
        <span className="min-w-0 flex-1 truncate">
          Setting up {languageName(downloading.from, reader)} → {languageName(downloading.to, reader)} on this PC, once ·{' '}
          <span className="numeric">
            {mb(downloading.done)} of {mb(downloading.total)} MB
          </span>
        </span>
        <span className="social-translate-meter" aria-hidden="true">
          <span style={{ width: `${share}%` }} />
        </span>
      </div>
    );
  }
  return (
    <div className="social-translate-bar social-translate-bar-bad" role="alert">
      <Languages size={12} />
      <span className="min-w-0 flex-1 truncate">{problem}</span>
      <button type="button" aria-label="Dismiss" className="text-ink-muted hover:text-ink" onClick={() => useTranslation.setState({ problem: null })}>
        <X size={12} />
      </button>
    </div>
  );
}

function Typing({ room }: { room: string }) {
  const typing = useSocial((s) => s.typing[room]);
  const profiles = useSocial((s) => s.profiles);
  const [, tick] = useState(0);
  useEffect(() => {
    const id = window.setInterval(() => tick((n) => n + 1), 1000);
    return () => window.clearInterval(id);
  }, []);
  const now = Date.now();
  const who = Object.entries(typing ?? {})
    .filter(([, until]) => until > now)
    .map(([sub]) => shownName(profiles[sub]));
  return (
    <div className="h-6 flex-none px-6 text-caption text-ink-muted" aria-live="polite">
      {who.length > 0 && (
        <span className="social-typing inline-flex items-center gap-2">
          <span />
          <span />
          <span />
          {who.length === 1 ? `${who[0]} is typing` : `${who.slice(0, 2).join(' and ')} are typing`}
        </span>
      )}
    </div>
  );
}

function Composer({
  room,
  title,
  reply,
  editing,
  onDone,
}: {
  room: RoomSummary;
  title: string;
  reply: Message | null;
  editing: Message | null;
  onDone: () => void;
}) {
  const profiles = useSocial((s) => s.profiles);
  const me = useSocial((s) => s.me);
  const mutedUntil = useSocial((s) => (room.kind === 'world' ? s.mutedUntil : null));
  const notify = useQor((s) => s.notify);
  const [text, setText] = useState('');
  const [busy, setBusy] = useState(false);
  const [members, setMembers] = useState<string[]>([]);
  const muted = !!mutedUntil && new Date(mutedUntil).getTime() > Date.now();
  const [suggest, setSuggest] = useState(0);
  const area = useRef<HTMLTextAreaElement>(null);
  const lastTyping = useRef(0);

  useEffect(() => {
    if (editing) setText(editing.body ?? '');
    area.current?.focus();
  }, [editing]);

  useEffect(() => {
    host
      .request<{ members: Member[]; profiles: Record<string, Profile> }>('rooms.members', { room: room.id })
      .then((r) => {
        useSocial.getState().remember(r.profiles);
        setMembers(r.members.map((m) => m.sub));
      })
      .catch(() => undefined);
  }, [room.id]);

  // An @name being typed at the caret, and who it could be.
  const partial = /(^|\s)@([A-Za-z0-9_]{0,20})$/.exec(text)?.[2];
  const candidates = useMemo(() => {
    if (partial === undefined) return [];
    return members
      .map((sub) => profiles[sub])
      .filter((p): p is Profile => !!p && p.qor_id.toLowerCase().startsWith(partial.toLowerCase()))
      .slice(0, 6);
  }, [partial, members, profiles]);

  const complete = (p: Profile) => {
    setText((t) => t.replace(/@([A-Za-z0-9_]{0,20})$/, `@${p.qor_id} `));
    setSuggest(0);
    area.current?.focus();
  };

  const send = async () => {
    const body = text.trim();
    if (!body || busy) return;
    setBusy(true);
    try {
      // The language it is written in, told on this PC; the writer's own when the words are too few (ADR-092).
      const told = await translateHost.detect(body).catch(() => null);
      const lang = told ?? me?.language ?? null;
      if (editing) {
        const r = await host.request<{ message: Message }>('messages.edit', { message: editing.id, body, ...(lang ? { lang } : {}) });
        useSocial.getState().putMessage(r.message);
      } else {
        const nonce = crypto.randomUUID();
        const r = await host.request<{ message: Message }>('messages.send', {
          room: room.id,
          body,
          ...(reply ? { reply_to: reply.id } : {}),
          nonce,
          ...(lang ? { lang } : {}),
        });
        useSocial.getState().putMessage(r.message);
      }
      setText('');
      onDone();
    } catch (e) {
      notify('bad', refusalOf(e).message);
    } finally {
      setBusy(false);
    }
  };

  const typed = (value: string) => {
    setText(value);
    setSuggest(0);
    const now = Date.now();
    if (value && now - lastTyping.current > 3000) {
      lastTyping.current = now;
      void host.request('typing', { room: room.id }).catch(() => undefined);
    }
  };

  const left = 4000 - text.length;
  if (muted)
    return (
      <div className="flex-none px-6 pb-5" data-muted>
        <p className="flex items-center gap-2 border border-edge bg-well px-3 py-3 text-ui text-ink-muted" role="status">
          <VolumeX size={14} className="text-bad" />
          A moderator muted you in World until {new Date(mutedUntil!).toLocaleString(undefined, { hour: '2-digit', minute: '2-digit', weekday: 'short' })}. You can
          still read, and talk anywhere else.
        </p>
      </div>
    );
  return (
    <div className="flex-none px-6 pb-5">
      {(reply || editing) && (
        <div className="flex items-center gap-2 border border-b-0 border-edge bg-raised px-3 py-1.5 text-caption text-ink-body">
          {editing ? <Pencil size={12} /> : <CornerUpLeft size={12} />}
          <span className="min-w-0 flex-1 truncate">
            {editing ? 'Editing your message' : `Replying to ${shownName(profiles[reply!.author])}: ${reply!.body ?? ''}`}
          </span>
          <button type="button" aria-label="Cancel" className="text-ink-muted hover:text-ink" onClick={() => { setText(''); onDone(); }}>
            <X size={13} />
          </button>
        </div>
      )}
      <div className="relative flex items-end gap-2 border border-edge bg-well px-3 py-2 focus-within:border-accent">
        {candidates.length > 0 && (
          <ul className="absolute bottom-full left-0 mb-1 w-64 border border-edge bg-raised p-1 shadow-lg" role="listbox" aria-label="People to mention">
            {candidates.map((p, i) => (
              <li key={p.sub}>
                <button
                  type="button"
                  role="option"
                  aria-selected={i === suggest}
                  className={`flex w-full items-center gap-2 px-2 py-1 text-left text-ui ${i === suggest ? 'bg-base text-ink' : 'text-ink-body'}`}
                  onMouseDown={(e) => {
                    e.preventDefault();
                    complete(p);
                  }}
                >
                  <Avatar name={p.qor_id} src={p.avatar_url} size={20} />
                  <span className="truncate">{shownName(p)}</span>
                  <span className="numeric ml-auto text-micro text-ink-faint">@{p.qor_id}</span>
                </button>
              </li>
            ))}
          </ul>
        )}
        <textarea
          ref={area}
          rows={1}
          className="max-h-40 min-h-[1.5rem] flex-1 resize-none bg-transparent text-body text-ink outline-none placeholder:text-ink-faint selectable"
          placeholder={
            room.kind === 'world' ? 'Say something to everyone, in any language' : `Message ${room.kind === 'direct' ? title : `#${title}`}`
          }
          aria-label={`Message ${title}`}
          value={text}
          maxLength={4000}
          onChange={(e) => typed(e.target.value)}
          onInput={(e) => {
            const el = e.currentTarget;
            el.style.height = 'auto';
            el.style.height = `${Math.min(el.scrollHeight, 160)}px`;
          }}
          onKeyDown={(e) => {
            if (candidates.length > 0 && (e.key === 'ArrowDown' || e.key === 'ArrowUp')) {
              e.preventDefault();
              setSuggest((n) => (n + (e.key === 'ArrowDown' ? 1 : candidates.length - 1)) % candidates.length);
            } else if (candidates.length > 0 && (e.key === 'Tab' || e.key === 'Enter')) {
              e.preventDefault();
              complete(candidates[suggest] ?? candidates[0]!);
            } else if (e.key === 'Enter' && !e.shiftKey) {
              e.preventDefault();
              void send();
            } else if (e.key === 'Escape' && (reply || editing)) {
              setText('');
              onDone();
            }
          }}
        />
        {left < 200 && <span className={`numeric text-micro ${left < 0 ? 'text-bad' : 'text-ink-faint'}`}>{left}</span>}
        <button type="button" className="btn btn-primary px-3 py-1.5" aria-label="Send" disabled={!text.trim() || busy} onClick={() => void send()}>
          {busy ? <Loader2 size={14} className="animate-spin" /> : <SendHorizontal size={14} />}
        </button>
      </div>
      <p className="mt-1 text-micro text-ink-faint">Enter to send · Shift+Enter for a new line · @ to mention</p>
    </div>
  );
}

/* ─────────────────────────────────── panels ───────────────────────────── */

function Members({ room }: { room: string }) {
  const profiles = useSocial((s) => s.profiles);
  const presence = useSocial((s) => s.presence);
  const summary = useSocial((s) => s.rooms[room]);
  const me = useSocial((s) => s.me);
  const notify = useQor((s) => s.notify);
  const [members, setMembers] = useState<Member[] | null>(null);

  const read = () =>
    host
      .request<{ members: Member[]; profiles: Record<string, Profile> }>('rooms.members', { room })
      .then((r) => {
        useSocial.getState().remember(r.profiles);
        setMembers(r.members);
      })
      .catch(() => setMembers([]));
  useEffect(() => {
    void read();
  }, [room]); // eslint-disable-line react-hooks/exhaustive-deps

  const manage = async (method: string, params: Record<string, unknown>) => {
    try {
      await host.request(method, { room, ...params });
      await read();
    } catch (e) {
      notify('bad', refusalOf(e).message);
    }
  };

  if (!members) return <p className="p-5 text-caption text-ink-muted">Reading who is here…</p>;
  const order = { online: 0, away: 1, offline: 2 } as const;
  const sorted = [...members].sort((a, b) => order[presence[a.sub]?.state ?? 'offline'] - order[presence[b.sub]?.state ?? 'offline']);
  return (
    <section className="px-4 pb-5" aria-label="Members">
      <p className="eyebrow mb-2">{summary?.kind === 'world' ? `Here now · ${members.length}` : `${members.length} in this room`}</p>
      <ul className="flex flex-col gap-1">
        {sorted.map((m) => {
          const p = profiles[m.sub];
          return (
            <li key={m.sub} className="group flex items-center gap-2.5" style={accentStyle(p)}>
              <button type="button" className="flex min-w-0 flex-1 items-center gap-2.5 py-1 text-left" onClick={() => useSocial.getState().showCard(m.sub)}>
                <span className="relative">
                  <Avatar name={p?.qor_id} src={p?.avatar_url ?? null} size={28} />
                  <PresenceDot presence={presence[m.sub]} className="absolute -bottom-0.5 -right-0.5 ring-2 ring-void" />
                </span>
                <span className="min-w-0 flex-1">
                  <span className="social-accent block truncate text-ui">{shownName(p)}</span>
                  {m.role !== 'member' && <span className="eyebrow text-accent">{m.role}</span>}
                </span>
              </button>
              {summary?.role === 'owner' && m.sub !== me?.sub && (
                <span className="hidden gap-1 group-hover:flex">
                  <button
                    type="button"
                    className="text-micro text-ink-muted hover:text-ink"
                    onClick={() => void manage('rooms.role', { sub: m.sub, role: m.role === 'moderator' ? 'member' : 'moderator' })}
                  >
                    {m.role === 'moderator' ? 'Unmod' : 'Mod'}
                  </button>
                  <button type="button" className="text-micro text-ink-muted hover:text-bad" onClick={() => void manage('rooms.remove', { sub: m.sub })}>
                    Remove
                  </button>
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}

/* ─────────────────────────────────── dialogs ──────────────────────────── */

function DialogFrame({ title, eyebrow, onClose, children }: { title: string; eyebrow: string; onClose: () => void; children: ReactNode }) {
  return (
    <div className="modal" role="dialog" aria-label={title} onKeyDown={(e) => e.key === 'Escape' && onClose()}>
      <div className="glass-solid cut modal-panel modal-narrow p-6">
        <div className="mb-5 flex items-start justify-between">
          <div>
            <p className="eyebrow mb-1 text-accent">{eyebrow}</p>
            <h2 className="heading text-heading">{title}</h2>
          </div>
          <button type="button" aria-label="Close" className="text-ink-muted hover:text-ink" onClick={onClose}>
            <X size={16} />
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

function useRequest() {
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);
  const run = async <T,>(fn: () => Promise<T>): Promise<T | null> => {
    setBusy(true);
    setProblem(null);
    try {
      return await fn();
    } catch (e) {
      setProblem(refusalOf(e).message);
      return null;
    } finally {
      setBusy(false);
    }
  };
  const note = problem && (
    <p role="alert" className="mt-3 text-caption text-bad">
      {problem}
    </p>
  );
  return { busy, run, note };
}

function NewRoom({ onClose }: { onClose: () => void }) {
  const putRoom = useSocial((s) => s.putRoom);
  const open = useSocial((s) => s.open);
  const [name, setName] = useState('');
  const [topic, setTopic] = useState('');
  const [kind, setKind] = useState<'public' | 'private'>('public');
  const { busy, run, note } = useRequest();
  const create = async () => {
    const r = await run(() => host.request<{ room: Room }>('rooms.create', { kind, name, topic }));
    if (r) {
      putRoom(r.room, 'owner');
      open(r.room.id);
      onClose();
    }
  };
  return (
    <DialogFrame title="New room" eyebrow="Social" onClose={onClose}>
      <label className="mb-4 block">
        <span className="eyebrow mb-2 block">Name</span>
        <input className="field" autoFocus maxLength={48} value={name} onChange={(e) => setName(e.target.value)} />
      </label>
      <label className="mb-4 block">
        <span className="eyebrow mb-2 block">Topic</span>
        <input className="field" maxLength={280} value={topic} onChange={(e) => setTopic(e.target.value)} />
      </label>
      <div className="mb-5 grid grid-cols-2 gap-2" role="radiogroup" aria-label="Who may join">
        {(['public', 'private'] as const).map((k) => (
          <button
            key={k}
            type="button"
            role="radio"
            aria-checked={kind === k}
            className={`border p-3 text-left ${kind === k ? 'border-accent bg-raised' : 'border-edge'}`}
            onClick={() => setKind(k)}
          >
            <span className="mb-1 flex items-center gap-1.5 text-ui font-semibold text-ink">
              {k === 'public' ? <Globe2 size={13} /> : <Lock size={13} />} {k === 'public' ? 'Public' : 'Private'}
            </span>
            <span className="text-caption text-ink-muted">{k === 'public' ? 'Anyone can find and join it.' : 'Only people you invite.'}</span>
          </button>
        ))}
      </div>
      <button type="button" className="btn btn-primary" disabled={busy || !name.trim()} onClick={() => void create()}>
        {busy ? <Loader2 size={13} className="animate-spin" /> : <Plus size={13} />} Create
      </button>
      {note}
    </DialogFrame>
  );
}

function Discover({ onClose }: { onClose: () => void }) {
  const rooms = useSocial((s) => s.rooms);
  const putRoom = useSocial((s) => s.putRoom);
  const open = useSocial((s) => s.open);
  const [found, setFound] = useState<{ room: Room; members: number }[] | null>(null);
  const { busy, run, note } = useRequest();
  useEffect(() => {
    void run(() => host.request<{ rooms: { room: Room; members: number }[] }>('rooms.discover')).then((r) => setFound(r?.rooms ?? []));
  }, []); // eslint-disable-line react-hooks/exhaustive-deps
  const join = async (room: Room) => {
    const r = await run(() => host.request<{ room: Room }>('rooms.join', { room: room.id }));
    if (r) {
      putRoom(r.room, 'member');
      open(r.room.id);
      onClose();
    }
  };
  return (
    <DialogFrame title="Find a room" eyebrow="Public rooms" onClose={onClose}>
      {found === null ? (
        <p className="text-caption text-ink-muted">Looking…</p>
      ) : found.length === 0 ? (
        <p className="text-ui text-ink-body">No public rooms yet. Start the first.</p>
      ) : (
        <ul className="flex max-h-[50vh] flex-col gap-2 overflow-y-auto">
          {found.map(({ room, members }) => (
            <li key={room.id} className="relative overflow-hidden border border-edge">
              <Banner backdrop={room.ambience} className="absolute inset-0 opacity-50" />
              <div className="relative flex items-center gap-3 p-3">
                <div className="min-w-0 flex-1">
                  <p className="truncate text-ui font-semibold text-ink">
                    <Hash size={12} className="mr-1 inline text-accent" />
                    {room.name}
                  </p>
                  <p className="truncate text-caption text-ink-body">{room.topic || `${members} ${members === 1 ? 'person' : 'people'}`}</p>
                </div>
                {rooms[room.id] ? (
                  <button type="button" className="btn btn-ghost" onClick={() => { open(room.id); onClose(); }}>
                    Open
                  </button>
                ) : (
                  <button type="button" className="btn btn-primary" disabled={busy} onClick={() => void join(room)}>
                    Join
                  </button>
                )}
              </div>
            </li>
          ))}
        </ul>
      )}
      {note}
    </DialogFrame>
  );
}

function NewDirect({ onClose }: { onClose: () => void }) {
  const putRoom = useSocial((s) => s.putRoom);
  const open = useSocial((s) => s.open);
  const [name, setName] = useState('');
  const { busy, run, note } = useRequest();
  const start = async () => {
    const r = await run(() => host.request<{ room: Room; other: Profile }>('dm.open', { qor_id: name.trim() }));
    if (r) {
      putRoom(r.room, 'member');
      useSocial.getState().remember({ [r.other.sub]: r.other });
      useSocial.setState((s) => ({ rooms: { ...s.rooms, [r.room.id]: { ...s.rooms[r.room.id]!, other: r.other.sub } } }));
      open(r.room.id);
      onClose();
    }
  };
  return (
    <DialogFrame title="Message someone" eyebrow="Direct" onClose={onClose}>
      <label className="mb-4 block">
        <span className="eyebrow mb-2 block">Their QOR ID</span>
        <input
          className="field numeric"
          autoFocus
          placeholder="name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && NAME.test(name.trim()) && void start()}
        />
      </label>
      <button type="button" className="btn btn-primary" disabled={busy || !NAME.test(name.trim())} onClick={() => void start()}>
        <MessageCirclePlus size={13} /> Start
      </button>
      {note}
    </DialogFrame>
  );
}

function Invite({ room, onClose }: { room: string; onClose: () => void }) {
  const notify = useQor((s) => s.notify);
  const [name, setName] = useState('');
  const { busy, run, note } = useRequest();
  const invite = async () => {
    const r = await run(() => host.request('rooms.invite', { room, qor_id: name.trim() }));
    if (r) {
      notify('ok', `${name.trim()} is invited.`);
      onClose();
    }
  };
  return (
    <DialogFrame title="Invite someone" eyebrow="Room" onClose={onClose}>
      <label className="mb-4 block">
        <span className="eyebrow mb-2 block">Their QOR ID</span>
        <input className="field numeric" autoFocus value={name} onChange={(e) => setName(e.target.value)} />
      </label>
      <button type="button" className="btn btn-primary" disabled={busy || !NAME.test(name.trim())} onClick={() => void invite()}>
        <UserPlus size={13} /> Invite
      </button>
      {note}
    </DialogFrame>
  );
}

function RoomSettings({ room, onClose }: { room: string; onClose: () => void }) {
  const summary = useSocial((s) => s.rooms[room]);
  const [name, setName] = useState(summary?.name ?? '');
  const [topic, setTopic] = useState(summary?.topic ?? '');
  const [ambience, setAmbience] = useState<Backdrop>(summary?.ambience ?? {});
  const { busy, run, note } = useRequest();
  if (!summary) return null;
  const save = async () => {
    const r = await run(() => host.request<{ room: Room }>('rooms.update', { room, name, topic, ambience }));
    if (r) {
      useSocial.setState((s) => ({ rooms: { ...s.rooms, [room]: { ...s.rooms[room]!, ...r.room } } }));
      onClose();
    }
  };
  const leave = async () => {
    const r = await run(() => host.request('rooms.leave', { room }));
    if (r) {
      useSocial.getState().dropRoom(room);
      onClose();
    }
  };
  return (
    <DialogFrame title="Room settings" eyebrow={summary.kind === 'private' ? 'Private room' : 'Public room'} onClose={onClose}>
      <label className="mb-4 block">
        <span className="eyebrow mb-2 block">Name</span>
        <input className="field" maxLength={48} value={name} onChange={(e) => setName(e.target.value)} />
      </label>
      <label className="mb-4 block">
        <span className="eyebrow mb-2 block">Topic</span>
        <input className="field" maxLength={280} value={topic} onChange={(e) => setTopic(e.target.value)} />
      </label>
      <fieldset className="mb-5">
        <legend className="eyebrow mb-2">Ambience</legend>
        <div className="grid grid-cols-3 gap-2">
          {BACKDROPS.map((b) => (
            <button
              key={b.id}
              type="button"
              aria-pressed={ambience.preset === b.id}
              onClick={() => setAmbience({ hue: 210, intensity: 0.6, ...ambience, preset: b.id })}
              className={`relative h-12 overflow-hidden border ${ambience.preset === b.id ? 'border-accent' : 'border-edge'}`}
            >
              <Banner backdrop={{ ...ambience, preset: b.id }} className="absolute inset-0" />
              <span className="relative z-10 m-1 inline-block bg-void/70 px-1 text-micro text-ink">{b.name}</span>
            </button>
          ))}
        </div>
        {ambience.preset && (
          <input
            type="range"
            min={0}
            max={360}
            aria-label="Ambience hue"
            value={ambience.hue ?? 210}
            onChange={(e) => setAmbience({ ...ambience, hue: Number(e.target.value) })}
            className="mt-3 w-full accent-[var(--accent)]"
            style={{ '--b-hue': ambience.hue } as CSSProperties}
          />
        )}
      </fieldset>
      <div className="flex flex-wrap gap-2">
        <button type="button" className="btn btn-primary" disabled={busy} onClick={() => void save()}>
          <Check size={13} /> Save
        </button>
        <button type="button" className="btn btn-ghost btn-danger" disabled={busy} onClick={() => void leave()}>
          Leave room
        </button>
      </div>
      {note}
    </DialogFrame>
  );
}
