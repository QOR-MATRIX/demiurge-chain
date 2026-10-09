// Does QQ, the QOR Engine, do what P3.1 says? (ADR-082, DIRECTION P3.1)
//
// P3.1: a creator opens QQ from the rail, sees a 2D viewport, adds a shape and
// an emitter, watches it play, presses Stop and the editor is back; the scene is
// saved as a `.qq.json` file whose change reads as a meaningful text diff in
// Projects. The host's own tests (`src-tauri/src/qq.rs`) prove that a scene is
// written only into a project's `scenes/` folder and only in QQ's shape. This
// check covers the rest, in a real rendering engine against the built frontend,
// with a stand-in host that records what it is asked to write:
//
//   - the surface opens from the rail and lists the starter scene's entities;
//   - the viewport is drawn by WebGL2, read back as pixels: the orb is the
//     orb's colour, its glow is lit beyond its edge, the background is the
//     scene's, and the block is where the scene puts it;
//   - adding an entity and typing into the inspector change what is saved, kept
//     to the field's step; dragging in the viewport moves the entity it grabbed;
//   - what is saved is canonical: saving twice is the same bytes, and changing
//     one value changes exactly one line;
//   - Play runs particles and they light the canvas; the inspector cannot edit
//     while playing; Stop restores the scene exactly, so the next save is
//     byte-for-byte the one before Play;
//   - with reduced motion, Play holds one still frame (no particles, no change
//     between two looks) until Step;
//   - a scene the host returns that is not QQ's is refused with a reason and
//     changes nothing; a good one replaces the scene;
//   - a scene name that is not one plain word is refused before the host is
//     asked;
//   - every class the surface uses is defined by the stylesheet.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome. WebGL2 comes
// from SwiftShader, Chrome's software renderer, so no GPU is needed. Set
// BROWSER_PATH to use another Chromium-family browser.
//
//   npm run build
//   node scripts/check-qq-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.
//
// Proven to fail, on 8 October 2026, by two faults planted in QQ, one at a time:
//
//   1. PLAY WROTE BACK INTO THE SCENE. The world was made to copy each body's
//      rotation into the editor's scene as it played. "edited back, the view
//      says the scene is as saved" and "the scene after Stop is byte for byte
//      the scene before Play" failed, naming the block's rotation. The first
//      version of this check MISSED it: the save after Stop wrote text the
//      editor had worked out before Play, so the corrupted scene was never
//      serialised. The check now edits a value and edits it back after Stop,
//      so the file is written from the scene the editor actually holds.
//   2. REDUCED MOTION IGNORED. The engine was made to run the loop even when
//      asked to hold. "and nothing moves on the canvas" (2,712 of 68,328
//      samples changed) and "and no particles run" failed.
//
// Learned while writing it: the launcher's intro plays over the whole window for
// its first seconds and takes the pointer, so the viewport is not read or
// pressed until the canvas is what is on top.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { inflateSync } from 'node:zlib';

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

// The project the stand-in host opens, and the scenes it holds. A second scene
// a person might open, and one that is not QQ's at all.
const PROJECT = {
  path: 'X:/fixture/a-game',
  name: 'a-game',
  branch: 'main',
  changes: [],
  history: [],
  branches: [{ name: 'main', head: true }],
  can_commit: true,
};
const SECOND = `{
  "qq": 1,
  "name": "second",
  "size": { "w": 640, "h": 360 },
  "background": "#101820",
  "entities": [
    {
      "id": "e1",
      "name": "Alpha",
      "transform": { "x": -100, "y": 0, "rotation": 0, "scale": 1 },
      "shape": { "kind": "rect", "w": 50, "h": 50, "colour": "#33ff99", "glow": 0.2 }
    },
    {
      "id": "e2",
      "name": "Beta",
      "transform": { "x": 100, "y": 0, "rotation": 0, "scale": 1 },
      "shape": { "kind": "circle", "w": 40, "h": 40, "colour": "#ff3399", "glow": 0.5 }
    }
  ]
}
`;
const FILES = { second: SECOND, broken: '{ "qq": 2, "name": "broken" }' };

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

// A PNG, decoded with nothing but Node's own zlib (as check-readability.mjs does).
function decodePng(buffer) {
  let pos = 8;
  let width = 0, height = 0, channels = 0;
  const idat = [];
  while (pos < buffer.length) {
    const length = buffer.readUInt32BE(pos);
    const type = buffer.toString('latin1', pos + 4, pos + 8);
    const data = buffer.subarray(pos + 8, pos + 8 + length);
    if (type === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      if (data[8] !== 8 || data[12] !== 0) throw new Error('unexpected PNG: not 8-bit, or interlaced');
      channels = data[9] === 6 ? 4 : data[9] === 2 ? 3 : 0;
      if (!channels) throw new Error(`unexpected PNG colour type ${data[9]}`);
    } else if (type === 'IDAT') idat.push(data);
    else if (type === 'IEND') break;
    pos += 12 + length;
  }
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const out = Buffer.alloc(height * stride);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const src = y * (stride + 1) + 1;
    const dst = y * stride;
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? out[dst + x - channels] : 0;
      const b = y > 0 ? out[dst - stride + x] : 0;
      const c = x >= channels && y > 0 ? out[dst - stride + x - channels] : 0;
      let v = raw[src + x];
      if (filter === 1) v += a;
      else if (filter === 2) v += b;
      else if (filter === 3) v += (a + b) >> 1;
      else if (filter === 4) {
        const p = a + b - c, pa = Math.abs(p - a), pb = Math.abs(p - b), pc = Math.abs(p - c);
        v += pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      }
      out[dst + x] = v & 255;
    }
  }
  return { width, height, channels, pixels: out };
}

// --- drive a real rendering engine ----------------------------------------

const profile = await mkdtemp(join(tmpdir(), 'qor-qq-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    // WebGL2 without a GPU: SwiftShader, Chrome's software renderer.
    '--use-angle=swiftshader',
    '--enable-unsafe-swiftshader',
    '--no-first-run',
    '--no-default-browser-check',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    'about:blank',
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
    if (i > 0) await sleep(100);
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      target = list.find((t) => t.type === 'page');
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
    if (r.result.exceptionDetails) throw new Error(`${expression}\n${JSON.stringify(r.result.exceptionDetails)}`);
    return r.result.result.value;
  };
  const until = async (expression, ms = 4000) => {
    const end = Date.now() + ms;
    while (Date.now() < end) {
      if (await evaluate(expression)) return true;
      await sleep(50);
    }
    return false;
  };

  const ACCOUNTS = [{ address: 'qor1fixture', path: "m/44'/0'/0'/0/0", index: 0, label: 'Fixture' }];
  const SESSION = { qor_id: 'fixture-qor-id', username: 'fixture', role: 'user', address: 'qor1fixture', avatar_url: null };
  const STATE = {
    version: '0.0.0-fixture',
    vault: { state: 'unlocked', accounts: ACCOUNTS, locks_in: 900 },
    session: SESSION,
    chain_endpoint: 'http://127.0.0.1:0',
    auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };
  const HOST_STUB = `
    window.__PROJECT__ = ${JSON.stringify(PROJECT)};
    window.__FILES__ = ${JSON.stringify(FILES)};
    window.__SAVES__ = [];
    window.__STUDIO_OPENS__ = [];
    window.__LAUNCHER_STATE__ = ${JSON.stringify(STATE)};
    window.__ACCOUNTS__ = ${JSON.stringify(ACCOUNTS)};
    window.__SESSION__ = ${JSON.stringify(SESSION)};
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd === 'qontrol_pick_folder') return Promise.resolve(window.__PROJECT__.path);
        if (cmd === 'qontrol_open' || cmd === 'qontrol_read') return Promise.resolve(window.__PROJECT__);
        if (cmd === 'qq_save_scene') {
          window.__SAVES__.push(args);
          window.__FILES__[args.name] = args.scene;
          return Promise.resolve({ ...window.__PROJECT__, changes: [{ path: 'scenes', state: 'untracked' }] });
        }
        if (cmd === 'qq_list_scenes') return Promise.resolve(Object.keys(window.__FILES__).sort());
        if (cmd === 'qq_open_studio') { window.__STUDIO_OPENS__.push(args); return Promise.resolve(null); }
        if (cmd === 'qq_load_scene') {
          const f = window.__FILES__[args.name];
          return f === undefined
            ? Promise.reject({ kind: 'qontrol', message: 'there is no scene called ' + args.name })
            : Promise.resolve(f);
        }
        if (cmd === 'launcher_state') return Promise.resolve(window.__LAUNCHER_STATE__);
        if (cmd === 'vault_status') return Promise.resolve({ state: 'unlocked', accounts: window.__ACCOUNTS__, locks_in: 900 });
        if (cmd === 'qor_restore') return Promise.resolve(window.__SESSION__);
        if (cmd === 'touch_vault' || cmd === 'vault_lock') return Promise.resolve(null);
        return Promise.reject({ kind: 'internal', message: 'no host in this check' });
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
  await send('Emulation.setDeviceMetricsOverride', { width: 1440, height: 900, deviceScaleFactor: 1, mobile: false });
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await send('Page.bringToFront');
  await send('Emulation.setFocusEmulationEnabled', { enabled: true });
  await sleep(800);

  const clickText = (label, scope = 'main') =>
    evaluate(`(() => {
      const b = [...document.querySelectorAll('${scope} button')].find((n) => n.textContent.trim() === ${JSON.stringify(label)});
      if (!b) throw new Error('no button ' + ${JSON.stringify(label)});
      if (b.disabled) return false;
      b.click();
      return true;
    })()`);
  const type = (selector, value) =>
    evaluate(`(() => {
      const input = document.querySelector(${JSON.stringify(selector)});
      if (!input) throw new Error('nothing matches ' + ${JSON.stringify(selector)});
      const proto = input instanceof HTMLSelectElement ? HTMLSelectElement.prototype : HTMLInputElement.prototype;
      Object.getOwnPropertyDescriptor(proto, 'value').set.call(input, ${JSON.stringify(value)});
      input.dispatchEvent(new Event(input instanceof HTMLSelectElement ? 'change' : 'input', { bubbles: true }));
      return true;
    })()`);
  const entityNames = () =>
    evaluate(`[...document.querySelectorAll('main ul[aria-label="Entities"] button')].map((b) => b.querySelector('span.truncate').textContent)`);
  const saves = () => evaluate(`window.__SAVES__`);
  const noticeText = () => evaluate(`[...document.querySelectorAll('.glass-solid')].map((n) => n.textContent).join(' | ')`);

  // Where the scene sits on screen, in CSS pixels: fitted into the canvas, centred.
  const canvasBox = () =>
    evaluate(`(() => { const r = document.querySelector('main [data-qq-canvas]').getBoundingClientRect(); return { x: r.left, y: r.top, w: r.width, h: r.height }; })()`);
  const sceneToScreen = (box, size, x, y) => {
    const scale = Math.min(box.w / size.w, box.h / size.h);
    const w = Math.floor(size.w * scale), h = Math.floor(size.h * scale);
    const left = box.x + Math.floor((box.w - w) / 2), top = box.y + Math.floor((box.h - h) / 2);
    const unitsPerPixel = size.w / w;
    return { sx: left + (x + size.w / 2) / unitsPerPixel, sy: top + (y + size.h / 2) / unitsPerPixel, unitsPerPixel, left, top, w, h };
  };
  const shot = async () => {
    const r = await send('Page.captureScreenshot', { format: 'png' });
    return decodePng(Buffer.from(r.result.data, 'base64'));
  };
  const pixel = (img, sx, sy) => {
    const o = (Math.round(sy) * img.width + Math.round(sx)) * img.channels;
    return [img.pixels[o], img.pixels[o + 1], img.pixels[o + 2]];
  };
  const luma = ([r, g, b]) => 0.2126 * r + 0.7152 * g + 0.0722 * b;

  // ── opening ───────────────────────────────────────────────────────────────
  const opened = await evaluate(`(() => {
    const b = [...document.querySelectorAll('nav[aria-label="Primary"] button')].find((n) => n.textContent.trim() === 'QQ');
    if (!b) return false;
    b.click();
    return true;
  })()`);
  check('the rail has a QQ button and it opens the surface', opened === true);
  await until(`Boolean(document.querySelector('main [data-qq-view]'))`);
  const heading = await evaluate(`document.querySelector('main h1')?.textContent ?? ''`);
  check('the surface is QQ', heading === 'QQ', heading);
  check('the starter scene lists its three entities', JSON.stringify(await entityNames()) === JSON.stringify(['Core', 'Ring', 'Ground']), JSON.stringify(await entityNames()));
  const drawing = await evaluate(`!document.body.innerText.includes('cannot draw WebGL2')`);
  check('the viewport is drawn with WebGL2', drawing === true);
  // The launcher's intro plays over everything for its first seconds; nothing
  // here is looked at or pressed until the canvas is what is on top.
  const onTop = await until(`(() => {
    const c = document.querySelector('main [data-qq-canvas]');
    const r = c.getBoundingClientRect();
    return document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2) === c;
  })()`, 15000);
  check('the viewport is on top once the intro is over', onTop === true);
  await sleep(300);

  // ── the viewport, as painted ──────────────────────────────────────────────
  const STARTER = { w: 960, h: 540 };
  let box = await canvasBox();
  {
    const img = await shot();
    const core = sceneToScreen(box, STARTER, 0, 0);
    const [r, g, b] = pixel(img, core.sx, core.sy);
    check('the orb is drawn in its colour at its place', r > 220 && g > 70 && g < 140 && b < 60, `${r},${g},${b}`);
    // The orb's radius is 42 units; 56 units out is beyond its edge, inside its glow.
    const halo = pixel(img, sceneToScreen(box, STARTER, 56, 0).sx, core.sy);
    const bg = pixel(img, core.left + 6, core.top + 6);
    check('the background is the scene\'s colour', Math.abs(bg[0] - 7) <= 3 && Math.abs(bg[1] - 8) <= 3 && Math.abs(bg[2] - 12) <= 3, bg.join(','));
    check('the glow lights the canvas beyond the orb\'s edge', luma(halo) > luma(bg) + 20 && luma(halo) < luma([r, g, b]), `halo ${halo.join(',')}`);
    const ring = sceneToScreen(box, STARTER, -260, 0);
    const blue = pixel(img, ring.sx, ring.sy);
    check('the block is drawn where the scene puts it', blue[2] > 200 && blue[0] < 150, blue.join(','));
  }

  // ── editing ──────────────────────────────────────────────────────────────
  await clickText('Orb');
  await sleep(200);
  const afterAdd = await entityNames();
  check('Add puts an Orb in the scene', afterAdd.length === 4 && afterAdd[3] === 'Orb', JSON.stringify(afterAdd));
  check(
    'and selects it, so the inspector shows its components',
    (await evaluate(`[...document.querySelectorAll('main ul[aria-label="Entities"] button')][3].getAttribute('aria-pressed')`)) === 'true' &&
      (await evaluate(`Boolean(document.querySelector('main fieldset [data-qq-field="x"]'))`)),
  );
  await type('main fieldset [data-qq-field="x"]', '123.45');
  await sleep(150);

  // Drag the Core: grabbed at its centre, moved 60 px right and 30 px down.
  const from = sceneToScreen(box, STARTER, 0, 0);
  await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: from.sx, y: from.sy });
  await send('Input.dispatchMouseEvent', { type: 'mousePressed', x: from.sx, y: from.sy, button: 'left', buttons: 1, clickCount: 1 });
  for (let i = 1; i <= 6; i++) {
    await send('Input.dispatchMouseEvent', { type: 'mouseMoved', x: from.sx + 10 * i, y: from.sy + 5 * i, button: 'left', buttons: 1 });
    await sleep(16);
  }
  await send('Input.dispatchMouseEvent', { type: 'mouseReleased', x: from.sx + 60, y: from.sy + 30, button: 'left', buttons: 0, clickCount: 1 });
  await sleep(200);
  const pressed = await evaluate(`[...document.querySelectorAll('main ul[aria-label="Entities"] button[aria-pressed="true"]')].map((b) => b.querySelector('span.truncate').textContent)`);
  check('pressing on the orb in the viewport selects it, under the Orb moved away', JSON.stringify(pressed) === JSON.stringify(['Core']), JSON.stringify(pressed));
  const movedX = Number(await evaluate(`document.querySelector('main fieldset [data-qq-field="x"]').value`));
  const movedY = Number(await evaluate(`document.querySelector('main fieldset [data-qq-field="y"]').value`));
  const wantX = 60 * from.unitsPerPixel, wantY = 30 * from.unitsPerPixel;
  check('dragging moves it by the distance dragged, in scene units', Math.abs(movedX - wantX) < 1.2 && Math.abs(movedY - wantY) < 1.2, `${movedX},${movedY} vs ${wantX.toFixed(1)},${wantY.toFixed(1)}`);

  // ── saving ───────────────────────────────────────────────────────────────
  await clickText('Save');
  await until(`window.__SAVES__.length === 1`);
  let saved = await saves();
  check('Save asks for a project, then asks the host to write the scene into it', saved.length === 1 && saved[0].path === PROJECT.path && saved[0].name === 'first-light', JSON.stringify(saved.map((s) => [s.path, s.name])));
  const first = saved[0]?.scene ?? '';
  let parsed = null;
  try { parsed = JSON.parse(first); } catch {}
  check('what is written is JSON of format 1', parsed?.qq === 1 && parsed?.name === 'first-light');
  const orb = parsed?.entities?.find((e) => e.name === 'Orb');
  const core = parsed?.entities?.find((e) => e.name === 'Core');
  check('the typed value is saved, kept to the field\'s step', orb?.transform?.x === 123.5, String(orb?.transform?.x));
  check('the dragged position is saved', Math.abs((core?.transform?.x ?? 1e9) - movedX) < 0.05 && Math.abs((core?.transform?.y ?? 1e9) - movedY) < 0.05);
  check(
    'each component is one line, so a change is one line',
    first.endsWith('}\n') && /\n      "transform": \{ "x": [-\d.]+, "y": [-\d.]+, "rotation": [-\d.]+, "scale": [-\d.]+ \},?\n/.test(first),
  );
  check('the view says Saved and lists the scene', (await evaluate(`[...document.querySelectorAll('main button')].some((b) => b.textContent.trim() === 'Saved')`)) && (await evaluate(`[...document.querySelectorAll('main ul[aria-label="Scenes in this project"] button')].map((b) => b.textContent).includes('first-light')`)));

  await clickText('Saved');
  await until(`window.__SAVES__.length === 2`);
  saved = await saves();
  check('saving the same scene again writes the same bytes', saved[1]?.scene === first);

  // One value changed: one line of the file changed.
  await evaluate(`[...document.querySelectorAll('main ul[aria-label="Entities"] button')][3].click()`);
  await sleep(150);
  await type('main fieldset [data-qq-field="glow"]', '0.25');
  await sleep(150);
  check('an edit after saving is shown as unsaved', (await evaluate(`document.body.innerText.includes('Unsaved changes.')`)) === true);
  await clickText('Save');
  await until(`window.__SAVES__.length === 3`);
  saved = await saves();
  const a = first.split('\n'), b = (saved[2]?.scene ?? '').split('\n');
  const differing = a.length === b.length ? a.filter((line, i) => line !== b[i]) : null;
  check('changing one value changes exactly one line of the file', differing?.length === 1 && b.find((l, i) => l !== a[i])?.includes('"glow": 0.25'), JSON.stringify(differing));
  const beforePlay = saved[2]?.scene;

  // ── playing ──────────────────────────────────────────────────────────────
  const quiet = await shot();
  await clickText('Play', '[data-qq-view] header');
  await sleep(1200);
  const particles = Number(await evaluate(`document.querySelector('main [data-qq-particles]')?.dataset.qqParticles ?? '0'`));
  check('Play runs the emitter: particles are live', particles > 20, String(particles));
  check('while playing, the inspector cannot edit', (await evaluate(`document.querySelector('main fieldset')?.disabled === true`)) === true);
  const lit = await shot();
  // Sparks are small and spread every way: count the pixels around the orb that
  // they lit, rather than averaging them into the orb's own glow.
  let litPixels = 0;
  const centre = sceneToScreen(box, STARTER, movedX, movedY);
  for (let dx = -150; dx <= 150; dx += 2) for (let dy = -150; dy <= 150; dy += 2) {
    if (luma(pixel(lit, centre.sx + dx, centre.sy + dy)) > luma(pixel(quiet, centre.sx + dx, centre.sy + dy)) + 25) litPixels += 1;
  }
  check('the particles light the canvas', litPixels > 25, `${litPixels} sampled pixels lit`);
  await clickText('Stop', '[data-qq-view] header');
  await sleep(200);
  check('Stop brings the editor back', (await evaluate(`Boolean([...document.querySelectorAll('[data-qq-view] header button')].find((b) => b.textContent.trim() === 'Play'))`)) && (await evaluate(`document.querySelector('main fieldset')?.disabled === false`)));
  check('and nothing the game did while playing is unsaved', (await evaluate(`document.body.innerText.includes('Unsaved changes.')`)) === false);
  // Edit and edit back, so the file is written from the scene the editor holds
  // now, not from text it worked out before Play.
  await type('main fieldset [data-qq-field="glow"]', '0.3');
  await sleep(100);
  await type('main fieldset [data-qq-field="glow"]', '0.25');
  await sleep(100);
  check('edited back, the view says the scene is as saved', (await evaluate(`[...document.querySelectorAll('main button')].some((b) => b.textContent.trim() === 'Saved')`)) === true);
  await evaluate(`[...document.querySelectorAll('main button')].find((b) => /^Saved?$/.test(b.textContent.trim())).click()`);
  await until(`window.__SAVES__.length === 4`);
  saved = await saves();
  const wasLines = (beforePlay ?? '').split('\n');
  check(
    'the scene after Stop is byte for byte the scene before Play',
    saved[3]?.scene === beforePlay,
    (saved[3]?.scene ?? '').split('\n').filter((l, i) => l !== wasLines[i]).join(' / '),
  );

  // ── reduced motion ───────────────────────────────────────────────────────
  await send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-motion', value: 'reduce' }] });
  await sleep(200);
  await clickText('Play', '[data-qq-view] header');
  await sleep(150);
  check('with reduced motion, Play says it is held still', (await evaluate(`document.body.innerText.includes('Held still: reduced motion is on')`)) === true);
  const held1 = await shot();
  await sleep(600);
  const held2 = await shot();
  // QQ's canvas only: the launcher's own backdrop behind the chrome is not QQ's to hold.
  let moved = 0, looked = 0;
  for (let y = Math.ceil(box.y) + 1; y < box.y + box.h - 1; y += 3) for (let x = Math.ceil(box.x) + 1; x < box.x + box.w - 1; x += 3) {
    looked += 1;
    if (pixel(held1, x, y).join() !== pixel(held2, x, y).join()) moved += 1;
  }
  check('and nothing moves on the canvas', moved === 0 && looked > 1000, `${moved} of ${looked} samples changed`);
  check('and no particles run', Number(await evaluate(`document.querySelector('main [data-qq-particles]')?.dataset.qqParticles ?? '-1'`)) === 0);
  await clickText('Step', '[data-qq-view] header');
  await sleep(150);
  check('Step advances it, at the person\'s pace', Number(await evaluate(`document.querySelector('main [data-qq-particles]')?.dataset.qqParticles ?? '0'`)) > 0);
  await clickText('Stop', '[data-qq-view] header');
  await send('Emulation.setEmulatedMedia', { features: [] });
  await sleep(150);

  // ── opening scenes ───────────────────────────────────────────────────────
  const listed = await evaluate(`[...document.querySelectorAll('main ul[aria-label="Scenes in this project"] button')].map((b) => b.textContent)`);
  check('the project\'s scenes are listed from the host', JSON.stringify(listed) === JSON.stringify(['broken', 'first-light', 'second']), JSON.stringify(listed));
  const countBefore = (await entityNames()).length;
  await evaluate(`[...document.querySelectorAll('main ul[aria-label="Scenes in this project"] button')].find((b) => b.textContent === 'broken').click()`);
  await sleep(300);
  check('a file that is not a QQ scene is refused with a reason', (await noticeText()).includes('format 1'), await noticeText());
  check('and the scene is unchanged', (await entityNames()).length === countBefore);
  await evaluate(`[...document.querySelectorAll('main ul[aria-label="Scenes in this project"] button')].find((b) => b.textContent === 'second').click()`);
  await until(`[...document.querySelectorAll('main ul[aria-label="Entities"] button')].length === 2`);
  check('a QQ scene from the project replaces the scene', JSON.stringify(await entityNames()) === JSON.stringify(['Alpha', 'Beta']), JSON.stringify(await entityNames()));
  check('and the name follows it, with nothing unsaved', (await evaluate(`document.querySelector('main input[aria-describedby="qq-name-hint"]').value`)) === 'second' && (await evaluate(`document.body.innerText.includes('Unsaved changes.')`)) === false);
  box = await canvasBox();
  {
    const img = await shot();
    const beta = sceneToScreen(box, { w: 640, h: 360 }, 100, 0);
    const [r, g, b] = pixel(img, beta.sx, beta.sy);
    check('and it is drawn: the second scene\'s circle at its place', r > 220 && g < 90 && b > 120, `${r},${g},${b}`);
  }

  // ── names ────────────────────────────────────────────────────────────────
  await type('main input[aria-describedby="qq-name-hint"]', 'Bad Name');
  await sleep(100);
  const before = (await saves()).length;
  await clickText('Save');
  await sleep(300);
  check('a scene name that is not one plain word is refused before the host is asked', (await saves()).length === before && (await noticeText()).includes('lowercase letters'), await noticeText());

  // ── QQ Studio ────────────────────────────────────────────────────────────
  // The 3D engine (ADR-083) is a process of its own: the button asks the host to start it, on the project that is open.
  await evaluate(`document.querySelector('main [data-qq-open-studio]').click()`);
  await until(`window.__STUDIO_OPENS__.length === 1`);
  const opens = await evaluate(`window.__STUDIO_OPENS__`);
  check('Open in QQ Studio asks the host to start it on the open project, and nothing more',
    opens.length === 1 && JSON.stringify(opens[0]) === JSON.stringify({ path: PROJECT.path }), JSON.stringify(opens));

  // ── the stylesheet ───────────────────────────────────────────────────────
  await evaluate(`[...document.querySelectorAll('main ul[aria-label="Entities"] button')][0].click()`);
  await sleep(150);
  const undefinedClasses = await evaluate(`(() => {
    const selectors = [];
    const walk = (list) => { for (const rule of list) { if (rule.selectorText) selectors.push(rule.selectorText); if (rule.cssRules) walk(rule.cssRules); } };
    for (const sheet of document.styleSheets) { try { walk(sheet.cssRules); } catch {} }
    const all = selectors.join(' ');
    const used = new Set();
    for (const el of document.querySelectorAll('main [class]')) for (const c of el.classList) used.add(c);
    return [...used].filter((c) => !c.startsWith('lucide')).filter((c) => !all.includes('.' + CSS.escape(c))).sort();
  })()`);
  check('every class the surface uses is defined by the stylesheet', undefinedClasses.length === 0, undefinedClasses.join(' '));
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
