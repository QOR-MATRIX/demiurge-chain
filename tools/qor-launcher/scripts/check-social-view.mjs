// Does the Social surface draw what Social says, and ask Social exactly what the person did? (ADR-091, L8.5)
//
// The host's tests prove the socket (src-tauri/src/social.rs: hello with the token, events, answers, refusals with
// their codes, reconnecting); Social's own tests prove its rules (services/social). This check covers what reaches
// the screen: it serves the built frontend in a real rendering engine with a stand-in host that answers
// `social_request` from a FIXTURE and delivers `social://` events, then drives the surface as a person would.
//
// What it holds the view to:
//   - Social connects as soon as the shell opens, and the surface says so while it is connecting;
//   - an account on a name QOR ID made is asked to choose its name before anything else, and the claim goes to QOR ID
//     and then reconnects Social under the new name;
//   - the conversations Social listed, direct and rooms apart, with Social's unread counts, which the rail adds up;
//   - a room's messages in order, with names, @mentions of the reader marked, a deleted message drawn without words,
//     and reactions with their counts;
//   - the composer: @ completes a member's name, Enter sends the words, the room and a retry nonce, and nothing else;
//   - a message arriving in another room raises that room's count; someone typing is shown;
//   - a profile card from a name, and the editor refusing an accent that would not read on the theme;
//   - a refusal shown in Social's own words; a session QOR ID has ended, said plainly.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome. Set BROWSER_PATH for another Chromium browser.
//
//   npm run build
//   node scripts/check-social-view.mjs [dist directory, default ./dist]
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

// --- the fixture: people, rooms and words that belong to no one ------------
const ME = '00000000-0000-4000-8000-00000000000a';
const BOB = '00000000-0000-4000-8000-00000000000b';
const CAROL = '00000000-0000-4000-8000-00000000000c';
const LOUNGE = '10000000-0000-4000-8000-000000000001';
const QUIET = '10000000-0000-4000-8000-000000000002';
const DIRECT = '10000000-0000-4000-8000-000000000003';
const T = '2026-10-15T20:00:00Z';
const profile = (sub, qor_id, extra = {}) => ({
  sub, qor_id, avatar_url: null, level: 4, display: '', bio: '', accent: null, banner: {}, showcase: [],
  status_line: '', seen_at: T, ...extra,
});
const PROFILES = {
  [ME]: profile(ME, 'fixture', { display: 'Fixture Person' }),
  [BOB]: profile(BOB, 'bob', { display: 'Bob of the Lattice', bio: 'Builds worlds in fixtures.', banner: { preset: 'lattice', hue: 180, intensity: 0.7 }, status_line: 'testing things' }),
  [CAROL]: profile(CAROL, 'carol', { display: 'Carol' }),
};
const room = (id, kind, name, extra = {}) => ({
  id, kind, name, topic: '', ambience: {}, created_at: T, role: 'member', unread: 0, last_seq: 0, members: 3, other: null, ...extra,
});
const ROOMS = [
  room(LOUNGE, 'public', 'Fixture Lounge', { unread: 2, last_seq: 4, topic: 'where fixtures meet' }),
  room(QUIET, 'private', 'Quiet Corner', { unread: 1, last_seq: 1 }),
  room(DIRECT, 'direct', '', { other: BOB, last_seq: 2, members: 2 }),
];
const message = (seq, author, body, extra = {}) => ({
  id: `20000000-0000-4000-8000-00000000000${seq}`, room: LOUNGE, seq, author, body, reply_to: null, reactions: [],
  edited_at: null, deleted: false, created_at: T, ...extra,
});
const HISTORY = [
  message(1, BOB, 'Welcome to the lounge, everyone.'),
  message(2, CAROL, null, { deleted: true }),
  message(3, BOB, 'hey @fixture, look at this', { reactions: [{ emoji: '🔥', by: [CAROL, BOB] }] }),
  message(4, CAROL, 'Lovely ambience in here.'),
];
const WELCOME = (chosen) => ({
  type: 'welcome', version: 1, me: PROFILES[ME], username_chosen: chosen, rooms: ROOMS, invites: [], blocked: [], presence: { [BOB]: { state: 'online', playing: 'Flux Four' } },
});

const results = [];
const check = (name, pass, detail = '') => {
  results.push({ name, pass });
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}${detail ? `  (${detail})` : ''}`);
};

// --- serve the build -------------------------------------------------------
const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.svg': 'image/svg+xml', '.png': 'image/png', '.json': 'application/json', '.woff2': 'font/woff2', '.ico': 'image/x-icon' };
const server = createServer(async (req, res) => {
  const path = decodeURIComponent(new URL(req.url, 'http://x').pathname);
  const file = join(dist, path === '/' ? 'index.html' : path.replace(/^\/+/, ''));
  try {
    const body = await readFile(file);
    res.writeHead(200, { 'content-type': TYPES[extname(file)] ?? 'application/octet-stream' }).end(body);
  } catch {
    res.writeHead(404).end('not found');
  }
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const appUrl = `http://127.0.0.1:${server.address().port}/`;

const userData = await mkdtemp(join(tmpdir(), 'qor-social-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
    ...(process.env.QOR_CHECK_NO_SANDBOX === '1' ? ['--no-sandbox'] : []),
    '--disable-gpu', '--no-first-run', '--no-default-browser-check',
    '--disable-backgrounding-occluded-windows', '--disable-renderer-backgrounding', '--disable-background-timer-throttling',
    `--remote-debugging-port=${port}`, `--user-data-dir=${userData}`, 'about:blank',
  ],
  { stdio: 'ignore' },
);
const finish = async () => {
  try { browser.kill(); } catch {}
  try { server.close(); } catch {}
  await rm(userData, { recursive: true, force: true }).catch(() => {});
};

try {
  let target;
  for (let i = 0; i < 300 && !target; i++) {
    if (i > 0) await sleep(100);
    try {
      target = (await (await fetch(`http://127.0.0.1:${port}/json/list`)).json()).find((t) => t.type === 'page');
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
  // Wait for what the next line reads, never for a length of time (HANDOFF.md §5).
  const eventually = async (expression, ms = 8000) => {
    const deadline = Date.now() + ms;
    for (;;) {
      if (await evaluate(expression)) return true;
      if (Date.now() > deadline) return false;
      await sleep(40);
    }
  };
  const until = async (expression, what) => {
    if (!(await eventually(expression))) throw new Error(`timed out waiting for ${what}: ${expression}`);
  };

  const ADDRESS = '5FixtureAddressForTheSocialCheck0000000000000000';
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const SESSION = { qor_id: 'key_0a1b2c3d', username: 'key_0a1b2c3d', discriminator: 1, role: 'user', address: ADDRESS, avatar_url: null, username_chosen: false };
  const STATE = {
    version: '0.0.0-fixture', vault: { state: 'unlocked', accounts: ACCOUNTS }, session: SESSION,
    chain_endpoint: 'ws://127.0.0.1:0', auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };

  const HOST_STUB = `
    window.__CALLS__ = [];
    window.__SOCIAL__ = { status: { state: 'off' }, history: ${JSON.stringify(HISTORY)}, profiles: ${JSON.stringify(PROFILES)}, refuse: null };
    window.__CALLBACKS__ = {};
    window.__LISTENERS__ = {};
    let callbacks = 0;
    window.__emit = (event, payload) => {
      for (const id of window.__LISTENERS__[event] ?? []) window.__CALLBACKS__[id]?.({ event, id, payload });
    };
    const answer = (method, params) => {
      const s = window.__SOCIAL__;
      if (s.refuse && s.refuse.method === method) return Promise.reject({ kind: 'social', code: s.refuse.code, message: s.refuse.message });
      switch (method) {
        case 'messages.history':
          return Promise.resolve({ messages: params.room === ${JSON.stringify(LOUNGE)} ? s.history : [], profiles: s.profiles });
        case 'messages.read': return Promise.resolve({ last_read: params.seq });
        case 'rooms.members':
          return Promise.resolve({ members: [{ sub: ${JSON.stringify(ME)}, role: 'member', joined_at: ${JSON.stringify(T)} },
            { sub: ${JSON.stringify(BOB)}, role: 'owner', joined_at: ${JSON.stringify(T)} },
            { sub: ${JSON.stringify(CAROL)}, role: 'member', joined_at: ${JSON.stringify(T)} }], profiles: s.profiles });
        case 'messages.send':
          return Promise.resolve({ message: { id: '30000000-0000-4000-8000-000000000001', room: params.room, seq: 5, author: ${JSON.stringify(ME)},
            body: params.body, reply_to: null, reactions: [], edited_at: null, deleted: false, created_at: ${JSON.stringify(T)} } });
        case 'profile.get': return Promise.resolve({ profiles: s.profiles });
        case 'typing': return Promise.resolve({});
        case 'dm.open': return Promise.resolve({ room: ${JSON.stringify({ id: DIRECT, kind: 'direct', name: '', topic: '', ambience: {}, created_at: T })}, other: s.profiles[${JSON.stringify(BOB)}] });
        default: return Promise.reject({ kind: 'social', code: 'bad_request', message: 'not in this fixture: ' + method });
      }
    };
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd.startsWith('social_') || cmd === 'qor_claim_username' || cmd === 'qor_username_available') window.__CALLS__.push({ cmd, args });
        if (cmd === 'plugin:event|listen') {
          (window.__LISTENERS__[args.event] ??= []).push(args.handler);
          return Promise.resolve(args.handler);
        }
        if (cmd === 'plugin:event|unlisten') return Promise.resolve(null);
        if (cmd === 'social_status') return Promise.resolve({ status: window.__SOCIAL__.status, welcome: null, endpoint: 'ws://127.0.0.1:8090/v1/socket' });
        if (cmd === 'social_connect' || cmd === 'social_disconnect') return Promise.resolve(null);
        if (cmd === 'social_request') return answer(args.method, args.params);
        if (cmd === 'qor_username_available') return Promise.resolve(args.username !== 'taken_one');
        if (cmd === 'qor_claim_username')
          return Promise.resolve({ ...window.__SESSION__, qor_id: args.username, username: args.username, username_chosen: true });
        if (cmd === 'drc369_assets') return Promise.resolve([]);
        if (cmd === 'launcher_state') return Promise.resolve(${JSON.stringify(STATE)});
        if (cmd === 'vault_status') return Promise.resolve({ state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} });
        if (cmd === 'qor_restore') return Promise.resolve(window.__SESSION__);
        if (cmd === 'touch_vault' || cmd === 'vault_lock' || cmd === 'vault_set_active') return Promise.resolve(null);
        return Promise.reject({ kind: 'internal', message: 'no host in this check' });
      },
      transformCallback: (cb) => { const id = ++callbacks; window.__CALLBACKS__[id] = cb; return id; },
      unregisterCallback: (id) => { delete window.__CALLBACKS__[id]; },
      convertFileSrc: (s) => s,
      metadata: { currentWindow: { label: 'main' }, currentWebview: { windowLabel: 'main', label: 'main' } },
      plugins: {},
    };
    window.__SESSION__ = ${JSON.stringify(SESSION)};
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
  `;

  await send('Page.enable');
  await send('Runtime.enable');
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
  await send('Page.bringToFront');
  await send('Emulation.setFocusEmulationEnabled', { enabled: true });
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell and its rail');

  const calls = (cmd) => evaluate(`window.__CALLS__.filter((c) => c.cmd === ${JSON.stringify(cmd)})`);
  const requests = (method) => evaluate(`window.__CALLS__.filter((c) => c.cmd === 'social_request' && c.args.method === ${JSON.stringify(method)}).map((c) => c.args.params)`);
  const emit = (event, payload) => evaluate(`window.__emit(${JSON.stringify(event)}, ${JSON.stringify(payload)})`);
  const text = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)})?.textContent ?? ''`);
  const click = (selector) => evaluate(`(() => { const n = document.querySelector(${JSON.stringify(selector)}); if (!n) return false; n.click(); return true; })()`);
  const clickText = (scope, label) => evaluate(`(() => { const n = [...document.querySelectorAll(${JSON.stringify(scope)})].find((b) => b.textContent.trim() === ${JSON.stringify(label)}); if (!n) return false; n.click(); return true; })()`);
  // Type into a React-controlled field: set the native value, then announce it.
  const type = (selector, value) => evaluate(`(() => {
    const el = document.querySelector(${JSON.stringify(selector)});
    const proto = el instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
    Object.getOwnPropertyDescriptor(proto, 'value').set.call(el, ${JSON.stringify(value)});
    el.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);
  const key = (selector, k) => evaluate(`document.querySelector(${JSON.stringify(selector)}).dispatchEvent(new KeyboardEvent('keydown', { key: ${JSON.stringify(k)}, bubbles: true }))`);

  // --- it connects with the shell, and says so meanwhile --------------------
  await until(`window.__CALLS__.some((c) => c.cmd === 'social_connect')`, 'Social connecting with the shell');
  check('Social connects as soon as the shell opens, before the surface is visited', (await calls('social_connect')).length >= 1);
  check('the rail offers Social', (await clickText('nav[aria-label="Primary"] button', 'Social')) === true);
  await until(`Boolean(document.querySelector('[data-social-connection]'))`, 'the connection state');
  check('while connecting, the surface says so', /Connecting/.test(await text('[data-social-connection]')));

  // --- a made name: choose one first ----------------------------------------
  await emit('social://status', { state: 'connected' });
  await emit('social://welcome', WELCOME(false));
  await until(`Boolean(document.querySelector('[data-social-claim="made"]'))`, 'the name claim');
  check('an account on a name QOR ID made is asked to choose its own first, naming the made one',
    (await text('[data-social-claim="made"]')).includes('key_0a1b2c3d'));
  check('and no conversation is shown before it has', !(await evaluate(`Boolean(document.querySelector('[data-social]'))`)));
  await type('[data-social-claim] input', 'taken_one');
  check('a taken name is said to be taken', await eventually(`/is taken/.test(document.querySelector('[data-social-claim]').textContent)`));
  await type('[data-social-claim] input', 'fresh_name');
  await until(`/yours to take/.test(document.querySelector('[data-social-claim]').textContent)`, 'the name to be free');
  const before = (await calls('social_connect')).length;
  await clickText('[data-social-claim] button', 'Claim fresh_name');
  await until(`window.__CALLS__.some((c) => c.cmd === 'qor_claim_username')`, 'the claim');
  const claimed = await calls('qor_claim_username');
  check('the claim goes to QOR ID with the name typed', claimed.length === 1 && claimed[0].args.username === 'fresh_name', JSON.stringify(claimed));
  check('then Social is told again who this is: a disconnect and a new connection',
    await eventually(`window.__CALLS__.filter((c) => c.cmd === 'social_disconnect').length >= 1 && window.__CALLS__.filter((c) => c.cmd === 'social_connect').length > ${before}`));

  // --- the conversations Social listed -------------------------------------
  await emit('social://welcome', WELCOME(true));
  await until(`document.querySelectorAll('[data-room-row]').length === 3`, 'three conversations');
  const rows = await evaluate(`[...document.querySelectorAll('nav[aria-label="Conversations"] section')].map((s) => [s.querySelector('.eyebrow').textContent, [...s.querySelectorAll('[data-room-row]')].map((r) => r.textContent.trim())])`);
  check('direct conversations and rooms are listed apart, the direct one by the other person',
    rows.some(([t, r]) => t === 'Direct' && r.length === 1 && r[0].includes('Bob of the Lattice')) && rows.some(([t, r]) => t === 'Rooms' && r.length === 2),
    JSON.stringify(rows));
  check('each room carries Social\'s unread count', (await text(`[data-room-row="${LOUNGE}"]`)).endsWith('2') && (await text(`[data-room-row="${QUIET}"]`)).endsWith('1'));
  check('and the rail adds them up', (await text('[data-social-unread]')) === '3');

  // --- a room's messages -----------------------------------------------------
  await click(`[data-room-row="${LOUNGE}"]`);
  await until(`document.querySelectorAll('[data-message]').length === 4`, 'the lounge history');
  const asked = await requests('messages.history');
  check('opening a room asks for its history once', asked.filter((p) => p.room === LOUNGE).length === 1, JSON.stringify(asked));
  const drawn = await evaluate(`[...document.querySelectorAll('[data-message]')].map((m) => m.textContent)`);
  check('messages are drawn in order, with the authors\' display lines', drawn[0].includes('Bob of the Lattice') && drawn[0].includes('Welcome to the lounge') && drawn[3].includes('Lovely ambience'), JSON.stringify(drawn));
  check('a deleted message keeps its place and none of its words', drawn[1].includes('This message was deleted.'));
  check('an @mention of the reader is marked as theirs', await evaluate(`[...document.querySelectorAll('.social-mention-me')].some((n) => n.textContent === '@fixture')`));
  check('a reaction shows its emoji and how many', await evaluate(`[...document.querySelectorAll('[data-message] button[aria-pressed]')].some((b) => b.textContent.includes('🔥') && b.textContent.includes('2'))`));
  check('reading the room clears its count', await eventually(`!/2$/.test(document.querySelector('[data-room-row="${LOUNGE}"]').textContent.trim())`));

  // --- the composer ----------------------------------------------------------
  const box = 'textarea[aria-label^="Message"]';
  await type(box, 'hello @bo');
  check('@ offers the room\'s members by name', await eventually(`[...document.querySelectorAll('[role="listbox"] [role="option"]')].some((o) => o.textContent.includes('@bob'))`));
  await key(box, 'Tab');
  check('and Tab completes the name', await eventually(`document.querySelector(${JSON.stringify(box)}).value === 'hello @bob '`));
  await type(box, 'hello @bob, the lounge looks great');
  await key(box, 'Enter');
  await until(`window.__CALLS__.some((c) => c.cmd === 'social_request' && c.args.method === 'messages.send')`, 'a send');
  const sent = (await requests('messages.send'))[0];
  check('Enter sends the words, the room and a retry nonce, and nothing else',
    sent.body === 'hello @bob, the lounge looks great' && sent.room === LOUNGE && typeof sent.nonce === 'string' && Object.keys(sent).sort().join() === 'body,nonce,room',
    JSON.stringify(sent));
  check('the sent message is drawn', await eventually(`[...document.querySelectorAll('[data-message]')].some((m) => m.textContent.includes('the lounge looks great'))`));
  check('and the box is cleared', await eventually(`document.querySelector(${JSON.stringify(box)}).value === ''`));

  // --- what arrives ----------------------------------------------------------
  await emit('social://event', { event: 'message.created', data: { message: { id: '40000000-0000-4000-8000-000000000001', room: QUIET, seq: 2, author: CAROL, body: 'psst', reply_to: null, reactions: [], edited_at: null, deleted: false, created_at: T } } });
  check('a message in another room raises that room\'s count', await eventually(`document.querySelector('[data-room-row="${QUIET}"]').textContent.trim().endsWith('2')`));
  await emit('social://event', { event: 'typing', data: { room: LOUNGE, sub: BOB } });
  check('someone typing is shown', await eventually(`/Bob of the Lattice is typing/.test(document.querySelector('section[aria-label="Conversation"]').textContent)`));

  // --- a refusal, in Social's words -----------------------------------------
  await evaluate(`window.__SOCIAL__.refuse = { method: 'messages.send', code: 'rate_limited', message: 'slow down: at most 10 of those every 10 seconds' }`);
  await type(box, 'one more');
  await key(box, 'Enter');
  check('a refusal is shown in Social\'s own words', await eventually(`document.body.textContent.includes('slow down: at most 10 of those every 10 seconds')`));
  await evaluate(`window.__SOCIAL__.refuse = null`);

  // --- profiles ----------------------------------------------------------------
  await evaluate(`[...document.querySelectorAll('[data-message] button')].find((b) => b.textContent.trim() === 'Bob of the Lattice').click()`);
  await until(`Boolean(document.querySelector('[data-profile-card]'))`, 'Bob\'s card');
  const card = await text('[data-profile-card]');
  check('a name opens its profile card: display line, QOR ID, presence and what they play, status and bio',
    card.includes('Bob of the Lattice') && card.includes('@bob') && card.includes('playing Flux Four') && card.includes('testing things') && card.includes('Builds worlds in fixtures.'), card);
  check('the card carries their banner', await evaluate(`Boolean(document.querySelector('[data-profile-card] .social-banner-lattice'))`));

  await click('button[aria-label="Your profile"]');
  await until(`(document.querySelector('[data-profile-card]')?.textContent ?? '').includes('Fixture Person')`, 'my own card');
  await clickText('[data-profile-card] button', 'Edit profile');
  await until(`Boolean(document.querySelector('[data-profile-editor]'))`, 'the editor');
  await type('[data-profile-editor] input[aria-label="Accent colour"]', '#111111');
  check('an accent that would not read on the theme is refused, and Save is held',
    await eventually(`document.querySelector('[data-profile-editor] [role="alert"]')?.textContent.includes('would not read') && [...document.querySelectorAll('[data-profile-editor] button')].find((b) => b.textContent.trim() === 'Save').disabled`));

  // --- a session QOR ID ended ---------------------------------------------------
  await evaluate(`document.querySelector('[data-profile-editor] button[aria-label="Close"]').click()`);
  await emit('social://status', { state: 'signed_out' });
  check('a session QOR ID has ended is said plainly', await eventually(`/Sign in again/.test(document.querySelector('[data-social-connection]')?.textContent ?? '')`));
} catch (error) {
  console.log(`FAIL  the check ran to its end  (${error.message.split('\n')[0]})`);
  results.push({ name: 'ran', pass: false });
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
