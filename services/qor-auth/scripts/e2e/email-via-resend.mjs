// End-to-end check: with email configured, an address is verified and a forgotten password is reset
// through Resend, so an account that gave an email is never locked out of recovery.
//
// Runs a stand-in for Resend's `POST /emails` on FAKE_RESEND_PORT (default 59925) and reads the links
// out of the messages it receives. Start qor-auth pointing at it:
//
//   RESEND_API_KEY=re_e2e_key EMAIL_FROM="Demiurge-Cloud <noreply@example.invalid>" \
//   RESEND_API_URL=http://127.0.0.1:59925 BASE_URL=https://example.invalid  <start qor-auth>
//   node scripts/e2e/email-via-resend.mjs [base url, default http://127.0.0.1:3100]
//
// This checks what QOR ID sends to Resend, not delivery by Resend itself. Needs Postgres, Redis and
// Node 22+. The lapsed-link part makes a token lapse with docker exec, and runs only when
// PG_CONTAINER and PG_DATABASE are set; otherwise it is reported as SKIP, never as a pass.
//
// It also opens the links as pages, as a mail scanner would, and checks that only the button spends
// a token. The webhook part signs bounce and complaint reports as Resend does; it runs only when
// RESEND_WEBHOOK_SECRET is set to the secret the service has, and is reported as SKIP otherwise.
// Exits non-zero if any check fails.
import { execSync } from 'node:child_process';
import { createServer } from 'node:http';
import { refuseRealEmail } from './_guard.mjs';

const BASE = process.argv[2] ?? 'http://127.0.0.1:3100';
await refuseRealEmail(BASE);
const PORT = Number(process.env.FAKE_RESEND_PORT ?? 59925);
const KEY = process.env.RESEND_API_KEY ?? 're_e2e_key';
const FROM = process.env.EMAIL_FROM ?? 'Demiurge-Cloud <noreply@example.invalid>';

let passed = 0;
let failed = 0;
const check = (name, ok, detail) => {
  if (ok) passed++;
  else failed++;
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}  (${detail})`);
};

const inbox = [];
const fake = createServer((req, res) => {
  let body = '';
  req.on('data', (c) => (body += c));
  req.on('end', () => {
    let json = null;
    try { json = JSON.parse(body); } catch {}
    inbox.push({ path: req.url, auth: req.headers.authorization, body: json });
    res.setHeader('content-type', 'application/json');
    res.end(JSON.stringify({ id: `fake-${inbox.length}` }));
  });
});
await new Promise((resolve) => fake.listen(PORT, '127.0.0.1', resolve));

async function call(method, path, body) {
  const res = await fetch(BASE + path, {
    method,
    headers: { 'content-type': 'application/json' },
    body: body === undefined ? undefined : JSON.stringify(body),
  });
  const text = await res.text();
  let json = null;
  try { json = JSON.parse(text); } catch {}
  return { status: res.status, json, text };
}

const messagesTo = (address) => inbox.filter((m) => m.body?.to?.includes(address));
async function waitFor(predicate, ms = 5000) {
  const until = Date.now() + ms;
  while (Date.now() < until) {
    const found = predicate();
    if (found) return found;
    await new Promise((r) => setTimeout(r, 50));
  }
  return predicate();
}

const stamp = Date.now() % 1e7;
const username = `mail${stamp}`;
const address = `${username}@example.invalid`;
const password = 'correct horse battery staple';

let r = await call('POST', '/api/v1/auth/register', { username, password, email: address });
check('register with an email', r.status === 201 && r.json?.email_verified === false, `${r.status}, verified=${r.json?.email_verified}`);
check('the response asks the user to verify, because a message was sent', /verify your email/i.test(r.json?.message ?? ''), r.json?.message);

const verification = await waitFor(() => messagesTo(address).find((m) => /verify-email\?token=/.test(m.body?.html ?? '')));
check('a verification message reached Resend', !!verification, verification ? 'received' : 'none');
check('it was sent to Resend\'s /emails with the configured key', verification?.path === '/emails' && verification?.auth === `Bearer ${KEY}`, `${verification?.path} ${verification?.auth ? 'Bearer …' : 'no auth'}`);
check('from the configured sender', verification?.body?.from === FROM, verification?.body?.from);
const verifyToken = verification?.body?.html?.match(/verify-email\?token=([0-9a-f]+)/)?.[1];

// Before verification, the address gets no reset link.
r = await call('POST', '/api/v1/auth/forgot-password', { identifier: address });
const unverifiedAnswer = r.text;
check('forgot-password is accepted for the unverified address', r.status === 200, r.status);
await new Promise((resolve) => setTimeout(resolve, 1000));
check('but no reset link is sent to an address not yet verified',
  !messagesTo(address).some((m) => /reset-password\?token=/.test(m.body?.html ?? '')), 'none sent');
r = await call('POST', '/api/v1/auth/forgot-password', { identifier: `nobody${stamp}` });
check('an unknown identifier gets the same answer', r.status === 200 && r.text === unverifiedAnswer, 'compared bodies');

r = await call('POST', '/api/v1/auth/verify-email', { token: verifyToken });
check('the link from the message verifies the address', r.status === 200, r.status);

r = await call('POST', '/api/v1/auth/forgot-password', { identifier: address });
check('forgot-password after verification', r.status === 200 && r.text === unverifiedAnswer, `${r.status}, same answer`);
const reset = await waitFor(() => messagesTo(address).find((m) => /reset-password\?token=/.test(m.body?.html ?? '')));
check('a reset link reached Resend for the verified address', !!reset, reset ? 'received' : 'none');
const resetToken = reset?.body?.html?.match(/reset-password\?token=([0-9a-f]+)/)?.[1];

const newPassword = 'a completely new password';
r = await call('POST', '/api/v1/auth/reset-password', { token: resetToken, new_password: newPassword });
check('the reset link resets the password', r.status === 200, `${r.status} ${r.text}`);
r = await call('POST', '/api/v1/auth/login', { identifier: username, password: newPassword });
check('the new password signs in', r.status === 200, r.status);

// ---------------------------------------------------------------- an account whose link lapsed
const { PG_CONTAINER, PG_DATABASE, PG_USER = 'qor' } = process.env;
if (!PG_CONTAINER || !PG_DATABASE) {
  console.log('SKIP  recovery through resend-verification  (set PG_CONTAINER and PG_DATABASE to run it)');
} else {
  const psql = (sql) =>
    execSync(`docker exec ${PG_CONTAINER} psql -U ${PG_USER} -d ${PG_DATABASE} -tA -c "${sql}"`).toString().trim();
  const lapsed = `lapsed${stamp}`;
  const lapsedAddress = `${lapsed}@example.invalid`;
  r = await call('POST', '/api/v1/auth/register', { username: lapsed, password, email: lapsedAddress });
  check('register the account whose link will lapse', r.status === 201, r.status);
  await waitFor(() => messagesTo(lapsedAddress).length > 0);

  // As if it had registered while email was off, two days ago: a token that lapsed.
  psql(`UPDATE users SET email_verification_token = 'lapsed-token', email_verification_expires_at = NOW() - INTERVAL '2 days', email_verification_sent_at = NOW() - INTERVAL '2 days', email_verification_count_since = NOW() - INTERVAL '2 days' WHERE username = '${lapsed}'`);
  const before = messagesTo(lapsedAddress).length;

  r = await call('POST', '/api/v1/auth/verify-email', { token: 'lapsed-token' });
  check('the lapsed link no longer verifies', r.status === 400, r.status);
  r = await call('POST', '/api/v1/auth/forgot-password', { identifier: lapsedAddress });
  await new Promise((resolve) => setTimeout(resolve, 800));
  check('and the unverified account gets no reset link', messagesTo(lapsedAddress).length === before, 'none sent');

  r = await call('POST', '/api/v1/auth/resend-verification', { identifier: lapsedAddress });
  const resendAnswer = r.text;
  check('resend-verification is accepted', r.status === 200, r.status);
  const fresh = await waitFor(() => messagesTo(lapsedAddress).slice(before).find((m) => /verify-email\?token=/.test(m.body?.html ?? '')));
  check('a new verification message reached Resend', !!fresh, fresh ? 'received' : 'none');
  const freshToken = fresh?.body?.html?.match(/verify-email\?token=([0-9a-f]+)/)?.[1];
  check('with a new token', !!freshToken && freshToken !== 'lapsed-token', 'compared');

  r = await call('POST', '/api/v1/auth/resend-verification', { identifier: `nobody${stamp}@example.invalid` });
  check('an unknown identifier gets the same answer', r.status === 200 && r.text === resendAnswer, 'compared bodies');
  r = await call('POST', '/api/v1/auth/resend-verification', { identifier: lapsed });
  check('a second request is accepted with the same answer', r.status === 200 && r.text === resendAnswer, 'compared bodies');
  await new Promise((resolve) => setTimeout(resolve, 1000));
  check('but sends nothing within five minutes', messagesTo(lapsedAddress).length === before + 1, `${messagesTo(lapsedAddress).length - before} new`);

  r = await call('POST', '/api/v1/auth/verify-email', { token: freshToken });
  check('the new link verifies the address', r.status === 200, r.status);
  r = await call('POST', '/api/v1/auth/forgot-password', { identifier: lapsedAddress });
  const recovery = await waitFor(() => messagesTo(lapsedAddress).find((m) => /reset-password\?token=/.test(m.body?.html ?? '')));
  check('the recovered account now receives a reset link', !!recovery, recovery ? 'received' : 'none');
}

// ---------------------------------------------------------------- adding an email to an account without one
async function authed(path, token, body) {
  const res = await fetch(BASE + path, {
    method: 'POST',
    headers: { 'content-type': 'application/json', ...(token ? { authorization: `Bearer ${token}` } : {}) },
    body: JSON.stringify(body),
  });
  const text = await res.text();
  let json = null;
  try { json = JSON.parse(text); } catch {}
  return { status: res.status, json, text };
}

const noMail = `nomail${stamp}`;
const added = `${noMail}@example.invalid`;
r = await call('POST', '/api/v1/auth/register', { username: noMail, password });
const noMailCodes = r.json?.backup_codes ?? [];
check('register an account without an email', r.status === 201 && noMailCodes.length === 10, `${r.status}, ${noMailCodes.length} codes`);
const noMailToken = (await call('POST', '/api/v1/auth/login', { identifier: noMail, password })).json?.access_token;

r = await authed('/api/v1/profile/email', null, { email: added, password });
check('adding an email without signing in is refused', r.status === 401, r.status);
r = await authed('/api/v1/profile/email', noMailToken, { email: added, password: 'not the password at all' });
check('adding an email with a signed-in session but the wrong password is refused', r.status === 401, r.status);
check('and nothing was sent', messagesTo(added).length === 0, `${messagesTo(added).length} sent`);

r = await authed('/api/v1/profile/email', noMailToken, { email: added, password });
check('adding an email with the session and the password is accepted', r.status === 200 && r.json?.current_email_notified === false, `${r.status} ${r.text}`);
const addLink = await waitFor(() => messagesTo(added).find((m) => /verify-email\?token=/.test(m.body?.html ?? '')));
check('a confirmation link reached the added address', !!addLink, addLink ? 'received' : 'none');

r = await call('POST', '/api/v1/auth/forgot-password', { identifier: added });
await new Promise((resolve) => setTimeout(resolve, 800));
check('before it is confirmed, the address gets no reset link', !messagesTo(added).some((m) => /reset-password\?token=/.test(m.body?.html ?? '')), 'none sent');

r = await call('POST', '/api/v1/auth/verify-email', { token: addLink?.body?.html?.match(/verify-email\?token=([0-9a-f]+)/)?.[1] });
check('following the link confirms the address', r.status === 200, r.status);
r = await call('POST', '/api/v1/auth/forgot-password', { identifier: added });
const addedReset = await waitFor(() => messagesTo(added).find((m) => /reset-password\?token=/.test(m.body?.html ?? '')));
check('once confirmed, the address receives reset links', !!addedReset, addedReset ? 'received' : 'none');
r = await call('POST', '/api/v1/auth/reset-password-backup', { username: noMail, backup_code: noMailCodes[0], new_password: 'a password set with a backup code' });
check('and the backup codes still work: both routes remain', r.status === 200 && r.json?.backup_codes_remaining === 9, `${r.status}, ${r.json?.backup_codes_remaining}`);

// ---------------------------------------------------------------- changing an existing address
if (!PG_CONTAINER || !PG_DATABASE) {
  console.log('SKIP  changing an existing address  (set PG_CONTAINER and PG_DATABASE to run it)');
} else {
  const psql = (sql) =>
    execSync(`docker exec ${PG_CONTAINER} psql -U ${PG_USER} -d ${PG_DATABASE} -tA -c "${sql}"`).toString().trim();
  const moved = `${noMail}.moved@example.invalid`;
  // The add above counts against the five-minute limit; move it into the past.
  psql(`UPDATE users SET email_verification_sent_at = NOW() - INTERVAL '10 minutes' WHERE username = '${noMail}'`);
  const movingToken = (await call('POST', '/api/v1/auth/login', { identifier: noMail, password: 'a password set with a backup code' })).json?.access_token;
  const oldBefore = messagesTo(added).length;
  r = await authed('/api/v1/profile/email', movingToken, { email: moved, password: 'a password set with a backup code' });
  check('changing the address is accepted, and the old address is told', r.status === 200 && r.json?.current_email_notified === true, `${r.status} ${r.text}`);
  const notice = await waitFor(() => messagesTo(added).slice(oldBefore).find((m) => /change of email address/i.test(m.body?.subject ?? '')));
  check('a notice reached the old address', !!notice, notice ? 'received' : 'none');
  check('and it carries no link', !!notice && !/token=/.test(notice.body?.html ?? ''), 'checked');
  r = await authed('/api/v1/profile/email', movingToken, { email: `${noMail}.again@example.invalid`, password: 'a password set with a backup code' });
  check('a second change within five minutes is refused with 429', r.status === 429, r.status);
  const moveLink = await waitFor(() => messagesTo(moved).find((m) => /verify-email\?token=/.test(m.body?.html ?? '')));
  r = await call('POST', '/api/v1/auth/verify-email', { token: moveLink?.body?.html?.match(/verify-email\?token=([0-9a-f]+)/)?.[1] });
  check('following the link moves the account to the new address', r.status === 200, r.status);
  check('the account now holds the new address', psql(`SELECT email FROM users WHERE username = '${noMail}'`) === moved, 'checked');
}

// ---------------------------------------------------------------- the pages the links open
// A mail scanner may open a link before the person does, so opening one must change nothing.
const pathOf = (link) => { try { const u = new URL(link); return u.pathname + u.search; } catch { return null; } };
const linkIn = (message, kind) => message?.body?.html?.match(new RegExp(`https?://[^"<\\s]+/${kind}\\?token=[0-9a-f]+`))?.[0];
async function page(method, path, form) {
  const res = await fetch(BASE + path, {
    method,
    redirect: 'manual',
    headers: form ? { 'content-type': 'application/x-www-form-urlencoded' } : {},
    body: form ? new URLSearchParams(form).toString() : undefined,
  });
  return { status: res.status, headers: res.headers, text: await res.text() };
}
const resetLinksTo = (to) => messagesTo(to).filter((m) => /reset-password\?token=/.test(m.body?.html ?? '')).length;

const pageUser = `page${stamp}`;
const pageAddress = `${pageUser}@example.invalid`;
r = await call('POST', '/api/v1/auth/register', { username: pageUser, password, email: pageAddress });
check('register the account whose links are opened as pages', r.status === 201, r.status);
const pageVerification = await waitFor(() => messagesTo(pageAddress).find((m) => /verify-email\?token=/.test(m.body?.html ?? '')));
const verifyPath = pathOf(linkIn(pageVerification, 'verify-email'));
check('the verification message links to the verification page', !!verifyPath && verifyPath.startsWith('/verify-email?token='), verifyPath ? 'found' : 'none');

let p;
for (let i = 0; i < 3; i++) p = await page('GET', verifyPath);
check('opening the link, three times as a scanner might, shows a button', p.status === 200 && p.text.includes('action="/verify-email"') && p.text.includes('Confirm email address'), p.status);
check('the page runs no script, cannot be framed or cached, and sends no referrer',
  !p.text.includes('<script') && p.headers.get('x-frame-options') === 'DENY' && p.headers.get('cache-control') === 'no-store'
    && p.headers.get('referrer-policy') === 'no-referrer' && /frame-ancestors 'none'/.test(p.headers.get('content-security-policy') ?? ''),
  'headers checked');
await call('POST', '/api/v1/auth/forgot-password', { identifier: pageAddress });
await new Promise((resolve) => setTimeout(resolve, 800));
check('opening it confirmed nothing: the address still gets no reset link', resetLinksTo(pageAddress) === 0, `${resetLinksTo(pageAddress)} sent`);

p = await page('POST', '/verify-email', { token: verifyPath?.split('token=')[1] });
check('pressing the button confirms the address', p.status === 200 && p.text.includes('Email address confirmed'), p.status);
p = await page('GET', verifyPath);
check('the used link says it no longer works and offers a new one', p.status === 410 && p.text.includes('This link no longer works') && p.text.includes('action="/resend-verification"'), p.status);

await call('POST', '/api/v1/auth/forgot-password', { identifier: pageAddress });
const pageReset = await waitFor(() => messagesTo(pageAddress).find((m) => /reset-password\?token=/.test(m.body?.html ?? '')));
const resetPath = pathOf(linkIn(pageReset, 'reset-password'));
const pageResetToken = resetPath?.split('token=')[1];
for (let i = 0; i < 3; i++) p = await page('GET', resetPath);
check('opening the reset link, three times, shows the password form', p.status === 200 && p.text.includes('name="new_password"'), p.status);
p = await page('POST', '/reset-password', { token: pageResetToken, new_password: 'one new password here', confirm_password: 'another new password' });
check('two different passwords are refused', p.status === 400 && p.text.includes('different'), p.status);
p = await page('POST', '/reset-password', { token: pageResetToken, new_password: 'short', confirm_password: 'short' });
check('a short password is refused', p.status === 400, p.status);
r = await call('POST', '/api/v1/auth/login', { identifier: pageUser, password });
check('neither opening the link nor a refused form changed the password', r.status === 200, r.status);
const pagePassword = 'a password set on the reset page';
p = await page('POST', '/reset-password', { token: pageResetToken, new_password: pagePassword, confirm_password: pagePassword });
check('pressing the button with a new password sets it', p.status === 200 && p.text.includes('Password changed'), p.status);
r = await call('POST', '/api/v1/auth/login', { identifier: pageUser, password: pagePassword });
check('the new password signs in', r.status === 200, r.status);
p = await page('GET', resetPath);
check('the used reset link says it no longer works and offers a new one', p.status === 410 && p.text.includes('action="/forgot-password"'), p.status);
let resetsBefore = resetLinksTo(pageAddress);
p = await page('POST', '/forgot-password', { identifier: pageUser });
check('asking for a new link on that page answers as the API does', p.status === 200 && p.text.includes('Check your email'), p.status);
const another = await waitFor(() => resetLinksTo(pageAddress) > resetsBefore);
check('and a new reset link is sent', !!another, another ? 'received' : 'none');

// ---------------------------------------------------------------- bounce and complaint reports from Resend
const WEBHOOK_SECRET = process.env.RESEND_WEBHOOK_SECRET;
if (!WEBHOOK_SECRET) {
  console.log('SKIP  Resend webhooks  (set RESEND_WEBHOOK_SECRET to the secret the service has, to run them)');
} else {
  const { createHmac } = await import('node:crypto');
  const sign = (secret, id, ts, body) =>
    `v1,${createHmac('sha256', Buffer.from(secret.replace(/^whsec_/, ''), 'base64')).update(`${id}.${ts}.${body}`).digest('base64')}`;
  let hookCount = 0;
  async function hook(event, { id = `msg_e2e_${stamp}_${++hookCount}`, secret = WEBHOOK_SECRET, ts = Math.floor(Date.now() / 1000), unsigned = false } = {}) {
    const body = JSON.stringify(event);
    const headers = { 'content-type': 'application/json' };
    if (!unsigned) Object.assign(headers, { 'svix-id': id, 'svix-timestamp': String(ts), 'svix-signature': sign(secret, id, ts, body) });
    const res = await fetch(`${BASE}/api/v1/webhooks/resend`, { method: 'POST', headers, body });
    const text = await res.text();
    let json = null;
    try { json = JSON.parse(text); } catch {}
    return { status: res.status, json };
  }
  const bounced = (to, type) => ({
    type: 'email.bounced',
    created_at: new Date().toISOString(),
    data: { email_id: 'e2e-email', from: FROM, to: [to], subject: 'Reset your QOR ID password', bounce: { message: 'e2e', subType: 'General', type } },
  });

  r = await hook(bounced(pageAddress, 'Permanent'), { unsigned: true });
  check('an unsigned report is refused', r.status === 401, r.status);
  r = await hook(bounced(pageAddress, 'Permanent'), { secret: 'whsec_plJ3nmyCDGBKInavdOK15jsl' });
  check('a report signed with another secret is refused', r.status === 401, r.status);
  r = await hook(bounced(pageAddress, 'Permanent'), { ts: Math.floor(Date.now() / 1000) - 600 });
  check('a correctly signed report ten minutes old is refused', r.status === 401, r.status);
  r = await hook(bounced(pageAddress, 'Temporary'));
  check('a signed temporary bounce is accepted', r.status === 200, r.status);
  resetsBefore = resetLinksTo(pageAddress);
  await call('POST', '/api/v1/auth/forgot-password', { identifier: pageUser });
  const stillSent = await waitFor(() => resetLinksTo(pageAddress) > resetsBefore);
  check('none of those marked the address: reset links still go to it', !!stillSent, stillSent ? 'received' : 'none');

  const once = `msg_e2e_${stamp}_permanent`;
  r = await hook(bounced(pageAddress.toUpperCase(), 'Permanent'), { id: once });
  check('a signed permanent bounce is accepted', r.status === 200 && r.json?.received === true, r.status);
  r = await hook(bounced(pageAddress.toUpperCase(), 'Permanent'), { id: once });
  check('the same delivery again is acted on once', r.status === 200 && r.json?.duplicate === true, JSON.stringify(r.json));
  resetsBefore = resetLinksTo(pageAddress);
  await call('POST', '/api/v1/auth/forgot-password', { identifier: pageUser });
  await new Promise((resolve) => setTimeout(resolve, 1000));
  check('after a permanent bounce, no reset link is sent to the address', resetLinksTo(pageAddress) === resetsBefore, `${resetLinksTo(pageAddress) - resetsBefore} sent`);
  const pageAccess = (await call('POST', '/api/v1/auth/login', { identifier: pageUser, password: pagePassword })).json?.access_token;
  const profile = await fetch(`${BASE}/api/v1/profile`, { headers: { authorization: `Bearer ${pageAccess}` } }).then((res) => res.json()).catch(() => null);
  check('the profile shows the address is undeliverable', profile?.email_deliverable === false, JSON.stringify(profile?.email_deliverable));

  resetsBefore = resetLinksTo(address);
  r = await hook({ type: 'email.complained', created_at: new Date().toISOString(), data: { email_id: 'e2e-email-2', from: FROM, to: [address], subject: 'Confirm your email address for QOR ID' } });
  check('a signed complaint is accepted', r.status === 200, r.status);
  await call('POST', '/api/v1/auth/forgot-password', { identifier: username });
  await new Promise((resolve) => setTimeout(resolve, 1000));
  check('after a complaint, nothing more is sent to that address', resetLinksTo(address) === resetsBefore, `${resetLinksTo(address) - resetsBefore} sent`);
}

fake.close();
console.log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
