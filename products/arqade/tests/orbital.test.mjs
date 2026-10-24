import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import * as orbital from '../games/orbital/logic/rules.mjs';

// Orbital's rules (P7.20, ADR-102): the same module the QQ game runs, here as ARQADE's service will run it to replay a
// run. A run QQ itself recorded is replayed below to the outcome QQ reached.
const play=(game,until,act)=>{for(let i=0;i<until&&game.phase!=='over';i++){act(game);orbital.step(game)}return game};
const perfect=g=>{if(orbital.within(g.angle,g.target))orbital.press(g,'lock')};
const clumsy=g=>{if(!orbital.within(g.angle,g.target)&&g.steps%90===0)orbital.press(g,'lock')};

test('a seed gives the same orbit every time, and another seed another',()=>{
 const a=orbital.create(7),b=orbital.create(7),c=orbital.create(8);
 assert.equal(a.target,b.target);assert.notEqual(a.target,c.target);
 orbital.click(a,'Play');orbital.click(b,'Play');play(a,5000,perfect);play(b,5000,perfect);
 assert.deepEqual(orbital.outcome(a),orbital.outcome(b));assert.equal(a.target,b.target);
});
test('a lock holds within the arc, across the turn too, and not beyond it',()=>{
 assert.ok(orbital.within(1,1));assert.ok(orbital.within(1.28,1));assert.ok(!orbital.within(1.3,1));
 assert.ok(orbital.within(0.1,6.2));assert.ok(orbital.within(6.2,0.1));assert.ok(!orbital.within(3,0));
});
test('a perfect player completes the orbit in ten locks, each faster',()=>{
 const g=orbital.create(42);orbital.click(g,'Play');let last=orbital.speed(0),turns=0;
 play(g,20000,x=>{if(x.locks>turns){turns=x.locks;assert.ok(orbital.speed(turns)>last);last=orbital.speed(turns)}perfect(x)});
 assert.deepEqual(orbital.outcome(g),{locks:10,misses:0,score:2000,over:true});
});
test('three misses lose the orbit, and play again starts afresh on the same seed\'s next numbers',()=>{
 const g=orbital.create(5);orbital.click(g,'Play');play(g,20000,clumsy);
 assert.deepEqual(orbital.outcome(g),{locks:0,misses:3,score:0,over:true});
 const target=g.target;orbital.click(g,'Play again');assert.equal(g.phase,'playing');assert.equal(g.misses,0);assert.notEqual(g.target,target);
});
test('nothing moves on the title or while paused, and a lock is judged only in play',()=>{
 const g=orbital.create(3);orbital.press(g,'lock');orbital.step(g);assert.equal(g.steps,0);assert.equal(g.angle,0);
 orbital.click(g,'Play');orbital.step(g);const at=g.angle;assert.ok(at>0);
 orbital.press(g,'pause');orbital.press(g,'lock');orbital.step(g);assert.equal(g.angle,at);assert.equal(g.phase,'paused');
 orbital.click(g,'Resume');orbital.step(g);assert.ok(g.angle>at);assert.equal(g.misses+g.locks,0);
});
test('a run replays to the outcome of the same events played live',()=>{
 const live=orbital.create(99),edges=[],clicks=[[3,['Play']]];let tick=0;
 for(;tick<20000&&live.phase!=='over';tick++){
  if(tick===3)orbital.click(live,'Play');
  if(live.phase==='playing'&&orbital.within(live.angle,live.target)&&tick%3===0){edges.push([tick,[['pressed','lock']]]);orbital.press(live,'lock')}
  orbital.step(live);
 }
 const run={qqRun:1,seed:99,ticks:tick,edges,clicks};
 assert.deepEqual(orbital.replay(run),orbital.outcome(live));assert.equal(orbital.outcome(live).locks,10);
 assert.throws(()=>orbital.replay({qqRun:2,seed:1,ticks:1}));assert.throws(()=>orbital.replay(null));
});
test('the run QQ kept as ARQADE played it replays here to the outcome QQ reached, and an altered one does not',()=>{
 // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which plays Orbital in ARQADE's window and replays this
 // same file with QQ's own replay.
 const run=JSON.parse(readFileSync(new URL('./fixtures/orbital-run.json',import.meta.url),'utf8'));
 assert.equal(run.qqRun,1);assert.equal(run.scene,'orbital');
 assert.deepEqual(run.outcome.Orbital,{locks:10,misses:1,score:2000,over:true});
 assert.deepEqual(orbital.replay(run),run.outcome.Orbital);
 // Another seed, or the first lock pressed 40 ticks later, is another run with another outcome.
 assert.notDeepEqual(orbital.replay({...run,seed:run.seed+1}),run.outcome.Orbital);
 const first=run.edges.findIndex(([,list])=>list.some(([what,action])=>what==='pressed'&&action==='lock'));
 const later=run.edges.map((e,i)=>i===first?[e[0]+40,e[1]]:e);
 assert.notDeepEqual(orbital.replay({...run,edges:later}),run.outcome.Orbital);
});
test('the rules keep to arithmetic every machine computes alike: no sine, cosine, logarithm or power decides anything',()=>{
 const source=readFileSync(new URL('../games/orbital/logic/rules.mjs',import.meta.url),'utf8').replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/.*$/gm,'');
 assert.doesNotMatch(source,/Math\.(sin|cos|tan|asin|acos|atan2?|sinh|cosh|tanh|exp|expm1|log\w*|pow|sqrt|cbrt|hypot|random)\b/);
 assert.doesNotMatch(source,/\*\*/);
});
