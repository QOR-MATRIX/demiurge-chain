// Does the real translation engine translate, with Mozilla's real files, in the launcher's own worker? (ADR-092)
//
// `check-social-view.mjs` drives the worker in its stand-in mode, since CI does not download Mozilla's files. This check
// runs the real thing, by hand: the engine and the Spanish to English models that the host's own code downloaded and
// checked (`real_mozilla_files_download_check_and_plan` in src-tauri/src/translate/tests.rs, which writes `plan.json`
// beside them), handed to `public/translate/worker.js` in a real rendering engine exactly as the interface hands them,
// then a sentence translated directly and another through English.
//
//   QOR_TRANSLATE_DIR=<folder> cargo test --lib real_mozilla -- --ignored     (in src-tauri)
//   npm run build
//   node scripts/check-translate-engine.mjs <folder> [dist directory, default ./dist]
//
// Exits non-zero if any check fails.

import { spawn } from 'node:child_process';
import { existsSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, extname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const files = process.argv[2] ? resolve(process.argv[2]) : null;
const dist = resolve(process.argv[3] ?? join(dirname(fileURLToPath(import.meta.url)), '..', 'dist'));
if (!files || !existsSync(join(files, 'plan.json'))) {
  console.error('No plan.json in the folder given. Run the host test that prepares it first (see the top of this file).');
  process.exit(2);
}
if (!existsSync(join(dist, 'translate', 'worker.js'))) {
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

const plan = JSON.parse(await readFile(join(files, 'plan.json'), 'utf8'));
const TYPES = { '.html': 'text/html', '.js': 'text/javascript' };
const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  try {
    if (path.startsWith('/files/') && /^[0-9a-f]{64}$/.test(path.slice(7))) {
      const body = await readFile(join(files, path.slice(7)));
      res.writeHead(200, { 'content-type': 'application/octet-stream' }).end(body);
    } else if (path === '/') {
      res.writeHead(200, { 'content-type': 'text/html' }).end('<!doctype html><title>engine</title>');
    } else {
      const file = join(dist, path.replace(/^\/+/, ''));
      const body = await readFile(file);
      res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' }).end(body);
    }
  } catch {
    res.writeHead(404).end('not found');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const origin = `http://127.0.0.1:${server.address().port}`;
const userData = await mkdtemp(join(tmpdir(), 'qor-engine-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  ['--headless=new', ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []), '--disable-gpu', '--no-first-run',
    `--remote-debugging-port=${port}`, `--user-data-dir=${userData}`, 'about:blank'],
  { stdio: 'ignore' },
);

try {
  let target;
  for (let i = 0; i < 300 && !target; i++) {
    await sleep(100);
    try {
      target = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find((t) => t.type === 'page');
    } catch {}
  }
  if (!target) throw new Error('The browser did not open the page');
  const ws = new WebSocket(target.webSocketDebuggerUrl);
  await new Promise((ok, bad) => { ws.onopen = ok; ws.onerror = bad; });
  let nextId = 0;
  const pending = new Map();
  ws.onmessage = (m) => { const msg = JSON.parse(m.data); if (msg.id && pending.has(msg.id)) { pending.get(msg.id)(msg); pending.delete(msg.id); } };
  const evaluate = (expression) => new Promise((ok) => {
    const id = ++nextId;
    pending.set(id, (r) => ok(r.result.exceptionDetails ? { error: JSON.stringify(r.result.exceptionDetails).slice(0, 400) } : r.result.result.value));
    ws.send(JSON.stringify({ id, method: 'Runtime.evaluate', params: { expression, returnByValue: true, awaitPromise: true } }));
  });

  // As the interface does it (src/social/translate.ts): the bytes of each file, transferred to the worker by kind.
  const run = (routePlan, sentence) => evaluate(`(async () => {
    const plan = ${JSON.stringify(routePlan)};
    const bytes = async (hash) => (await fetch('/files/' + hash)).arrayBuffer();
    const worker = new Worker('/translate/worker.js');
    const answer = (type) => new Promise((ok) => worker.addEventListener('message', ({ data }) => { if (type.includes(data.type)) ok(data); }));
    const transfer = [];
    const engine = await bytes(plan.engine.hash); transfer.push(engine);
    const pairs = [];
    for (const p of plan.pairs) {
      const files = {};
      for (const f of p.files) { const b = await bytes(f.hash); transfer.push(b); files[f.kind] = { name: f.name, bytes: b }; }
      pairs.push({ from: p.from, to: p.to, files });
    }
    const started = performance.now();
    worker.postMessage({ type: 'init', engine, pairs }, transfer);
    const ready = await answer(['ready', 'error']);
    if (ready.type !== 'ready') return { error: ready.message };
    const loaded = performance.now() - started;
    const t0 = performance.now();
    worker.postMessage({ type: 'translate', id: 1, text: ${JSON.stringify(sentence)} });
    const done = await answer(['done', 'failed']);
    worker.terminate();
    return { ...done, loaded: Math.round(loaded), took: Math.round(performance.now() - t0) };
  })()`);

  // The page is the server's own before a worker is made from a path on it.
  await new Promise((ok) => {
    const id = ++nextId;
    pending.set(id, ok);
    ws.send(JSON.stringify({ id, method: 'Page.navigate', params: { url: `${origin}/` } }));
  });
  for (let i = 0; i < 100 && (await evaluate('location.origin + document.readyState')) !== `${origin}complete`; i++) await sleep(100);
  const direct = await run(plan, '  Hola a todos, ¿qué tal estáis hoy? Me encanta este juego.  ');
  check('the engine starts with Mozilla\'s files, as the worker is handed them', !direct.error && direct.type === 'done', JSON.stringify(direct));
  check('and translates Spanish into English',
    typeof direct.text === 'string' && /hello|hi\b/i.test(direct.text) && /game/i.test(direct.text), JSON.stringify(direct.text));
  check('keeping the whitespace around the words', typeof direct.text === 'string' && direct.text.startsWith('  ') && direct.text.endsWith('  '));
  check('in well under a second once loaded', direct.took < 1000, `${direct.took} ms; loading took ${direct.loaded} ms`);

  // Through English: the same model twice stands in for two (es→en, then en→… would need a second download); what is
  // checked is that the engine accepts two models and runs them one after the other.
  const twice = await run({ ...plan, pairs: [plan.pairs[0], plan.pairs[0]] }, 'Buenas noches');
  check('two models run one after the other, as through English', !twice.error && twice.type === 'done', JSON.stringify(twice));
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  try { browser.kill(); } catch {}
  try { server.close(); } catch {}
  await rm(userData, { recursive: true, force: true }).catch(() => {});
}
const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
