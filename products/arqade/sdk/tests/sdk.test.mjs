import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

// The real modules, transpiled; imports between them rewritten to the emitted files.
const dir=path.resolve('work/sdk-tests');fs.mkdirSync(dir,{recursive:true});
for(const name of ['amount','vault-policy']){
  const src=fs.readFileSync(`sdk/src/${name}.ts`,'utf8').replace("from './amount'","from './amount.mjs'");
  fs.writeFileSync(path.join(dir,name+'.mjs'),ts.transpileModule(src,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
}
const A=await import(pathToFileURL(path.join(dir,'amount.mjs')).href);
const P=await import(pathToFileURL(path.join(dir,'vault-policy.mjs')).href);
const S=BigInt(10)**BigInt(18);

test('CGT parses to exact Sparks and formats back',()=>{
  assert.equal(A.parseCgt('0'),0n);
  assert.equal(A.parseCgt('12.5'),12n*S+S/2n);
  assert.equal(A.parseCgt('0.000000000000000001'),1n);
  assert.equal(A.formatCgt(12n*S+S/2n),'12.5');
  assert.equal(A.formatCgt(1n),'0.000000000000000001');
  assert.equal(A.formatCgt(A.EXISTENTIAL_DEPOSIT),'100');
  for(const s of ['1','100','0.1','340282366920938463463.374607431768211455'])assert.equal(A.formatCgt(A.parseCgt(s)),s);
});

test('excess precision, malformed text and amounts past u128 are refused, never rounded',()=>{
  assert.throws(()=>A.parseCgt('0.0000000000000000001'),/decimal places/);
  for(const bad of ['','-1','1e3','01','1.','.5','1,000',' 1','0x10','NaN'])assert.throws(()=>A.parseCgt(bad),RangeError,bad);
  assert.equal(A.parseCgt('340282366920938463463.374607431768211455'),A.U128_MAX);
  assert.throws(()=>A.parseCgt('340282366920938463463.374607431768211456'),/u128/);
  assert.throws(()=>A.formatCgt(-1n),/u128/);
  assert.equal(A.sparksFromJson('42'),42n);
  for(const bad of [42,'4.2','-1','',null])assert.throws(()=>A.sparksFromJson(bad),RangeError);
});

// A developer's example policy and made-up bounds: neither is a platform value (U-16).
const bounds={minLoosenDelayBlocks:10,maxAccrualExpiryBlocks:1000,maxRuleVersions:4};
const ok={maxPayout:5n*S,epochBlocks:100,epochBudget:50n*S,perRecipientPerEpoch:10n*S,ruleVersions:['flux-four@3'],accrualExpiryBlocks:500,loosenDelayBlocks:20};

test('a sound policy has no problems; each unsound one is named',()=>{
  assert.deepEqual(P.validatePolicy(ok,bounds),[]);
  const cases=[
    [{maxPayout:0n},/maxPayout is zero/],
    [{epochBudget:A.U128_MAX+1n},/epochBudget is outside/],
    [{epochBlocks:0},/epochBlocks must be/],
    [{loosenDelayBlocks:1.5},/loosenDelayBlocks must be/],
    [{maxPayout:11n*S},/could never be paid/],
    [{perRecipientPerEpoch:60n*S,maxPayout:5n*S},/larger than epochBudget/],
    [{loosenDelayBlocks:9},/below the protocol's minimum/],
    [{accrualExpiryBlocks:1001},/above the protocol's maximum/],
    [{ruleVersions:[]},/ruleVersions is empty/],
    [{ruleVersions:['a@1','b@1','c@1','d@1','e@1']},/more than 4/],
    [{ruleVersions:['a@1','a@1']},/repeats/],
    [{ruleVersions:['Flux Four']},/not name@number/],
  ];
  for(const [change,re] of cases){const problems=P.validatePolicy({...ok,...change},bounds);assert.ok(problems.some(p=>re.test(p)),`${JSON.stringify(change,(k,v)=>typeof v==='bigint'?v.toString():v)} -> ${problems}`)}
});

test('loosening is told apart from tightening, field by field',()=>{
  assert.deepEqual(P.loosens(ok,ok),[]);
  assert.deepEqual(P.loosens(ok,{...ok,maxPayout:1n*S,epochBudget:20n*S,ruleVersions:[],epochBlocks:200}),[]);
  assert.deepEqual(P.loosens(ok,{...ok,maxPayout:6n*S}),['maxPayout']);
  assert.deepEqual(P.loosens(ok,{...ok,epochBlocks:50}),['epochBlocks']);
  assert.deepEqual(P.loosens(ok,{...ok,loosenDelayBlocks:15}),['loosenDelayBlocks']);
  assert.deepEqual(P.loosens(ok,{...ok,ruleVersions:['flux-four@3','flux-four@4']}),['ruleVersions']);
  assert.deepEqual(P.loosens(ok,{...ok,accrualExpiryBlocks:600,perRecipientPerEpoch:20n*S}),['perRecipientPerEpoch','accrualExpiryBlocks']);
});
