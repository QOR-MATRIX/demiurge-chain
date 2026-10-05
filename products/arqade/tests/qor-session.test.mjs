import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DatabaseSync} from 'node:sqlite';
import {createHash} from 'node:crypto';
import fs from 'node:fs';
import path from 'node:path';
import {pathToFileURL} from 'node:url';
import ts from 'typescript';

// The real sign-in module against a stand-in QOR ID and the arcade's own migrations in SQLite.
const dir=path.resolve('work/qor-session-tests');fs.mkdirSync(dir,{recursive:true});
const out=path.join(dir,'qor-session.mjs');
fs.writeFileSync(out,ts.transpileModule(fs.readFileSync('lib/qor-session.ts','utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText);
const Q=await import(pathToFileURL(out).href);

function database(){
  const sqlite=new DatabaseSync(':memory:');
  for(const f of fs.readdirSync('drizzle').filter(f=>f.endsWith('.sql')).sort())
    for(const stmt of fs.readFileSync(path.join('drizzle',f),'utf8').split('--> statement-breakpoint')) if(stmt.trim()) sqlite.exec(stmt);
  const db={prepare:sql=>({bind:(...v)=>({run:async()=>sqlite.prepare(sql).run(...v),first:async()=>sqlite.prepare(sql).get(...v)??null})})};
  return {sqlite,db};
}

const CONFIG={issuer:'https://id.test',clientId:'arqade',clientSecret:'s'.repeat(40),redirectUri:'https://arqade.test/api/auth/callback'};
const b64url=b=>Buffer.from(b).toString('base64').replace(/\+/g,'-').replace(/\//g,'_').replace(/=+$/,'');

/** A stand-in QOR ID: codes bound to a challenge, rotating refresh tokens, revocable sessions. */
function qorId(){
  const s={codes:new Map(),access:new Map(),refresh:new Map(),revoked:[],calls:[]};
  let n=0;
  const issue=sub=>{const a='acc'+(++n),r='ref'+n;s.access.set(a,sub);s.refresh.set(r,sub);return {access_token:a,refresh_token:r,expires_in:900}};
  s.code=(challenge,sub='sub-1')=>{const c='code'+(++n);s.codes.set(c,{challenge,sub});return c};
  s.fetch=async(url,init={})=>{
    const u=new URL(url);s.calls.push(u.pathname);
    const form=new URLSearchParams(init.body||'');
    if(u.pathname==='/oauth/token'){
      if(form.get('client_secret')!==CONFIG.clientSecret)return Response.json({error:'invalid_client'},{status:401});
      if(form.get('grant_type')==='authorization_code'){
        const c=s.codes.get(form.get('code'));s.codes.delete(form.get('code'));
        const ok=c&&b64url(createHash('sha256').update(form.get('code_verifier')).digest())===c.challenge&&form.get('redirect_uri')===CONFIG.redirectUri;
        return ok?Response.json(issue(c.sub)):Response.json({error:'invalid_grant'},{status:400});
      }
      const sub=s.refresh.get(form.get('refresh_token'));s.refresh.delete(form.get('refresh_token'));
      return sub?Response.json(issue(sub)):Response.json({error:'invalid_grant'},{status:400});
    }
    if(u.pathname==='/oauth/userinfo'){
      const sub=s.access.get((init.headers?.authorization||'').replace('Bearer ',''));
      return sub?Response.json({sub,qor_id:'player#0001',username:'player',chain_account:null}):new Response(null,{status:401});
    }
    if(u.pathname==='/oauth/revoke'){s.revoked.push(form.get('token'));s.refresh.delete(form.get('token'));return new Response(null,{status:200})}
    return new Response(null,{status:404});
  };
  return s;
}

function deps(db,q,now=()=>1_000_000){return {db,fetch:q.fetch,now,config:CONFIG}}

async function signedIn(db,q,clock){
  const d=deps(db,q,clock);
  const {location,state}=await Q.startLogin(d);
  const challenge=new URL(location).searchParams.get('code_challenge');
  const code=q.code(challenge);
  return Q.finishLogin(d,new URLSearchParams({code,state}),state);
}

test('a sign-in starts at QOR ID with a state and an S256 challenge of a verifier kept here',async()=>{
  const {sqlite,db}=database();const q=qorId();
  const {location,state}=await Q.startLogin(deps(db,q));
  const u=new URL(location);
  assert.equal(u.origin+u.pathname,'https://id.test/oauth/authorize');
  assert.equal(u.searchParams.get('client_id'),'arqade');
  assert.equal(u.searchParams.get('redirect_uri'),CONFIG.redirectUri);
  assert.equal(u.searchParams.get('code_challenge_method'),'S256');
  assert.equal(u.searchParams.get('state'),state);
  const row=sqlite.prepare('SELECT verifier FROM qor_logins WHERE state=?').get(state);
  assert.equal(u.searchParams.get('code_challenge'),b64url(createHash('sha256').update(row.verifier).digest()));
  assert.equal(u.searchParams.get('code_verifier'),null,'the verifier never leaves the server');
});

test('finishing needs the state this browser carried, once, within ten minutes',async()=>{
  const {db}=database();const q=qorId();
  let t=1_000_000;const clock=()=>t;
  const d=deps(db,q,clock);
  const {location,state}=await Q.startLogin(d);
  const code=q.code(new URL(location).searchParams.get('code_challenge'));
  await assert.rejects(Q.finishLogin(d,new URLSearchParams({code,state}),null),/did not start in this browser/);
  await assert.rejects(Q.finishLogin(d,new URLSearchParams({code,state}),'another'),/did not start in this browser/);
  // The mismatch did not spend the pending sign-in; the right browser still can.
  const ok=await Q.finishLogin(d,new URLSearchParams({code,state}),state);
  assert.equal(ok.profile.username,'player');
  await assert.rejects(Q.finishLogin(d,new URLSearchParams({code,state}),state),/expired/,'single use');

  const late=await Q.startLogin(d);
  t+=Q.LOGIN_TTL_MS+1;
  await assert.rejects(Q.finishLogin(d,new URLSearchParams({code:'x',state:late.state}),late.state),/expired/);
  const refused=await Q.startLogin(d);
  await assert.rejects(Q.finishLogin(d,new URLSearchParams({error:'access_denied',state:refused.state}),refused.state),/did not sign you in/);
});

test('the session is stored by the hash of the cookie, never the cookie, with its tokens server-side',async()=>{
  const {sqlite,db}=database();const q=qorId();
  const {session,profile}=await signedIn(db,q);
  assert.equal(profile.sub,'sub-1');
  const rows=sqlite.prepare('SELECT id,access_token,refresh_token,sub FROM qor_sessions').all();
  assert.equal(rows.length,1);
  assert.notEqual(rows[0].id,session);
  assert.equal(rows[0].id,b64url(createHash('sha256').update(session).digest()));
  assert.ok(rows[0].access_token&&rows[0].refresh_token);
});

test('who is signed in is asked of QOR ID; an expired access token is refreshed and rotated',async()=>{
  const {sqlite,db}=database();const q=qorId();
  const {session}=await signedIn(db,q);
  assert.equal((await Q.currentProfile(deps(db,q),session)).qorId,'player#0001');
  const before=sqlite.prepare('SELECT access_token,refresh_token FROM qor_sessions').get();
  q.access.delete(before.access_token); // the access token expired at QOR ID
  assert.equal((await Q.currentProfile(deps(db,q),session)).username,'player');
  const after=sqlite.prepare('SELECT access_token,refresh_token FROM qor_sessions').get();
  assert.notEqual(after.refresh_token,before.refresh_token,'rotated');
  assert.equal(await Q.currentProfile(deps(db,q),'not a session'),null);
});

test('a session QOR ID ended ends here at once',async()=>{
  const {sqlite,db}=database();const q=qorId();
  const {session}=await signedIn(db,q);
  q.access.clear();q.refresh.clear(); // revoked at QOR ID
  assert.equal(await Q.currentProfile(deps(db,q),session),null);
  assert.equal(sqlite.prepare('SELECT COUNT(*) n FROM qor_sessions').get().n,0);
});

test('a session past its eight hours ends without asking QOR ID',async()=>{
  const {db}=database();const q=qorId();
  let t=1_000_000;
  const {session}=await signedIn(db,q,()=>t);
  t+=Q.SESSION_TTL_MS+1;q.calls.length=0;
  assert.equal(await Q.currentProfile(deps(db,q,()=>t),session),null);
  assert.deepEqual(q.calls,[]);
});

test('sign-out deletes the session here and revokes it at QOR ID',async()=>{
  const {sqlite,db}=database();const q=qorId();
  const {session}=await signedIn(db,q);
  const refresh=sqlite.prepare('SELECT refresh_token FROM qor_sessions').get().refresh_token;
  await Q.logout(deps(db,q),session);
  assert.equal(sqlite.prepare('SELECT COUNT(*) n FROM qor_sessions').get().n,0);
  assert.deepEqual(q.revoked,[refresh]);
});

test('an arcade explorer is bound to one QOR identity, and one QOR identity to one explorer',async()=>{
  const {sqlite,db}=database();
  for(const id of ['p1','p2'])sqlite.prepare('INSERT INTO players (id,alias,created,seen) VALUES (?,?,?,?)').run(id,'Explorer-'+id,1,1);
  assert.equal(await Q.linkPlayer(db,'p1','sub-1'),'linked');
  assert.equal(await Q.linkPlayer(db,'p1','sub-1'),'already');
  assert.equal(await Q.linkPlayer(db,'p2','sub-1'),'taken');
  assert.equal(await Q.linkPlayer(db,'p1','sub-2'),'taken','an explorer already bound keeps its identity');
  assert.equal(sqlite.prepare('SELECT qor_id FROM players WHERE id=?').get('p2').qor_id,null);
});

test('sign-in is off unless the site is configured with a client and a real secret',()=>{
  assert.equal(Q.configFrom({}),null);
  assert.equal(Q.configFrom({QOR_CLIENT_ID:'arqade',QOR_CLIENT_SECRET:'short',QOR_REDIRECT_URI:'https://a/cb'}),null);
  const c=Q.configFrom({QOR_CLIENT_ID:'arqade',QOR_CLIENT_SECRET:'x'.repeat(32),QOR_REDIRECT_URI:'https://a/cb'});
  assert.equal(c.issuer,'https://id.qorsync.dev');
  assert.equal(Q.cookie('a=1; arq_session=abc; b=2','arq_session'),'abc');
  assert.match(Q.setCookie('arq_session','v',60,true),/HttpOnly; SameSite=Lax; Max-Age=60; Secure$/);
});
