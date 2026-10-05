// The server's real database, network, clock and settings for `lib/qor-session.ts`.
import { cookies } from 'next/headers';
import { database } from './db';
import { ARCADE_RECHECK_MS, SESSION_COOKIE, configFrom, currentProfile, type Db, type Deps, type Profile } from './qor-session';

export function deps(): Deps {
  return {
    db: database() as unknown as Db,
    fetch: (input, init) => fetch(input, init),
    now: Date.now,
    config: configFrom(process.env),
  };
}

/** The QOR ID person behind this request's session cookie, for the live arcade, or null. */
export async function sessionProfile(): Promise<Profile | null> {
  const held = (await cookies()).get(SESSION_COOKIE)?.value ?? null;
  return currentProfile(deps(), held, ARCADE_RECHECK_MS);
}

/** Where every sign-in step returns the browser: the QOR Identity screen, with what happened. A path, not
 * an address, so the browser stays on whichever host it is on. */
export function backToIdentity(params: Record<string, string>): string {
  return `/?${new URLSearchParams(params).toString()}#qor-identity`;
}
