// Is every piece of text in the launcher readable, as it is actually painted? (L1.2, ADR-051)
//
// check-contrast.mjs does the arithmetic: every ink token against every background
// token, and over the worst colour the QFX canvas could paint beneath the chrome's
// scrim. That arithmetic ASSUMES the layers are stacked the way the design says —
// canvas, then scrim, then the interface on top. On 22 September 2026 they were
// not: the canvas and the scrim were rendered inside #qor-root, so the rule meant
// to lift the interface above them lifted all three together, and the scrim —
// the theme's base colour at 90% — was painted OVER most of the interface. Vault,
// Settings, Inventory, the rail and the title bar sat under it; only Nexus, whose
// content happens to carry its own z-index, escaped. Every check passed, and the
// owner found it by using the launcher.
//
// So this check measures what is painted, not what is declared. For every screen
// the launcher has — the Gate's three states and every surface on the rail — in
// every theme, with the backdrop live and with it off, it:
//
//   1. replaces the canvas with a deliberately hostile backdrop: a flat white panel
//      in the canvas's exact place in the stacking order (same parent, same
//      z-index), so whatever the canvas can paint, this is worse for light text;
//   2. takes two screenshots: one as rendered, one with every glyph made
//      transparent. Their difference is the text AS PAINTED, including anything
//      laid over it, and the second is the background actually behind it;
//   3. holds every run of visible text to WCAG 2.2 AA: 4.5:1, or 3:1 for large
//      text (24px, or 18.66px bold), measured two ways. Its DECLARED colour, with
//      its alpha and every ancestor's opacity, composited over the background
//      actually behind it, must reach the threshold exactly. Its PAINTED glyphs
//      must reach PAINTED_SHARE of it: anti-aliasing keeps a thin stem short of
//      its colour, and an overlay leaves far less. Text in a disabled control is
//      exempt, as WCAG exempts inactive components, and is counted so the
//      exemption stays visible.
//
// Text is measured where it is visible: in the viewport, not clipped away, and at
// the top and the bottom of each surface's scrolling area. "Not clipped away"
// includes a dialog that scrolls inside the window (1 October 2026): the part of
// a run that an overflowing ancestor has scrolled out of its box is not on
// screen, so it is not measured there, and the dialog is measured again
// scrolled to its end, where it is.
//
// No dependencies beyond Node 22+ and an installed Edge or Chrome.
//
//   npm run build
//   node scripts/check-readability.mjs [dist directory, default ./dist]
//
// Exits non-zero if any text fails.
//
// Proven to fail first, on 22 September 2026, against the launcher as it was:
// the scrim painted over the interface, and this check failed on every theme.

import { spawn } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { mkdtemp, readFile, rm } from 'node:fs/promises';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { inflateSync } from 'node:zlib';
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

// Every shipped theme, read from the theme table so a new one is covered without an edit.
const THEME_IDS = [
  ...readFileSync(join(here, '..', 'src', 'styles', 'themes.ts'), 'utf8').matchAll(/^\s+id: '([a-z-]+)',\r?$/gm),
].map((m) => m[1]);
// For iterating on one theme: READABILITY_THEMES=architect. A run meant as
// evidence leaves it unset, and then every theme is measured.
const THEMES = process.env.READABILITY_THEMES ? process.env.READABILITY_THEMES.split(',') : THEME_IDS;

// Anti-aliasing can keep the brightest pixel of a thin stem short of its colour,
// so the PAINTED measurement is held to this share of the AA threshold. Set from
// the check's own calibration line (painted over declared, for text whose colour
// passes): on 22 September 2026 its minimum was 0.983 and its median 1.000 at
// twice the device scale. An overlay leaves far less: text under the 90% scrim
// measured 1.00 to 1.54:1 painted, about a third of the threshold. The DECLARED
// measurement is held to the threshold exactly.
const PAINTED_SHARE = 0.9;

const HOSTILE = '#ffffff';
const WIDTH = 1440;
const HEIGHT = 900;
const SCALE = 2; // twice the pixels per glyph stem, so anti-aliasing does not read as low contrast

const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

const results = [];
/** How often the test browser stopped producing frames and had to be revived. */
let recovered = 0;
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

const profile = await mkdtemp(join(tmpdir(), 'qor-readability-'));
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
    // A headless window can count as occluded, and an occluded page produces no
    // frames and throttles its timers: every screenshot would wait for ever.
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

// A stand-in host. Which screen the app shows is chosen by `qor-check.mode` in
// localStorage: the Gate's four screens -- 'absent' (a first run), 'locked' (a
// Windows Hello vault on its last Hello, cancelled), 'locked-passphrase' (a
// passphrase vault on its last passphrase) and 'locked-keychain' (a keychain
// vault that would not open) -- and 'inside', the signed-in shell with every
// surface (ADR-056).
const ADDRESS = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const ACCOUNTS = [{ address: ADDRESS, account_id: '0x' + 'd4'.repeat(32), path: '', index: 0, label: 'Main' }];
const SESSION = { qor_id: 'fixture#0001', username: 'fixture', discriminator: 1, role: 'user', address: ADDRESS, avatar_url: null };
const TOKEN = { symbol: 'CGT', name: 'Creator God Token', decimals: 18, sub_unit: 'Spark' };
// The first is listed, so the price on a card is on the screen being measured;
// the second carries a listing its previous holder made, which is void.
const ASSETS = [
  { collection: 0, item: 0, name: 'first-song', origin: { algo: 'BLAKE3-256', root: '1f'.repeat(32), size: 211 }, current: { algo: 'BLAKE3-256', root: '3d'.repeat(32), size: 377 }, commit: { kind: 'SHA-1', id: 'a1'.repeat(20) }, revisable: true, listing: { seller: ADDRESS, price_sparks: '1200500000000000000000', price_cgt: '1,200.50', void: false } },
  { collection: 0, item: 1, name: 'final-mix', origin: { algo: 'BLAKE3-256', root: '4c'.repeat(32), size: 99 }, current: { algo: 'BLAKE3-256', root: '4c'.repeat(32), size: 99 }, commit: { kind: 'SHA-1', id: 'b2'.repeat(20) }, revisable: false, listing: { seller: '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy', price_sparks: '900000000000000000000', price_cgt: '900.00', void: true } },
];
// Selling and buying (L4.6). What a sale pays, with a part the chain would
// refuse, so the warning under it is on the screen too; and someone else's
// asset, found by its number, whose fingerprint is not the one that was pasted,
// so the card's and the dialog's warnings are measured as well.
const OTHER = '5DAAnrj7VHTznn2AWBemMuyBwZWs6FNFjdyVXUeYum3PTXFy';
const PAYOUTS = [
  { kind: 'source', address: '5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y', share: null, amount_sparks: '60000000000000000000', amount_cgt: '60.00', to_buyer: false },
  { kind: 'royalty', address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', share: '9.5%', amount_sparks: '108300000000000000000', amount_cgt: '108.30', to_buyer: true },
  { kind: 'seller', address: OTHER, share: null, amount_sparks: '1031700000000000000000', amount_cgt: '1,031.70', to_buyer: false },
];
const BREAKDOWN = {
  price_sparks: '1200000000000000000000', price_cgt: '1,200.00',
  source: { collection: 2, item: 7 }, source_share: '5%', payouts: PAYOUTS,
  blocked: 'A sale at this price cannot settle as things stand. 5FLSigC9HGRKVhB9FiEo4Y3koPsNmBmLJbpXg2mp1hXcS59Y is owed 60.00 CGT from it, and that account holds too little to stay open on that: an account needs 100.00 CGT to exist.',
};
const FOUND = {
  asset: { collection: 9, item: 3, name: 'someone-elses-remix', origin: { algo: 'BLAKE3-256', root: '7a'.repeat(32), size: 512 }, current: { algo: 'BLAKE3-256', root: '7a'.repeat(32), size: 512 }, commit: { kind: 'SHA-1', id: 'd4'.repeat(20) }, revisable: true, listing: { seller: OTHER, price_sparks: '1200000000000000000000', price_cgt: '1,200.00', void: false } },
  holder: OTHER, held_by_viewer: false, derived_from: { collection: 2, item: 7 },
  terms: { recipients: [{ address: '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty', share: '9.5%' }], remix: '12%' },
  source_terms: null,
  breakdown: { ...BREAKDOWN, blocked: null },
  viewer_free_cgt: '8,894.50', cannot_buy: null, pasted_root_matches: false,
};
// The Market (L7.2): one listing of each standing, the walk cut short at its
// bound and one listing unreadable, so every notice the screen can draw is on
// the screen being measured.
const listed = (item, name, standing, holder, viewerHolds, isVoid, reason, derived) => ({
  asset: { ...FOUND.asset, item, name, listing: { ...FOUND.asset.listing, void: isVoid } },
  holder, held_by_viewer: viewerHolds, derived_from: derived, standing, reason,
});
const MARKET = {
  listings: [
    listed(3, 'someone-elses-remix', 'buyable', OTHER, false, false, null, { collection: 2, item: 7 }),
    listed(4, 'nested-stem', 'held_in_place', OTHER, false, false, 'Nobody can buy this asset as it stands. It is nested inside asset 4/9, and the chain will not move it until its holder takes it out.', null),
    { ...listed(5, 'my-loop', 'yours', ADDRESS, true, false, null, null), asset: { ...FOUND.asset, item: 5, name: 'my-loop', listing: { ...FOUND.asset.listing, seller: ADDRESS } } },
    listed(6, 'handed-on', 'void', ADDRESS, true, true, 'A listing made by an earlier holder of this asset is still on chain. It is void: nobody can buy from it. You hold the asset, and you can clear it.', null),
  ],
  offset: 0, matching: 500, on_chain: 500, yours: 2, truncated: true, bound: 500, unreadable: 1,
  block_number: 810, block_hash: '0x' + 'ab'.repeat(32), endpoint: 'ws://127.0.0.1:9944', chain_name: 'Demiurge Development',
};
const VOCABULARY = [
  { id: 'physical', name: 'Physical (offline)', note: 'The chain moves the record, not the object.', fields: [{ id: 'arrives', label: 'What actually arrives', options: [] }, { id: 'who-ships', label: 'Who sends it', options: ['You', 'Someone else'] }] },
];
// A project with a change whose diff has every kind of line: the rows are
// tinted by kind, and text over a tint is text over a background too.
const PROJECT = {
  path: 'X:/check/a-project', name: 'a-project', branch: 'main',
  changes: [{ path: 'sessions/verse.rpp', state: 'modified' }, { path: 'stems/take.wav', state: 'untracked' }],
  history: [{ id: 'e5'.repeat(20), short: 'e5e5e5e', summary: 'Rough out the verse', author: 'Check', time: 1758400000 }],
  // A second branch, so a Switch button is on the screen being measured.
  branches: [{ name: 'main', head: true }, { name: 'alternate-bridge', head: false }],
  can_commit: true,
};
// What the guard holds back when Commit is pressed (P1.2): a list a person
// reads before a secret or a huge file enters history for good.
const HELD = [
  { path: '.env', concern: 'credential', reason: 'an environment file, where secrets are usually kept' },
  { path: 'stems/take.wav', concern: 'large', reason: '72 MiB' },
];
const DIFF = {
  path: 'sessions/verse.rpp', state: 'modified', from: null,
  body: {
    kind: 'text', truncated: true,
    lines: [
      { kind: 'hunk', text: '@@ -3,4 +3,4 @@', old: null, new: null },
      { kind: 'context', text: 'TEMPO 92 4 4', old: 3, new: 3 },
      { kind: 'remove', text: 'VOLPAN 0.81 0 -1', old: 4, new: null },
      { kind: 'add', text: 'VOLPAN 0.64 0 -1', old: null, new: 4 },
      { kind: 'context', text: 'MUTESOLO 0 0 0', old: 5, new: 5 },
    ],
  },
};
const HOST_STUB = `
  (() => {
    const mode = localStorage.getItem('qor-check.mode') || 'inside';
    const unlocked = { state: 'unlocked', accounts: ${JSON.stringify(ACCOUNTS)} };
    const sealed = mode === 'locked-passphrase' ? 'passphrase' : mode === 'locked-keychain' ? 'keychain' : 'hello';
    const vault = mode === 'absent' ? { state: 'absent' } : mode.startsWith('locked') ? { state: 'locked', sealed_with: sealed } : unlocked;
    // Opening is refused in every locked screen, so each carries its alert: a
    // cancelled Hello, or a keychain that lost the key.
    const session = mode === 'inside' ? ${JSON.stringify(SESSION)} : null;
    const ok = (v) => Promise.resolve(v);
    const no = (kind, message) => Promise.reject({ kind, message });
    window.__TAURI_INTERNALS__ = {
      invoke: (cmd, args) => {
        if (cmd === 'launcher_state')
          return ok({ version: '0.0.0-check', vault, session, chain_endpoint: 'ws://127.0.0.1:9944', auth_endpoint: 'http://127.0.0.1:8080/api/v1', token: ${JSON.stringify(TOKEN)} });
        if (cmd === 'vault_status') return ok(vault);
        if (cmd === 'vault_unlock')
          return mode === 'locked-passphrase'
            ? no('passphrase_vault', 'this vault was sealed with a passphrase')
            : mode === 'locked-keychain'
              ? no('keychain', 'this computer account no longer holds the vault key')
              : no('declined', 'declined at the confirmation prompt');
        if (cmd === 'vault_preview_phrase') return ok(${JSON.stringify(ADDRESS)});
        if (cmd === 'qor_restore') return session ? ok(session) : no('not_authenticated', 'no session');
        if (cmd === 'qor_username_available') return ok(true);
        if (cmd === 'drc369_assets') return ok(${JSON.stringify(ASSETS)});
        if (cmd === 'drc369_sale_preview') return ok(${JSON.stringify(BREAKDOWN)});
        if (cmd === 'drc369_sale') return ok(${JSON.stringify(FOUND)});
        if (cmd === 'drc369_market') return ok(${JSON.stringify(MARKET)});
        if (cmd === 'listing_vocabulary') return ok(${JSON.stringify(VOCABULARY)});
        if (cmd === 'listing_drafts') return ok([]);
        if (cmd === 'cgt_balance') return ok({ address: args.address, sparks: '8994500000000000000000', cgt: '8994.5', display: '8,994.50 CGT' });
        if (cmd === 'chain_status')
          return ok({ endpoint: 'ws://127.0.0.1:9944', reachable: true, chain_name: 'Demiurge Development', block_number: 812, finalized_number: 810, latency_ms: 3, detail: null });
        if (cmd === 'cgt_nonce') return ok(3);
        if (cmd === 'cgt_history') return no('rpc', 'Transaction history is not available yet (ADR-028).');
        if (cmd === 'qontrol_pick_folder') return ok(${JSON.stringify(PROJECT.path)});
        if (cmd === 'qontrol_open' || cmd === 'qontrol_read') return ok(${JSON.stringify(PROJECT)});
        if (cmd === 'qontrol_diff') return ok([${JSON.stringify(DIFF)}]);
        if (cmd === 'qontrol_check') return ok(${JSON.stringify(HELD)});
        if (cmd === 'touch_vault' || cmd === 'vault_lock') return ok(null);
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

// In the page: every run of visible text, where it is and what it declares.
const COLLECT = `(() => {
  const items = [];
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = node.textContent.trim();
    if (!text) continue;
    const el = node.parentElement;
    if (!el || el.closest('script, style, noscript, svg')) continue;
    const cs = getComputedStyle(el);
    if (cs.visibility !== 'visible' || Number(cs.opacity) === 0) continue;
    const range = document.createRange();
    range.selectNodeContents(node);
    // The boxes of every ancestor that scrolls and has more than it shows. What
    // lies outside one is scrolled out of sight, wherever a hit test lands: a
    // dialog's overlay is an ancestor of everything in the dialog, so a point
    // in the clipped part "hits" an ancestor and used to count as visible.
    const boxes = [];
    for (let n = el.parentElement; n && n !== document.documentElement; n = n.parentElement) {
      const o = getComputedStyle(n);
      const scrolls = /(auto|scroll)/.test(o.overflowY) && n.scrollHeight > n.clientHeight + 1;
      const scrollsAcross = /(auto|scroll)/.test(o.overflowX) && n.scrollWidth > n.clientWidth + 1;
      if (scrolls || scrollsAcross) boxes.push(n.getBoundingClientRect());
    }
    for (const r of range.getClientRects()) {
      let x0 = Math.max(0, r.left), y0 = Math.max(0, r.top);
      let x1 = Math.min(innerWidth, r.right), y1 = Math.min(innerHeight, r.bottom);
      for (const b of boxes) {
        x0 = Math.max(x0, b.left); y0 = Math.max(y0, b.top);
        x1 = Math.min(x1, b.right); y1 = Math.min(y1, b.bottom);
      }
      if (x1 - x0 < 2 || y1 - y0 < 4) continue;
      // Visible here, not clipped away or under another element. The scrim and
      // the canvas take no pointer events, so they never count as a cover.
      const hit = document.elementFromPoint((x0 + x1) / 2, (y0 + y1) / 2);
      if (!hit || !(hit === el || el.contains(hit) || hit.contains(el))) continue;
      let opacity = 1;
      for (let n = el; n; n = n.parentElement) opacity *= Number(getComputedStyle(n).opacity);
      items.push({
        text: text.slice(0, 48),
        x0, y0, x1, y1,
        opacity,
        size: parseFloat(cs.fontSize),
        weight: Number(cs.fontWeight) || 400,
        color: cs.color,
        disabled: Boolean(el.closest(':disabled, [aria-disabled="true"]')),
      });
    }
  }
  return items;
})()`;

// A PNG, decoded with nothing but Node's own zlib: Chrome's screenshots are 8-bit
// RGB or RGBA, non-interlaced, which is all this has to read.
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
    } else if (type === 'IDAT') {
      idat.push(data);
    } else if (type === 'IEND') {
      break;
    }
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
  return { width, height, channels, data: out };
}

const LINEAR = Array.from({ length: 256 }, (_, i) => {
  const v = i / 255;
  return v <= 0.04045 ? v / 12.92 : ((v + 0.055) / 1.055) ** 2.4;
});
const rgbLuminance = (r, g, b) => 0.2126 * LINEAR[r] + 0.7152 * LINEAR[g] + 0.0722 * LINEAR[b];
const luminance = (img, x, y) => {
  const i = (y * img.width + x) * img.channels;
  return rgbLuminance(img.data[i], img.data[i + 1], img.data[i + 2]);
};
const parseColour = (css) => {
  const m = /rgba?\(([^)]+)\)/.exec(css);
  if (!m) return null;
  const [r, g, b, a = '1'] = m[1].split(/[ ,/]+/).filter(Boolean);
  return [Number(r), Number(g), Number(b), Number(a)];
};
const ratio = (p, q) => (Math.max(p, q) + 0.05) / (Math.min(p, q) + 0.05);

// Two measurements of each run, from the two screenshots. The background is the
// pixel of median luminance behind the run, with the text hidden.
//
//   DECLARED: the text's own colour, with its alpha and every ancestor's opacity,
//   composited over that background. Exact, and blind to anything laid over the
//   text.
//
//   PAINTED: the pixel in the rendered shot that stands furthest from the
//   background. The only measurement that sees an overlay: under one, even the
//   brightest glyph pixel is dim. It reads a little low on thin text, because
//   anti-aliasing keeps a narrow stem from reaching its full colour.
function analyse(renderedPng, backgroundPng, items) {
  const A = decodePng(Buffer.from(renderedPng, 'base64'));
  const B = decodePng(Buffer.from(backgroundPng, 'base64'));
  const s = A.width / WIDTH;
  return items.map((it) => {
    const x0 = Math.floor(it.x0 * s), x1 = Math.min(A.width, Math.ceil(it.x1 * s));
    const y0 = Math.floor(it.y0 * s), y1 = Math.min(A.height, Math.ceil(it.y1 * s));
    const behind = [];
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) behind.push([luminance(B, x, y), (y * B.width + x) * B.channels]);
    behind.sort((p, q) => p[0] - q[0]);
    const [back, at] = behind[Math.floor(behind.length / 2)];
    let painted = 1;
    for (let y = y0; y < y1; y++) for (let x = x0; x < x1; x++) painted = Math.max(painted, ratio(luminance(A, x, y), back));
    const colour = parseColour(it.color);
    let declared = null;
    if (colour) {
      const alpha = colour[3] * it.opacity;
      const mix = (k) => Math.round(alpha * colour[k] + (1 - alpha) * B.data[at + k]);
      declared = ratio(rgbLuminance(mix(0), mix(1), mix(2)), back);
    }
    return { ...it, painted, declared };
  });
}

try {
  let target;
  // About 30 s: a CI runner has taken more than 10 s to give Chrome its first page (6 October 2026).
  for (let i = 0; i < 300 && !target; i++) {
    // Pause before every retry, not only after a refused connection: the browser can answer before its page
    // exists, and the loop then gave up within milliseconds on a slow runner (CI, 6 October 2026).
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
    new Promise((ok, bad) => {
      const id = ++nextId;
      const timer = setTimeout(() => {
        pending.delete(id);
        bad(new Error(`the browser did not answer ${method} within 60 s`));
      }, 60000);
      pending.set(id, (msg) => {
        clearTimeout(timer);
        ok(msg);
      });
      ws.send(JSON.stringify({ id, method, params }));
    });
  const evaluate = async (expression) => {
    const r = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
    if (r.result.exceptionDetails) throw new Error(`${expression.slice(0, 200)}\n${JSON.stringify(r.result.exceptionDetails)}`);
    return r.result.result.value;
  };
  // A headless page that the window manager has decided is not visible produces
  // no frames, and a screenshot then waits for one that never comes -- the stall
  // recorded in HANDOFF.md section 5. It is the test browser, not the app, so it
  // is recovered from rather than tolerated: bring the page to front, restore
  // focus emulation, and ask once more. A second stall fails the run, loudly.
  const shoot = async () => {
    const capture = () =>
      send('Page.captureScreenshot', { format: 'png', optimizeForSpeed: true });
    try {
      return (await capture()).result.data;
    } catch (stalled) {
      recovered += 1;
      console.log(`  (the browser stopped producing frames; bringing it back: ${stalled.message})`);
      await send('Page.bringToFront');
      await send('Emulation.setFocusEmulationEnabled', { enabled: true });
      await sleep(400);
      return (await capture()).result.data;
    }
  };

  await send('Page.enable');
  await send('Runtime.enable');
  await send('Emulation.setDeviceMetricsOverride', { width: WIDTH, height: HEIGHT, deviceScaleFactor: SCALE, mobile: false });
  await send('Page.addScriptToEvaluateOnNewDocument', { source: HOST_STUB });
  await send('Page.navigate', { url: appUrl });
  for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);

  // Motion is reduced so nothing is mid-animation when it is measured. It changes
  // no layer: a live backdrop under reduced motion is held still, and the canvas
  // and scrim are shown exactly as they are when it moves.
  const boot = async (theme, ambience, mode) => {
    await evaluate(`localStorage.setItem('qor.theme', ${JSON.stringify(theme)})`);
    await evaluate(`localStorage.setItem('qor.a11y', ${JSON.stringify(JSON.stringify({ ambience, motion: 'reduced' }))})`);
    await evaluate(`localStorage.setItem('qor-check.mode', ${JSON.stringify(mode)})`);
    events.length = 0;
    await send('Page.reload', { ignoreCache: true });
    for (let i = 0; i < 100 && !events.some((e) => e.method === 'Page.loadEventFired'); i++) await sleep(100);
    await send('Page.bringToFront');
    await send('Emulation.setFocusEmulationEnabled', { enabled: true });
    await sleep(1000);
    if (ambience !== 'off') {
      // The hostile backdrop takes the canvas's exact place in the stacking order.
      const placed = await evaluate(`(() => {
        const canvas = document.querySelector('canvas.qfx-canvas');
        if (!canvas) return false;
        canvas.style.visibility = 'hidden';
        const panel = document.createElement('div');
        panel.id = 'check-hostile-backdrop';
        panel.setAttribute('style', 'position:fixed;inset:0;z-index:0;pointer-events:none;background:${HOSTILE}');
        canvas.after(panel);
        return getComputedStyle(document.querySelector('.qfx-scrim')).display !== 'none';
      })()`);
      if (!placed) throw new Error('the canvas or the scrim was not where this check expects them');
    }
  };

  const HIDE = `(() => {
    const s = document.createElement('style');
    s.id = 'check-hide-text';
    s.textContent = '*, *::before, *::after, ::placeholder { color: transparent !important; -webkit-text-fill-color: transparent !important; text-shadow: none !important; caret-color: transparent !important; }';
    document.head.appendChild(s);
  })()`;
  const SHOW = `document.getElementById('check-hide-text')?.remove()`;

  // Measure what is on screen now; returns the runs that fail.
  const measure = async () => {
    await sleep(250);
    const items = await evaluate(COLLECT);
    const rendered = await shoot();
    await evaluate(HIDE);
    await sleep(120);
    const background = await shoot();
    await evaluate(SHOW);
    return analyse(rendered, background, items);
  };

  // Every scrolling area on the current screen, taken to its end.
  const SCROLL_TO_END = `(() => {
    let moved = 0;
    for (const el of document.querySelectorAll('main *, main')) {
      const cs = getComputedStyle(el);
      if (/(auto|scroll)/.test(cs.overflowY) && el.scrollHeight > el.clientHeight + 8) {
        el.scrollTop = el.scrollHeight;
        moved += 1;
      }
    }
    return moved;
  })()`;

  const SURFACES = ['Nexus', 'Vault', 'Inventory', 'Market', 'Projects', 'Library', 'Social', 'Mesh', 'Chain', 'Gates', 'Settings'];
  const GATE = [
    ['absent', 'the Gate, with no vault'],
    ['locked', 'the Gate, the last Windows Hello, cancelled'],
    ['locked-passphrase', 'the Gate, the last passphrase'],
    ['locked-keychain', 'the Gate, a vault that would not open'],
  ];

  let measured = 0;
  let exempt = 0;
  const calibration = [];
  const judge = (label, runs) => {
    const failing = [];
    for (const run of runs) {
      if (run.disabled) { exempt += 1; continue; }
      measured += 1;
      const large = run.size >= 24 || (run.size >= 18.66 && run.weight >= 700);
      const need = large ? 3 : 4.5;
      const what = `${run.size}px, ${run.color}`;
      if (run.declared !== null && run.declared < need) {
        failing.push(`"${run.text}" its colour gives ${run.declared.toFixed(2)}:1 (needs ${need}; ${what})`);
      } else if (run.painted < need * PAINTED_SHARE) {
        failing.push(`"${run.text}" is painted at ${run.painted.toFixed(2)}:1: something is over it, or it is not drawn as declared (${what})`);
      }
      if (run.declared !== null && run.declared >= need) calibration.push(run.painted / run.declared);
    }
    check(
      `${label}: every visible run of text reaches AA`,
      failing.length === 0,
      failing.length ? `${failing.length} of ${runs.length} fail — ${failing.slice(0, 4).join('; ')}${failing.length > 4 ? '; …' : ''}` : `${runs.length} runs`,
    );
  };

  check('the theme table was read, and it is not empty', THEME_IDS.length > 0, THEME_IDS.join(', '));

  for (const theme of THEMES) {
    for (const ambience of ['live', 'off']) {
      const backdrop = ambience === 'off' ? 'backdrop off' : 'a hostile backdrop';
      for (const [mode, what] of GATE) {
        await boot(theme, ambience, mode);
        judge(`${theme}, ${backdrop}, ${what}`, await measure());

        // Restoring from a recovery phrase: the screen that shows back the
        // account a typed phrase opens, read before anything is replaced.
        if (mode === 'absent') {
          await evaluate(`[...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'I already have a recovery phrase')?.click()`);
          await sleep(500);
          await evaluate(`(() => {
            const el = document.querySelector('textarea[aria-label="Recovery phrase"]');
            if (!el) return;
            Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, 'value').set.call(el, Array(24).fill('word').join(' '));
            el.dispatchEvent(new Event('input', { bubbles: true }));
          })()`);
          await sleep(200);
          await evaluate(`[...document.querySelectorAll('button')].find((b) => b.textContent.trim() === 'Check the phrase')?.click()`);
          await sleep(500);
          if (await evaluate(`document.body.innerText.includes(${JSON.stringify(ADDRESS)})`)) {
            judge(`${theme}, ${backdrop}, the Gate, a recovery phrase checked`, await measure());
          } else {
            check(`${theme}, ${backdrop}, the Gate's restore screen shows the account`, false);
          }
        }
      }
      // Onboarding (src/components/onboarding, src/qfx/ceremony): the bubble that
      // offers a QOR ID, the card that checks a name as it is typed, and the
      // tutorial behind the glowing notification. The ceremony's carve-out from
      // the design rules keeps this obligation: its text is measured like the rest.
      await boot(theme, ambience, 'unnamed');
      if (await evaluate(`Boolean(document.querySelector('[data-claim-bubble]'))`)) {
        judge(`${theme}, ${backdrop}, the bubble offering a QOR ID`, await measure());
        await evaluate(`document.querySelector('[data-claim-bubble]')?.click()`);
        await sleep(400);
        await evaluate(`(() => {
          const el = document.querySelector('[data-claim-card] input');
          if (!el) return;
          Object.getOwnPropertyDescriptor(HTMLInputElement.prototype, 'value').set.call(el, 'architect');
          el.dispatchEvent(new Event('input', { bubbles: true }));
        })()`);
        await sleep(900);
        if (await evaluate(`document.querySelector('[data-claim-card]')?.innerText.includes('yours to take') ?? false`)) {
          judge(`${theme}, ${backdrop}, a name checked as it is typed`, await measure());
        } else {
          check(`${theme}, ${backdrop}, the name card checks a name`, false);
        }
      } else {
        check(`${theme}, ${backdrop}, the bubble offering a QOR ID shows`, false);
      }

      await boot(theme, ambience, 'inside');
      await evaluate(`document.querySelector('[data-awaken]')?.click()`);
      await sleep(900);
      if (await evaluate(`Boolean(document.querySelector('[data-tutorial]'))`)) {
        judge(`${theme}, ${backdrop}, the tutorial's first chapter`, await measure());
        for (let i = 0; i < 5; i++) {
          await evaluate(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowRight' }))`);
          await sleep(500);
        }
        // Measured once the last chapter alone is drawn and fully opaque: a
        // chapter fading out is dim for a moment, and that is not the defect.
        await evaluate(`new Promise((done) => {
          const start = performance.now();
          const poll = () => {
            const heads = document.querySelectorAll('[data-tutorial] h2');
            let ok = heads.length === 1 && heads[0].textContent.startsWith('Go forth');
            for (let n = heads[0]; ok && n && n !== document.body; n = n.parentElement) {
              if (getComputedStyle(n).opacity !== '1') ok = false;
            }
            if (ok || performance.now() - start > 5000) done(ok);
            else setTimeout(poll, 100);
          };
          poll();
        })`);
        judge(`${theme}, ${backdrop}, the tutorial's last chapter`, await measure());
        await evaluate(`window.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape' }))`);
        await sleep(700);
      } else {
        check(`${theme}, ${backdrop}, the glowing notification opens the tutorial`, false);
      }

      for (const surface of SURFACES) {
        const opened = await evaluate(`(() => {
          const b = [...document.querySelectorAll('nav[aria-label="Primary"] button')].find((n) => n.textContent.trim() === ${JSON.stringify(surface)});
          if (!b) return false;
          const before = document.querySelector('main > div');
          const already = b.getAttribute('aria-current') === 'page' || b.getAttribute('aria-pressed') === 'true';
          window.__checkOldView = already ? null : before;
          b.click();
          return true;
        })()`);
        if (!opened) {
          check(`${theme}, ${backdrop}, ${surface}: the rail opens it`, false);
          continue;
        }
        await send('Page.bringToFront');
        await send('Emulation.setFocusEmulationEnabled', { enabled: true });
        // The view cross-fades on opacity, which reduced motion does not remove.
        // Measured only once the old view is gone and the new one is fully
        // opaque: a fading view is dim for a moment, and that is not the defect
        // this looks for.
        const settled = await evaluate(`new Promise((done) => {
          const start = performance.now();
          const poll = () => {
            const views = document.querySelectorAll('main > div');
            const old = window.__checkOldView;
            const ok = (!old || !old.isConnected) && views.length === 1 && getComputedStyle(views[0]).opacity === '1';
            if (ok || performance.now() - start > 15000) done(ok);
            else setTimeout(poll, 100);
          };
          poll();
        })`);
        if (!settled) {
          check(`${theme}, ${backdrop}, ${surface}: the view finished its transition`, false);
          continue;
        }
        judge(`${theme}, ${backdrop}, ${surface}`, await measure());
        if ((await evaluate(SCROLL_TO_END)) > 0) {
          judge(`${theme}, ${backdrop}, ${surface}, scrolled to the end`, await measure());
        }

        // A project, open, with one file's diff showing: the only place text sits
        // on a tint that means something (an added or removed line).
        if (surface === 'Projects') {
          await evaluate(`[...document.querySelectorAll('main button')].find((n) => n.textContent.trim().includes('Open a folder'))?.click()`);
          await sleep(400);
          await evaluate(`document.querySelector('main [data-change]')?.click()`);
          await sleep(400);
          if (await evaluate(`Boolean(document.querySelector('main [data-diff-line="add"]'))`)) {
            judge(`${theme}, ${backdrop}, a file's diff`, await measure());
            if ((await evaluate(SCROLL_TO_END)) > 0) {
              judge(`${theme}, ${backdrop}, a file's diff, scrolled to the end`, await measure());
            }
          } else {
            check(`${theme}, ${backdrop}, a file's diff opens`, false);
          }

          // The discard question (P1.2), read before a change is thrown away.
          await evaluate(`document.querySelector('main [data-discard]')?.click()`);
          await sleep(300);
          if (await evaluate(`Boolean(document.querySelector('main [data-discard-confirm]'))`)) {
            judge(`${theme}, ${backdrop}, the discard question`, await measure());
            await evaluate(`[...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === 'Keep it')?.click()`);
            await sleep(200);
          } else {
            check(`${theme}, ${backdrop}, the discard question opens`, false);
          }

          // What the guard holds back (P1.2), read before committing anyway.
          await evaluate(`(() => {
            const input = document.querySelector('main input[placeholder="What did you change?"]');
            if (!input) return;
            const setter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
            setter.call(input, 'A message');
            input.dispatchEvent(new Event('input', { bubbles: true }));
          })()`);
          await sleep(200);
          await evaluate(`[...document.querySelectorAll('main button')].find((n) => n.textContent.trim() === 'Commit')?.click()`);
          await sleep(400);
          if (await evaluate(`Boolean(document.querySelector('main [data-held]'))`)) {
            judge(`${theme}, ${backdrop}, what the guard holds back`, await measure());
            if ((await evaluate(SCROLL_TO_END)) > 0) {
              judge(`${theme}, ${backdrop}, what the guard holds back, scrolled to the end`, await measure());
            }
          } else {
            check(`${theme}, ${backdrop}, the guard's list opens`, false);
          }
        }

        // What an asset raises is a screen too: the menu, the trade dialog and
        // its warning are drawn over the backdrop like everything else, and a
        // person reads the warning before giving away what they own.
        if (surface === 'Inventory') {
          // React renders on its own schedule, so the click and the question
          // about what it drew are two steps with a wait between them.
          await evaluate(`document.querySelector('main [data-asset-more]')?.click()`);
          await sleep(250);
          const menu = await evaluate(`Boolean(document.querySelector('[data-asset-menu]'))`);
          if (menu) {
            judge(`${theme}, ${backdrop}, an asset's menu`, await measure());
          } else {
            check(`${theme}, ${backdrop}, an asset's menu opens`, false);
          }

          await evaluate(`document.querySelector('[data-asset-menu] [role="menuitem"]')?.click()`);
          await sleep(300);
          const composing = await evaluate(`Boolean(document.querySelector('[data-trade-dialog]'))`);
          if (composing) {
            judge(`${theme}, ${backdrop}, a trade being composed`, await measure());
            const warned = await evaluate(`(() => {
              const to = document.querySelector('[data-trade-to]');
              const set = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set;
              set.call(to, '5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty');
              to.dispatchEvent(new Event('input', { bubbles: true }));
              const message = document.querySelector('[data-trade-message]');
              set.call(message, 'for the album');
              message.dispatchEvent(new Event('input', { bubbles: true }));
              return true;
            })()`);
            await sleep(200);
            if (warned) {
              await evaluate(`document.querySelector('[data-trade-review]').click()`);
              await sleep(250);
              judge(`${theme}, ${backdrop}, the warning before a trade`, await measure());
            }
            // Leave the surface as it was found.
            await evaluate(`(() => {
              const close = document.querySelector('[data-trade-dialog] [aria-label="Close"]');
              if (close) close.click();
            })()`);
            await sleep(200);
          } else {
            check(`${theme}, ${backdrop}, a trade dialog opens`, false);
          }

          // Selling and buying are screens a person reads before CGT moves
          // (L4.6): what a sale pays, the description that is published
          // nowhere, an asset found by its number, and the purchase dialog.
          // Each step waits for what it is about to measure; an element that
          // must be there is addressed without `?.`, so a missing one stops
          // the run at the line that is wrong.
          const shown = async (expression) => {
            const deadline = Date.now() + 8000;
            for (;;) {
              if (await evaluate(expression)) return true;
              if (Date.now() > deadline) return false;
              await sleep(50);
            }
          };
          const typeInto = (selector, text) => evaluate(`(() => {
            const input = document.querySelector(${JSON.stringify(selector)});
            Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, 'value').set.call(input, ${JSON.stringify(text)});
            input.dispatchEvent(new Event('input', { bubbles: true }));
          })()`);

          await evaluate(`document.querySelector('main [data-asset-more]').click()`);
          if (await shown(`Boolean(document.querySelector('[data-asset-menu] [data-asset-sell]'))`)) {
            await evaluate(`document.querySelector('[data-asset-menu] [data-asset-sell]').click()`);
          }
          if (await shown(`Boolean(document.querySelector('[data-sell-dialog]'))`)) {
            await typeInto('[data-sell-price]', '1200');
            if (await shown(`Boolean(document.querySelector('[data-sell-dialog] [data-breakdown-blocked]'))`)) {
              judge(`${theme}, ${backdrop}, a listing, with what a sale pays`, await measure());
            } else {
              check(`${theme}, ${backdrop}, the listing form shows what a sale pays`, false);
            }
            await evaluate(`document.querySelector('[data-sell-describe]').click()`);
            if (await shown(`Boolean(document.querySelector('[data-sell-category="physical"]'))`)) {
              await evaluate(`document.querySelector('[data-sell-category="physical"]').click()`);
              await shown(`Boolean(document.querySelector('[data-sell-note]'))`);
              judge(`${theme}, ${backdrop}, a listing's description`, await measure());
              // A long form scrolls inside the dialog; its end is read too.
              if ((await evaluate(SCROLL_TO_END)) > 0) {
                judge(`${theme}, ${backdrop}, a listing's description, scrolled to the end`, await measure());
              }
            } else {
              check(`${theme}, ${backdrop}, the listing's description opens`, false);
            }
            await evaluate(`document.querySelector('[data-sell-dialog] [aria-label="Close"]').click()`);
            await shown(`!document.querySelector('[data-sell-dialog]')`);
          } else {
            check(`${theme}, ${backdrop}, the listing form opens`, false);
          }

          await typeInto('[data-find-input]', '9/3');
          await evaluate(`document.querySelector('[data-find-submit]').click()`);
          if (await shown(`Boolean(document.querySelector('[data-found-asset] [data-asset-buy]'))`)) {
            await evaluate(`document.querySelector('[data-found-asset]').scrollIntoView({ block: 'center' })`);
            judge(`${theme}, ${backdrop}, an asset found by its number`, await measure());
            await evaluate(`document.querySelector('[data-found-asset] [data-asset-buy]').click()`);
            if (await shown(`Boolean(document.querySelector('[data-buy-dialog] [data-breakdown]'))`)) {
              judge(`${theme}, ${backdrop}, the purchase dialog`, await measure());
              await evaluate(`document.querySelector('[data-buy-dialog] [aria-label="Close"]').click()`);
              await shown(`!document.querySelector('[data-buy-dialog]')`);
            } else {
              check(`${theme}, ${backdrop}, the purchase dialog opens`, false);
            }
          } else {
            check(`${theme}, ${backdrop}, an asset is found by its number`, false);
          }
        }
      }
    }
  }

  calibration.sort((p, q) => p - q);
  const at = (q) => calibration[Math.min(calibration.length - 1, Math.floor(q * calibration.length))]?.toFixed(3);
  console.log(`\ncalibration, painted over declared for text whose colour passes: min ${at(0)}, 1st percentile ${at(0.01)}, median ${at(0.5)}`);
  console.log(`${measured} runs of text measured; ${exempt} in disabled controls exempt, as WCAG exempts inactive components.`);
} finally {
  await finish();
}

if (recovered > 0) console.log(`
The browser stopped producing frames ${recovered} time(s) and was brought back; no measurement was skipped.`);

const failed = results.filter((r) => !r.pass).length;
console.log(`\nRESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed === 0 ? 0 : 1);
