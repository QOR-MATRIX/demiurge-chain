// Build a launcher release for the update channel, check what was built, and publish it only when asked (ADR-111, L6.2).
//
//   node scripts/release.mjs --notes notes.md              # build, sign, check, and write latest.json; publish nothing
//   node scripts/release.mjs --notes notes.md --publish    # and then publish the release and replace the channel's manifest
//   node scripts/release.mjs --self-test                   # check this script's own signature check on the test fixtures
//
// What it does, in order:
//   1. Reads the version from tauri.conf.json, which package.json and Cargo.toml must match.
//   2. Takes the update key from this PC (scripts/updater-key.ps1): its file's path, and its password, handed to the
//      build in its environment only; neither is written anywhere or printed.
//   3. Builds as `npm run app:build` does, with tauri.release.conf.json too, so the installer is signed.
//   4. Checks the signature it produced against the public key in tauri.conf.json, and that it was made for this
//      version's installer (`QOR Launcher_<version>_x64-setup.exe`), as the launcher itself will before installing.
//   5. Writes latest.json beside the installer: the version, the notes, the date, and the installer's address and
//      signature.
//   6. Only with --publish: makes the pre-release `launcher-v<version>` with the installer, then replaces latest.json in
//      the pre-release `launcher-channel`, reads it back from its public address, and downloads the installer it names
//      and checks it again: the same bytes as built, and a signature that verifies.
//
// Windows only, like the installer that carries QOR Media and ARQADE.

import { execFileSync, spawnSync } from 'node:child_process';
import { createHash, createPublicKey, verify } from 'node:crypto';
import { copyFileSync, existsSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const REPO = 'QOR-MATRIX/demiurge-chain';
const CHANNEL_TAG = 'launcher-channel';
const CHANNEL = `https://github.com/${REPO}/releases/download/${CHANNEL_TAG}/latest.json`;
const PRODUCT = 'QOR Launcher';

const args = process.argv.slice(2);
const flag = (name) => args.includes(name);
const option = (name) => {
  const at = args.indexOf(name);
  return at >= 0 ? args[at + 1] : undefined;
};
const fail = (message) => {
  console.error(`release: ${message}`);
  process.exit(1);
};

// ── minisign, as the launcher checks it ─────────────────────────────────────────────────────────────────────────────

/** The lines of a minisign file Tauri wrote: it base64-encodes the whole text. */
const lines = (b64) => Buffer.from(b64.trim(), 'base64').toString('utf8').split('\n').map((l) => l.trim());

function ed25519(publicKey) {
  // An Ed25519 public key in SubjectPublicKeyInfo form: a fixed prefix, then the key's 32 bytes.
  const der = Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), publicKey]);
  return createPublicKey({ key: der, format: 'der', type: 'spki' });
}

/**
 * Verify `data` against a Tauri signature and public key (each the base64 of minisign's text), and return the name of
 * the file the signature was made for. Refuses a legacy signature, as the launcher does: the signature is over the
 * BLAKE2b-512 of the data ("ED"), and the trusted comment is covered by the global signature.
 */
export function verifySignature(data, signatureB64, publicKeyB64) {
  const keyLine = Buffer.from(lines(publicKeyB64)[1], 'base64');
  const [, sigLine, trustedLine, globalLine] = lines(signatureB64);
  const sig = Buffer.from(sigLine, 'base64');
  const global = Buffer.from(globalLine, 'base64');
  if (keyLine.length !== 42 || keyLine.subarray(0, 2).toString() !== 'Ed') throw new Error('the public key is not minisign Ed25519');
  if (sig.length !== 74) throw new Error('the signature is not minisign Ed25519');
  if (sig.subarray(0, 2).toString() !== 'ED') throw new Error('a legacy signature is refused');
  if (!sig.subarray(2, 10).equals(keyLine.subarray(2, 10))) throw new Error('the signature names another key');
  const key = ed25519(keyLine.subarray(10));
  const digest = createHash('blake2b512').update(data).digest();
  if (!verify(null, digest, key, sig.subarray(10))) throw new Error('the signature does not match the data and key');
  if (!trustedLine.startsWith('trusted comment: ')) throw new Error('the signature has no trusted comment');
  const trusted = trustedLine.slice('trusted comment: '.length);
  if (!verify(null, Buffer.concat([sig.subarray(10), Buffer.from(trusted, 'utf8')]), key, global))
    throw new Error('the trusted comment is not covered by the signature');
  const file = trusted.split('\t').find((f) => f.startsWith('file:'))?.slice('file:'.length);
  if (!file) throw new Error('the signature names no file');
  return file;
}

const installerName = (version) => `${PRODUCT}_${version}_x64-setup.exe`;

// ── --self-test ─────────────────────────────────────────────────────────────────────────────────────────────────────

if (flag('--self-test')) {
  const fixtures = join(root, 'src-tauri', 'tests', 'fixtures', 'update');
  const data = readFileSync(join(fixtures, `${installerName('9.9.9')}`));
  const sig = readFileSync(join(fixtures, `${installerName('9.9.9')}.sig`), 'utf8');
  const key = readFileSync(join(fixtures, 'test.key.pub.b64'), 'utf8');
  const results = [];
  const expect = (name, fn, ok) => {
    let passed;
    try {
      const got = fn();
      passed = ok(got, null);
    } catch (e) {
      passed = ok(null, e);
    }
    results.push(passed);
    console.log(`${passed ? 'PASS' : 'FAIL'}  ${name}`);
  };
  expect('the fixture verifies, signed for its own name', () => verifySignature(data, sig, key), (file) => file === installerName('9.9.9'));
  const changed = Buffer.from(data);
  changed[0] ^= 1;
  expect('a changed byte is refused', () => verifySignature(changed, sig, key), (_, e) => /does not match/.test(e?.message ?? ''));
  const channelKey = JSON.parse(readFileSync(join(root, 'src-tauri', 'tauri.conf.json'), 'utf8')).plugins.updater.pubkey;
  expect("the channel's own key did not sign it", () => verifySignature(data, sig, channelKey), (_, e) => /another key|does not match/.test(e?.message ?? ''));
  const failed = results.filter((r) => !r).length;
  console.log(`RESULT: ${results.length - failed} passed, ${failed} failed`);
  process.exit(failed ? 1 : 0);
}

// ── a release ───────────────────────────────────────────────────────────────────────────────────────────────────────

if (process.platform !== 'win32') fail('releases are built on Windows, where the installer carries QOR Media and ARQADE.');
const notesPath = option('--notes');
if (!notesPath || !existsSync(notesPath)) fail('say what changed: --notes <file> (plain text: the launcher shows it as written, and GitHub as Markdown).');
const notes = readFileSync(notesPath, 'utf8').trim();

// 1. The version, in every place that names it.
const conf = JSON.parse(readFileSync(join(root, 'src-tauri', 'tauri.conf.json'), 'utf8'));
const version = conf.version;
const pkg = JSON.parse(readFileSync(join(root, 'package.json'), 'utf8')).version;
const cargo = /^version\s*=\s*"([^"]+)"/m.exec(readFileSync(join(root, 'src-tauri', 'Cargo.toml'), 'utf8'))?.[1];
if (pkg !== version || cargo !== version) fail(`versions differ: tauri.conf.json ${version}, package.json ${pkg}, Cargo.toml ${cargo}.`);
const tag = `launcher-v${version}`;
console.log(`release: QOR Launcher ${version}`);

// 2. The key, from this PC.
const keyScript = join(root, 'scripts', 'updater-key.ps1');
const keyPath = join(process.env.APPDATA ?? '', 'Demiurge', 'updater', 'qor-launcher.key');
if (!existsSync(keyPath)) fail(`no update key on this PC (${keyPath}). See scripts/updater-key.ps1.`);
const password = execFileSync('pwsh', ['-NoProfile', '-File', keyScript, '-Password'], { encoding: 'utf8' }).trim();

// 3. The build, signed. The bundler reads the key from TAURI_SIGNING_PRIVATE_KEY (its file's contents) and ignores
// TAURI_SIGNING_PRIVATE_KEY_PATH, which only `tauri signer sign` reads.
const env = {
  ...process.env,
  TAURI_SIGNING_PRIVATE_KEY: readFileSync(keyPath, 'utf8').trim(),
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: password,
};
const run = (command, commandArgs) => {
  const result = spawnSync(command, commandArgs, { cwd: root, env, stdio: 'inherit', shell: process.platform === 'win32' });
  if (result.status !== 0) fail(`${command} ${commandArgs.join(' ')} failed (${result.status}).`);
};
run('node', ['scripts/build-helper.mjs']);
run('node', ['scripts/stage-qor-media.mjs']);
run('node', ['scripts/stage-arqade.mjs']);
run('npx', ['--no-install', 'tauri', 'build', '--config', 'src-tauri/tauri.bundle.conf.json', '--config', 'src-tauri/tauri.release.conf.json']);

// 4. What was built, checked as the launcher will check it.
const bundle = join(root, 'src-tauri', 'target', 'release', 'bundle', 'nsis');
const installer = join(bundle, installerName(version));
if (!existsSync(installer) || !existsSync(`${installer}.sig`)) fail(`no signed installer at ${installer}.`);
const signature = readFileSync(`${installer}.sig`, 'utf8').trim();
let signedFor;
try {
  signedFor = verifySignature(readFileSync(installer), signature, conf.plugins.updater.pubkey);
} catch (e) {
  fail(`the installer does not verify against tauri.conf.json's key: ${e.message}.`);
}
if (signedFor !== installerName(version)) fail(`the installer was signed as ${signedFor}, not ${installerName(version)}.`);
console.log(`release: the installer verifies against the channel's key, signed as ${signedFor}`);

// 5. The manifest.
const asset = `QOR-Launcher_${version}_x64-setup.exe`;
const url = `https://github.com/${REPO}/releases/download/${tag}/${asset}`;
const manifest = {
  version,
  notes,
  pub_date: new Date().toISOString().replace(/\.\d{3}Z$/, 'Z'),
  platforms: {
    'windows-x86_64-nsis': { signature, url },
    'windows-x86_64': { signature, url },
  },
};
const manifestPath = join(bundle, 'latest.json');
writeFileSync(manifestPath, `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`release: wrote ${manifestPath}`);

if (!flag('--publish')) {
  console.log('release: built and checked; nothing published (add --publish).');
  process.exit(0);
}

// 6. Publishing: the version's pre-release, then the channel's manifest.
const git = (...a) => execFileSync('git', a, { cwd: root, encoding: 'utf8' }).trim();
if (git('status', '--porcelain')) fail('the working tree has changes; publish from a clean, pushed commit.');
const commit = git('rev-parse', 'HEAD');
const gh = (...a) => {
  const result = spawnSync('gh', a, { cwd: root, encoding: 'utf8' });
  return { ok: result.status === 0, out: `${result.stdout}${result.stderr}`.trim() };
};
if (gh('release', 'view', tag, '--repo', REPO).ok) fail(`${tag} is already published.`);
const staging = mkdtempSync(join(tmpdir(), 'qor-release-'));
try {
  const named = join(staging, asset);
  copyFileSync(installer, named);
  const notesFile = join(staging, 'notes.md');
  writeFileSync(notesFile, `${notes}\n`);
  const made = gh('release', 'create', tag, named, '--repo', REPO, '--prerelease', '--target', commit,
    '--title', `QOR Launcher ${version} (tester build)`, '--notes-file', notesFile);
  if (!made.ok) fail(`could not publish ${tag}: ${made.out}`);
  if (!gh('release', 'view', CHANNEL_TAG, '--repo', REPO).ok) {
    const channel = gh('release', 'create', CHANNEL_TAG, '--repo', REPO, '--prerelease', '--target', commit,
      '--title', 'QOR Launcher update channel',
      '--notes', 'The launcher reads latest.json here to learn of a new version (ADR-111). Install from the version\'s own release.');
    if (!channel.ok) fail(`could not make ${CHANNEL_TAG}: ${channel.out}`);
  }
  const uploaded = gh('release', 'upload', CHANNEL_TAG, manifestPath, '--repo', REPO, '--clobber');
  if (!uploaded.ok) fail(`could not replace the channel's manifest: ${uploaded.out}`);
} finally {
  rmSync(staging, { recursive: true, force: true });
}

// Read back from the public addresses, as a launcher will: the manifest, then the installer it names, checked again.
// A just-published asset can take a moment to be served, so each is asked for a few times before giving up.
const fetchBack = async (address) => {
  for (let attempt = 1; ; attempt++) {
    const response = await fetch(address, { redirect: 'follow', cache: 'no-store' }).catch((e) => ({ ok: false, status: e.message }));
    if (response.ok) return response;
    if (attempt === 6) fail(`${address} answers ${response.status} after publishing.`);
    await new Promise((r) => setTimeout(r, 5000));
  }
};
const live = await (await fetchBack(CHANNEL)).json();
const entry = live.platforms?.['windows-x86_64'];
if (live.version !== version || entry?.signature !== signature || entry?.url !== url)
  fail(`the channel answers ${live.version} at ${entry?.url}, not what was published.`);
const fetched = Buffer.from(await (await fetchBack(entry.url)).arrayBuffer());
const sha256 = (bytes) => createHash('sha256').update(bytes).digest('hex');
if (sha256(fetched) !== sha256(readFileSync(installer))) fail(`the installer at ${entry.url} is not the one built.`);
try {
  if (verifySignature(fetched, entry.signature, conf.plugins.updater.pubkey) !== installerName(version)) throw new Error('signed for another file');
} catch (e) {
  fail(`the published installer does not verify: ${e.message}.`);
}
console.log(`release: published ${tag}; ${CHANNEL} announces ${version}, and its installer downloads back, ` +
  `${fetched.length} bytes, SHA-256 ${sha256(fetched)}, verified against the channel's key.`);
