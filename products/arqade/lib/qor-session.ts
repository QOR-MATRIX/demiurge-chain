// QOR ID sign-in for ARQADE (P7.3; ADR-043, ADR-069 decision 4, ADR-073).
//
// ARQADE's server is the OAuth client. The browser is sent to QOR ID's own page and comes back with a
// code; the server exchanges it, with PKCE and ARQADE's client secret, and keeps the tokens in its own
// database. The browser holds one HttpOnly cookie naming an ARQADE session, and never a QOR ID token.
//
// Who a person is comes from QOR ID's `/oauth/userinfo`, asked with the access token, so a sign-out or
// a revocation at QOR ID ends the session here. The identity card asks on every check; the live arcade,
// which polls every two seconds, accepts an answer up to 30 seconds old (ADR-074). ARQADE never holds
// QOR ID's signing secret.
//
// Everything is passed in (database, fetch, clock, settings), so the whole flow runs in tests.

export type QorConfig = {
  /** QOR ID's origin, e.g. https://id.qorsync.dev */
  issuer: string;
  clientId: string;
  clientSecret: string;
  /** This app's callback, registered at QOR ID exactly. */
  redirectUri: string;
};

/** The part of the database interface (lib/db.ts) used here; the tests pass PGlite. */
export type Db = {
  prepare(sql: string): { bind(...values: unknown[]): { run(): Promise<unknown>; first<T>(): Promise<T | null> } };
};

export type Deps = { db: Db; fetch: typeof fetch; now: () => number; config: QorConfig | null };

/** Level and XP as QOR ID keeps them (ADR-078); present when QOR ID was just asked. */
export type Progress = { level: number; xp: number; level_xp: number; next_level_xp: number; next_unlock: string | null };
export type Profile = { sub: string; qorId: string; username: string; chainAccount: string | null; progress?: Progress };

export const LOGIN_COOKIE = 'arq_login';
export const SESSION_COOKIE = 'arq_session';
/** A pending sign-in lasts as long as QOR ID's page does. */
export const LOGIN_TTL_MS = 10 * 60 * 1000;
/** As long as QOR ID's session for an app (ADR-073). */
export const SESSION_TTL_MS = 8 * 60 * 60 * 1000;
/** How old QOR ID's last answer may be for the live arcade (ADR-074). */
export const ARCADE_RECHECK_MS = 30 * 1000;

const b64url = (bytes: Uint8Array) =>
  btoa(String.fromCharCode(...bytes)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');

function random(n = 32): string {
  return b64url(crypto.getRandomValues(new Uint8Array(n)));
}

export async function sha256url(text: string): Promise<string> {
  return b64url(new Uint8Array(await crypto.subtle.digest('SHA-256', new TextEncoder().encode(text))));
}

/** A cookie's value from a Cookie header, or null. */
export function cookie(header: string | null, name: string): string | null {
  for (const part of (header ?? '').split(';')) {
    const [k, ...v] = part.trim().split('=');
    if (k === name) return v.join('=') || null;
  }
  return null;
}

export function setCookie(name: string, value: string, maxAgeSecs: number, secure: boolean, path = '/'): string {
  return `${name}=${value}; Path=${path}; HttpOnly; SameSite=Lax; Max-Age=${maxAgeSecs}${secure ? '; Secure' : ''}`;
}

export class QorError extends Error {
  constructor(public status: number, message: string) {
    super(message);
  }
}

function configured(deps: Deps): QorConfig {
  if (!deps.config) throw new QorError(503, 'QOR ID sign-in is not configured for this site yet.');
  return deps.config;
}

/** Start a sign-in: remember a state and a PKCE verifier, and say where to send the browser. */
export async function startLogin(deps: Deps): Promise<{ location: string; state: string }> {
  const config = configured(deps);
  const state = random();
  const verifier = random();
  const now = deps.now();
  await deps.db.prepare('DELETE FROM qor_logins WHERE created < ?').bind(now - LOGIN_TTL_MS).run();
  await deps.db.prepare('INSERT INTO qor_logins (state, verifier, created) VALUES (?, ?, ?)').bind(state, verifier, now).run();
  const url = new URL('/oauth/authorize', config.issuer);
  url.search = new URLSearchParams({
    response_type: 'code',
    client_id: config.clientId,
    redirect_uri: config.redirectUri,
    code_challenge: await sha256url(verifier),
    code_challenge_method: 'S256',
    state,
  }).toString();
  return { location: url.toString(), state };
}

async function tokenRequest(deps: Deps, fields: Record<string, string>) {
  const config = configured(deps);
  const r = await deps.fetch(new URL('/oauth/token', config.issuer), {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ client_id: config.clientId, client_secret: config.clientSecret, ...fields }).toString(),
  });
  const data = (await r.json().catch(() => ({}))) as { access_token?: string; refresh_token?: string; expires_in?: number; error?: string };
  return { ok: r.ok && !!data.access_token && !!data.refresh_token, data };
}

async function whoIs(deps: Deps, access: string): Promise<Profile | null> {
  const config = configured(deps);
  const r = await deps.fetch(new URL('/oauth/userinfo', config.issuer), { headers: { authorization: `Bearer ${access}` } });
  if (r.status === 401) return null;
  if (!r.ok) throw new QorError(503, 'QOR ID could not be reached. Try again in a moment.');
  const u = (await r.json()) as { sub?: string; qor_id?: string; username?: string; chain_account?: string | null; progress?: Progress };
  if (!u.sub || !u.qor_id || !u.username) throw new QorError(503, 'QOR ID answered unexpectedly.');
  return { sub: u.sub, qorId: u.qor_id, username: u.username, chainAccount: u.chain_account ?? null, ...(u.progress ? { progress: u.progress } : {}) };
}

/**
 * Finish a sign-in. `state` must match both the browser's cookie and a pending sign-in, which is taken
 * at once: a code delivered to a browser that did not start this sign-in is refused (login CSRF).
 * Returns the new session's cookie value and the person.
 */
export async function finishLogin(
  deps: Deps,
  query: URLSearchParams,
  loginCookie: string | null,
): Promise<{ session: string; profile: Profile }> {
  const config = configured(deps);
  const state = query.get('state');
  if (!state || !loginCookie || state !== loginCookie) throw new QorError(400, 'This sign-in did not start in this browser. Start again.');
  const pending = await deps.db
    .prepare('SELECT verifier, created FROM qor_logins WHERE state = ?')
    .bind(state)
    .first<{ verifier: string; created: number }>();
  await deps.db.prepare('DELETE FROM qor_logins WHERE state = ?').bind(state).run();
  if (!pending || deps.now() - pending.created > LOGIN_TTL_MS) throw new QorError(400, 'This sign-in expired. Start again.');
  if (query.get('error')) throw new QorError(400, 'QOR ID did not sign you in. Start again.');
  const code = query.get('code');
  if (!code) throw new QorError(400, 'QOR ID sent no code. Start again.');

  const t = await tokenRequest(deps, {
    grant_type: 'authorization_code',
    code,
    redirect_uri: config.redirectUri,
    code_verifier: pending.verifier,
  });
  if (!t.ok) throw new QorError(400, 'QOR ID did not accept the sign-in. Start again.');
  const profile = await whoIs(deps, t.data.access_token!);
  if (!profile) throw new QorError(400, 'QOR ID did not accept the sign-in. Start again.');

  const session = random();
  const now = deps.now();
  await deps.db
    .prepare(
      'INSERT INTO qor_sessions (id, sub, qor_id, username, chain_account, access_token, refresh_token, created, expires, checked) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)',
    )
    .bind(await sha256url(session), profile.sub, profile.qorId, profile.username, profile.chainAccount, t.data.access_token, t.data.refresh_token, now, now + SESSION_TTL_MS, now)
    .run();
  return { session, profile };
}

type Row = {
  id: string;
  sub: string;
  qor_id: string;
  username: string;
  chain_account: string | null;
  access_token: string;
  refresh_token: string;
  expires: number;
  checked: number;
};

/**
 * Who the session's person is, asked of QOR ID unless it answered within `maxAgeMs`. A revoked or
 * expired session at QOR ID ends this one: the row is deleted and null returned. An expired access
 * token is refreshed once, rotating the refresh token as QOR ID requires.
 */
export async function currentProfile(deps: Deps, sessionCookie: string | null, maxAgeMs = 0): Promise<Profile | null> {
  if (!sessionCookie || !deps.config) return null;
  const id = await sha256url(sessionCookie);
  const row = await deps.db
    .prepare('SELECT id, sub, qor_id, username, chain_account, access_token, refresh_token, expires, checked FROM qor_sessions WHERE id = ?')
    .bind(id)
    .first<Row>();
  if (!row) return null;
  const end = async () => {
    await deps.db.prepare('DELETE FROM qor_sessions WHERE id = ?').bind(id).run();
    return null;
  };
  if (deps.now() > row.expires) return end();
  if (maxAgeMs > 0 && deps.now() - row.checked <= maxAgeMs) {
    return { sub: row.sub, qorId: row.qor_id, username: row.username, chainAccount: row.chain_account };
  }
  const confirmed = async (p: Profile) => {
    await deps.db
      .prepare('UPDATE qor_sessions SET qor_id = ?, username = ?, chain_account = ?, checked = ? WHERE id = ?')
      .bind(p.qorId, p.username, p.chainAccount, deps.now(), id)
      .run();
    return p;
  };

  const first = await whoIs(deps, row.access_token);
  if (first) return confirmed(first);
  const t = await tokenRequest(deps, { grant_type: 'refresh_token', refresh_token: row.refresh_token });
  if (!t.ok) return end();
  await deps.db
    .prepare('UPDATE qor_sessions SET access_token = ?, refresh_token = ? WHERE id = ?')
    .bind(t.data.access_token, t.data.refresh_token, id)
    .run();
  const again = await whoIs(deps, t.data.access_token!);
  return again ? confirmed(again) : end();
}

/** Sign out: QOR ID ends its session for ARQADE, and the row goes. */
export async function logout(deps: Deps, sessionCookie: string | null): Promise<void> {
  if (!sessionCookie) return;
  const id = await sha256url(sessionCookie);
  const row = await deps.db.prepare('SELECT refresh_token FROM qor_sessions WHERE id = ?').bind(id).first<{ refresh_token: string }>();
  await deps.db.prepare('DELETE FROM qor_sessions WHERE id = ?').bind(id).run();
  if (row && deps.config) {
    await deps.fetch(new URL('/oauth/revoke', deps.config.issuer), {
      method: 'POST',
      headers: { 'content-type': 'application/x-www-form-urlencoded' },
      body: new URLSearchParams({ client_id: deps.config.clientId, client_secret: deps.config.clientSecret, token: row.refresh_token }).toString(),
    }).catch(() => undefined);
  }
}

/**
 * Tell QOR ID that the signed-in person did one of ARQADE's tasks (ADR-078: a first match, a first payment). QOR ID
 * checks ARQADE's secret, that the token is live and ARQADE's, and that the task is ARQADE's to report, and grants its XP
 * once however often it is told. A failure changes nothing for the player, so it is reported, not thrown.
 */
export async function reportTask(deps: Deps, sessionCookie: string | null, task: 'first-match' | 'first-payment'): Promise<boolean> {
  if (!sessionCookie || !deps.config) return false;
  const row = await deps.db
    .prepare('SELECT access_token FROM qor_sessions WHERE id = ?')
    .bind(await sha256url(sessionCookie))
    .first<{ access_token: string }>();
  if (!row) return false;
  const r = await deps.fetch(new URL('/oauth/progress', deps.config.issuer), {
    method: 'POST',
    headers: { 'content-type': 'application/x-www-form-urlencoded' },
    body: new URLSearchParams({ client_id: deps.config.clientId, client_secret: deps.config.clientSecret, token: row.access_token, task }).toString(),
  }).catch(() => null);
  return !!r?.ok;
}

/** The settings from the server's environment, or null when sign-in is not set up. */
export function configFrom(env: Record<string, unknown>): QorConfig | null {
  const issuer = typeof env.QOR_ID_URL === 'string' && env.QOR_ID_URL ? env.QOR_ID_URL : 'https://id.qorsync.dev';
  const clientId = env.QOR_CLIENT_ID, clientSecret = env.QOR_CLIENT_SECRET, redirectUri = env.QOR_REDIRECT_URI;
  if (typeof clientId !== 'string' || typeof clientSecret !== 'string' || typeof redirectUri !== 'string') return null;
  if (!clientId || clientSecret.length < 32 || !redirectUri) return null;
  return { issuer, clientId, clientSecret, redirectUri };
}
