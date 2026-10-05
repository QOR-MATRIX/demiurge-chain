// Rehearse a runtime upgrade before the owner signs it on the devnet.
//
// Against a LOCAL node started from a development specification whose genesis
// code was replaced by the devnet's own runtime (read from wss://rpc.qorsync.dev
// with state_getStorage(":code")), this script:
//   1. checks the node runs the old runtime and has Sudo;
//   2. leaves state behind that the upgrade must keep: a transfer and a DRC-369 mint;
//   3. upgrades exactly as the owner will, sudo.sudoUncheckedWeight(system.setCode(wasm));
//   4. checks the new spec_version, that Sudo survived with the same key, that the
//      transfer and the asset are still there, and that the new pallet works.
//
// It signs only with the well-known development keys, and refuses any node whose
// chain type is not Development. It never touches the devnet.
//
//   node rehearse-upgrade.mjs ws://127.0.0.1:9955 <path to new .compact.compressed.wasm> <expected spec_version>

import { readFileSync } from 'node:fs';
import { ApiPromise, WsProvider, Keyring } from '@polkadot/api';
import { cryptoWaitReady } from '@polkadot/util-crypto';

const [endpoint, wasmPath, expected] = process.argv.slice(2);
if (!endpoint || !wasmPath || !expected) throw new Error('usage: rehearse-upgrade.mjs <ws endpoint> <wasm> <expected spec_version>');

function check(ok, what) {
  console.log(`${ok ? 'ok  ' : 'FAIL'} ${what}`);
  if (!ok) process.exitCode = 1;
}

/** Sign, send, and resolve at finality with the events, rejecting a dispatch error. */
function send(api, tx, signer) {
  return new Promise((resolve, reject) => {
    tx.signAndSend(signer, ({ status, events, dispatchError }) => {
      if (dispatchError) {
        const e = dispatchError.isModule ? api.registry.findMetaError(dispatchError.asModule) : null;
        reject(new Error(e ? `${e.section}.${e.name}` : dispatchError.toString()));
      } else if (status.isFinalized) {
        resolve(events);
      }
    }).catch(reject);
  });
}

await cryptoWaitReady();
const keyring = new Keyring({ type: 'sr25519' });
const [alice, bob, charlie] = ['//Alice', '//Bob', '//Charlie'].map((uri) => keyring.addFromUri(uri));
let api = await ApiPromise.create({ provider: new WsProvider(endpoint), noInitWarn: true });

const chainType = (await api.rpc.system.chainType()).toString();
if (chainType !== 'Development') throw new Error(`refusing a ${chainType} chain: this script is for a local rehearsal only`);

const before = api.runtimeVersion.specVersion.toNumber();
console.log(`node runs ${api.runtimeVersion.specName} spec_version ${before}`);
check(before < Number(expected), `the running spec_version ${before} is below ${expected}`);
const sudoKey = (await api.query.sudo.key()).toString();
check(sudoKey === alice.address, 'Sudo is present and its key is Alice');

// State the upgrade must keep.
const CGT = 10n ** 18n;
await send(api, api.tx.balances.transferKeepAlive(charlie.address, 1234n * CGT), alice);
const charlieBefore = (await api.query.system.account(charlie.address)).data.free.toBigInt();
const content = { algo: 'Blake3_256', root: '0x' + '11'.repeat(32), size: 512 };
const mintEvents = await send(api, api.tx.drc369.mint(content, { Sha1: '0x' + '22'.repeat(20) }, 'cartridge', true, null), alice);
const minted = mintEvents.find(({ event }) => event.section === 'drc369' && event.method === 'Minted').event.data;
const [collection, item] = [minted.collection.toNumber(), minted.item.toNumber()];
console.log(`minted (${collection}, ${item}); Charlie holds ${charlieBefore / CGT} CGT`);

// The upgrade, exactly as the owner will sign it.
const wasm = readFileSync(wasmPath);
console.log(`setting ${wasm.length} bytes of code`);
await send(api, api.tx.sudo.sudoUncheckedWeight(api.tx.system.setCode('0x' + wasm.toString('hex')), { refTime: 0, proofSize: 0 }), alice);

// The new runtime takes effect from the next block; reconnect to read its metadata.
await new Promise((r) => setTimeout(r, 15000));
await api.disconnect();
api = await ApiPromise.create({ provider: new WsProvider(endpoint), noInitWarn: true });
const after = api.runtimeVersion.specVersion.toNumber();
check(after === Number(expected), `spec_version is now ${after}, expected ${expected}`);
check(api.runtimeVersion.specName.toString() === 'demiurge', 'spec_name is still demiurge');
check(!!api.tx.sudo && (await api.query.sudo.key()).toString() === alice.address, 'Sudo survived, with the same key');
check((await api.query.system.account(charlie.address)).data.free.toBigInt() === charlieBefore, 'the transfer is still there');
check((await api.query.nfts.item(collection, item)).isSome, 'the minted asset is still there');
check(!!api.tx.arqWallet, 'ArqWallet is in the new metadata');

// The new pallet works on the upgraded chain.
const policy = {
  maxPayout: 10n * CGT, epochBlocks: 100, epochBudget: 1000n * CGT, perRecipientPerEpoch: 50n * CGT,
  ruleVersions: ['flux@1'], accrualExpiry: 1000,
  loosenDelay: api.consts.arqWallet.minLoosenDelay.toNumber(),
};
await send(api, api.tx.arqWallet.create(collection, item, policy, 1000n * CGT), alice);
await send(api, api.tx.arqWallet.setAuthority(collection, item, bob.address), alice);
const now = (await api.rpc.chain.getHeader()).number.toNumber();
const paid = await send(api, api.tx.arqWallet.payout(collection, item, '0x' + '33'.repeat(32), now, 'flux@1', charlie.address, 5n * CGT), bob);
check(paid.some(({ event }) => event.section === 'arqWallet' && event.method === 'Paid'), 'an ARQ Wallet paid a player after the upgrade');
check((await api.query.system.account(charlie.address)).data.free.toBigInt() === charlieBefore + 5n * CGT, 'the player received exactly 5 CGT');

// Rounds (spec_version 8 onwards): a prize held, then settled to a winner.
if (api.tx.arqWallet.openRound) {
  const round = '0x' + '44'.repeat(32);
  const opensAt = (await api.rpc.chain.getHeader()).number.toNumber();
  await send(api, api.tx.arqWallet.openRound(collection, item, round, 20n * CGT, 'flux@1', opensAt + 2), bob);
  check((await api.query.arqWallet.held([collection, item])).toBigInt() === 20n * CGT, 'a round holds its 20 CGT prize');
  while ((await api.rpc.chain.getHeader()).number.toNumber() < opensAt + 2) await new Promise((r) => setTimeout(r, 2000));
  const settled = await send(api, api.tx.arqWallet.settleRound(collection, item, round, [[charlie.address, 15n * CGT]]), bob);
  check(settled.some(({ event }) => event.section === 'arqWallet' && event.method === 'RoundSettled'), 'the round settled');
  check((await api.query.arqWallet.held([collection, item])).toBigInt() === 0n, 'nothing is held after settlement');
  check((await api.query.system.account(charlie.address)).data.free.toBigInt() === charlieBefore + 20n * CGT, 'the winner received exactly 15 CGT more');
}

await api.disconnect();
console.log(process.exitCode ? 'REHEARSAL FAILED' : 'REHEARSAL PASSED');
