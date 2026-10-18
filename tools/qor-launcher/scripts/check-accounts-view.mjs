// Does the launcher say whose account is on screen, and keep each QOR ID on its own account? (launcher 0.1.12)
//
// The money belongs to an account's key, and a QOR ID is a name bound to one key. On a PC with two QOR IDs the
// launcher showed the first account's balance under either name (the owner, 18 October 2026). This check serves the
// built frontend in a real rendering engine with a stand-in host holding two accounts, one the signed-in QOR ID's, and
// holds the screens to:
//   - the Vault labels each account with the QOR ID it belongs to, and marks the signed-in one;
//   - on another account, it says plainly that it is not the signed-in QOR ID's, and switches to its own in one click;
//   - the rail's account card says so too;
//   - signed out, Settings lists each account under its QOR ID and signs in as the one chosen, by its address;
//   - an account with no QOR ID yet claims a name for itself, never for another account;
//   - a new QOR ID is asked for on a new account, by name.
//
//   npm run build
//   node scripts/check-accounts-view.mjs [dist directory, default ./dist]
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
const FREE = '5UnclaimedFixtureAccount00000000000000000000000000';

const results = [];
const check = (name, pass, detail = '') => {
  results.push({ name, pass });
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}${detail ? `  (${detail})` : ''}`);
};

// --- serve the build -------------------------------------------------------
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.woff2': 'font/woff2', '.ico': 'image/x-icon' };
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

const userData = await mkdtemp(join(tmpdir(), 'qor-accounts-'));
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

  const ACCOUNTS = [
    { address: GOD, account_id: '0x' + '01'.repeat(32), path: '', index: 0, label: 'Main' },
    { address: YAL, account_id: '0x' + '02'.repeat(32), path: '//0', index: 1, label: 'Second' },
    { address: FREE, account_id: '0x' + '03'.repeat(32), path: '//1', index: 2, label: 'Third' },
  ];
  const SESSION = { qor_id: 'Yaldobaoth', username: 'Yaldobaoth', discriminator: 1, role: 'user', address: YAL, avatar_url: null, username_chosen: true };
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: SESSION,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };

  const HOST_STUB = `
    window.__CALLS__ = [];
    window.__CALLBACKS__ = {};
    let callbacks = 0;
    const balance = (address) => ({ address, sparks: '0', cgt: address === '${GOD}' ? '9000' : address === '${YAL}' ? '25' : '0', display: (address === '${GOD}' ? '9,000.00' : address === '${YAL}' ? '25.00' : '0.00') + ' CGT' });
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        window.__CALLS__.push({ cmd, args });
        switch (cmd) {
          case 'plugin:event|listen': return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'launcher_state': return Promise.resolve(${JSON.stringify(STATE)});
          case 'vault_status': return Promise.resolve({ state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} });
          // The host signed in as Yaldobaoth with its own account, the one last in use.
          case 'vault_owners': return Promise.resolve({ owners: { '${GOD}': 'godmode', '${YAL}': 'Yaldobaoth' }, last: '${YAL}' });
          case 'cgt_balance': return Promise.resolve(balance(args.address));
          case 'cgt_nonce': return Promise.resolve(0);
          case 'qor_logout': return Promise.resolve(null);
          case 'qor_sign_in':
            if (args.address === '${FREE}') return Promise.resolve({ accounts: ${JSON.stringify(ACCOUNTS)}, address: '${FREE}', session: null, needs_name: true, sign_in_problem: null });
            return Promise.resolve({ accounts: ${JSON.stringify(ACCOUNTS)}, address: args.address, session: { ...${JSON.stringify(SESSION)}, qor_id: 'godmode', username: 'godmode', address: args.address }, needs_name: false, sign_in_problem: null });
          case 'qor_register_with_key':
            return Promise.resolve({ ...${JSON.stringify(SESSION)}, qor_id: args.username, username: args.username, address: args.address });
          case 'qor_new_identity':
            return Promise.resolve({ accounts: ${JSON.stringify(ACCOUNTS)}, address: '${FREE}', session: { ...${JSON.stringify(SESSION)}, qor_id: args.username, username: args.username, address: '${FREE}' }, needs_name: false, sign_in_problem: null });
          case 'touch_vault': case 'vault_lock': case 'vault_set_active': case 'social_disconnect': case 'social_connect': return Promise.resolve(null);
          case 'social_status': return Promise.resolve({ status: { state: 'off' }, welcome: null, endpoint: '' });
          default: return Promise.reject({ kind: 'internal', message: 'no host in this check' });
        }
      },
      transformCallback: (cb) => { const id = ++callbacks; window.__CALLBACKS__[id] = cb; return id; },
      unregisterCallback: (id) => { delete window.__CALLBACKS__[id]; },
      convertFileSrc: (s) => s,
      metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
      plugins: {},
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  `;

  await send('Page.enable');
  await send('Runtime.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await send('Page.bringToFront');
  await send('Emulation.setFocusEmulationEnabled', { enabled: true });
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell and its rail');

  const calls = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)}).map((c) => c.args)`);
  const text = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)})?.textContent ?? ''`);
  const click = (selector) => evaluate(`(() => { const n = document.querySelector(${JSON.stringify(selector)}); if (!n) return false; n.click(); return true; })()`);
  const clickText = (scope, label) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => b.textContent.trim().startsWith(${JSON.stringify(label)})); if (!n) return false; n.click(); return true; })()`);
  const type = (selector, value) => evaluate(`(() => {
    const el = document.querySelector(${JSON.stringify(selector)});
    Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(el, ${JSON.stringify(value)});
    el.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);

  const activeSet = () => calls('vault_set_active');

  // --- arriving: the signed-in QOR ID's own account, never simply the first ---------------------------------------
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell and its rail');
  check("arriving, the account in use is the signed-in QOR ID's own, not the first", await eventually(`window.__CALLS__.some((c) => c.cmd === 'vault_set_active')`)
    && (await activeSet()).every((a) => a.address === YAL), JSON.stringify(await activeSet()));
  check('and the rail shows no warning', !(await evaluate(`Boolean(document.querySelector('[data-rail-mismatch]'))`)));

  // --- the Vault: whose each account is ----------------------------------------------------------------------------
  check('the rail offers the Vault', (await clickText('nav[aria-label="Primary"] button', 'Vault')) === true);
  await until(`document.querySelectorAll('[data-vault-owner]').length >= 2`, 'the account labels');
  const labels = await evaluate(`[...document.querySelectorAll('[data-vault-owner]')].map((n) => n.getAttribute('data-vault-owner'))`);
  check('each account is labelled with the QOR ID it belongs to', labels.includes('godmode') && labels.includes('Yaldobaoth'), JSON.stringify(labels));
  check('an account no QOR ID has signed in as carries no label', labels.length === 2);
  check('on its own account, no warning', !(await evaluate(`Boolean(document.querySelector('[data-vault-mismatch]'))`)));
  await clickText('aside li button', 'Main');
  check("on godmode's account, it says this is not Yaldobaoth's", await eventually(`(document.querySelector('[data-vault-mismatch]')?.textContent ?? '').includes('Yaldobaoth')`)
    && (await text('[data-vault-mismatch]')).includes("godmode's") && (await text('[data-vault-mismatch]')).includes('spends this account'), await text('[data-vault-mismatch]'));
  check('and the rail warns too', await eventually(`(document.querySelector('[data-rail-mismatch]')?.textContent ?? '').includes("Not Yaldobaoth's wallet")`));
  await click('[data-vault-switch]');
  check("one click shows Yaldobaoth's own account", await eventually(`!document.querySelector('[data-vault-mismatch]')`)
    && (await activeSet()).at(-1)?.address === YAL);

  // --- signed out: choose which QOR ID to be -------------------------------------------------------------------------
  check('the rail offers Settings', (await clickText('nav[aria-label="Primary"] button', 'Settings')) === true);
  await until(`[...document.querySelectorAll('button')].some((b) => b.textContent.includes('Sign out'))`, 'the sign-out button');
  await clickText('button', 'Sign out');
  await sleep(300);
  await clickText('nav[aria-label="Primary"] button', 'Settings');
  await until(`Boolean(document.querySelector('[data-signin]'))`, 'the sign-in panel');
  const rows = await evaluate(`[...document.querySelectorAll('[data-signin-as]')].map((b) => b.textContent.trim())`);
  check('signed out, each account offers to sign in as its own QOR ID', rows.includes('Sign in as Yaldobaoth') && rows.includes('Sign in as godmode') && rows.includes('Sign in with this account'), JSON.stringify(rows));
  check('the account last in use is first', rows[0] === 'Sign in as Yaldobaoth', JSON.stringify(rows));
  await click(`[data-signin-as="${GOD}"]`);
  check("signing in as godmode asks the host with godmode's account", await eventually(`window.__CALLS__.some((c) => c.cmd === 'qor_sign_in')`)
    && (await calls('qor_sign_in'))[0].address === GOD, JSON.stringify(await calls('qor_sign_in')));
  check("and godmode's account is then the one in use", await eventually(`window.__CALLS__.filter((c) => c.cmd === 'vault_set_active').at(-1)?.args.address === ${JSON.stringify(GOD)}`));

  await until(`[...document.querySelectorAll('button')].some((b) => b.textContent.includes('Sign out'))`, 'signed in again');
  await clickText('button', 'Sign out');
  await sleep(300);
  await clickText('nav[aria-label="Primary"] button', 'Settings');
  await until(`Boolean(document.querySelector('[data-signin]'))`, 'the sign-in panel again');
  await click(`[data-signin-as="${FREE}"]`);
  check('an account with no QOR ID yet offers to claim a name for itself', await eventually(`Boolean(document.querySelector('[data-signin-account="${FREE}"] [data-signin-claim]'))`));
  await type(`[data-signin-account="${FREE}"] [data-signin-claim] input`, 'Sophia');
  await clickText(`[data-signin-account="${FREE}"] [data-signin-claim] button`, 'Claim');
  check('the name is claimed for that account, not the first', await eventually(`window.__CALLS__.some((c) => c.cmd === 'qor_register_with_key')`)
    && (await calls('qor_register_with_key'))[0].address === FREE && (await calls('qor_register_with_key'))[0].username === 'Sophia', JSON.stringify(await calls('qor_register_with_key')));

  await until(`[...document.querySelectorAll('button')].some((b) => b.textContent.includes('Sign out'))`, 'signed in as Sophia');
  await clickText('button', 'Sign out');
  await sleep(300);
  await clickText('nav[aria-label="Primary"] button', 'Settings');
  await until(`Boolean(document.querySelector('[data-signin-new]'))`, 'the new QOR ID form');
  check('a new QOR ID says it gets its own account and balance', (await text('[data-signin-new]')).includes('its own empty balance'));
  check('it waits for a name', await evaluate(`document.querySelector('[data-signin-new-create]').disabled`));
  await type('[data-signin-new-name]', 'Barbelo');
  await click('[data-signin-new-create]');
  check('a new QOR ID is asked for on a new account, by its name', await eventually(`window.__CALLS__.some((c) => c.cmd === 'qor_new_identity')`)
    && (await calls('qor_new_identity'))[0].username === 'Barbelo', JSON.stringify(await calls('qor_new_identity')));
  check('and signed in as it', await eventually(`[...document.querySelectorAll('dd')].some((d) => d.textContent.trim() === 'Barbelo')`));

  const words = await evaluate(`document.querySelector('main')?.textContent ?? ''`);
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
