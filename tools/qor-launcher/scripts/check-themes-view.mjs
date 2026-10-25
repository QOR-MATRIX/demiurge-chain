// Do the themes a level unlocks open only at that level? (ADR-078 decision 4, DIRECTION P7.16)
//
// QOR ID decides what a level unlocks (services/qor-auth/src/handlers/progress.rs, UNLOCKS) and lists it in the
// profile's progress; the launcher only reads it. This check serves the built frontend in a real rendering engine with a
// stand-in host whose QOR ID answers progress as the test says, then visits Settings as a person would.
//
// What it holds the picker to:
//   - Dusk and Nocturne are offered to everyone, each named with the level that unlocks it, and cannot be chosen
//     before it: pressing one changes nothing;
//   - a theme everyone has is chosen as before;
//   - once QOR ID lists "Theme: Dusk" (read again when the window regains focus), Dusk is chosen and the whole interface
//     wears it; Nocturne, not yet listed, stays locked;
//   - no word of income, yield or returns appears (ADR-008).
//
//   npm run build
//   node scripts/check-themes-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { devtoolsPort } from './chrome.mjs';

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

const userData = await mkdtemp(join(tmpdir(), 'qor-themes-'));
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--force-device-scale-factor=1.5',
    '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling',
    '--remote-debugging-port=0', `--user-data-dir=${userData}`, 'about:blank',
  ],
  { stdio: ['ignore', 'ignore', 'pipe'] },
);
const finish = async () => {
  try { browser.kill(); } catch {}
  try { server.close(); } catch {}
  await rm(userData, { recursive: true, force: true }).catch(() => {});
};

try {
  let target;
  const port = await devtoolsPort(browser);
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

  const ADDRESS = '5FixtureAddressForTheThemesCheck0000000000000000';
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const SESSION = { qor_id: 'fixture', username: 'fixture', discriminator: 1, role: 'user', address: ADDRESS, avatar_url: null, username_chosen: true };
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: SESSION,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark', sub_units_per_cgt: 1000, atomic_unit: 'ember' },
  };
  // Level 1: a ring style, and no theme yet.
  const LEVEL_1 = { level: 1, xp: 120, level_xp: 100, next_level_xp: 250, next_unlock: 'Theme: Dusk', unlocked: ['Ring style: Ember'], tasks: [] };
  const LEVEL_2 = { level: 2, xp: 260, level_xp: 250, next_level_xp: 400, next_unlock: 'Ring style: Lattice', unlocked: ['Ring style: Ember', 'Theme: Dusk'], tasks: [] };

  const HOST_STUB = `
    window.__PROGRESS__ = ${JSON.stringify(LEVEL_1)};
    window.__CALLBACKS__ = {};
    let callbacks = 0;
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        switch (cmd) {
          case 'plugin:event|listen': return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'launcher_state': return Promise.resolve(${JSON.stringify(STATE)});
          case 'vault_status': return Promise.resolve({ state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} });
          case 'qor_progress': return Promise.resolve(JSON.parse(JSON.stringify(window.__PROGRESS__)));
          case 'update_status': return Promise.resolve({ current: '0.0.0-fixture', status: { state: 'unchecked' }, automatic: true, checked_at: null });
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

  const clickText = (scope, label) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => b.textContent.trim().startsWith(${JSON.stringify(label)})); if (!n) return false; n.click(); return true; })()`);
  const button = (label) => `[...document.querySelectorAll('button')].find((b) => b.getAttribute('aria-label') === ${JSON.stringify(label)})`;
  const press = (label) => evaluate(`(() => { const b = ${button(label)}; if (!b) return false; b.click(); return true; })()`);
  const wearing = () => evaluate(`document.documentElement.getAttribute('data-theme')`);
  const accent = () => evaluate(`getComputedStyle(document.documentElement).getPropertyValue('--accent').trim().toUpperCase()`);

  check('the rail offers Settings', (await clickText('nav[aria-label="Primary"] button', 'Settings')) === true);
  await until(`Boolean(${button('Obsidian theme')})`, 'the theme picker');
  check('it opens in Obsidian, the one look', (await wearing()) === 'obsidian', await wearing());

  // --- level 1: both locked, each with its level ------------------------------------------------------------------------
  check('Dusk is offered, named with the level that unlocks it, and cannot be chosen',
    await eventually(`(() => { const b = ${button('Dusk theme, unlocks at level 2')}; return Boolean(b && b.disabled && b.textContent.includes('Lv 2')); })()`));
  check('Nocturne likewise, at level 5',
    await evaluate(`(() => { const b = ${button('Nocturne theme, unlocks at level 5')}; return Boolean(b && b.disabled && b.textContent.includes('Lv 5')); })()`));
  await press('Dusk theme, unlocks at level 2');
  await sleep(200);
  check('pressing a locked theme changes nothing', (await wearing()) === 'obsidian', await wearing());

  // --- a theme everyone has -----------------------------------------------------------------------------------------------
  check('a theme everyone has is chosen as before', (await press('Abyss theme')) && (await eventually(`document.documentElement.getAttribute('data-theme') === 'abyss'`)), await wearing());

  // --- level 2: QOR ID lists Dusk -----------------------------------------------------------------------------------------
  await evaluate(`(window.__PROGRESS__ = ${JSON.stringify(LEVEL_2)}, window.dispatchEvent(new Event('focus')), true)`);
  check('once QOR ID lists it, Dusk can be chosen', await eventually(`(() => { const b = ${button('Dusk theme')}; return Boolean(b && !b.disabled && !b.textContent.includes('Lv')); })()`));
  await press('Dusk theme');
  check('and the interface wears it', await eventually(`document.documentElement.getAttribute('data-theme') === 'dusk'`), await wearing());
  check('its accent is Dusk\'s apricot', (await accent()) === '#FF9E64', await accent());
  check('Nocturne, not yet listed, stays locked', await evaluate(`(() => { const b = ${button('Nocturne theme, unlocks at level 5')}; return Boolean(b && b.disabled); })()`));

  // --- the words ----------------------------------------------------------------------------------------------------------
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
