// Is each QOR ID on this PC a profile of its own, with its own wallet, closed completely when it signs out? (ADR-099,
// launcher 0.1.13)
//
// The owner, 18 October 2026: "QOR IDs need to be separate entirely. A new app sign in should be entirely dependent on
// the data for that account. If an account logs out, that should separate the amounts completely." This check serves
// the built frontend in a real rendering engine with a stand-in host holding three profiles, and holds the screens to:
//   - signed out, Who's playing? lists this PC's QOR IDs by name, shows no balance, and opens no wallet;
//   - a QOR ID opens only with its PIN, a short PIN is never sent, and a wrong one is said and opens nothing;
//   - opened, the wallet is that QOR ID's own account and balance;
//   - signing out closes it completely: the host is asked, and nothing of its account or balance stays on screen;
//   - a QOR ID with no PIN chooses one, typed twice, as it signs out;
//   - a new QOR ID is a new wallet, never another QOR ID's account, and the Vault offers no second account;
//   - an account an older wallet's phrase used becomes a QOR ID of its own, by its index.
//
//   npm run build
//   node scripts/check-profiles-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const dist = resolve(process.argv[2] ?? join(dirname(fileURLToPath(import.meta.url)), '..', 'dist'));
if (!existsSync(join(dist, 'index.html'))) {
  console.error(`No build at ${dist}. Run \`npm run build\` first.`);
  process.exit(2);
}
const BROWSER =
  process.env.BROWSER_PATH ??
  [
    'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
    'C:/Program Files/Google/Chrome/Application/chrome.exe',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium',
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
  ].find((p) => existsSync(p));
if (!BROWSER) {
  console.error('No Chromium-family browser found. Set BROWSER_PATH.');
  process.exit(2);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// --- the fixture -------------------------------------------------------------------------------------------------
const GOD = '5GodmodeFixtureAccount000000000000000000000000000';
const YAL = '5YaldobaothFixtureAccount000000000000000000000000';
const OLD = '5UnnamedFixtureAccount0000000000000000000000000000';
const NEW = '5BrandNewFixtureAccount000000000000000000000000000';
const SPLIT = '5SecondOfAnOldPhraseFixture000000000000000000000000';
const PIN = '482913';

const results = [];
const check = (name, pass, detail = '') => {
  results.push({ name, pass });
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}${detail ? `  (${detail})` : ''}`);
};

// --- serve the build -------------------------------------------------------
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.woff2': 'font/woff2', '.woff': 'font/woff', '.ico': 'image/x-icon' };
const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const file = join(dist, path === '/' ? 'index.html' : path.replace(/^\/+/, ''));
  try {
    const body = await readFile(file);
    res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' }).end(body);
  } catch {
    res.writeHead(404).end('not found');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const appUrl = `http://127.0.0.1:${server.address().port}/`;

const userData = await mkdtemp(join(tmpdir(), 'qor-profiles-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--force-device-scale-factor=1.5',
    '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling',
    `--remote-debugging-port=${port}`, `--user-data-dir=${userData}`, 'about:blank',
  ],
  { stdio: 'ignore' },
);
const finish = async () => {
  try { browser.kill(); } catch {}
  try { server.close(); } catch {}
  await rm(userData, { recursive: true, force: true }).catch(() => {});
};

// The stand-in host keeps the profiles and which one is signed in, as profiles.json does.
const HOST_STUB = `
  window.__CALLS__ = [];
  let callbacks = 0;
  const S = {
    signedIn: null,
    profiles: [
      { id: 'p-god', name: 'godmode', address: '${GOD}', has_pin: true, wait_seconds: 0 },
      { id: 'p-yal', name: 'Yaldobaoth', address: '${YAL}', has_pin: true, wait_seconds: 0 },
      { id: 'p-old', name: null, address: '${OLD}', has_pin: false, wait_seconds: 0 },
    ],
  };
  window.__S__ = S;
  const BAL = { '${GOD}': '9,000.00', '${YAL}': '25.00', '${NEW}': '0.00', '${SPLIT}': '57,900.00' };
  const SPARKS = { '${GOD}': '9,000,000', '${YAL}': '25,000', '${NEW}': '0', '${SPLIT}': '57,900,000' };
  const me = () => S.profiles.find((p) => p.id === S.signedIn) ?? null;
  const views = () => S.profiles.map((p) => ({ ...p, signed_in: p.id === S.signedIn }));
  const account = (p) => ({ address: p.address, account_id: '0x' + '01'.repeat(32), path: '', index: 0, label: 'Primary' });
  const vaultNow = () => (me() ? { state: 'unlocked', accounts: [account(me())] } : { state: 'absent' });
  const session = (p) => (p && p.name ? { qor_id: p.name, username: p.name, discriminator: 1, role: 'user', address: p.address, avatar_url: null, username_chosen: true } : null);
  const arrival = () => ({ accounts: [account(me())], address: me().address, session: session(me()), needs_name: !me().name, sign_in_problem: null });
  const refuse = (kind, message) => Promise.reject({ kind, message });
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      window.__CALLS__.push({ cmd, args });
      switch (cmd) {
        case 'plugin:event|listen': return Promise.resolve(args.handler);
        case 'plugin:event|unlisten': return Promise.resolve(null);
        case 'launcher_state': return Promise.resolve({ version: '0.0.0-fixture', profiles: views(), vault: vaultNow(), session: session(me()), chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0', token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' } });
        case 'vault_status': return Promise.resolve(vaultNow());
        case 'profiles_list': return Promise.resolve(views());
        case 'profile_open': {
          const p = S.profiles.find((x) => x.id === args.id);
          if (S.signedIn) return refuse('vault_open', 'signed in');
          if (args.pin !== '${PIN}') return refuse('pin', 'That PIN is not right.');
          S.signedIn = p.id;
          return Promise.resolve(arrival());
        }
        case 'profile_sign_out': {
          const p = me();
          if (!p) return refuse('no_profile', 'none');
          if (!p.has_pin && !args.pin) return refuse('pin', 'Choose a PIN for this QOR ID first, so only you can open it again.');
          if (args.pin) p.has_pin = true;
          S.signedIn = null;
          return Promise.resolve(null);
        }
        case 'profile_set_pin': return Promise.resolve(null);
        case 'vault_generate_phrase': return Promise.resolve('abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about');
        case 'vault_create': {
          if (S.signedIn) return refuse('vault_open', 'signed in');
          S.profiles.push({ id: 'p-new', name: null, address: '${NEW}', has_pin: false, wait_seconds: 0 });
          S.signedIn = 'p-new';
          return Promise.resolve(arrival());
        }
        case 'profile_other_accounts': return Promise.resolve([{ index: 1, address: '${SPLIT}', balance: '57,900.00 CGT' }]);
        case 'profile_split': {
          const from = me();
          if (!from.has_pin && !args.pin) return refuse('pin', 'Choose a PIN first.');
          if (args.pin) from.has_pin = true;
          S.profiles.push({ id: 'p-split', name: 'Barbelo', address: '${SPLIT}', has_pin: false, wait_seconds: 0 });
          S.signedIn = 'p-split';
          return Promise.resolve(arrival());
        }
        case 'cgt_balance': return Promise.resolve({ address: args.address, embers: '0', cgt: '0', display: (BAL[args.address] ?? '0.00') + ' CGT', in_sparks: (SPARKS[args.address] ?? '0') + ' Sparks' });
        case 'cgt_nonce': return Promise.resolve(0);
        case 'qor_restore': return refuse('not_authenticated', 'not signed in');
        case 'touch_vault': case 'vault_lock': case 'vault_set_active': case 'social_disconnect': case 'social_connect': return Promise.resolve(null);
        case 'social_status': return Promise.resolve({ status: { state: 'off' }, welcome: null, endpoint: '' });
        default: return refuse('internal', 'no host in this check');
      }
    },
    transformCallback: () => ++callbacks,
    unregisterCallback: () => {},
    convertFileSrc: (s) => s,
    metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
    plugins: {},
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
`;

try {
  let target;
  for (let i = 0; i < 300 && !target; i++) {
    if (i > 0) await sleep(100);
    try {
      target = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find((t) => t.type === 'page');
    } catch {
      await sleep(100);
    }
  }
  if (!target) throw new Error('The browser did not expose a page to inspect');
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((ok, bad) => { ws.onopen = ok; ws.onerror = bad; });
  let nextId = 0;
  const pending = new Map();
  const events = [];
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); } else if (msg.method) events.push(msg);
  };
  const send = (method, params = {}) => new Promise((ok) => { const id = ++nextId; pending.set(id, ok); ws.send(JSON.stringify({ id, method, params })); });
  const evaluate = async (expression) => {
    const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (r.result.exceptionDetails) throw new Error(`${expression}\n${JSON.stringify(r.result.exceptionDetails)}`);
    return r.result.result.value;
  };
  const eventually = async (expression, ms = 8000) => {
    const deadline = Date.now() + ms;
    for (;;) {
      if (await evaluate(expression)) return true;
      if (Date.now() > deadline) return false;
      await sleep(40);
    }
  };
  const until = async (expression, what) => {
    if (!(await eventually(expression))) throw new Error(`timed out waiting for ${what}: ${expression}`);
  };
  const calls = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)}).map((c) => c.args ?? {})`);
  const text = (selector = 'body') => evaluate(`document.querySelector(${JSON.stringify(selector)})?.textContent ?? ''`);
  const click = (selector) => evaluate(`(() => { const n = document.querySelector(${JSON.stringify(selector)}); if (!n) return false; n.click(); return true; })()`);
  const clickText = (scope, label) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => b.textContent.trim().startsWith(${JSON.stringify(label)})); if (!n) return false; n.click(); return true; })()`);
  const type = (selector, value) => evaluate(`(() => {
    const el = document.querySelector(${JSON.stringify(selector)});
    if (!el) return false;
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(el, ${JSON.stringify(value)});
    el.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);
  const disabled = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)})?.disabled ?? null`);
  const settings = async () => {
    await clickText('nav[aria-label="Primary"] button', 'Settings');
    await until(`Boolean(document.querySelector('[data-profile-panel]'))`, 'the profile panel in Settings');
  };

  await send('Page.enable');
  await send('Runtime.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await send('Page.bringToFront');
  await send('Emulation.setFocusEmulationEnabled', { enabled: true });

  // --- signed out: Who's playing? ------------------------------------------------------------------------------------
  await until(`Boolean(document.querySelector('[data-gate-who]'))`, "Who's playing?");
  const names = await evaluate(`[...document.querySelectorAll('[data-gate-profile]')].map((b) => b.textContent.trim())`);
  check("signed out, Who's playing? lists this PC's QOR IDs by name", names.some((n) => n.startsWith('godmode')) && names.some((n) => n.startsWith('Yaldobaoth')) && names.length === 3, JSON.stringify(names));
  const gateText = await text();
  check('and shows no balance', !/CGT|9,000|57,900/.test(gateText) && (await calls('cgt_balance')).length === 0);
  check('and opens no wallet on its own', (await calls('vault_unlock')).length === 0 && (await calls('profile_open')).length === 0);

  // --- a QOR ID opens only with its PIN --------------------------------------------------------------------------
  await click('[data-gate-profile="p-god"]');
  await until(`Boolean(document.querySelector('[data-gate-pin] [data-pin]'))`, 'the PIN field');
  check("choosing godmode asks for godmode's PIN", (await text('[data-gate-pin]')).includes('godmode'));
  await type('[data-gate-pin] [data-pin]', '4829');
  check('a PIN shorter than six digits cannot be sent', (await disabled('[data-gate-open]')) === true);
  await type('[data-gate-pin] [data-pin]', '000000');
  await click('[data-gate-open]');
  check('a wrong PIN is said, and opens nothing', await eventually(`[...document.querySelectorAll('[role="alert"]')].some((n) => n.textContent.includes('not right'))`)
    && !(await evaluate(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`)));
  check('only the PIN typed was sent, for the QOR ID chosen', JSON.stringify(await calls('profile_open')) === JSON.stringify([{ id: 'p-god', pin: '000000' }]), JSON.stringify(await calls('profile_open')));
  await type('[data-gate-pin] [data-pin]', PIN);
  await click('[data-gate-open]');
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell after the right PIN');
  check("opened, the wallet is godmode's own account and balance", await eventually(`(document.querySelector('nav[aria-label="Primary"]')?.textContent ?? '').includes('9,000.00 CGT')`) && !(await text()).includes('25.00 CGT'));

  // --- the Vault holds one account ---------------------------------------------------------------------------------
  await clickText('nav[aria-label="Primary"] button', 'Vault');
  await until(`[...document.querySelectorAll('.eyebrow')].some((n) => n.textContent.trim() === 'Account')`, 'the Vault');
  check('the Vault offers no second account', !(await evaluate(`Boolean(document.querySelector('[title="Derive another account from this vault"]'))`))
    && (await evaluate(`document.querySelectorAll('aside li').length`)) === 1);
  check('the Vault shows the balance in Sparks too, a thousandth of a CGT each (ADR-101)',
    await eventually(`(document.querySelector('[data-vault-sparks]')?.textContent ?? '').trim() === '9,000,000 Sparks'`), await text('[data-vault-sparks]'));

  // --- signing out closes it completely ----------------------------------------------------------------------------
  await settings();
  await click('[data-profile-sign-out]');
  await until(`Boolean(document.querySelector('[data-profile-confirm-sign-out]'))`, 'the sign-out confirmation');
  check('a QOR ID with a PIN signs out without choosing another', !(await evaluate(`Boolean(document.querySelector('[data-profile-leaving] [data-pin]'))`)));
  await click('[data-profile-confirm-sign-out]');
  await until(`Boolean(document.querySelector('[data-gate-who]'))`, "Who's playing? after signing out");
  check('signing out asks the host, with no PIN to set', JSON.stringify(await calls('profile_sign_out')) === JSON.stringify([{ pin: null }]), JSON.stringify(await calls('profile_sign_out')));
  const after = await text();
  check("nothing of godmode's account or balance stays on screen", !after.includes('9,000') && !after.includes(GOD.slice(0, 10)) && !after.includes('CGT'));

  // --- a QOR ID with no PIN on this PC --------------------------------------------------------------------------
  await click('[data-gate-profile="p-old"]');
  await until(`Boolean(document.querySelector('[data-gate-pin]'))`, 'the PIN step');
  check('a QOR ID with no PIN here cannot be opened, and says to restore it', !(await evaluate(`Boolean(document.querySelector('[data-gate-open]'))`))
    && (await text('[data-gate-pin]')).includes('recovery phrase'));
  await clickText('[data-gate-pin] button', 'Back');

  // --- a new QOR ID is a new wallet ------------------------------------------------------------------------------
  await until(`Boolean(document.querySelector('[data-gate-new]'))`, 'New QOR ID');
  await click('[data-gate-new]');
  await until(`(document.body.textContent ?? '').includes('A new QOR ID')`, 'the new QOR ID step');
  check('a new QOR ID says it gets its own wallet and an empty balance', (await text()).includes('empty balance'));
  await clickText('button', 'Begin');
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell for the new QOR ID');
  check('a new QOR ID is made from a new phrase, as a wallet of its own', (await calls('vault_generate_phrase')).length === 1 && (await calls('vault_create')).length === 1);
  check("and shows its own empty balance, never another QOR ID's", await eventually(`(document.querySelector('nav[aria-label="Primary"]')?.textContent ?? '').includes('0.00 CGT')`) && !(await text()).includes('9,000') && !(await text()).includes('57,900'));

  // --- an account an older phrase used becomes its own QOR ID ------------------------------------------------------
  await settings();
  await click('[data-profile-look]');
  await until(`Boolean(document.querySelector('[data-profile-others]'))`, 'the other accounts');
  check("the older phrase's other accounts are offered with their balances", (await text('[data-profile-others]')).includes('57,900.00 CGT'));
  await click('[data-profile-split="1"]');
  await until(`Boolean(document.querySelector('[data-profile-splitting]'))`, 'the split step');
  check('a QOR ID with no PIN chooses one before it is left', await evaluate(`document.querySelectorAll('[data-profile-splitting] [data-pin]').length === 2`)
    && (await disabled('[data-profile-confirm-split]')) === true);
  await type('[data-profile-splitting] [data-pin="new"]', PIN);
  await type('[data-profile-splitting] [data-pin="again"]', '482914');
  check('two different PINs are refused', (await disabled('[data-profile-confirm-split]')) === true && (await text('[data-profile-splitting]')).includes('not the same'));
  await type('[data-profile-splitting] [data-pin="again"]', PIN);
  await click('[data-profile-confirm-split]');
  check('the account is made its own QOR ID by its index, with the PIN chosen', await eventually(`window.__CALLS__.some((c) => c.cmd === 'profile_split')`)
    && JSON.stringify(await calls('profile_split')) === JSON.stringify([{ index: 1, pin: PIN }]), JSON.stringify(await calls('profile_split')));
  check('and the launcher switches to it, with its own balance', await eventually(`(document.querySelector('nav[aria-label="Primary"]')?.textContent ?? '').includes('57,900.00 CGT')`));

  // --- a QOR ID with no PIN chooses one as it signs out --------------------------------------------------------------
  await settings();
  await click('[data-profile-sign-out]');
  await until(`Boolean(document.querySelector('[data-profile-leaving]'))`, 'the sign-out step');
  check('signing out a QOR ID with no PIN asks for one, twice', await evaluate(`document.querySelectorAll('[data-profile-leaving] [data-pin]').length === 2`)
    && (await disabled('[data-profile-confirm-sign-out]')) === true);
  await type('[data-profile-leaving] [data-pin="new"]', '771204');
  await type('[data-profile-leaving] [data-pin="again"]', '771204');
  await click('[data-profile-confirm-sign-out]');
  await until(`Boolean(document.querySelector('[data-gate-who]'))`, "Who's playing? again");
  check('and sends that PIN with the sign-out', (await calls('profile_sign_out')).at(-1)?.pin === '771204', JSON.stringify(await calls('profile_sign_out')));
  check('every QOR ID is listed again, the new ones among them', (await evaluate(`document.querySelectorAll('[data-gate-profile]').length`)) === 5);

  const words = await text();
  const banned = /\b(APY|yield|passive income|ROI|investment|returns?)\b/i.exec(words);
  check('no word of income, yield or returns appears (ADR-008)', !banned, banned?.[0] ?? '');
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
