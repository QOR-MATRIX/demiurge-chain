import {test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

// The real reader, transpiled, against a scripted node. No network.
const dir=path.resolve('work/chain-tests');fs.mkdirSync(dir,{recursive:true});
const out=path.join(dir,'chain.mjs');
fs.writeFileSync(out,ts.transpileModule(fs.readFileSync('lib/chain.ts','utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
const {DEVNET,readChain,blockNumber,httpRpc}=await import(pathToFileURL(out).href);

const FIN='0x'+'ab'.repeat(32);
function node(over={}){
  const answers={chain_getBlockHash:DEVNET.genesis,system_chain:DEVNET.name,chain_getFinalizedHead:FIN,...over};
  return async(method,params)=>{
    if(method==='chain_getHeader')return params.length?(over.finalizedHeader??{number:'0x1cd8'}):(over.bestHeader??{number:'0x1cdc'});
    if(!(method in answers))throw Error('unexpected '+method);
    return answers[method];
  };
}

test('reads the devnet: finalized and best numbers, checked against its genesis',async()=>{
  const s=await readChain(node(),()=>new Date('2026-10-04T00:00:00Z'));
  assert.deepEqual(s,{network:'Demiurge Devnet',genesis:DEVNET.genesis,finalized:0x1cd8,best:0x1cdc,observedAt:'2026-10-04T00:00:00.000Z'});
});

test('refuses a chain whose genesis is not the devnet, before reading anything else',async()=>{
  const calls=[];const rpc=async(m,p)=>{calls.push(m);return node({chain_getBlockHash:'0x'+'00'.repeat(32)})(m,p)};
  await assert.rejects(readChain(rpc),/Wrong network/);
  assert.deepEqual(calls,['chain_getBlockHash']);
});

test('refuses a chain with the right genesis and another name',async()=>{
  await assert.rejects(readChain(node({system_chain:'Development'})),/Wrong network/);
});

test('refuses malformed heads and headers, and a finalized block past the best one',async()=>{
  await assert.rejects(readChain(node({chain_getFinalizedHead:'0x12'})),/finalized head/);
  await assert.rejects(readChain(node({finalizedHeader:{number:7}})),/Invalid block header/);
  await assert.rejects(readChain(node({finalizedHeader:{number:'0x20'},bestHeader:{number:'0x10'}})),/Invalid chain response/);
  assert.throws(()=>blockNumber({number:'0x'+'f'.repeat(14)}),/Invalid block header/);
  assert.throws(()=>blockNumber(null),/Invalid block header/);
  assert.equal(blockNumber({number:'0x0'}),0);
});

test('the HTTP transport posts to the fixed endpoint and refuses errors and non-OK answers',async()=>{
  const seen=[];
  const ok=async(url,init)=>{seen.push([url,JSON.parse(init.body).method]);return new Response(JSON.stringify({jsonrpc:'2.0',id:1,result:'x'}))};
  assert.equal(await httpRpc(ok)('system_chain',[]),'x');
  assert.deepEqual(seen,[[DEVNET.rpc,'system_chain']]);
  await assert.rejects(httpRpc(async()=>new Response('{}',{status:502}))('system_chain',[]),/RPC unavailable/);
  await assert.rejects(httpRpc(async()=>new Response(JSON.stringify({error:{code:-32601}})))('x',[]),/RPC error/);
  await assert.rejects(httpRpc(async()=>new Response(JSON.stringify({id:1})))('x',[]),/RPC error/);
});
