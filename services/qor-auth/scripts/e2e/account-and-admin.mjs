// End-to-end check: registration, password reset, profile stubs and the admin surface, through real
// routing and middleware.
//
//   node scripts/e2e/account-and-admin.mjs [base url, default http://127.0.0.1:3100]
//
// Needs qor-auth running against Postgres and Redis with email NOT configured, and Node 22+.
// The admin and token-reset parts write to the database directly (promoting an account to god,
// planting a reset token). They run only when PG_CONTAINER names the Postgres container and
// PG_DATABASE the service's database (PG_USER defaults to qor); otherwise they are reported as SKIP,
// never as a pass. Exits non-zero if any check fails.
import { execSync } from 'node:child_process';
import { refuseRealEmail } from './_guard.mjs';

const BASE = process.argv[2] ?? 'http://127.0.0.1:3100';
await refuseRealEmail(BASE);
const { PG_CONTAINER, PG_DATABASE, PG_USER = 'qor' } = process.env;
let passed = 0;
let failed = 0;
const check = (name, ok, detail) => {
  if (ok) passed++;
  else failed++;
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}  (${detail})`);
};
const skip = (name) => console.log(`SKIP  ${name}  (set PG_CONTAINER and PG_DATABASE to run it)`);

async function call(method, path, { token, body, form } = {}) {
  const headers = {};
  if (token) headers.authorization = `Bearer ${token}`;
  let payload;
  if (form) payload = form;
  else if (body !== undefined) {
    headers['content-type'] = 'application/json';
    payload = JSON.stringify(body);
  }
  const res = await fetch(BASE + path, { method, headers, body: payload });
  const text = await res.text();
  let json = null;
  try { json = JSON.parse(text); } catch {}
  return { status: res.status, json, text };
}

const psql = (sql) =>
  execSync(`docker exec ${PG_CONTAINER} psql -U ${PG_USER} -d ${PG_DATABASE} -tA -c "${sql}"`).toString().trim();
const claim = (token, name) => JSON.parse(Buffer.from(token.split('.')[1], 'base64url').toString())[name];
const stamp = Date.now() % 1e7;
const password = 'correct horse battery staple';
const register = (username, email) =>
  call('POST', '/api/v1/auth/register', { body: { username, password, ...(email ? { email } : {}) } });
const login = (identifier, pw = password) => call('POST', '/api/v1/auth/login', { body: { identifier, password: pw } });

// ---------------------------------------------------------------- registration
const mailUser = `acm${stamp}`;
let r = await register(mailUser, `${mailUser}@example.invalid`);
check('register with an email: 201', r.status === 201, r.status);
check('the response says the email is unverified', r.json?.email_verified === false, r.json?.email_verified);
check('the response does not ask the user to check mail that was never sent',
  /not configured/i.test(r.json?.message ?? ''), r.json?.message);
const plainUser = `acp${stamp}`;
r = await register(plainUser, null);
check('register without an email: 201, nothing to verify', r.status === 201 && r.json?.email_verified === true, `${r.status}, ${r.json?.email_verified}`);
const plainCodes = r.json?.backup_codes ?? [];
check('an account without an email receives ten distinct backup codes',
  plainCodes.length === 10 && new Set(plainCodes).size === 10, `${plainCodes.length} codes`);

// ---------------------------------------------------------------- forgot-password
const forgotten = [];
for (const identifier of [`${mailUser}@example.invalid`, mailUser, plainUser, `nobody${stamp}`]) {
  forgotten.push(await call('POST', '/api/v1/auth/forgot-password', { body: { identifier } }));
}
check('forgot-password is refused with 503 when email is not configured',
  forgotten.every((f) => f.status === 503), forgotten.map((f) => f.status).join(','));
check('forgot-password answers every identifier identically',
  forgotten.every((f) => f.text === forgotten[0].text), 'compared bodies');

// ---------------------------------------------------------------- profile stubs
const plainToken = (await login(plainUser)).json?.access_token;
r = await call('POST', '/api/v1/profile', { token: plainToken, body: { display_name: 'changed' } });
check('profile update refuses with 501', r.status === 501, r.status);
const form = new FormData();
form.append('avatar', new Blob([new Uint8Array([0x89, 0x50, 0x4e, 0x47, 0, 0, 0, 0])], { type: 'image/png' }), 'a.png');
r = await call('POST', '/api/v1/profile/avatar', { token: plainToken, form });
check('avatar upload refuses with 501', r.status === 501, r.status);
r = await call('GET', '/api/v1/profile', { token: plainToken });
check('the profile returns no balance it did not read',
  r.status === 200 && !JSON.stringify(r.json ?? {}).includes('cgt_balance'), `${r.status}, on_chain=${JSON.stringify(r.json?.on_chain)}`);

// ---------------------------------------------------------------- admin
r = await call('GET', '/api/v1/admin/stats', { token: plainToken });
check('a non-god token is refused on the admin routes', r.status === 403, r.status);

// ---------------------------------------------------------------- backup codes
const beforeBackupReset = (await login(plainUser)).json?.access_token;
const backupPassword = 'a password set with a backup code';
r = await call('POST', '/api/v1/auth/reset-password-backup', { body: { username: plainUser, backup_code: plainCodes[0], new_password: backupPassword } });
check('a backup code resets the password', r.status === 200, `${r.status} ${r.text}`);
check('the response says how many codes remain', r.json?.backup_codes_remaining === 9, r.json?.backup_codes_remaining);
r = await call('GET', '/api/v1/profile', { token: beforeBackupReset });
check('a session from before the backup-code reset is refused', r.status === 401, r.status);
check('the new password signs in', (await login(plainUser, backupPassword)).status === 200, 'checked');
r = await call('POST', '/api/v1/auth/reset-password-backup', { body: { username: plainUser, backup_code: plainCodes[0], new_password: 'yet another long password' } });
check('a spent backup code is refused', r.status === 401, r.status);
const refusedBackup = r.text;
r = await call('POST', '/api/v1/auth/reset-password-backup', { body: { username: `nobody${stamp}`, backup_code: plainCodes[1], new_password: 'yet another long password' } });
check('an unknown username gets the same answer as a spent code', r.status === 401 && r.text === refusedBackup, `${r.status}, compared bodies`);
r = await call('POST', '/api/v1/auth/reset-password-backup', { body: { username: plainUser, backup_code: plainCodes[1].toLowerCase(), new_password: 'a third long password here' } });
check('an unspent code typed in lower case still works once', r.status === 200 && r.json?.backup_codes_remaining === 8, `${r.status}, ${r.json?.backup_codes_remaining}`);

// ---------------------------------------------------------------- regenerating backup codes
const currentPlainPassword = 'a third long password here';
r = await call('POST', '/api/v1/profile/backup-codes', { body: { password: currentPlainPassword } });
check('regenerating backup codes without signing in is refused', r.status === 401, r.status);
const regenToken = (await login(plainUser, currentPlainPassword)).json?.access_token;
r = await call('POST', '/api/v1/profile/backup-codes', { token: regenToken, body: { password: 'not the password at all' } });
check('regenerating with a wrong password is refused', r.status === 401, r.status);
r = await call('POST', '/api/v1/profile/backup-codes', { token: regenToken, body: { password: currentPlainPassword } });
const freshCodes = r.json?.backup_codes ?? [];
check('regenerating with the password issues ten new codes',
  r.status === 200 && freshCodes.length === 10 && !freshCodes.some((c) => plainCodes.includes(c)), `${r.status}, ${freshCodes.length} codes`);
r = await call('POST', '/api/v1/auth/reset-password-backup', { body: { username: plainUser, backup_code: plainCodes[2], new_password: 'a fourth long password here' } });
check('an old code that was never spent stops working', r.status === 401, r.status);
r = await call('POST', '/api/v1/auth/reset-password-backup', { body: { username: plainUser, backup_code: freshCodes[0], new_password: 'a fourth long password here' } });
check('a new code works, and nine remain', r.status === 200 && r.json?.backup_codes_remaining === 9, `${r.status}, ${r.json?.backup_codes_remaining}`);
const mailToken = (await login(mailUser, password)).json?.access_token;
r = await call('POST', '/api/v1/profile/backup-codes', { token: mailToken, body: { password } });
check('an account with an email address cannot regenerate backup codes', r.status === 400, r.status);

if (!PG_CONTAINER || !PG_DATABASE) {
  skip('admin routes with a god token');
  skip('password reset with a valid token');
} else {
  const godUser = `acg${stamp}`;
  const target = `act${stamp}`;
  await register(godUser, null);
  await register(target, null);
  psql(`UPDATE users SET role = 'god' WHERE username = '${godUser}'`);
  const god = (await login(godUser)).json?.access_token;
  const godId = claim(god, 'sub');
  check('a god account signs in with the role claim the check expects', claim(god, 'role') === 'god', claim(god, 'role'));

  r = await call('GET', '/api/v1/admin/stats', { token: god });
  check('admin stats are reachable with a god token', r.status === 200, r.status);
  check('stats report numbers, with the god sign-in counted',
    Number.isInteger(r.json?.active_sessions) && r.json.active_sessions >= 1 && r.json?.logins_24h >= 1,
    `active_sessions=${r.json?.active_sessions}, logins_24h=${r.json?.logins_24h}`);
  r = await call('GET', '/api/v1/admin/users?per_page=100', { token: god });
  check('the user list includes accounts without an email', r.status === 200 && r.json?.users?.some((u) => u.email === null), r.status);
  r = await call('GET', '/api/v1/admin/users?per_page=0', { token: god });
  check('a page size of 0 is refused', r.status === 400, r.status);

  const missing = crypto.randomUUID();
  for (const [what, path, body] of [
    ['ban', `/api/v1/admin/users/${missing}/ban`, {}],
    ['unban', `/api/v1/admin/users/${missing}/unban`, undefined],
    ['role', `/api/v1/admin/users/${missing}/role`, { role: 'user' }],
  ]) {
    r = await call('POST', path, { token: god, body });
    check(`${what} of a user that does not exist is 404`, r.status === 404, r.status);
  }

  const targetId = psql(`SELECT id FROM users WHERE username = '${target}'`);
  const targetToken = (await login(target)).json?.access_token;
  r = await call('POST', `/api/v1/admin/users/${targetId}/ban`, { token: god, body: { reason: 'e2e' } });
  check('ban an existing user', r.status === 200, r.status);
  r = await call('GET', '/api/v1/profile', { token: targetToken });
  check("the banned user's token is refused at once", r.status === 401, r.status);
  r = await login(target);
  check('the banned user cannot sign in', r.status !== 200, r.status);
  r = await call('POST', `/api/v1/admin/users/${godId}/ban`, { token: god, body: {} });
  check('an administrator cannot ban their own account', r.status === 400, r.status);
  r = await call('POST', `/api/v1/admin/users/${targetId}/unban`, { token: god });
  check('unban the user', r.status === 200, r.status);
  r = await call('POST', `/api/v1/admin/users/${targetId}/unban`, { token: god });
  check('unbanning an account that is not banned is refused', r.status === 400, r.status);
  const targetAgain = (await login(target)).json?.access_token;
  check('the unbanned user signs in again', !!targetAgain, targetAgain ? 'tokens' : 'none');
  r = await call('POST', `/api/v1/admin/users/${targetId}/role`, { token: god, body: { role: 'moderator' } });
  check('change the role of an existing user', r.status === 200, r.status);
  r = await call('GET', '/api/v1/profile', { token: targetAgain });
  check("the user's token from before the role change is refused", r.status === 401, r.status);
  check('the role is stored', psql(`SELECT role::text FROM users WHERE id = '${targetId}'`) === 'moderator', 'checked');

  r = await call('GET', '/api/v1/admin/audit?per_page=100', { token: god });
  const mine = (r.json?.logs ?? []).filter((l) => l.details?.target_user_id === targetId);
  check('ban, unban and role change are each audited under the god account',
    ['user_banned', 'user_unbanned', 'user_role_changed'].every((a) => mine.some((l) => l.action === a && l.user_id === godId)),
    mine.map((l) => l.action).join(','));
  check('no audit row names the nil user id',
    psql(`SELECT COUNT(*) FROM audit_log WHERE user_id = '00000000-0000-0000-0000-000000000000'`) === '0', 'checked');
  r = await call('POST', '/api/v1/admin/tokens/transfer', { token: god, body: {} });
  check('admin CGT transfer still refuses with 501', r.status === 501, r.status);

  // Password reset by token. Two live sessions exist first; a reset must end both.
  const before = [(await login(mailUser)).json?.access_token, (await login(mailUser)).json?.access_token];
  check('two live sessions before the reset', before.every(Boolean), 'signed in twice');
  const resetToken = `e2e${stamp}${Math.random().toString(16).slice(2)}`;
  psql(`INSERT INTO password_resets (user_id, token, expires_at) SELECT id, '${resetToken}', NOW() + INTERVAL '1 hour' FROM users WHERE username = '${mailUser}'`);
  const newPassword = 'another quite long password';
  r = await call('POST', '/api/v1/auth/reset-password', { body: { token: resetToken, new_password: newPassword } });
  check('reset-password with a valid token succeeds', r.status === 200, `${r.status} ${r.text}`);
  for (const [i, token] of before.entries()) {
    r = await call('GET', '/api/v1/profile', { token });
    check(`session ${i + 1} from before the reset is refused at once`, r.status === 401, r.status);
  }
  check('the new password signs in', (await login(mailUser, newPassword)).status === 200, 'checked');
  check('the old password no longer does', (await login(mailUser)).status !== 200, 'checked');
  r = await call('POST', '/api/v1/auth/reset-password', { body: { token: resetToken, new_password: 'yet another long password' } });
  check('the same token cannot be used twice', r.status === 400, r.status);
}

console.log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
