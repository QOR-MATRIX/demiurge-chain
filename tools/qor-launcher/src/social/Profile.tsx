/**
 * Profiles per QOR ID (docs/blueprints/social.md section 5): a card opened from any message, and an editor with a live
 * preview. The name, avatar and level are QOR ID's; the rest is the person's own: a display line, a status, a bio, an
 * accent their name and messages carry for everyone, a banner from the QFX backdrops, and a shelf of DRC-369 assets.
 */

import { useEffect, useMemo, useState, type CSSProperties } from 'react';
import { Ban, Check, Loader2, MessageCircle, Pencil, ShieldCheck, Sparkles, X } from 'lucide-react';

import { Avatar } from '../components/chrome/Avatar';
import { assets, explain, type OwnedAsset } from '../lib/ipc';
import { selectActiveAccount, useQor } from '../state/store';
import { PROFILE_ACCENTS } from '../styles/themes';
import { Banner, BACKDROPS } from './Banner';
import {
  host,
  refusalOf,
  shownName,
  useSocial,
  type Backdrop,
  type Presence,
  type Profile,
} from './model';

/** The accents offered (a palette, so it lives with the theme tokens). Any other `#rrggbb` is checked. */
const ACCENTS = PROFILE_ACCENTS;

function luminance(hex: string): number {
  const c = hex.replace('#', '');
  const channel = (i: number) => {
    const v = parseInt(c.slice(i, i + 2), 16) / 255;
    return v <= 0.03928 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
  };
  return 0.2126 * channel(0) + 0.7152 * channel(2) + 0.0722 * channel(4);
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

/** The theme's grounds, as hex, read from the page. */
function grounds(): string[] {
  const style = getComputedStyle(document.documentElement);
  return ['--void', '--base', '--surface', '--raised']
    .map((v) => style.getPropertyValue(v).trim())
    .filter((v) => /^#[0-9a-f]{6}$/i.test(v));
}

/** Whether an accent reads as text (AA, 4.5:1) on every ground of the current theme. */
export function readableAccent(accent: string): boolean {
  if (!/^#[0-9a-f]{6}$/i.test(accent)) return false;
  const all = grounds();
  return all.length === 0 || all.every((g) => contrast(accent, g) >= 4.5);
}

/** A person's accent as a style, for their name and the rail beside their words; none if it would not read here. */
export function accentStyle(p: Profile | undefined | null): CSSProperties | undefined {
  if (!p?.accent || !readableAccent(p.accent)) return undefined;
  return { '--who': p.accent } as CSSProperties;
}

export function PresenceDot({ presence, className = '' }: { presence: Presence | undefined; className?: string }) {
  const state = presence?.state ?? 'offline';
  const tone = state === 'online' ? 'dot-ok' : state === 'away' ? 'dot-warn' : 'bg-edge';
  return <span className={`dot ${tone} ${className}`} aria-label={state} role="img" />;
}

function presenceText(p: Presence | undefined): string {
  if (!p || p.state === 'offline') return 'Offline';
  const base = p.state === 'away' ? 'Away' : 'Online';
  return p.playing ? `${base} · playing ${p.playing}` : base;
}

/** The card, in the side panel. */
export function ProfileCard({ sub, onEdit }: { sub: string; onEdit: () => void }) {
  const profile = useSocial((s) => s.profiles[sub]);
  const presence = useSocial((s) => s.presence[sub]);
  const me = useSocial((s) => s.me);
  const blocked = useSocial((s) => s.blocked.includes(sub));
  const open = useSocial((s) => s.open);
  const putRoom = useSocial((s) => s.putRoom);
  const notify = useQor((s) => s.notify);
  const [busy, setBusy] = useState(false);
  const mine = me?.sub === sub;

  if (!profile) {
    return <p className="p-6 text-caption text-ink-muted">Reading their profile…</p>;
  }

  const message = async () => {
    setBusy(true);
    try {
      const r = await host.request<{ room: Parameters<typeof putRoom>[0] }>('dm.open', { qor_id: profile.qor_id });
      putRoom(r.room, 'member');
      useSocial.setState((s) => ({ rooms: { ...s.rooms, [r.room.id]: { ...s.rooms[r.room.id]!, other: sub } } }));
      open(r.room.id);
    } catch (e) {
      notify('bad', refusalOf(e).message);
    } finally {
      setBusy(false);
    }
  };

  const toggleBlock = async () => {
    setBusy(true);
    try {
      const r = await host.request<{ blocked: string[] }>(blocked ? 'unblock' : 'block', { sub });
      useSocial.setState({ blocked: r.blocked });
      notify('ok', blocked ? `${profile.qor_id} is unblocked.` : `${profile.qor_id} is blocked: nothing of theirs reaches you.`);
    } catch (e) {
      notify('bad', refusalOf(e).message);
    } finally {
      setBusy(false);
    }
  };

  return (
    <article aria-label={`${shownName(profile)}'s profile`} className="flex flex-col" data-profile-card>
      <Banner backdrop={profile.banner} className="h-28" />
      <div className="-mt-9 px-5">
        <Avatar name={profile.qor_id} src={profile.avatar_url} size={64} level={profile.level} />
      </div>
      <div className="px-5 pb-5 pt-3" style={accentStyle(profile)}>
        <h2 className="social-accent text-title font-semibold">{shownName(profile)}</h2>
        <p className="numeric text-caption text-ink-muted">@{profile.qor_id}</p>
        <p className="mt-2 flex items-center gap-2 text-caption text-ink-body">
          <PresenceDot presence={presence} />
          {presenceText(presence)}
        </p>
        {profile.status_line && <p className="mt-1 text-caption italic text-ink-body">“{profile.status_line}”</p>}
        {profile.bio && <p className="mt-4 whitespace-pre-line text-ui text-ink-body selectable">{profile.bio}</p>}

        {profile.showcase.length > 0 && (
          <section className="mt-5" aria-label="Showcase">
            <p className="eyebrow mb-2">Showcase</p>
            <ul className="grid grid-cols-3 gap-2">
              {profile.showcase.map((id) => (
                <li key={id} className="border border-edge bg-well px-2 py-2 text-center">
                  <Sparkles size={14} className="mx-auto mb-1 text-accent" aria-hidden="true" />
                  <span className="numeric block text-micro text-ink-body">#{id}</span>
                </li>
              ))}
            </ul>
            {!mine && (
              <p className="mt-2 text-micro text-ink-faint">
                DRC-369 assets they chose to show; whether they still hold each is not checked here yet.
              </p>
            )}
          </section>
        )}

        <div className="mt-5 flex flex-wrap gap-2">
          {mine ? (
            <button type="button" className="btn" onClick={onEdit}>
              <Pencil size={13} /> Edit profile
            </button>
          ) : (
            <>
              <button type="button" className="btn btn-primary" disabled={busy || blocked} onClick={() => void message()}>
                <MessageCircle size={13} /> Message
              </button>
              <button type="button" className="btn btn-ghost btn-danger" disabled={busy} onClick={() => void toggleBlock()}>
                <Ban size={13} /> {blocked ? 'Unblock' : 'Block'}
              </button>
            </>
          )}
        </div>
      </div>
    </article>
  );
}

/** The editor: every change shown on a live card before it is saved. */
export function ProfileEditor({ onClose }: { onClose: () => void }) {
  const me = useSocial((s) => s.me);
  const notify = useQor((s) => s.notify);
  const account = useQor(selectActiveAccount);
  const [display, setDisplay] = useState(me?.display ?? '');
  const [status, setStatus] = useState(me?.status_line ?? '');
  const [bio, setBio] = useState(me?.bio ?? '');
  const [accent, setAccent] = useState<string | null>(me?.accent ?? null);
  const [banner, setBanner] = useState<Backdrop>(me?.banner ?? {});
  const [showcase, setShowcase] = useState<string[]>(me?.showcase ?? []);
  const [owned, setOwned] = useState<OwnedAsset[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [problem, setProblem] = useState<string | null>(null);

  useEffect(() => {
    if (!account) return;
    assets
      .of(account.address)
      .then(setOwned)
      .catch(() => setOwned([]));
  }, [account]);

  const accentOk = accent === null || readableAccent(accent);
  const preview: Profile | null = useMemo(
    () => (me ? { ...me, display, status_line: status, bio, accent, banner, showcase } : null),
    [me, display, status, bio, accent, banner, showcase],
  );
  if (!me || !preview) return null;

  const save = async () => {
    setBusy(true);
    setProblem(null);
    try {
      const r = await host.request<{ profile: Profile }>('profile.update', {
        display,
        status_line: status,
        bio,
        accent,
        banner,
        showcase,
      });
      useSocial.setState((s) => ({ me: r.profile, profiles: { ...s.profiles, [r.profile.sub]: r.profile } }));
      notify('ok', 'Your profile is saved; everyone who shares a room with you sees it now.');
      onClose();
    } catch (e) {
      setProblem(refusalOf(e).message);
    } finally {
      setBusy(false);
    }
  };

  const toggleAsset = (id: string) =>
    setShowcase((list) => (list.includes(id) ? list.filter((x) => x !== id) : list.length >= 6 ? list : [...list, id]));

  return (
    <div className="modal" role="dialog" aria-label="Edit your profile" onKeyDown={(e) => e.key === 'Escape' && onClose()}>
      <div className="glass-solid cut modal-panel grid w-[min(60rem,100%)] grid-cols-[1fr_18rem] gap-0 p-0" data-profile-editor>
        <div className="max-h-[80vh] overflow-y-auto p-6">
          <div className="mb-5 flex items-start justify-between">
            <div>
              <p className="eyebrow mb-1 text-accent">Social</p>
              <h2 className="heading text-heading">Your profile</h2>
            </div>
            <button type="button" aria-label="Close" className="text-ink-muted hover:text-ink" onClick={onClose}>
              <X size={16} />
            </button>
          </div>

          <label className="mb-4 block">
            <span className="eyebrow mb-2 block">Display line</span>
            <input className="field" maxLength={48} value={display} placeholder={me.qor_id} onChange={(e) => setDisplay(e.target.value)} />
          </label>
          <label className="mb-4 block">
            <span className="eyebrow mb-2 block">Status</span>
            <input className="field" maxLength={80} value={status} placeholder="What you are up to" onChange={(e) => setStatus(e.target.value)} />
          </label>
          <label className="mb-5 block">
            <span className="eyebrow mb-2 block">Bio</span>
            <textarea className="field min-h-[5.5rem] resize-y" maxLength={600} value={bio} onChange={(e) => setBio(e.target.value)} />
            <span className="numeric mt-1 block text-right text-micro text-ink-faint">{bio.length} / 600</span>
          </label>

          <fieldset className="mb-5">
            <legend className="eyebrow mb-2">Accent</legend>
            <div className="flex flex-wrap items-center gap-2">
              <button
                type="button"
                aria-pressed={accent === null}
                className={`h-7 rounded-full border px-3 text-micro ${accent === null ? 'border-accent text-ink' : 'border-edge text-ink-muted'}`}
                onClick={() => setAccent(null)}
              >
                Theme
              </button>
              {ACCENTS.map((a) => (
                <button
                  key={a}
                  type="button"
                  aria-label={`Accent ${a}`}
                  aria-pressed={accent === a}
                  className={`h-7 w-7 rounded-full border-2 ${accent === a ? 'border-ink' : 'border-transparent'}`}
                  style={{ background: a }}
                  onClick={() => setAccent(a)}
                />
              ))}
              <input
                className="field w-28 py-1 numeric"
                aria-label="Accent colour"
                value={accent ?? ''}
                placeholder="#rrggbb"
                onChange={(e) => setAccent(e.target.value.trim().toLowerCase() || null)}
              />
            </div>
            {!accentOk && (
              <p role="alert" className="mt-2 text-caption text-bad">
                That colour would not read on this theme's grounds. Choose a brighter one.
              </p>
            )}
          </fieldset>

          <fieldset className="mb-5">
            <legend className="eyebrow mb-2">Banner</legend>
            <div className="grid grid-cols-3 gap-2">
              {BACKDROPS.map((b) => (
                <button
                  key={b.id}
                  type="button"
                  aria-pressed={banner.preset === b.id}
                  onClick={() => setBanner({ hue: 250, intensity: 0.6, ...banner, preset: b.id })}
                  className={`relative h-14 overflow-hidden border text-left ${banner.preset === b.id ? 'border-accent' : 'border-edge'}`}
                >
                  <Banner backdrop={{ ...banner, preset: b.id }} className="absolute inset-0" />
                  <span className="relative z-10 m-1.5 inline-block bg-void/70 px-1.5 text-micro text-ink">{b.name}</span>
                </button>
              ))}
            </div>
            {banner.preset && (
              <div className="mt-3 grid grid-cols-2 gap-4">
                <label className="block">
                  <span className="eyebrow mb-1 block">Hue</span>
                  <input
                    type="range"
                    min={0}
                    max={360}
                    value={banner.hue ?? 250}
                    onChange={(e) => setBanner({ ...banner, hue: Number(e.target.value) })}
                    className="w-full accent-[var(--accent)]"
                  />
                </label>
                <label className="block">
                  <span className="eyebrow mb-1 block">Intensity</span>
                  <input
                    type="range"
                    min={0}
                    max={100}
                    value={Math.round((banner.intensity ?? 0.6) * 100)}
                    onChange={(e) => setBanner({ ...banner, intensity: Number(e.target.value) / 100 })}
                    className="w-full accent-[var(--accent)]"
                  />
                </label>
              </div>
            )}
            {banner.preset && (
              <button type="button" className="btn btn-ghost mt-2 px-0" onClick={() => setBanner({})}>
                No banner
              </button>
            )}
          </fieldset>

          <fieldset>
            <legend className="eyebrow mb-2">Showcase · up to six of your assets</legend>
            {owned === null ? (
              <p className="text-caption text-ink-muted">
                <Loader2 size={12} className="mr-1 inline animate-spin" /> Reading what you hold…
              </p>
            ) : owned.length === 0 ? (
              <p className="text-caption text-ink-muted">This account holds no DRC-369 assets to show yet.</p>
            ) : (
              <ul className="grid grid-cols-2 gap-2">
                {owned.map((a) => {
                  const id = `${a.collection}/${a.item}`;
                  const chosen = showcase.includes(id);
                  return (
                    <li key={id}>
                      <button
                        type="button"
                        aria-pressed={chosen}
                        onClick={() => toggleAsset(id)}
                        className={`flex w-full items-center gap-2 border px-3 py-2 text-left ${chosen ? 'border-accent bg-raised' : 'border-edge'}`}
                      >
                        {chosen ? <Check size={13} className="text-accent" /> : <Sparkles size={13} className="text-ink-faint" />}
                        <span className="min-w-0 flex-1 truncate text-ui">{a.name}</span>
                        <span className="numeric text-micro text-ink-faint">#{id}</span>
                      </button>
                    </li>
                  );
                })}
              </ul>
            )}
          </fieldset>

          {problem && (
            <p role="alert" className="mt-4 text-caption text-bad">
              {problem}
            </p>
          )}
          <div className="mt-6 flex gap-2">
            <button type="button" className="btn btn-primary" disabled={busy || !accentOk} onClick={() => void save()}>
              {busy ? <Loader2 size={13} className="animate-spin" /> : <ShieldCheck size={13} />} Save
            </button>
            <button type="button" className="btn btn-ghost" onClick={onClose}>
              Cancel
            </button>
          </div>
        </div>

        <aside aria-label="Preview" className="border-l border-edge bg-void/40">
          <p className="eyebrow px-5 pt-5">Preview</p>
          <PreviewCard profile={preview} />
        </aside>
      </div>
    </div>
  );
}

function PreviewCard({ profile }: { profile: Profile }) {
  return (
    <div className="m-4 overflow-hidden border border-edge">
      <Banner backdrop={profile.banner} className="h-24" />
      <div className="-mt-8 px-4">
        <Avatar name={profile.qor_id} src={profile.avatar_url} size={56} level={profile.level} />
      </div>
      <div className="px-4 pb-4 pt-2" style={accentStyle(profile)}>
        <p className="social-accent text-title font-semibold">{shownName(profile)}</p>
        <p className="numeric text-caption text-ink-muted">@{profile.qor_id}</p>
        {profile.status_line && <p className="mt-1 text-caption italic">“{profile.status_line}”</p>}
        <div className="social-rail mt-3 py-1 pl-3 text-ui text-ink-body">How your words look in a room.</div>
      </div>
    </div>
  );
}

/** For messages where only an explained error is wanted. */
export const explainSocial = (e: unknown) => refusalOf(e).message || explain(e);
