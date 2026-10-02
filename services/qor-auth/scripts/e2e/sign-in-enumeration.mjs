// End-to-end check: sign-in discloses nothing about which accounts exist. An unknown username, an
// unknown email, a wrong password, a locked account and a banned account all get the same answer,
// and an unknown username takes about as long to refuse as a wrong password on a real account.
//
//   node scripts/e2e/sign-in-enumeration.mjs [base url, default http://127.0.0.1:3100]
//
// Needs qor-auth running against Postgres and Redis, and Node 22+. The banned-account check writes
// to the database with docker exec and runs only when PG_CONTAINER and PG_DATABASE are set;
// otherwise it is reported as SKIP, never as a pass. Timing is measured on this machine, so the
// bound is generous. Exits non-zero if any check fails.
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

async function call(path, body) {
  const started = performance.now();
  const res = await fetch(BASE + path, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  const text = await res.text();
  return { status: res.status, text, ms: performance.now() - started };
}

const stamp = Date.now() % 1e7;
const password = 'correct horse battery staple';
const register = (username) => call('/api/v1/auth/register', { username, password });
const login = (identifier, pw) => call('/api/v1/auth/login', { identifier, password: pw });

const real = `sia${stamp}`;
const locked = `sil${stamp}`;
const banned = `sib${stamp}`;
for (const username of [real, locked, banned]) {
  const r = await register(username);
  if (r.status !== 201) throw new Error(`register ${username}: ${r.status} ${r.text}`);
}

// Lock one account: five wrong passwords, the default limit.
for (let i = 0; i < 5; i++) await login(locked, 'not the password at all');

const refusals = {
  'an unknown username': await login(`nobody${stamp}`, password),
  'an unknown email address': await login(`nobody${stamp}@example.invalid`, password),
  'a wrong password on a real account': await login(real, 'not the password at all'),
  'the right password on a locked account': await login(locked, password),
};
if (PG_CONTAINER && PG_DATABASE) {
  execSync(`docker exec ${PG_CONTAINER} psql -U ${PG_USER} -d ${PG_DATABASE} -tA -c "UPDATE users SET status = 'banned' WHERE username = '${banned}'"`);
  refusals['the right password on a banned account'] = await login(banned, password);
} else {
  console.log('SKIP  the right password on a banned account  (set PG_CONTAINER and PG_DATABASE to run it)');
}

const first = Object.values(refusals)[0];
for (const [what, r] of Object.entries(refusals)) {
  check(`${what} is refused with 401`, r.status === 401, r.status);
  check(`${what} gets the same body as an unknown username`, r.text === first.text, r.text);
}
const disclosing = Object.values(refusals).filter((r) => /not found|locked|not active|sign up/i.test(r.text));
check('no refusal says whether the account exists, is locked or is active', disclosing.length === 0, first.text);

// Timing: median of repeated attempts. The unknown username is compared with a wrong password on
// a fresh real account, which stays under the lockout limit.
const timed = `sit${stamp}`;
await register(timed);
const median = (xs) => xs.sort((a, b) => a - b)[Math.floor(xs.length / 2)];
const unknownMs = [];
const wrongMs = [];
for (let i = 0; i < 4; i++) {
  unknownMs.push((await login(`ghost${stamp}${i}`, password)).ms);
  wrongMs.push((await login(timed, `wrong password number ${i}`)).ms);
}
const unknownMedian = median(unknownMs);
const wrongMedian = median(wrongMs);
const ratio = unknownMedian / wrongMedian;
check('an unknown username takes about as long to refuse as a wrong password',
  ratio > 1 / 1.5 && ratio < 1.5, `unknown ${unknownMedian.toFixed(0)} ms, wrong password ${wrongMedian.toFixed(0)} ms, ratio ${ratio.toFixed(2)}`);

const ok = await login(real, password);
check('the real account still signs in with its password', ok.status === 200, ok.status);

console.log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
