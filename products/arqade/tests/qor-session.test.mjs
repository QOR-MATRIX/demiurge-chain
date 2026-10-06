import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {compile,postgres} from './postgres.mjs';

// The real sign-in module against a stand-in QOR ID and Postgres (PGlite) with the site's migrations.
const Q=await import(compile('lib/qor-session.ts','qor-session'));
async function database(){const {pg,db}=await postgres();return {db,rows:async(sql,params=[])=>(await pg.query(sql,params)).rows}}

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
      return sub?Response.json({sub,qor_id:'player',username:'player',chain_account:null}):new Response(null,{status:401});
    }
    if(u.pathname==='/oauth/progress'){if(form.get('client_secret')!==CONFIG.clientSecret)return new Response(null,{status:401});s.reports=[...(s.reports||[]),{token:form.get('token'),task:form.get('task')}];return s.access.has(form.get('token'))?Response.json({level:0}):new Response(null,{status:401})}
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
  const {rows,db}=await database();const q=qorId();
  const {location,state}=await Q.startLogin(deps(db,q));
  const u=new URL(location);
  assert.equal(u.origin+u.pathname,'https://id.test/oauth/authorize');
  assert.equal(u.searchParams.get('client_id'),'arqade');
  assert.equal(u.searchParams.get('redirect_uri'),CONFIG.redirectUri);
  assert.equal(u.searchParams.get('code_challenge_method'),'S256');
  assert.equal(u.searchParams.get('state'),state);
  const [row]=await rows('SELECT verifier FROM qor_logins WHERE state=$1',[state]);
  assert.equal(u.searchParams.get('code_challenge'),b64url(createHash('sha256').update(row.verifier).digest()));
  assert.equal(u.searchParams.get('code_verifier'),null,'the verifier never leaves the server');
});

test('finishing needs the state this browser carried, once, within ten minutes',async()=>{
  const {db}=await database();const q=qorId();
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
  const {rows,db}=await database();const q=qorId();
  const {session,profile}=await signedIn(db,q);
  assert.equal(profile.sub,'sub-1');
  const all=await rows('SELECT id,access_token,refresh_token,sub FROM qor_sessions');
  assert.equal(all.length,1);
  assert.notEqual(all[0].id,session);
  assert.equal(all[0].id,b64url(createHash('sha256').update(session).digest()));
  assert.ok(all[0].access_token&&all[0].refresh_token);
});

test('who is signed in is asked of QOR ID; an expired access token is refreshed and rotated',async()=>{
  const {rows,db}=await database();const q=qorId();
  const {session}=await signedIn(db,q);
  assert.equal((await Q.currentProfile(deps(db,q),session)).qorId,'player');
  const [before]=await rows('SELECT access_token,refresh_token FROM qor_sessions');
  q.access.delete(before.access_token); // the access token expired at QOR ID
  assert.equal((await Q.currentProfile(deps(db,q),session)).username,'player');
  const [after]=await rows('SELECT access_token,refresh_token FROM qor_sessions');
  assert.notEqual(after.refresh_token,before.refresh_token,'rotated');
  assert.equal(await Q.currentProfile(deps(db,q),'not a session'),null);
});

test('a session QOR ID ended ends here at once',async()=>{
  const {rows,db}=await database();const q=qorId();
  const {session}=await signedIn(db,q);
  q.access.clear();q.refresh.clear(); // revoked at QOR ID
  assert.equal(await Q.currentProfile(deps(db,q),session),null);
  assert.equal((await rows('SELECT COUNT(*) AS n FROM qor_sessions'))[0].n,0);
});

test('a session past its eight hours ends without asking QOR ID',async()=>{
  const {db}=await database();const q=qorId();
  let t=1_000_000;
  const {session}=await signedIn(db,q,()=>t);
  t+=Q.SESSION_TTL_MS+1;q.calls.length=0;
  assert.equal(await Q.currentProfile(deps(db,q,()=>t),session),null);
  assert.deepEqual(q.calls,[]);
});

test('sign-out deletes the session here and revokes it at QOR ID',async()=>{
  const {rows,db}=await database();const q=qorId();
  const {session}=await signedIn(db,q);
  const [{refresh_token:refresh}]=await rows('SELECT refresh_token FROM qor_sessions');
  await Q.logout(deps(db,q),session);
  assert.equal((await rows('SELECT COUNT(*) AS n FROM qor_sessions'))[0].n,0);
  assert.deepEqual(q.revoked,[refresh]);
});

test('the live arcade reuses the answer from QOR ID for 30 seconds, then asks again; the identity card always asks',async()=>{
  const {db}=await database();const q=qorId();
  let t=1_000_000;const clock=()=>t;
  const {session}=await signedIn(db,q,clock);
  q.calls.length=0;
  assert.equal((await Q.currentProfile(deps(db,q,clock),session,Q.ARCADE_RECHECK_MS)).qorId,'player');
  assert.deepEqual(q.calls,[],'answered from the session QOR ID confirmed at sign-in');
  assert.equal((await Q.currentProfile(deps(db,q,clock),session)).qorId,'player');
  assert.deepEqual(q.calls,['/oauth/userinfo'],'no allowance given: QOR ID is asked');
  q.access.clear();q.refresh.clear(); // revoked at QOR ID
  t+=Q.ARCADE_RECHECK_MS-1;q.calls.length=0;
  assert.ok(await Q.currentProfile(deps(db,q,clock),session,Q.ARCADE_RECHECK_MS),'within the allowance');
  t+=2;
  assert.equal(await Q.currentProfile(deps(db,q,clock),session,Q.ARCADE_RECHECK_MS),null,'past it, the revocation ends the session');
  assert.ok(q.calls.includes('/oauth/userinfo'));
});

test('sign-in is off unless the site is configured with a client and a real secret',()=>{
  assert.equal(Q.configFrom({}),null);
  assert.equal(Q.configFrom({QOR_CLIENT_ID:'arqade',QOR_CLIENT_SECRET:'short',QOR_REDIRECT_URI:'https://a/cb'}),null);
  const c=Q.configFrom({QOR_CLIENT_ID:'arqade',QOR_CLIENT_SECRET:'x'.repeat(32),QOR_REDIRECT_URI:'https://a/cb'});
  assert.equal(c.issuer,'https://id.qorsync.dev');
  assert.equal(Q.cookie('a=1; arq_session=abc; b=2','arq_session'),'abc');
  assert.match(Q.setCookie('arq_session','v',60,true),/HttpOnly; SameSite=Lax; Max-Age=60; Secure$/);
});

test('a task is reported to QOR ID with the secret of the app and the token of the player, and not without a session',async()=>{
  const {db}=await database();const q=qorId();
  const {session}=await signedIn(db,q);
  assert.equal(await Q.reportTask(deps(db,q),session,'first-match'),true);
  assert.equal(q.reports.length,1);
  assert.equal(q.reports[0].task,'first-match');
  assert.ok(q.access.has(q.reports[0].token),'the access token QOR ID issued to this session');
  assert.equal(await Q.reportTask(deps(db,q),null,'first-match'),false,'no session, nothing reported');
  assert.equal(await Q.reportTask(deps(db,q),'not a session','first-match'),false);
  assert.equal(q.reports.length,1);
});
