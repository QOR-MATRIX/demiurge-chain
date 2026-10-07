// Does the Market draw exactly what the host read from the chain? (L7.2)
//
// The host's tests prove that `src-tauri/src/chain/market.rs` arranges, windows
// and classifies listings correctly, and its live test reads a listing back
// from a real node. That says nothing about what reaches the screen. This check
// covers the other half: it serves the built frontend in a real rendering
// engine, answers `drc369_market` (and the sale commands Buy and Withdraw use)
// with a FIXTURE whose contents are known, opens the Market and reads back what
// was drawn and what was asked.
//
// What it holds the view to:
//   - every listing the host returned, drawn in the host's order, as a card
//     with its name, its price in the host's own words, its holder and what it
//     was remixed from;
//   - a listing that cannot be bought (void, or held in place by nesting) drawn
//     where it falls, with the host's reason and no Buy; the viewer's own with
//     Withdraw; a void one the viewer holds with Clear;
//   - the host's counts, the block and node it read at, the truncation notice
//     and the unreadable count, none of them worked out here;
//   - the filter, the order and the pages sent to the host as asked, the page
//     reset when the filter changes, and an older answer never drawn over a
//     newer one;
//   - Buy through the one purchase path: the host reads the asset again
//     (`drc369_sale`) and the existing dialog sends the price and fingerprint
//     the HOST gave it;
//   - the explanation of where the list comes from behind the information icon,
//     the empty state, and the error state in the host's words.
//
// The fixture's numbers are ones no arithmetic in the view could reproduce:
// prices whose display string does not follow from their Sparks, counts that
// do not follow from the cards on the page, and a purchase breakdown that is
// not a split of the card's price. A view that computed any of them would draw
// something else.
//
// Like the other view checks it deliberately opens no chain.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome. Set
// BROWSER_PATH to use another Chromium-family browser.
//
//   npm run build
//   node scripts/check-market-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails. The faults it was proven against, and the
// one it did not catch, are in tools/qor-launcher/README.md.

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

// --- the fixture -------------------------------------------------------------
// Every value is arbitrary and belongs to no real asset.
const ADDRESS = '5FixtureAddressForTheMarketCheck00000000000000000';
const SELLER = '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy';
const NESTER = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const ELSEWHERE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';
const ref = (byte, size) => ({ algo: 'BLAKE3-256', root: byte.repeat(32), size });

// The price a card shows is the host's string. Its Sparks are deliberately
// not what that string says, so a view that formatted Sparks itself would draw
// another number.
const asset = (collection, item, name, byte, revisable, seller, priceCgt, priceSparks, isVoid) => ({
  collection,
  item,
  name,
  origin: ref(byte, 100 + item),
  current: ref(byte, 100 + item),
  commit: { kind: 'SHA-1', id: byte.repeat(20) },
  revisable,
  listing: { seller, price_sparks: priceSparks, price_cgt: priceCgt, void: isVoid },
});

const BUYABLE = {
  asset: asset(7, 2, 'night-drive', '7c', true, SELLER, '4,321.09', '17', false),
  holder: SELLER,
  held_by_viewer: false,
  derived_from: null,
  standing: 'buyable',
  reason: null,
};
// Buyable, a remix, and the one whose purchase cannot be opened (the host's
// second read of it fails).
const REMIX = {
  asset: asset(2, 7, 'remix-of-the-hook', '2a', false, ELSEWHERE, '0.000000000000000003', '999', false),
  holder: ELSEWHERE,
  held_by_viewer: false,
  derived_from: { collection: 1, item: 1 },
  standing: 'buyable',
  reason: null,
};
const NESTED_REASON =
  'Nobody can buy this asset as it stands. It is nested inside asset 4/9, and the chain will not move it until its holder takes it out.';
const NESTED = {
  asset: asset(3, 9, 'nested-stem', '39', true, NESTER, '88.80', '1', false),
  holder: NESTER,
  held_by_viewer: false,
  derived_from: null,
  standing: 'held_in_place',
  reason: NESTED_REASON,
};
const MINE = {
  asset: asset(5, 1, 'my-loop', '51', true, ADDRESS, '17.25', '4', false),
  holder: ADDRESS,
  held_by_viewer: true,
  derived_from: null,
  standing: 'yours',
  reason: null,
};
const VOID_MINE_REASON =
  'A listing made by an earlier holder of this asset is still on chain. It is void: nobody can buy from it. You hold the asset, and you can clear it.';
const VOID_MINE = {
  asset: asset(6, 0, 'handed-on', '60', false, SELLER, '900.00', '5', true),
  holder: ADDRESS,
  held_by_viewer: true,
  derived_from: null,
  standing: 'void',
  reason: VOID_MINE_REASON,
};
const VOID_OTHER_REASON =
  'This listing is void: the account that listed the asset no longer holds it, so nobody can buy it.';
const VOID_OTHER = {
  asset: asset(6, 4, 'gone-elsewhere', '64', false, SELLER, '12.00', '6', true),
  holder: ELSEWHERE,
  held_by_viewer: false,
  derived_from: null,
  standing: 'void',
  reason: VOID_OTHER_REASON,
};
// The viewer's own, held in place: theirs, with the reason beside it.
const MINE_NESTED_REASON =
  'Nobody can buy this while it stays as it is. It holds 2 assets nested inside it, and the chain will not move it until you take them out.';
const MINE_NESTED = {
  asset: asset(5, 3, 'my-bundle', '53', false, ADDRESS, '61.61', '8', false),
  holder: ADDRESS,
  held_by_viewer: true,
  derived_from: null,
  standing: 'yours',
  reason: MINE_NESTED_REASON,
};
const SECOND_PAGE_ONE = {
  asset: asset(8, 8, 'second-page', '88', true, SELLER, '1,000,000.00', '9', false),
  holder: SELLER,
  held_by_viewer: false,
  derived_from: null,
  standing: 'buyable',
  reason: null,
};

const BLOCK = { block_number: 81_234, block_hash: '0x' + 'ab'.repeat(32), endpoint: 'ws://fixture-node:9944', chain_name: 'Demiurge Fixture' };
// The first window of everyone's listings. In NO order the view could produce
// by sorting: not by price, not by number, not by name. 500 read, the bound
// reached, two of them the viewer's (only one is on this page), and one in
// this window unreadable.
const ALL_FIRST = [NESTED, BUYABLE, VOID_OTHER, MINE, REMIX, VOID_MINE];
const page = (listings, extra) => ({
  listings,
  offset: 0,
  matching: 500,
  on_chain: 500,
  yours: 2,
  truncated: true,
  bound: 500,
  unreadable: 1,
  ...BLOCK,
  ...extra,
});
const PAGES = {
  all_0: page(ALL_FIRST),
  all_24: page([SECOND_PAGE_ONE], { offset: 24, unreadable: 0 }),
  // Dearest first: the host's order, again not one the view could derive.
  high_0: page([REMIX, VOID_MINE, BUYABLE, MINE, NESTED, VOID_OTHER]),
  yours_0: page([MINE_NESTED, MINE], { matching: 2, unreadable: 0 }),
  others_0: page([BUYABLE, NESTED, VOID_OTHER, REMIX, VOID_MINE], { matching: 498, unreadable: 3 }),
  empty: page([], { matching: 0, on_chain: 0, yours: 0, truncated: false, unreadable: 0 }),
};

// What Buy reads again for 7/2: the price moved since the Market was read, and
// the breakdown is the host's (no arithmetic on 4,321.09 or 4,500.00 gives it).
const sparks = (cgt) => `${cgt.replace(/[,.]/g, '')}0000000000000000`;
const payout = (kind, address, share, cgt) => ({
  kind, address, share, amount_sparks: sparks(cgt), amount_cgt: cgt, to_buyer: false,
});
const SALE = {
  asset: { ...BUYABLE.asset, listing: { seller: SELLER, price_sparks: sparks('4,500.00'), price_cgt: '4,500.00', void: false } },
  holder: SELLER,
  held_by_viewer: false,
  derived_from: null,
  terms: { recipients: [{ address: NESTER, share: '7.25%' }], remix: '3%' },
  source_terms: null,
  breakdown: {
    price_sparks: sparks('4,500.00'),
    price_cgt: '4,500.00',
    source: null,
    source_share: null,
    payouts: [payout('royalty', NESTER, '7.25%', '311.11'), payout('seller', SELLER, null, '4,188.89')],
    blocked: null,
  },
  viewer_free_cgt: '9,876.54',
  cannot_buy: null,
  pasted_root_matches: null,
};
const RECEIPT = {
  collection: 7, item: 2, name: 'night-drive', seller: SELLER, buyer: ADDRESS,
  price_sparks: SALE.breakdown.price_sparks, price_cgt: '4,500.00',
  payouts: SALE.breakdown.payouts, tx_hash: '0x07', block_hash: '0x08',
};
const OUTAGE = 'the node did not answer: fixture outage while reading 2/7';
const NO_RUNTIME =
  'this node could not list what is for sale (it may be running a runtime older than selling): fixture storage error';
const UNREACHABLE = 'could not reach ws://fixture-node:9944: fixture connection refused';

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

const profile = await mkdtemp(join(tmpdir(), 'qor-market-'));
const port = 9800 + Math.floor(Math.random() * 400);
const browser = spawn(
  BROWSER,
  [
    '--headless=new',
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
  // About 30 s: a CI runner has taken more than 10 s to give Chrome its first page (6 October 2026).
  for (let i = 0; i < 300 && !target; i++) {
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

  // A stand-in host. `drc369_market` answers from PAGES by what it was asked;
  // `__MARKET_MODE__` turns it into an empty chain, a refusal, or an answer
  // that is held back until the check releases it.
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const SESSION = {
    qor_id: 'fixture-qor-id', username: 'fixture', discriminator: 1,
    role: 'user', address: ADDRESS, avatar_url: null,
  };
  const STATE = {
    version: '0.0.0-fixture',
    vault: { state: 'unlocked', accounts: ACCOUNTS },
    session: SESSION,
    chain_endpoint: 'ws://127.0.0.1:0',
    auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };
  const HOST_STUB = `
    window.__PAGES__ = ${JSON.stringify(PAGES)};
    window.__MARKET_CALLS__ = [];
    window.__MARKET_MODE__ = 'normal';
    window.__HELD__ = [];
    window.__SALE_CALLS__ = [];
    window.__BUY_CALLS__ = [];
    window.__UNLIST_CALLS__ = [];
    window.__LAUNCHER_STATE__ = ${JSON.stringify(STATE)};
    window.__ACCOUNTS__ = ${JSON.stringify(ACCOUNTS)};
    window.__SESSION__ = ${JSON.stringify(SESSION)};
    const answerFor = (args) => {
      const key = args.order === 'price_high' ? 'high_' + args.offset : args.show + '_' + args.offset;
      return window.__PAGES__[key] ?? { ...window.__PAGES__.empty, offset: args.offset };
    };
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd === 'drc369_market') {
          window.__MARKET_CALLS__.push(args);
          const mode = window.__MARKET_MODE__;
          if (mode === 'empty') return Promise.resolve(window.__PAGES__.empty);
          if (mode === 'fail') return Promise.reject({ kind: 'rpc', message: ${JSON.stringify(NO_RUNTIME)} });
          if (mode === 'unreachable') return Promise.reject({ kind: 'network', message: ${JSON.stringify(UNREACHABLE)} });
          // Held back: answered only when the check releases it, after a
          // newer look has been asked for and answered.
          if (mode === 'hold-others' && args.show === 'others')
            return new Promise((answer) => {
              window.__RELEASE__ = () => answer(answerFor(args));
            });
          return Promise.resolve(answerFor(args));
        }
        if (cmd === 'drc369_sale') {
          window.__SALE_CALLS__.push(args);
          if (args.asset === '2/7') return Promise.reject({ kind: 'rpc', message: ${JSON.stringify(OUTAGE)} });
          return Promise.resolve(${JSON.stringify(SALE)});
        }
        if (cmd === 'drc369_buy') {
          window.__BUY_CALLS__.push(args);
          return Promise.resolve(${JSON.stringify(RECEIPT)});
        }
        if (cmd === 'drc369_unlist') {
          window.__UNLIST_CALLS__.push(args);
          return Promise.resolve({ collection: args.collection, item: args.item, tx_hash: '0x09', block_hash: '0x0a' });
        }
        if (cmd === 'drc369_assets') return Promise.resolve(window.__HELD__);
        if (cmd === 'cgt_history') return Promise.reject({ kind: 'rpc', message: 'no history (ADR-028)' });
        if (cmd === 'launcher_state') return Promise.resolve(window.__LAUNCHER_STATE__);
        if (cmd === 'vault_status')
          return Promise.resolve({ state: 'unlocked', accounts: window.__ACCOUNTS__ });
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
  await send('Page.bringToFront');
  await send('Emulation.setFocusEmulationEnabled', { enabled: true });
  await until(`Boolean(document.querySelector('nav[aria-label="Primary"]'))`, 'the shell and its rail');

  // --- reached from the Nexus, and from the rail ---------------------------
  await until(`Boolean(document.querySelector('main button[aria-label^="Market: "]'))`, 'the Market tile in the Nexus');
  await evaluate(`document.querySelector('main button[aria-label^="Market: "]').click()`);
  check('the Nexus has a Market tile and it opens the Market',
    await eventually(`document.querySelector('main h1')?.textContent.trim() === 'Market'`));

  const railOpens = (label) => `(() => {
    const b = [...document.querySelectorAll('nav[aria-label="Primary"] button')]
      .find((n) => n.textContent.trim() === ${JSON.stringify(label)});
    if (!b) return false;
    b.click();
    return true;
  })()`;
  check('the rail has an Inventory button', (await evaluate(railOpens('Inventory'))) === true);
  await until(`document.querySelector('main h1')?.textContent.trim() === 'Inventory'`, 'the Inventory');
  const callsBeforeRail = await evaluate(`window.__MARKET_CALLS__.length`);
  check('the rail has a Market button', (await evaluate(railOpens('Market'))) === true);
  await until(`document.querySelector('main h1')?.textContent.trim() === 'Market'`, 'the Market from the rail');
  check('and it marks the Market as the current surface',
    (await evaluate(`[...document.querySelectorAll('nav[aria-label="Primary"] button')].find((n) => n.textContent.trim() === 'Market').getAttribute('aria-current')`)) === 'page');
  await until(`document.querySelectorAll('main [data-market-asset]').length === ${ALL_FIRST.length}`, 'the first page of listings');

  // --- what was asked ------------------------------------------------------
  const lastCall = () => evaluate(`window.__MARKET_CALLS__[window.__MARKET_CALLS__.length - 1]`);
  const first = await lastCall();
  check('opening it asks the host once more', (await evaluate(`window.__MARKET_CALLS__.length`)) === callsBeforeRail + 1);
  check('the first look asks for everyone\'s listings, cheapest first, from the start, one page, as this account',
    first.viewer === ADDRESS && first.show === 'all' && first.order === 'price_low' && first.offset === 0 && first.limit === 24,
    JSON.stringify(first));

  // --- what was drawn ------------------------------------------------------
  const drawn = () => evaluate(`[...document.querySelectorAll('main [data-market-asset]')].map((card) => ({
    id: card.getAttribute('data-market-asset'),
    standing: card.getAttribute('data-market-standing'),
    name: card.querySelector('[data-asset-name]')?.textContent.trim() ?? '',
    price: card.querySelector('[data-asset-price]')?.textContent.replace(/\\s+/g, ' ').trim() ?? null,
    holder: card.querySelector('[data-market-holder]')?.textContent.trim() ?? null,
    holderTitle: card.querySelector('[data-market-holder]')?.getAttribute('title') ?? null,
    seller: card.querySelector('[data-market-seller]')?.getAttribute('title') ?? null,
    source: card.querySelector('[data-market-source]')?.textContent.trim() ?? null,
    reason: card.querySelector('[data-market-reason]')?.textContent.replace(/\\s+/g, ' ').trim() ?? null,
    yours: Boolean(card.querySelector('[data-market-yours]')),
    buy: card.querySelectorAll('[data-market-buy]').length,
    withdraw: card.querySelector('[data-market-withdraw]')?.textContent.trim() ?? null,
    menu: Boolean(card.querySelector('[data-asset-more]')),
    root: card.querySelector('[data-asset-root]')?.textContent.trim() ?? '',
  }))`);
  const ids = (list) => list.map((l) => `${l.asset.collection}/${l.asset.item}`);

  const assertCards = async (fixture, when) => {
    const cards = await drawn();
    check(`${when}: one card per listing the host returned, in the host's order`,
      JSON.stringify(cards.map((c) => c.id)) === JSON.stringify(ids(fixture)),
      cards.map((c) => c.id).join(' '));
    for (const listing of fixture) {
      const id = `${listing.asset.collection}/${listing.asset.item}`;
      const card = cards.find((c) => c.id === id);
      if (!card) { check(`${when}: ${id} is drawn`, false); continue; }
      const live = !listing.asset.listing.void;
      const okName = card.name === listing.asset.name;
      const okStanding = card.standing === listing.standing;
      const okPrice = live ? card.price === `${listing.asset.listing.price_cgt} CGT` : card.price === null;
      const okHolder = listing.held_by_viewer
        ? card.holder === 'This account'
        : card.holderTitle === listing.holder && card.holder !== 'This account';
      const okSeller = live ? card.seller === null : card.seller === listing.asset.listing.seller;
      const okSource = listing.derived_from
        ? card.source === `Asset ${listing.derived_from.collection}/${listing.derived_from.item}`
        : card.source === null;
      const okReason = card.reason === listing.reason;
      const okRoot = card.root === `${listing.asset.current.algo} ${listing.asset.current.root}`;
      check(`${when}: ${id} shows its name, standing, reference, holder and source as the host gave them`,
        okName && okStanding && okRoot && okHolder && okSeller && okSource,
        JSON.stringify({ okName, okStanding, okRoot, okHolder, okSeller, okSource }));
      check(`${when}: ${id} ${live ? `shows the host's price, ${listing.asset.listing.price_cgt} CGT` : 'is void and shows no price anyone could pay'}`,
        okPrice, String(card.price));
      check(`${when}: ${id} ${listing.reason ? 'says why it cannot be bought, in the host\'s words' : 'carries no reason'}`,
        okReason, String(card.reason));
      const wantBuy = listing.standing === 'buyable' ? 1 : 0;
      const wantWithdraw = listing.standing === 'yours'
        ? 'Withdraw'
        : listing.standing === 'void' && listing.held_by_viewer ? 'Clear void listing' : null;
      check(`${when}: ${id} offers ${wantBuy ? 'Buy' : 'no Buy'} and ${wantWithdraw ?? 'nothing to withdraw'}`,
        card.buy === wantBuy && card.withdraw === wantWithdraw && card.menu === false,
        JSON.stringify({ buy: card.buy, withdraw: card.withdraw, menu: card.menu }));
      check(`${when}: ${id} ${listing.standing === 'yours' ? 'is marked as yours' : 'is not marked as yours'}`,
        card.yours === (listing.standing === 'yours'));
    }
  };

  await assertCards(ALL_FIRST, 'everyone\'s');

  const text = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)})?.textContent.replace(/\\s+/g, ' ').trim() ?? null`);
  const counts = await text('[data-market-counts]');
  check('the counts are the host\'s: where this page is among those that match',
    counts?.includes('Listings 1–24 of 500'), counts);
  check('and how many were read, and how many are this account\'s, by the host\'s count',
    counts?.includes('500 read from the chain, 2 of them yours'), counts);
  const source = await text('[data-market-source]');
  check('it names the finalised block, the chain and the node it was read through',
    source?.includes('81234') && source.includes('Demiurge Fixture') && source.includes('ws://fixture-node:9944'), source);
  const truncated = await text('[data-market-truncated]');
  check('the chain holds more than one look reads: it says so, with the bound, and names the indexer',
    truncated !== null && truncated.includes('more than 500 listings') && /indexer/.test(truncated) && /M5\.4/.test(truncated),
    truncated);
  const unreadable = await text('[data-market-unreadable]');
  check('a listing whose asset cannot be read is counted and said, not silently dropped',
    unreadable !== null && unreadable.startsWith('1 listing on this page'), unreadable);

  // Where the list comes from: behind the information icon, in the document.
  const tip = await evaluate(`(() => {
    const icon = document.querySelector('main header [data-infotip]');
    const id = icon && icon.getAttribute('aria-describedby');
    const bubble = id && document.getElementById(id);
    return bubble ? bubble.textContent : null;
  })()`);
  check('the information icon says where the list comes from: the connected node, at its finalised block',
    tip !== null && /node you are connected to/.test(tip) && /finalised block/.test(tip), String(tip));
  check('and what it is not: no indexer, no search, no history, nothing by newest',
    tip !== null && /no indexer/.test(tip) && /no search/.test(tip) && /no history/.test(tip) && /newest/.test(tip), String(tip));

  // The controls are the host's, and only the host's.
  const controls = await evaluate(`({
    show: [...document.querySelectorAll('main [data-market-show]')].map((b) => [b.getAttribute('data-market-show'), b.getAttribute('aria-pressed')]),
    order: [...document.querySelectorAll('main [data-market-order]')].map((b) => [b.getAttribute('data-market-order'), b.getAttribute('aria-pressed')]),
    newest: /newest/i.test([...document.querySelectorAll('main button')].map((b) => b.textContent).join(' ')),
  })`);
  check('the filter offers exactly the host\'s three, everyone\'s pressed',
    JSON.stringify(controls.show) === JSON.stringify([['all', 'true'], ['others', 'false'], ['yours', 'false']]),
    JSON.stringify(controls.show));
  check('the order offers exactly the host\'s three, cheapest first pressed, and no "newest"',
    JSON.stringify(controls.order) === JSON.stringify([['price_low', 'true'], ['price_high', 'false'], ['number', 'false']]) && !controls.newest,
    JSON.stringify(controls.order));

  // Every class the surface uses must exist in the stylesheet (see
  // check-projects-view.mjs for why).
  const undefinedClasses = () => evaluate(`(() => {
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
    return [...used]
      .filter((c) => !c.startsWith('lucide'))
      .filter((c) => !all.includes('.' + CSS.escape(c)))
      .sort();
  })()`);
  const undefinedOnTheSurface = await undefinedClasses();
  check('every class the surface uses is defined by the stylesheet',
    undefinedOnTheSurface.length === 0, undefinedOnTheSurface.join(' '));

  // The close-up carries the holder in full.
  await evaluate(`document.querySelector('main [data-market-asset="3/9"] [data-asset-name]').click()`);
  await until(`Boolean(document.querySelector('[data-asset-closeup] [data-closeup-holder]'))`, 'the close-up of a listing');
  check('a listing opens at size, with its holder in full',
    (await text('[data-asset-closeup] [data-closeup-holder]')) === NESTER);
  await evaluate(`document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
  await until(`!document.querySelector('[data-asset-closeup]')`, 'the close-up to close');

  // --- pages ------------------------------------------------------------------
  const pages = await evaluate(`({
    previous: document.querySelector('main [data-market-previous]')?.disabled ?? null,
    next: document.querySelector('main [data-market-next]')?.disabled ?? null,
  })`);
  check('there are pages, and on the first one only Next is offered',
    pages.previous === true && pages.next === false, JSON.stringify(pages));
  await evaluate(`document.querySelector('main [data-market-next]').click()`);
  await until(`document.querySelector('main [data-market-asset="8/8"]') !== null`, 'the second page');
  let call = await lastCall();
  check('Next asks for the next window: the host\'s page size on from where this one started',
    call.offset === 24 && call.limit === 24 && call.show === 'all' && call.order === 'price_low', JSON.stringify(call));
  await assertCards(PAGES.all_24.listings, 'the second page');
  check('and the counts say where it is',
    (await text('[data-market-counts]'))?.includes('Listings 25–48 of 500'), await text('[data-market-counts]'));
  check('nothing unreadable on this page, so nothing is said about it',
    (await text('[data-market-unreadable]')) === null);
  check('on the second page Previous is offered',
    (await evaluate(`document.querySelector('main [data-market-previous]').disabled`)) === false);
  await evaluate(`document.querySelector('main [data-market-previous]').click()`);
  await until(`document.querySelector('main [data-market-asset="3/9"]') !== null`, 'the first page again');
  call = await lastCall();
  check('Previous asks for the window before this one', call.offset === 0 && call.show === 'all', JSON.stringify(call));
  await evaluate(`document.querySelector('main [data-market-next]').click()`);
  await until(`document.querySelector('main [data-market-asset="8/8"]') !== null`, 'the second page again');

  // A filter changed from the second page starts again from the first.
  await evaluate(`document.querySelector('main [data-market-show="yours"]').click()`);
  await until(`document.querySelector('main [data-market-asset="5/3"]') !== null`, 'this account\'s listings');
  call = await lastCall();
  check('the filter is sent to the host, and a new filter starts from the first page',
    call.show === 'yours' && call.offset === 0 && call.order === 'price_low', JSON.stringify(call));
  await assertCards(PAGES.yours_0.listings, 'yours');
  check('the filter pressed is the one asked for',
    (await evaluate(`document.querySelector('main [data-market-show="yours"]').getAttribute('aria-pressed')`)) === 'true');
  check('with everything on one page there are no pages to turn',
    (await evaluate(`Boolean(document.querySelector('main [data-market-pages]'))`)) === false);

  // An older answer never overwrites a newer one: "Others'" is asked for and
  // held back, "Yours" is asked for again and answered, then the old one comes.
  await evaluate(`window.__MARKET_MODE__ = 'hold-others'`);
  await evaluate(`document.querySelector('main [data-market-show="others"]').click()`);
  await until(`typeof window.__RELEASE__ === 'function'`, 'the held look to be asked');
  await evaluate(`document.querySelector('main [data-market-show="yours"]').click()`);
  await until(`window.__MARKET_CALLS__[window.__MARKET_CALLS__.length - 1].show === 'yours'`, 'the newer look');
  await sleep(150);
  await evaluate(`window.__RELEASE__()`);
  await sleep(400);
  check('an answer to an older look is dropped: the newer one stays drawn',
    JSON.stringify((await drawn()).map((c) => c.id)) === JSON.stringify(ids(PAGES.yours_0.listings)),
    (await drawn()).map((c) => c.id).join(' '));
  await evaluate(`window.__MARKET_MODE__ = 'normal'`);

  // Order: sent to the host, and the host's answer drawn as it came.
  await evaluate(`document.querySelector('main [data-market-show="all"]').click()`);
  await until(`window.__MARKET_CALLS__[window.__MARKET_CALLS__.length - 1].show === 'all'`, 'everyone\'s again');
  await evaluate(`document.querySelector('main [data-market-order="price_high"]').click()`);
  await until(`window.__MARKET_CALLS__[window.__MARKET_CALLS__.length - 1].order === 'price_high'`, 'dearest first to be asked');
  await until(`document.querySelector('main [data-market-asset]')?.getAttribute('data-market-asset') === '2/7'`, 'the dearest-first answer');
  call = await lastCall();
  check('the order is sent to the host, from the first page',
    call.order === 'price_high' && call.show === 'all' && call.offset === 0, JSON.stringify(call));
  await assertCards(PAGES.high_0.listings, 'dearest first');
  await evaluate(`document.querySelector('main [data-market-order="price_low"]').click()`);
  await until(`document.querySelector('main [data-market-asset]')?.getAttribute('data-market-asset') === '3/9'`, 'cheapest first again');

  // --- Buy, through the one purchase path ---------------------------------
  // A purchase that cannot be opened: the host's words, under the controls.
  const marketCallsBeforeProblem = await evaluate(`window.__MARKET_CALLS__.length`);
  await evaluate(`document.querySelector('main [data-market-asset="2/7"] [data-market-buy]').click()`);
  await until(`Boolean(document.querySelector('main [data-market-problem]'))`, 'the problem opening a purchase');
  const problem = await text('main [data-market-problem]');
  check('when the asset cannot be read again, it says so in the host\'s words, and opens nothing',
    problem?.includes(OUTAGE) && problem.includes('2/7') && !(await evaluate(`Boolean(document.querySelector('[data-buy-dialog]'))`)),
    problem);
  check('and the Market is read again', await eventually(`window.__MARKET_CALLS__.length > ${marketCallsBeforeProblem}`));

  const salesBefore = await evaluate(`window.__SALE_CALLS__.length`);
  await evaluate(`document.querySelector('main [data-market-asset="7/2"] [data-market-buy]').click()`);
  await until(`Boolean(document.querySelector('[data-buy-dialog] [data-breakdown]'))`, 'the purchase dialog');
  const saleCall = await evaluate(`window.__SALE_CALLS__[window.__SALE_CALLS__.length - 1]`);
  check('Buy asks the host for the asset again, by its number, as this account',
    (await evaluate(`window.__SALE_CALLS__.length`)) === salesBefore + 1 && saleCall.asset === '7/2' && saleCall.viewer === ADDRESS,
    JSON.stringify(saleCall));
  check('the problem from before is gone once a purchase opens',
    (await text('main [data-market-problem]')) === null);
  const dialog = await evaluate(`(() => {
    const d = document.querySelector('[data-buy-dialog]');
    return {
      confirm: d.querySelector('[data-buy-confirm]').textContent.trim(),
      text: d.textContent.replace(/\\s+/g, ' '),
    };
  })()`);
  check('the one purchase dialog opens, priced as the host read it now, not as the card said',
    dialog.confirm === 'Buy for 4,500.00 CGT', dialog.confirm);
  check('and it shows the host\'s breakdown, part for part',
    dialog.text.includes('311.11') && dialog.text.includes('4,188.89'), dialog.text.slice(0, 200));
  check('the person is told the price moved since the list was read',
    await eventually(`document.body.textContent.includes('The price changed since this list was read: it is 4,500.00 CGT now.')`));

  const marketBeforeBuy = await evaluate(`window.__MARKET_CALLS__.length`);
  await evaluate(`document.querySelector('[data-buy-dialog] [data-buy-confirm]').click()`);
  await until(`!document.querySelector('[data-buy-dialog]')`, 'the purchase dialog to close');
  const buy = await evaluate(`window.__BUY_CALLS__`);
  check('buying sends one purchase, with the price and the fingerprint the host gave the dialog',
    buy.length === 1 && buy[0].from === ADDRESS && buy[0].collection === 7 && buy[0].item === 2 &&
      buy[0].priceSparks === SALE.breakdown.price_sparks && buy[0].root === SALE.asset.current.root,
    JSON.stringify(buy));
  check('the person is told it is theirs, at the price the chain settled',
    await eventually(`document.body.textContent.includes('Bought "night-drive" for 4,500.00 CGT.')`));
  check('and the Market is read from the chain again, not adjusted here',
    await eventually(`window.__MARKET_CALLS__.length > ${marketBeforeBuy}`));

  // --- Withdraw, and clearing a void listing --------------------------------
  await until(`Boolean(document.querySelector('main [data-market-asset="5/1"] [data-market-withdraw]'))`, 'the viewer\'s own listing');
  const marketBeforeWithdraw = await evaluate(`window.__MARKET_CALLS__.length`);
  await evaluate(`document.querySelector('main [data-market-asset="5/1"] [data-market-withdraw]').click()`);
  await until(`window.__UNLIST_CALLS__.length === 1`, 'the withdrawal');
  const unlist = await evaluate(`window.__UNLIST_CALLS__[0]`);
  check('Withdraw sends this account\'s withdrawal of that listing',
    unlist.from === ADDRESS && unlist.collection === 5 && unlist.item === 1, JSON.stringify(unlist));
  check('and the Market is read again afterwards',
    await eventually(`window.__MARKET_CALLS__.length > ${marketBeforeWithdraw}`));
  await until(`Boolean(document.querySelector('main [data-market-asset="6/0"] [data-market-withdraw]'))`, 'the void listing the viewer holds');
  await evaluate(`document.querySelector('main [data-market-asset="6/0"] [data-market-withdraw]').click()`);
  await until(`window.__UNLIST_CALLS__.length === 2`, 'the clearing');
  const clear = await evaluate(`window.__UNLIST_CALLS__[1]`);
  check('Clear void listing sends the clearing of that listing, by the account that holds it',
    clear.from === ADDRESS && clear.collection === 6 && clear.item === 0, JSON.stringify(clear));
  check('and says so', await eventually(`document.body.textContent.includes('The void listing is cleared.')`));

  // --- empty, and failed ----------------------------------------------------
  await evaluate(`window.__MARKET_MODE__ = 'empty'`);
  await evaluate(`document.querySelector('main [data-market-refresh]').click()`);
  await until(`Boolean(document.querySelector('main [data-market-empty="empty"]'))`, 'the empty Market');
  check('an empty chain: no cards, and it says nothing is listed',
    (await drawn()).length === 0 && /Nothing is listed for sale/.test(await text('main [data-market-empty]')),
    await text('main [data-market-empty]'));
  check('and no truncation notice when the walk was not cut short',
    (await text('[data-market-truncated]')) === null);
  await evaluate(`document.querySelector('main [data-market-show="yours"]').click()`);
  await until(`/nothing listed/.test(document.querySelector('main [data-market-empty]')?.textContent ?? '')`, 'the empty Yours');
  check('nothing of yours: it says how to list something',
    /Inventory/.test(await text('main [data-market-empty]')), await text('main [data-market-empty]'));

  await evaluate(`window.__MARKET_MODE__ = 'fail'`);
  await evaluate(`document.querySelector('main [data-market-refresh]').click()`);
  await until(`Boolean(document.querySelector('main [data-market-empty="failed"]'))`, 'the failed Market');
  const failed = await text('main [data-market-empty]');
  check('when the chain cannot be read: no cards, and the host\'s words',
    (await drawn()).length === 0 && failed.includes(NO_RUNTIME), failed);
  check('and it does not claim that nothing is listed', !/nothing listed|Nothing is listed/.test(failed), failed);

  await evaluate(`window.__MARKET_MODE__ = 'unreachable'`);
  await evaluate(`document.querySelector('main [data-market-refresh]').click()`);
  await until(`document.querySelector('main [data-market-empty="failed"]')?.textContent.includes('fixture connection refused')`, 'the unreachable node');
  check('a node that cannot be reached is said in the host\'s words too, not only in a friendly line',
    (await text('main [data-market-empty]')).includes(UNREACHABLE), await text('main [data-market-empty]'));
} finally {
  await finish();
}

const failedCount = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failedCount} passed, ${failedCount} failed`);
process.exit(failedCount === 0 ? 0 : 1);
