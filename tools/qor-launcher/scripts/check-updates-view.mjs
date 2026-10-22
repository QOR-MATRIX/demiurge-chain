// Does the launcher's update channel ask the host only what the person chose, and say plainly what it found? (ADR-111,
// L6.2)
//
// The host's tests prove the channel itself (src-tauri/src/update.rs: a release accepted only with the launcher's key
// and its own installer's name; a changed byte, another version's installer and another key refused; the configuration
// naming the channel and nothing dangerous). This check covers what reaches the screen: it serves the built frontend in
// a real rendering engine with a stand-in host that answers the `update_*` commands and announces the channel's status,
// then drives Settings and the title bar as a person would.
//
// What it holds the page to:
//   - nothing is said, checked or installed on arrival but reading what the host knows;
//   - Check now asks the host once; what it finds is announced with the version in use, its notes as written, and what
//     happens next: the signature checked before anything installs, Windows asking once, the launcher reopening;
//   - the title bar says an update is ready, and takes the person to it;
//   - Update and restart asks the host to install, once, naming nothing: no address, key or file comes from the page;
//   - fetching shows how far, verified it says the launcher closes and opens again, refused it shows the host's words;
//   - up to date, it says so with when it checked;
//   - checking by itself is on by default, and turning it off asks the host exactly that;
//   - the page never calls the updater plugin itself, and no word of income, yield or returns appears (ADR-008).
//
//   npm run build
//   node scripts/check-updates-view.mjs [dist directory, default ./dist]
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

const userData = await mkdtemp(join(tmpdir(), 'qor-updates-'));
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

  const ACCOUNTS = [{ address: '5FixtureAddressForTheUpdatesCheck000000000000000', account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: null,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };
  const READY = { state: 'available', version: '0.2.0', notes: 'Fixes the Library.\nOpens faster.', date: '2026-10-23 00:00:00 +00:00:00' };

  const HOST_STUB = `
    window.__CALLS__ = [];
    window.__CALLBACKS__ = {};
    window.__LISTENERS__ = {};
    window.__NEXT__ = 'ready';
    let callbacks = 0;
    window.__UPDATE__ = { current: '0.0.0-fixture', status: { state: 'unchecked' }, automatic: true, checked_at: null };
    window.__EMIT__ = (event, payload) => {
      for (const id of window.__LISTENERS__[event] ?? []) window.__CALLBACKS__[id]?.({ event, id, payload });
    };
    const view = () => JSON.parse(JSON.stringify(window.__UPDATE__));
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd.startsWith('update_') || cmd.startsWith('plugin:updater')) window.__CALLS__.push({ cmd, args });
        const u = window.__UPDATE__;
        switch (cmd) {
          case 'plugin:event|listen':
            (window.__LISTENERS__[args.event] ??= []).push(args.handler);
            return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'update_status': return Promise.resolve(view());
          case 'update_check':
            if (window.__NEXT__ === 'offline')
              return Promise.reject({ kind: 'update', message: 'The launcher could not check for updates: the address did not answer' });
            u.status = window.__NEXT__ === 'ready' ? ${JSON.stringify(READY)} : { state: 'up_to_date' };
            u.checked_at = 1792700000;
            window.__EMIT__('update-status', u.status);
            return Promise.resolve(view());
          // On Windows a verified install closes the launcher, so the call does not return; a refused one returns the
          // host's error after announcing it, as the check settles below.
          case 'update_install': return new Promise((ok, bad) => { window.__INSTALL__ = { ok, bad }; });
          case 'update_set_automatic':
            u.automatic = args.on;
            return Promise.resolve(view());
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

  const calls = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)}).map((c) => c.args ?? null)`);
  const text = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)})?.textContent ?? ''`);
  const click = (selector) => evaluate(`(() => { const n = document.querySelector(${JSON.stringify(selector)}); if (!n) return false; n.click(); return true; })()`);
  const clickText = (scope, label) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => b.textContent.trim().startsWith(${JSON.stringify(label)})); if (!n) return false; n.click(); return true; })()`);
  const emit = (status) => evaluate(`(window.__EMIT__('update-status', ${JSON.stringify(status)}), true)`);
  const line = () => text('[data-update-line]');
  const pill = () => evaluate(`Boolean(document.querySelector('[data-update-ready]'))`);

  // --- arriving: nothing found yet, and nothing in the title bar ------------------------------------------------------
  check('nothing is said in the title bar before an update is found', !(await pill()));
  check('the rail offers Settings', (await clickText('nav[aria-label="Primary"] button', 'Settings')) === true);
  await until(`Boolean(document.querySelector('[data-updates]'))`, 'the Updates panel');
  check('Settings says which launcher this is and that it has not checked yet', (await line()) === 'QOR Launcher 0.0.0-fixture. Not checked for updates yet.', await line());
  check('it offers no field to type an address, a key or a file', await evaluate(`document.querySelectorAll('[data-updates] input, [data-updates] textarea').length === 0`));
  check('nothing was asked of the host but to read', (await calls('update_check')).length === 0 && (await calls('update_install')).length === 0);

  // --- checking, and finding one --------------------------------------------------------------------------------------
  await click('[data-update-check]');
  check('Check now asks the host to check, once', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'update_check').length === 1`));
  check('a newer version is announced with the one in use', await eventually(`(document.querySelector('[data-update-line]')?.textContent ?? '') === 'QOR Launcher 0.2.0 is ready. You have 0.0.0-fixture.'`), await line());
  check('its notes are shown as written', (await text('[data-update-notes]')) === 'Fixes the Library.\nOpens faster.', await text('[data-update-notes]'));
  const caution = await text('[data-updates]');
  check('it says the update is checked against the launcher\'s signature before anything installs', caution.includes("checked against the launcher's own signature before anything installs"));
  check('and that Windows asks once, then the launcher reopens on the new version', caution.includes('Windows asks once for permission') && caution.includes('opens again on the new version'));
  check('nothing was installed by finding it', (await calls('update_install')).length === 0);

  // --- the title bar says so, and takes the person to it ----------------------------------------------------------------
  check('the title bar says an update is ready', await eventually(`Boolean(document.querySelector('[data-update-ready]'))`));
  await clickText('nav[aria-label="Primary"] button', 'Nexus');
  await until(`!document.querySelector('[data-updates]')`, 'leaving Settings');
  await click('[data-update-ready]');
  check('which opens Settings at the update', await eventually(`Boolean(document.querySelector('[data-update-install]'))`));

  // --- installing only when asked -------------------------------------------------------------------------------------
  await click('[data-update-install]');
  check('Update and restart asks the host to install, once', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'update_install').length === 1`));
  const asked = await calls('update_install');
  check('naming nothing: the host installs only what its own check found', asked.length === 1 && (asked[0] === null || Object.keys(asked[0]).length === 0), JSON.stringify(asked));
  await emit({ state: 'downloading', version: '0.2.0', received: 512, total: 1024 });
  check('while it is fetched, the page says how far, and a bar shows it', await eventually(`(document.querySelector('[data-update-line]')?.textContent ?? '') === 'Fetching QOR Launcher 0.2.0: 50%' && document.querySelector('[data-update-progress]')?.getAttribute('aria-valuenow') === '50'`), await line());
  check('and Check now waits', await evaluate(`document.querySelector('[data-update-check]').disabled`));
  await emit({ state: 'installing', version: '0.2.0' });
  check('verified, it says the launcher closes and opens again', await eventually(`(document.querySelector('[data-update-line]')?.textContent ?? '') === 'Installing QOR Launcher 0.2.0. The launcher closes and opens again.'`), await line());
  check('and the title bar no longer offers it', !(await pill()));

  // --- a refusal, in the host's own words -----------------------------------------------------------------------------
  const refusal = "The update was refused: its signature does not match the launcher's key. Nothing was installed.";
  await emit({ state: 'failed', message: refusal });
  await evaluate(`(window.__INSTALL__.bad({ kind: 'update', message: ${JSON.stringify(refusal)} }), true)`);
  check('a refused update is shown in the host\'s own words', await eventually(`(document.querySelector('[data-update-line]')?.textContent ?? '') === ${JSON.stringify(refusal)}`), await line());
  check('and Check now can be pressed again', !(await evaluate(`document.querySelector('[data-update-check]').disabled`)));

  // --- up to date ------------------------------------------------------------------------------------------------------
  await evaluate(`(window.__NEXT__ = 'none', true)`);
  await click('[data-update-check]');
  check('up to date, it says so, with when it checked', await eventually(`/^QOR Launcher 0\\.0\\.0-fixture is the newest\\. Checked at .+\\.$/.test(document.querySelector('[data-update-line]')?.textContent ?? '')`), await line());

  // --- the switch -------------------------------------------------------------------------------------------------------
  check('checking by itself is on by default, and says so', await evaluate(`document.querySelector('[data-update-automatic] [role="switch"]')?.getAttribute('aria-checked') === 'true'`));
  await click('[data-update-automatic] [role="switch"]');
  check('turning it off asks the host exactly that', await eventually(`JSON.stringify(window.__CALLS__.filter((c) => c.cmd === 'update_set_automatic').map((c) => c.args)) === JSON.stringify([{ on: false }])`));
  check('and it shows as off', await eventually(`document.querySelector('[data-update-automatic] [role="switch"]')?.getAttribute('aria-checked') === 'false'`));

  // --- the boundary, and the words --------------------------------------------------------------------------------------
  check('the interface never called the updater plugin itself', (await evaluate(`window.__CALLS__.filter((c) => c.cmd.startsWith('plugin:updater')).length`)) === 0);
  const words = await text('[data-updates]');
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
