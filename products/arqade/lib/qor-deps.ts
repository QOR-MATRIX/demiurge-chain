// The worker's real database, network, clock and settings for `lib/qor-session.ts`.
import { env } from 'cloudflare:workers';
import { database } from './arena-store';
import { configFrom, type Db, type Deps } from './qor-session';

export function deps(): Deps {
  return {
    db: database() as unknown as Db,
    fetch: (input, init) => fetch(input, init),
    now: Date.now,
    config: configFrom(env as unknown as Record<string, unknown>),
  };
}

/** Where every sign-in step returns the browser: the QOR Identity screen, with what happened. */
export function backToIdentity(request: Request, params: Record<string, string>): string {
  const url = new URL('/', request.url);
  url.search = new URLSearchParams(params).toString();
  url.hash = 'qor-identity';
  return url.toString();
}
