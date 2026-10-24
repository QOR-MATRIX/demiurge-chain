import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as rift from '../games/rift-survivor/logic/rules.mjs';

// Rift Survivor's rules (P7.20, ADR-102): the same module the QQ game runs, here as ARQADE's service will run it to
// replay a run. A run QQ itself recorded is replayed below to the outcome QQ reached.
const DIRS=[[0,0],[1,0],[-1,0],[0,1],[0,-1],[1,1],[1,-1],[-1,1],[-1,-1]];
// Which of the eight ways (or none) keeps the ship furthest from what threatens it (enemies, rift pulses about to arm,
// the arena's edges) and nearest the closest fragment to collect.
const pilotWay=g=>{
 let best=0,score=-Infinity;
 for(let i=0;i<DIRS.length;i++){
  const [dx,dy]=DIRS[i],k=(dx&&dy)?0.7071067811865476:1,x=g.x+dx*k*30,y=g.y+dy*k*30;
  let s=0;for(const e of g.enemies){const d=(e.x-x)**2+(e.y-y)**2;s-=40000/(d+1)}
  for(const p of g.pulses)if(p.age<rift.PULSE_ENDS&&(p.x-x)**2+(p.y-y)**2<120**2)s-=50;
  const f=g.fragments.map(f=>(f.x-x)**2+(f.y-y)**2).sort((a,b)=>a-b)[0];if(f!==undefined)s+=8000/(f+400);
  s-=((x-500)/500)**4+((y-310)/310)**4;
  if(s>score){score=s;best=i}
 }
 return DIRS[best];
};
const hold=(g,[dx,dy],into)=>{const want={right:dx>0,left:dx<0,down:dy>0,up:dy<0};
 for(const a of ['up','down','left','right'])if(want[a]!==g.held[a]){(want[a]?rift.press:rift.release)(g,a);into?.push([want[a]?'pressed':'released',a])}};
const fly=(g,steps,way=pilotWay,perk=0,each=()=>{})=>{for(let i=0;i<steps&&g.phase!=='over';i++){if(g.phase==='choosing')rift.press(g,'perk'+(perk++%3+1));hold(g,way(g));rift.step(g);each(g)}return g};

test('a seed gives the same run every time, and another seed another',()=>{
 const run=seed=>{const g=rift.create(seed);rift.click(g,'Play');fly(g,1800);return JSON.stringify(rift.outcome(g))+g.x+','+g.y};
 assert.equal(run(7),run(7));assert.notEqual(run(7),run(8));
});
test('enemies come every 48 steps at first and every 11 at the most, and the armoured ones only as the run goes on',()=>{
 assert.equal(rift.spawnEvery(0),48);assert.equal(rift.spawnEvery(5400),16);assert.equal(rift.spawnEvery(9000),11);
 const g=rift.create(3);rift.click(g,'Play');fly(g,47,()=>[0,0]);assert.equal(g.enemies.length,0);fly(g,1,()=>[0,0]);assert.equal(g.enemies.length,1);
 assert.equal(g.enemies[0].kind,0);
 const late=rift.create(4);rift.click(late,'Play');fly(late,3000);assert.ok(late.enemies.some(e=>e.kind===1)||late.kills>0);
});
test('the ship moves four a step, a diagonal by the half root, stops at the edges, and holds still on the title and paused',()=>{
 const g=rift.create(5);rift.press(g,'right');rift.step(g);assert.equal(g.x,500);assert.equal(g.steps,0);
 rift.click(g,'Play');rift.step(g);assert.equal(g.x,504);
 rift.press(g,'down');rift.step(g);assert.equal(g.x,504+4*0.7071067811865476);assert.equal(g.y,310+4*0.7071067811865476);
 rift.press(g,'pause');rift.step(g);assert.equal(g.steps,2);rift.press(g,'pause');rift.step(g);assert.equal(g.steps,3);
 rift.release(g,'down');for(let i=0;i<200;i++)rift.step(g);assert.equal(g.x,rift.WIDTH-rift.MARGIN);
});
test('the ship fires on the nearest enemy, kills score more in a chain, and fragments are drawn in and collected',()=>{
 const g=rift.create(11);rift.click(g,'Play');let shots=0,fragments=0,chain=0;
 fly(g,1200,pilotWay,0,x=>{shots+=x.said.filter(s=>s.what==='shot').length;fragments+=x.said.filter(s=>s.what==='fragment').length;chain=Math.max(chain,x.chain)});
 assert.ok(shots>10);assert.ok(g.kills>5);assert.ok(fragments>0);assert.ok(chain>1);
 assert.ok(g.score>=g.kills*20+fragments*15);
});
test('a ship that stands still is breached by enemies and pulses, the hull holding a moment after each, and four end the run',()=>{
 const g=rift.create(9);rift.click(g,'Play');const at=[];
 fly(g,rift.RUN_STEPS*2,()=>[0,0],0,x=>{if(x.said.some(s=>s.what==='hit'))at.push(x.steps)});
 assert.equal(g.hull,0);assert.equal(at.length,4);assert.equal(g.phase,'over');
 for(let i=1;i<at.length;i++)assert.ok(at[i]-at[i-1]>=rift.HOLD_STEPS);
 assert.equal(rift.outcome(g).survived,false);
});
test('ten kills evolve the signal: the world holds while a perk is chosen, and each perk does what it says',()=>{
 const g=rift.create(21);rift.click(g,'Play');
 for(let i=0;i<5400&&g.phase==='playing';i++){hold(g,pilotWay(g));rift.step(g)}
 assert.equal(g.phase,'choosing');assert.equal(g.level,2);assert.ok(g.kills>=10);
 const steps=g.steps;rift.step(g);assert.equal(g.steps,steps);
 const a=structuredClone(g),b=structuredClone(g),c=structuredClone(g);
 rift.click(a,'Overclock');assert.equal(a.fireEvery,17);assert.equal(a.phase,'playing');
 rift.press(b,'perk2');assert.equal(b.spread,1);
 c.hull=1;rift.click(c,'Second wind');assert.equal(c.hull,3);assert.equal(c.magnet,130);
 // A volley with a spread of one is three shots, fanned 0.14 radians apart.
 for(let i=0;i<60&&!b.said.some(x=>x.what==='shot');i++)rift.step(b);
 const volley=b.shots.filter(x=>x.life===rift.SHOT_STEPS-1);assert.equal(volley.length,3);
 const [l,m,r]=volley,dot=(p,q)=>(p.vx*q.vx+p.vy*q.vy)/(Math.sqrt(p.vx**2+p.vy**2)*Math.sqrt(q.vx**2+q.vy**2));
 assert.ok(Math.abs(dot(l,m)-0.9902159962126371)<1e-12);assert.ok(Math.abs(dot(m,r)-0.9902159962126371)<1e-12);
});
test('a pilot who keeps away lasts far longer than one who stands still',()=>{
 const still=rift.create(33),moving=rift.create(33);rift.click(still,'Play');rift.click(moving,'Play');
 fly(still,rift.RUN_STEPS,()=>[0,0]);fly(moving,rift.RUN_STEPS);
 assert.ok(moving.steps>still.steps*1.5,`${moving.steps} against ${still.steps}`);
});
test('a run replays to the outcome of the same events played live',()=>{
 const live=rift.create(99),edges=[],clicks=[[3,['Play']]];let tick=0,perk=0;
 for(;tick<6000&&live.phase!=='over';tick++){
  if(tick===3)rift.click(live,'Play');
  if(live.phase==='choosing'){const name=rift.PERKS[perk++%3];clicks.push([tick,[name]]);rift.click(live,name)}
  if(live.phase==='playing'){const list=[];hold(live,pilotWay(live),list);if(list.length)edges.push([tick,list])}
  rift.step(live);
 }
 const run={qqRun:1,seed:99,ticks:tick,edges,clicks};
 assert.deepEqual(rift.replay(run),rift.outcome(live));assert.ok(rift.outcome(live).level>1);
 assert.throws(()=>rift.replay({qqRun:2,seed:1,ticks:1}));assert.throws(()=>rift.replay(null));
});
const fixture=new URL('./fixtures/rift-survivor-run.json',import.meta.url);
test('the run QQ kept as ARQADE played it replays here to the outcome QQ reached, and an altered one does not',()=>{
 // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which steers Rift Survivor in ARQADE's window until the
 // signal evolves, chooses perks, then leaves the ship standing, and replays this same file with QQ's own replay.
 const run=JSON.parse(readFileSync(fixture,'utf8'));
 assert.equal(run.qqRun,1);assert.equal(run.scene,'rift-survivor');
 assert.deepEqual(run.outcome['Rift Survivor'],{score:1452,kills:35,level:3,hull:0,seconds:28,survived:false,over:true});
 assert.deepEqual(rift.replay(run),run.outcome['Rift Survivor']);
 // Another seed, or the same run with the ship never moved, is another run with another outcome.
 assert.notDeepEqual(rift.replay({...run,seed:run.seed+1}),run.outcome['Rift Survivor']);
 const moves=['up','down','left','right'],still=run.edges.map(([tick,list])=>[tick,list.filter(([,action])=>!moves.includes(action))]);
 assert.notDeepEqual(rift.replay({...run,edges:still}),run.outcome['Rift Survivor']);
});
test('the rules\' whole state at the end of the kept run is pinned, so no change to the rules passes unseen',()=>{
 const run=JSON.parse(readFileSync(fixture,'utf8'));
 const edges=new Map(run.edges.map(([t,l])=>[t,l])),clicks=new Map(run.clicks.map(([t,l])=>[t,l])),g=rift.create(run.seed);
 for(let t=0;t<run.ticks;t++){for(const b of clicks.get(t)||[])rift.click(g,b);for(const [w,a] of edges.get(t)||[])(w==='pressed'?rift.press:rift.release)(g,a);rift.step(g)}
 assert.equal(createHash('sha256').update(JSON.stringify(g)).digest('hex'),'f4bda32ada999917ead8e9f0034a95b24245e5b58056297fb58b57fadb141afb');
});
test('the rules keep to arithmetic every machine computes alike: a square root, but no sine, cosine, power or logarithm',()=>{
 const source=readFileSync(new URL('../games/rift-survivor/logic/rules.mjs',import.meta.url),'utf8').replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/.*$/gm,'');
 assert.doesNotMatch(source,/Math\.(sin|cos|tan|asin|acos|atan2?|sinh|cosh|tanh|exp|expm1|log\w*|pow|cbrt|hypot|random)\b/);
 assert.doesNotMatch(source,/\*\*/);
});
