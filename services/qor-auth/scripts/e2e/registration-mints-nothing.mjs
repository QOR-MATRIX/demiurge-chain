// End-to-end check: registration makes no call to any chain and claims no DMRG (migration inventory R-3).
//
// Stands up a fake JSON-RPC node on 9944 (the old default) and 59944, and counts every request either
// receives. The fake node answers every call with success, so a build that still called the faucet
// would believe it minted. To test a build that still reads BLOCKCHAIN_RPC_URL, start it with
// BLOCKCHAIN_RPC_URL=http://127.0.0.1:59944.
//
//   node scripts/e2e/registration-mints-nothing.mjs [base url, default http://127.0.0.1:3100]
//
// Needs qor-auth running against Postgres and Redis, and Node 22+. Exits non-zero if any check fails.
import { createServer } from 'node:http';
import { refuseRealEmail } from './_guard.mjs';

const BASE = process.argv[2] ?? 'http://127.0.0.1:3100';
await refuseRealEmail(BASE);
const seen = [];

function fakeNode(port) {
  return new Promise((resolve) => {
    const server = createServer((req, res) => {
      let body = '';
      req.on('data', (c) => (body += c));
      req.on('end', () => {
        seen.push({ port, body });
        res.setHeader('content-type', 'application/json');
        res.end(JSON.stringify({ jsonrpc: '2.0', id: 1, result: { success: true, amount: '100', message: 'ok' } }));
      });
    });
    server.on('error', (e) => {
      console.log(`note: fake node could not listen on ${port}: ${e.code}`);
      resolve(null);
    });
    server.listen(port, '127.0.0.1', () => resolve(server));
  });
}

let passed = 0;
let failed = 0;
function check(name, ok, detail) {
  if (ok) passed++;
  else failed++;
  console.log(`${ok ? 'PASS' : 'FAIL'}  ${name}  (${detail})`);
}

const servers = [await fakeNode(9944), await fakeNode(59944)];
check('fake chain node listening on 59944', !!servers[1], servers[1] ? 'up' : 'down');

async function register(body) {
  const res = await fetch(`${BASE}/api/v1/auth/register`, {
    method: 'POST',
    headers: { 'content-type': 'application/json' },
    body: JSON.stringify(body),
  });
  return { status: res.status, json: await res.json().catch(() => null) };
}

// Every name the currency answers to, because this check is worthless if it looks for a name
// nothing uses any more. The ticker is `CGT` (ADR-045, superseding ADR-034, which had made it
// `DMRG`). BOTH markers stay, deliberately: a check that tracked only the current ticker would
// have passed while testing nothing across either change. `cgt` also survives in identifiers such as
// `starter_cgt_minted` until the Substrate implementation replaces them. A response naming either
// would be claiming a balance this route must never create.
//
// If the ticker changes again, add it here in the same change. A stale list still passes, and a
// passing check that tests nothing is worse than no check.
const CURRENCY_MARKERS = [/dmrg/i, /cgt/i];

const stamp = Date.now() % 1e8;
const password = 'correct horse battery staple';

const noEmail = await register({ username: `nomint${stamp}`, password });
check('register without email succeeds', noEmail.status === 201, noEmail.status);
const withEmail = await register({ username: `nomintm${stamp}`, password, email: `nomint${stamp}@example.invalid` });
check('register with email succeeds', withEmail.status === 201, withEmail.status);

await new Promise((r) => setTimeout(r, 500));
const claims = seen.filter((s) => s.body.includes('claimStarter')).length;
check('registration sent nothing to any chain node', seen.length === 0, `${seen.length} request(s), ${claims} claimStarter`);

for (const [label, r] of [['without email', noEmail], ['with email', withEmail]]) {
  const json = r.json ?? {};
  check(`response ${label} makes no minting claim`, !('starter_cgt_minted' in json), `starter_cgt_minted ${'starter_cgt_minted' in json ? 'present' : 'absent'}`);
  check(`response ${label} mentions no currency`, !CURRENCY_MARKERS.some((m) => m.test(JSON.stringify(json))), json.message ?? '');
}

for (const s of servers) s?.close();
console.log(`RESULT: ${passed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
