// Is text readable over ANY backdrop a theme could paint? (QFX layer one)
//
// QFX lets a creator put a GPU canvas behind the interface. A creator's theme is
// not a trusted input: it can paint any colour, at any brightness, anywhere, and
// it can change while someone is reading.
//
// So readability cannot be the theme's job. The chrome carries a scrim — one
// layer between the canvas and the content, whose opacity lives in qor.css and
// is deliberately NOT a theme token — and the guarantee is arithmetic: whatever
// the canvas paints, the composited background behind text stays close enough to
// the interface's own background that contrast holds.
//
// This check does not sample a few theme colours. Sampling proves those colours
// are fine and misses the adversarial one — the colour closest to the text,
// which is exactly where contrast collapses. It solves for the worst case over
// the whole colour cube and asserts the floor holds there. If the worst case
// passes, every theme passes.
//
// It runs against a REAL rendering engine and reads REAL computed styles, so it
// fails if someone changes a token, changes the scrim, or adds a rule that
// weakens either.
//
//   npm run build
//   node scripts/check-contrast.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.
//
// Proven to fail: on 21 September 2026 --qfx-scrim-alpha was lowered from 0.86
// to 0.30, which is the change someone would make to let more of a pretty
// backdrop through. Every worst-case check failed, on the default theme and on
// the deliberately loud test theme alike, while the interface still looked fine
// to the eye — which is the entire reason this is arithmetic and not judgement.

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const dist = resolve(process.argv[2] ?? join(here, '..', 'dist'));
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

const CONTRAST_FLOOR = 4.5; // WCAG 2.x AA for body text.

// EVERY THEME THE LAUNCHER SHIPS, each applied by the app itself: the check
// stores the theme's id where the app keeps it and reloads, so styles/themes.ts
// writes the tokens exactly as it does for a person. The list of ids is read
// from themes.ts, so a theme added there is covered here without anyone
// remembering to add it. Values are never copied into this file.
//
// Each theme is held to two things, for every ink step down to --ink-faint:
//   - 4.5:1 or better on every opaque background token it can sit on (void,
//     base, surface, raised, well), with no backdrop at all;
//   - 4.5:1 or better over ANY colour the QFX canvas could paint, through the
//     chrome's scrim.
// --ink-faint used to fail the first of those in every theme, at 2.87 to 3.26:1,
// before QFX existed. It was fixed on 22 September 2026 by the owner's decision,
// and this check was proven to fail against the old values first.
//
// Then one theme that does not ship: a deliberately loud one whose background
// is pushed to within a hair of its text, which is the worst thing a theme can
// do to readability short of setting them equal. It is reported, not failed,
// because a theme that is unreadable with the canvas off is a design-system
// problem the scrim cannot be asked to fix.
const THEME_IDS = [
  ...readFileSync(join(here, '..', 'src', 'styles', 'themes.ts'), 'utf8').matchAll(/^\s+id: '([a-z-]+)',\r?$/gm),
].map((m) => m[1]);
const LOUD = `root.style.setProperty('--base', '#f2f2f4'); root.style.setProperty('--ink', '#ffffff'); root.style.setProperty('--ink-body', '#fafafa'); root.style.setProperty('--ink-muted', '#f0f0f2'); root.style.setProperty('--ink-faint', '#e8e8ea')`;

const results = [];
const check = (name, pass, detail = '') => {
  results.push({ name, pass });
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}${detail ? `  (${detail})` : ''}`);
};

// --- serve the build -------------------------------------------------------

const TYPES = {
  '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css',
  '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json',
  '.woff2': 'font/woff2', '.ico': 'image/x-icon',
};

const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const file = join(dist, path === '/' ? 'index.html' : path.replace(/^\/+/, ''));
  try {
    const body = await readFile(file);
    res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' });
    res.end(body);
  } catch {
    res.writeHead(404).end('not found');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const appUrl = `http://127.0.0.1:${server.address().port}/`;

// --- drive a real rendering engine ----------------------------------------

const profile = await mkdtemp(join(tmpdir(), 'qor-contrast-'));
const port = 9400 + Math.floor(Math.random() * 300);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    // CI's container cannot give Chromium a sandbox (.woodpecker/launcher.yaml).
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu', '--no-first-run', '--no-default-browser-check',
    `--remote-debugging-port=${port}`, `--user-data-dir=${profile}`, 'about:blank',
  ],
  { stdio: 'ignore' },
);

const finish = async () => {
  try { browser.kill(); } catch {}
  try { server.close(); } catch {}
  await rm(profile, { recursive: true, force: true }).catch(() => {});
};

try {
  let target;
  // About 30 s: a CI runner has taken more than 10 s to give Chrome its first page (6 October 2026).
  for (let i = 0; i < 300 && !target; i++) {
    // Pause before every retry, not only after a refused connection: the browser can answer before its page
    // exists, and the loop then gave up within milliseconds on a slow runner (CI, 6 October 2026).
    if (i > 0) await sleep(100);
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      target = list.find((t) => t.type === 'page');
    } catch { await sleep(100); }
  }
  if (!target) throw new Error('The browser did not expose a page to inspect');

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((ok, bad) => { ws.onopen = ok; ws.onerror = bad; });
  let nextId = 0;
  const pending = new Map();
  const events = [];
  ws.onmessage = (m) => {
    const msg = JSON.parse(m.data);
    if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); }
    else if (msg.method) events.push(msg);
  };
  const send = (method, params = {}) =>
    new Promise((ok) => { const id = ++nextId; pending.set(id, ok); ws.send(JSON.stringify({ id, method, params })); });
  const evaluate = async (expression) => {
    const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (r.result.exceptionDetails) throw new Error(`${expression}\n${JSON.stringify(r.result.exceptionDetails)}`);
    return r.result.result.value;
  };

  await send('Page.enable');
  await send('Runtime.enable');
  // A stand-in for the Tauri host, as check-accessibility.mjs uses: every
  // command fails the way an unreachable host would, which is enough for the
  // app to run its bootstrap, and its bootstrap is what applies the theme.
  await send('Page.addScriptToEvaluateOnNewDocument', {
    source: `
      window.__TAURI_INTERNALS__ = {
        invoke: () => Promise.reject({ kind: 'internal', message: 'no host in this check' }),
        transformCallback: () => 0,
        unregisterCallback: () => {},
        convertFileSrc: (s) => s,
        metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
        plugins: {},
      };
      window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
    `,
  });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await sleep(500);

  // The arithmetic, in the page, against real computed styles. Kept in one
  // expression so it reads the same values the browser actually resolved.
  const MATHS = `
    (apply => {
      const root = document.documentElement;
      root.removeAttribute('style');
      if (apply) eval(apply);

      const cs = getComputedStyle(root);
      const parse = (v) => {
        v = (v || '').trim();
        const hex = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(v);
        if (hex) { const h = hex[1].length === 3 ? hex[1].split('').map(c=>c+c).join('') : hex[1];
                   return [0,2,4].map(i => parseInt(h.slice(i, i+2), 16)); }
        const fn = /^rgba?\\(([^)]+)\\)$/i.exec(v);
        if (fn) { const p = fn[1].split(/[,\\s/]+/).filter(Boolean).map(Number); if (p.length >= 3) return p.slice(0,3); }
        return null;
      };
      const lum = (c) => {
        const ch = (x) => { const v = x/255; return v <= 0.03928 ? v/12.92 : Math.pow((v+0.055)/1.055, 2.4); };
        return 0.2126*ch(c[0]) + 0.7152*ch(c[1]) + 0.0722*ch(c[2]);
      };
      const ratio = (a, b) => { const la = lum(a), lb = lum(b); const hi = Math.max(la,lb), lo = Math.min(la,lb); return (hi+0.05)/(lo+0.05); };
      const over = (under, on, alpha) => under.map((u, i) => Math.round(on[i]*alpha + u*(1-alpha)));

      const scrim = parse(cs.getPropertyValue('--base'));
      const alpha = parseFloat(cs.getPropertyValue('--qfx-scrim-alpha'));
      const inks = {
        ink: parse(cs.getPropertyValue('--ink')),
        body: parse(cs.getPropertyValue('--ink-body')),
        muted: parse(cs.getPropertyValue('--ink-muted')),
        faint: parse(cs.getPropertyValue('--ink-faint')),
      };
      const grounds = Object.fromEntries(
        ['--void', '--base', '--surface', '--raised', '--well'].map((t) => [t, parse(cs.getPropertyValue(t))]),
      );
      if (!scrim || !Number.isFinite(alpha)) return { error: 'could not read the scrim' };

      // The adversaries: the extremes of the colour cube, and the ink itself,
      // which is the exact worst case. Luminance is monotonic per channel and
      // the composite is linear in the canvas colour, so the minimum over the
      // whole cube is attained at one of these.
      const out = { theme: root.getAttribute('data-theme'), alpha, scrim, worst: {}, baseline: {}, opaque: {} };
      for (const [name, ink] of Object.entries(inks)) {
        if (!ink) { out.worst[name] = null; out.baseline[name] = null; out.opaque[name] = null; continue; }
        // What the theme achieves on its own, with no canvas behind anything.
        out.baseline[name] = ratio(ink, scrim);
        const adversaries = [[0,0,0], [255,255,255], ink];
        out.worst[name] = Math.min(...adversaries.map(c => ratio(ink, over(c, scrim, alpha))));
        // The weakest opaque background this ink can sit on, and which it is.
        let weakest = null;
        for (const [token, ground] of Object.entries(grounds)) {
          if (!ground) continue;
          const r = ratio(ink, ground);
          if (!weakest || r < weakest.ratio) weakest = { token, ratio: r };
        }
        out.opaque[name] = weakest;
      }
      return out;
    })`;

  // Load a shipped theme the way the app does: its id where the app stores it,
  // then a reload, so styles/themes.ts applies the real tokens.
  const loadTheme = async (id) => {
    await evaluate(`localStorage.setItem('qor.theme', ${JSON.stringify(id)})`);
    events.length = 0;
    await send('Page.reload', { ignoreCache: true });
    for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
    await sleep(500);
    return evaluate(`document.documentElement.getAttribute('data-theme')`);
  };

  check('the theme table was read, and it is not empty', THEME_IDS.length > 0, THEME_IDS.join(', '));

  const CASES = [
    ...THEME_IDS.map((id) => ({ name: `the ${id} theme`, id, apply: null })),
    { name: 'a deliberately loud theme', id: THEME_IDS[0], apply: LOUD, loud: true },
  ];

  for (const kase of CASES) {
    const applied = await loadTheme(kase.id);
    if (!kase.loud && applied !== kase.id) {
      check(`${kase.name}: the app applied it`, false, `data-theme is ${applied}`);
      continue;
    }
    const r = await evaluate(`${MATHS}(${JSON.stringify(kase.apply)})`);
    if (r?.error) { check(`${kase.name}: the scrim is readable`, false, r.error); continue; }

    // With no canvas at all: every ink step on every opaque ground. Not asked of
    // the loud theme, which is unreadable by construction.
    if (!kase.loud) {
      for (const [name, weakest] of Object.entries(r.opaque)) {
        if (!weakest) { check(`${kase.name}: ${name} is readable on every background`, false, 'token missing'); continue; }
        check(
          `${kase.name}: ${name} text reads at the floor on every opaque background`,
          weakest.ratio >= CONTRAST_FLOOR,
          `weakest ${weakest.ratio.toFixed(2)}:1 on ${weakest.token}, floor ${CONTRAST_FLOOR}:1`,
        );
      }
    }

    check(
      `${kase.name}: the scrim is opaque enough to be a guarantee`,
      r.alpha >= 0.5,
      `alpha ${r.alpha}`,
    );
    for (const [name, worst] of Object.entries(r.worst)) {
      if (worst === null) { check(`${kase.name}: ${name} is readable`, false, 'token missing'); continue; }
      const baseline = r.baseline[name];

      // THE GUARANTEE, stated as the thing that actually matters: a backdrop
      // cannot take a readable theme below the floor.
      //
      // An earlier draft asserted the backdrop cost "almost nothing" — within
      // 10% of the theme's own contrast. That was the wrong invariant and it
      // failed honestly: at alpha 0.86 the canvas can shift the background by
      // 14%, which costs the default theme about 30% of its contrast while
      // still leaving 13.58:1. Losing 30% of 19:1 does not make text hard to
      // read; dropping below 4.5:1 does. The floor is the claim.
      if (baseline >= CONTRAST_FLOOR) {
        check(
          `${kase.name}: ${name} text stays above the floor over ANY backdrop`,
          worst >= CONTRAST_FLOOR,
          `worst case ${worst.toFixed(2)}:1, theme's own ${baseline.toFixed(2)}:1, floor ${CONTRAST_FLOOR}:1`,
        );
      } else {
        // The theme is already unreadable with the canvas off, so there is
        // nothing for QFX to guarantee. Reported rather than skipped silently:
        // a check that quietly passes over the interesting case is the kind
        // this file exists to avoid.
        check(
          `${kase.name}: ${name} — the THEME is below the floor before any backdrop (a design-system problem, not QFX's)`,
          true,
          `theme's own ${baseline.toFixed(2)}:1`,
        );
      }
    }
  }

  // Maximum contrast is a stated need. The backdrop must be gone entirely, not
  // merely dimmer, and the scrim fully opaque.
  const maximum = await evaluate(`(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', 'architect');
    root.setAttribute('data-contrast', 'maximum');
    const canvas = document.createElement('canvas'); canvas.className = 'qfx-canvas';
    const scrim = document.createElement('div'); scrim.className = 'qfx-scrim';
    document.body.append(canvas, scrim);
    const out = {
      canvas: getComputedStyle(canvas).display,
      scrim: parseFloat(getComputedStyle(scrim).opacity),
    };
    canvas.remove(); scrim.remove();
    root.removeAttribute('data-contrast');
    return out;
  })()`);
  check('at maximum contrast the backdrop is removed entirely', maximum?.canvas === 'none', maximum?.canvas);
  check('at maximum contrast the scrim is fully opaque', maximum?.scrim === 1, `${maximum?.scrim}`);

  // Ambience Off removes the scrim, so it must remove the canvas in the same
  // breath: a frame left on screen with nothing over it is outside every number
  // above. This replaces a case that set data-ambience and read it back, which
  // could not fail.
  const off = await evaluate(`(() => {
    const root = document.documentElement;
    root.setAttribute('data-ambience', 'off');
    const canvas = document.createElement('canvas'); canvas.className = 'qfx-canvas';
    const scrim = document.createElement('div'); scrim.className = 'qfx-scrim';
    document.body.append(canvas, scrim);
    const out = { canvas: getComputedStyle(canvas).display, scrim: getComputedStyle(scrim).display };
    canvas.remove(); scrim.remove();
    root.setAttribute('data-ambience', 'live');
    return out;
  })()`);
  check(
    'with ambience off the canvas is hidden whenever the scrim is',
    off?.scrim !== 'none' || off?.canvas === 'none',
    `canvas ${off?.canvas}, scrim ${off?.scrim}`,
  );
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
