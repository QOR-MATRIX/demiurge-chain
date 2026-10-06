import {test} from 'node:test';
import assert from 'node:assert/strict';
import {webcrypto as nodeCrypto, createPublicKey, verify} from 'node:crypto';
import fs from 'node:fs';
import {compile,postgres} from './postgres.mjs';

// Tips paid through the QOR Launcher (ADR-071 decision 6, ADR-076, ADR-077): the request ARQADE signs, how a payment
// is recognised in a block's events, and the tips flow against Postgres with a stand-in chain.
compile('lib/chain.ts','chain');
compile('sdk/src/amount.ts','amount');
const P=await import(compile('lib/pay.ts','pay',[["from './chain'","from './chain.mjs'"]]));
const T=await import(compile('lib/tips.ts','tips',[["from '../sdk/src/amount'","from './amount.mjs'"],["from './pay'","from './pay.mjs'"]]));

const SEED=Buffer.alloc(32,7);
const PKCS8=Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),SEED]).toString('base64');
const ALICE='5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const NOW=1_800_000_000;

test('the link ARQADE signs is the one the launcher pins and accepts',async()=>{
  const req=P.newRequest(ALICE,5n*10n**18n,'Tip for Flux Four',NOW-300,'0123456789abcdef0123456789abcdef');
  req.exp=NOW+300;
  const link=await P.payLink(req,await P.signingKey(PKCS8));
  const pinned=fs.readFileSync('../../tools/qor-launcher/src-tauri/src/pay.rs','utf8').match(/const ARQADE_LINK: &str = "([^"]+)"/)[1];
  assert.equal(link,pinned,'tools/qor-launcher/src-tauri/src/pay.rs pins this exact link');
  // The signature verifies over exactly the request's bytes, with the matching public key.
  const params=new URLSearchParams(link.slice('qor://pay?'.length));
  const bytes=Buffer.from(params.get('r'),'hex');
  assert.deepEqual(Object.keys(JSON.parse(bytes)),['v','app','id','to','amount','label','genesis','exp'],'the launcher refuses any other field');
  const pub=createPublicKey({key:Buffer.from('302a300506032b6570032100'+(await (async()=>{const k=await nodeCrypto.subtle.importKey('pkcs8',Buffer.from(PKCS8,'base64'),{name:'Ed25519'},true,['sign']);const jwk=await nodeCrypto.subtle.exportKey('jwk',k);return Buffer.from(jwk.x,'base64url').toString('hex')})()),'hex'),format:'der',type:'spki'});
  assert.ok(verify(null,bytes,pub,Buffer.from(params.get('s'),'hex')));
});

test('a request stays within the owner\'s cap and has a sane description',()=>{
  assert.throws(()=>P.newRequest(ALICE,0n,'Tip',NOW));
  assert.throws(()=>P.newRequest(ALICE,P.CAP_SPARKS+1n,'Tip',NOW));
  assert.ok(P.newRequest(ALICE,P.CAP_SPARKS,'Tip',NOW));
  assert.throws(()=>P.newRequest(ALICE,1n,'',NOW));
  assert.throws(()=>P.newRequest(ALICE,1n,'x'.repeat(81),NOW));
  assert.throws(()=>P.newRequest(ALICE,1n,'line\nbreak',NOW));
  assert.equal(P.newRequest(ALICE,1n,'Tip',NOW).exp-NOW,P.REQUEST_TTL_SECS);
  assert.ok(P.REQUEST_TTL_SECS<=15*60,'the launcher refuses more than fifteen minutes');
});

const ev=(extrinsic,section,method,data)=>({extrinsic,section,method,data});
const TO='0xaa',PAYER='0xbb',HASH='0xremark',AMOUNT='5000000000000000000';

test('a payment is the remark and the right transfer in one extrinsic, and nothing less',()=>{
  const remark=ev(2,'system','Remarked',{sender:PAYER,hash:HASH});
  const transfer=ev(2,'balances','Transfer',{from:PAYER,to:TO,amount:AMOUNT});
  assert.equal(P.paymentIn([remark,transfer],HASH,TO,AMOUNT),PAYER);
  assert.equal(P.paymentIn([remark],HASH,TO,AMOUNT),null,'a remark alone is not a payment');
  assert.equal(P.paymentIn([transfer],HASH,TO,AMOUNT),null,'a transfer alone is not this request');
  assert.equal(P.paymentIn([remark,{...transfer,extrinsic:3}],HASH,TO,AMOUNT),null,'in another extrinsic');
  assert.equal(P.paymentIn([remark,{...transfer,data:{...transfer.data,to:'0xcc'}}],HASH,TO,AMOUNT),null,'to someone else');
  assert.equal(P.paymentIn([remark,{...transfer,data:{...transfer.data,amount:'1'}}],HASH,TO,AMOUNT),null,'a different amount');
  assert.equal(P.paymentIn([remark,{...transfer,data:{...transfer.data,from:'0xdd'}}],HASH,TO,AMOUNT),null,'paid by someone other than the remark\'s sender');
  assert.equal(P.paymentIn([remark,transfer,ev(2,'system','ExtrinsicFailed',{})],HASH,TO,AMOUNT),null,'a failed extrinsic');
});

/** A stand-in chain: a finalised head that moves, and blocks that hold a payment once one is made. */
function chain(){
  const c={head:100,paidAt:null,scans:[]};
  c.finalizedNumber=async()=>c.head;
  c.scanForPayment=async(request,from,to)=>{c.scans.push([from,to]);const last=Math.min(to,from+39);if(c.paidAt!==null&&c.paidAt>=from&&c.paidAt<=last)return{found:{block:c.paidAt,hash:'0xblock',payer:PAYER},scannedTo:c.paidAt};return{found:null,scannedTo:last}};
  return c;
}

async function setup(){
  const {db,one}=await postgres();
  for(const id of ['p1','p2'])await one('INSERT INTO players (id,alias,created,seen) VALUES ($1,$2,1,1)',[id,id]);
  return {db,one,settings:{key:await P.signingKey(PKCS8),creator:ALICE}};
}

test('a tip is asked for only by name, amount and limits',async()=>{
  const {db,settings}=await setup();const c=chain();
  await assert.rejects(T.askTip(db,c,settings,'p1','nogame','10',NOW*1000),e=>e.status===400);
  await assert.rejects(T.askTip(db,c,settings,'p1','flux','ten',NOW*1000),e=>e.status===400);
  await assert.rejects(T.askTip(db,c,settings,'p1','flux','100000.000000000000000001',NOW*1000),e=>e.status===400);
  await assert.rejects(T.askTip(db,c,settings,'p1','flux','0',NOW*1000),e=>e.status===400);
  const tip=await T.askTip(db,c,settings,'p1','flux','2.5',NOW*1000);
  assert.equal(tip.amount,'2500000000000000000');
  assert.equal(tip.label,'Tip for Flux Four');
  assert.match(tip.link,/^qor:\/\/pay\?r=[0-9a-f]+&s=[0-9a-f]{128}$/);
  for(let i=0;i<4;i++)await T.askTip(db,c,settings,'p1','flux','1',NOW*1000);
  await assert.rejects(T.askTip(db,c,settings,'p1','flux','1',NOW*1000),e=>e.status===429,'five open at once');
  assert.ok(await T.askTip(db,c,settings,'p2','flux','1',NOW*1000),'another player is not limited by the first');
});

test('a tip is paid when a finalised block holds it, searched from where the last check stopped',async()=>{
  const {db,settings}=await setup();const c=chain();
  const tip=await T.askTip(db,c,settings,'p1','orbital','3',NOW*1000);
  c.head=105;
  assert.equal((await T.tipStatus(db,c,'p1',tip.id,NOW*1000)).status,'waiting');
  assert.deepEqual(c.scans.at(-1),[101,105],'from the block after the one finalised when it was asked');
  c.head=110;c.paidAt=108;
  const paid=await T.tipStatus(db,c,'p1',tip.id,NOW*1000);
  assert.equal(paid.status,'paid');assert.equal(paid.block,108);assert.equal(paid.payer,PAYER);
  assert.deepEqual(c.scans.at(-1),[106,110],'never searching a block twice');
  await assert.rejects(T.tipStatus(db,c,'p2',tip.id,NOW*1000),e=>e.status===404,'only the player who asked sees it');
  const again=c.scans.length;await T.tipStatus(db,c,'p1',tip.id,NOW*1000);
  assert.equal(c.scans.length,again,'a paid tip is not searched again');
});

test('an unpaid tip expires only after its time and the blocks after it are searched',async()=>{
  const {db,settings}=await setup();const c=chain();
  const tip=await T.askTip(db,c,settings,'p1','synapse','1',NOW*1000);
  const late=(tip.expires+120)*1000;
  assert.equal((await T.tipStatus(db,c,'p1',tip.id,(tip.expires-1)*1000)).status,'waiting','not before its time');
  c.head=200;
  assert.equal((await T.tipStatus(db,c,'p1',tip.id,late)).status,'waiting','blocks 101-140 searched; more to search');
  assert.equal((await T.tipStatus(db,c,'p1',tip.id,late)).status,'waiting','141-180');
  assert.equal((await T.tipStatus(db,c,'p1',tip.id,late)).status,'expired','181-200: every block searched, none paid');
});
