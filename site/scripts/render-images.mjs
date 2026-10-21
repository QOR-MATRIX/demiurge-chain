// Render the site's images with a headless Chromium-family browser: the share image (og.png, from og.html) and the
// icons (icon-32.png, icon-180.png, icon-512.png, from favicon.svg). Run after changing either source:
//
//   node site/scripts/render-images.mjs
//
// Needs Microsoft Edge or Chrome (set BROWSER_PATH for another), and no network: og.html's fonts are the site's own (tokens.css, ADR-108).

import { execFileSync } from 'node:child_process';
import { existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const site = resolve(here, '..');
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

const work = mkdtempSync(join(tmpdir(), 'qorsync-images-'));
const shoot = (url, out, width, height, wait = 3000) => {
  execFileSync(BROWSER, [
    '--headless=new', '--disable-gpu', '--hide-scrollbars', '--no-first-run', '--force-device-scale-factor=1',
    `--user-data-dir=${join(work, 'profile')}`, `--window-size=${width},${height}`,
    `--virtual-time-budget=${wait}`, `--screenshot=${out}`, url,
  ], { stdio: 'ignore' });
  if (!existsSync(out)) throw new Error(`${out} was not written`);
  console.log(`wrote ${out}`);
};

try {
  shoot(pathToFileURL(join(here, 'og.html')).href, join(site, 'og.png'), 1200, 630, 6000);

  const svg = readFileSync(join(site, 'favicon.svg'), 'utf8');
  for (const size of [32, 180, 512]) {
    const page = join(work, `icon-${size}.html`);
    writeFileSync(page, `<!doctype html><html><body style="margin:0;background:#06070a">` +
      `<div style="width:${size}px;height:${size}px">${svg.replace('<svg ', `<svg width="${size}" height="${size}" `)}</div></body></html>`);
    shoot(pathToFileURL(page).href, join(site, `icon-${size}.png`), size, size, 500);
  }
} finally {
  rmSync(work, { recursive: true, force: true });
}
