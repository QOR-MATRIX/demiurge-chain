// Does the Inventory show exactly the assets the chain holds for this account? (M4.1)
//
// The host's live test proves that `src-tauri/src/chain/assets.rs` reads a real
// chain correctly: a mint against a development node finalises and appears in
// the owner's enumeration, read from storage. That says nothing about what
// reaches the screen. This check covers the other half: it serves the built
// frontend in a real rendering engine, answers the `drc369_*` commands with a
// FIXTURE whose contents are known, opens the Inventory and reads back what was
// drawn.
//
// What it is for: for every asset, the project name, the content reference the
// chain holds NOW (not the one it was minted with, if it was revised), the
// commit it pins and whether it is permanent — each from the host and nothing
// else. Since the assets became cards (QFX layer two), the face carries identity
// and the close-up carries the rest, so this opens each card and reads that too.
// It also drives a trade (L4.4, L4.5): the menu opened by the keyboard and by the
// pointer's secondary button, the assets chosen, the warning before anything is
// sent, and what reaches the host.
// And it drives a sale from both sides (L4.6): a price typed and what the host
// says a sale at it pays, drawn amount for amount; a listing declined and then
// approved; the card's price taken from the chain's answer; a withdrawal; and a
// purchase of an asset looked up by its number, refused once and then settled.
// The fixture's amounts are ones no arithmetic on the price reproduces, so a
// view that worked a split out for itself could not draw them.
// And after "Make permanent", the view must draw the chain's next answer,
// not flip a flag it believes: the host's second answer carries an asset the
// first did not, which an optimistic view cannot show.
//
// Like check-projects-view.mjs it deliberately opens no chain. Checking the view
// against the same storage the host reads would pass if both were wrong
// together.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome. Set
// BROWSER_PATH to use another Chromium-family browser.
//
//   npm run build
//   node scripts/check-inventory-view.mjs [dist directory, default ./dist]
//
// Exits non-zero if any check fails.
//
// Proven to fail, on 22 September 2026, before it was trusted, by three faults
// injected into the VIEW (a fault in the fixture's own values moves both sides
// of every assertion together and proves nothing; check-projects-view.mjs
// records how that was learnt):
//
//   1. Inventory.tsx drew the reference an asset was MINTED with in place of the
//      one it carries now.
//   2. After "Make permanent", Inventory.tsx flipped the asset to permanent in
//      its own state instead of asking the host again.
//   3. Inventory.tsx labelled every asset "Permanent" whatever the host said.
//
// Each made this check fail; the commit that added it records the counts.
//
// The selling and buying half was proven the same way on 1 October 2026; the
// faults and what each made fail are in tools/qor-launcher/README.md.

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

// The fixture. Every value is arbitrary and belongs to no real asset.
const ADDRESS = '5FixtureAddressForTheInventoryCheck0000000000000';
const ref = (byte, size) => ({ algo: 'BLAKE3-256', root: byte.repeat(32), size });

const FRESH = {
  collection: 4,
  item: 0,
  name: 'first-song',
  origin: ref('1f', 211),
  current: ref('1f', 211),
  commit: { kind: 'SHA-1', id: 'a1'.repeat(20) },
  revisable: true,
  listing: null,
};
// Revised once, then made permanent: what it carries now differs from what it
// was minted with, and the view must show the former as its reference.
const REVISED = {
  collection: 4,
  item: 1,
  name: 'revised-track',
  origin: ref('2e', 305),
  current: ref('3d', 377),
  commit: { kind: 'SHA-1', id: 'b2'.repeat(20) },
  revisable: false,
  listing: null,
};
const BEFORE = [FRESH, REVISED];
// The host's answer AFTER "Make permanent": the first asset is permanent, and
// one that was minted meanwhile has arrived. An optimistic view cannot know it.
const ARRIVED = {
  collection: 4,
  item: 2,
  name: 'arrived-meanwhile',
  origin: ref('4c', 99),
  current: ref('4c', 99),
  commit: { kind: 'SHA-1', id: 'c3'.repeat(20) },
  revisable: true,
  listing: null,
};
const AFTER = [{ ...FRESH, revisable: false }, REVISED, ARRIVED];
// After a trade of REVISED and ARRIVED, the host answers with what is left. A
// view that removed the cards itself would show the same thing, which is why
// the check also counts the calls and reads the arguments.
const TRADED = [{ ...FRESH, revisable: false }];
const RECIPIENT = '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty';
const VOCABULARY = [
  {
    id: 'gaming',
    name: 'Gaming',
    note: null,
    fields: [
      { id: 'kind', label: 'What it is', options: ['Game', 'Tool'] },
      { id: 'players', label: 'Players', options: ['One', 'Online together'] },
    ],
  },
  {
    id: 'physical',
    name: 'Physical (offline)',
    note: 'The chain moves the record, not the object.',
    fields: [{ id: 'arrives', label: 'What actually arrives', options: [] }],
  },
];
const PARTNER = {
  address: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy',
  label: null,
  last_traded: 1_758_600_000,
  trades: 3,
};
const MESSAGE = 'for the album';

// --- selling and buying (L4.6) ----------------------------------------------
// Every amount below is the HOST's, and none follows from its price by any
// arithmetic: 61.25, 113.70 and 1,025.55 are not 5% of 1,200.50, 9.5% of the
// rest and what is left. A view that computed a split would draw other numbers.
const SELLER = '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy';
const ROYALTY = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const SOURCE = '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y';
const sparks = (cgt) => `${cgt.replace(/[,.]/g, '')}0000000000000000`;
const payout = (kind, address, share, cgt) => ({
  kind, address, share, amount_sparks: sparks(cgt), amount_cgt: cgt, to_buyer: false,
});
const BREAKDOWN = {
  price_sparks: sparks('1,200.50'),
  price_cgt: '1,200.50',
  source: { collection: 2, item: 7 },
  source_share: '5%',
  payouts: [
    payout('source', SOURCE, null, '61.25'),
    payout('royalty', ROYALTY, '9.5%', '113.70'),
    payout('seller', ADDRESS, null, '1,025.55'),
  ],
  blocked: null,
};
// A price no sale could settle at: the host says why, and the view shows it.
const BLOCKED = {
  price_sparks: sparks('7.00'),
  price_cgt: '7.00',
  source: null,
  source_share: null,
  payouts: [payout('royalty', ROYALTY, '9.5%', '0.66'), payout('seller', ADDRESS, null, '6.34')],
  blocked: `A sale at this price cannot settle as things stand. ${ROYALTY} is owed 0.66 CGT from it, and that account holds too little to stay open on that.`,
};
// Answered late, after the price in the field has moved on.
const SLOW = {
  price_sparks: sparks('55.00'),
  price_cgt: '55.00',
  source: null,
  source_share: null,
  payouts: [payout('seller', ADDRESS, null, '55.00')],
  blocked: null,
};
// What the CHAIN says the listing is, once listed: deliberately not the price
// that was typed, so a card that drew what it believed cannot pass.
const LISTING = { seller: ADDRESS, price_sparks: sparks('1,234.00'), price_cgt: '1,234.00', void: false };
const LISTED = [{ ...FRESH, revisable: false, listing: LISTING }];
const SAME = {
  price_sparks: LISTING.price_sparks,
  price_cgt: LISTING.price_cgt,
  source: null,
  source_share: null,
  payouts: [payout('seller', ADDRESS, null, '1,234.00')],
  blocked: null,
};
// Someone else's asset, reached by its number.
const FOUND_ASSET = {
  collection: 9,
  item: 3,
  name: 'someone-elses-remix',
  origin: ref('7a', 512),
  current: ref('7a', 512),
  commit: { kind: 'SHA-1', id: 'd4'.repeat(20) },
  revisable: true,
  listing: { seller: SELLER, price_sparks: sparks('3,000.00'), price_cgt: '3,000.00', void: false },
};
const TERMS = { recipients: [{ address: ROYALTY, share: '9.5%' }], remix: '12%' };
const FOUND = {
  asset: FOUND_ASSET,
  holder: SELLER,
  held_by_viewer: false,
  derived_from: { collection: 2, item: 7 },
  terms: TERMS,
  source_terms: { recipients: [{ address: SOURCE, share: '40%' }], remix: '5%' },
  breakdown: {
    price_sparks: sparks('3,000.00'),
    price_cgt: '3,000.00',
    source: { collection: 2, item: 7 },
    source_share: '5%',
    payouts: [
      payout('source', SOURCE, null, '151.10'),
      payout('royalty', ROYALTY, '9.5%', '270.20'),
      payout('seller', SELLER, null, '2,578.70'),
    ],
    blocked: null,
  },
  viewer_free_cgt: '8,894.50',
  cannot_buy: null,
  pasted_root_matches: true,
};
// For sale, and out of this account's reach: the HOST says so and says why. A
// card that judged it for itself would offer Buy, since a price is on it.
const POOR = {
  ...FOUND,
  asset: { ...FOUND_ASSET, item: 4, name: 'out-of-reach' },
  cannot_buy:
    'This account cannot cover it. Buying takes 3,000.00 CGT and the account has 894.50 CGT it can spend: it holds 994.50 CGT, and 100.00 CGT of that has to stay for the account to remain open.',
  pasted_root_matches: null,
};
// What a buyer pastes: the line the seller's own menu copies.
const PASTED = `BLAKE3-256 ${FOUND_ASSET.current.root} (asset ${FOUND_ASSET.collection}/${FOUND_ASSET.item})`;
// The same asset after its seller raised the price.
const REPRICED = {
  ...FOUND,
  asset: {
    ...FOUND_ASSET,
    listing: { ...FOUND_ASSET.listing, price_sparks: sparks('3,500.00'), price_cgt: '3,500.00' },
  },
  breakdown: {
    ...FOUND.breakdown,
    price_sparks: sparks('3,500.00'),
    price_cgt: '3,500.00',
    payouts: [
      payout('source', SOURCE, null, '176.05'),
      payout('royalty', ROYALTY, '9.5%', '315.90'),
      payout('seller', SELLER, null, '3,008.05'),
    ],
  },
  pasted_root_matches: null,
};
// And after this account bought it: held here, listed nowhere.
const BOUGHT_ASSET = { ...FOUND_ASSET, listing: null };
const BOUGHT = [{ ...FRESH, revisable: false }, BOUGHT_ASSET];
const HELD = {
  ...FOUND,
  asset: BOUGHT_ASSET,
  holder: ADDRESS,
  held_by_viewer: true,
  breakdown: null,
  viewer_free_cgt: '5,394.50',
  cannot_buy: 'This asset is not listed for sale.',
  pasted_root_matches: null,
};
const RECEIPT = {
  collection: FOUND_ASSET.collection,
  item: FOUND_ASSET.item,
  name: FOUND_ASSET.name,
  seller: SELLER,
  buyer: ADDRESS,
  price_sparks: sparks('3,500.00'),
  price_cgt: '3,500.00',
  payouts: REPRICED.breakdown.payouts,
  tx_hash: '0x07',
  block_hash: '0x08',
};
// The price as a person types it. `parseFloat` reads this as 1.
const TYPED = '1,200.50';
const PRICE_CHANGED =
  'The price changed since you looked: it was 3,000.00 CGT and it is 3,500.00 CGT now. Nothing was sent.';
const NOT_A_NUMBER =
  "That is not an asset's number. It is the two numbers on an asset's card, like 4/0, or the reference its holder copied from the asset's menu.";

const HISTORY_REFUSAL =
  'Transaction history is not available yet. A Substrate node serves no history RPC; it comes from an indexer over the chain\'s events (ADR-028), which is not built.';

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

const profile = await mkdtemp(join(tmpdir(), 'qor-inventory-'));
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

  // A stand-in host: the fixture for drc369_*, history refused as the real host
  // refuses it, and enough of the bootstrap for the shell to render its rail.
  const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + '00'.repeat(32), path: '', index: 0, label: 'Fixture' }];
  const SESSION = {
    qor_id: 'fixture-qor-id', username: 'fixture', discriminator: 1,
    role: 'user', address: ADDRESS, avatar_url: null,
  };
  const STATE = {
    version: '0.0.0-fixture',
    vault: { state: 'unlocked', accounts: ACCOUNTS, locks_in: 900 },
    session: SESSION,
    chain_endpoint: 'ws://127.0.0.1:0',
    auth_endpoint: 'http://127.0.0.1:0',
    token: { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' },
  };
  const HOST_STUB = `
    window.__BEFORE__ = ${JSON.stringify(BEFORE)};
    window.__AFTER__ = ${JSON.stringify(AFTER)};
    window.__TRADED__ = ${JSON.stringify(TRADED)};
    window.__LISTED__ = ${JSON.stringify(LISTED)};
    window.__BOUGHT__ = ${JSON.stringify(BOUGHT)};
    window.__PREVIEW_CALLS__ = [];
    window.__LIST_CALLS__ = [];
    window.__UNLIST_CALLS__ = [];
    window.__SALE_CALLS__ = [];
    window.__BUY_CALLS__ = [];
    window.__SALE_MODE__ = 'listed';
    window.__DECLINE_NEXT__ = false;
    window.__REFUSE_BUY__ = false;
    window.__TRADE_CALLS__ = [];
    window.__LISTING_SAVES__ = [];
    window.__VOCABULARY__ = ${JSON.stringify(VOCABULARY)};
    window.__MODE__ = 'before';
    window.__ASSET_CALLS__ = [];
    window.__PERMANENT_CALLS__ = [];
    window.__LAUNCHER_STATE__ = ${JSON.stringify(STATE)};
    window.__ACCOUNTS__ = ${JSON.stringify(ACCOUNTS)};
    window.__SESSION__ = ${JSON.stringify(SESSION)};
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd === 'drc369_assets') {
          window.__ASSET_CALLS__.push(args);
          if (window.__MODE__ === 'fail')
            return Promise.reject({ kind: 'rpc', message: 'the node did not answer: fixture outage' });
          if (window.__MODE__ === 'empty') return Promise.resolve([]);
          if (window.__MODE__ === 'traded') return Promise.resolve(window.__TRADED__);
          if (window.__MODE__ === 'listed') return Promise.resolve(window.__LISTED__);
          if (window.__MODE__ === 'bought') return Promise.resolve(window.__BOUGHT__);
          return Promise.resolve(window.__MODE__ === 'after' ? window.__AFTER__ : window.__BEFORE__);
        }
        if (cmd === 'drc369_make_permanent') {
          window.__PERMANENT_CALLS__.push(args);
          window.__MODE__ = 'after';
          return Promise.resolve({ collection: args.collection, item: args.item, tx_hash: '0x01', block_hash: '0x02' });
        }
        if (cmd === 'listing_vocabulary') return Promise.resolve(window.__VOCABULARY__);
        if (cmd === 'listing_drafts') return Promise.resolve([]);
        if (cmd === 'listing_save') {
          window.__LISTING_SAVES__.push(args.draft);
          return Promise.resolve({
            ...args.draft,
            price_sparks: '1',
            created: 1,
            updated: 2,
          });
        }
        if (cmd === 'drc369_sale_preview') {
          window.__PREVIEW_CALLS__.push(args);
          if (args.price === 'abc')
            return Promise.reject({ kind: 'bad_amount', message: 'amount is not valid: "abc" is not a number' });
          if (args.price === '7') return Promise.resolve(${JSON.stringify(BLOCKED)});
          // Held back until the check releases it, after the field has moved on.
          if (args.price === '55')
            return new Promise((answer) => {
              window.__RELEASE_SLOW__ = () => answer(${JSON.stringify(SLOW)});
            });
          if (args.price === ${JSON.stringify(LISTING.price_cgt)}) return Promise.resolve(${JSON.stringify(SAME)});
          return Promise.resolve(${JSON.stringify(BREAKDOWN)});
        }
        if (cmd === 'drc369_list') {
          window.__LIST_CALLS__.push(args);
          if (window.__DECLINE_NEXT__) {
            window.__DECLINE_NEXT__ = false;
            return Promise.reject({ kind: 'declined', message: 'declined at the confirmation prompt; nothing was signed or changed' });
          }
          window.__MODE__ = 'listed';
          return Promise.resolve({
            collection: args.collection,
            item: args.item,
            price_sparks: ${JSON.stringify(BREAKDOWN.price_sparks)},
            price_cgt: ${JSON.stringify(BREAKDOWN.price_cgt)},
            tx_hash: '0x05',
            block_hash: '0x06',
          });
        }
        if (cmd === 'drc369_unlist') {
          window.__UNLIST_CALLS__.push(args);
          window.__MODE__ = 'traded';
          return Promise.resolve({ collection: args.collection, item: args.item, tx_hash: '0x09', block_hash: '0x0a' });
        }
        if (cmd === 'drc369_sale') {
          window.__SALE_CALLS__.push(args);
          if (!/\\d+\\/\\d+/.test(args.asset))
            return Promise.reject({ kind: 'qontrol', message: ${JSON.stringify(NOT_A_NUMBER)} });
          if (args.asset === '9/4') return Promise.resolve(${JSON.stringify(POOR)});
          if (window.__SALE_MODE__ === 'repriced') return Promise.resolve(${JSON.stringify(REPRICED)});
          if (window.__SALE_MODE__ === 'bought') return Promise.resolve(${JSON.stringify(HELD)});
          return Promise.resolve(${JSON.stringify(FOUND)});
        }
        if (cmd === 'drc369_buy') {
          window.__BUY_CALLS__.push(args);
          if (window.__REFUSE_BUY__) {
            window.__REFUSE_BUY__ = false;
            return Promise.reject({ kind: 'qontrol', message: ${JSON.stringify(PRICE_CHANGED)} });
          }
          window.__MODE__ = 'bought';
          window.__SALE_MODE__ = 'bought';
          return Promise.resolve(${JSON.stringify(RECEIPT)});
        }
        if (cmd === 'trade_partners') return Promise.resolve([${JSON.stringify(PARTNER)}]);
        if (cmd === 'drc369_trade') {
          window.__TRADE_CALLS__.push(args);
          window.__MODE__ = 'traded';
          return Promise.resolve({
            moved: args.items,
            to: args.to,
            tx_hash: '0x03',
            block_hash: '0x04',
          });
        }
        if (cmd === 'cgt_history')
          return Promise.reject({ kind: 'rpc', message: ${JSON.stringify(HISTORY_REFUSAL)} });
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

  // Open the Inventory the way a person does: the rail button with that label.
  const opened = await evaluate(`(() => {
    const b = [...document.querySelectorAll('nav[aria-label="Primary"] button')]
      .find((n) => n.textContent.trim() === 'Inventory');
    if (!b) return false;
    b.click();
    return true;
  })()`);
  check('the rail has an Inventory button and it opens the surface', opened === true);
  await sleep(600);

  const heading = await evaluate(`document.querySelector('main h1')?.textContent.trim() ?? ''`);
  check('the surface is Inventory', heading === 'Inventory', heading);

  // Everything drawn for one asset, by its collection/item.
  const drawn = () => evaluate(`[...document.querySelectorAll('main [data-asset]')].map((row) => ({
    id: row.getAttribute('data-asset'),
    text: row.textContent,
    name: row.querySelector('[data-asset-name]')?.textContent.trim() ?? '',
    status: row.querySelector('[data-asset-status]')?.textContent.trim() ?? '',
    root: row.querySelector('[data-asset-root]')?.textContent.trim() ?? '',
    commit: row.querySelector('[data-asset-commit]')?.textContent.trim() ?? '',
    permanentButtons: [...row.querySelectorAll('button')].filter((b) => b.textContent.trim() === 'Make permanent').length,
  }))`);

  const calls = await evaluate(`window.__ASSET_CALLS__`);
  check(
    'the host was asked for the active account\'s assets',
    Array.isArray(calls) && calls.length >= 1 && calls.every((c) => c.address === ADDRESS),
    JSON.stringify(calls),
  );

  const assertMatches = (rows, fixture, when) => {
    check(
      `${when}: one row per asset the host returned`,
      rows.length === fixture.length,
      `${rows.length} drawn, ${fixture.length} returned`,
    );
    for (const asset of fixture) {
      const id = `${asset.collection}/${asset.item}`;
      const row = rows.find((r) => r.id === id);
      check(`${when}: asset ${id} is drawn`, Boolean(row));
      if (!row) continue;
      check(`${when}: ${id} shows its project name`, row.name === asset.name, row.name);
      check(
        `${when}: ${id} shows the reference it carries NOW`,
        row.root === `${asset.current.algo} ${asset.current.root}`,
        row.root,
      );

      check(
        `${when}: ${id} shows the commit it pins, by hash`,
        row.commit.includes(asset.commit.id) && row.commit.includes(asset.commit.kind),
        row.commit,
      );
      check(
        `${when}: ${id} reads ${asset.revisable ? 'Revisable' : 'Permanent'}`,
        row.status === (asset.revisable ? 'Revisable' : 'Permanent'),
        row.status,
      );
      check(
        `${when}: ${id} offers "Make permanent" only if it is revisable`,
        row.permanentButtons === (asset.revisable ? 1 : 0),
        `${row.permanentButtons}`,
      );
    }
  };

  // The card face carries identity; the close-up carries the rest. Opened by the
  // card's own title button, as a person opens it, and closed with Escape.
  const closeUp = async (fixture, when) => {
    for (const asset of fixture) {
      const id = `${asset.collection}/${asset.item}`;
      const opened = await evaluate(`(() => {
        const card = document.querySelector('main [data-asset="${id}"]');
        const open = card && card.querySelector('[data-asset-name]');
        if (!open) return false;
        open.click();
        return true;
      })()`);
      await sleep(250);
      const shown = await evaluate(`(() => {
        const panel = document.querySelector('[data-asset-closeup]');
        return panel ? panel.textContent : null;
      })()`);
      check(`${when}: ${id} opens at size`, opened === true && typeof shown === 'string');
      if (typeof shown === 'string') {
        check(
          `${when}: ${id}'s close-up shows its manifest size`,
          shown.includes(`${asset.current.size} bytes`),
        );
        check(
          `${when}: ${id}'s close-up shows the reference it carries now`,
          shown.includes(asset.current.root),
        );
        const revised = asset.origin.root !== asset.current.root;
        check(
          `${when}: ${id}'s close-up ${revised ? 'shows' : 'does not show'} what it was minted as`,
          revised ? shown.includes('Minted as') && shown.includes(asset.origin.root) : !shown.includes('Minted as'),
        );
      }
      await evaluate(`document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
      await sleep(200);
      check(
        `${when}: ${id} closes again`,
        (await evaluate(`Boolean(document.querySelector('[data-asset-closeup]'))`)) === false,
      );
    }
  };

  assertMatches(await drawn(), BEFORE, 'before');
  await closeUp(BEFORE, 'before');

  const history = await evaluate(`document.querySelector('main')?.textContent.includes('ADR-028') ?? false`);
  check('history says why it is absent, in the host\'s words', history === true);

  // Every class the surface uses must exist in the stylesheet (see the same
  // check in check-projects-view.mjs for why). A function, because the dialogs
  // opened further down are drawn inside the surface and are held to it too.
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
  check(
    'every class the surface uses is defined by the stylesheet',
    undefinedOnTheSurface.length === 0,
    undefinedOnTheSurface.join(' '),
  );

  // Wait for what the next line is about to read, rather than for a length of
  // time: a fixed sleep is a flake waiting for a loaded machine (HANDOFF.md §5).
  // `until` throws, naming what never came, so a missing screen stops the run
  // at the line that is wrong instead of failing forty checks after it.
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

  // Make the revisable one permanent.
  const before = (await evaluate(`window.__ASSET_CALLS__.length`)) ?? 0;
  const clicked = await evaluate(`(() => {
    const row = document.querySelector('main [data-asset="${FRESH.collection}/${FRESH.item}"]');
    const b = row && [...row.querySelectorAll('button')].find((n) => n.textContent.trim() === 'Make permanent');
    if (!b) return false;
    b.click();
    return true;
  })()`);
  check('"Make permanent" can be pressed on the revisable asset', clicked === true);
  await sleep(700);

  const permanentCalls = await evaluate(`window.__PERMANENT_CALLS__`);
  check(
    'the host was asked to make it permanent exactly once',
    Array.isArray(permanentCalls) && permanentCalls.length === 1,
    `${permanentCalls?.length}`,
  );
  check(
    'for the active account and exactly that asset',
    permanentCalls?.[0]?.from === ADDRESS &&
      permanentCalls?.[0]?.collection === FRESH.collection &&
      permanentCalls?.[0]?.item === FRESH.item,
    JSON.stringify(permanentCalls?.[0]),
  );

  // THE ONE THAT MATTERS: afterwards the view asked the chain again and drew
  // that answer, including an asset it could not have known about.
  const after = (await evaluate(`window.__ASSET_CALLS__.length`)) ?? 0;
  check('afterwards the view asked the host again', after > before, `${before} -> ${after}`);
  assertMatches(await drawn(), AFTER, 'after');
  await closeUp(AFTER, 'after');


  // --- an asset's own menu, and a trade (L4.4, L4.5) ------------------------
  //
  // The menu must be reachable without a mouse, so it is opened here the way a
  // keyboard opens it: focus the card's button, then activate it. The pointer's
  // secondary button is checked too, because that is what the owner asked for.
  const menuId = `${REVISED.collection}/${REVISED.item}`;
  const addId = `${ARRIVED.collection}/${ARRIVED.item}`;

  const focused = await evaluate(`(() => {
    const button = document.querySelector('main [data-asset="${menuId}"] [data-asset-more]');
    if (!button) return 'no button';
    button.focus();
    return document.activeElement === button ? 'focused' : 'not focusable';
  })()`);
  check('an asset carries a menu button the keyboard can reach', focused === 'focused', String(focused));

  await evaluate(`document.activeElement.click()`);
  await sleep(200);
  const menu = await evaluate(`(() => {
    const menu = document.querySelector('[data-asset-menu]');
    if (!menu) return null;
    const items = [...menu.querySelectorAll('[role="menuitem"]')];
    return {
      role: menu.getAttribute('role'),
      focusedIsItem: items.includes(document.activeElement),
      sellDisabled: menu.querySelector('[data-asset-sell]')?.disabled === true,
      names: items.map((n) => n.textContent.trim()),
      text: menu.textContent,
    };
  })()`);
  check('the keyboard opens the menu', menu !== null);
  check('it is a menu, and it takes focus', menu?.role === 'menu' && menu?.focusedIsItem === true);
  // Before the Market (L7.2) this read "nobody browses to it"; now a listing is
  // seen there, and what it lacks is search.
  check('Sell is offered, and the menu says the Market shows a listing, with no search',
    menu?.sellDisabled === false && /the Market shows it/i.test(menu?.text ?? '') && /no search/i.test(menu?.text ?? ''));
  // The owner looked for "Trade" and found "Send…", because the label used to
  // depend on how many assets the account held. A feature does not hide its own
  // name: these are the three things an asset offers, named.
  check('the menu names Trade first, then Sell, then Copy reference',
    JSON.stringify(menu?.names) === JSON.stringify(['Trade…', 'Sell', 'Copy reference']),
    JSON.stringify(menu?.names));

  // Escape closes it, and the secondary pointer button opens it again.
  await evaluate(`document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
  await sleep(150);
  check('Escape closes the menu', (await evaluate(`Boolean(document.querySelector('[data-asset-menu]'))`)) === false);

  const rightClicked = await evaluate(`(() => {
    const card = document.querySelector('main [data-asset="${menuId}"]');
    if (!card) return false;
    card.dispatchEvent(new MouseEvent('contextmenu', { bubbles: true, cancelable: true, clientX: 200, clientY: 200 }));
    return true;
  })()`);
  await sleep(200);
  check('the secondary pointer button opens it too', rightClicked === true &&
    (await evaluate(`Boolean(document.querySelector('[data-asset-menu]'))`)) === true);

  // Trade opens the dialog, with the asset the menu was raised from in it.
  await evaluate(`document.querySelector('[data-asset-menu] [role="menuitem"]').click()`);
  await sleep(250);
  const composing = await evaluate(`(() => {
    const dialog = document.querySelector('[data-trade-dialog]');
    if (!dialog) return null;
    const panel = dialog.querySelector('[role="dialog"]');
    return {
      step: panel?.getAttribute('data-trade-step'),
      chosen: [...dialog.querySelectorAll('[data-trade-picked] [data-trade-pick]')].map((n) => n.getAttribute('data-trade-pick')),
      offered: [...dialog.querySelectorAll('[data-trade-others] [data-trade-pick]')].map((n) => n.getAttribute('data-trade-pick')),
      reviewDisabled: dialog.querySelector('[data-trade-review]')?.disabled === true,
    };
  })()`);
  check('Trade opens the dialog at its first step', composing?.step === 'compose');

  const sides = await evaluate(`(() => {
    const dialog = document.querySelector('[data-trade-dialog]');
    if (!dialog) return null;
    const partner = dialog.querySelector('[data-trade-partners] [data-trade-partner]');
    return {
      leaving: Boolean(dialog.querySelector('[data-trade-from]')),
      going: Boolean(dialog.querySelector('[data-trade-to-side]')),
      lane: Boolean(dialog.querySelector('[data-trade-lane]')),
      laneHidden: dialog.querySelector('[data-trade-lane]')?.getAttribute('aria-hidden') === 'true',
      partner: partner ? partner.getAttribute('data-trade-partner') : null,
    };
  })()`);
  check('the trade has two sides and a lane between them',
    sides?.leaving === true && sides?.going === true && sides?.lane === true);
  check('the lane is decoration, and says so to assistive technology', sides?.laneHidden === true);
  check('someone traded with before is offered', sides?.partner === PARTNER.address,
    String(sides?.partner));

  // Choosing one fills the destination, so a second trade is two clicks.
  await evaluate(`document.querySelector('[data-trade-partners] [data-trade-partner]').click()`);
  await sleep(200);
  check('choosing them fills the destination',
    (await evaluate(`document.querySelector('[data-trade-to]')?.value`)) === PARTNER.address);
  check('the asset the menu was raised from is in the trade', JSON.stringify(composing?.chosen) === JSON.stringify([menuId]));
  check('the account\'s other assets are offered', (composing?.offered ?? []).includes(addId));
  check('with no destination it cannot be reviewed', composing?.reviewDisabled === true);

  // React owns these inputs, so type the way a person does: through the
  // native setter, then the event React listens for.
  const type = (selector, text) => evaluate(`(() => {
    const input = document.querySelector(${JSON.stringify(selector)});
    if (!input) return false;
    const set = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
    set.call(input, ${JSON.stringify(text)});
    input.dispatchEvent(new Event('input', { bubbles: true }));
    return true;
  })()`);
  check('the destination can be typed', (await type('[data-trade-to]', RECIPIENT)) === true);
  check('a message can be typed', (await type('[data-trade-message]', MESSAGE)) === true);
  await evaluate(`document.querySelector('[data-trade-others] [data-trade-pick="${addId}"]').click()`);
  await sleep(200);
  const chosen = await evaluate(`[...document.querySelectorAll('[data-trade-picked] [data-trade-pick]')].map((n) => n.getAttribute('data-trade-pick'))`);
  check('another asset can be added to the trade', chosen?.length === 2 && chosen.includes(addId), JSON.stringify(chosen));

  // Nothing has been sent, and nothing may be sent before the warning.
  check('nothing has been sent yet', (await evaluate(`window.__TRADE_CALLS__.length`)) === 0);
  // Tag Review's element before it is pressed. If React reuses it as Send, Send
  // cross-fades out of the primary button's colours for 200ms, and the
  // readability check measured it mid-fade (2026-09-26).
  await evaluate(`document.querySelector('[data-trade-review]').dataset.wasReview = 'yes'`);
  await evaluate(`document.querySelector('[data-trade-review]').click()`);
  await sleep(250);
  check('Send is a new button, not Review turned into it',
    (await evaluate(`document.querySelector('[data-trade-send]')?.dataset.wasReview ?? 'fresh'`)) === 'fresh');
  const warning = await evaluate(`(() => {
    const panel = document.querySelector('[data-trade-dialog] [role="dialog"]');
    const shown = document.querySelector('[data-trade-warning]');
    return panel && shown ? { step: panel.getAttribute('data-trade-step'), text: shown.textContent } : null;
  })()`);
  check('reviewing sends nothing', (await evaluate(`window.__TRADE_CALLS__.length`)) === 0);
  check('the second step is the warning', warning?.step === 'confirm');
  check('it names the destination', (warning?.text ?? '').includes(RECIPIENT));
  check('it names every asset in the trade',
    (warning?.text ?? '').includes(REVISED.name) && (warning?.text ?? '').includes(ARRIVED.name));
  check('it says ownership transfers and cannot be undone',
    /cannot be undone/i.test(warning?.text ?? '') && /ownership transfers/i.test(warning?.text ?? ''));
  check('it repeats that the message is public and permanent',
    /read, for ever|anyone can read/i.test(warning?.text ?? '') && (warning?.text ?? '').includes(MESSAGE));

  // Sending asks the host once, with exactly what was chosen.
  const asksBefore = (await evaluate(`window.__ASSET_CALLS__.length`)) ?? 0;
  await evaluate(`document.querySelector('[data-trade-send]').click()`);
  await sleep(800);
  const sent = await evaluate(`window.__TRADE_CALLS__`);
  check('sending asks the host exactly once', Array.isArray(sent) && sent.length === 1, JSON.stringify(sent));
  const call = sent?.[0] ?? {};
  check('it sends the address that was typed', call.to === RECIPIENT, String(call.to));
  check('it sends both assets and nothing else',
    JSON.stringify((call.items ?? []).map((i) => `${i.collection}/${i.item}`).sort()) ===
      JSON.stringify([menuId, addId].sort()), JSON.stringify(call.items));
  check('it sends the message that was typed', call.message === MESSAGE, String(call.message));

  // And afterwards the view draws what the chain says, not what it hoped.
  check('afterwards it asks the host what is held',
    ((await evaluate(`window.__ASSET_CALLS__.length`)) ?? 0) > asksBefore);
  check('the dialog closes', (await evaluate(`Boolean(document.querySelector('[data-trade-dialog]'))`)) === false);
  assertMatches(await drawn(), TRADED, 'after the trade');


  // --- Sell: a listing published on chain (L4.6) ----------------------------
  // The asset the trade raised its menu from has just left this account, so the
  // listing is made for the one still held. No `?.` on these clicks: a card
  // that is not there must throw here, not pass by doing nothing. And no fixed
  // sleeps: each step waits for what it is about to read.
  const sellId = `${FRESH.collection}/${FRESH.item}`;
  const openMenu = async (id) => {
    await evaluate(`document.querySelector('main [data-asset="${id}"] [data-asset-more]').click()`);
    await until(`Boolean(document.querySelector('[data-asset-menu]'))`, 'the asset\'s menu');
  };
  const menuNames = () => evaluate(
    `[...document.querySelectorAll('[data-asset-menu] [role="menuitem"]')].map((n) => n.textContent.trim())`,
  );
  const pressMenu = (name) => evaluate(`(() => {
    const item = [...document.querySelectorAll('[data-asset-menu] [role="menuitem"]')]
      .find((n) => n.textContent.trim() === ${JSON.stringify(name)});
    item.click();
  })()`);
  // What the breakdown on screen says, row by row, as text.
  const breakdownDrawn = (inside) => evaluate(`(() => {
    const box = document.querySelector('${inside} [data-breakdown]');
    if (!box) return null;
    return {
      rows: [...box.querySelectorAll('[data-payout]')].map((row) => ({
        kind: row.getAttribute('data-payout'),
        text: row.textContent.replace(/\\s+/g, ' ').trim(),
        amount: row.lastElementChild.textContent.trim(),
      })),
      source: box.querySelector('[data-breakdown-source]')?.textContent.replace(/\\s+/g, ' ').trim() ?? null,
      total: box.querySelector('[data-breakdown-total]').textContent.replace(/\\s+/g, ' ').trim(),
      note: box.querySelector('[data-breakdown-note]').textContent,
      blocked: box.querySelector('[data-breakdown-blocked]')?.textContent ?? null,
    };
  })()`);
  const previewState = () => evaluate(`document.querySelector('[data-sell-preview]').getAttribute('data-sell-preview')`);
  const listDisabled = () => evaluate(`document.querySelector('[data-sell-list]').disabled`);

  await openMenu(sellId);
  await pressMenu('Sell');
  await until(`Boolean(document.querySelector('[data-sell-dialog]'))`, 'the listing form');

  const sell = await evaluate(`(() => {
    const dialog = document.querySelector('[data-sell-dialog]');
    return {
      reach: dialog.querySelector('[data-sell-reach]').textContent.replace(/\\s+/g, ' '),
      asset: Boolean(dialog.querySelector('[data-sell-asset]')),
      price: dialog.querySelector('[data-sell-price]').value,
      priceLabelled: dialog.querySelector('label[for="sell-price"]') !== null,
      preview: dialog.querySelector('[data-sell-preview]').getAttribute('data-sell-preview'),
      listDisabled: dialog.querySelector('[data-sell-list]').disabled,
      listLabel: dialog.querySelector('[data-sell-list]').textContent.trim(),
      withdraw: Boolean(dialog.querySelector('[data-sell-withdraw]')),
      described: Boolean(dialog.querySelector('[data-sell-description]')),
    };
  })()`);
  check('Sell opens a listing form beside the asset', sell.asset === true);
  check('it says, in the product, that a listing is public and on chain',
    /public and on chain/i.test(sell.reach), sell.reach);
  // Since the Market (L7.2): it is seen there, with no search, and the number is
  // how to point a buyer at it. Before the Market this said "no storefront".
  check('and that the Market shows it with no search, so the number is how to point a buyer at it',
    /Market/.test(sell.reach) && /no search/i.test(sell.reach) && sell.reach.includes(sellId), sell.reach);
  check('the price starts empty: nothing here suggests one', sell.price === '');
  check('the price field has a label', sell.priceLabelled === true);
  check('with no price there is nothing to list, and nothing was asked of the host',
    sell.listDisabled === true && sell.preview === 'none' &&
      (await evaluate(`window.__PREVIEW_CALLS__.length`)) === 0);
  check('an unlisted asset is offered "List for sale" and no withdrawal',
    sell.listLabel === 'List for sale' && sell.withdraw === false, sell.listLabel);
  check('the description is another screen, not part of the listing', sell.described === false);

  // A typed price goes to the host as it was typed, and what the host answers
  // is what is drawn: every amount, in the chain's order, and no sum made here.
  await type('[data-sell-price]', '1200.5');
  await until(`document.querySelector('[data-sell-preview]').getAttribute('data-sell-preview') === 'ready'`, 'the breakdown');
  const previewCalls = await evaluate(`window.__PREVIEW_CALLS__`);
  check('the host is asked what that price would pay, with the price as typed',
    previewCalls.length === 1 && previewCalls[0].price === '1200.5' &&
      `${previewCalls[0].collection}/${previewCalls[0].item}` === sellId,
    JSON.stringify(previewCalls));
  const drawnBreakdown = await breakdownDrawn('[data-sell-dialog]');
  check('every part the host returned is drawn, in the order the chain pays them',
    JSON.stringify(drawnBreakdown.rows.map((r) => r.kind)) === JSON.stringify(BREAKDOWN.payouts.map((p) => p.kind)),
    JSON.stringify(drawnBreakdown.rows.map((r) => r.kind)));
  check('each part shows the host\'s amount and nothing recomputed',
    JSON.stringify(drawnBreakdown.rows.map((r) => r.amount)) === JSON.stringify(BREAKDOWN.payouts.map((p) => p.amount_cgt)),
    JSON.stringify(drawnBreakdown.rows.map((r) => r.amount)));
  check('the work it was remixed from is named, with its share of the price',
    (drawnBreakdown.source ?? '').includes(`${BREAKDOWN.source.collection}/${BREAKDOWN.source.item}`) &&
      (drawnBreakdown.source ?? '').includes(BREAKDOWN.source_share), String(drawnBreakdown.source));
  check('a royalty names its share and who receives it',
    drawnBreakdown.rows[1].text.includes(BREAKDOWN.payouts[1].share) &&
      drawnBreakdown.rows[1].text.includes(BREAKDOWN.payouts[1].address.slice(0, 6)), drawnBreakdown.rows[1].text);
  check('the seller\'s own line reads "You keep"', drawnBreakdown.rows[2].text.startsWith('You keep'), drawnBreakdown.rows[2].text);
  check('the total is the price the host returned',
    drawnBreakdown.total.includes('A buyer pays') && drawnBreakdown.total.includes(BREAKDOWN.price_cgt), drawnBreakdown.total);
  check('it says nothing else is taken', /nothing else is taken/i.test(drawnBreakdown.note));
  check('a price that would settle can be listed', (await listDisabled()) === false);

  // A price the chain would refuse every sale at: the host's reason is shown,
  // and it cannot be listed.
  await type('[data-sell-price]', '7');
  await until(`Boolean(document.querySelector('[data-sell-dialog] [data-breakdown-blocked]'))`, 'the blocked breakdown');
  const blocked = await breakdownDrawn('[data-sell-dialog]');
  check('a price no sale could settle at shows the host\'s reason',
    (blocked.blocked ?? '').includes('cannot settle'), String(blocked.blocked));
  check('and cannot be listed', (await listDisabled()) === true);

  // A price that is not one: the host's words, not a guess.
  await type('[data-sell-price]', 'abc');
  await until(`document.querySelector('[data-sell-preview]').getAttribute('data-sell-preview') === 'refused'`, 'the refusal');
  const priceRefusal = await evaluate(`document.querySelector('[data-sell-price-refusal]').textContent`);
  check('a price that is not a number is refused in the host\'s words',
    priceRefusal.includes('is not a number') && !priceRefusal.includes('[object'), priceRefusal);
  check('and cannot be listed either', (await listDisabled()) === true);

  // A late answer for a price no longer in the field is dropped: the breakdown
  // on screen is for the price on screen, or a person approves the wrong one.
  await type('[data-sell-price]', '55');
  await until(`typeof window.__RELEASE_SLOW__ === 'function'`, 'the slow preview to be asked for');
  // Typed as a person types a large number. The host reads the separator; a
  // view that parsed the price itself would send something else.
  await type('[data-sell-price]', TYPED);
  await until(`document.querySelector('[data-sell-preview]').getAttribute('data-sell-preview') === 'ready'`, 'the breakdown again');
  await evaluate(`window.__RELEASE_SLOW__()`);
  // The one wait here that is a length of time: it is for something that must
  // NOT happen, and there is nothing to wait for but a moment in which it could.
  await evaluate(`new Promise((done) => setTimeout(done, 120))`);
  const afterLate = await breakdownDrawn('[data-sell-dialog]');
  check('a late answer for another price does not replace the breakdown',
    afterLate.total.includes(BREAKDOWN.price_cgt) && !afterLate.total.includes(SLOW.price_cgt), afterLate.total);

  // Declined in the host's dialog: nothing is listed, the form stays, and the
  // person is told in words.
  check('nothing has been listed yet', (await evaluate(`window.__LIST_CALLS__.length`)) === 0);
  await evaluate(`window.__DECLINE_NEXT__ = true`);
  const assetAsksBeforeDecline = await evaluate(`window.__ASSET_CALLS__.length`);
  await evaluate(`document.querySelector('[data-sell-list]').click()`);
  await until(`Boolean(document.querySelector('[data-sell-refusal]'))`, 'the refusal after declining');
  const declinedText = await evaluate(`document.querySelector('[data-sell-refusal]').textContent`);
  check('a declined listing says nothing was signed', /nothing was signed/i.test(declinedText), declinedText);
  check('the form stays open after declining', (await evaluate(`Boolean(document.querySelector('[data-sell-dialog]'))`)) === true);
  check('and the card shows no listing',
    (await evaluate(`Boolean(document.querySelector('main [data-asset="${sellId}"] [data-asset-listing]'))`)) === false);
  check('a declined listing does not re-read the chain as if something changed',
    (await evaluate(`window.__ASSET_CALLS__.length`)) === assetAsksBeforeDecline);

  // Approved: the host is sent exactly what was typed, once, and the card then
  // draws the listing the CHAIN reports, which the fixture makes differ from
  // what was typed so a view that drew its own belief cannot pass.
  await evaluate(`document.querySelector('[data-sell-list]').click()`);
  await until(`!document.querySelector('[data-sell-dialog]')`, 'the form to close after listing');
  check('the person is told what to send a buyer',
    await eventually(`document.body.textContent.includes(${JSON.stringify(`A buyer needs this asset’s number: ${sellId}`)})`));
  const listCalls = await evaluate(`window.__LIST_CALLS__`);
  check('listing asks the host twice in all: the declined one and this one', listCalls.length === 2, `${listCalls.length}`);
  const listCall = listCalls[1] ?? {};
  check('it sends the active account, that asset and the price exactly as typed, unparsed',
    listCall.from === ADDRESS && `${listCall.collection}/${listCall.item}` === sellId && listCall.price === TYPED,
    JSON.stringify(listCall));
  await until(`Boolean(document.querySelector('main [data-asset="${sellId}"] [data-asset-listing]'))`, 'the listing on the card');
  const listedCard = await evaluate(`(() => {
    const card = document.querySelector('main [data-asset="${sellId}"]');
    return {
      listing: card.querySelector('[data-asset-listing]').textContent.replace(/\\s+/g, ' ').trim(),
      price: card.querySelector('[data-asset-price]').textContent.replace(/\\s+/g, ' ').trim(),
      withdraw: card.querySelector('[data-asset-withdraw]')?.textContent.trim() ?? null,
    };
  })()`);
  check('the card says it is for sale', listedCard.listing.startsWith('For sale'), listedCard.listing);
  check('at the price the chain reports, not the one that was typed',
    listedCard.price === `${LISTING.price_cgt} CGT`, listedCard.price);
  check('a listed card offers Withdraw', listedCard.withdraw === 'Withdraw', String(listedCard.withdraw));

  // A listed asset's menu offers a new price and a withdrawal in Sell's place.
  await openMenu(sellId);
  const listedMenu = await menuNames();
  check('a listed asset\'s menu names Trade, Change price, Withdraw listing, Copy reference',
    JSON.stringify(listedMenu) === JSON.stringify(['Trade…', 'Change price', 'Withdraw listing', 'Copy reference']),
    JSON.stringify(listedMenu));
  check('the menu says a listing is on chain, that the Market shows it, and that it has no search',
    /price on chain/i.test(await evaluate(`document.querySelector('[data-asset-menu]').textContent`)) &&
      /Market/.test(await evaluate(`document.querySelector('[data-asset-menu]').textContent`)) &&
      /no search/i.test(await evaluate(`document.querySelector('[data-asset-menu]').textContent`)));
  await pressMenu('Change price');
  await until(`Boolean(document.querySelector('[data-sell-dialog] [data-sell-current]'))`, 'the form for a listed asset');
  await until(`document.querySelector('[data-sell-preview]').getAttribute('data-sell-preview') === 'ready'`, 'the listed price\'s breakdown');
  const relisting = await evaluate(`(() => {
    const dialog = document.querySelector('[data-sell-dialog]');
    return {
      current: dialog.querySelector('[data-sell-current]').textContent.replace(/\\s+/g, ' '),
      price: dialog.querySelector('[data-sell-price]').value,
      listLabel: dialog.querySelector('[data-sell-list]').textContent.trim(),
      listDisabled: dialog.querySelector('[data-sell-list]').disabled,
      withdraw: dialog.querySelector('[data-sell-withdraw]')?.textContent.trim() ?? null,
    };
  })()`);
  check('the form says what it is listed at now', relisting.current.includes(`${LISTING.price_cgt} CGT`), relisting.current);
  check('and starts from that price', relisting.price === LISTING.price_cgt, relisting.price);
  check('the same price again cannot be sent: there is nothing to change',
    relisting.listLabel === 'Change the price' && relisting.listDisabled === true, relisting.listLabel);
  check('a listed asset can be withdrawn from the form', relisting.withdraw === 'Withdraw listing', String(relisting.withdraw));

  // --- the description: drafted on this machine, published nowhere ---------
  // Its own screen, reached from the listing, which keeps the price typed there.
  await type('[data-sell-price]', '1200.5');
  await evaluate(`document.querySelector('[data-sell-describe]').click()`);
  await until(`Boolean(document.querySelector('[data-sell-description]'))`, 'the description screen');
  const describing = await evaluate(`(() => {
    const dialog = document.querySelector('[data-sell-dialog]');
    const panel = dialog.querySelector('[role="dialog"]');
    return {
      step: panel.getAttribute('data-sell-step'),
      label: panel.getAttribute('aria-label'),
      listing: Boolean(dialog.querySelector('[data-sell-list]')),
      focused: document.activeElement === dialog.querySelector('[data-sell-title]'),
      unpublished: dialog.querySelector('[data-sell-unpublished]').textContent.replace(/\\s+/g, ' '),
      categories: [...dialog.querySelectorAll('[data-sell-category]')].map((n) => n.getAttribute('data-sell-category')),
      fields: [...dialog.querySelectorAll('[data-sell-field]')].length,
      title: dialog.querySelector('[data-sell-title]').value,
      saveDisabled: dialog.querySelector('[data-sell-save]').disabled,
      price: dialog.querySelector('[data-sell-draft-price]').textContent.replace(/\\s+/g, ' '),
    };
  })()`);
  check('the description is a screen of its own, named for what it is, with nothing on it that lists',
    describing.step === 'describe' && describing.label === 'Describe it' && describing.listing === false,
    JSON.stringify(describing));
  check('the keyboard starts in its title', describing.focused === true);
  check('it says which price the draft keeps: the one typed on the listing screen',
    describing.price.includes('1200.5'), describing.price);
  check('it says, in the product, that a description is not published',
    /not published/i.test(describing.unpublished) && /this machine only/i.test(describing.unpublished),
    describing.unpublished);
  check('the categories come from the host, not from this view',
    JSON.stringify(describing.categories) === JSON.stringify(VOCABULARY.map((c) => c.id)),
    JSON.stringify(describing.categories));
  check('no category is chosen, so nothing is asked yet', describing.fields === 0, String(describing.fields));
  check('the title starts as the name the chain holds', describing.title === FRESH.name, String(describing.title));
  check('it cannot be saved before a kind is chosen', describing.saveDisabled === true);
  check('every class the listing form uses is defined by the stylesheet',
    (await undefinedClasses()).length === 0, (await undefinedClasses()).join(' '));

  // Choosing a kind asks that kind's questions, and only those.
  await evaluate(`document.querySelector('[data-sell-category="gaming"]').click()`);
  await until(`document.querySelectorAll('[data-sell-field]').length === 2`, 'the gaming questions');
  const gaming = await evaluate(`[...document.querySelectorAll('[data-sell-field]')].map((n) => n.getAttribute('data-sell-field'))`);
  check('choosing Gaming asks the gaming questions',
    JSON.stringify(gaming) === JSON.stringify(['kind', 'players']), JSON.stringify(gaming));

  await evaluate(`document.querySelector('[data-sell-category="physical"]').click()`);
  await until(`Boolean(document.querySelector('[data-sell-note]'))`, 'the physical kind\'s warning');
  const physical = await evaluate(`(() => ({
    fields: [...document.querySelectorAll('[data-sell-field]')].map((n) => n.getAttribute('data-sell-field')),
    note: document.querySelector('[data-sell-note]').textContent,
    labelled: document.querySelector('label[for="sell-answer-arrives"]') !== null,
  }))()`);
  check('choosing another kind asks different questions',
    JSON.stringify(physical.fields) === JSON.stringify(['arrives']), JSON.stringify(physical.fields));
  check('a kind that needs a warning shows it before anything is typed',
    /not the object/i.test(physical.note), String(physical.note));
  check('a question answered by typing has a label', physical.labelled === true);

  // What is typed is what the host is sent: no more, no less.
  await evaluate(`document.querySelector('[data-sell-category="gaming"]').click()`);
  await until(`Boolean(document.querySelector('[data-sell-option="kind:Tool"]'))`, 'the gaming answers');
  await type('[data-sell-title]', 'A tower defence');
  await evaluate(`document.querySelector('[data-sell-option="kind:Tool"]').click()`);
  await until(`document.querySelector('[data-sell-option="kind:Tool"]').getAttribute('aria-pressed') === 'true'`, 'the answer to be chosen');
  check('nothing has been saved yet', (await evaluate(`window.__LISTING_SAVES__.length`)) === 0);
  await evaluate(`document.querySelector('[data-sell-save]').click()`);
  await until(`Boolean(document.querySelector('[data-sell-saved]'))`, 'the draft to be saved');
  const saves = await evaluate(`window.__LISTING_SAVES__`);
  check('saving sends the host one draft', Array.isArray(saves) && saves.length === 1, JSON.stringify(saves));
  const draft = saves[0] ?? {};
  check('it sends the title, the kind and the price that were typed',
    draft.title === 'A tower defence' && draft.category === 'gaming' && draft.price_cgt === '1200.5',
    JSON.stringify(draft));
  check('it sends the answer that was chosen, and no answer that was not',
    JSON.stringify(draft.details) === JSON.stringify([{ field: 'kind', value: 'Tool' }]),
    JSON.stringify(draft.details));
  check('it is for the asset whose menu was used',
    `${draft.collection}/${draft.item}` === sellId, `${draft.collection}/${draft.item}`);
  check('and the person is told it went no further',
    /not published anywhere/i.test(await evaluate(`document.querySelector('[data-sell-saved]').textContent`)));
  check('saving a description lists nothing', (await evaluate(`window.__LIST_CALLS__.length`)) === 2);

  // Back to the listing: the same dialog, the price still as it was typed.
  await evaluate(`document.querySelector('[data-sell-back]').click()`);
  await until(`document.querySelector('[data-sell-dialog] [role="dialog"]').getAttribute('data-sell-step') === 'price'`, 'the listing screen again');
  check('going back returns to the listing, with the price as it was typed',
    (await evaluate(`document.querySelector('[data-sell-price]').value`)) === '1200.5');
  check('and the listing now offers the description that was saved',
    (await evaluate(`document.querySelector('[data-sell-describe]').textContent.trim()`)) === 'Your description');

  await evaluate(`document.querySelector('[data-sell-dialog] [aria-label="Close"]').click()`);
  await until(`!document.querySelector('[data-sell-dialog]')`, 'the listing form to close');
  check('the listing form closes', (await evaluate(`Boolean(document.querySelector('[data-sell-dialog]'))`)) === false);

  // Withdraw, from the card: the host is asked once, for that asset, and the
  // card then draws what the chain says.
  check('nothing has been withdrawn yet', (await evaluate(`window.__UNLIST_CALLS__.length`)) === 0);
  await evaluate(`document.querySelector('main [data-asset="${sellId}"] [data-asset-withdraw]').click()`);
  await until(`!document.querySelector('main [data-asset="${sellId}"] [data-asset-listing]')`, 'the listing to leave the card');
  const unlistCalls = await evaluate(`window.__UNLIST_CALLS__`);
  check('withdrawing asks the host exactly once', unlistCalls.length === 1, `${unlistCalls.length}`);
  check('for the active account and exactly that asset',
    unlistCalls[0]?.from === ADDRESS && `${unlistCalls[0]?.collection}/${unlistCalls[0]?.item}` === sellId,
    JSON.stringify(unlistCalls[0]));
  check('afterwards the card offers no withdrawal',
    (await evaluate(`Boolean(document.querySelector('main [data-asset="${sellId}"] [data-asset-withdraw]'))`)) === false);
  assertMatches(await drawn(), TRADED, 'after withdrawing');


  // --- Buy: one asset, looked up by its number (L4.6) -----------------------
  // There is no catalogue (ADR-028), so the surface starts from a number its
  // holder gave the buyer, and it must say so rather than look like a shop.
  const find = await evaluate(`(() => {
    const section = document.querySelector('main [data-find]');
    return {
      honest: section.querySelector('[data-find-honest]').textContent.replace(/\\s+/g, ' '),
      labelled: section.querySelector('label[for="find-asset"]') !== null,
      submitDisabled: section.querySelector('[data-find-submit]').disabled,
      cards: section.querySelectorAll('[data-found-asset]').length,
    };
  })()`);
  check('buying names the Market, says it has no search, and what to ask a holder for',
    /Market/.test(find.honest) && /no search/i.test(find.honest) && /number/i.test(find.honest), find.honest);
  check('the lookup field has a label', find.labelled === true);
  check('with nothing typed there is nothing to look up, and nothing is listed unasked',
    find.submitDisabled === true && find.cards === 0 && (await evaluate(`window.__SALE_CALLS__.length`)) === 0);

  // Something that is not a number: the host's words, and no card.
  await type('[data-find-input]', 'nonsense');
  await evaluate(`document.querySelector('[data-find-submit]').click()`);
  await until(`Boolean(document.querySelector('[data-find-refusal]'))`, 'the lookup\'s refusal');
  const findRefusal = await evaluate(`document.querySelector('[data-find-refusal]').textContent`);
  check('what is not an asset\'s number is refused in the host\'s words',
    findRefusal.includes('not an asset\'s number') && !findRefusal.includes('[object'), findRefusal);
  check('and no card is drawn for it', (await evaluate(`document.querySelectorAll('[data-found-asset]').length`)) === 0);

  // For sale, but not to this account: whether it can be bought is the host's
  // judgement, drawn as given. The card shows the reason and offers no Buy.
  const poorId = `${POOR.asset.collection}/${POOR.asset.item}`;
  await type('[data-find-input]', poorId);
  await evaluate(`document.querySelector('[data-find-submit]').click()`);
  await until(`Boolean(document.querySelector('[data-found-asset="${poorId}"]'))`, 'the card of an asset out of reach');
  const poor = await evaluate(`(() => {
    const card = document.querySelector('[data-found-asset="${poorId}"]');
    return {
      price: card.querySelector('[data-asset-price]').textContent.replace(/\\s+/g, ' ').trim(),
      cannot: card.querySelector('[data-asset-cannot-buy]')?.textContent ?? null,
      buy: Boolean(card.querySelector('[data-asset-buy]')),
    };
  })()`);
  check('an asset the host says this account cannot buy shows the host\'s reason',
    poor.cannot === POOR.cannot_buy, String(poor.cannot));
  check('and offers no Buy, though it has a price',
    poor.buy === false && poor.price === `${POOR.asset.listing.price_cgt} CGT`, JSON.stringify(poor));
  check('the refusal from the lookup before it is gone',
    (await evaluate(`Boolean(document.querySelector('[data-find-refusal]'))`)) === false);

  // A number: the host is sent what was typed and who is asking, and the asset
  // comes back as a card. Submitted as the Enter key submits it.
  const foundId = `${FOUND.asset.collection}/${FOUND.asset.item}`;
  await type('[data-find-input]', PASTED);
  await evaluate(`document.querySelector('main [data-find] form').requestSubmit()`);
  await until(`Boolean(document.querySelector('[data-found-asset="${foundId}"]'))`, 'the found asset\'s card');
  const saleCalls = await evaluate(`window.__SALE_CALLS__`);
  check('the host is asked for exactly what was pasted, for the active account',
    saleCalls.length === 3 && saleCalls[2].asset === PASTED && saleCalls[2].viewer === ADDRESS,
    JSON.stringify(saleCalls));
  check('one asset is shown at a time: the one that was asked for',
    (await evaluate(`document.querySelectorAll('[data-found-asset]').length`)) === 1);
  const foundCard = () => evaluate(`(() => {
    const card = document.querySelector('[data-found-asset="${foundId}"]');
    if (!card) return null;
    return {
      name: card.querySelector('[data-asset-name]').textContent.trim(),
      root: card.querySelector('[data-asset-root]').textContent.trim(),
      price: card.querySelector('[data-asset-price]')?.textContent.replace(/\\s+/g, ' ').trim() ?? null,
      holder: card.querySelector('[data-asset-holder]').textContent.trim(),
      holderInFull: card.querySelector('[data-asset-holder]').getAttribute('title'),
      match: card.querySelector('[data-asset-match]')?.textContent ?? null,
      cannot: card.querySelector('[data-asset-cannot-buy]')?.textContent ?? null,
      buy: Boolean(card.querySelector('[data-asset-buy]')),
      menu: Boolean(card.querySelector('[data-asset-more]')),
      withdraw: Boolean(card.querySelector('[data-asset-withdraw]')),
      permanent: [...card.querySelectorAll('button')].filter((b) => b.textContent.trim() === 'Make permanent').length,
    };
  })()`);
  let card = await foundCard();
  check('the found asset is drawn as a card, by its name and the reference it carries now',
    card.name === FOUND.asset.name && card.root === `${FOUND.asset.current.algo} ${FOUND.asset.current.root}`,
    JSON.stringify(card));
  check('it shows the price the chain holds', card.price === `${FOUND.asset.listing.price_cgt} CGT`, String(card.price));
  check('and who holds it, with the whole address behind the short one',
    card.holder.startsWith(FOUND.holder.slice(0, 8)) && card.holderInFull === FOUND.holder, card.holder);
  check('it says the pasted fingerprint is the one the asset carries', /fingerprint you pasted/i.test(card.match ?? ''));
  check('a listed asset someone else holds offers Buy', card.buy === true && card.cannot === null);
  check('and nothing only its holder may do: no menu, no withdrawal, no Make permanent',
    card.menu === false && card.withdraw === false && card.permanent === 0, JSON.stringify(card));
  check('a found asset is not counted among the assets this account holds',
    (await drawn()).length === TRADED.length, `${(await drawn()).length}`);

  // The close-up of a found asset carries its holder and its royalty terms.
  await evaluate(`document.querySelector('[data-found-asset="${foundId}"] [data-asset-name]').click()`);
  await until(`Boolean(document.querySelector('[data-asset-closeup]'))`, 'the found asset\'s close-up');
  const foundCloseUp = await evaluate(`document.querySelector('[data-asset-closeup]').textContent.replace(/\\s+/g, ' ')`);
  check('its close-up names its holder in full', foundCloseUp.includes(FOUND.holder));
  check('and the royalty terms the chain holds',
    foundCloseUp.includes(`${FOUND.terms.recipients[0].share} of every sale to ${FOUND.terms.recipients[0].address}`) &&
      foundCloseUp.includes(`owes them ${FOUND.terms.remix}`), foundCloseUp);
  check('and what it was remixed from',
    foundCloseUp.includes(`Asset ${FOUND.derived_from.collection}/${FOUND.derived_from.item}`));
  await evaluate(`document.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }))`);
  await until(`!document.querySelector('[data-asset-closeup]')`, 'the close-up to close');

  // Buy opens one screen, and that screen is the warning.
  await evaluate(`document.querySelector('[data-found-asset="${foundId}"] [data-asset-buy]').click()`);
  await until(`Boolean(document.querySelector('[data-buy-dialog]'))`, 'the purchase dialog');
  const buying = await evaluate(`(() => {
    const dialog = document.querySelector('[data-buy-dialog]');
    const panel = dialog.querySelector('[role="dialog"]');
    return {
      label: panel.getAttribute('aria-label'),
      modal: panel.getAttribute('aria-modal'),
      focused: document.activeElement === panel || panel.contains(document.activeElement),
      asset: dialog.querySelector('[data-buy-asset]').textContent.replace(/\\s+/g, ' '),
      warning: dialog.querySelector('[data-buy-warning]').textContent.replace(/\\s+/g, ' '),
      balance: dialog.querySelector('[data-buy-balance]').textContent.replace(/\\s+/g, ' '),
      revisable: dialog.querySelector('[data-buy-revisable]')?.textContent ?? null,
      confirm: dialog.querySelector('[data-buy-confirm]').textContent.trim(),
      confirmDisabled: dialog.querySelector('[data-buy-confirm]').disabled,
    };
  })()`);
  check('Buy opens a dialog named for the asset, and focus moves into it',
    buying.label === `Buy ${FOUND.asset.name}` && buying.modal === 'true' && buying.focused === true,
    JSON.stringify(buying));
  check('it shows the asset and its whole fingerprint',
    buying.asset.includes(foundId) && buying.asset.includes(FOUND.asset.current.root));
  const buyBreakdown = await breakdownDrawn('[data-buy-dialog]');
  check('it shows where the price goes: every part the host returned, in order',
    JSON.stringify(buyBreakdown.rows.map((r) => [r.kind, r.amount])) ===
      JSON.stringify(FOUND.breakdown.payouts.map((p) => [p.kind, p.amount_cgt])),
    JSON.stringify(buyBreakdown.rows));
  check('the seller is named, not "you"', buyBreakdown.rows.at(-1).text.startsWith('The seller'), buyBreakdown.rows.at(-1).text);
  check('the total is what the buyer pays, as the host returned it',
    buyBreakdown.total.includes('You pay') && buyBreakdown.total.includes(FOUND.breakdown.price_cgt), buyBreakdown.total);
  check('it says the purchase cannot be undone, and that no more than the price can be taken',
    /cannot be undone/i.test(buying.warning) && /will not pay more/i.test(buying.warning), buying.warning);
  check('it says what the account can spend', buying.balance.includes(`${FOUND.viewer_free_cgt} CGT`), buying.balance);
  check('a revisable asset says it can still be revised', /still revisable/i.test(buying.revisable ?? ''));
  check('the button names the price', buying.confirm === `Buy for ${FOUND.breakdown.price_cgt} CGT` && buying.confirmDisabled === false, buying.confirm);
  check('every class the purchase dialog uses is defined by the stylesheet',
    (await undefinedClasses()).length === 0, (await undefinedClasses()).join(' '));
  check('opening the dialog buys nothing', (await evaluate(`window.__BUY_CALLS__.length`)) === 0);

  // The price changed since the buyer looked: the host refuses, in words, and
  // the dialog reads the asset again and draws THAT, so nobody is left looking
  // at a price the chain no longer holds.
  const salesBeforeRefusal = await evaluate(`window.__SALE_CALLS__.length`);
  await evaluate(`window.__REFUSE_BUY__ = true; window.__SALE_MODE__ = 'repriced'`);
  await evaluate(`document.querySelector('[data-buy-confirm]').click()`);
  await until(`Boolean(document.querySelector('[data-buy-reread]'))`, 'the purchase\'s refusal, and the screen read again');
  const buyRefusal = await evaluate(`document.querySelector('[data-buy-refusal]').textContent`);
  check('a refused purchase shows the host\'s reason in words',
    buyRefusal.includes('The price changed since you looked') && !buyRefusal.includes('[object'), buyRefusal);
  check('the dialog stays open after a refusal', (await evaluate(`Boolean(document.querySelector('[data-buy-dialog]'))`)) === true);
  const firstBuy = (await evaluate(`window.__BUY_CALLS__`))[0] ?? {};
  check('the host was sent the price and the fingerprint that were on screen',
    firstBuy.from === ADDRESS && `${firstBuy.collection}/${firstBuy.item}` === foundId &&
      firstBuy.priceSparks === FOUND.breakdown.price_sparks && firstBuy.root === FOUND.asset.current.root,
    JSON.stringify(firstBuy));
  check('after a refusal the dialog asks the host for the asset again',
    (await evaluate(`window.__SALE_CALLS__.length`)) === salesBeforeRefusal + 1);
  const reread = await breakdownDrawn('[data-buy-dialog]');
  check('and draws the price and the parts the chain holds now',
    reread.total.includes(REPRICED.breakdown.price_cgt) &&
      JSON.stringify(reread.rows.map((r) => r.amount)) === JSON.stringify(REPRICED.breakdown.payouts.map((p) => p.amount_cgt)),
    JSON.stringify(reread));
  check('the button names the new price, and the refusal says the screen was read again',
    (await evaluate(`document.querySelector('[data-buy-confirm]').textContent.trim()`)) === `Buy for ${REPRICED.breakdown.price_cgt} CGT` &&
      /read from the chain again/i.test(buyRefusal), buyRefusal);

  // Approved: one call, with the price now on screen; then the chain is asked
  // what is held, and the card that offered it says who holds it.
  const asksBeforeBuy = await evaluate(`window.__ASSET_CALLS__.length`);
  await evaluate(`document.querySelector('[data-buy-confirm]').click()`);
  await until(`!document.querySelector('[data-buy-dialog]')`, 'the purchase dialog to close after buying');
  check('the person is told it is theirs, at the price the chain settled',
    await eventually(`document.body.textContent.includes(${JSON.stringify(`for ${RECEIPT.price_cgt} CGT. It is in your Inventory.`)})`));
  const buyCalls = await evaluate(`window.__BUY_CALLS__`);
  check('buying asks the host once more, and only once', buyCalls.length === 2, `${buyCalls.length}`);
  check('with the price that was on screen when Buy was pressed',
    buyCalls[1]?.priceSparks === REPRICED.breakdown.price_sparks && buyCalls[1]?.root === REPRICED.asset.current.root,
    JSON.stringify(buyCalls[1]));
  await until(`window.__ASSET_CALLS__.length > ${asksBeforeBuy}`, 'the chain to be asked what is held');
  await until(`Boolean(document.querySelector('main [data-asset="${foundId}"]'))`, 'the bought asset among those held');
  assertMatches(await drawn(), BOUGHT, 'after buying');
  await until(`Boolean(document.querySelector('[data-found-asset="${foundId}"] [data-asset-cannot-buy]'))`, 'the found card to say it is held');
  card = await foundCard();
  check('the card that offered it now says this account holds it, and offers no Buy',
    card.buy === false && card.holder === 'This account' && card.cannot === HELD.cannot_buy,
    JSON.stringify(card));

  // Nothing held: it says so.
  await evaluate(`window.__MODE__ = 'empty'`);
  const refresh = `(() => {
    const b = [...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === 'Refresh');
    if (!b) return false;
    b.click();
    return true;
  })()`;
  check('there is a Refresh', (await evaluate(refresh)) === true);
  await sleep(600);
  let rows = await drawn();
  let body = await evaluate(`document.querySelector('main')?.textContent ?? ''`);
  check('with nothing held there are no rows', rows.length === 0, `${rows.length}`);
  check('with nothing held it says so', body.includes('No assets yet'));

  // The chain cannot be read: it says that, not "nothing held".
  await evaluate(`window.__MODE__ = 'fail'`);
  await evaluate(refresh);
  await sleep(600);
  rows = await drawn();
  body = await evaluate(`document.querySelector('main')?.textContent ?? ''`);
  check('when the chain cannot be read there are no rows', rows.length === 0, `${rows.length}`);
  check(
    'when the chain cannot be read it says so, with the host\'s reason',
    body.includes('Could not read') && body.includes('fixture outage'),
  );
  check(
    'and it does not claim that nothing is held',
    !body.includes('No assets yet'),
  );
} finally {
  await finish();
}

const failed = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
