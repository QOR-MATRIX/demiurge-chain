import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

const dir=path.resolve('work/sdk-profile-tests');fs.mkdirSync(dir,{recursive:true});
for(const name of ['amount','profile']){
  const src=fs.readFileSync(`sdk/src/${name}.ts`,'utf8').replace("from './amount'","from './amount.mjs'");
  fs.writeFileSync(path.join(dir,name+'.mjs'),ts.transpileModule(src,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
}
const P=await import(pathToFileURL(path.join(dir,'profile.mjs')).href);
const S=BigInt(10)**BigInt(18);
const template=JSON.parse(fs.readFileSync('sdk/templates/arqade-project.json','utf8'));

const HASH='a'.repeat(64);
const img=n=>({url:`https://example.org/${n}.png`,alt:`Screenshot ${n}`});
const concept={...template,creator:{qorSub:'2f6b1c1e-8a3d-4c1b-9f0e-1a2b3c4d5e6f',displayName:'Ada'}};
const prototype={...concept,stage:'prototype',build:{hash:HASH,entry:'index.html',sizeBytes:1234,controls:['arrows'],permanent:false},contentRating:'everyone',art:{...concept.art,cover:img('cover')}};
const early={...prototype,stage:'early-access',ruleVersions:['my-game@1'],price:{kind:'fixed',sparks:(25n*S).toString()},support:{url:'https://example.org/help'},sandbox:{passedForBuild:HASH},art:{...prototype.art,screenshots:[img(1),img(2),img(3)]}};
const released={...early,stage:'released',build:{...early.build,permanent:true},items:[{id:'blue-hull',name:'Blue hull',description:'A ship skin.',sparks:(2n*S).toString(),editionCap:500}]};

test('the template is a concept that needs only the creator filled in',()=>{
  assert.deepEqual(P.checkReadiness(template,'concept').map(m=>m.missing),["the creator's QOR ID (its account id)"]);
  assert.equal(P.readyStage(template),null);
  assert.equal(P.readyStage(concept),'concept');
});

test('each stage publishes only when it and every earlier stage are met',()=>{
  assert.equal(P.readyStage(prototype),'prototype');
  assert.equal(P.readyStage(early),'early-access');
  assert.equal(P.readyStage(released),'released');
  assert.deepEqual(P.checkReadiness(released,'released'),[]);
  const missing=P.checkReadiness(concept,'early-access');
  assert.ok(missing.some(m=>m.stage==='prototype'&&m.missing==='a playable build'));
  assert.ok(missing.some(m=>m.stage==='early-access'&&/support page/.test(m.missing)));
});

test('sandbox checks must be for this exact build, and a release must be permanent',()=>{
  const rebuilt={...early,build:{...early.build,hash:'b'.repeat(64)}};
  assert.ok(P.checkReadiness(rebuilt,'early-access').some(m=>/this exact build/.test(m.missing)));
  assert.ok(P.checkReadiness({...released,build:{...released.build,permanent:false}},'released').some(m=>m.missing==='a permanent build'));
});

test('a game costs nothing or up to 10,000 CGT, in whole Sparks',()=>{
  assert.equal(P.MAX_GAME_PRICE,10000n*S);
  assert.equal(P.priceProblem({kind:'free'}),null);
  assert.equal(P.priceProblem({kind:'fixed',sparks:(10000n*S).toString()}),null);
  assert.match(P.priceProblem({kind:'fixed',sparks:(10000n*S+1n).toString()}),/above 10,000 CGT/);
  assert.match(P.priceProblem({kind:'fixed',sparks:'0'}),/use free/);
  for(const bad of ['1.5','-1','1e18','',' 1'])assert.match(P.priceProblem({kind:'fixed',sparks:bad}),/whole number/);
  assert.match(P.priceProblem(undefined),/no price/);
});

test('in-game items need unique ids, whole-Spark prices and sane edition caps; none is capped by price',()=>{
  const bad={...released,items:[
    {id:'x',name:'A',description:'a',sparks:'1'},
    {id:'x',name:'B',description:'b',sparks:'0'},
    {id:'Bad Id',name:'',description:'c',sparks:'1',editionCap:0},
  ]};
  const missing=P.checkReadiness(bad,'released').map(m=>m.missing);
  assert.ok(missing.includes('item "x": a unique id'));
  assert.ok(missing.includes('item x: a price in whole Sparks above zero'));
  assert.ok(missing.includes('item "Bad Id": a unique id'));
  assert.ok(missing.includes('item Bad Id: a name and a description'));
  assert.ok(missing.includes('item Bad Id: an edition cap above zero, or none'));
  const dear={...released,items:[{id:'big',name:'Big',description:'d',sparks:(20000n*S).toString()}]};
  assert.deepEqual(P.checkReadiness(dear,'released'),[]);
});
