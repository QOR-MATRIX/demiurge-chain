// Does the Projects surface show exactly what the repository contains? (Qontrol)
//
// The host's own tests already prove that `src-tauri/src/qontrol/` reads a real
// repository correctly, and that a commit through the gitoxide/libgit2 split
// leaves a tree that both gitoxide and `git status --porcelain` call clean. They
// say nothing about what reaches the screen. This check covers the other half:
// it serves the built frontend in a real rendering engine, answers the
// `qontrol_*` commands with a FIXTURE REPOSITORY whose contents are known, opens
// the Projects surface and reads back what was drawn.
//
// What it is for: a version-control interface that shows what it believes
// rather than what is on disk is the failure that matters. A creator sees a
// commit that did not happen, or a clean tree that is not clean, and finds out
// much later. So this asserts that every path, every commit summary and every
// branch on screen came from the host, and that after a commit the view redraws
// from what the host returned rather than from what it had.
//
// It deliberately does NOT open a real repository. Checking the view against the
// same git the host reads would pass even if both were wrong together — the same
// reason check-gates-view.mjs refuses to read docs/GATES.toml.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome. Set
// BROWSER_PATH to use another Chromium-family browser.
//
//   npm run build
//   node scripts/check-projects-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.
//
// Proven to fail, on 21 September 2026, by TWO injections — and the first
// attempt at proving it taught something worth writing down.
//
//   1. THE VIEW WAS BROKEN. Projects.tsx was altered to render the literal text
//      "a file" in place of each path the host reported. Three checks failed.
//      This is the injection that matters: it is a fault in the thing under
//      test.
//
//   2. THE POST-COMMIT FIXTURE WAS LEFT DIRTY. The host's answer after the
//      commit was made to report the same changes as before. Four checks failed,
//      including the one that catches a view keeping its own picture of the
//      repository instead of redrawing from what came back.
//
//   3. (22 September 2026) THE VIEW'S CLASSES NAMED NOTHING. The check that
//      every class the surface uses is defined by the stylesheet was run against
//      Projects.tsx as it shipped, with `button`, `button-quiet` and `input`:
//      it failed, naming exactly those three. The view was then given the
//      stylesheet's own `btn btn-primary`, `btn btn-ghost` and `field`, and it
//      passes.
//
//   4. (23 September 2026) THE DIFF (P1.1), three faults planted in the view,
//      one at a time, each caught by the check meant for it and by no other:
//      - the guard that drops a late answer removed: "a late answer for a file
//        no longer chosen is dropped" and "so none of that file's lines are on
//        screen" failed, because the first file's lines were drawn under the
//        second file's choice;
//      - an added line drawn as "a line" instead of its text: the check for
//        that line failed;
//      - the diff kept when the project is redrawn: after the commit, the
//        stale path was still on screen, inside the diff panel.
//
// What did NOT work, and why it is recorded rather than quietly dropped:
// altering a value INSIDE the fixture — renaming a changed path, rewriting a
// commit summary — changed nothing, because the assertions iterate over that
// same fixture. Both sides moved together and every check still passed. A
// fixture cannot be used to falsify itself. The assertions are still written
// that way on purpose, because the question here is "did the view show what the
// host returned", not "does the fixture equal a constant" — but it means a
// future proof of this check must inject the fault into the VIEW or into the
// DIVERGENCE between two host answers, never into the fixture's own values.

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

// The fixture repository. Every value is arbitrary and belongs to no real
// project, which is the point: the view must show these and only these.
const DIRTY = {
  path: 'X:/fixture/a-project',
  name: 'a-project',
  branch: 'main',
  changes: [
    { path: 'sessions/take-04.qsession', state: 'modified' },
    { path: 'stems/vox-take-04.wav', state: 'untracked' },
    { path: 'reference/old-mix.flac', state: 'removed' },
  ],
  history: [
    { id: 'a'.repeat(40), short: 'aaaaaaa', summary: 'Rough out the second verse', author: 'Fixture', time: 1758400000 },
    { id: 'b'.repeat(40), short: 'bbbbbbb', summary: 'Lay out the project', author: 'Fixture', time: 1758300000 },
  ],
  branches: [
    { name: 'main', head: true },
    { name: 'alternate-bridge', head: false },
  ],
  can_commit: true,
};

// What the host returns AFTER the commit. The tree is clean and the history has
// gained the new commit. If the view kept its own picture, the old changes would
// still be on screen.
const CLEAN = {
  ...DIRTY,
  changes: [],
  history: [
    { id: 'c'.repeat(40), short: 'ccccccc', summary: 'Commit from the check', author: 'Fixture', time: 1758500000 },
    ...DIRTY.history,
  ],
};

// What the host says changed inside each file (P1.1). One text diff with every
// kind of line, and one binary file, which has no lines to show.
const DIFFS = {
  'sessions/take-04.qsession': {
    path: 'sessions/take-04.qsession', state: 'modified', from: null,
    body: {
      kind: 'text', truncated: false,
      lines: [
        { kind: 'hunk', text: '@@ -12,3 +12,3 @@', old: null, new: null },
        { kind: 'context', text: 'track vox gain=-3.0', old: 12, new: 12 },
        { kind: 'remove', text: 'track keys gain=-6.5', old: 13, new: null },
        { kind: 'add', text: 'track keys gain=-4.0', old: null, new: 13 },
        { kind: 'context', text: 'track bass gain=-2.0', old: 14, new: 14 },
      ],
    },
  },
  'stems/vox-take-04.wav': {
    path: 'stems/vox-take-04.wav', state: 'untracked', from: null,
    body: { kind: 'binary', old_size: null, new_size: 2202009 },
  },
};

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

const profile = await mkdtemp(join(tmpdir(), 'qor-projects-'));
const port = 9800 + Math.floor(Math.random() * 400);
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

const finish = async () => {
  try { browser.kill(); } catch {}
  try { server.close(); } catch {}
  await rm(profile, { recursive: true, force: true }).catch(() => {});
};

try {
  let target;
  for (let i = 0; i < 100 && !target; i++) {
    // Pause before every retry, not only after a refused connection: the browser can answer before its page
    // exists, and the loop then gave up within milliseconds on a slow runner (CI, 6 October 2026).
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
    if (r.result.exceptionDetails) {
      throw new Error(`${expression}\n${JSON.stringify(r.result.exceptionDetails)}`);
    }
    return r.result.result.value;
  };

  // A stand-in host. It answers the qontrol commands with the fixture, and
  // enough of the bootstrap for the shell to render with its rail: without an
  // unlocked vault the app stays on the Gate screen and there is no rail.
  const ACCOUNTS = [{ address: 'qor1fixture', path: "m/44'/0'/0'/0/0", index: 0, label: 'Fixture' }];
  const SESSION = {
    qor_id: 'fixture-qor-id', username: 'fixture', discriminator: 1,
    role: 'user', address: 'qor1fixture', avatar_url: null,
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
    window.__DIRTY__ = ${JSON.stringify(DIRTY)};
    window.__CLEAN__ = ${JSON.stringify(CLEAN)};
    window.__DIFFS__ = ${JSON.stringify(DIFFS)};
    window.__DIFF_CALLS__ = [];
    window.__SLOW__ = null;
    window.__COMMIT_CALLS__ = [];
    window.__MINT_CALLS__ = [];
    window.__CHECK_CALLS__ = [];
    window.__HELD__ = [];
    window.__DISCARD_CALLS__ = [];
    window.__SWITCH_CALLS__ = [];
    window.__REFUSE_SWITCH__ = null;
    window.__BRANCH_CALLS__ = [];
    window.__LAUNCHER_STATE__ = ${JSON.stringify(STATE)};
    window.__ACCOUNTS__ = ${JSON.stringify(ACCOUNTS)};
    window.__SESSION__ = ${JSON.stringify(SESSION)};
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd === 'qontrol_pick_folder') return Promise.resolve(window.__DIRTY__.path);
        if (cmd === 'qontrol_open' || cmd === 'qontrol_read') return Promise.resolve(window.__DIRTY__);
        if (cmd === 'qontrol_diff') {
          window.__DIFF_CALLS__.push(args);
          const d = window.__DIFFS__[args.file];
          const delay = window.__SLOW__ === args.file ? 900 : 0;
          return new Promise((r) => setTimeout(() => r(d ? [d] : []), delay));
        }
        if (cmd === 'qontrol_commit') {
          window.__COMMIT_CALLS__.push(args);
          return Promise.resolve(window.__CLEAN__);
        }
        if (cmd === 'qontrol_check') {
          window.__CHECK_CALLS__.push(args);
          return Promise.resolve(window.__HELD__);
        }
        if (cmd === 'qontrol_discard') {
          window.__DISCARD_CALLS__.push(args);
          const d = window.__DIRTY__;
          return Promise.resolve({ ...d, changes: d.changes.filter((c) => !args.files.includes(c.path)) });
        }
        if (cmd === 'qontrol_switch') {
          window.__SWITCH_CALLS__.push(args);
          if (window.__REFUSE_SWITCH__)
            return Promise.reject({ kind: 'qontrol', message: window.__REFUSE_SWITCH__ });
          const d = window.__DIRTY__;
          return Promise.resolve({
            ...d, branch: args.branch,
            branches: d.branches.map((b) => ({ ...b, head: b.name === args.branch })),
          });
        }
        if (cmd === 'qontrol_branch') {
          window.__BRANCH_CALLS__.push(args);
          const d = window.__DIRTY__;
          return Promise.resolve({ ...d, branches: [...d.branches, { name: args.name, head: false }] });
        }
        if (cmd === 'qontrol_mint') {
          window.__MINT_CALLS__.push(args);
          return Promise.resolve({
            collection: 7, item: 3, name: 'a-project',
            reference: { algo: 'BLAKE3-256', root: '9e'.repeat(32), size: 412 },
            commit: { kind: 'SHA-1', id: 'c'.repeat(40) },
            tx_hash: '0x' + '11'.repeat(32), block_hash: '0x' + '22'.repeat(32),
          });
        }
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

  // Open Projects the way a person does: the rail button with that label.
  const opened = await evaluate(`(() => {
    const b = [...document.querySelectorAll('nav[aria-label="Primary"] button')]
      .find((n) => n.textContent.trim() === 'Projects');
    if (!b) return false;
    b.click();
    return true;
  })()`);
  check('the rail has a Projects button and it opens the surface', opened === true);
  await sleep(400);

  const heading = await evaluate(`document.querySelector('main h1')?.textContent.trim() ?? ''`);
  check('the surface is Projects', heading === 'Projects', heading);

  // Before a folder is open it must say so, not show an empty project.
  const empty = await evaluate(`document.querySelector('main')?.textContent.includes('No project open') ?? false`);
  check('with nothing open it says so rather than drawing an empty project', empty === true);

  // Open the fixture project.
  const openClicked = await evaluate(`(() => {
    const b = [...document.querySelectorAll('main button')]
      .find((n) => n.textContent.trim().includes('Open a folder'));
    if (!b) return false;
    b.click();
    return true;
  })()`);
  check('there is a way to open a folder', openClicked === true);
  await sleep(600);

  const text = async () => evaluate(`document.querySelector('main')?.textContent ?? ''`);
  let body = await text();

  check('the project name from the host is shown', body.includes(DIRTY.name), DIRTY.name);
  check('the project path from the host is shown', body.includes(DIRTY.path));

  // Every changed path, exactly as the host reported it.
  for (const change of DIRTY.changes) {
    check(`the changed path ${change.path} is shown`, body.includes(change.path));
    check(`its state reads "${change.state}"`, body.includes(change.state));
  }
  check(
    'the number of changes matches the host',
    body.includes(`${DIRTY.changes.length} changed`),
    `${DIRTY.changes.length}`,
  );

  // Every commit, by summary and short id.
  for (const commit of DIRTY.history) {
    check(`the commit "${commit.summary}" is shown`, body.includes(commit.summary));
    check(`its short id ${commit.short} is shown`, body.includes(commit.short));
  }

  // Branches, and which one is current.
  for (const branch of DIRTY.branches) {
    check(`the branch ${branch.name} is shown`, body.includes(branch.name));
  }
  check('the branch you are on is marked', body.includes('on this one'));

  // Diffs (P1.1). Choosing a change asks the host for that file's diff and draws
  // exactly what came back; the view computes nothing of its own.
  const choose = (path) =>
    evaluate(`(() => {
      const b = document.querySelector('main [data-change="${path}"]');
      if (!b) return false;
      b.click();
      return true;
    })()`);
  const shownDiff = () => evaluate(`document.querySelector('main [data-diff]')?.getAttribute('data-diff') ?? null`);

  const textDiff = DIFFS['sessions/take-04.qsession'];
  check('a change can be chosen', (await choose(textDiff.path)) === true);
  await sleep(300);
  let calls = await evaluate(`window.__DIFF_CALLS__`);
  check('the host was asked for that one file', calls.length === 1 && calls[0]?.file === textDiff.path, JSON.stringify(calls));
  check('in the open project', calls[0]?.path === DIRTY.path, calls[0]?.path);
  check('the diff shown is that file\'s', (await shownDiff()) === textDiff.path);

  const drawn = await evaluate(`[...document.querySelectorAll('main [data-diff-line]')].map((n) => ({
    kind: n.getAttribute('data-diff-line'),
    text: n.textContent,
    ins: Boolean(n.querySelector('ins')),
    del: Boolean(n.querySelector('del')),
  }))`);
  const expected = textDiff.body.lines.filter((l) => l.kind !== 'hunk');
  check('every line the host returned is drawn, and no other', drawn.length === expected.length, `${drawn.length} of ${expected.length}`);
  for (const [i, line] of expected.entries()) {
    const row = drawn[i];
    check(`line ${i + 1} is a ${line.kind} line reading "${line.text}"`, row?.kind === line.kind && row?.text.includes(line.text), row?.text);
    const number = line.old ?? line.new;
    check(`  and carries its line number ${number}`, row?.text.includes(String(number)));
  }
  check('an added line is marked as inserted text, not by colour alone', drawn.some((r) => r.kind === 'add' && r.ins));
  check('a removed line is marked as deleted text, not by colour alone', drawn.some((r) => r.kind === 'remove' && r.del));
  check(
    'the hunk header is drawn',
    (await evaluate(`document.querySelector('main [data-diff-hunk]')?.textContent.trim() ?? ''`)) === textDiff.body.lines[0].text,
  );
  check(
    'the chosen change is marked as chosen',
    (await evaluate(`document.querySelector('main [data-change="${textDiff.path}"]')?.getAttribute('aria-pressed')`)) === 'true',
  );

  // Choosing it again puts it away.
  await choose(textDiff.path);
  await sleep(200);
  check('choosing the same change again closes its diff', (await shownDiff()) === null);

  // A binary file has no lines, and says so with the sizes the host gave.
  const bin = DIFFS['stems/vox-take-04.wav'];
  await choose(bin.path);
  await sleep(300);
  const binText = await evaluate(`document.querySelector('main [data-diff-binary]')?.textContent ?? ''`);
  check('a binary file says it is not shown as lines', binText.includes('binary'), binText);
  check('with the size the host reported', binText.includes('2.1 MB') && binText.includes('none'), binText);
  check('and draws no lines', (await evaluate(`document.querySelectorAll('main [data-diff-line]').length`)) === 0);

  // THE RACE. The person chooses one file, then another before the first answer
  // arrives. The late answer is for a file nobody is looking at any more, and
  // drawing it would put one file's changes under another's name.
  await choose(bin.path); // put it away
  await sleep(200);
  await evaluate(`window.__SLOW__ = ${JSON.stringify(textDiff.path)}`);
  await choose(textDiff.path);
  await sleep(50);
  await choose(bin.path);
  await sleep(1300);
  check('a late answer for a file no longer chosen is dropped', (await shownDiff()) === bin.path, await shownDiff());
  check(
    'so none of that file\'s lines are on screen',
    (await evaluate(`document.querySelectorAll('main [data-diff-line]').length`)) === 0,
  );
  await evaluate(`window.__SLOW__ = null`);

  // Leave the text diff open, so the class check below counts its classes and
  // the commit below has to take it away.
  await choose(textDiff.path);
  await sleep(300);
  check('the text diff is open again before committing', (await shownDiff()) === textDiff.path);

  // Mint (M4.1). With changes on disk there is nothing to press: a mint pins a
  // commit, and what is not committed would not be in the asset.
  const mintButton = `[...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === 'Mint this commit')`;
  check('with uncommitted changes there is no Mint button', (await evaluate(`Boolean(${mintButton})`)) === false);
  check('with uncommitted changes it says to commit first', body.includes('Commit first'));

  // Every class the surface uses must exist in the stylesheet. A control whose
  // class names nothing renders as the browser's default and nothing fails: the
  // Projects surface shipped that way on 21 September 2026 with `button`,
  // `button-quiet` and `input`, none of which the stylesheet defines. Checked
  // with the New panel open too, so its controls are counted, and again below
  // with the guard's list and the discard question open (P1.2).
  const openNew = await evaluate(`(() => {
    const b = [...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === 'New');
    if (!b) return false;
    b.click();
    return true;
  })()`);
  await sleep(300);
  const classesNamingNothing = () => evaluate(`(() => {
    const selectors = [];
    const walk = (list) => {
      for (const rule of list) {
        if (rule.selectorText) selectors.push(rule.selectorText);
        if (rule.cssRules) walk(rule.cssRules);
      }
    };
    for (const sheet of document.styleSheets) {
      try { walk(sheet.cssRules); } catch {}
    }
    const all = selectors.join(' ');
    const used = new Set();
    for (const el of document.querySelectorAll('main [class]')) for (const c of el.classList) used.add(c);
    // lucide marks its icons with classes of its own that carry no style.
    return [...used]
      .filter((c) => !c.startsWith('lucide'))
      .filter((c) => !all.includes('.' + CSS.escape(c)))
      .sort();
  })()`);
  const undefinedClasses = await classesNamingNothing();
  check('the New panel opens, so its controls are checked too', openNew === true);
  check(
    'every class the surface uses is defined by the stylesheet',
    undefinedClasses.length === 0,
    undefinedClasses.join(' '),
  );
  await evaluate(`(() => {
    const b = [...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === 'New');
    if (b) b.click();
  })()`);
  await sleep(200);

  // A commit must not be possible without a message.
  const disabledWithoutMessage = await evaluate(`(() => {
    const b = [...document.querySelectorAll('main button')]
      .find((n) => n.textContent.trim() === 'Commit');
    return b ? b.disabled === true : null;
  })()`);
  check('Commit is refused until there is a message', disabledWithoutMessage === true);

  // Type a message and commit.
  const typed = await evaluate(`(() => {
    const input = [...document.querySelectorAll('main input')]
      .find((n) => n.placeholder && n.placeholder.includes('What did you change'));
    if (!input) return false;
    const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    setter.call(input, 'Commit from the check');
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);
  check('the commit message field accepts typing', typed === true);
  await sleep(200);

  const committed = await evaluate(`(() => {
    const b = [...document.querySelectorAll('main button')]
      .find((n) => n.textContent.trim() === 'Commit');
    if (!b || b.disabled) return false;
    b.click();
    return true;
  })()`);
  check('Commit becomes available once a message is typed', committed === true);
  await sleep(700);

  // The message the view sent is the message that was typed — not a stale one,
  // and not one the view invented.
  const sent = await evaluate(`window.__COMMIT_CALLS__`);
  check('the host was asked to commit exactly once', Array.isArray(sent) && sent.length === 1, `${sent?.length}`);
  check(
    'the message the host received is the one that was typed',
    sent?.[0]?.message === 'Commit from the check',
    sent?.[0]?.message,
  );
  check('the path the host received is the open project', sent?.[0]?.path === DIRTY.path);
  const asked = await evaluate(`window.__CHECK_CALLS__`);
  check('the guard was asked first, once, for this project', asked.length === 1 && asked[0]?.path === DIRTY.path, JSON.stringify(asked));
  check(
    'with every change ticked, no file list is sent: the host commits everything',
    sent?.[0]?.files == null && asked[0]?.files == null,
    JSON.stringify(sent?.[0]?.files),
  );
  check('and no warning is accepted that the guard did not raise', (sent?.[0]?.accepted ?? []).length === 0);

  // THE ONE THAT MATTERS. After the commit the view must draw what the host
  // returned. A view holding its own picture would still show the old changes.
  body = await text();
  check(
    'after committing, the new commit from the host is on screen',
    body.includes('Commit from the check'),
  );
  for (const change of DIRTY.changes) {
    check(
      `after committing, the stale change ${change.path} is GONE`,
      !body.includes(change.path),
    );
  }
  check(
    'after committing it says nothing has changed',
    body.includes('Nothing has changed'),
  );

  // Now the tree is clean, the commit a mint would pin is shown by its FULL
  // hash — the one the host returned, never a branch name — and Mint appears.
  const pinned = await evaluate(`document.querySelector('main [data-mint-commit]')?.textContent.trim() ?? ''`);
  check('the commit a mint would pin is shown by its full hash', pinned === CLEAN.history[0].id, pinned);
  check('once the tree is clean there is a Mint button', (await evaluate(`Boolean(${mintButton})`)) === true);

  await evaluate(`(${mintButton}).click()`);
  await sleep(600);
  const minted = await evaluate(`window.__MINT_CALLS__`);
  check('the host was asked to mint exactly once', Array.isArray(minted) && minted.length === 1, `${minted?.length}`);
  check('for the open project', minted?.[0]?.path === DIRTY.path, minted?.[0]?.path);
  check('from the active account', minted?.[0]?.from === 'qor1fixture', minted?.[0]?.from);
  check(
    'and the view sent nothing else: the host computes the reference',
    Object.keys(minted?.[0] ?? {}).sort().join(',') === 'from,path',
    Object.keys(minted?.[0] ?? {}).join(','),
  );
  body = await text();
  check('the receipt the host returned is shown', body.includes('Minted as asset 7/3'));
  check('with the finalised block it names', body.includes('0x' + '22'.repeat(32)));

  // --- P1.2: per-file commit, the guard, discard, switch, branch -------------
  //
  // The fixture project is opened again, dirty, and each act is driven the way a
  // person drives it. Every assertion is about what the view SENT and what it
  // DREW from the answer; the host's own tests prove what those acts do on disk.

  const click = (selector) =>
    evaluate(`(() => {
      const n = document.querySelector(${JSON.stringify(selector)});
      if (!n || n.disabled) return false;
      n.click();
      return true;
    })()`);
  const clickText = (label) =>
    evaluate(`(() => {
      const b = [...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === ${JSON.stringify(label)});
      if (!b || b.disabled) return false;
      b.click();
      return true;
    })()`);
  const type = (selector, value) =>
    evaluate(`(() => {
      const input = document.querySelector(${JSON.stringify(selector)});
      if (!input) return false;
      const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
      setter.call(input, ${JSON.stringify(value)});
      input.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
  const reopen = async () => {
    await clickText('Open a folder');
    await sleep(600);
  };

  await reopen();
  const boxes = await evaluate(`[...document.querySelectorAll('main [data-include]')].map((n) => ({ path: n.getAttribute('data-include'), checked: n.checked, label: n.getAttribute('aria-label') }))`);
  check('every change has a box for the next commit', boxes.length === DIRTY.changes.length, `${boxes.length}`);
  check('and every box starts ticked', boxes.every((b) => b.checked));
  check('each box is named for its file', boxes.every((b) => b.label === `Include ${b.path} in the next commit`));

  // Leave one out. The button says how many of how many.
  const left = 'stems/vox-take-04.wav';
  await click(`main [data-include="${left}"]`);
  await sleep(200);
  const partial = `Commit ${DIRTY.changes.length - 1} of ${DIRTY.changes.length}`;
  check('with one unticked, the button says what will be committed', (await text()).includes(partial), partial);

  // The guard holds two files back. Nothing is committed until the person answers.
  const HELD = [
    { path: '.env', concern: 'credential', reason: 'an environment file, where secrets are usually kept' },
    { path: 'sessions/bounce.wav', concern: 'large', reason: '72 MiB' },
  ];
  await evaluate(`window.__HELD__ = ${JSON.stringify(HELD)}; window.__CHECK_CALLS__ = []; window.__COMMIT_CALLS__ = []`);
  await type('main input[placeholder="What did you change?"]', 'Two of three');
  await sleep(150);
  await clickText(partial);
  await sleep(500);
  const chosenFiles = DIRTY.changes.map((c) => c.path).filter((p) => p !== left);
  let checks = await evaluate(`window.__CHECK_CALLS__`);
  check('the guard was asked about exactly the ticked files', JSON.stringify(checks[0]?.files) === JSON.stringify(chosenFiles), JSON.stringify(checks[0]?.files));
  check('nothing was committed before the person answered', (await evaluate(`window.__COMMIT_CALLS__.length`)) === 0);
  const heldShown = await evaluate(`[...document.querySelectorAll('main [data-held-file]')].map((n) => n.textContent)`);
  check('every file the guard held back is listed', heldShown.length === HELD.length, `${heldShown.length}`);
  for (const w of HELD) {
    check(`${w.path} is listed with its reason`, heldShown.some((t) => t.includes(w.path) && t.includes(w.reason)));
  }
  check('a large file is called large', heldShown.some((t) => t.includes('large, 72 MiB')));
  const heldClasses = await classesNamingNothing();
  check('every class in the held-back list is defined by the stylesheet', heldClasses.length === 0, heldClasses.join(' '));

  // Go back: nothing sent, the message kept.
  await clickText('Go back');
  await sleep(200);
  check('Go back closes the list', (await evaluate(`Boolean(document.querySelector('main [data-held]'))`)) === false);
  check('and commits nothing', (await evaluate(`window.__COMMIT_CALLS__.length`)) === 0);
  check(
    'and keeps the message that was typed',
    (await evaluate(`document.querySelector('main input[placeholder="What did you change?"]')?.value`)) === 'Two of three',
  );

  // Commit anyway: the yes goes back to the host, file by file, concern by concern.
  await clickText(partial);
  await sleep(500);
  await clickText('Commit anyway');
  await sleep(600);
  const anyway = await evaluate(`window.__COMMIT_CALLS__`);
  check('Commit anyway commits once', anyway.length === 1, `${anyway.length}`);
  check('with exactly the ticked files', JSON.stringify(anyway[0]?.files) === JSON.stringify(chosenFiles), JSON.stringify(anyway[0]?.files));
  check(
    'and a yes for exactly what was held back, by file and concern and nothing more',
    JSON.stringify(anyway[0]?.accepted) === JSON.stringify(HELD.map(({ path, concern }) => ({ path, concern }))),
    JSON.stringify(anyway[0]?.accepted),
  );
  await evaluate(`window.__HELD__ = []`);

  // Discard. Offered on a modified file, with a question first.
  await reopen();
  await choose(textDiff.path);
  await sleep(300);
  check('a modified file offers a discard', (await evaluate(`Boolean(document.querySelector('main [data-discard]'))`)) === true);
  await click('main [data-discard]');
  await sleep(200);
  const question = await evaluate(`document.querySelector('main [data-discard-confirm]')?.textContent ?? ''`);
  check('discarding asks first, naming the file', question.includes(textDiff.path), question);
  check('and says the change is lost', question.includes('lost'));
  const discardClasses = await classesNamingNothing();
  check('every class in the discard question is defined by the stylesheet', discardClasses.length === 0, discardClasses.join(' '));
  await clickText('Keep it');
  await sleep(200);
  check('Keep it sends nothing', (await evaluate(`window.__DISCARD_CALLS__.length`)) === 0);
  check('and puts the question away', (await evaluate(`Boolean(document.querySelector('main [data-discard-confirm]'))`)) === false);

  await click('main [data-discard]');
  await sleep(200);
  await clickText('Discard the change');
  await sleep(600);
  const discards = await evaluate(`window.__DISCARD_CALLS__`);
  check(
    'the host is asked to discard that one file, in this project',
    discards.length === 1 && discards[0]?.path === DIRTY.path && JSON.stringify(discards[0]?.files) === JSON.stringify([textDiff.path]),
    JSON.stringify(discards),
  );
  body = await text();
  check('the discarded change is gone from the list the host returned', !body.includes(textDiff.path));

  // A new file is never offered a discard: that would be deleting it.
  await reopen();
  await choose('stems/vox-take-04.wav');
  await sleep(300);
  check('an untracked file offers no discard', (await evaluate(`Boolean(document.querySelector('main [data-discard]'))`)) === false);
  await choose('stems/vox-take-04.wav');
  await sleep(200);

  // Switching. Offered on every branch but the current one.
  check('the current branch offers no switch', (await evaluate(`Boolean(document.querySelector('main [data-switch="main"]'))`)) === false);
  check('another branch offers one', (await evaluate(`Boolean(document.querySelector('main [data-switch="alternate-bridge"]'))`)) === true);

  // Refused: the host's words reach the person, and the view does not pretend.
  const refusal = 'switching to alternate-bridge would overwrite changes you have not committed. Commit or discard them first; nothing was changed';
  await evaluate(`window.__REFUSE_SWITCH__ = ${JSON.stringify(refusal)}`);
  await click('main [data-switch="alternate-bridge"]');
  await sleep(600);
  check('a refused switch shows the host\'s reason', (await evaluate(`document.body.textContent`)).includes(refusal));
  check(
    'and the view still says you are on main',
    (await evaluate(`document.querySelector('main [data-branch="main"]')?.textContent ?? ''`)).includes('on this one'),
  );
  await evaluate(`window.__REFUSE_SWITCH__ = null`);

  await click('main [data-switch="alternate-bridge"]');
  await sleep(600);
  const switches = await evaluate(`window.__SWITCH_CALLS__`);
  check('the host is asked to switch to that branch', switches.at(-1)?.branch === 'alternate-bridge' && switches.at(-1)?.path === DIRTY.path, JSON.stringify(switches.at(-1)));
  check(
    'and the view draws the branch the host now reports',
    (await evaluate(`document.querySelector('main [data-branch="alternate-bridge"]')?.textContent ?? ''`)).includes('on this one'),
  );

  // A new branch: the name typed is the name sent, and the list is the host's.
  await type('main [data-branch-name]', 'second-chorus');
  await sleep(150);
  await clickText('Make the branch');
  await sleep(600);
  const made = await evaluate(`window.__BRANCH_CALLS__`);
  check('the host is asked to make the branch that was typed', made.length === 1 && made[0]?.name === 'second-chorus' && made[0]?.path === DIRTY.path, JSON.stringify(made));
  check('and the new branch is drawn from the answer', (await evaluate(`Boolean(document.querySelector('main [data-branch="second-chorus"]'))`)) === true);
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
