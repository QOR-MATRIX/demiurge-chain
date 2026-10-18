// Holds qorsync.dev to what search engines and AI crawlers read (ADR-100), and to the project's language rule (ADR-008).
//
//   node site/scripts/check-site.mjs
//
// Reads the files only; nothing is fetched. Exits non-zero if any check fails.

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const repo = resolve(site, '..');
const ORIGIN = 'https://qorsync.dev';
const REPO_URL = 'https://github.com/QOR-MATRIX/demiurge-chain';

const results = [];
const check = (name, pass, detail = '') => {
  results.push(pass);
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}${detail ? `  (${detail})` : ''}`);
};
const read = (f) => readFileSync(join(site, f), 'utf8');
const html = read('index.html');
const attr = (re) => (html.match(re) ?? [])[1] ?? '';
const meta = (key) => attr(new RegExp(`<meta (?:name|property)="${key}" content="([^"]*)"`));

// --- the head ---------------------------------------------------------------------------------------------------
const title = attr(/<title>([^<]*)<\/title>/);
check('a title of 30 to 60 characters, naming Demiurge', title.length >= 30 && title.length <= 60 && title.includes('Demiurge'), `${title.length}: ${title}`);
const description = meta('description');
check('a description of 70 to 160 characters', description.length >= 70 && description.length <= 160, `${description.length}`);
check('a canonical address at the home domain', attr(/<link rel="canonical" href="([^"]*)"/) === `${ORIGIN}/`);
check('the page is in English and indexable', /<html lang="en">/.test(html) && /content="index, follow/.test(html));
for (const key of ['og:title', 'og:description', 'og:url', 'og:image', 'twitter:card', 'twitter:image']) {
  check(`${key} is set`, meta(key).length > 0);
}
check('og:url is the canonical address', meta('og:url') === `${ORIGIN}/`);

// --- the share image is the size it claims ---------------------------------------------------------------------
const png = (f) => {
  const b = readFileSync(join(site, f));
  return b.toString('ascii', 1, 4) === 'PNG' ? { w: b.readUInt32BE(16), h: b.readUInt32BE(20) } : null;
};
const og = existsSync(join(site, 'og.png')) && png('og.png');
check('og.png is a 1200 × 630 PNG', og && og.w === 1200 && og.h === 630, og ? `${og.w} × ${og.h}` : 'missing');
for (const [f, n] of [['icon-32.png', 32], ['icon-180.png', 180], ['icon-512.png', 512]]) {
  const i = existsSync(join(site, f)) && png(f);
  check(`${f} is ${n} × ${n}`, i && i.w === n && i.h === n);
}

// --- structured data ---------------------------------------------------------------------------------------------
let graph = [];
try {
  graph = JSON.parse(attr(/<script type="application\/ld\+json">([\s\S]*?)<\/script>/))['@graph'] ?? [];
} catch (e) {
  check('the structured data parses', false, e.message);
}
const types = graph.map((n) => n['@type']);
check('the structured data names the organisation, site, source, launcher and ARQADE',
  ['Organization', 'WebSite', 'SoftwareSourceCode', 'SoftwareApplication', 'WebApplication'].every((t) => types.includes(t)), types.join(', '));
const org = graph.find((n) => n['@type'] === 'Organization') ?? {};
check('the organisation points at the current repository', (org.sameAs ?? []).includes(REPO_URL));
check('the source is the current repository', graph.find((n) => n['@type'] === 'SoftwareSourceCode')?.codeRepository === REPO_URL);

// --- crawler files -----------------------------------------------------------------------------------------------
check('robots.txt allows crawling and names the sitemap', /Allow: \//.test(read('robots.txt')) && read('robots.txt').includes(`Sitemap: ${ORIGIN}/sitemap.xml`));
const locs = [...read('sitemap.xml').matchAll(/<loc>([^<]+)<\/loc>/g)].map((m) => m[1]);
check('the sitemap lists the canonical address', locs.includes(`${ORIGIN}/`), locs.join(' '));
check('llms.txt names the current repository', read('llms.txt').includes(REPO_URL));
const keys = readdirSync(site).filter((f) => /^[0-9a-f]{32}\.txt$/.test(f));
check('one IndexNow key file, holding its own name', keys.length === 1 && read(keys[0]).trim() === keys[0].slice(0, 32), keys.join(' '));

// --- the page itself ---------------------------------------------------------------------------------------------
check('one h1', (html.match(/<h1[\s>]/g) ?? []).length === 1);
const local = [...html.matchAll(/(?:href|src)="(\/[^"#?]*)"/g)].map((m) => m[1]).filter((p) => p !== '/');
const missing = local.filter((p) => !existsSync(join(site, p)));
check('every local file the page names exists', missing.length === 0, missing.join(' '));
const docs = [...new Set([...html.matchAll(/github\.com\/QOR-MATRIX\/demiurge-chain\/blob\/main\/([^"#]+)/g)].map((m) => m[1]))];
const gone = docs.filter((d) => !existsSync(join(repo, d)));
check('every repository document the page links exists', gone.length === 0, gone.join(' ') || `${docs.length} documents`);
const words = html.replace(/<script[\s\S]*?<\/script>/g, '').replace(/<[^>]+>/g, ' ') + read('llms.txt') + read('404.html');
const banned = /\b(APY|yield|passive income|ROI|investment|returns?)\b/i.exec(words);
check('no word of income, yield or returns (ADR-008)', !banned, banned?.[0] ?? '');

const failed = results.filter((r) => !r).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
