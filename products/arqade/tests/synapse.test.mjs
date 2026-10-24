import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as synapse from '../games/synapse/logic/rules.mjs';

// Synapse's rules (P7.20, ADR-102): the same module the QQ game runs, here as ARQADE's service will run it to replay a
// run. A run QQ itself recorded is replayed below to the outcome QQ reached.
const until=(g,cond,limit=10000)=>{for(let i=0;i<limit&&!cond(g)&&g.phase!=='over';i++)synapse.step(g);return g};
const repeat=(g,upTo=g.signal.length)=>{for(let i=0;i<upTo;i++)synapse.press(g,'node'+(g.signal[g.cursor]+1))};
const toRepeat=g=>until(g,x=>x.stage==='repeat');

test('a seed gives the same signal every time, and another seed another',()=>{
 const play=seed=>{const g=synapse.create(seed);synapse.click(g,'Play');for(let r=0;r<7;r++){toRepeat(g);repeat(g)}return g.signal.join('')};
 assert.equal(play(7),play(7));assert.notEqual(play(7),play(8));
});
test('the signal shows each node for 28 steps, 42 apart, after a lead of 30, and then is the player\'s',()=>{
 const g=synapse.create(3);synapse.click(g,'Play');
 const seen=[];for(let i=0;i<synapse.showing(1);i++){seen.push(synapse.lit(g));synapse.step(g)}
 assert.equal(seen.slice(0,30).every(n=>n===-1),true);assert.equal(seen.slice(30,58).every(n=>n===g.signal[0]),true);
 assert.equal(seen.slice(58).every(n=>n===-1),true);assert.equal(g.stage,'repeat');
 repeat(g);assert.equal(g.signal.length,2);assert.equal(g.stage,'watch');
 toRepeat(g);assert.equal(g.shown,synapse.showing(2));
});
test('a press while the signal shows is no answer',()=>{
 const g=synapse.create(4);synapse.click(g,'Play');synapse.step(g);synapse.press(g,'node1');synapse.press(g,'node2');
 assert.equal(g.phase,'playing');assert.equal(g.cursor,0);assert.equal(g.stage,'watch');
});
test('eight signals repeated master the game for 1500',()=>{
 const g=synapse.create(42);synapse.click(g,'Play');
 for(let r=1;r<=8;r++){toRepeat(g);assert.equal(g.signal.length,r);repeat(g)}
 assert.deepEqual(synapse.outcome(g),{score:1500,length:8,mastered:true,over:true});
});
test('one node out of turn loses the signal, scored for each whole signal and each node of the last',()=>{
 const g=synapse.create(9);synapse.click(g,'Play');
 for(let r=1;r<=3;r++){toRepeat(g);repeat(g)}
 toRepeat(g);repeat(g,2);
 const wrong=(g.signal[g.cursor]+1)%4;synapse.press(g,'node'+(wrong+1));
 assert.deepEqual(synapse.outcome(g),{score:3*150+2*20,length:4,mastered:false,over:true});
 assert.deepEqual(g.said.at(-1),{what:'wrong',node:wrong});
 synapse.click(g,'Play again');assert.equal(g.phase,'playing');assert.equal(g.signal.length,1);assert.equal(g.score,0);
});
test('nothing moves on the title or while paused',()=>{
 const g=synapse.create(5);synapse.step(g);assert.equal(g.shown,0);assert.equal(g.steps,0);
 synapse.click(g,'Play');synapse.step(g);synapse.press(g,'pause');synapse.step(g);assert.equal(g.shown,1);assert.equal(g.phase,'paused');
 synapse.click(g,'Resume');synapse.step(g);assert.equal(g.shown,2);
 toRepeat(g);synapse.press(g,'pause');synapse.press(g,'node'+(g.signal[0]+1));assert.equal(g.cursor,0);
 synapse.press(g,'pause');synapse.press(g,'node'+(g.signal[0]+1));assert.equal(g.signal.length,2);
});
test('a run replays to the outcome of the same events played live',()=>{
 const live=synapse.create(99),edges=[],clicks=[[3,['Play']]];let tick=0;
 for(;tick<20000&&live.phase!=='over';tick++){
  if(tick===3)synapse.click(live,'Play');
  // A player who repeats each signal a few ticks after it is theirs, and then, on the sixth, presses wrongly.
  if(live.phase==='playing'&&live.stage==='repeat'&&tick%5===0){
   const right=live.signal[live.cursor],node=live.signal.length===6&&live.cursor===3?(right+1)%4:right;
   edges.push([tick,[['pressed','node'+(node+1)],['released','node'+(node+1)]]]);synapse.press(live,'node'+(node+1));
  }
  synapse.step(live);
 }
 const run={qqRun:1,seed:99,ticks:tick,edges,clicks};
 assert.deepEqual(synapse.replay(run),synapse.outcome(live));assert.deepEqual(synapse.outcome(live),{score:810,length:6,mastered:false,over:true});
 assert.throws(()=>synapse.replay({qqRun:2,seed:1,ticks:1}));assert.throws(()=>synapse.replay(null));
});
const fixture=new URL('./fixtures/synapse-run.json',import.meta.url);
test('the run QQ kept as ARQADE played it replays here to the outcome QQ reached, and an altered one does not',()=>{
 // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which plays Synapse in ARQADE's window and replays this
 // same file with QQ's own replay.
 const run=JSON.parse(readFileSync(fixture,'utf8'));
 assert.equal(run.qqRun,1);assert.equal(run.scene,'synapse');
 assert.deepEqual(run.outcome.Synapse,{score:450,length:4,mastered:false,over:true});
 assert.deepEqual(synapse.replay(run),run.outcome.Synapse);
 // Another seed, or the same run without its last press (the node out of turn), is another run with another outcome.
 assert.notDeepEqual(synapse.replay({...run,seed:run.seed+1}),run.outcome.Synapse);
 const last=run.edges.findLastIndex(([,list])=>list.some(([what,action])=>what==='pressed'&&action.startsWith('node')));
 assert.notDeepEqual(synapse.replay({...run,edges:run.edges.filter((e,i)=>i!==last)}),run.outcome.Synapse);
});
test('the rules\' whole state at the end of the kept run is pinned, so no change to the rules passes unseen',()=>{
 const run=JSON.parse(readFileSync(fixture,'utf8'));
 const edges=new Map(run.edges.map(([t,l])=>[t,l])),clicks=new Map(run.clicks.map(([t,l])=>[t,l])),g=synapse.create(run.seed);
 for(let t=0;t<run.ticks;t++){for(const b of clicks.get(t)||[])synapse.click(g,b);for(const [w,a] of edges.get(t)||[])if(w==='pressed')synapse.press(g,a);synapse.step(g)}
 assert.equal(createHash('sha256').update(JSON.stringify(g)).digest('hex'),'182b57d26b2a5e204e0d7421134889bc188df817055576bcad84688498680256');
});
test('the rules keep to whole numbers every machine computes alike: no sine, cosine, logarithm or power decides anything',()=>{
 const source=readFileSync(new URL('../games/synapse/logic/rules.mjs',import.meta.url),'utf8').replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/.*$/gm,'');
 assert.doesNotMatch(source,/Math\.(sin|cos|tan|asin|acos|atan2?|sinh|cosh|tanh|exp|expm1|log\w*|pow|sqrt|cbrt|hypot|random)\b/);
 assert.doesNotMatch(source,/\*\*/);
});
