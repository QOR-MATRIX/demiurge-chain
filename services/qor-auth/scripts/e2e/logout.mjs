// End-to-end check: an access token stops working as soon as its session is deleted.
//
//   node scripts/e2e/logout.mjs [base url, default http://127.0.0.1:3100]
//
// Needs qor-auth running against Postgres and Redis, and Node 22+. The Redis-outage check stops and
// restarts a Docker container; it runs only when REDIS_CONTAINER names the Redis container this
// service uses, and is reported as SKIP otherwise, never as a pass. Exits non-zero if any check fails.
import { execSync } from 'node:child_process';
import { refuseRealEmail } from './_guard.mjs';

const BASE = process.argv[2] ?? 'http://127.0.0.1:3100';
await refuseRealEmail(BASE);
let passed = 0;
let failed = 0;

function check(name, ok, detail) {
  if (ok) passed++;
  else failed++;
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}  (${detail})`);
}

async function call(method, path, { token, body } = {}) {
  const headers = { 'content-type': 'application/json' };
  if (token) headers.authorization = `Bearer ${token}`;
  const res = await fetch(BASE + path, { method, headers, body: body ? JSON.stringify(body) : undefined });
  let json = null;
  try { json = await res.json(); } catch {}
  return { status: res.status, json };
}

const accepted = (s) => s >= 200 && s < 300;
const username = `logout${Date.now() % 1e8}`;
const password = 'correct horse battery staple';

const reg = await call('POST', '/api/v1/auth/register', { body: { username, password } });
check('register a password account', reg.status >= 200 && reg.status < 300, reg.status);

const loginA = await call('POST', '/api/v1/auth/login', { body: { identifier: username, password } });
const loginB = await call('POST', '/api/v1/auth/login', { body: { identifier: username, password } });
check('first sign-in issues tokens', !!loginA.json?.access_token, loginA.status);
check('second sign-in issues tokens', !!loginB.json?.access_token, loginB.status);
const A = loginA.json ?? {};
const B = loginB.json ?? {};

// Nested root routes have no trailing slash: `/api/v1/profile/` is a 404 that never reaches auth.
let r = await call('GET', '/api/v1/profile', { token: A.access_token });
check('session A: access token accepted before logout', accepted(r.status), r.status);

r = await call('GET', '/api/v1/profile');
check('no token refused', r.status === 401, r.status);
r = await call('GET', '/api/v1/profile', { token: 'not.a.token' });
check('malformed token refused', r.status === 401, r.status);

r = await call('POST', '/api/v1/auth/logout', { token: A.access_token });
check('session A: logout', r.status === 204, r.status);

r = await call('GET', '/api/v1/profile', { token: A.access_token });
check('session A: access token refused at once after logout', r.status === 401, r.status);
r = await call('GET', '/api/v1/agents', { token: A.access_token });
check('session A: refused on the agent routes too', r.status === 401, r.status);
r = await call('POST', '/api/v1/auth/logout', { token: A.access_token });
check('session A: a second logout with the same token refused', r.status === 401, r.status);
r = await call('POST', '/api/v1/auth/refresh', { body: { refresh_token: A.refresh_token } });
check('session A: refresh token refused after logout', r.status === 401, r.status);

r = await call('GET', '/api/v1/profile', { token: B.access_token });
check('session B: unaffected by logging out session A', accepted(r.status), r.status);
r = await call('POST', '/api/v1/auth/refresh', { body: { refresh_token: B.refresh_token } });
check('session B: refresh still works', r.status === 200 && !!r.json?.access_token, r.status);
const C = r.json ?? {};
r = await call('GET', '/api/v1/profile', { token: C.access_token });
check('session B: refreshed access token accepted', accepted(r.status), r.status);

// Redis down: the request must be refused, and reported as a failure rather than a signed-out user.
const container = process.env.REDIS_CONTAINER;
if (!container) {
  console.log('SKIP  Redis unavailable: refused with 500  (set REDIS_CONTAINER to run it)');
} else {
  execSync(`docker stop ${container}`, { stdio: 'ignore' });
  try {
    r = await call('GET', '/api/v1/profile', { token: C.access_token });
    check('Redis unavailable: refused with 500, not let through', r.status === 500, r.status);
  } finally {
    execSync(`docker start ${container}`, { stdio: 'ignore' });
  }
}

console.log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
