import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

const dir=path.resolve('work/sdk-wallet-tests');fs.mkdirSync(dir,{recursive:true});
for(const name of ['amount','arq-wallet-policy','arq-wallet']){
  const src=fs.readFileSync(`sdk/src/${name}.ts`,'utf8').replace("from './amount'","from './amount.mjs'").replace("from './arq-wallet-policy'","from './arq-wallet-policy.mjs'");
  fs.writeFileSync(path.join(dir,name+'.mjs'),ts.transpileModule(src,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
}
const W=await import(pathToFileURL(path.join(dir,'arq-wallet.mjs')).href);
const hex=b=>Buffer.from(b).toString('hex');

test('SS58 encodes a known account exactly (Alice, prefix 42)',()=>{
  const alice=Buffer.from('d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d','hex');
  assert.equal(W.ss58(alice),'5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY');
  assert.throws(()=>W.ss58(alice.subarray(1)),/32 bytes/);
});

test('a wallet account is modl, the pallet id and the Cartridge, zero-padded: the bytes the chain pins',()=>{
  // chain/pallets/arq-wallet/src/tests.rs pins the same hex for (1, 2).
  assert.equal(hex(W.arqWalletAccountId(1,2)),'6d6f646c646d672f617271770100000002000000000000000000000000000000');
  assert.notEqual(W.arqWalletAddress(1,2),W.arqWalletAddress(2,1));
  assert.match(W.arqWalletAddress(0,0),/^5[1-9A-HJ-NP-Za-km-z]{47}$/);
  assert.throws(()=>W.arqWalletAccountId(-1,0),/u32/);
  assert.throws(()=>W.arqWalletAccountId(0,2**32),/u32/);
});

test('outcome and round ids are deterministic, separated by domain and by field boundaries',()=>{
  const a=W.outcomeId('flux','match-7','seat-1');
  assert.equal(a,W.outcomeId('flux','match-7','seat-1'));
  assert.match(a,/^0x[0-9a-f]{64}$/);
  assert.notEqual(a,W.outcomeId('flux','match-7','seat-2'));
  assert.notEqual(W.outcomeId('flux','ab','c'),W.outcomeId('flux','a','bc'));
  assert.notEqual(W.outcomeId('flux','r1'),W.roundId('flux','r1'));
  assert.throws(()=>W.outcomeId('flux'),/non-empty/);
  assert.throws(()=>W.outcomeId('','x'),/non-empty/);
});

test('policies and payouts become the chain call arguments, amounts as decimal Sparks strings',()=>{
  const S=10n**18n;
  const p={maxPayout:5n*S,epochBlocks:100,epochBudget:50n*S,perRecipientPerEpoch:10n*S,ruleVersions:['flux@1'],accrualExpiryBlocks:500,loosenDelayBlocks:600};
  assert.deepEqual(W.toChainPolicy(p),{maxPayout:(5n*S).toString(),epochBlocks:100,epochBudget:(50n*S).toString(),perRecipientPerEpoch:(10n*S).toString(),ruleVersions:['flux@1'],accrualExpiry:500,loosenDelay:600});
  const outcome=W.outcomeId('flux','m1');
  assert.deepEqual(W.payoutArgs({collection:3,item:4,outcome,issuedAt:99,ruleVersion:'flux@1',to:'5Grw',amount:2n*S}),[3,4,outcome,99,'flux@1','5Grw',(2n*S).toString()]);
  assert.throws(()=>W.payoutArgs({collection:3,item:4,outcome:'0x12',issuedAt:99,ruleVersion:'flux@1',to:'x',amount:1n}),/outcomeId/);
  assert.throws(()=>W.payoutArgs({collection:3,item:4,outcome,issuedAt:99,ruleVersion:'flux@1',to:'x',amount:0n}),/nothing/);
  assert.throws(()=>W.toChainPolicy({...p,maxPayout:-1n}),/u128/);
});
