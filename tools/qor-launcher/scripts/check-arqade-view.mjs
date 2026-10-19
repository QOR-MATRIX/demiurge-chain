// Does the ARQADE screen dock ARQADE's app and tell it what it needs, and fall back to the website's window? (ADR-102,
// DIRECTION P7.19)
//
// The host's tests prove the app's side (src-tauri/src/arqade/app.rs: its queue, the sign-in it may ask for and every
// check on it, and nothing of a sign-in in the log), and ARQADE's own Qt tests prove the app. This check covers what
// reaches the screen: it serves the built frontend in a real rendering engine with a stand-in host that answers the
// `arqade_*` commands, then visits the screen as a person would.
//
// What it holds the view to:
//   - with the app here, the screen starts it and leaves it the whole area, told in physical pixels;
//   - the app is told the launcher's colours, as `#rrggbb` tokens;
//   - it is told who is playing: signed in, their QOR ID, their level, and their balance in Sparks and in CGT, exactly
//     as the host wrote them out; and whether the person wants less motion, from the system's own preference;
//   - leaving the screen hides the app, and coming back shows it again;
//   - when the app ends while the screen shows, the screen says so and offers to open it again, and the website;
//   - with the app not installed, the screen offers the website's window, which opens through `arqade_open`;
//   - when the app cannot start, the screen says why and still offers the website.
//
//   npm run build
//   node scripts/check-arqade-view.mjs [dist directory, default ./dist]
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

const userData = await mkdtemp(join(tmpdir(), 'qor-arqade-'));
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

  const ADDRESS = '5FixtureAddressForTheArqadeCheck00000000000000';
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const SESSION = { qor_id: 'fixture', username: 'fixture', discriminator: 1, role: 'user', address: ADDRESS, avatar_url: null, username_chosen: true };
  const PROGRESS = { level: 4, xp: 450, level_xp: 400, next_level_xp: 600, next_unlock: null, unlocked: [], tasks: [] };
  const BALANCE = { address: ADDRESS, embers: '37000000000000000000000', cgt: '37,000', display: '37,000 CGT', in_sparks: '37,000,000 Sparks' };
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: SESSION,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark', sub_units_per_cgt: 1000, atomic_unit: 'ember' },
  };

  const HOST_STUB = `
    window.__CALLS__ = [];
    // Whether ARQADE's app is installed here, and whether it fails to start.
    window.__ARQ__ = { here: true, fails: false };
    window.__CALLBACKS__ = {};
    window.__LISTENERS__ = {};
    let callbacks = 0;
    window.__emit = (event, payload) => {
      for (const id of window.__LISTENERS__[event] ?? []) window.__CALLBACKS__[id]?.({ event, id, payload });
    };
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd.startsWith('arqade')) window.__CALLS__.push({ cmd, args });
        switch (cmd) {
          case 'plugin:event|listen':
            (window.__LISTENERS__[args.event] ??= []).push(args.handler);
            return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'arqade_app_open':
            return window.__ARQ__.fails ? Promise.reject({ kind: 'internal', message: 'ARQADE did not start: no graphics card' }) : Promise.resolve(window.__ARQ__.here);
          case 'arqade_app_place': case 'arqade_app_hide': case 'arqade_app_theme': case 'arqade_app_player': case 'arqade_app_prefs': case 'arqade_open':
            return Promise.resolve(null);
          case 'launcher_state': return Promise.resolve(${JSON.stringify(STATE)});
          case 'vault_status': return Promise.resolve({ state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} });
          case 'qor_progress': return Promise.resolve(${JSON.stringify(PROGRESS)});
          case 'cgt_balance': return Promise.resolve(${JSON.stringify(BALANCE)});
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
  // The system asks for less motion; the launcher's own setting is left at "system", so it follows.
  await send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-motion', value: 'reduce' }] });
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await send('Page.bringToFront');
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell and its rail');

  const calls = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)}).map((c) => c.args ?? null)`);
  const count = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)}).length`);
  const clickText = (scope, label) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => b.textContent.trim().startsWith(${JSON.stringify(label)})); if (!n) return false; n.click(); return true; })()`);
  const rail = (label) => clickText('nav[aria-label="Primary"] button', label);
  // Away from ARQADE and back, once the screen has really gone: a screen left mid-transition is kept, not made anew.
  const revisit = async () => {
    await rail('Nexus');
    await until(`!document.querySelector('[data-arqade-dock]') && ![...document.querySelectorAll('main button')].some((b) => b.textContent.trim() === 'Open ARQADE')`, 'the screen to go');
    await rail('ARQADE');
  };

  // --- the app here, docked ---------------------------------------------------------------------------
  check('the rail offers ARQADE', (await rail('ARQADE')) === true);
  check('the screen starts ARQADE\'s app', await eventually(`window.__CALLS__.some((c) => c.cmd === 'arqade_app_open')`));
  await until(`Boolean(document.querySelector('[data-arqade-dock]'))`, 'the docked area');
  check('and leaves it the whole area, with no page of its own over it', await evaluate(`!document.querySelector('main h1') || !document.querySelector('[data-arqade-dock]').closest('main')?.querySelector('h1')`));
  const area = await evaluate(`(() => { const r = document.querySelector('[data-arqade-dock]').getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height }; })()`);
  check('the area is named for a screen reader', (await evaluate(`document.querySelector('[data-arqade-dock]').getAttribute('aria-label')`)) === 'ARQADE');
  await until(`window.__CALLS__.some((c) => c.cmd === 'arqade_app_place')`, 'a place');
  const placed = (await calls('arqade_app_place')).at(-1).rect;
  check('the app is told where to sit, in physical pixels',
    placed.x === Math.round(area.x * 1.5) && placed.y === Math.round(area.y * 1.5) && placed.w === Math.round(area.w * 1.5) && placed.h === Math.round(area.h * 1.5),
    `${JSON.stringify(placed)} for ${JSON.stringify(area)}`);
  await until(`window.__CALLS__.some((c) => c.cmd === 'arqade_app_theme')`, 'a theme');
  const colors = (await calls('arqade_app_theme')).at(-1).colors;
  check('it is told the launcher\'s colours, as #rrggbb tokens',
    Object.keys(colors).length >= 6 && ['void', 'accent', 'ink'].every((t) => /^#[0-9a-f]{6}$/.test(colors[t] ?? '')), JSON.stringify(colors));
  check('it is told who is playing: signed in, their QOR ID, level, and balance in Sparks and CGT as the host wrote them',
    await eventually(`window.__CALLS__.some((c) => c.cmd === 'arqade_app_player' && c.args.player.signed_in === true && c.args.player.name === 'fixture' && c.args.player.level === 4 && c.args.player.sparks === '37,000,000 Sparks' && c.args.player.cgt === '37,000')`),
    JSON.stringify((await calls('arqade_app_player')).at(-1)));
  check('and that the person wants less motion', await eventually(`window.__CALLS__.some((c) => c.cmd === 'arqade_app_prefs' && c.args.prefs.reduced_motion === true)`));

  // --- leaving and coming back -------------------------------------------------------------------------
  const hidesBefore = await count('arqade_app_hide');
  await rail('Nexus');
  check('leaving the screen hides the app', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'arqade_app_hide').length > ${hidesBefore}`));
  await until(`!document.querySelector('[data-arqade-dock]')`, 'the screen to go');
  const placesBefore = await count('arqade_app_place');
  await rail('ARQADE');
  check('coming back shows it again', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'arqade_app_place').length > ${placesBefore}`));

  // --- the app ends while the screen shows ---------------------------------------------------------------
  await until(`Boolean(document.querySelector('[data-arqade-dock]'))`, 'the docked area again');
  await evaluate(`window.__emit('arqade://exited', { product: 'arqade', code: 0, stderr_tail: '' })`);
  check('when it ends, the screen says so', await eventually(`Boolean(document.querySelector('[data-arqade-ended]'))`));
  check('and offers the website', await evaluate(`[...document.querySelectorAll('main button')].some((b) => b.textContent.trim() === 'Open ARQADE')`));
  const opensBefore = await count('arqade_app_open');
  await clickText('[data-arqade-ended] button', 'Open ARQADE again');
  check('and opens the app again when asked', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'arqade_app_open').length > ${opensBefore}`)
    && await eventually(`Boolean(document.querySelector('[data-arqade-dock]'))`));

  // --- the app not installed --------------------------------------------------------------------------------
  await evaluate(`window.__ARQ__.here = false`);
  await revisit();
  await until(`[...document.querySelectorAll('main button')].some((b) => b.textContent.trim() === 'Open ARQADE')`, 'the website panel');
  check('with the app not installed, the screen offers the website\'s window', !(await evaluate(`Boolean(document.querySelector('[data-arqade-dock]'))`)));
  await clickText('main button', 'Open ARQADE');
  check('which opens through the host', await eventually(`window.__CALLS__.some((c) => c.cmd === 'arqade_open')`));

  // --- the app failing to start ------------------------------------------------------------------------------
  await evaluate(`window.__ARQ__ = { here: true, fails: true }`);
  await revisit();
  check('when the app cannot start, the screen says why', await eventually(`(document.querySelector('main [role=alert]')?.textContent ?? '').includes('no graphics card')`));
  check('and still offers the website', await evaluate(`[...document.querySelectorAll('main button')].some((b) => b.textContent.trim() === 'Open ARQADE')`));
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
