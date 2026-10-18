// Tell IndexNow (Bing, Yandex, Seznam, Naver and others) that qorsync.dev changed, so they re-crawl it now rather than
// whenever they next pass. The key is the 32-hex-digit file at the site's root, which the live site must already serve.
//
//   node site/scripts/indexnow.mjs [url ...]     (default: the homepage and llms.txt)

import { readdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const site = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const HOST = 'qorsync.dev';
const key = readdirSync(site).find((f) => /^[0-9a-f]{32}\.txt$/.test(f))?.slice(0, 32);
if (!key) {
  console.error('No IndexNow key file in site/.');
  process.exit(2);
}
const urls = process.argv.slice(2).length ? process.argv.slice(2) : [`https://${HOST}/`, `https://${HOST}/llms.txt`];

const live = await fetch(`https://${HOST}/${key}.txt`);
if (!live.ok || (await live.text()).trim() !== key) {
  console.error(`https://${HOST}/${key}.txt does not serve the key yet; publish the site first.`);
  process.exit(1);
}
const response = await fetch('https://api.indexnow.org/indexnow', {
  method: 'POST',
  headers: { 'content-type': 'application/json; charset=utf-8' },
  body: JSON.stringify({ host: HOST, key, keyLocation: `https://${HOST}/${key}.txt`, urlList: urls }),
});
console.log(`IndexNow answered ${response.status} for ${urls.join(', ')}`);
process.exit(response.ok ? 0 : 1);
