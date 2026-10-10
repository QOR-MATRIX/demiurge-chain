// The QQ Player in a browser, measured (DIRECTION P3.2): how much a person downloads, how long the game takes to show
// its first frame proper, and the frame rate after it; and that the first frame is the game, not an empty page.
//
//   node products/qq/player/measure-web.mjs [build/player] [--frame <first.png>] [--software]
//
// `build/player` is the Qt for WebAssembly build's player folder (default %LOCALAPPDATA%\qq-wasm\player, where
// build.ps1 -Web puts it). The folder is served over HTTP on this machine, cross-origin isolated as a threaded
// WebAssembly page must be, and opened in a headless Chromium-family browser (BROWSER_PATH to choose one), drawing on
// the machine's GPU, or on SwiftShader, Chrome's software renderer, with --software.
//
// Three seconds after its first frame proper the Player prints "QQ-MEASURE {json}" to the console, with times from
// when the page began to load: what a person waits through. The console of every worker (Qt's threads) is read too,
// since a thread's error is reported only there. One line of JSON is printed: the sizes (as served, and gzip and
// brotli compressed, which is what a web server would send), the Player's measurement, the renderer WebGL reported,
// and how many colours the first frame has. Exit code 0 measured, 1 not (the reason is printed).

import { spawn } from 'node:child_process';
import { existsSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { mkdtemp, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { extname, join, resolve } from 'node:path';
import { brotliCompressSync, constants, gzipSync, inflateSync } from 'node:zlib';

const args = process.argv.slice(2);
const option = (name) => {
  const i = args.indexOf(name);
  return i >= 0 ? args.splice(i, 2)[1] : undefined;
};
const frameFile = option('--frame');
const software = args.includes('--software');
const positional = args.filter((a) => !a.startsWith('--'));
const folder = resolve(positional[0] ?? join(process.env.LOCALAPPDATA ?? '', 'qq-wasm', 'player'));
if (!existsSync(join(folder, 'qq-player.html'))) {
  console.error(`No QQ Player web build at ${folder}. Run \`pwsh products/qq/build.ps1 -Web\` first.`);
  process.exit(1);
}

const BROWSER =
  process.env.BROWSER_PATH ??
  [
    'C:/Program Files/Google/Chrome/Application/chrome.exe',
    'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    'C:/Program Files/Microsoft/Edge/Application/msedge.exe',
    '/usr/bin/google-chrome',
    '/usr/bin/chromium',
  ].find((p) => existsSync(p));
if (!BROWSER) {
  console.error('No Chromium-family browser found. Set BROWSER_PATH.');
  process.exit(1);
}

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const fail = (why) => {
  console.error(`NOT MEASURED: ${why}`);
  process.exitCode = 1;
};

// --- what a person downloads ------------------------------------------------

const served = ['qq-player.html', 'qtloader.js', 'qq-player.js', 'qq-player.wasm'];
const sizes = {};
for (const name of served) {
  const bytes = readFileSync(join(folder, name));
  sizes[name] = {
    bytes: bytes.length,
    gzip: gzipSync(bytes, { level: 9 }).length,
    brotli: brotliCompressSync(bytes, { params: { [constants.BROTLI_PARAM_QUALITY]: 11 } }).length,
  };
}
const total = (k) => Object.values(sizes).reduce((sum, s) => sum + s[k], 0);

// --- serve the build ------------------------------------------------------------

const TYPES = {
  '.html': 'text/html; charset=utf-8',
  '.js': 'text/javascript; charset=utf-8',
  '.wasm': 'application/wasm',
  '.svg': 'image/svg+xml',
};
const server = createServer((req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname).replace(/^\/+/, '');
  const file = resolve(folder, path || 'qq-player.html');
  if (!file.startsWith(folder) || !existsSync(file) || !statSync(file).isFile()) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, {
    'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream',
    'Cache-Control': 'no-store',
    // The Player is built with threads (Qt Multimedia, and so spatial audio, come only in that kit), and a page gets the
    // shared memory threads need only when it is cross-origin isolated. Whoever serves the Player sends these two.
    'Cross-Origin-Opener-Policy': 'same-origin',
    'Cross-Origin-Embedder-Policy': 'require-corp',
  });
  res.end(readFileSync(file));
});
await new Promise((ok) => server.listen(0, '127.0.0.1', ok));
const origin = `http://127.0.0.1:${server.address().port}`;

// --- a PNG's colours, to tell a world from an empty page ---------------------------

function colours(png) {
  let pos = 8;
  let width = 0;
  let height = 0;
  let type = 0;
  const idat = [];
  while (pos < png.length) {
    const length = png.readUInt32BE(pos);
    const kind = png.toString('ascii', pos + 4, pos + 8);
    const data = png.subarray(pos + 8, pos + 8 + length);
    if (kind === 'IHDR') {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      if (data[8] !== 8 || data[12] !== 0) throw new Error('only 8-bit, non-interlaced PNGs are read');
      type = data[9];
    } else if (kind === 'IDAT') {
      idat.push(data);
    }
    pos += 12 + length;
  }
  const channels = { 2: 3, 6: 4 }[type];
  if (!channels) throw new Error(`PNG colour type ${type} is not read`);
  const raw = inflateSync(Buffer.concat(idat));
  const stride = width * channels;
  const pixels = Buffer.alloc(stride * height);
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)];
    const line = raw.subarray(y * (stride + 1) + 1, (y + 1) * (stride + 1));
    for (let x = 0; x < stride; x++) {
      const a = x >= channels ? pixels[y * stride + x - channels] : 0;
      const b = y > 0 ? pixels[(y - 1) * stride + x] : 0;
      const c = x >= channels && y > 0 ? pixels[(y - 1) * stride + x - channels] : 0;
      const p = a + b - c;
      const pa = Math.abs(p - a);
      const pb = Math.abs(p - b);
      const pc = Math.abs(p - c);
      const paeth = pa <= pb && pa <= pc ? a : pb <= pc ? b : c;
      const v = line[x] + [0, a, b, (a + b) >> 1, paeth][filter];
      pixels[y * stride + x] = v & 255;
    }
  }
  // As tst_player counts them: every fourth pixel, to four bits a channel.
  const seen = new Set();
  for (let y = 0; y < height; y += 4) {
    for (let x = 0; x < width; x += 4) {
      const i = y * stride + x * channels;
      seen.add(((pixels[i] & 0xf0) << 16) | ((pixels[i + 1] & 0xf0) << 8) | (pixels[i + 2] & 0xf0));
    }
  }
  return seen.size;
}

// --- open it in a browser -------------------------------------------------------------

const profile = await mkdtemp(join(tmpdir(), 'qq-web-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    // On a computer with two GPUs, the faster, as the native Player chooses (runtime/gpu.h).
    ...(software
      ? ['--use-angle=swiftshader', '--enable-unsafe-swiftshader']
      : ['--enable-gpu', '--ignore-gpu-blocklist', '--force_high_performance_gpu']),
    '--window-size=1280,720',
    '--no-first-run',
    '--no-default-browser-check',
    `--remote-debugging-port=${port}`,
    `--user-data-dir=${profile}`,
    'about:blank',
  ],
  { stdio: 'ignore' },
);

try {
  let target;
  for (let i = 0; i < 300 && !target; i++) {
    await sleep(100);
    try {
      const list = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      target = list.find((t) => t.type === 'page');
    } catch {}
  }
  if (!target) throw new Error('the browser did not expose a page to inspect');

  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((ok, bad) => {
    ws.onopen = ok;
    ws.onerror = bad;
  });
  let nextId = 0;
  const pending = new Map();
  let measured = null;
  const problems = [];
  const consoleLines = [];
  // Each of Qt's threads is a Web Worker, whose errors reach only its own console: every worker is attached to
  // (flattened, each with its own session) and its console read like the page's.
  const send = (method, params = {}, sessionId = undefined) =>
    new Promise((ok) => {
      const id = ++nextId;
      pending.set(id, ok);
      ws.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
    });
  const evaluate = async (expression, sessionId) =>
    (await send('Runtime.evaluate', { expression, returnByValue: true }, sessionId)).result?.result?.value;
  const watch = async (sessionId) => {
    await send('Target.setAutoAttach', { autoAttach: true, waitForDebuggerOnStart: false, flatten: true }, sessionId);
    await send('Runtime.enable', {}, sessionId);
  };
  ws.onmessage = (message) => {
    const msg = JSON.parse(message.data);
    if (msg.id && pending.has(msg.id)) {
      pending.get(msg.id)(msg);
      pending.delete(msg.id);
    } else if (msg.method === 'Target.attachedToTarget') {
      watch(msg.params.sessionId).catch(() => {});
    } else if (msg.method === 'Runtime.consoleAPICalled') {
      const text = msg.params.args.map((a) => a.value ?? a.description ?? '').join(' ');
      consoleLines.push(text);
      const at = text.indexOf('QQ-MEASURE ');
      if (at >= 0) {
        measured = JSON.parse(text.slice(at + 'QQ-MEASURE '.length));
      } else if (msg.params.type === 'error' || /could not|error/i.test(text)) {
        problems.push(text);
      }
    } else if (msg.method === 'Runtime.exceptionThrown') {
      problems.push(msg.params.exceptionDetails.exception?.description ?? msg.params.exceptionDetails.text);
    }
  };

  await send('Page.enable');
  await send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 720, deviceScaleFactor: 1, mobile: false });
  await send('Target.setAutoAttach', { autoAttach: true, waitForDebuggerOnStart: false, flatten: true });
  await send('Page.navigate', { url: `${origin}/qq-player.html` });
  await watch(undefined);

  const renderer = await (async () => {
    for (let i = 0; i < 50; i++) {
      const r = await evaluate(`(() => {
        const gl = document.createElement('canvas').getContext('webgl2');
        if (!gl) return 'no WebGL2';
        const info = gl.getExtension('WEBGL_debug_renderer_info');
        return info ? gl.getParameter(info.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER);
      })()`);
      if (r) return r;
      await sleep(100);
    }
    return 'unknown';
  })();

  // The Player reports three seconds after its first frame proper; a cold start compiles 40 MB of WebAssembly.
  const end = Date.now() + 120000;
  while (!measured && Date.now() < end) await sleep(250);

  // What the page shows, measured or not: a failure is easier to read with a picture of it.
  const shot = Buffer.from((await send('Page.captureScreenshot', { format: 'png' })).result.data, 'base64');
  if (frameFile) writeFileSync(frameFile, shot);
  const frameColours = colours(shot);

  console.log(
    JSON.stringify({
      bytes: total('bytes'),
      gzip: total('gzip'),
      brotli: total('brotli'),
      files: sizes,
      renderer,
      measured,
      frameColours,
      problems,
    }),
  );
  if (!measured) {
    fail(`no QQ-MEASURE line within two minutes${problems.length ? `: ${problems.join(' | ')}` : ''}`);
    console.error(`The console's last lines:\n${consoleLines.slice(-30).join('\n')}`);
  } else if (measured.problem) fail(`the scene could not play: ${measured.problem}`);
  else if (frameColours <= 200) fail(`the page shows ${frameColours} colours, not a world`);
  ws.close();
} catch (e) {
  fail(e.message);
} finally {
  try {
    browser.kill();
  } catch {}
  server.close();
  await sleep(300);
  await rm(profile, { recursive: true, force: true }).catch(() => {});
}
