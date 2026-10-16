/**
 * Friends and circles (ADR-092 decision 2).
 *
 * Friendship is mutual: asked by username, answered by the one asked. Friends see each other's presence. A circle is the
 * reader's own named, coloured group of friends, which nobody else sees; it opens a private room for the group in one
 * step. Every change is Social's answer, never a guess made here first.
 */

import { useMemo, useState, type CSSProperties } from 'react';
import { Check, CircleDot, Clock, Loader2, MessageCircle, MoreHorizontal, Plus, Trash2, UserCheck, UserMinus, UserPlus, Users, X } from 'lucide-react';

import { Avatar } from '../components/chrome/Avatar';
import { useQor } from '../state/store';
import { PROFILE_ACCENTS } from '../styles/themes';
import { host, refusalOf, shownName, useSocial, type Circle, type Profile, type Room } from './model';
import { accentStyle, PresenceDot } from './Profile';

const NAME = /^[A-Za-z0-9_]{3,20}$/;

/** Ask Social, tell the reader what went wrong in Social's words, and give back the answer or null. */
function useAsk() {
  const notify = useQor((s) => s.notify);
  const [busy, setBusy] = useState(false);
  const ask = async <T,>(method: string, params: Record<string, unknown>, ok?: string): Promise<T | null> => {
    setBusy(true);
    try {
      const r = await host.request<T>(method, params);
      if (ok) notify('ok', ok);
      return r;
    } catch (e) {
      notify('bad', refusalOf(e).message);
      return null;
    } finally {
      setBusy(false);
    }
  };
  return { busy, ask };
}

/* ─────────────────────────────── the panel ─────────────────────────────── */

export function FriendsPanel() {
  const friends = useSocial((s) => s.friends);
  const incoming = useSocial((s) => s.incoming);
  const outgoing = useSocial((s) => s.outgoing);
  const circles = useSocial((s) => s.circles);
  const profiles = useSocial((s) => s.profiles);
  const presence = useSocial((s) => s.presence);
  const [circle, setCircle] = useState<string | null>(null);

  const shown = useMemo(() => {
    const within = circle ? (circles.find((c) => c.id === circle)?.members ?? []) : friends;
    const order = { online: 0, away: 1, offline: 2 } as const;
    return [...within].sort(
      (a, b) =>
        order[presence[a]?.state ?? 'offline'] - order[presence[b]?.state ?? 'offline'] ||
        shownName(profiles[a]).localeCompare(shownName(profiles[b])),
    );
  }, [circle, circles, friends, presence, profiles]);
  const online = shown.filter((s) => (presence[s]?.state ?? 'offline') !== 'offline');
  const away = shown.filter((s) => (presence[s]?.state ?? 'offline') === 'offline');

  return (
    <div className="min-h-0 flex-1 overflow-y-auto px-2 pb-4 pt-2" data-friends>
      <AddFriend />

      {incoming.length > 0 && (
        <section className="mb-3" aria-label="Friend requests">
          <p className="eyebrow px-2 pb-1 pt-2 text-accent">Asking to be friends · {incoming.length}</p>
          <ul className="flex flex-col gap-1">
            {incoming.map((r) => (
              <Request key={r.sub} sub={r.sub} incoming />
            ))}
          </ul>
        </section>
      )}

      <CircleBar circles={circles} chosen={circle} onChoose={setCircle} />

      <section aria-label={circle ? 'Friends in this circle' : 'Friends'}>
        {friends.length === 0 ? (
          <div className="social-friends-empty mx-2 mt-3 border border-dashed border-edge p-4 text-center">
            <Users size={18} className="mx-auto mb-2 text-accent" />
            <p className="text-ui text-ink">No friends yet</p>
            <p className="mt-1 text-caption text-ink-muted">Add someone by their QOR ID, or from their card in any room.</p>
          </div>
        ) : (
          <>
            <p className="eyebrow px-2 pb-1 pt-2">Online · {online.length}</p>
            <ul className="flex flex-col gap-0.5">
              {online.map((sub) => (
                <FriendRow key={sub} sub={sub} />
              ))}
            </ul>
            {away.length > 0 && (
              <>
                <p className="eyebrow px-2 pb-1 pt-3">Offline · {away.length}</p>
                <ul className="flex flex-col gap-0.5 opacity-75">
                  {away.map((sub) => (
                    <FriendRow key={sub} sub={sub} />
                  ))}
                </ul>
              </>
            )}
            {circle && shown.length === 0 && (
              <p className="px-2 py-2 text-caption text-ink-muted">Nobody in this circle yet: add friends from the ··· beside their name.</p>
            )}
          </>
        )}
      </section>

      {outgoing.length > 0 && (
        <section className="mt-3" aria-label="Requests you sent">
          <p className="eyebrow px-2 pb-1 pt-2">Waiting for an answer · {outgoing.length}</p>
          <ul className="flex flex-col gap-1">
            {outgoing.map((r) => (
              <Request key={r.sub} sub={r.sub} incoming={false} />
            ))}
          </ul>
        </section>
      )}
    </div>
  );
}

function AddFriend() {
  const [name, setName] = useState('');
  const { busy, ask } = useAsk();
  const ok = NAME.test(name.trim());
  const add = async () => {
    const r = await ask<{ friends: boolean }>('friends.request', { qor_id: name.trim() });
    if (r) {
      useQor.getState().notify('ok', r.friends ? `You and ${name.trim()} are friends.` : `Asked ${name.trim()} to be friends.`);
      setName('');
    }
  };
  return (
    <div className="mx-1 mb-2 flex gap-1">
      <input
        className="field py-1.5 text-ui numeric"
        placeholder="Add a friend by QOR ID"
        aria-label="Add a friend by QOR ID"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === 'Enter' && ok && !busy && void add()}
      />
      <button type="button" className="btn btn-primary px-2.5" aria-label="Ask to be friends" disabled={!ok || busy} onClick={() => void add()}>
        {busy ? <Loader2 size={13} className="animate-spin" /> : <UserPlus size={13} />}
      </button>
    </div>
  );
}

function Request({ sub, incoming }: { sub: string; incoming: boolean }) {
  const p = useSocial((s) => s.profiles[sub]);
  const { busy, ask } = useAsk();
  return (
    <li className={`social-request flex items-center gap-2.5 px-2 py-1.5 ${incoming ? 'social-request-in' : ''}`} style={accentStyle(p)} data-request={sub}>
      <button type="button" onClick={() => useSocial.getState().showCard(sub)} aria-label={`${shownName(p)}'s profile`}>
        <Avatar name={p?.qor_id} src={p?.avatar_url ?? null} size={30} level={p?.level ?? null} />
      </button>
      <span className="min-w-0 flex-1">
        <span className="social-accent block truncate text-ui font-semibold">{shownName(p)}</span>
        <span className="numeric block truncate text-micro text-ink-muted">@{p?.qor_id ?? '…'}</span>
      </span>
      {incoming ? (
        <span className="flex gap-1">
          <button type="button" className="btn btn-primary px-2 py-1" aria-label="Accept" title="Accept" disabled={busy} onClick={() => void ask('friends.accept', { sub })}>
            <Check size={13} />
          </button>
          <button type="button" className="btn btn-ghost px-2 py-1" aria-label="Decline" title="Decline" disabled={busy} onClick={() => void ask('friends.decline', { sub })}>
            <X size={13} />
          </button>
        </span>
      ) : (
        <button type="button" className="btn btn-ghost px-2 py-1 text-micro" disabled={busy} onClick={() => void ask('friends.cancel', { sub })}>
          Cancel
        </button>
      )}
    </li>
  );
}

function FriendRow({ sub }: { sub: string }) {
  const p = useSocial((s) => s.profiles[sub]);
  const presence = useSocial((s) => s.presence[sub]);
  const circles = useSocial((s) => s.circles);
  const [menu, setMenu] = useState(false);
  const { busy, ask } = useAsk();

  const message = async () => {
    const r = await ask<{ room: Room }>('dm.open', { qor_id: p?.qor_id });
    if (!r) return;
    const social = useSocial.getState();
    social.putRoom(r.room, 'member');
    useSocial.setState((s) => ({ rooms: { ...s.rooms, [r.room.id]: { ...s.rooms[r.room.id]!, other: sub } } }));
    social.open(r.room.id);
  };
  const place = async (circle: Circle) => {
    const inside = circle.members.includes(sub);
    const r = await ask<{ circle: Circle }>(inside ? 'circles.remove' : 'circles.add', { circle: circle.id, sub });
    if (r) useSocial.getState().putCircle(r.circle);
  };

  return (
    <li className="group relative" style={accentStyle(p)} data-friend={sub}>
      <div className="flex items-center gap-2.5 px-2 py-1.5 hover:bg-raised">
        <button type="button" className="flex min-w-0 flex-1 items-center gap-2.5 text-left" onClick={() => useSocial.getState().showCard(sub)}>
          <span className="relative">
            <Avatar name={p?.qor_id} src={p?.avatar_url ?? null} size={30} level={p?.level ?? null} />
            <PresenceDot presence={presence} className="absolute -bottom-0.5 -left-0.5 ring-2 ring-void" />
          </span>
          <span className="min-w-0 flex-1">
            <span className="social-accent block truncate text-ui">{shownName(p)}</span>
            <span className="block truncate text-micro text-ink-muted">
              {presence?.playing ? `Playing ${presence.playing}` : p?.status_line || `@${p?.qor_id ?? '…'}`}
            </span>
          </span>
        </button>
        <span className="flex gap-0.5 opacity-0 transition-opacity group-hover:opacity-100 group-focus-within:opacity-100">
          <button type="button" className="p-1 text-ink-muted hover:text-ink" title="Message" aria-label={`Message ${shownName(p)}`} disabled={busy} onClick={() => void message()}>
            <MessageCircle size={14} />
          </button>
          <button type="button" className="p-1 text-ink-muted hover:text-ink" title="More" aria-label="More" aria-expanded={menu} onClick={() => setMenu((m) => !m)}>
            <MoreHorizontal size={14} />
          </button>
        </span>
      </div>
      {menu && (
        <div className="absolute right-2 top-9 z-20 w-56 border border-edge bg-raised p-1 shadow-lg" role="menu" aria-label="Friend">
          <p className="eyebrow px-2 pb-1 pt-1">Circles</p>
          {circles.length === 0 && <p className="px-2 pb-1 text-micro text-ink-faint">Make a circle first, from the bar above.</p>}
          {circles.map((c) => (
            <button
              key={c.id}
              type="button"
              role="menuitemcheckbox"
              aria-checked={c.members.includes(sub)}
              className="flex w-full items-center gap-2 px-2 py-1 text-left text-ui text-ink-body hover:bg-base"
              disabled={busy}
              onClick={() => void place(c)}
            >
              <span className="social-circle-dot" style={{ background: c.color ?? 'var(--accent)' }} />
              <span className="min-w-0 flex-1 truncate">{c.name}</span>
              {c.members.includes(sub) && <Check size={12} className="text-accent" />}
            </button>
          ))}
          <div className="my-1 h-px bg-edge" />
          <button
            type="button"
            role="menuitem"
            className="flex w-full items-center gap-2 px-2 py-1 text-left text-ui text-bad hover:bg-base"
            disabled={busy}
            onClick={() => {
              setMenu(false);
              void ask('friends.remove', { sub }, `${shownName(p)} is no longer a friend.`);
            }}
          >
            <UserMinus size={13} /> Remove friend
          </button>
        </div>
      )}
    </li>
  );
}

/* ─────────────────────────────── circles ──────────────────────────────── */

function CircleBar({ circles, chosen, onChoose }: { circles: Circle[]; chosen: string | null; onChoose: (id: string | null) => void }) {
  const [making, setMaking] = useState(false);
  const [editing, setEditing] = useState<Circle | null>(null);
  const current = circles.find((c) => c.id === chosen) ?? null;
  return (
    <section className="mb-1 px-1" aria-label="Circles">
      <div className="flex flex-wrap items-center gap-1 pt-1">
        <button type="button" aria-pressed={chosen === null} className={`social-chip ${chosen === null ? 'social-chip-on' : ''}`} onClick={() => onChoose(null)}>
          All friends
        </button>
        {circles.map((c) => (
          <button
            key={c.id}
            type="button"
            aria-pressed={chosen === c.id}
            className={`social-chip ${chosen === c.id ? 'social-chip-on' : ''}`}
            style={{ '--chip': c.color ?? 'var(--accent)' } as CSSProperties}
            onClick={() => onChoose(chosen === c.id ? null : c.id)}
            data-circle={c.id}
          >
            <span className="social-circle-dot" style={{ background: c.color ?? 'var(--accent)' }} />
            {c.name}
            <span className="numeric text-ink-faint">{c.members.length}</span>
          </button>
        ))}
        <button type="button" className="social-chip" aria-label="New circle" title="New circle" onClick={() => setMaking(true)}>
          <Plus size={11} /> Circle
        </button>
      </div>
      {current && <CircleTools circle={current} onEdit={() => setEditing(current)} onGone={() => onChoose(null)} />}
      {(making || editing) && (
        <CircleEditor
          circle={editing}
          onClose={() => {
            setMaking(false);
            setEditing(null);
          }}
        />
      )}
    </section>
  );
}

function CircleTools({ circle, onEdit, onGone }: { circle: Circle; onEdit: () => void; onGone: () => void }) {
  const { busy, ask } = useAsk();
  const open = async () => {
    const r = await ask<{ room: Room }>('circles.open', { circle: circle.id }, `${circle.name} is open; its friends are invited.`);
    if (r) {
      useSocial.getState().putRoom(r.room, 'owner');
      useSocial.getState().open(r.room.id);
    }
  };
  const remove = async () => {
    if ((await ask('circles.delete', { circle: circle.id })) !== null) {
      useSocial.setState((s) => ({ circles: s.circles.filter((c) => c.id !== circle.id) }));
      onGone();
    }
  };
  return (
    <div className="mt-1.5 flex items-center gap-1 px-1">
      <button type="button" className="btn btn-primary px-2.5 py-1 text-micro" disabled={busy || circle.members.length === 0} onClick={() => void open()}>
        <MessageCircle size={12} /> Group chat
      </button>
      <button type="button" className="btn btn-ghost px-2 py-1 text-micro" disabled={busy} onClick={onEdit}>
        <CircleDot size={12} /> Edit
      </button>
      <button type="button" className="btn btn-ghost btn-danger ml-auto px-2 py-1" aria-label={`Delete ${circle.name}`} title="Delete circle" disabled={busy} onClick={() => void remove()}>
        <Trash2 size={12} />
      </button>
    </div>
  );
}

function CircleEditor({ circle, onClose }: { circle: Circle | null; onClose: () => void }) {
  const [name, setName] = useState(circle?.name ?? '');
  const [color, setColor] = useState<string | null>(circle?.color ?? PROFILE_ACCENTS[0] ?? null);
  const { busy, ask } = useAsk();
  const save = async () => {
    const r = circle
      ? await ask<{ circle: Circle }>('circles.update', { circle: circle.id, name, color })
      : await ask<{ circle: Circle }>('circles.create', { name, color });
    if (r) {
      useSocial.getState().putCircle(r.circle);
      onClose();
    }
  };
  return (
    <div className="modal" role="dialog" aria-label={circle ? 'Edit circle' : 'New circle'} onKeyDown={(e) => e.key === 'Escape' && onClose()}>
      <div className="glass-solid cut modal-panel modal-narrow p-6">
        <div className="mb-5 flex items-start justify-between">
          <div>
            <p className="eyebrow mb-1 text-accent">Circles · only you see them</p>
            <h2 className="heading text-heading">{circle ? 'Edit circle' : 'New circle'}</h2>
          </div>
          <button type="button" aria-label="Close" className="text-ink-muted hover:text-ink" onClick={onClose}>
            <X size={16} />
          </button>
        </div>
        <label className="mb-4 block">
          <span className="eyebrow mb-2 block">Name</span>
          <input className="field" autoFocus maxLength={32} placeholder="Squad" value={name} onChange={(e) => setName(e.target.value)} />
        </label>
        <fieldset className="mb-5">
          <legend className="eyebrow mb-2">Colour</legend>
          <div className="flex flex-wrap gap-2">
            {PROFILE_ACCENTS.map((a) => (
              <button
                key={a}
                type="button"
                aria-label={`Colour ${a}`}
                aria-pressed={color === a}
                className={`h-7 w-7 rounded-full border-2 ${color === a ? 'border-ink' : 'border-transparent'}`}
                style={{ background: a }}
                onClick={() => setColor(a)}
              />
            ))}
          </div>
        </fieldset>
        <button type="button" className="btn btn-primary" disabled={busy || !name.trim()} onClick={() => void save()}>
          {busy ? <Loader2 size={13} className="animate-spin" /> : <Check size={13} />} {circle ? 'Save' : 'Make circle'}
        </button>
      </div>
    </div>
  );
}

/* ─────────────────────────────── on a card ─────────────────────────────── */

/** The friend button on someone's card: ask, answer, or see that you are friends (and end it). */
export function FriendButton({ profile }: { profile: Profile }) {
  const friends = useSocial((s) => s.friends.includes(profile.sub));
  const asked = useSocial((s) => s.outgoing.some((r) => r.sub === profile.sub));
  const asking = useSocial((s) => s.incoming.some((r) => r.sub === profile.sub));
  const { busy, ask } = useAsk();
  if (friends)
    return (
      <button
        type="button"
        className="btn btn-ghost"
        disabled={busy}
        title="Remove friend"
        onClick={() => void ask('friends.remove', { sub: profile.sub }, `${shownName(profile)} is no longer a friend.`)}
        data-friend-state="friends"
      >
        <UserCheck size={13} className="text-ok" /> Friends
      </button>
    );
  if (asking)
    return (
      <button type="button" className="btn btn-primary" disabled={busy} onClick={() => void ask('friends.accept', { sub: profile.sub })} data-friend-state="asking">
        <Check size={13} /> Accept friend
      </button>
    );
  if (asked)
    return (
      <button type="button" className="btn btn-ghost" disabled={busy} onClick={() => void ask('friends.cancel', { sub: profile.sub })} data-friend-state="asked">
        <Clock size={13} /> Requested · cancel
      </button>
    );
  return (
    <button
      type="button"
      className="btn"
      disabled={busy}
      onClick={() => void ask('friends.request', { qor_id: profile.qor_id })}
      data-friend-state="none"
    >
      <UserPlus size={13} /> Add friend
    </button>
  );
}
