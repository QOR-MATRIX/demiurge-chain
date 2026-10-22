// Every ARQADE screen as it is drawn, for design review (ADR-108): one PNG each, at a desktop's size and a phone's.
//
//   npm run build
//   node scripts/shots.mjs <folder>
//
// It starts the production build (`next start`, no database) and visits it in a headless Chromium-family browser
// (BROWSER_PATH to choose one): the intro, the ground rules, then every section on the rail, and a game opened. It
// checks nothing; `check-phone.mjs` checks the layout.

import { spawn } from 'node:child_process';
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const out = resolve(process.argv[2] ?? join(root, '.shots'));
mkdirSync(out, { recursive: true });
if (!existsSync(join(root, '.next', 'BUILD_ID'))) {
  console.error('No production build. Run `npm run build` first.');
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
  ].find((p) => existsSync(p));
if (!BROWSER) {
  console.error('No Chromium-family browser found. Set BROWSER_PATH.');
  process.exit(2);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const sitePort = 3400 + Math.floor(Math.random() * 400);
const site = spawn(process.execPath, [join(root, 'node_modules', 'next', 'dist', 'bin', 'next'), 'start', '-p', String(sitePort)], {
  cwd: root,
  stdio: 'ignore',
  env: { ...process.env, PORT: String(sitePort) },
});
const url = `http://localhost:${sitePort}/`;
const userData = await mkdtemp(join(tmpdir(), 'arqade-shots-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  ['--headless=new', '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--hide-scrollbars',
   `--remote-debugging-port=${port}`, `--user-data-dir=${userData}`, 'about:blank'],
  { stdio: 'ignore' },
);
let saved = 0;

try {
  for (let i = 0; i < 300; i++) {
    try {
      if ((await fetch(url)).ok) break;
    } catch {}
    await sleep(100);
  }
  let target;
  for (let i = 0; i < 300 && !target; i++) {
    if (i > 0) await sleep(100);
    try {
      target = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find((t) => t.type === 'page');
    } catch {}
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
  const evaluate = async (expression) => (await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true })).result?.result?.value;
  const until = async (expression, ms = 10000) => {
    const deadline = Date.now() + ms;
    while (!(await evaluate(expression))) {
      if (Date.now() > deadline) throw new Error(`timed out waiting for ${expression}`);
      await sleep(50);
    }
  };
  const clickText = (scope, pattern) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => new RegExp(${JSON.stringify(pattern)}, 'i').test(b.textContent)); if (!n) return false; n.click(); return true; })()`);
  const shot = async (name) => {
    await sleep(700);
    const png = (await send('Page.captureScreenshot', { format: 'png' })).result.data;
    writeFileSync(join(out, `${name}.png`), Buffer.from(png, 'base64'));
    saved += 1;
  };

  await send('Page.enable');
  await send('Runtime.enable');
  for (const [size, metrics] of [
    ['desktop', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false }],
    ['phone', { width: 390, height: 844, deviceScaleFactor: 2, mobile: true }],
  ]) {
    await send('Emulation.setDeviceMetricsOverride', metrics);
    await send('Page.navigate', { url });
    await evaluate(`localStorage.clear()`);
    await send('Page.reload');
    await until(`[...document.querySelectorAll('button')].some((b) => /skip intro/i.test(b.textContent))`);
    await shot(`${size}-intro`);
    await clickText('button', 'skip intro');
    await until(`[...document.querySelectorAll('button')].some((b) => /enter the local arcade/i.test(b.textContent))`);
    await shot(`${size}-ground-rules`);
    await clickText('button', 'enter the local arcade');
    await until(`!!document.querySelector('.rail nav button')`);
    await shot(`${size}-overview`);
    const sections = await evaluate(`[...document.querySelectorAll('.rail nav button')].map((b) => b.getAttribute('aria-label') || b.textContent.trim())`);
    for (const [i, section] of sections.entries()) {
      await evaluate(`document.querySelectorAll('.rail nav button')[${i}].click()`);
      await shot(`${size}-${String(i + 1).padStart(2, '0')}-${section.toLowerCase().replace(/[^a-z0-9]+/g, '-')}`);
    }
    await evaluate(`[...document.querySelectorAll('.rail nav button')].find((b) => /play/i.test(b.getAttribute('aria-label') || b.textContent))?.click()`);
    await sleep(300);
    if (await clickText('.game-card', 'orbital')) {
      await until(`!!document.querySelector('.game-modal')`);
      await shot(`${size}-game`);
      await evaluate(`document.querySelector('.game-modal [data-slot=dialog-close]')?.click()`);
    }
  }
  console.log(`${saved} screens saved in ${out}`);
} catch (error) {
  console.error(`stopped after ${saved} screens: ${error.message.split('\n')[0]}`);
  process.exitCode = 1;
} finally {
  try { browser.kill(); } catch {}
  try { site.kill(); } catch {}
  await rm(userData, { recursive: true, force: true }).catch(() => {});
}
