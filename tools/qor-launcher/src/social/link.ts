/**
 * Social stays connected while the launcher is open and someone is signed in (ADR-091, L8.5), whichever surface is
 * showing: that is what lets the rail count what is unread, and a conversation open on return without a gap. Mounted
 * once, by the shell.
 */

import { useEffect } from 'react';

import { useQor } from '../state/store';
import { host, listenToSocial, useSocial, type Profile, type Welcome } from './model';

/** Fetch the profiles a welcome names but does not carry: the other person in each direct room, and inviters. */
async function knowEveryone(w: Welcome) {
  const known = useSocial.getState().profiles;
  const wanted = new Set<string>();
  for (const r of w.rooms) if (r.other && !known[r.other]) wanted.add(r.other);
  for (const i of w.invites) if (!known[i.inviter]) wanted.add(i.inviter);
  if (wanted.size === 0) return;
  const got = await host
    .request<{ profiles: Record<string, Profile> }>('profile.get', { subs: [...wanted] })
    .catch(() => null);
  if (got) useSocial.getState().remember(got.profiles);
}

export function useSocialLink() {
  const signedIn = useQor((s) => Boolean(s.session));
  const name = useQor((s) => s.session?.qor_id ?? null);

  useEffect(() => {
    const social = useSocial.getState();
    const stop = listenToSocial({
      status: social.setStatus,
      welcome: (w) => {
        useSocial.getState().welcome(w);
        void knowEveryone(w);
      },
      event: (e) => {
        useSocial.getState().apply(e);
        // Someone new spoke, joined or invited: learn who they are.
        const sub = (e.data.message as { author?: string } | undefined)?.author ?? (e.data.inviter as string | undefined);
        if (sub && !useSocial.getState().profiles[sub]) {
          void host
            .request<{ profiles: Record<string, Profile> }>('profile.get', { subs: [sub] })
            .then((r) => useSocial.getState().remember(r.profiles))
            .catch(() => undefined);
        }
      },
    });
    // The host may have connected before this listened: take its word for where things stand.
    host
      .status()
      .then(({ status, welcome }) => {
        useSocial.getState().setStatus(status);
        if (welcome && status.state === 'connected') {
          useSocial.getState().welcome(welcome);
          void knowEveryone(welcome);
        }
      })
      .catch(() => undefined);
    return stop;
  }, []);

  useEffect(() => {
    if (signedIn) void host.connect().catch(() => undefined);
    else void host.disconnect().catch(() => undefined);
  }, [signedIn, name]);
}

/** Everything unread, for the rail. */
export function useUnread(): number {
  return useSocial((s) => Object.values(s.rooms).reduce((n, r) => n + r.unread, 0));
}
