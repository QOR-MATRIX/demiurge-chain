// The update channel end to end, on this PC (ADR-111, L6.2): a launcher installed, offered an update, and updated, by the
// real updater and the real installer. Not part of `npm run check`: it builds two launchers, installs one, and takes
// some minutes. Windows only.
//
//   node scripts/update-e2e.mjs
//
// It builds a separate product, "QOR Launcher Update Test", with its own identifier, data folder, link scheme and
// install folder, so the launcher installed on this PC is never touched; installed for this Windows user only, so
// nothing asks for permission. Two versions, 9.0.1 and 9.0.2, signed with a key made for this run and thrown away after.
// A server on this PC plays the channel (the only build that allows a plain-HTTP channel is this one, by its own
// configuration). The installed app is driven through WebView2's debugging port, with the same calls Settings makes
// (`update_check`, `update_install`, `update_status`): a fresh install has no profile yet and opens on Who's playing?,
// and the page itself is held by scripts/check-updates-view.mjs.
//
// What it holds the channel to:
//   1. a tampered installer is refused by the plugin's check, nothing is installed, and the launcher keeps running;
//   2. a manifest announcing 9.0.3 with 9.0.2's genuine, correctly signed installer is refused by the host's own check;
//   3. the real update installs without a question asked, and the launcher opens again on 9.0.2, up to date.
// Then the test product is uninstalled and everything it made is removed.

import { execFileSync, spawn, spawnSync } from 'node:child_process';
import { copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { createServer } from 'node:http';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const PRODUCT = 'QOR Launcher Update Test';
const BINARY = 'qor-launcher-update-test';
const IDENTIFIER = 'cloud.demiurge.qor-launcher.update-test';
const PORT = 18490;
const DEBUG_PORT = 9334;
const OLD = '9.0.1';
const NEW = '9.0.2';

if (process.platform !== 'win32') {
  console.error('update-e2e: Windows only.');
  process.exit(2);
}

const work = mkdtempSync(join(tmpdir(), 'qor-update-e2e-'));
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const results = [];
const check = (name, pass, detail = '') => {
  results.push(pass);
  console.log(`${pass ? 'PASS' : 'FAIL'}  ${name}${detail ? `  (${detail})` : ''}`);
};
const installDir = join(process.env.LOCALAPPDATA, PRODUCT);
const installedExe = join(installDir, `${BINARY}.exe`);

// ── a key for this run only ─────────────────────────────────────────────────────────────────────────────────────────
const key = join(work, 'e2e.key');
execFileSync('npx', ['--no-install', 'tauri', 'signer', 'generate', '--ci', '-p', 'e2e', '-w', key], { cwd: root, shell: true, stdio: 'ignore' });
const pubkey = readFileSync(`${key}.pub`, 'utf8').trim();

// ── two builds of the test product ───────────────────────────────────────────────────────────────────────────────────
function build(version) {
  const config = join(work, `e2e-${version}.json`);
  writeFileSync(config, JSON.stringify({
    productName: PRODUCT,
    mainBinaryName: BINARY,
    identifier: IDENTIFIER,
    version,
    bundle: { createUpdaterArtifacts: true, targets: ['nsis'], windows: { nsis: { installMode: 'currentUser' } } },
    plugins: {
      'deep-link': { desktop: { schemes: ['qor-update-test'] } },
      updater: {
        pubkey,
        endpoints: [`http://127.0.0.1:${PORT}/latest.json`],
        dangerousInsecureTransportProtocol: true,
        windows: { installMode: 'passive' },
      },
    },
  }));
  console.log(`update-e2e: building ${PRODUCT} ${version}`);
  const result = spawnSync('npx', ['--no-install', 'tauri', 'build', '--config', config], {
    cwd: root, shell: true, stdio: ['ignore', 'ignore', 'inherit'],
    // The bundler reads the key's contents from TAURI_SIGNING_PRIVATE_KEY, not a path.
    env: { ...process.env, TAURI_SIGNING_PRIVATE_KEY: readFileSync(key, 'utf8').trim(), TAURI_SIGNING_PRIVATE_KEY_PASSWORD: 'e2e' },
  });
  if (result.status !== 0) throw new Error(`the ${version} build failed (${result.status})`);
  const name = `${PRODUCT}_${version}_x64-setup.exe`;
  const built = join(root, 'src-tauri', 'target', 'release', 'bundle', 'nsis', name);
  const kept = join(work, name);
  copyFileSync(built, kept);
  copyFileSync(`${built}.sig`, `${kept}.sig`);
  return { name, path: kept, signature: readFileSync(`${kept}.sig`, 'utf8').trim() };
}

// ── the channel, played here ───────────────────────────────────────────────────────────────────────────────────────
let served = { manifest: null, installer: null };
const server = createServer((req, res) => {
  if (req.url === '/latest.json' && served.manifest) return res.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify(served.manifest));
  if (req.url === '/installer.exe' && served.installer) return res.writeHead(200, { 'content-type': 'application/octet-stream' }).end(served.installer);
  res.writeHead(404).end();
});
const offer = (version, installer, signature) => {
  served = {
    installer,
    manifest: { version, notes: `The test product, ${version}.`, pub_date: '2026-10-22T00:00:00Z',
      platforms: { 'windows-x86_64': { signature, url: `http://127.0.0.1:${PORT}/installer.exe` } } },
  };
};

// ── driving the installed app ──────────────────────────────────────────────────────────────────────────────────────
let app = null;
function start() {
  app = spawn(installedExe, [], {
    detached: true, stdio: 'ignore',
    env: { ...process.env, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${DEBUG_PORT}` },
  });
  app.unref();
}
function stop() {
  spawnSync('taskkill', ['/F', '/T', '/IM', `${BINARY}.exe`], { stdio: 'ignore' });
}
async function connect(ms = 60000) {
  const deadline = Date.now() + ms;
  for (;;) {
    try {
      const targets = await (await fetch(`http://127.0.0.1:${DEBUG_PORT}/json/list`)).json();
      const page = targets.find((t) => t.type === 'page');
      if (page) {
        const ws = new WebSocket(page.webSocketDebuggerUrl);
        await new Promise((ok, bad) => { ws.onopen = ok; ws.onerror = bad; });
        let id = 0;
        const pending = new Map();
        ws.onmessage = (m) => { const msg = JSON.parse(m.data); pending.get(msg.id)?.(msg); pending.delete(msg.id); };
        const evaluate = (expression) => new Promise((ok) => {
          const n = ++id;
          pending.set(n, (msg) => ok(msg.result?.result?.value));
          ws.send(JSON.stringify({ id: n, method: 'Runtime.evaluate', params: { expression, returnByValue: true, awaitPromise: true } }));
        });
        const eventually = async (expression, wait = 30000) => {
          const until = Date.now() + wait;
          for (;;) {
            try { if (await evaluate(expression)) return true; } catch { /* a page between loads */ }
            if (Date.now() > until) return false;
            await sleep(250);
          }
        };
        return { ws, evaluate, eventually };
      }
    } catch { /* not up yet */ }
    if (Date.now() > deadline) throw new Error('the installed launcher exposed no page to drive');
    await sleep(500);
  }
}
// The calls Settings makes, from the installed app's own page.
const ipc = (command) => `window.__TAURI_INTERNALS__.invoke('${command}').then((v) => JSON.stringify(v ?? null), (e) => JSON.stringify({ refused: e }))`;
const view = async (page, command = 'update_status') => JSON.parse((await page.evaluate(ipc(command))) ?? 'null');
const installedVersion = () => {
  const out = spawnSync('pwsh', ['-NoProfile', '-Command', `(Get-Item '${installedExe}').VersionInfo.ProductVersion`], { encoding: 'utf8' });
  return out.stdout.trim();
};

async function offeredAndRefused(what, version, expectWords) {
  start();
  const page = await connect();
  await page.eventually(`Boolean(window.__TAURI_INTERNALS__)`);
  const found = await view(page, 'update_check');
  check(`${what}: the update is found and offered`, found?.status?.state === 'available' && found.status.version === version, JSON.stringify(found?.status));
  const answer = JSON.parse(await page.evaluate(ipc('update_install')));
  const after = await view(page);
  const said = after?.status?.message ?? '';
  check(`${what}: it is refused, in the host's words`, Boolean(answer?.refused) && after?.status?.state === 'failed' && expectWords.test(said), said);
  await sleep(1500);
  check(`${what}: the launcher is still running`, (await page.evaluate('1 + 1')) === 2);
  check(`${what}: nothing was installed`, installedVersion() === OLD, installedVersion());
  page.ws.close();
  stop();
  await sleep(1500);
}

try {
  stop();
  const old = build(OLD);
  const fresh = build(NEW);
  await new Promise((r) => server.listen(PORT, '127.0.0.1', r));

  // The old version, installed for this user.
  const installed = spawnSync(old.path, ['/S'], { stdio: 'ignore' });
  check(`${PRODUCT} ${OLD} installs for this user without a question`, installed.status === 0 && existsSync(installedExe));
  check(`and it is ${OLD}`, installedVersion() === OLD, installedVersion());

  // 1. A tampered installer.
  const tampered = Buffer.from(readFileSync(fresh.path));
  tampered[tampered.length - 1] ^= 1;
  offer(NEW, tampered, fresh.signature);
  await offeredAndRefused('a tampered installer', NEW, /could not be fetched or did not verify/);

  // 2. Another version's installer, its signature genuine: only the host's own check sees it.
  offer('9.0.3', readFileSync(fresh.path), fresh.signature);
  await offeredAndRefused("9.0.2's installer announced as 9.0.3", '9.0.3', new RegExp(`signed as ${fresh.name.replaceAll('.', '\\.')}`));

  // 3. The real update.
  offer(NEW, readFileSync(fresh.path), fresh.signature);
  start();
  let page = await connect();
  await page.eventually(`Boolean(window.__TAURI_INTERNALS__)`);
  const found = await view(page, 'update_check');
  check(`the real ${NEW} is found and offered`, found?.status?.state === 'available' && found.status.version === NEW, JSON.stringify(found?.status));
  // On Windows a verified install closes the launcher, so this call never answers.
  await page.evaluate(`(window.__TAURI_INTERNALS__.invoke('update_install'), true)`);
  page.ws.close();
  // The launcher closes, the installer runs without a question, and opens it again.
  let updated = false;
  for (let i = 0; i < 120 && !updated; i++) {
    await sleep(1000);
    updated = installedVersion() === NEW;
  }
  check(`the installer put ${NEW} in place without a question`, updated, installedVersion());
  page = await connect(90000);
  await page.eventually(`Boolean(window.__TAURI_INTERNALS__)`);
  check('and the launcher opened again', (await page.evaluate('1 + 1')) === 2);
  const now = await view(page, 'update_check');
  check(`on ${NEW}, it is up to date`, now?.current === NEW && now?.status?.state === 'up_to_date', JSON.stringify(now));
  page.ws.close();
} catch (error) {
  check('the run reached its end', false, error.message);
} finally {
  stop();
  server.close();
  const uninstaller = join(installDir, 'uninstall.exe');
  if (existsSync(uninstaller)) spawnSync(uninstaller, ['/S'], { stdio: 'ignore' });
  await sleep(3000);
  for (const dir of [installDir, join(process.env.APPDATA, IDENTIFIER), join(process.env.LOCALAPPDATA, IDENTIFIER)])
    rmSync(dir, { recursive: true, force: true });
  rmSync(work, { recursive: true, force: true });
  console.log(`update-e2e: removed ${PRODUCT}, its data and the run's key`);
}

const failed = results.filter((r) => !r).length;
console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
process.exit(failed ? 1 : 0);
