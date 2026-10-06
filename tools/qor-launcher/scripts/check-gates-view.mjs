// Does the Gates surface show exactly what the host computed? (roadmap L2.2)
//
// The host's own tests already prove that `src-tauri/src/gates.rs` applies the
// counting rules in docs/GATES.toml and nothing else, including against the real
// file. They say nothing about what reaches the screen. This check covers the
// other half: it serves the built frontend in a real rendering engine, answers
// `gates_report` with a fixture whose numbers are known, opens the Gates surface
// and reads back what was rendered.
//
// What it is for: the view must display the host's numbers and never compute,
// round, weight or estimate one of its own. A dashboard that quietly invents a
// number is worse than no dashboard, because its numbers get quoted.
//
// It deliberately does NOT read docs/GATES.toml. Checking the view against the
// same file the host reads would pass even if both were wrong together.
//
// No dependencies beyond Node 22+ (for the global WebSocket) and an installed
// Edge or Chrome. Set BROWSER_PATH to use another Chromium-family browser.
//
//   npm run build
//   node scripts/check-gates-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.
//
// Proven to fail: on 17 September 2026 the report handed to the page was altered
// (one gate's met count raised by one, and a bar given to the gate with an
// unmeasurable unit). Four checks failed, including the one that forbids a bar
// while anything is unmeasurable. A check nobody has seen fail is not evidence.

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

// The fixture. Every number here is arbitrary and unrelated to the real gates,
// which is the point: the view must show these and only these.
//
// It exercises each state the host can report: a gate that passed, a gate with
// nothing unmeasurable (so it has a bar), and a gate with an unmeasurable unit
// (so it must have no bar at all, however tempting a partial one looks).
const FIXTURE = {
  repo: 'X:/fixture/repo',
  status: 'Fixture',
  accepted: '1999-12-31',
  problems: ['a problem the host reported'],
  suites: [],
  gates: [
    {
      id: 'fixture-passed',
      name: 'Fixture Passed',
      means: 'Everything in this gate is met.',
      passed: true,
      met: 3,
      not_met: 0,
      unmeasurable: 0,
      bar: { met: 3, total: 3 },
      criteria: [
        {
          id: 'fixture-passed.one',
          kind: 'check',
          says: 'A criterion that is met.',
          units: [{ label: 'unit-a', state: 'met', detail: 'done', asserted: false }],
        },
      ],
    },
    {
      id: 'fixture-measurable',
      name: 'Fixture Measurable',
      means: 'Nothing here is unmeasurable, so it has a bar.',
      passed: false,
      met: 7,
      not_met: 5,
      unmeasurable: 0,
      bar: { met: 7, total: 12 },
      criteria: [
        {
          id: 'fixture-measurable.one',
          kind: 'roadmap',
          says: null,
          units: [
            { label: 'unit-b', state: 'met', detail: 'ticked', asserted: false },
            { label: 'unit-c', state: 'not_met', detail: 'not ticked', asserted: true },
          ],
        },
      ],
    },
    {
      id: 'fixture-unmeasurable',
      name: 'Fixture Unmeasurable',
      means: 'One unit cannot be measured, so there is no bar.',
      passed: false,
      met: 2,
      not_met: 1,
      unmeasurable: 4,
      bar: null,
      criteria: [
        {
          id: 'fixture-unmeasurable.one',
          kind: 'coverage',
          says: 'A criterion with an unreadable signal.',
          units: [{ label: 'unit-d', state: 'unmeasurable', detail: 'no report', asserted: false }],
        },
      ],
    },
  ],
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

const profile = await mkdtemp(join(tmpdir(), 'qor-gates-'));
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

let target;
for (let i = 0; i < 100 && !target; i++) {
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

// A stand-in host. It answers `gates_report` with the fixture, and answers just
// enough of the bootstrap for the shell to render with its rail: without an
// unlocked vault the app stays on the Gate screen and there is no rail to click.
// Everything else fails the way an unreachable host would.
const ACCOUNTS = [{ address: 'qor1fixture', path: "m/44'/0'/0'/0/0", index: 0, label: 'Fixture' }];
const SESSION = {
  qor_id: 'fixture-qor-id',
  username: 'fixture',
  discriminator: 1,
  role: 'user',
  address: 'qor1fixture',
  avatar_url: null,
};
const STATE = {
  version: '0.0.0-fixture',
  vault: { state: 'unlocked', accounts: ACCOUNTS, locks_in: 900 },
  session: SESSION,
  chain_endpoint: 'http://127.0.0.1:0',
  auth_endpoint: 'http://127.0.0.1:0',
  token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
};
const HOST_STUB = `
  window.__GATES_FIXTURE__ = ${JSON.stringify(FIXTURE)};
  window.__LAUNCHER_STATE__ = ${JSON.stringify(STATE)};
  window.__ACCOUNTS__ = ${JSON.stringify(ACCOUNTS)};
  window.__SESSION__ = ${JSON.stringify(SESSION)};
  window.__TAURI_INTERNALS__ = {
    invoke: (cmd) => {
      if (cmd === 'gates_report') return Promise.resolve(window.__GATES_FIXTURE__);
      if (cmd === 'launcher_state') return Promise.resolve(window.__LAUNCHER_STATE__);
      if (cmd === 'vault_status')
        return Promise.resolve({ state: 'unlocked', accounts: window.__ACCOUNTS__, locks_in: 900 });
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
await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
await send('Page.navigate', { url: appUrl });
for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
await sleep(800);

// Open the Gates surface the way a person does: the rail button with that label.
const opened = await evaluate(`(() => {
  const b = [...document.querySelectorAll('nav[aria-label="Primary"] button')]
    .find((n) => n.textContent.trim() === 'Gates');
  if (!b) return false;
  b.click();
  return true;
})()`);
await sleep(600);

// --- the checks ------------------------------------------------------------

const results = [];
const check = (name, actual, expected) => {
  const pass = typeof expected === 'function' ? expected(actual) : actual === expected;
  results.push({ name, pass, actual });
};

// Read a gate's rendered summary card by its name.
const card = (gateName) => `(() => {
  const p = [...document.querySelectorAll('p')].find((n) => n.textContent.trim() === ${JSON.stringify(gateName)});
  const panel = p?.closest('div')?.parentElement;
  if (!panel) return null;
  const counts = {};
  panel.querySelectorAll('dt').forEach((dt) => {
    counts[dt.textContent.trim()] = Number(dt.nextElementSibling?.textContent.trim());
  });
  const bar = panel.querySelector('[role="progressbar"]');
  return {
    counts,
    passedLabel: [...panel.querySelectorAll('p')].map((n) => n.textContent.trim())
      .find((t) => t === 'Passed' || t === 'Not passed') ?? null,
    hasBar: !!bar,
    valuenow: bar ? Number(bar.getAttribute('aria-valuenow')) : null,
    valuemax: bar ? Number(bar.getAttribute('aria-valuemax')) : null,
    noBarNote: !![...panel.querySelectorAll('p')]
      .find((n) => n.textContent.includes('No bar while any unit cannot be measured')),
  };
})()`;

check('the Gates surface opens from the rail', opened, true);

const bodyText = await evaluate('document.body.innerText');
check('the host\'s repository path is shown, not invented', bodyText.includes('X:/fixture/repo'), true);
check('a problem the host reported is shown', bodyText.includes('a problem the host reported'), true);

for (const gate of FIXTURE.gates) {
  const c = await evaluate(card(gate.name));
  check(`${gate.name}: a summary card is rendered`, c !== null, true);
  if (!c) continue;
  check(`${gate.name}: Met shows the host's number`, c.counts.Met, gate.met);
  check(`${gate.name}: Not met shows the host's number`, c.counts['Not met'], gate.not_met);
  check(`${gate.name}: Unmeasurable shows the host's number`, c.counts.Unmeasurable, gate.unmeasurable);
  check(`${gate.name}: passed state matches the host`, c.passedLabel, gate.passed ? 'Passed' : 'Not passed');

  if (gate.bar) {
    check(`${gate.name}: a bar appears, since nothing is unmeasurable`, c.hasBar, true);
    check(`${gate.name}: the bar's value is the host's met count`, c.valuenow, gate.bar.met);
    check(`${gate.name}: the bar's maximum is the host's total`, c.valuemax, gate.bar.total);
  } else {
    check(`${gate.name}: no bar while a unit cannot be measured`, c.hasBar, false);
    check(`${gate.name}: it says why there is no bar`, c.noBarNote, true);
  }
}

// Every criterion and unit the host sent must be on the page. A dashboard that
// silently drops a not-met unit reads as better progress than there is.
for (const gate of FIXTURE.gates) {
  for (const criterion of gate.criteria) {
    check(`${criterion.id} is listed`, bodyText.includes(criterion.id), true);
    for (const unit of criterion.units) {
      check(`${criterion.id}: unit ${unit.label} is listed`, bodyText.includes(unit.label), true);
    }
  }
}

// The view must not report a total of its own devising.
const totals = await evaluate(`(() => {
  const out = [];
  document.querySelectorAll('dl').forEach((dl) => {
    const n = [...dl.querySelectorAll('dd')].map((d) => Number(d.textContent.trim()));
    if (n.length === 3) out.push(n);
  });
  return out;
})()`);
check(
  'every card shows three counts that sum to the host\'s unit total',
  totals.length >= FIXTURE.gates.length &&
    FIXTURE.gates.every((g) => totals.some(([m, n, u]) => m === g.met && n === g.not_met && u === g.unmeasurable)),
  true,
);

// --- report ----------------------------------------------------------------

let passed = 0;
let failed = 0;
for (const r of results) {
  if (r.pass) { passed++; console.log(`PASS  ${r.name}`); }
  else { failed++; console.log(`FAIL  ${r.name}  (got ${JSON.stringify(r.actual)})`); }
}
console.log(`RESULT: ${passed} passed, ${failed} failed`);

ws.close();
browser.kill();
server.close();
await rm(profile, { recursive: true, force: true }).catch(() => {});
process.exit(failed ? 1 : 0);
