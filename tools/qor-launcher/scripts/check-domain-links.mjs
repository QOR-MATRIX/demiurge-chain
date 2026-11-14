// SPDX-License-Identifier: Apache-2.0
// Node 24 strips the registry's types. No frontend, service or credential is opened.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { SYSTEMS } from '../src/lib/systems.ts';

const developer = SYSTEMS.find((entry) => entry.id === 'developers');
assert.equal(developer.state, 'live');
assert.equal(developer.url, 'https://github.com/QOR-MATRIX/QOR-OS/blob/main/sdk/README.md');
assert.match(developer.tagline, /repository access/i);
for (const id of ['explorer', 'studio', 'sophia', 'scatter', 'agents', 'bounties']) {
  const entry = SYSTEMS.find((item) => item.id === id);
  assert.equal(entry.state, 'forming', id);
  assert.equal(entry.url, undefined, id);
  assert.equal(entry.localUrl, undefined, id);
  assert.match(entry.blockedOn, /frozen/i, id);
}
const config = JSON.parse(readFileSync(new URL('../src-tauri/tauri.conf.json', import.meta.url), 'utf8'));
assert.equal(config.identifier, 'cloud.demiurge.qor-launcher');
const sources = config.app.security.csp['connect-src'].split(/\s+/);
assert(sources.includes('https://qorsync.dev'));
assert(sources.includes('https://rpc.qorsync.dev'));
assert(!sources.includes('*'));
console.log('Launcher links: canonical developer guide, frozen integrations disabled, application identity retained.');
