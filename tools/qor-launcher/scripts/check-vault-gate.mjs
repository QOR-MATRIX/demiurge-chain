// Does the launcher open with nothing asked, and never wait on QOR ID? (ADR-056)
//
// The host's tests prove that the vault seals under a key in the keychain,
// opens with nothing asked, moves a Windows Hello or passphrase vault once, and
// sets an old vault aside when restoring. They say nothing about the screens.
//
// This serves the built frontend in a real rendering engine against a stand-in
// host that records every command it is sent, and asserts what matters to a
// person opening the launcher:
//
//   - a vault whose key is in the keychain opens before the first screen, and
//     the shell shows with no lock screen and nothing typed;
//   - QOR ID being down stops nothing: the shell shows, and Settings offers to
//     sign in again with the vault's key, never a password;
//   - a first run makes the vault from one button, sending the host the phrase
//     and nothing else;
//   - a Windows Hello vault asks Hello once on arrival, one last time, and a
//     cancel is not asked again on its own;
//   - a passphrase vault asks for its passphrase one last time, and Hello is not
//     asked for it;
//   - a typed phrase is shown back as the account it opens BEFORE anything is
//     restored, and replacing a vault says it is set aside, not deleted;
//   - Settings shows the recovery phrase after the host's own dialog.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome.
//
//   npm run build
//   node scripts/check-vault-gate.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.
//
// Its first runs were flaky, and not because of the app. A headless page the
// window manager thinks is hidden draws no frames, so the Gate's cross-fade
// never finished; and a step can end while the previous screen is still drawn.
// So the page is kept in front, and every click and every field waits for its
// target instead of sleeping a fixed time. A step walk that fails says where
// it stopped.

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

// Arbitrary values that belong to no real vault.
const RESTORED = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
const ACCOUNTS = [{ address: RESTORED, account_id: '0x' + '8e'.repeat(32), path: '', index: 0, label: 'Main' }];
const SESSION = { qor_id: 'fixture#0001', username: 'fixture', discriminator: 1, role: 'user', address: RESTORED, avatar_url: null };
const TOKEN = { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' };
// Twenty-four words as a person might type them: mixed case, doubled spaces, a
// line break. The host must receive them normalised.
const WORDS = Array.from({ length: 24 }, (_, i) => `word${String.fromCharCode(97 + (i % 26))}${i}`);
const TYPED = `  ${WORDS.slice(0, 12).join('  ').toUpperCase()}\n${WORDS.slice(12).join(' ')}  `;
const NORMALISED = WORDS.join(' ').toLowerCase();
const PASSPHRASE = 'the old passphrase, typed once more';
// What the launcher shows for a cancelled Windows Hello prompt (`explain` in
// src/lib/ipc.ts): the host reports a cancel as `declined`.
const CANCELLED = 'You cancelled, so nothing was signed, changed or opened.';
const KEY_GONE = 'this computer account no longer holds the vault\'s key. Restore the vault from your recovery phrase';
// The host's refusal of a bad phrase, and what the launcher shows for that
// kind of error, which is its own sentence.
const BAD_PHRASE = 'invalid mnemonic: checksum mismatch';
const BAD_PHRASE_SHOWN = 'That recovery phrase is not valid. Check the spelling and word order.';

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

// --- a stand-in host --------------------------------------------------------
//
// Configured per scenario through localStorage, so a reload starts it afresh:
//   vault    'absent' | 'locked' | 'inside' (open and signed in)
//   sealed   'keychain' | 'hello' | 'passphrase': what a locked vault is sealed with
//   unlock   'arrive' | 'cancel' | 'fail' (the keychain lost its key)
//   qor      'up' | 'down': whether QOR ID signs in when the vault opens
//   preview  'ok' | 'bad'
// Every command is recorded in window.__CALLS__.
const HOST_STUB = `
  (() => {
    const config = JSON.parse(localStorage.getItem('qor-check.host') || '{}');
    const accounts = ${JSON.stringify(ACCOUNTS)};
    const session0 = ${JSON.stringify(SESSION)};
    const unlocked = () => ({ state: 'unlocked', accounts });
    let vault =
      config.vault === 'absent' ? { state: 'absent' }
      : config.vault === 'locked' ? { state: 'locked', sealed_with: config.sealed ?? 'keychain' }
      : unlocked();
    let session = config.vault === 'inside' ? session0 : null;
    let qorUp = config.qor !== 'down';
    const arrive = () => {
      vault = unlocked();
      session = qorUp ? session0 : null;
      return { accounts, session, needs_name: false, sign_in_problem: qorUp ? null : 'network error: QOR ID is not reachable' };
    };
    window.__CALLS__ = [];
    window.__QOR_UP__ = () => { qorUp = true; };
    const ok = (v) => Promise.resolve(v);
    const no = (kind, message) => Promise.reject({ kind, message });
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        window.__CALLS__.push({ cmd, args: args ?? null });
        if (cmd === 'launcher_state')
          return ok({ version: '0.0.0-check', vault, session, chain_endpoint: 'ws://127.0.0.1:9944', auth_endpoint: 'http://127.0.0.1:8080/api/v1', token: ${JSON.stringify(TOKEN)} });
        if (cmd === 'vault_status') return ok(vault);
        if (cmd === 'qor_restore') return session ? ok(session) : no('not_authenticated', 'no session');
        if (cmd === 'vault_generate_phrase') return ok(${JSON.stringify(WORDS.join(' '))});
        if (cmd === 'vault_preview_phrase')
          return config.preview === 'bad' ? no('bad_mnemonic', ${JSON.stringify(BAD_PHRASE)}) : ok(${JSON.stringify(RESTORED)});
        if (cmd === 'vault_create' || cmd === 'vault_restore' || cmd === 'vault_move_to_keychain') return ok(arrive());
        if (cmd === 'vault_unlock') {
          if (vault.state === 'locked' && vault.sealed_with === 'passphrase')
            return no('passphrase_vault', 'this vault was sealed with a passphrase');
          if (config.unlock === 'cancel') return no('declined', 'declined at the confirmation prompt');
          if (config.unlock === 'fail') return no('keychain', ${JSON.stringify(KEY_GONE)});
          return ok(arrive());
        }
        if (cmd === 'qor_sign_in') return ok(arrive());
        if (cmd === 'vault_export_phrase') return ok(${JSON.stringify(WORDS.join(' '))});
        if (cmd === 'vault_lock') return ok(null);
        return no('internal', 'no host in this check');
      },
      transformCallback: () => 0,
      unregisterCallback: () => {},
      convertFileSrc: (s) => s,
      metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
      plugins: {},
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  })();
`;

// --- drive a real rendering engine ----------------------------------------

const profile = await mkdtemp(join(tmpdir(), 'qor-vault-gate-'));
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
    '--disable-backgrounding-occluded-windows',
    '--disable-renderer-backgrounding',
    '--disable-background-timer-throttling',
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

  await send('Page.enable');
  await send('Runtime.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);

  // Wait until `expression` is true in the page, for up to `ms`. Returns
  // whether it became true. Fixed sleeps made this check flaky under load.
  const until = async (expression, ms = 5000) => {
    const start = Date.now();
    for (;;) {
      if (await evaluate(`Boolean(${expression})`)) return true;
      if (Date.now() - start > ms) return false;
      await sleep(100);
    }
  };
  // The screen has stopped changing: one screen is drawn (the Gate's steps
  // cross-fade, and the shell's views too), it is fully opaque, and the host
  // has been quiet for a moment, so whatever it answered has been drawn.
  const settle = async () => {
    await until(`(() => {
      const heads = document.querySelectorAll('h1');
      if (heads.length !== 1) return false;
      for (let n = heads[0]; n; n = n.parentElement) if (getComputedStyle(n).opacity !== '1') return false;
      return true;
    })()`);
    let seen = -1;
    for (let i = 0; i < 40; i++) {
      const now = await evaluate(`window.__CALLS__?.length ?? 0`);
      if (now === seen) break;
      seen = now;
      await sleep(250);
    }
  };

  // Start a scenario from a fresh page. Motion is reduced so the Gate's steps
  // swap without a cross-fade to wait out.
  const boot = async (host) => {
    await evaluate(`localStorage.setItem('qor-check.host', ${JSON.stringify(JSON.stringify(host))})`);
    await evaluate(`localStorage.setItem('qor.a11y', ${JSON.stringify(JSON.stringify({ ambience: 'off', motion: 'reduced' }))})`);
    events.length = 0;
    await send('Page.reload', { ignoreCache: true });
    for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
    // A headless page the window manager thinks is hidden produces no frames,
    // and then the Gate's cross-fade never finishes and "Opening the gate…"
    // stays on screen: seen here, and recorded in HANDOFF.md section 5.
    await send('Page.bringToFront');
    await send('Emulation.setFocusEmulationEnabled', { enabled: true });
    await settle();
  };

  const calls = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)})`);
  const text = () => evaluate(`document.body.innerText`);
  const heading = () => evaluate(`document.querySelector('h1')?.textContent.trim() ?? ''`);
  const eyebrow = () => evaluate(`document.querySelector('h1')?.previousElementSibling?.textContent.trim() ?? ''`);
  const alert = () => evaluate(`document.querySelector('[role="alert"]')?.textContent.trim() ?? ''`);
  const inside = () => evaluate(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`);
  // A button by its exact visible text, in `scope`.
  const button = (label, scope = 'body') =>
    `[...document.querySelectorAll(${JSON.stringify(`${scope} button`)})].find((b) => b.textContent.trim() === ${JSON.stringify(label)})`;
  // Whether a button is on screen now, and whether one appears within a few
  // seconds: what should be there is waited for, what should not is not.
  const present = (label, scope) => evaluate(`Boolean(${button(label, scope)})`);
  const has = (label, scope) => until(button(label, scope), 3000);
  const headingIs = (expected) => until(`document.querySelector('h1')?.textContent.trim() === ${JSON.stringify(expected)}`, 3000);
  const enabled = (label, scope) => evaluate(`(() => { const b = ${button(label, scope)}; return b ? !b.disabled : null; })()`);
  // Clicks a button once it is on screen and enabled, waiting a few seconds for
  // it: under load a screen can still be arriving when the previous step ends.
  const click = async (label, scope) => {
    await until(`(() => { const b = ${button(label, scope)}; return b && !b.disabled; })()`, 3000);
    const done = await evaluate(`(() => { const b = ${button(label, scope)}; if (!b || b.disabled) return false; b.click(); return true; })()`);
    await sleep(100);
    await settle();
    return done;
  };
  // React reads the value through the element's own setter, so it is set that
  // way and announced with an input event, as typing would.
  const type = async (selector, value) => {
    await until(`document.querySelector(${JSON.stringify(selector)})`, 3000);
    const done = await evaluate(`(() => {
      const el = document.querySelector(${JSON.stringify(selector)});
      if (!el) return false;
      const proto = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
      Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, ${JSON.stringify(value)});
      el.dispatchEvent(new Event('input', { bubbles: true }));
      return true;
    })()`);
    await sleep(100);
    await settle();
    return done;
  };

  // Any field that takes a secret: a passphrase or a password.
  const secretFields = () => evaluate(`document.querySelectorAll('input[type="password"]').length`);
  const noSecret = async (where) => check(`${where}: no passphrase or password is asked`, (await secretFields()) === 0);
  // A command sent with no arguments: Tauri sends an empty object.
  const none = (call) => !!call && Object.keys(call.args ?? {}).length === 0;
  const gateShown = () => evaluate(`Boolean(document.querySelector('.glass-solid.cut h1'))`);
  const openSettings = async () => {
    await evaluate(`[...document.querySelectorAll('nav[aria-label="Primary"] button')].find((n) => n.textContent.trim() === 'Settings')?.click()`);
    await until(`document.querySelector('main')?.getAttribute('aria-label') === 'Settings'`);
    await settle();
  };

  // --- 1. a keychain vault opens with nothing asked ------------------------

  await boot({ vault: 'locked', sealed: 'keychain', unlock: 'arrive' });
  check('a keychain vault reaches the shell on its own', await inside());
  check('with no lock screen drawn', !(await gateShown()));
  check('it is opened once, with nothing sent', (await calls('vault_unlock')).length === 1 && none((await calls('vault_unlock'))[0]));
  await noSecret('opening the launcher');
  check('the title bar has no lock countdown', !(await evaluate(`/\\b\\d{1,2}:\\d{2}\\b/.test(document.querySelector('header')?.innerText ?? '')`)));

  // --- 2. QOR ID down stops nothing -----------------------------------------

  await boot({ vault: 'locked', sealed: 'keychain', unlock: 'arrive', qor: 'down' });
  check('with QOR ID down, the shell still shows', await inside());
  check('and no lock screen is drawn', !(await gateShown()));
  await openSettings();
  check('Settings says it is not signed in', (await text()).includes('Not signed in to QOR ID'));
  await noSecret('Settings, not signed in');
  check('there is no password route', !(await text()).toLowerCase().includes('password'));
  await evaluate(`window.__QOR_UP__()`);
  await click('Sign in to QOR ID', 'main');
  check('signing in asks the host with the vault, once', (await calls('qor_sign_in')).length === 1);
  check('no password sign-in is ever sent', (await calls('qor_login')).length === 0);
  check('the QOR ID is then shown', (await text()).includes(SESSION.qor_id));

  // --- 3. a first run --------------------------------------------------------

  await boot({ vault: 'absent' });
  check('with no vault, the Gate welcomes', (await headingIs('Welcome')), await heading());
  check('it says there is nothing to type', (await text()).includes('nothing to'));
  await noSecret('the welcome screen');
  await click('Begin');
  const create = await calls('vault_create');
  check('Begin creates the vault once', create.length === 1, `${create.length} calls`);
  check(
    'with the phrase the host made, and nothing else',
    JSON.stringify(Object.keys(create[0]?.args ?? {})) === '["phrase"]' && create[0]?.args?.phrase === WORDS.join(' '),
    JSON.stringify(Object.keys(create[0]?.args ?? {})),
  );
  check('and reaches the shell', await inside());

  // --- 4. restoring with no vault here ------------------------------------

  await boot({ vault: 'absent', preview: 'ok' });
  check('the Gate offers to restore from a phrase already held', await click('I already have a recovery phrase'));
  check('the restore screen asks for the phrase', (await headingIs('Enter your recovery phrase')), await heading());
  check('with no vault here, it does not talk about setting one aside', !(await text()).includes('set aside'));
  check('"Check the phrase" is refused while nothing is typed', (await enabled('Check the phrase')) === false);
  await type('textarea[aria-label="Recovery phrase"]', 'one two three four five');
  check('"Check the phrase" is refused for a length no phrase has', (await enabled('Check the phrase')) === false);
  await type('textarea[aria-label="Recovery phrase"]', TYPED);
  check('twenty-four words, typed untidily, are counted', (await text()).includes('24 words'));
  await click('Check the phrase');
  const preview = await calls('vault_preview_phrase');
  check('the host is asked which account the phrase opens, once', preview.length === 1, `${preview.length} calls`);
  check('the host receives the phrase normalised', preview[0]?.args?.phrase === NORMALISED);
  check('the account the host names is shown', (await text()).includes(RESTORED));
  check('nothing has been restored yet', (await calls('vault_restore')).length === 0);
  await type('textarea[aria-label="Recovery phrase"]', TYPED + ' extra');
  check('editing the phrase withdraws the account shown', !(await text()).includes(RESTORED));
  check('and withdraws "That is my account"', !(await present('That is my account')));
  await type('textarea[aria-label="Recovery phrase"]', TYPED);
  await click('Check the phrase');
  await noSecret('restoring');
  await click('That is my account');
  const restore = await calls('vault_restore');
  check('"That is my account" restores once', restore.length === 1, `${restore.length} calls`);
  check(
    'with the phrase that was checked, and nothing else',
    restore[0]?.args?.phrase === NORMALISED && JSON.stringify(Object.keys(restore[0]?.args ?? {})) === '["phrase"]',
  );
  check('a restore reaches the shell', await inside());

  // --- 5. a phrase the host refuses ---------------------------------------

  await boot({ vault: 'absent', preview: 'bad' });
  await click('I already have a recovery phrase');
  await type('textarea[aria-label="Recovery phrase"]', TYPED);
  await click('Check the phrase');
  check('a phrase the host refuses is reported as a bad phrase', (await alert()) === BAD_PHRASE_SHOWN, await alert());
  check('and offers no way on', !(await present('That is my account')));
  check('Back returns to the welcome', (await click('Back')) && (await headingIs('Welcome')));

  // --- 6. a keychain vault that will not open ------------------------------

  await boot({ vault: 'locked', sealed: 'keychain', unlock: 'fail' });
  check('a vault that will not open shows why it stopped', (await headingIs('Open your vault')), await heading());
  await noSecret('a vault that will not open');
  const before = (await calls('vault_unlock')).length;
  await click('Try again');
  check('"Try again" asks the host again', (await calls('vault_unlock')).length === before + 1);
  check('and says what to do', (await alert()).toLowerCase().includes('restore the vault from your recovery phrase'), await alert());
  check(
    'the recovery phrase is the way back',
    (await click('Restore from your recovery phrase')) && (await headingIs('Enter your recovery phrase')),
  );
  check('replacing a vault says it is set aside, not deleted', (await text()).includes('set aside, not deleted'));

  // --- 7. a Windows Hello vault, one last time -----------------------------

  await boot({ vault: 'locked', sealed: 'hello', unlock: 'cancel' });
  check('a Hello vault asks one last time', (await headingIs('Windows Hello, one last time')), await heading());
  check('Hello is asked once on arrival', (await calls('vault_unlock')).length === 1);
  check('a cancel is reported', (await alert()) === CANCELLED, await alert());
  await sleep(1500);
  check('and is not asked again on its own', (await calls('vault_unlock')).length === 1);
  await noSecret('the last Windows Hello');
  await click('Use Windows Hello');
  check('pressing the button asks again', (await calls('vault_unlock')).length === 2);
  check('a cancel leaves the Gate up', !(await inside()));
  await click('Restore from your recovery phrase');
  check('Back returns', (await click('Back')) && (await headingIs('Windows Hello, one last time')));
  check('without asking Hello again on its own', (await calls('vault_unlock')).length === 2);

  await boot({ vault: 'locked', sealed: 'hello', unlock: 'arrive' });
  check('Hello approving the move reaches the shell', await inside());
  check('asked once', (await calls('vault_unlock')).length === 1);

  // --- 8. a passphrase vault, one last time --------------------------------

  await boot({ vault: 'locked', sealed: 'passphrase' });
  check('a passphrase vault asks for it one last time', (await headingIs('Your old passphrase, one last time')), await heading());
  check('Hello is not asked for it', (await calls('vault_unlock')).length === 0);
  check('which is not reported as an error', (await alert()) === '', await alert());
  check('the button is refused until the passphrase is typed', (await enabled('Open and keep it open')) === false);
  await type('input[aria-label="Your old passphrase"]', PASSPHRASE);
  await click('Open and keep it open');
  const move = await calls('vault_move_to_keychain');
  check('the host is asked to move it once, with the passphrase typed', move.length === 1 && move[0]?.args?.passphrase === PASSPHRASE);
  check('a moved vault reaches the shell', await inside());

  // --- 9. Settings ----------------------------------------------------------

  await boot({ vault: 'inside' });
  await openSettings();
  await noSecret('Settings');
  check('Settings has no Windows Hello switch', !(await text()).includes('Windows Hello'));
  await click('Show recovery phrase', 'main');
  const exported = await calls('vault_export_phrase');
  check('the host is asked once, with nothing sent', exported.length === 1 && none(exported[0]), JSON.stringify(exported));
  check('and the phrase is shown', (await text()).includes(WORDS[23]));
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
