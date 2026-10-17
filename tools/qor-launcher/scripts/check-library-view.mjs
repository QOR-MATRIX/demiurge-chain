// Does the Library draw what the index holds, and ask the host exactly what the person did? (ADR-094, L9.1 and L9.2)
//
// The host's tests prove the index (src-tauri/src/library: folders, scan, watch, search, thumbnails) and QOR Media's
// queue (src-tauri/src/media). This check covers what reaches the screen: it serves the built frontend in a real
// rendering engine with a stand-in host that answers the `library_*` and `media_*` commands from a FIXTURE, then drives
// the Library as a person would.
//
// What it holds the view to:
//   - with no folders, the folders a person might start with are offered, chosen, and added in one step;
//   - Home: a hero for what was last opened, with Resume, and shelves by kind, each tile with its title and one fact;
//   - a picture's tile shows its thumbnail, from the bytes the host sends;
//   - `/` searches from anywhere, results come as the person types, and Escape clears them;
//   - a kind's tab shows that kind; Games says plainly what waits;
//   - arrow keys move the focus ring along a shelf and between shelves; Enter opens, through `library_open`;
//   - a game controller does the same: a direction takes the first tile, then moves and repeats while held, A opens;
//   - QOR Media is told where to sit in physical pixels, and to hide when the person leaves the Library;
//   - Folders: one is taken out, a suggested one added;
//   - when the host says the index changed, the shelves are read again.
//
//   npm run build
//   node scripts/check-library-view.mjs [dist directory, default ./dist]
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

// --- the fixture: files that belong to no one ------------------------------------------------------------------
const VIDEOS = 'C:\\Users\\Fixture\\Videos';
const PICTURES = 'C:\\Users\\Fixture\\Pictures';
const item = (name, kind, extra = {}) => ({
  path: `${kind === 'picture' ? PICTURES : VIDEOS}\\${name}`, folder: kind === 'picture' ? PICTURES : VIDEOS, name, kind,
  size: 734003200, modified: 1760000000, added: 1760000000, opened: null, position: null, duration: null, width: null,
  height: null, taken: null, camera: null, title: null, artist: null, album: null, ...extra,
});
const FILM = item('Night Drive.mkv', 'video', { duration: 5420, height: 2160, opened: 1760100000, position: 1200 });
const CLIP = item('Holiday clip.mp4', 'video', { duration: 95, height: 1080 });
const SONG = item('lattice.flac', 'audio', { title: 'Midnight Lattice', artist: 'Fixture Band', duration: 245 });
const PHOTO = item('sunset.png', 'picture', { width: 4032, height: 3024 });
const NOTES = item('notes.md', 'document', { size: 2048 });
const SHELF = { video: [FILM, CLIP], audio: [SONG], picture: [PHOTO], document: [NOTES] };
// A JPEG of one pixel, as the host would send a thumbnail.
const JPEG = [255, 216, 255, 224, 0, 16, 74, 70, 73, 70, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 255, 219, 0, 67, 0, 8, 6, 6, 7, 6, 5, 8, 7, 7, 7, 9, 9, 8, 10, 12, 20, 13, 12, 11, 11, 12, 25, 18, 19, 15, 20, 29, 26, 31, 30, 29, 26, 28, 28, 32, 36, 46, 39, 32, 34, 44, 35, 28, 28, 40, 55, 41, 44, 48, 49, 52, 52, 52, 31, 39, 57, 61, 56, 50, 60, 46, 51, 52, 50, 255, 192, 0, 11, 8, 0, 1, 0, 1, 1, 1, 17, 0, 255, 196, 0, 20, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 255, 196, 0, 20, 16, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 255, 218, 0, 8, 1, 1, 0, 0, 63, 0, 42, 159, 255, 217];

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

const userData = await mkdtemp(join(tmpdir(), 'qor-library-'));
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

  const ADDRESS = '5FixtureAddressForTheLibraryCheck000000000000000';
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: null,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };

  const HOST_STUB = `
    window.__CALLS__ = [];
    window.__LIB__ = { folders: [], suggested: [${JSON.stringify(VIDEOS)}, ${JSON.stringify(PICTURES)}] };
    window.__CALLBACKS__ = {};
    window.__LISTENERS__ = {};
    let callbacks = 0;
    window.__emit = (event, payload) => {
      for (const id of window.__LISTENERS__[event] ?? []) window.__CALLBACKS__[id]?.({ event, id, payload });
    };
    const SHELF = ${JSON.stringify(SHELF)};
    const ALL = [...SHELF.video, ...SHELF.audio, ...SHELF.picture, ...SHELF.document];
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd.startsWith('library_') || cmd.startsWith('media_')) window.__CALLS__.push({ cmd, args });
        const lib = window.__LIB__;
        switch (cmd) {
          case 'plugin:event|listen':
            (window.__LISTENERS__[args.event] ??= []).push(args.handler);
            return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'library_folders': return Promise.resolve({ folders: lib.folders, suggested: lib.suggested.filter((s) => !lib.folders.includes(s)) });
          case 'library_add_folder':
            lib.folders = [...lib.folders, args.path ?? 'C:\\\\Picked'];
            return Promise.resolve(args.path ?? 'C:\\\\Picked');
          case 'library_remove_folder':
            lib.folders = lib.folders.filter((f) => f !== args.path);
            return Promise.resolve(null);
          case 'library_shelf':
            if (!lib.folders.length) return Promise.resolve([]);
            if (args.order === 'opened') return Promise.resolve([SHELF.video[0]]);
            return Promise.resolve(args.kind ? SHELF[args.kind] ?? [] : ALL);
          case 'library_counts': return Promise.resolve(lib.folders.length ? [['video', 2], ['audio', 1], ['picture', 1], ['document', 1]] : []);
          case 'library_search':
            return Promise.resolve(ALL.filter((i) => i.name.toLowerCase().includes(args.query.toLowerCase().trim()) || (i.title ?? '').toLowerCase().includes(args.query.toLowerCase().trim())));
          case 'library_thumbnail': return Promise.resolve(new Uint8Array(${JSON.stringify(JPEG)}).buffer);
          case 'library_open': case 'library_reveal': case 'library_scan': case 'media_place': case 'media_hide':
            return Promise.resolve(cmd === 'library_scan' ? { seen: 5, added: 0, changed: 0, removed: 0 } : null);
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
  const key = (k) => evaluate(`(document.activeElement ?? document.body).dispatchEvent(new KeyboardEvent('keydown', { key: ${JSON.stringify(k)}, bubbles: true }))`);
  const focused = () => evaluate(`document.activeElement?.getAttribute('data-tile') ?? document.activeElement?.tagName`);

  // --- first run: choose folders -------------------------------------------------
  check('the rail offers the Library', (await clickText('nav[aria-label="Primary"] button', 'Library')) === true);
  await until(`Boolean(document.querySelector('[data-library-welcome]'))`, 'the first-run card');
  const offered = await evaluate(`[...document.querySelectorAll('[data-library-welcome] input[type=checkbox]')].map((c) => c.checked)`);
  check('with no folders, the folders a person might start with are offered, already chosen', offered.length === 2 && offered.every(Boolean));
  await clickText('[data-library-welcome] button', 'Add these 2');
  check('and added in one step, each by its path', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'library_add_folder').length === 2`)
    && JSON.stringify((await calls('library_add_folder')).map((a) => a.path)) === JSON.stringify([VIDEOS, PICTURES]));

  // --- Home ---------------------------------------------------------------------------
  await until(`Boolean(document.querySelector('[data-library-hero]'))`, 'the hero');
  const hero = await text('[data-library-hero]');
  check('the hero is what was last opened, with Resume and its facts', hero.includes('Night Drive') && hero.includes('Resume') && hero.includes('1:30:20') && hero.includes('2160p'), hero);
  const shelves = await evaluate(`[...document.querySelectorAll('[data-shelf]')].map((s) => s.getAttribute('data-shelf'))`);
  check('shelves by kind follow', ['Recently added', 'Videos', 'Music', 'Pictures', 'Documents'].every((s) => shelves.includes(s)), JSON.stringify(shelves));
  const song = await text(`[data-shelf="Music"] [data-tile]`);
  check('a song shows its title, artist and length', song.includes('Midnight Lattice') && song.includes('Fixture Band') && song.includes('4:05'), song);
  check('a picture shows its size', (await text(`[data-shelf="Pictures"] [data-tile]`)).includes('4032 × 3024'));
  check('a picture tile shows its thumbnail from the host\'s bytes', await eventually(`Boolean(document.querySelector('[data-shelf="Pictures"] [data-tile] img')?.src.startsWith('blob:'))`));
  check('a part-played film shows how far it got', await eventually(`Boolean(document.querySelector('[data-library] .library-progress'))`));

  // --- docking QOR Media ------------------------------------------------------------
  const placed = await calls('media_place');
  const last = placed[placed.length - 1]?.rect;
  const css = await evaluate(`(() => { const r = document.querySelector('[data-library]').getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height }; })()`);
  check('QOR Media is told where to sit, in physical pixels', !!last && last.w === Math.round(css.w * 1.5) && last.x === Math.round(css.x * 1.5), JSON.stringify({ last, css }));

  // --- keys --------------------------------------------------------------------------
  await evaluate(`document.querySelector('[data-shelf="Recently added"] [data-tile]').focus()`);
  const first = await focused();
  await key('ArrowRight');
  const second = await focused();
  check('ArrowRight moves along a shelf', first !== second && second.includes('Holiday'), `${first} → ${second}`);
  await key('ArrowDown');
  check('ArrowDown moves to the shelf below', (await evaluate(`document.activeElement?.closest('[data-shelf]')?.getAttribute('data-shelf')`)) === 'Videos');
  await evaluate(`document.activeElement.click()`);
  check('opening a tile asks the host to open exactly that file', await eventually(`window.__CALLS__.some((c) => c.cmd === 'library_open')`)
    && (await calls('library_open'))[0].path.endsWith('.mkv') || (await calls('library_open'))[0]?.path.endsWith('.mp4'));

  // --- a game controller, in the standard layout -------------------------------------------
  // The page reads controllers through the Gamepad API; here a stand-in controller is what it reads.
  await evaluate(`(() => {
    const buttons = Array.from({ length: 17 }, () => ({ pressed: false, touched: false, value: 0 }));
    window.__PAD__ = { id: 'fixture pad', index: 0, connected: true, mapping: 'standard', timestamp: 0, axes: [0, 0, 0, 0], buttons };
    navigator.getGamepads = () => [window.__PAD__, null, null, null];
    window.dispatchEvent(new Event('gamepadconnected'));
    document.activeElement?.blur();
  })()`);
  const pad = (button, down) => evaluate(`window.__PAD__.buttons[${button}].pressed = ${down}`);
  const tap = async (button) => { await pad(button, true); await sleep(80); await pad(button, false); await sleep(80); };
  const opensBefore = (await calls('library_open')).length;
  await tap(15);
  check("a controller's first press takes the first tile", await eventually(`document.activeElement === document.querySelector('[data-library] [data-tile]')`));
  await tap(15);
  check('the D-pad moves along a shelf', await eventually(`document.activeElement === document.querySelectorAll('[data-library] [data-row]')[0].querySelectorAll('[data-tile]')[1]`), await focused());
  await evaluate(`window.__PAD__.axes[1] = 1`);
  await sleep(80);
  await evaluate(`window.__PAD__.axes[1] = 0`);
  check('the left stick moves to the shelf below', await eventually(`document.activeElement?.closest('[data-row]') === document.querySelectorAll('[data-library] [data-row]')[1]`), await focused());
  await evaluate(`document.querySelector('[data-shelf="Recently added"] [data-tile]').focus()`);
  // Held for 650 ms: once at once, again at 380 ms, then every 110 ms (490, 600): three steps along.
  await pad(15, true);
  await sleep(650);
  await pad(15, false);
  const along = await evaluate(`[...document.activeElement.closest('[data-row]').querySelectorAll('[data-tile]')].indexOf(document.activeElement)`);
  check('held, a direction repeats, quickly after the first pause', along >= 3, String(along));
  // A held as long opens once: opening does not repeat.
  await pad(0, true);
  await sleep(650);
  await pad(0, false);
  await sleep(300);
  const opened = (await calls('library_open')).length - opensBefore;
  check('A opens the focused tile, once, however long it is held', opened === 1, String(opened));
  await evaluate(`window.__PAD__.connected = false; navigator.getGamepads = () => [null, null, null, null]`);

  // --- search ---------------------------------------------------------------------------
  await key('/');
  check('/ puts the cursor in search from anywhere in the Library', await eventually(`document.activeElement?.hasAttribute('data-library-search')`));
  await type('[data-library-search]', 'holi');
  check('results come as the person types', await eventually(`(document.querySelector('[data-library-results]')?.textContent ?? '').includes('Holiday clip')`)
    && (await calls('library_search')).some((a) => a.query === 'holi'));
  await key('Escape');
  check('Escape clears the search', await eventually(`!document.querySelector('[data-library-results]') && Boolean(document.querySelector('[data-library-hero]'))`));

  // --- tabs ----------------------------------------------------------------------------
  await click('[data-library-tab="audio"]');
  check('a kind\'s tab shows only that kind', await eventually(`[...document.querySelectorAll('[data-library] [data-tile]')].length === 1 && document.querySelector('[data-library] [data-tile]').textContent.includes('Midnight Lattice')`));
  await click('[data-library-tab="games"]');
  check('Games says plainly what it waits for', (await text('[data-library-panel]')).includes('L7.1'));

  // --- folders ---------------------------------------------------------------------------
  await click('[data-library-tab="folders"]');
  await until(`Boolean(document.querySelector('[data-library-folders]'))`, 'the folders');
  await evaluate(`[...document.querySelectorAll('[data-library-folders] button')].find((b) => b.getAttribute('aria-label') === ${JSON.stringify(`Take ${PICTURES} out of the Library`)})?.click()`);
  check('a folder is taken out by its path', await eventually(`window.__CALLS__.some((c) => c.cmd === 'library_remove_folder' && c.args.path === ${JSON.stringify(PICTURES)})`));
  await until(`[...document.querySelectorAll('[data-library-folders] button')].some((b) => b.textContent.trim() === 'Add')`, 'the suggestion back');
  await clickText('[data-library-folders] button', 'Add');
  check('and a suggested one added back', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'library_add_folder' && c.args.path === ${JSON.stringify(PICTURES)}).length === 2`));

  // --- changes and leaving ------------------------------------------------------------------
  await click('[data-library-tab="all"]');
  await until(`Boolean(document.querySelector('[data-library-hero]'))`, 'Home again');
  const before = (await calls('library_shelf')).length;
  await evaluate(`window.__emit('library://changed', null)`);
  check('when the host says the index changed, the shelves are read again', await eventually(`window.__CALLS__.filter((c) => c.cmd === 'library_shelf').length > ${before}`));
  await clickText('nav[aria-label="Primary"] button', 'Nexus');
  check('leaving the Library hides QOR Media', await eventually(`window.__CALLS__.some((c) => c.cmd === 'media_hide')`));
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
