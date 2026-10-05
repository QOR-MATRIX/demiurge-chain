import {test} from 'node:test';
import assert from 'node:assert/strict';
import {compile,postgres} from './postgres.mjs';

// The real rules and SQL against Postgres (PGlite) with the site's migrations, injecting only who is
// signed in: the QOR ID profile the session would carry (ADR-074).
const engine=await import(compile('lib/arena-engine.ts','engine'));
compile('lib/origin.ts','origin');
const {db,one}=await postgres();
globalThis.__arenaTest={db,user:null};
const storeWith=(name,connect)=>compile('lib/arena-store.ts',name,[
 ["import {database as connect,failure,Unavailable} from './db';",`import {failure,Unavailable} from './db.mjs';const connect=${connect};`],
 ["import {sessionProfile} from './qor-deps';","const sessionProfile=async()=>globalThis.__arenaTest.user;"],
 ["from './arena-engine'","from './engine.mjs'"],
 ["from './origin'","from './origin.mjs'"]]);
const store=await import(storeWith('store','()=>globalThis.__arenaTest.db'));
const chat=await import(compile('app/api/chat/route.ts','chat',[["from '@/lib/arena-store'","from './store.mjs'"]]));
const live=await import(compile('app/api/live/route.ts','live',[["from '@/lib/arena-store'","from './store.mjs'"]]));
async function user(name){globalThis.__arenaTest.user={sub:'sub-'+name,qorId:name,username:name,chainAccount:null};return store.identity()}
const A=await user('player-a'),B=await user('player-b'),C=await user('spectator');
const req=(data,origin='https://arcade.test')=>new Request('https://arcade.test/api/chat',{method:'POST',headers:{Origin:origin,'Content-Type':'application/json'},body:JSON.stringify(data)});

test('Flux rejects illegal moves and resolves vertical and diagonal wins',()=>{
 let b=engine.initialBoard('flux');for(const c of[0,1,0,1,0,1,0])b=engine.moveBoard('flux',b,c);assert.equal(b.winner,1);assert.throws(()=>engine.moveBoard('flux',b,2));
 b=engine.initialBoard('flux');assert.throws(()=>engine.moveBoard('flux',b,7));assert.throws(()=>engine.moveBoard('flux',b,1.5));for(let i=0;i<6;i++)b=engine.moveBoard('flux',b,0);assert.throws(()=>engine.moveBoard('flux',b,0));
 b=engine.initialBoard('flux');for(const c of[0,1,1,2,4,2,2,3,4,3,5,3,3])b=engine.moveBoard('flux',b,c);assert.equal(b.winner,1);
});
test('Reversi legal turns preserve occupancy and finish 100 varied games',()=>{
 for(let seed=0;seed<100;seed++){let b=engine.initialBoard('reversi'),n=0;while(b.winner===null){const legal=engine.legalMoves('reversi',b);assert.ok(legal.length);const count=b.cells.filter(Boolean).length;b=engine.moveBoard('reversi',b,legal[(seed*17+n*11)%legal.length]);assert.equal(b.cells.filter(Boolean).length,count+1);assert.ok(++n<=32)}const one=b.cells.filter(v=>v===1).length,two=b.cells.filter(v=>v===2).length;assert.equal(b.winner,one===two?0:one>two?1:2)}
});
test('A player is their QOR ID: named by it, keyed by a hash of the account, never the account id; origins enforced',async()=>{
 assert.equal(A.name,'player-a');assert.equal(A.qorVerified,true);assert.match(A.id,/^[0-9a-f]{64}$/);assert.notEqual(A.id,'sub-player-a');
 globalThis.__arenaTest.user=null;await assert.rejects(store.identity(),e=>e.status===401&&/QOR ID/.test(e.message));
 globalThis.__arenaTest.user={sub:'sub-player-a',qorId:'renamed',username:'renamed',chainAccount:null};const again=await store.identity();assert.equal(again.id,A.id);assert.equal(again.name,'renamed','a renamed QOR ID shows at once');
 await user('player-a');assert.throws(()=>store.sameOrigin(req({},'https://evil.test')),e=>e.status===403);assert.throws(()=>store.sameOrigin(new Request('https://arcade.test/api/chat')),e=>e.status===403);
});
test('Two-player lifecycle: unauthorized moves, racing moves, exact-once ranks and early resign',async()=>{
 let m=await store.createMatch(A.id,'flux');assert.equal(typeof m.created,'number');await assert.rejects(store.actionMatch(m,A.id,'join'),e=>e.status===400);m=await store.actionMatch(m,B.id,'join');await assert.rejects(store.actionMatch(m,C.id,'move',0,m.revision),e=>e.status===403);await assert.rejects(store.actionMatch(m,B.id,'move',0,m.revision),e=>e.status===400);
 const race=await Promise.allSettled([store.actionMatch(m,A.id,'move',0,m.revision),store.actionMatch(m,A.id,'move',0,m.revision)]);assert.equal(race.filter(r=>r.status==='fulfilled').length,1);assert.equal(race.find(r=>r.status==='rejected').reason.status,409);m=await store.getMatch(m.id);
 for(const [p,c]of[[B,1],[A,0],[B,1],[A,0],[B,1]])m=await store.actionMatch(m,p.id,'move',c,m.revision);const old=m;m=await store.actionMatch(m,A.id,'move',0,m.revision);assert.equal(m.status,'done');assert.equal(m.settled,1);await assert.rejects(store.actionMatch(old,A.id,'move',0,old.revision),e=>e.status===409);
 const rank=await one('SELECT * FROM standings WHERE player=$1',[A.id]);assert.equal(rank.points,30);assert.equal(rank.wins,1);assert.equal((await one('SELECT points FROM standings WHERE player=$1',[B.id])).points,5);
 let early=await store.createMatch(A.id,'reversi');early=await store.actionMatch(early,B.id,'join');early=await store.actionMatch(early,A.id,'resign',null,early.revision);assert.equal(early.status,'done');assert.equal((await one('SELECT COUNT(*) AS n FROM standings WHERE game=$1',['reversi'])).n,0);
 let timeout=await store.createMatch(B.id,'flux');timeout=await store.actionMatch(timeout,A.id,'join');await one('UPDATE matches SET deadline=0 WHERE id=$1',[timeout.id]);timeout=await store.expire(await store.getMatch(timeout.id));assert.equal(timeout.winner,2);assert.equal(timeout.status,'done');
});
test('Database batch rolls back on errors',async()=>{const before=(await one('SELECT seen FROM players WHERE id=$1',[A.id])).seen;await assert.rejects(db.batch([db.prepare('UPDATE players SET seen=1 WHERE id=?').bind(A.id),db.prepare('INSERT INTO nonexistent VALUES (1)')]));assert.equal((await one('SELECT seen FROM players WHERE id=$1',[A.id])).seen,before)});
test('Chat enforces identity, atomic rate limits, message limits and ownership',async()=>{
 await user('player-a');assert.equal((await chat.POST(req({action:'send',text:'x'.repeat(281)}))).status,400);assert.equal((await chat.POST(req({action:'send',text:'hello'},'https://evil.test'))).status,403);
 const sent=await chat.POST(req({action:'send',text:'Local test <script> inert text',player:B.id,qor_id:'forged'}));assert.equal(sent.status,201);const{id}=await sent.json();assert.equal((await one('SELECT player FROM messages WHERE id=$1',[id])).player,A.id);
 const race=await Promise.all([chat.POST(req({action:'send',text:'too soon'})),chat.POST(req({action:'send',text:'also too soon'}))]);assert.deepEqual(race.map(r=>r.status),[429,429]);
 await user('player-b');await chat.POST(req({action:'delete',id}));assert.equal((await one('SELECT deleted FROM messages WHERE id=$1',[id])).deleted,0);assert.equal((await chat.POST(req({action:'report',id,reason:'Spam'}))).status,200);await chat.POST(req({action:'report',id,reason:'Spam'}));assert.equal((await one('SELECT COUNT(*) AS n FROM reports')).n,1);
 await user('player-a');await chat.POST(req({action:'delete',id}));assert.equal((await one('SELECT deleted FROM messages WHERE id=$1',[id])).deleted,1);
});
test('Global ranking aggregates both games before top-100 limit and omits deleted chat',async()=>{
 await one('INSERT INTO standings(player,game,wins,points,updated) VALUES ($1,$2,$3,$4,$5)',[B.id,'reversi',2,60,Date.now()]);await user('player-a');const r=await live.GET();assert.equal(r.status,200);const data=await r.json();const global=data.standings.filter(s=>s.game==='all');assert.equal(global[0].player,B.id);assert.equal(global[0].points,65);assert.equal(global[0].alias,'player-b');assert.equal(typeof data.online,'number');assert.equal(data.messages.length,0);assert.equal(data.me.id,A.id);
});
test('Without a database the arcade says it is being connected',async()=>{
 const off=await import(storeWith('store-off',"()=>{throw new Unavailable('DATABASE_URL is not set.')}"));
 assert.throws(()=>off.database(),e=>e.status===503&&/being connected/.test(e.message));
});
