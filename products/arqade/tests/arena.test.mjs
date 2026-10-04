import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DatabaseSync} from 'node:sqlite';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

// Run the real rules and SQL against isolated SQLite, injecting only platform bindings.
const dir=path.resolve('work/arena-tests');fs.mkdirSync(dir,{recursive:true});
function compile(source,name,replacements=[]){let s=fs.readFileSync(source,'utf8');for(const [a,b]of replacements)s=s.replace(a,b);const output=path.join(dir,name+'.mjs');fs.writeFileSync(output,ts.transpileModule(s,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);return pathToFileURL(output).href}
const engine=await import(compile('lib/arena-engine.ts','engine'));
const sqlite=new DatabaseSync(':memory:');sqlite.exec(fs.readFileSync('drizzle/0000_new_mother_askani.sql','utf8'));
const db={prepare(sql){return{bind(...values){return statement(sql,values)},...statement(sql,[])}},async batch(statements){sqlite.exec('BEGIN');try{const result=statements.map(s=>s.execute());sqlite.exec('COMMIT');return result}catch(e){sqlite.exec('ROLLBACK');throw e}}};
function statement(sql,values){return{execute(){const p=sqlite.prepare(sql);if(p.columns().length)return{results:p.all(...values),meta:{changes:0}};const r=p.run(...values);return{results:[],meta:{changes:Number(r.changes)}}},async run(){return this.execute()},async all(){return this.execute()},async first(){return this.execute().results[0]??null}}}
globalThis.__arenaTest={env:{DB:db},user:null};
const store=await import(compile('lib/arena-store.ts','store',[["import {env} from 'cloudflare:workers';","const env=globalThis.__arenaTest.env;"],["import {getChatGPTUser} from '../app/chatgpt-auth';","const getChatGPTUser=async()=>globalThis.__arenaTest.user;"],["from './arena-engine'","from './engine.mjs'"]]));
const chat=await import(compile('app/api/chat/route.ts','chat',[["from '@/lib/arena-store'","from './store.mjs'"]]));
const live=await import(compile('app/api/live/route.ts','live',[["from '@/lib/arena-store'","from './store.mjs'"]]));
async function user(id){globalThis.__arenaTest.user={userId:id};return store.identity()}
const A=await user('test-player-a'),B=await user('test-player-b'),C=await user('test-spectator');
const req=(data,origin='https://arcade.test')=>new Request('https://arcade.test/api/chat',{method:'POST',headers:{Origin:origin,'Content-Type':'application/json'},body:JSON.stringify(data)});

test('Flux rejects illegal moves and resolves vertical and diagonal wins',()=>{
 let b=engine.initialBoard('flux');for(const c of[0,1,0,1,0,1,0])b=engine.moveBoard('flux',b,c);assert.equal(b.winner,1);assert.throws(()=>engine.moveBoard('flux',b,2));
 b=engine.initialBoard('flux');assert.throws(()=>engine.moveBoard('flux',b,7));assert.throws(()=>engine.moveBoard('flux',b,1.5));for(let i=0;i<6;i++)b=engine.moveBoard('flux',b,0);assert.throws(()=>engine.moveBoard('flux',b,0));
 b=engine.initialBoard('flux');for(const c of[0,1,1,2,4,2,2,3,4,3,5,3,3])b=engine.moveBoard('flux',b,c);assert.equal(b.winner,1);
});
test('Reversi legal turns preserve occupancy and finish 100 varied games',()=>{
 for(let seed=0;seed<100;seed++){let b=engine.initialBoard('reversi'),n=0;while(b.winner===null){const legal=engine.legalMoves('reversi',b);assert.ok(legal.length);const count=b.cells.filter(Boolean).length;b=engine.moveBoard('reversi',b,legal[(seed*17+n*11)%legal.length]);assert.equal(b.cells.filter(Boolean).length,count+1);assert.ok(++n<=32)}const one=b.cells.filter(v=>v===1).length,two=b.cells.filter(v=>v===2).length;assert.equal(b.winner,one===two?0:one>two?1:2)}
});
test('Identity is server-derived, private aliases reveal no account data; origins enforced',async()=>{
 assert.match(A.name,/^Explorer-[A-F0-9]{8}$/);assert.equal(A.qorVerified,false);globalThis.__arenaTest.user=null;await assert.rejects(store.identity(),e=>e.status===401);await user('test-player-a');assert.throws(()=>store.sameOrigin(req({},'https://evil.test')),e=>e.status===403);assert.throws(()=>store.sameOrigin(new Request('https://arcade.test/api/chat')),e=>e.status===403);
});
test('Two-player lifecycle: unauthorized moves, racing moves, exact-once ranks and early resign',async()=>{
 let m=await store.createMatch(A.id,'flux');await assert.rejects(store.actionMatch(m,A.id,'join'),e=>e.status===400);m=await store.actionMatch(m,B.id,'join');await assert.rejects(store.actionMatch(m,C.id,'move',0,m.revision),e=>e.status===403);await assert.rejects(store.actionMatch(m,B.id,'move',0,m.revision),e=>e.status===400);
 const race=await Promise.allSettled([store.actionMatch(m,A.id,'move',0,m.revision),store.actionMatch(m,A.id,'move',0,m.revision)]);assert.equal(race.filter(r=>r.status==='fulfilled').length,1);assert.equal(race.find(r=>r.status==='rejected').reason.status,409);m=await store.getMatch(m.id);
 for(const [p,c]of[[B,1],[A,0],[B,1],[A,0],[B,1]])m=await store.actionMatch(m,p.id,'move',c,m.revision);const old=m;m=await store.actionMatch(m,A.id,'move',0,m.revision);assert.equal(m.status,'done');assert.equal(m.settled,1);await assert.rejects(store.actionMatch(old,A.id,'move',0,old.revision),e=>e.status===409);
 let rank=sqlite.prepare('SELECT * FROM standings WHERE player=?').get(A.id);assert.equal(rank.points,30);assert.equal(rank.wins,1);assert.equal(sqlite.prepare('SELECT points FROM standings WHERE player=?').get(B.id).points,5);
 let early=await store.createMatch(A.id,'reversi');early=await store.actionMatch(early,B.id,'join');early=await store.actionMatch(early,A.id,'resign',null,early.revision);assert.equal(early.status,'done');assert.equal(sqlite.prepare('SELECT COUNT(*) AS n FROM standings WHERE game=?').get('reversi').n,0);
 let timeout=await store.createMatch(B.id,'flux');timeout=await store.actionMatch(timeout,A.id,'join');sqlite.prepare('UPDATE matches SET deadline=0 WHERE id=?').run(timeout.id);timeout=await store.expire(await store.getMatch(timeout.id));assert.equal(timeout.winner,2);assert.equal(timeout.status,'done');
});
test('Database batch rolls back on errors',async()=>{const before=sqlite.prepare('SELECT seen FROM players WHERE id=?').get(A.id).seen;await assert.rejects(db.batch([db.prepare('UPDATE players SET seen=1 WHERE id=?').bind(A.id),db.prepare('INSERT INTO nonexistent VALUES (1)')]));assert.equal(sqlite.prepare('SELECT seen FROM players WHERE id=?').get(A.id).seen,before)});
test('Chat enforces identity, atomic rate limits, message limits and ownership',async()=>{
 await user('test-player-a');assert.equal((await chat.POST(req({action:'send',text:'x'.repeat(281)}))).status,400);assert.equal((await chat.POST(req({action:'send',text:'hello'},'https://evil.test'))).status,403);
 const sent=await chat.POST(req({action:'send',text:'Local test <script> inert text',player:B.id,qor_id:'forged'}));assert.equal(sent.status,201);const{id}=await sent.json();assert.equal(sqlite.prepare('SELECT player FROM messages WHERE id=?').get(id).player,A.id);
 const race=await Promise.all([chat.POST(req({action:'send',text:'too soon'})),chat.POST(req({action:'send',text:'also too soon'}))]);assert.deepEqual(race.map(r=>r.status),[429,429]);
 await user('test-player-b');await chat.POST(req({action:'delete',id}));assert.equal(sqlite.prepare('SELECT deleted FROM messages WHERE id=?').get(id).deleted,0);assert.equal((await chat.POST(req({action:'report',id,reason:'Spam'}))).status,200);await chat.POST(req({action:'report',id,reason:'Spam'}));assert.equal(sqlite.prepare('SELECT COUNT(*) AS n FROM reports').get().n,1);
 await user('test-player-a');await chat.POST(req({action:'delete',id}));assert.equal(sqlite.prepare('SELECT deleted FROM messages WHERE id=?').get(id).deleted,1);
});
test('Global ranking aggregates both games before top-100 limit and omits deleted chat',async()=>{
 sqlite.prepare('INSERT INTO standings(player,game,wins,points,updated) VALUES (?,?,?,?,?)').run(B.id,'reversi',2,60,Date.now());await user('test-player-a');const r=await live.GET();assert.equal(r.status,200);const data=await r.json();const global=data.standings.filter(s=>s.game==='all');assert.equal(global[0].player,B.id);assert.equal(global[0].points,65);assert.equal(data.messages.length,0);assert.equal(data.me.id,A.id);
});
