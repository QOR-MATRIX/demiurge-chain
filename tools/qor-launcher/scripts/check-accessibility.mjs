// Do the accessibility settings take effect? (roadmap L1.2)
//
// Serves the built frontend, opens it in a headless Chromium-family browser with
// a stand-in for the Tauri host (so the app's own applyTheme and applyA11y run at
// startup), sets the attributes applyA11y writes, and reads computed styles over
// the DevTools protocol. It checks the rendering engine's answer, not the source.
//
// No dependencies beyond Node 22+ (for the global WebSocket) and an installed
// Edge or Chrome. Set BROWSER_PATH to use another Chromium-family browser.
//
//   npm run build
//   node scripts/check-accessibility.mjs [dist directory, default ./dist]
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
    'C:/Program Files/Google/Chrome/Application/chrome.exe',
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium',
    '/usr/bin/chromium-browser',
    '/usr/bin/microsoft-edge',
  ].find((path) => existsSync(path));
if (!BROWSER) {
  console.error('No Edge or Chrome found. Set BROWSER_PATH to a Chromium-family browser.');
  process.exit(2);
}

const TYPES = {
  '.html': 'text/html',
  '.js': 'text/javascript',
  '.css': 'text/css',
  '.svg': 'image/svg+xml',
  '.png': 'image/png',
};

const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const file = join(dist, path === '/' ? 'index.html' : path);
  try {
    const body = await readFile(file);
    res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
    res.end(body);
  } catch {
    res.writeHead(404).end();
  }
});
await new Promise((ok) => server.listen(0, '127.0.0.1', ok));
const appUrl = `http://127.0.0.1:${server.address().port}/`;

const profile = await mkdtemp(join(tmpdir(), 'qor-a11y-'));
const port = 9300 + Math.floor(Math.random() * 500);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    // CI's container cannot give Chromium a sandbox (.woodpecker/launcher.yaml).
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu',
    '--no-first-run',
    '--no-default-browser-check',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    'about:blank',
  ],
  { stdio: 'ignore' },
);

const sleep = (ms) => new Promise((ok) => setTimeout(ok, ms));
let target;
// About 30 s: a CI runner has taken more than 10 s to give Chrome its first page (6 October 2026).
for (let i = 0; i < 300 && !target; i++) {
  try {
    const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
    target = list.find((t) => t.type === 'page');
  } catch {
    // Not answering yet.
  }
  // Pause on every miss, not only when the browser refuses the connection: it can answer before its page exists,
  // and a loop that did not wait then gave up within milliseconds on a slow runner (CI, 6 October 2026).
  if (!target) await sleep(100);
}
if (!target) throw new Error('The browser did not expose a page to inspect');

const ws = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((ok, bad) => {
  ws.onopen = ok;
  ws.onerror = bad;
});
let nextId = 0;
const pending = new Map();
const events = [];
ws.onmessage = (message) => {
  const msg = JSON.parse(message.data);
  if (msg.id && pending.has(msg.id)) {
    pending.get(msg.id)(msg);
    pending.delete(msg.id);
  } else if (msg.method) {
    events.push(msg);
  }
};
const send = (method, params = {}) =>
  new Promise((ok) => {
    const id = ++nextId;
    pending.set(id, ok);
    ws.send(JSON.stringify({ id, method, params }));
  });
const evaluate = async (expression) => {
  const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (r.result.exceptionDetails) {
    throw new Error(`${expression}\n${JSON.stringify(r.result.exceptionDetails)}`);
  }
  return r.result.result.value;
};

// A stand-in for the Tauri host. Every command fails the way an unreachable host
// would, which is enough for the shell to render and run its bootstrap.
const HOST_STUB = `
  window.__TAURI_INTERNALS__ = {
    invoke: () => Promise.reject({ kind: 'internal', message: 'no host in this check' }),
    transformCallback: () => 0,
    unregisterCallback: () => {},
    convertFileSrc: (s) => s,
    metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
    plugins: {},
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };

  // Every frame callback anything schedules, counted. The QFX checks below ask
  // "is a callback being scheduled", which is the only honest test of "still":
  // a paused loop and a slow one both look motionless in a single screenshot.
  window.__frames = 0;
  const schedule = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = (callback) => {
    window.__frames += 1;
    return schedule(callback);
  };
`;

await send('Page.enable');
await send('Runtime.enable');
await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
await send('Page.navigate', { url: appUrl });
for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) {
  await sleep(100);
}
await sleep(800); // let the bootstrap effect run applyTheme and applyA11y

const results = [];
const check = (name, actual, expected) => {
  const pass = typeof expected === 'function' ? expected(actual) : actual === expected;
  results.push({ name, pass, actual });
};

const reset = `(() => {
  const r = document.documentElement;
  r.setAttribute('data-contrast', 'normal');
  r.setAttribute('data-motion', 'system');
  for (const a of ['data-solid', 'data-bold-focus', 'data-readable']) r.removeAttribute(a);
  document.querySelectorAll('[data-check]').forEach((n) => n.remove());
})()`;
const probe = (tag, cls) => `(() => {
  const n = document.createElement('${tag}');
  n.className = '${cls}'; n.setAttribute('data-check', ''); n.textContent = 'Probe';
  document.body.appendChild(n); return n;
})()`;
const token = (name) =>
  `getComputedStyle(document.documentElement).getPropertyValue('${name}').trim().toLowerCase()`;
const setAttr = (name, value = '') => `document.documentElement.setAttribute('${name}', '${value}')`;

check(
  'the app applied a theme at startup',
  await evaluate(`document.documentElement.getAttribute('data-theme')`),
  'architect',
);

// Contrast.
await evaluate(reset);
check('normal contrast: --ink-muted is the theme value', await evaluate(token('--ink-muted')), '#93a0ae');
await evaluate(setAttr('data-contrast', 'high'));
check('high contrast raises --ink-muted', await evaluate(token('--ink-muted')), '#b4bfca');
check('high contrast raises --edge', await evaluate(token('--edge')), '#49525f');
await evaluate(setAttr('data-contrast', 'maximum'));
check('maximum contrast raises --ink-faint', await evaluate(token('--ink-faint')), '#b9c2cc');
await evaluate(probe('div', 'surface'));
check(
  'maximum contrast makes panels opaque (no backdrop blur)',
  await evaluate(`getComputedStyle(document.querySelector('[data-check].surface')).backdropFilter`),
  'none',
);

// Reduced transparency.
await evaluate(reset);
await evaluate(setAttr('data-solid'));
await evaluate(probe('div', 'glass'));
check(
  'reduced transparency makes glass opaque',
  await evaluate(`getComputedStyle(document.querySelector('[data-check].glass')).backdropFilter`),
  'none',
);

// Bold focus on a text field.
await evaluate(reset);
await evaluate(setAttr('data-bold-focus'));
await evaluate(`(() => { const n = ${probe('input', 'field')}; n.focus(); })()`);
check(
  'bold focus shows a 3px ring on a focused field',
  await evaluate(`getComputedStyle(document.querySelector('[data-check].field')).outlineWidth`),
  '3px',
);

// Readable text on a heading.
await evaluate(reset);
await evaluate(setAttr('data-readable'));
await evaluate(probe('p', 'heading'));
check(
  'readable text drops the uppercase on headings',
  await evaluate(`getComputedStyle(document.querySelector('[data-check].heading')).textTransform`),
  'none',
);

// Motion.
const buttonTransition = `(() => {
  const n = document.querySelector('[data-check].btn') ?? ${probe('button', 'btn')};
  return getComputedStyle(n).transitionDuration;
})()`;
const animated = (v) => v.split(',').some((d) => parseFloat(d) >= 0.1);
const still = (v) => v.split(',').every((d) => parseFloat(d) < 0.001);

await evaluate(reset);
check('system motion, OS allows motion: buttons animate', await evaluate(buttonTransition), animated);
await evaluate(setAttr('data-motion', 'reduced'));
check('reduced motion in the app stills buttons', await evaluate(buttonTransition), still);

await send('Emulation.setEmulatedMedia', {
  features: [{ name: 'prefers-reduced-motion', value: 'reduce' }],
});
await evaluate(reset);
check('system motion, OS reduces motion: buttons are still', await evaluate(buttonTransition), still);
await evaluate(setAttr('data-motion', 'full'));
check('full motion in the app overrides the OS setting', await evaluate(buttonTransition), animated);

// --- QFX: the backdrop does what the Ambience setting says --------------------
//
// These drive the real application. Settings are stored where the app keeps
// them and the page is reloaded, so the app's own loadA11y, applyA11y and
// effectiveAmbience decide what `data-ambience` becomes; nothing here computes
// the expected answer and then writes it. A runtime change is made by writing
// `data-ambience` exactly as applyA11y does when the setting moves.
//
// What is measured is behaviour, not markup: whether a frame callback is still
// being scheduled, and whether the canvas and the scrim are displayed.
//
// This replaces six cases that could not fail. They computed the expected
// ambience inside the check, wrote it to the attribute and read it back, so they
// passed whatever the app did. Proven on 21 September 2026: with
// effectiveAmbience planted to ignore reduced motion, the cases below that
// depend on it fail; so do the Off cases against the canvas as it was, which
// kept its last frame on screen with the scrim removed.
const boot = async (settings, osReducesMotion = false) => {
  await send('Emulation.setEmulatedMedia', {
    features: [{ name: 'prefers-reduced-motion', value: osReducesMotion ? 'reduce' : 'no-preference' }],
  });
  await evaluate(`localStorage.setItem('qor.a11y', ${JSON.stringify(JSON.stringify(settings))})`);
  events.length = 0;
  await send('Page.reload', { ignoreCache: true });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) {
    await sleep(100);
  }
  // A headless page can come back from a reload reporting itself hidden, and a
  // hidden page runs no frame callbacks at all -- correctly. Every "is a frame
  // being scheduled" case below would then read zero whatever the app did, so
  // the page is brought to the front and focused, and its visibility is a
  // precondition checked on every boot rather than assumed.
  await send('Page.bringToFront');
  await send('Emulation.setFocusEmulationEnabled', { enabled: true });
  await sleep(1200); // bootstrap, then the canvas settles into what it was told
  // The intro splash (src/qfx/ceremony/Intro.tsx) animates for about two
  // seconds on every open, and its frames are not the backdrop's. The cases
  // below measure the backdrop, so they wait for the intro to be gone first.
  for (let i = 0; i < 60 && (await evaluate(`Boolean(document.querySelector('[data-intro]'))`)); i++) {
    await sleep(100);
  }
  await sleep(400);
  check(
    'precondition: the page is visible, so frame callbacks run',
    await evaluate('document.visibilityState'),
    'visible',
  );
};
const framesOver = async (ms) => {
  const before = await evaluate('window.__frames');
  await sleep(ms);
  return (await evaluate('window.__frames')) - before;
};
const display = (selector) =>
  evaluate(`(() => { const n = document.querySelector('${selector}'); return n ? getComputedStyle(n).display : 'absent'; })()`);
const ambienceNow = () => evaluate(`document.documentElement.getAttribute('data-ambience')`);
const setAmbience = async (value) => {
  await evaluate(setAttr('data-ambience', value));
  await sleep(300);
};
const moving = (n) => n >= 20; // about sixty a second while live
const stillFrames = (n) => n === 0;

// Defaults. The positive control: if the backdrop is not running here, every
// "still" check below would pass on a dead canvas and prove nothing.
await boot({});
check('defaults: ambience is live', await ambienceNow(), 'live');
check('live: the backdrop schedules frames (positive control)', await framesOver(1000), moving);
check('live: the canvas is shown', await display('canvas.qfx-canvas'), 'block');
check('live: the scrim is shown', await display('.qfx-scrim'), 'block');

// Changing the setting while the launcher is open.
await setAmbience('still');
check('live -> still: no frame is scheduled', await framesOver(1000), stillFrames);
check('live -> still: the scrim stays', await display('.qfx-scrim'), 'block');
await setAmbience('off');
check('still -> off: no frame is scheduled', await framesOver(1000), stillFrames);
check('still -> off: the canvas is hidden, not left on its last frame', await display('canvas.qfx-canvas'), 'none');
check('still -> off: the scrim is removed', await display('.qfx-scrim'), 'none');
await setAmbience('live');
check('off -> live: the backdrop starts again', await framesOver(1000), moving);
check('off -> live: the canvas and scrim return', `${await display('canvas.qfx-canvas')}/${await display('.qfx-scrim')}`, 'block/block');

// Off as the stored choice: nothing is drawn from the first frame, and turning
// it on later still works.
await boot({ ambience: 'off' });
check('stored off: ambience is off', await ambienceNow(), 'off');
check('stored off: no frame is scheduled', await framesOver(1000), stillFrames);
check('stored off: the canvas is hidden', await display('canvas.qfx-canvas'), 'none');
await setAmbience('live');
check('stored off, then live: the backdrop starts', await framesOver(1000), moving);

// Reduced motion wins, decided by the app's own effectiveAmbience.
await boot({ ambience: 'live', motion: 'reduced' });
check('motion reduced in the app + ambience live gives still', await ambienceNow(), 'still');
check('motion reduced in the app: no frame is scheduled', await framesOver(1000), stillFrames);
await boot({ ambience: 'live', motion: 'system' }, true);
check('motion system + OS reduces motion gives still', await ambienceNow(), 'still');
check('OS reduces motion: no frame is scheduled', await framesOver(1000), stillFrames);
await boot({ ambience: 'live', motion: 'full' }, true);
check('motion full in the app overrides the OS: live', await ambienceNow(), 'live');

// Maximum contrast is a stated need: the backdrop goes, not merely dims.
await boot({ contrast: 'maximum' });
check('maximum contrast hides the canvas', await display('canvas.qfx-canvas'), 'none');

await boot({});
await evaluate(reset);

for (const r of results) {
  console.log(`${r.pass ? 'PASS' : 'FAIL'}  ${r.name}  (${JSON.stringify(r.actual)})`);
}
const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);

ws.close();
browser.kill();
server.close();
await sleep(500);
await rm(profile, { recursive: true, force: true }).catch(() => {});
process.exit(failed ? 1 : 0);
