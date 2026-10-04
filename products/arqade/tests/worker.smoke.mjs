import assert from 'node:assert/strict';
const origin='http://127.0.0.1:8787',tag=Date.now();
async function call(who,path,data){const r=await fetch(origin+path,{method:data?'POST':'GET',headers:{'oai-authenticated-user-id':`qa-${tag}-${who}`,'oai-authenticated-user-email':`${who}@example.test`,Origin:origin,'Content-Type':'application/json'},body:data?JSON.stringify(data):undefined});return{status:r.status,data:await r.json()}}
assert.equal((await fetch(origin+'/api/live')).status,401);
const a=await call('a','/api/live'),b=await call('b','/api/live');assert.equal(a.status,200);assert.equal(b.status,200);assert.notEqual(a.data.me.id,b.data.me.id);
let r=await call('a','/api/matches',{game:'flux'});assert.equal(r.status,201);let m=r.data;const path='/api/matches/'+m.id;
r=await call('b',path,{action:'join'});assert.equal(r.status,200);m=r.data;
for(const[who,cell]of[['a',0],['b',1],['a',0],['b',1],['a',0],['b',1],['a',0]]){r=await call(who,path,{action:'move',cell,revision:m.revision});assert.equal(r.status,200);m=r.data}
assert.equal(m.status,'done');assert.equal(m.state.winner,1);const ranks=(await call('a','/api/live')).data.standings;assert.equal(ranks.find(p=>p.player===a.data.me.id&&p.game==='all').points,30);
const sent=await call('a','/api/chat',{action:'send',text:'Local API verification. Disposable test message.'});assert.equal(sent.status,201);assert.ok((await call('b','/api/live')).data.messages.some(x=>x.id===sent.data.id));await call('a','/api/chat',{action:'delete',id:sent.data.id});assert.ok(!(await call('b','/api/live')).data.messages.some(x=>x.id===sent.data.id));
console.log('PASS: production worker HTTP, two authenticated players, full Flux match, persistent rankings, cross-player chat and deletion, unauthenticated 401. Local database only.');

