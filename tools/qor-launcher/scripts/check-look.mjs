// Does the launcher wear Demiurge's one look? (ADR-108, ADR-109; DIRECTION P2.9)
//
// docs/design/tokens.mjs --check proves the generated files are current and every text colour reads; check-design.mjs
// keeps colour literals in the token files, and check-contrast.mjs holds every theme, Obsidian among them, as painted.
// This check serves the built frontend in a real rendering engine, against a stand-in host, and holds what is left:
//
//   - a launcher that had another theme stored, and no record of the move, opens on Obsidian and records the move
//     (ADR-109: everyone moves once);
//   - a theme picked after the move stays;
//   - Obsidian is the source's palette, value by value, read from docs/design/tokens.json, its structure the gilt;
//   - the four faces are the launcher's own: each loads from the build, the reading, interface and numeric text is set
//     in them, and nothing is asked of any other address;
//   - the wordmark is Cinzel in gilt (the tool register);
//   - the window's close button, under the pointer, is the status red with dark ink, 4.5:1 or more;
//   - the person's Motion setting reaches the host, for what the launcher starts (ADR-108 decision 5): less motion
//     when the system asks for it under "system", and all of it when the person chose "full".
//
//   npm run build
//   node scripts/check-look.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { devtoolsPort } from './chrome.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const dist = resolve(process.argv[2] ?? join(here, '..', 'dist'));
if (!existsSync(join(dist, 'index.html'))) {
  console.error(`No build at ${dist}. Run \`npm run build\` first.`);
  process.exit(2);
}
const source = JSON.parse(readFileSync(join(here, '..', '..', '..', 'docs', 'design', 'tokens.json'), 'utf8'));
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

// WCAG 2.2's contrast ratio of two opaque colours, given as rgb(r, g, b).
const channels = (css) => (css.match(/[\d.]+/g) ?? []).slice(0, 3).map(Number);
const ratio = (a, b) => {
  const lum = ([r, g, b]) => {
    const lin = (c) => ((c /= 255) <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
  };
  const [x, y] = [lum(channels(a)), lum(channels(b))].sort((p, q) => q - p);
  return (x + 0.05) / (y + 0.05);
};
const hex = (css) => `#${channels(css).map((c) => c.toString(16).padStart(2, '0')).join('')}`;

// --- serve the build -------------------------------------------------------
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.woff2': 'font/woff2', '.ico': 'image/x-icon' };
const served = [];
const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  served.push(path);
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

const userData = await mkdtemp(join(tmpdir(), 'qor-look-'));
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu', '--no-first-run', '--no-default-browser-check',
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

  const ADDRESS = '5FixtureAddressForTheLookCheck0000000000000000';
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: null,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark', sub_units_per_cgt: 1000, atomic_unit: 'ember' },
  };
  const HOST_STUB = `
    window.__CALLS__ = [];
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        window.__CALLS__.push({ cmd, args });
        switch (cmd) {
          case 'plugin:event|listen': return Promise.resolve(args.handler);
          case 'plugin:event|unlisten': return Promise.resolve(null);
          case 'launcher_motion': return Promise.resolve(null);
          case 'launcher_state': return Promise.resolve(${JSON.stringify(STATE)});
          case 'vault_status': return Promise.resolve({ state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} });
          case 'touch_vault': case 'vault_lock': case 'vault_set_active': case 'social_disconnect': case 'social_connect': return Promise.resolve(null);
          case 'social_status': return Promise.resolve({ status: { state: 'off' }, welcome: null, endpoint: '' });
          default: return Promise.reject({ kind: 'internal', message: 'no host in this check' });
        }
      },
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s) => s,
      metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
      plugins: {},
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  `;

  await send('Page.enable');
  await send('Runtime.enable');
  await send('Network.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  const load = async () => {
    events.length = 0;
    await send('Page.navigate', { url: appUrl });
    for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
    await send('Page.bringToFront');
    await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell and its rail');
  };

  // A launcher from before the move: another theme stored, no record of the move. The system asks for less motion and
  // the person's setting is "system".
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await evaluate(`localStorage.clear(); localStorage.setItem('qor.theme', 'sanguine'); true`);
  await send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-motion', value: 'reduce' }] });
  await load();

  // --- everyone moves once ----------------------------------------------------------------------------------------
  const theme = await evaluate(`document.documentElement.getAttribute('data-theme')`);
  check('a launcher that had Sanguine opens on Obsidian', theme === 'obsidian', theme);
  check('and records the move, with Obsidian stored',
    (await evaluate(`localStorage.getItem('qor.theme.moved') === 'obsidian' && localStorage.getItem('qor.theme') === 'obsidian'`)) === true);

  // --- Obsidian is the source's -------------------------------------------------------------------------------------
  const C = source.color;
  const want = {
    '--void': C.void, '--base': C.base, '--surface': C.surface, '--raised': C.raised, '--well': C.well,
    '--accent': C.accent, '--accent-bright': C.accentBright, '--accent-dim': C.accentDim, '--counter': C.giltDim,
    '--ink': C.ink, '--ink-body': C.inkBody, '--ink-muted': C.inkMuted, '--ink-faint': C.inkFaint,
    '--edge': C.edge, '--edge-soft': C.edgeSoft,
  };
  const got = await evaluate(`(() => { const s = getComputedStyle(document.documentElement); return Object.fromEntries(${JSON.stringify(Object.keys(want))}.map((k) => [k, s.getPropertyValue(k).trim().toLowerCase()])); })()`);
  const differ = Object.entries(want).filter(([k, v]) => got[k] !== v.toLowerCase());
  check('Obsidian is the source\'s palette, value by value, its structure the gilt', differ.length === 0,
    differ.map(([k, v]) => `${k} ${got[k]} not ${v}`).join(', '));

  // --- the four faces, the launcher's own ---------------------------------------------------------------------------
  const families = Object.values(source.font).map((f) => f.family);
  const loaded = await evaluate(`(async () => {
    const asks = [['400 13px "Barlow"'], ['600 13px "Barlow Semi Condensed"'], ['400 13px "IBM Plex Mono"'], ['600 13px "Cinzel"']];
    for (const [font] of asks) await document.fonts.load(font, 'QOR 0123');
    await document.fonts.ready;
    return [...document.fonts].filter((f) => f.status === 'loaded').map((f) => f.family.replaceAll('"', ''));
  })()`);
  const missing = families.filter((f) => !loaded.includes(f));
  check('each of the four faces loads from the build', missing.length === 0, missing.length ? `missing ${missing.join(', ')}` : [...new Set(loaded)].join(', '));
  // Bold reading text (font-semibold, font-bold) is drawn from Barlow's own SemiBold, not thickened by the browser from a
  // lighter face, which smears it.
  const semibold = await evaluate(`(async () => {
    await document.fonts.load('600 13px "Barlow"', 'Interface size');
    return [...document.fonts].some((f) => f.family.replaceAll('"', '') === 'Barlow' && String(f.weight) === '600' && f.status === 'loaded');
  })()`);
  check('bold reading text has a face of its own, not one the browser thickens', semibold === true);
  const fontOf = (selector) => evaluate(`(() => { const n = document.querySelector(${JSON.stringify(selector)}); return n ? getComputedStyle(n).fontFamily : null; })()`);
  const body = await fontOf('body');
  const heading = await fontOf('nav[aria-label="Primary"] button');
  const numeric = await evaluate(`(() => { const n = document.querySelector('.numeric'); return n ? getComputedStyle(n).fontFamily : null; })()`);
  check('reading text is set in Barlow', /^"?Barlow"?,/.test(body ?? ''), body);
  check('the rail is set in Barlow Semi Condensed', /^"?Barlow Semi Condensed"?/.test(heading ?? ''), heading);
  check('numbers are set in IBM Plex Mono', /^"?IBM Plex Mono"?/.test(numeric ?? ''), numeric);
  const elsewhere = events
    .filter((e) => e.method === 'Network.requestWillBeSent')
    .map((e) => e.params.request.url)
    .filter((u) => !u.startsWith(appUrl) && !u.startsWith('data:') && !u.startsWith('blob:'));
  check('and nothing is asked of any other address', elsewhere.length === 0, elsewhere.slice(0, 3).join(', '));
  check('the faces come from the build itself', served.some((p) => /\.woff2$/.test(p)), served.filter((p) => /\.woff2$/.test(p)).length + ' faces served');

  // --- the wordmark ----------------------------------------------------------------------------------------------
  const mark = await evaluate(`(() => { const n = [...document.querySelectorAll('header span')].find((s) => s.textContent.trim() === 'QOR'); if (!n) return null; const s = getComputedStyle(n); return { font: s.fontFamily, colour: s.color }; })()`);
  check('the wordmark is Cinzel in gilt', Boolean(mark) && /^"?Cinzel"?/.test(mark.font) && hex(mark.colour) === C.giltBright, JSON.stringify(mark));

  // --- closing, under the pointer -------------------------------------------------------------------------------
  // Tailwind 4 puts every hover: style under @media (hover: hover), which a browser without a pointer never matches
  // and which DevTools cannot emulate: CI's runner draws no hover at all. So the button's own hover: rules are read
  // from the stylesheet, whatever media they sit under, and their colours resolved on a probe, as hovering paints them.
  const close = await evaluate(`(() => {
    const button = document.querySelector('button[aria-label="Close"]');
    const wanted = [...button.classList].filter((c) => c.startsWith('hover:')).map((c) => CSS.escape(c));
    const found = {};
    const walk = (rules) => {
      for (const rule of rules) {
        if (rule.cssRules) walk(rule.cssRules);
        if (rule.selectorText && wanted.some((c) => rule.selectorText.includes('.' + c + ':hover')))
          for (const p of ['background-color', 'color']) if (rule.style.getPropertyValue(p)) found[p] = rule.style.getPropertyValue(p).trim();
      }
    };
    for (const sheet of document.styleSheets) { try { walk(sheet.cssRules); } catch {} }
    const probe = document.createElement('div');
    probe.style.backgroundColor = found['background-color'] ?? 'transparent';
    probe.style.color = found.color ?? 'inherit';
    document.body.appendChild(probe);
    const s = getComputedStyle(probe);
    const painted = { fill: s.backgroundColor, ink: s.color };
    probe.remove();
    return painted;
  })()`);
  check('the close button, under the pointer, is the status red with dark ink, 4.5:1 or more',
    hex(close.fill) === C.bad && ratio(close.ink, close.fill) >= 4.5, `${hex(close.fill)} with ${hex(close.ink)}, ${ratio(close.ink, close.fill).toFixed(2)}:1`);

  // --- the person's Motion setting, for what the launcher starts --------------------------------------------------
  check('less motion reaches the host when the system asks for it under "system"',
    await eventually(`window.__CALLS__.some((c) => c.cmd === 'launcher_motion' && c.args.reduced === true)`));

  // --- a theme picked after the move stays, and so does "full" motion ----------------------------------------------
  await evaluate(`localStorage.setItem('qor.theme', 'abyss'); localStorage.setItem('qor.a11y', JSON.stringify({ motion: 'full' })); true`);
  await load();
  const after = await evaluate(`document.documentElement.getAttribute('data-theme')`);
  check('a theme picked after the move stays', after === 'abyss', after);
  check('all of the motion reaches the host when the person chose "full"',
    await eventually(`window.__CALLS__.some((c) => c.cmd === 'launcher_motion' && c.args.reduced === false)`));
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
