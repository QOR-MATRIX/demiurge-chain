// SPDX-License-Identifier: Apache-2.0
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import test from 'node:test';

const output = new URL('../dist/', import.meta.url);
const moduleUrl = new URL('analytics.mjs', output);
const { beforeSend } = await import(moduleUrl);
let sequence = 0;

async function loadPage(url, preferences = {}) {
  const scripts = [];
  const messages = [];
  globalThis.window = { location: new URL(url), navigator: preferences };
  globalThis.document = {
    head: {
      querySelector: () => scripts[0],
      appendChild: (script) => scripts.push(script),
    },
    createElement: (tag) => {
      assert.equal(tag, 'script');
      return { dataset: {} };
    },
  };
  const originalLog = console.log;
  console.log = (...args) => messages.push(args.join(' '));
  try {
    await import(`${moduleUrl}?case=${sequence++}`);
    return { scripts, messages, queue: window.vaq, mode: window.vam };
  } finally {
    console.log = originalLog;
    delete globalThis.window;
    delete globalThis.document;
  }
}

test('homepage events discard queries, fragments and extra properties', () => {
  assert.deepEqual(beforeSend({
    type: 'pageview', url: 'https://qorsync.dev/?code=sample-value#sample-fragment',
    extra: 'sample-extra',
  }), { type: 'pageview', url: 'https://qorsync.dev/' });
});

test('private paths, foreign origins, credentials, malformed URLs and custom events are refused', () => {
  for (const url of [
    'https://qorsync.dev/private/sample-value', 'https://id.qorsync.dev/',
    'https://qorsync.dev.example.org/', 'http://qorsync.dev/',
    'https://sample-user@qorsync.dev/', 'invalid', '/',
  ]) assert.equal(beforeSend({ type: 'pageview', url }), null);
  assert.equal(beforeSend({ type: 'event', url: 'https://qorsync.dev/' }), null);
  assert.equal(beforeSend(null), null);
});

test('the installed browser SDK injects one same-origin script after registering redaction', async () => {
  const result = await loadPage('https://qorsync.dev/');
  assert.equal(result.scripts.length, 1);
  assert.equal(result.scripts[0].src, '/_vercel/insights/script.js');
  assert.equal(result.scripts[0].dataset.endpoint, '/_vercel/insights');
  assert.equal(result.scripts[0].dataset.sdkv, '2.0.1');
  assert.equal(result.scripts[0].defer, true);
  assert.equal(result.mode, 'production');
  assert.equal(result.queue[0][0], 'beforeSend');
  assert.equal(result.queue[0][1]({ type: 'event', url: 'https://qorsync.dev/' }), null);
  assert.deepEqual(result.messages, []);
});

test('local, preview, alias, insecure and non-homepage visits do not initialise analytics', async () => {
  for (const url of [
    'http://localhost:3000/', 'https://preview.example.org/',
    'https://www.qorsync.dev/', 'http://qorsync.dev/', 'https://qorsync.dev/private/',
  ]) {
    const result = await loadPage(url);
    assert.deepEqual(result.scripts, []);
    assert.equal(result.queue, undefined);
    assert.deepEqual(result.messages, []);
  }
});

test('browser privacy opt-outs prevent analytics initialisation', async () => {
  for (const preferences of [{ doNotTrack: '1' }, { doNotTrack: 'yes' }, { globalPrivacyControl: true }]) {
    const result = await loadPage('https://qorsync.dev/', preferences);
    assert.deepEqual(result.scripts, []);
    assert.equal(result.queue, undefined);
  }
});

test('build output contains public assets and the unmodified SDK with its licence only', () => {
  const files = readdirSync(output, { recursive: true }).map((file) => file.replaceAll('\\', '/'));
  for (const name of ['node_modules', 'scripts', 'package.json', 'package-lock.json', 'README.md', '.env']) {
    assert.equal(files.includes(name), false);
  }
  assert.equal(readFileSync(new URL('vendor/vercel-analytics.mjs', output), 'utf8'),
    readFileSync(fileURLToPath(import.meta.resolve('@vercel/analytics')), 'utf8'));
  assert.match(readFileSync(new URL('vendor/vercel-analytics.LICENSE.txt', output), 'utf8'), /MIT License/);
  const html = readFileSync(new URL('index.html', output), 'utf8');
  assert.match(html, /<script type="module" src="\/analytics\.mjs"><\/script>/);
  for (const [, asset] of html.matchAll(/(?:href|src)="(\/[^"#?]+)"/g)) {
    assert.ok(files.includes(asset.slice(1)), `missing public asset ${asset}`);
  }
  const config = JSON.parse(readFileSync(new URL('../vercel.json', output), 'utf8'));
  const headers = config.headers.find((rule) => rule.source === '/(.*)').headers;
  const csp = headers.find((header) => header.key === 'Content-Security-Policy').value;
  assert.match(csp, /script-src 'self';/);
  assert.match(csp, /connect-src 'self';/);
  assert.equal(headers.find((header) => header.key === 'Referrer-Policy').value, 'no-referrer');
});
