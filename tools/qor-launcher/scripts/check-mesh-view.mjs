// Does the Mesh page ask the host exactly what the person chose, and say plainly what seeding is and is not? (ADR-095,
// ADR-097, L10.1 to L10.4)
//
// The host's tests prove the Mesh itself (src-tauri/src/mesh: two real nodes, verified transfer, the limits, refusing a
// seeder whose bytes changed). This check covers what reaches the screen: it serves the built frontend in a real
// rendering engine with a stand-in host that answers the `mesh_*` commands, then drives the page as a person would.
//
// What it holds the page to:
//   - seeding is off on arrival, and the page says so, says that seeding shows the person's network address to the peers
//     they serve, and says that to be paid the Mesh signs its own work, asked once, and can never send their CGT;
//   - on, the account the Mesh signs for is shown; off, the page says it signs for none;
//   - a file is fetched by its fingerprint only once one is typed, and the host's refusal is shown in its own words;
//   - an escrow's form suggests no price, sends every amount as exact Sparks, and refuses more than 18 decimals;
//   - seeding cannot be turned on until both limits are chosen, and is asked for with exactly the limits chosen;
//   - on, it shows what is held, whose it is, and how much of the disk given is used;
//   - a ticket is asked for by the file's root and put on the clipboard;
//   - fetching asks the host with the ticket typed, and the host's refusal is shown in its own words;
//   - new limits while seeding are asked for only when the person keeps them, and turning seeding off keeps the limits;
//   - no word of income, yield or returns appears on the page (ADR-008).
//
//   npm run build
//   node scripts/check-mesh-view.mjs [dist directory, default ./dist]
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
const GIB = 1024 ** 3;
const MB = 1000 ** 2;
const OWN = 'aa'.repeat(32);
const GOT = 'bb'.repeat(32);
const TICKET = 'blobfixtureticketnamingafileandthepcthathasit';
const FOUND = 'cc'.repeat(32);
const ACCOUNT = '5MeshFixtureAccountSignedForByTheMeshGrantXXXXXXXX';
const MIB = 1024 * 1024;

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

const userData = await mkdtemp(join(tmpdir(), 'qor-mesh-'));
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

  const ADDRESS = '5FixtureAddressForTheMeshCheck0000000000000000000';
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: null,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };

  const HOST_STUB = `
    window.__CALLS__ = [];
    window.__CLIP__ = [];
    window.__MESH__ = {
      settings: { seeding: false, diskBytes: 0, uploadBytesPerSecond: 0 },
      held: [],
      paid: { account: null, holdings: [], pendingReceipts: 0, paidSparks: '0', paidCgt: '0.00 CGT', payments: [] },
    };
    window.__CALLBACKS__ = {};
    window.__LISTENERS__ = {};
    let callbacks = 0;
    Object.defineProperty(navigator, 'clipboard', { value: { writeText: (t) => { window.__CLIP__.push(t); return Promise.resolve(); } } });
    const status = () => {
      const m = window.__MESH__;
      const used = m.held.reduce((n, h) => n + h.size, 0);
      return { settings: m.settings, node: m.settings.seeding ? 'fixturenode' : null, held: m.held.length, usedBytes: used, servedBytes: 3 * 1024 * 1024, paid: m.paid };
    };
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd.startsWith('mesh_')) window.__CALLS__.push({ cmd, args });
        const m = window.__MESH__;
        switch (cmd) {
          case 'plugin:event|listen':
            (window.__LISTENERS__[args.event] ??= []).push(args.handler);
            return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'mesh_status': return Promise.resolve(status());
          case 'mesh_held': return Promise.resolve(m.settings.seeding || m.held.some((h) => h.kind === 'fetched') ? m.held : []);
          case 'mesh_configure': {
            const s = args.settings;
            if (s.seeding && (!s.diskBytes || !s.uploadBytesPerSecond))
              return Promise.reject({ kind: 'mesh', message: 'Choose how much disk and upload to give the Mesh before seeding.' });
            m.settings = s;
            if (s.seeding && !m.held.some((h) => h.kind === 'own')) m.held = [{ root: '${OWN}', size: 300 * 1024, kind: 'own' }, ...m.held];
            // The host asks for the Mesh grant in its own dialog as seeding turns on; here the person approved it.
            m.paid = { ...m.paid, account: s.seeding ? '${ACCOUNT}' : null };
            return Promise.resolve(status());
          }
          case 'mesh_allow_paid':
            m.paid = { ...m.paid, account: '${ACCOUNT}' };
            return Promise.resolve(m.paid);
          case 'mesh_fetch_root':
            if (args.root !== '${FOUND}') return Promise.reject({ kind: 'mesh', message: 'Nobody on the chain is paid to hold that file. Ask for a Mesh ticket instead.' });
            m.held = [...m.held, { root: '${FOUND}', size: 1024 * 1024, kind: 'fetched' }];
            return Promise.resolve({ root: '${FOUND}', size: 1024 * 1024 });
          case 'mesh_open_escrow': return Promise.resolve(7);
          case 'mesh_ticket': return Promise.resolve('${TICKET}');
          case 'mesh_fetch':
            if (args.ticket !== '${TICKET}') return Promise.reject({ kind: 'mesh', message: 'That is not a Mesh ticket.' });
            m.held = [...m.held, { root: '${GOT}', size: 2 * 1024 * 1024, kind: 'fetched' }];
            return Promise.resolve({ root: '${GOT}', size: 2 * 1024 * 1024 });
          case 'launcher_state': return Promise.resolve(${JSON.stringify(STATE)});
          case 'vault_status': return Promise.resolve({ state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} });
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

  const page = () => evaluate(`document.querySelector('[data-mesh]')?.textContent ?? ''`);
  const choose = (name, label) => clickText(`[data-mesh-choice="${name}"] button`, label);
  const configured = () => calls('mesh_configure');

  // --- arriving: off, and plain about it -----------------------------------------------------
  check('the rail offers the Mesh', (await clickText('nav[aria-label="Primary"] button', 'Mesh')) === true);
  await until(`Boolean(document.querySelector('[data-mesh-state]'))`, 'the Mesh page');
  check('seeding is off on arrival, and the page says so', (await text('[data-mesh] h2')).includes('Seeding is off'));
  const terms = await text('[data-mesh-terms]');
  check('it says seeding shows your network address to the peers you serve', terms.includes('network address') && terms.includes('peers you serve'), terms);
  check('it says to be paid the Mesh signs its own work, asked once', terms.includes('signs its own work') && terms.includes('asked once'), terms);
  check('and that it can never send their CGT', terms.includes('never send your CGT'), terms);
  check('off, the page says the Mesh signs for no account', (await text('[data-mesh-paid]')).includes('not signing for any account'));
  check('nothing was asked of the host but to read', (await configured()).length === 0);

  // --- turning it on -----------------------------------------------------------------------------
  check('seeding cannot be turned on before both limits are chosen', await evaluate(`document.querySelector('[data-mesh-on]').disabled`));
  await choose('disk', '20');
  check('nor with only the disk chosen', await evaluate(`document.querySelector('[data-mesh-on]').disabled`));
  await choose('upload', '5 MB/s');
  await until(`!document.querySelector('[data-mesh-on]').disabled`, 'the button to turn seeding on');
  check('a chosen limit shows as chosen', await evaluate(`document.querySelector('[data-mesh-choice="disk"] [aria-checked="true"]')?.textContent.trim() === '20.0 GiB'`));
  await click('[data-mesh-on]');
  check('seeding is asked for with exactly the limits chosen', await eventually(`window.__CALLS__.some((c) => c.cmd === 'mesh_configure')`)
    && JSON.stringify((await configured())[0]?.settings) === JSON.stringify({ seeding: true, diskBytes: 20 * GIB, uploadBytesPerSecond: 5 * MB }), JSON.stringify((await configured())[0]));
  check('on, the page says it is seeding', await eventually(`(document.querySelector('[data-mesh] h2')?.textContent ?? '') === 'Seeding'`));
  check('what is held is listed, as your work', await eventually(`(document.querySelector('[data-mesh-item="${OWN}"]')?.textContent ?? '').includes('your work')`));
  check('and how much of the disk given is used', (await text('[data-mesh-stats]')).includes('of 20.0 GiB'), await text('[data-mesh-stats]'));
  check('on, the account the Mesh signs for is shown', await eventually(`Boolean(document.querySelector('[data-mesh-account]'))`)
    && (await text('[data-mesh-account]')).startsWith(ACCOUNT.slice(0, 8)));

  // --- a ticket ----------------------------------------------------------------------------------
  await click(`[data-mesh-item="${OWN}"] [data-mesh-ticket]`);
  check('a ticket is asked for by the file\'s root', await eventually(`window.__CALLS__.some((c) => c.cmd === 'mesh_ticket')`)
    && (await calls('mesh_ticket'))[0].root === OWN);
  check('and put on the clipboard', await eventually(`window.__CLIP__[0] === '${TICKET}'`));
  await click(`[data-mesh-item="${OWN}"] [data-mesh-copy-root]`);
  check("a file's whole fingerprint can be copied, to fetch it by", await eventually(`window.__CLIP__[1] === '${OWN}'`));

  // --- fetching ----------------------------------------------------------------------------------
  check('fetching waits for a ticket', await evaluate(`document.querySelector('[data-mesh-fetch]').disabled`));
  await type('[data-mesh-fetch-input]', 'not a ticket');
  await click('[data-mesh-fetch]');
  check('a refused ticket is shown in the host\'s own words', await eventually(`document.body.textContent.includes('That is not a Mesh ticket.')`));
  await type('[data-mesh-fetch-input]', `  ${TICKET}  `.trim());
  await click('[data-mesh-fetch]');
  check('fetching asks the host with the ticket typed', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'mesh_fetch').length === 2`)
    && (await calls('mesh_fetch'))[1].ticket === TICKET);
  check('what was fetched joins the list, as fetched', await eventually(`(document.querySelector('[data-mesh-item="${GOT}"]')?.textContent ?? '').includes('fetched')`));

  // --- by fingerprint ------------------------------------------------------------------------------
  await type('[data-mesh-root-input]', 'abc');
  check('fetching by fingerprint waits for a whole one', await evaluate(`document.querySelector('[data-mesh-fetch-root]').disabled`));
  await type('[data-mesh-root-input]', 'dd'.repeat(32));
  await click('[data-mesh-fetch-root]');
  check('a fingerprint nobody holds is refused in the host\'s own words', await eventually(`document.body.textContent.includes('Nobody on the chain is paid to hold that file.')`));
  await type('[data-mesh-root-input]', FOUND);
  await click('[data-mesh-fetch-root]');
  check('fetching asks the host with the fingerprint typed', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'mesh_fetch_root').length === 2`)
    && (await calls('mesh_fetch_root'))[1].root === FOUND);
  check('what was found joins the list', await eventually(`Boolean(document.querySelector('[data-mesh-item="${FOUND}"]'))`));

  // --- an escrow ------------------------------------------------------------------------------------
  check('only one\'s own work is offered an escrow', await evaluate(`Boolean(document.querySelector('[data-mesh-item="${OWN}"] [data-mesh-escrow-open]')) && !document.querySelector('[data-mesh-item="${GOT}"] [data-mesh-escrow-open]')`));
  await click(`[data-mesh-item="${OWN}"] [data-mesh-escrow-open]`);
  await until(`Boolean(document.querySelector('[data-mesh-escrow]'))`, 'the escrow form');
  check('the form suggests no price', await evaluate(`[...document.querySelectorAll('[data-mesh-escrow-field]')].every((f) => f.value === '')`));
  check('and says what collusion can draw', (await text('[data-mesh-escrow-limit]')).includes('downloading from each other'));
  check('it cannot be sent empty', await evaluate(`document.querySelector('[data-mesh-escrow-submit]').disabled`));
  const fill = async (name, value) => type(`[data-mesh-escrow-field="${name}"]`, value);
  await fill('copies', '2');
  await fill('storage', '0.5');
  await fill('serving', '1.25');
  await fill('cap', '10');
  await fill('deposit', '0.0000000000000000001');
  check('more than 18 decimals is refused, not cut', await evaluate(`document.querySelector('[data-mesh-escrow-submit]').disabled`));
  await fill('deposit', '100');
  await until(`!document.querySelector('[data-mesh-escrow-submit]').disabled`, 'the escrow button');
  await click('[data-mesh-escrow-submit]');
  check('the escrow is asked for in exact Sparks, for the file chosen', await eventually(`window.__CALLS__.some((c) => c.cmd === 'mesh_open_escrow')`)
    && JSON.stringify((await calls('mesh_open_escrow'))[0].terms) === JSON.stringify({
      root: OWN, size: 300 * 1024, copies: 2,
      storagePriceSparks: '500000000000000000', servingPricePerMibSparks: '1250000000000000000',
      servingCapPerPeriod: 10 * MIB, depositSparks: '100000000000000000000',
    }), JSON.stringify((await calls('mesh_open_escrow'))[0]));
  check('and the form closes once it is open', await eventually(`!document.querySelector('[data-mesh-escrow]')`));

  // --- new limits, and off ---------------------------------------------------------------------------
  const before = (await configured()).length;
  await choose('disk', '50');
  check('new limits are not asked for until kept', (await configured()).length === before && await eventually(`Boolean(document.querySelector('[data-mesh-apply]'))`));
  await click('[data-mesh-apply]');
  check('kept, they are asked for with seeding still on', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'mesh_configure').length === ${before + 1}`)
    && JSON.stringify((await configured())[before].settings) === JSON.stringify({ seeding: true, diskBytes: 50 * GIB, uploadBytesPerSecond: 5 * MB }));
  await click('[data-mesh-off]');
  check('turning seeding off keeps the limits', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'mesh_configure').length === ${before + 2}`)
    && JSON.stringify((await configured())[before + 1].settings) === JSON.stringify({ seeding: false, diskBytes: 50 * GIB, uploadBytesPerSecond: 5 * MB }));
  check('and the page says it is off', await eventually(`(document.querySelector('[data-mesh] h2')?.textContent ?? '') === 'Seeding is off'`));
  check('and that the Mesh signs for no account', await eventually(`(document.querySelector('[data-mesh-paid]')?.textContent ?? '').includes('not signing for any account')`));

  // --- the words -------------------------------------------------------------------------------------
  const words = await page();
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
