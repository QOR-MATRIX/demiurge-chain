import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as runner from '../games/void-runner/logic/rules.mjs';

// Void Runner's rules (P7.20, ADR-102): the same module the QQ game runs, here as ARQADE's service will run it to replay
// a run. A run QQ itself recorded is replayed below to the outcome QQ reached.
const play=(game,until,act)=>{for(let i=0;i<until&&game.phase!=='over';i++){act(game);runner.step(game)}return game};
// A pilot: into the lane where the next thing to arrive is a gem, or else away from a hazard about to arrive.
const pilot=g=>{
 const soon=lane=>g.coming.filter(t=>t.lane===lane&&t.z>0.75).sort((a,b)=>b.z-a.z)[0];
 const worth=lane=>{const t=soon(lane);return !t?0:t.gem?1:-10};
 let best=g.lane;for(let lane=0;lane<runner.LANES;lane++)if(worth(lane)>worth(best)||(worth(lane)===worth(best)&&Math.abs(lane-g.lane)<Math.abs(best-g.lane)))best=lane;
 while(g.lane<best)runner.press(g,'right');while(g.lane>best)runner.press(g,'left');
};
const sitter=()=>{};

test('a seed gives the same run every time, and another seed another',()=>{
 const a=runner.create(7),b=runner.create(7),c=runner.create(8);
 for(const g of [a,b,c])runner.click(g,'Play');
 play(a,600,pilot);play(b,600,pilot);play(c,600,pilot);
 assert.deepEqual(a.coming,b.coming);assert.deepEqual(runner.outcome(a),runner.outcome(b));
 assert.notDeepEqual(a.coming.map(t=>[t.lane,t.gem]),c.coming.map(t=>[t.lane,t.gem]));
});
test('something is thrown every 38 steps, and the rush quickens',()=>{
 const g=runner.create(3);runner.click(g,'Play');
 play(g,37,sitter);assert.equal(g.thrown,0);play(g,1,sitter);assert.equal(g.thrown,1);play(g,38,sitter);assert.equal(g.thrown,2);
 assert.ok(runner.speed(runner.RUN_STEPS)>runner.speed(0));
});
test('a hazard in the lane breaches the hull, which then holds a moment, and three breaches end the run',()=>{
 const g=runner.create(11);runner.click(g,'Play');let hits=0,held=0;
 play(g,runner.RUN_STEPS,x=>{hits+=x.said.filter(s=>s==='hit').length;held+=x.said.filter(s=>s==='held').length});
 hits+=g.said.filter(s=>s==='hit').length;
 const end=runner.outcome(g);
 assert.equal(end.hull,0);assert.equal(end.over,true);assert.equal(end.survived,false);assert.equal(hits,runner.HULL);
 assert.ok(end.seconds<45);assert.equal(end.score,end.gems*runner.GEM_POINTS+Math.floor(g.steps/6));
});
test('a pilot who dodges every hazard survives forty-five seconds with the hull whole',()=>{
 const g=runner.create(42);runner.click(g,'Play');play(g,runner.RUN_STEPS+10,pilot);
 const end=runner.outcome(g);
 assert.deepEqual({hull:end.hull,seconds:end.seconds,survived:end.survived,over:end.over},{hull:3,seconds:45,survived:true,over:true});
 assert.ok(end.gems>5);assert.equal(end.score,end.gems*100+450);
});
test('the lanes stop at the edges, and nothing moves on the title or while paused',()=>{
 const g=runner.create(5);runner.press(g,'left');runner.step(g);assert.equal(g.lane,1);assert.equal(g.steps,0);
 runner.click(g,'Play');runner.press(g,'left');runner.press(g,'left');assert.equal(g.lane,0);
 runner.press(g,'right');runner.press(g,'right');runner.press(g,'right');assert.equal(g.lane,2);
 runner.step(g);runner.press(g,'pause');assert.equal(g.phase,'paused');runner.press(g,'left');runner.step(g);
 assert.equal(g.lane,2);assert.equal(g.steps,1);
 runner.click(g,'Resume');runner.step(g);assert.equal(g.steps,2);
 play(g,runner.RUN_STEPS,sitter);assert.equal(g.phase,'over');runner.click(g,'Play again');assert.equal(g.phase,'playing');assert.equal(g.hull,3);assert.equal(g.lane,1);
});
test('a run replays to the outcome of the same events played live',()=>{
 const live=runner.create(99),edges=[],clicks=[[3,['Play']]];let tick=0;
 for(;tick<4000&&live.phase!=='over';tick++){
  if(tick===3)runner.click(live,'Play');
  if(live.phase==='playing'){const before=live.lane,list=[];pilot(live);
   for(let d=live.lane-before;d!==0;d-=Math.sign(d))list.push(['pressed',d>0?'right':'left'],['released',d>0?'right':'left']);
   if(list.length)edges.push([tick,list])}
  runner.step(live);
 }
 const run={qqRun:1,seed:99,ticks:tick,edges,clicks};
 assert.deepEqual(runner.replay(run),runner.outcome(live));assert.equal(runner.outcome(live).survived,true);
 assert.throws(()=>runner.replay({qqRun:2,seed:1,ticks:1}));assert.throws(()=>runner.replay(null));
});
test('the run QQ kept as ARQADE played it replays here to the outcome QQ reached, and an altered one does not',()=>{
 // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which steers Void Runner in ARQADE's window, then steers
 // into three hazards, and replays this same file with QQ's own replay.
 const run=JSON.parse(readFileSync(new URL('./fixtures/void-runner-run.json',import.meta.url),'utf8'));
 assert.equal(run.qqRun,1);assert.equal(run.scene,'void-runner');
 assert.deepEqual(run.outcome['Void Runner'],{score:637,gems:5,hull:0,seconds:13,survived:false,over:true});
 assert.deepEqual(runner.replay(run),run.outcome['Void Runner']);
 // Another seed, or the same run with no change of lane, is another run with another outcome.
 assert.notDeepEqual(runner.replay({...run,seed:run.seed+1}),run.outcome['Void Runner']);
 const still=run.edges.map(([tick,list])=>[tick,list.filter(([,action])=>action!=='left'&&action!=='right')]);
 assert.notDeepEqual(runner.replay({...run,edges:still}),run.outcome['Void Runner']);
});
test('the rules\' whole state at the end of the kept run is pinned, so no change to the rules passes unseen',()=>{
 // A small change can leave one run's outcome as it was; its state at the end still moves. A change here is a new
 // version of the rules: once ARQADE's service confirms runs (P7.22), runs kept under the old ones replay under those.
 const run=JSON.parse(readFileSync(new URL('./fixtures/void-runner-run.json',import.meta.url),'utf8'));
 const edges=new Map(run.edges.map(([t,l])=>[t,l])),clicks=new Map(run.clicks.map(([t,l])=>[t,l])),g=runner.create(run.seed);
 for(let t=0;t<run.ticks;t++){for(const b of clicks.get(t)||[])runner.click(g,b);for(const [w,a] of edges.get(t)||[])if(w==='pressed')runner.press(g,a);runner.step(g)}
 assert.equal(createHash('sha256').update(JSON.stringify(g)).digest('hex'),'dc0bdb595dc60faa3d92049f521824ad4823d0cf891d6293b1fb0eb455016523');
});
test('the rules keep to arithmetic every machine computes alike: no sine, cosine, logarithm or power decides anything',()=>{
 const source=readFileSync(new URL('../games/void-runner/logic/rules.mjs',import.meta.url),'utf8').replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/.*$/gm,'');
 assert.doesNotMatch(source,/Math\.(sin|cos|tan|asin|acos|atan2?|sinh|cosh|tanh|exp|expm1|log\w*|pow|sqrt|cbrt|hypot|random)\b/);
 assert.doesNotMatch(source,/\*\*/);
});
