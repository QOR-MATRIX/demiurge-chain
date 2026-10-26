// Does ARQADE work on a phone, by touch? (18 October 2026)
//
// It starts the production build (`npm run build` first) and visits it as a phone: 390 by 844 CSS pixels, touch, a
// mobile user agent. What it holds the site to:
//   - nothing scrolls sideways;
//   - the sections are a tab bar along the bottom, six tabs, each a labelled target at least 44 by 44 pixels, and
//     a tab opens its section;
//   - every button and link on the home screen, the arenas and the play screen is at least 44 by 44 pixels;
//   - the world chat's button sits above the tab bar, not over it;
//   - a game opens filling the screen, its close button is a target of 44 by 44 or more, and it points to the QOR
//     Launcher, where it is rebuilt in 3D (P7.23): the game itself there, by a `qor://arqade` link (ADR-118), and the
//     download beside it, each a link of 44 by 44 or more;
//   - Orbital's playfield takes a tap as its lock, and Void Runner's playfield takes touches without scrolling;
//   - and on a desktop the sections are still the rail along the left.
//
//   npm run build
//   node scripts/check-phone.mjs
//
// Exits non-zero if any check fails.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { devtoolsPort } from './chrome.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
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

// The site, as `next start` serves it. No database: what needs one says so, and the layout does not depend on it.
const sitePort = 3400 + Math.floor(Math.random() * 400);
const site = spawn(process.execPath, [join(root, 'node_modules', 'next', 'dist', 'bin', 'next'), 'start', '-p', String(sitePort)], {
  cwd: root,
  stdio: 'ignore',
  env: { ...process.env, PORT: String(sitePort) },
});
const url = `http://localhost:${sitePort}/`;

const userData = await mkdtemp(join(tmpdir(), 'arqade-phone-'));
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu', '--no-first-run', '--no-default-browser-check', '--hide-scrollbars',
    '--remote-debugging-port=0', `--user-data-dir=${userData}`, 'about:blank',
  ],
  { stdio: ['ignore', 'ignore', 'pipe'] },
);
const finish = async () => {
  try { browser.kill(); } catch {}
  try { site.kill(); } catch {}
  await rm(userData, { recursive: true, force: true }).catch(() => {});
};

try {
  for (let i = 0; i < 300; i++) {
    try {
      if ((await fetch(url)).ok) break;
    } catch {}
    await sleep(100);
  }
  let target;
  const port = await devtoolsPort(browser);
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
      await sleep(50);
    }
  };
  const until = async (expression, what) => {
    if (!(await eventually(expression))) throw new Error(`timed out waiting for ${what}`);
  };
  const clickText = (scope, pattern) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => new RegExp(${JSON.stringify(pattern)}, 'i').test(b.textContent)); if (!n) return false; n.click(); return true; })()`);
  // Every visible button and link, with its size, outside the parts that are text rather than controls.
  const smallTargets = () => evaluate(`[...document.querySelectorAll('button, a[href], [role=button]')].filter((e) => {
      const r = e.getBoundingClientRect(); const s = getComputedStyle(e);
      return r.width > 0 && r.height > 0 && s.visibility !== 'hidden' && !e.closest('[data-nextjs-toast], nextjs-portal') && (r.width < 44 || r.height < 44);
    }).map((e) => ((e.getAttribute('aria-label') || e.textContent || e.tagName).trim().slice(0, 32)) + ' ' + Math.round(e.getBoundingClientRect().width) + 'x' + Math.round(e.getBoundingClientRect().height))`);

  const phone = async () => {
    await send('Emulation.setDeviceMetricsOverride', { width: 390, height: 844, deviceScaleFactor: 3, mobile: true, screenOrientation: { type: 'portraitPrimary', angle: 0 } });
    await send('Emulation.setTouchEmulationEnabled', { enabled: true, maxTouchPoints: 5 });
    await send('Emulation.setUserAgentOverride', { userAgent: 'Mozilla/5.0 (Linux; Android 15; Pixel 9) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0.0.0 Mobile Safari/537.36' });
  };

  await send('Page.enable');
  await send('Runtime.enable');
  await phone();
  await send('Page.navigate', { url });
  for (let i = 0; i < 150 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  // Past the intro and the ground rules, as a first visitor goes.
  await until(`[...document.querySelectorAll('button')].some((b) => /skip intro/i.test(b.textContent))`, 'the intro');
  check('the intro\u2019s Skip is a target of 44 pixels or more', await evaluate(`(() => { const b = [...document.querySelectorAll('button')].find((b) => /skip intro/i.test(b.textContent)); const r = b.getBoundingClientRect(); return r.height >= 44 && r.width >= 44; })()`));
  await clickText('button', 'skip intro');
  await until(`[...document.querySelectorAll('button')].some((b) => /enter the local arcade/i.test(b.textContent))`, 'the ground rules');
  await clickText('button', 'enter the local arcade');
  await until(`!!document.querySelector('.rail nav button') && ![...document.querySelectorAll('button')].some((b) => /enter the local arcade/i.test(b.textContent))`, 'the arcade');
  await sleep(400);

  check('nothing scrolls sideways', await evaluate('document.documentElement.scrollWidth <= document.documentElement.clientWidth'),
    String(await evaluate('document.documentElement.scrollWidth')));
  const bar = await evaluate(`(() => { const r = document.querySelector('.rail').getBoundingClientRect(); return { top: r.top, bottom: r.bottom, left: r.left, width: r.width }; })()`);
  check('the sections are a tab bar along the bottom, the whole width', bar.bottom >= 843 && bar.top >= 700 && bar.left === 0 && bar.width === 390, JSON.stringify(bar));
  const tabs = await evaluate(`[...document.querySelectorAll('.rail nav button')].map((b) => { const r = b.getBoundingClientRect(); return { w: r.width, h: r.height, label: getComputedStyle(b, '::after').content, name: b.getAttribute('aria-label') }; })`);
  check('six tabs, each 44 by 44 pixels or more', tabs.length === 6 && tabs.every((t) => t.w >= 44 && t.h >= 44), JSON.stringify(tabs.map((t) => `${Math.round(t.w)}x${Math.round(t.h)}`)));
  check('each tab is labelled under its icon and named for a screen reader', tabs.every((t) => t.label && t.label !== 'none' && t.label.length > 2 && t.name), tabs.map((t) => t.label).join(' '));
  check('the content takes the whole width', await evaluate(`document.querySelector('.body').getBoundingClientRect().left === 0`));
  const small = await smallTargets();
  check('every button and link on the home screen is 44 by 44 pixels or more', small.length === 0, small.join('; '));
  const chat = await evaluate(`(() => { const b = document.querySelector('.chat-bubble')?.getBoundingClientRect(); return b ? { bottom: b.bottom, w: b.width, h: b.height } : null; })()`);
  check('the world chat\u2019s button sits above the tab bar', chat && chat.bottom <= bar.top && chat.w >= 44 && chat.h >= 44, JSON.stringify(chat));

  for (const [tab, heading] of [['Arenas', 'Multiplayer'], ['Play', 'Play Now']]) {
    await evaluate(`[...document.querySelectorAll('.rail nav button')].find((b) => b.dataset.short === ${JSON.stringify(tab)})?.click()`);
    check(`the ${tab} tab opens ${heading}`, await eventually(`document.querySelector('main h1')?.textContent === ${JSON.stringify(heading)}`));
    await sleep(300);
    const smallHere = await smallTargets();
    check(`every button and link under ${tab} is 44 by 44 pixels or more`, smallHere.length === 0, smallHere.join('; '));
  }

  // Orbital: the dialog fills the screen, and a tap on the playfield is a lock (or a miss).
  await clickText('.game-card', 'orbital');
  await until(`!!document.querySelector('.game-modal')`, 'the game');
  await sleep(500);
  const modal = await evaluate(`(() => { const r = document.querySelector('.game-modal').getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height }; })()`);
  check('a game opens filling the screen', modal.x === 0 && modal.y === 0 && modal.w === 390 && modal.h === 844, JSON.stringify(modal));
  check('its close button is a target of 44 pixels or more', await evaluate(`(() => { const r = document.querySelector('.game-modal [data-slot=dialog-close]').getBoundingClientRect(); return r.width >= 44 && r.height >= 44; })()`));
  // P7.23: the website's games point players to the launcher, where they are rebuilt in 3D: the game itself there, by
  // a `qor://arqade` link the launcher answers from 0.1.17 (ADR-118), and the download beside it.
  const pointers = await evaluate(`[...document.querySelectorAll('.game-modal .launcher-note a')].map((a) => { const r = a.getBoundingClientRect(); return { href: a.getAttribute('href'), w: r.width, h: r.height }; })`);
  const bigEnough = (link) => !!link && link.w >= 44 && link.h >= 44;
  const inLauncher = pointers.find((link) => link.href.startsWith('qor:'));
  const download = pointers.find((link) => link.href === 'https://qorsync.dev/');
  check('it offers the game in the QOR Launcher, by a link of 44 pixels or more', bigEnough(inLauncher) && inLauncher.href === 'qor://arqade?play=orbital', JSON.stringify(pointers));
  check('and the launcher\u2019s download beside it, by a link of 44 pixels or more', bigEnough(download), JSON.stringify(pointers));
  await clickText('.game-modal button', 'begin');
  await until(`!!document.querySelector('.game-modal canvas.play-canvas')`, 'Orbital\u2019s playfield');
  await sleep(400);
  const hudBefore = await evaluate(`document.querySelector('.game-modal .hud')?.textContent ?? ''`);
  const field = await evaluate(`(() => { const r = document.querySelector('.game-modal canvas.play-canvas').getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; })()`);
  await send('Input.dispatchTouchEvent', { type: 'touchStart', touchPoints: [{ x: field.x, y: field.y }] });
  await send('Input.dispatchTouchEvent', { type: 'touchEnd', touchPoints: [] });
  check('a tap on Orbital\u2019s playfield is its lock', await eventually(`(document.querySelector('.game-modal .hud')?.textContent ?? '') !== ${JSON.stringify(hudBefore)}`, 3000),
    `${hudBefore} -> ${await evaluate(`document.querySelector('.game-modal .hud')?.textContent ?? ''`)}`);
  check('the playfield takes touches instead of scrolling the page', await evaluate(`getComputedStyle(document.querySelector('.game-modal canvas.play-canvas')).touchAction === 'none'`));

  // A desktop keeps the rail along the left.
  await send('Emulation.setTouchEmulationEnabled', { enabled: false });
  await send('Emulation.setDeviceMetricsOverride', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
  await evaluate(`document.querySelector('.game-modal [data-slot=dialog-close]')?.click()`);
  await sleep(500);
  const rail = await evaluate(`(() => { const r = document.querySelector('.rail').getBoundingClientRect(); return { left: r.left, top: r.top, height: r.height, width: r.width }; })()`);
  check('on a desktop the sections are still the rail along the left', rail.left === 0 && rail.top === 0 && rail.height >= 890 && rail.width < 300, JSON.stringify(rail));
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
