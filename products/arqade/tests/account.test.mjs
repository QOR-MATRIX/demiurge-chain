import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

// The reader, transpiled with its imports rewritten, against bytes a real node produced: a local --dev chain at
// spec_version 8, after Alice minted two named assets, read at the finalized block (4 October 2026). @polkadot/api
// decoded the same balance as free 0x000000000000d3702653ceaf6f4c8000 and reserved 0x0000000000000051f57afe3e31b38000.
const dir=path.resolve('work/account-tests');fs.mkdirSync(dir,{recursive:true});
const rewrite=s=>s.replace("from './amount'","from './amount.mjs'").replace("from './arq-wallet-policy'","from './arq-wallet-policy.mjs'")
  .replace("from '../sdk/src/amount'","from './amount.mjs'").replace("from '../sdk/src/arq-wallet'","from './arq-wallet.mjs'").replace("from './chain'","from './chain.mjs'");
for(const [src,name] of [['sdk/src/amount.ts','amount'],['sdk/src/arq-wallet-policy.ts','arq-wallet-policy'],['sdk/src/arq-wallet.ts','arq-wallet'],['lib/chain.ts','chain'],['lib/account.ts','account']]){
  fs.writeFileSync(path.join(dir,name+'.mjs'),ts.transpileModule(rewrite(fs.readFileSync(src,'utf8')),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
}
const A=await import(pathToFileURL(path.join(dir,'account.mjs')).href);
const C=await import(pathToFileURL(path.join(dir,'chain.mjs')).href);
const W=await import(pathToFileURL(path.join(dir,'arq-wallet.mjs')).href);

const ALICE='5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const KEY='0x26aa394eea5630e07c48ae0c9558cef7b99d880ec681799c0cf30e8886371da9de1e86a9a8c739864cf3cc5ec2bea59fd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d';
const RAW_ACCOUNT='0x0200000002000000010000000000000000804c6fafce532670d30000000000000080b3313efe7af551000000000000000000000000000000000000000000000000000000000000000000000000000080';
const RAW_ASSETS='0x08000000000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa000200000000000000aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa00020000000000000100222222222222222222222222222222222222222201000024426c75652068756c6c000000000100000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb000200000000000000bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb00020000000000000101333333333333333333333333333333333333333333333333333333333333333300010000000000000000012852656d697820e2849632';
const FREE=BigInt('0x000000000000d3702653ceaf6f4c8000');
const RESERVED=BigInt('0x0000000000000051f57afe3e31b38000');
const CGT=10n**18n;

test('the storage key of an account is the one the chain uses',()=>{
  assert.equal(A.systemAccountKey(W.ss58Decode(ALICE)),KEY);
});

test('a balance decodes exactly as @polkadot/api decoded it, and spendable keeps the account open',()=>{
  const b=A.decodeAccount(RAW_ACCOUNT);
  assert.equal(b.free,FREE);
  assert.equal(b.reserved,RESERVED);
  assert.equal(b.frozen,0n);
  assert.equal(b.spendable,FREE-100n*CGT);
  assert.deepEqual(A.decodeAccount(null),{free:0n,reserved:0n,frozen:0n,spendable:0n});
  assert.throws(()=>A.decodeAccount(RAW_ACCOUNT.slice(0,-2)),/Invalid chain response/);
  assert.throws(()=>A.decodeAccount(RAW_ACCOUNT+'00'),/Invalid chain response/);
});

test('a frozen amount beyond what is reserved is not spendable',()=>{
  const u128=x=>Buffer.from(x.toString(16).padStart(32,'0'),'hex').reverse().toString('hex');
  const raw='0x'+'00'.repeat(16)+u128(1000n*CGT)+u128(0n)+u128(600n*CGT)+u128(0n);
  assert.equal(A.decodeAccount(raw).spendable,400n*CGT);
});

test('owned assets decode as the chain encoded them: names, content, commits, remix provenance',()=>{
  assert.deepEqual(A.decodeOwnedAssets(RAW_ASSETS),[
    {collection:0,item:0,name:'Blue hull',content:'blake3:'+'aa'.repeat(32),origin:'blake3:'+'aa'.repeat(32),revisable:true,derivedFrom:null,remixDepth:0},
    {collection:0,item:1,name:'Remix №2',content:'blake3:'+'bb'.repeat(32),origin:'blake3:'+'bb'.repeat(32),revisable:false,derivedFrom:[0,0],remixDepth:1},
  ]);
  assert.deepEqual(A.decodeOwnedAssets('0x00'),[]);
  assert.throws(()=>A.decodeOwnedAssets(RAW_ASSETS.slice(0,-4)),/Invalid chain response/);
  assert.throws(()=>A.decodeOwnedAssets('0x04'),/Invalid chain response/);
});

test('an address for another network or with a broken checksum is refused before anything is read',async()=>{
  const calls=[];const rpc=async m=>{calls.push(m);return null};
  await assert.rejects(A.readAccount(rpc,W.ss58(W.ss58Decode(ALICE),0)),/another network/i);
  await assert.rejects(A.readAccount(rpc,ALICE.slice(0,-1)+(ALICE.endsWith('Y')?'Z':'Y')),/checksum/i);
  await assert.rejects(A.readAccount(rpc,'not an address'),/SS58/);
  assert.deepEqual(calls,[]);
});

test('readAccount checks the network, reads at the finalized block and returns amounts as strings',async()=>{
  const FIN='0x'+'ab'.repeat(32);const seen=[];
  const rpc=async(m,p)=>{seen.push([m,p]);switch(m){
    case 'chain_getBlockHash':return C.DEVNET.genesis;case 'system_chain':return C.DEVNET.name;
    case 'chain_getFinalizedHead':return FIN;case 'chain_getHeader':return {number:'0x10'};
    case 'state_getStorage':return RAW_ACCOUNT;case 'state_call':return RAW_ASSETS;default:throw Error(m)}};
  const s=await A.readAccount(rpc,ALICE,()=>new Date('2026-10-04T00:00:00Z'));
  assert.equal(s.network,'Demiurge Devnet');
  assert.equal(s.balance.free,FREE.toString());
  assert.equal(s.assetCount,2);assert.equal(s.assets[1].name,'Remix №2');
  assert.deepEqual(seen.find(([m])=>m==='state_getStorage')[1],[KEY,FIN]);
  assert.deepEqual(seen.find(([m])=>m==='state_call')[1],['Drc369Api_assets_of','0x'+Buffer.from(W.ss58Decode(ALICE)).toString('hex'),FIN]);
  const wrong=async(m,p)=>m==='chain_getBlockHash'?'0x'+'00'.repeat(32):rpc(m,p);
  await assert.rejects(A.readAccount(wrong,ALICE),/Wrong network/);
});
