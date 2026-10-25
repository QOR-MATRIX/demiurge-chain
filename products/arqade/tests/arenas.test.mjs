import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as arena from '../games/arenas/logic/arena.mjs';
import {initialBoard,moveBoard,legalMoves} from '../lib/arena-engine.ts';

// The arenas' side of their conversation with ARQADE (P7.24, ADR-117): the module Flux Four and Rift Reversi run in QQ,
// here as Node runs it to replay a run. The boards below are made by the server's own rules (lib/arena-engine.ts), so
// what the game is told is what the server would send.
const ID='00000000-0000-4000-8000-000000000001',OTHER='00000000-0000-4000-8000-000000000002';
const snapshot=(game,board,o={})=>({id:ID,game,status:'active',revision:1,moves:0,deadline:90000,serverNow:0,
 host:{id:'p-1',name:'sophia',qorVerified:true},guest:{id:'p-2',name:'barbelo',qorVerified:true},seat:1,
 legal:board.winner===null?legalMoves(game,board):[],state:board,...o});
const lobby=(rows,signedIn=true,why='')=>({lobby:rows,signedIn,why});
const following=(game,seat=1)=>{const s=arena.create(game);arena.heard(s,lobby([]));arena.newMatch(s);
 arena.heard(s,{match:snapshot(game,initialBoard(game),{status:'waiting',guest:null,seat})});return s};
const playing=(game,seat=1)=>{const s=following(game,seat);arena.heard(s,{match:snapshot(game,initialBoard(game),{revision:2,seat})});return s};

test('it asks for the lobby, and keeps only this arena\'s open matches and the player\'s own',()=>{
 const s=arena.create('flux');assert.deepEqual(arena.begin(s),{ask:'lobby'});
 arena.heard(s,lobby([
  {id:ID,game:'flux',status:'waiting',alias:'sophia',seat:0},{id:OTHER,game:'reversi',status:'waiting',alias:'barbelo',seat:0},
  {id:'00000000-0000-4000-8000-000000000003',game:'flux',status:'active',alias:'me',seat:2},
  {id:'00000000-0000-4000-8000-000000000004',game:'flux',status:'active',alias:'zoe',seat:0}]));
 assert.deepEqual(s.open,[{id:ID,host:'sophia'}]);
 assert.deepEqual(s.mine,{id:'00000000-0000-4000-8000-000000000003',status:'active'});
 const say=arena.say(s);assert.equal(say.status,'1 open lobby');assert.equal(say.join,'Join sophia');
 assert.equal(say.canJoin,true);assert.equal(say.canReturn,true);assert.equal(say.canCreate,true);
 assert.deepEqual(arena.join(s),{ask:'join',id:ID});assert.equal(arena.join(s),null,'one ask at a time');
});

test('signed out it says why, and asks for nothing',()=>{
 const s=arena.create('reversi');arena.heard(s,lobby([{id:ID,game:'reversi',status:'waiting',alias:'sophia',seat:0}],false,'Sign in to the launcher.'));
 assert.equal(arena.say(s).status,'Sign in to the launcher.');assert.equal(arena.newMatch(s),null);assert.equal(arena.join(s),null);
 assert.equal(arena.say(s).canCreate,false);assert.equal(arena.say(s).canJoin,false);
 arena.heard(s,lobby([],false));assert.match(arena.say(s).status,/QOR ID/);
});

test('a match created waits for a challenger, can be closed, and plays once one joins',()=>{
 const s=arena.create('flux');arena.heard(s,lobby([]));
 assert.deepEqual(arena.newMatch(s),{ask:'create'});assert.equal(arena.say(s).status,'Opening your lobby…');
 const changed=arena.heard(s,{match:snapshot('flux',initialBoard('flux'),{status:'waiting',guest:null})});
 assert.equal(changed.took,true);assert.equal(arena.screen(s),'waiting');assert.equal(s.waiting,'');
 assert.match(arena.say(s).waiting,/Your lobby is open/);
 assert.deepEqual(arena.closeLobby(s),{ask:'cancel',id:ID,revision:1});
 const joined=arena.heard(s,{match:snapshot('flux',initialBoard('flux'),{revision:2})});
 assert.equal(joined.began,true);assert.equal(arena.screen(s),'match');assert.equal(arena.say(s).turn,'YOUR MOVE');
 assert.equal(arena.say(s).host,'sophia  (you)');assert.equal(arena.say(s).guest,'barbelo');
});

test('a move is the cursor\'s column at the board last seen, and the server\'s answer is the board drawn',()=>{
 const s=playing('flux');arena.press(s,'right');assert.equal(s.cursor,4);
 assert.deepEqual(arena.press(s,'choose'),{ask:'move',id:ID,cell:4,revision:2});
 assert.equal(arena.press(s,'choose'),null,'nothing more until it is answered');assert.equal(arena.say(s).turn,'SENDING YOUR MOVE…');
 const board=moveBoard('flux',initialBoard('flux'),4);
 const changed=arena.heard(s,{match:snapshot('flux',board,{revision:3,moves:1})});
 assert.equal(changed.moved,true);assert.equal(s.match.placed,39);assert.equal(s.match.cells[39],1);
 assert.equal(arena.say(s).turn,'THEIR MOVE');assert.equal(arena.press(s,'choose'),null,'not their move to make');
 // An older board, arriving late, is never drawn; nor is another match's.
 assert.equal(arena.heard(s,{match:snapshot('flux',initialBoard('flux'),{revision:2})}).took,false);
 assert.equal(arena.heard(s,{match:snapshot('flux',initialBoard('flux'),{id:OTHER,revision:9})}).took,false);
 assert.equal(s.match.revision,3);
});

test('1 to 7 drop straight into a column, and the cursor passes over a full one',()=>{
 let board=initialBoard('flux');for(const c of [0,1,0,1,0,1])board=moveBoard('flux',board,c);
 for(const c of [3,3,3,3,3,3]){board={...board,cells:board.cells.slice()};board.cells[board.cells.indexOf(0,0)]=0}
 const s=playing('flux');let full={...board,cells:board.cells.slice()};for(let r=0;r<6;r++)full.cells[r*7+2]=r%2+1;
 arena.heard(s,{match:snapshot('flux',full,{revision:3,legal:[0,1,3,4,5,6]})});
 s.cursor=1;arena.press(s,'right');assert.equal(s.cursor,3,'column 3 has no room');
 assert.equal(arena.press(s,'column3'),null,'a full column is no move');
 assert.deepEqual(arena.press(s,'column6'),{ask:'move',id:ID,cell:5,revision:3});
});

test('Rift Reversi\'s cursor moves a cell at a time, jumps between moves, and starts on one',()=>{
 const s=playing('reversi');const legal=legalMoves('reversi',initialBoard('reversi'));
 assert.deepEqual(legal,[8,13,22,27]);assert.equal(s.cursor,8,'the legal move nearest the middle');
 s.cursor=0;arena.press(s,'left');arena.press(s,'up');assert.equal(s.cursor,0);
 arena.press(s,'right');arena.press(s,'down');assert.equal(s.cursor,7);
 arena.press(s,'next');assert.equal(s.cursor,8);arena.press(s,'next');assert.equal(s.cursor,13);
 arena.press(s,'previous');assert.equal(s.cursor,8);arena.press(s,'previous');assert.equal(s.cursor,27);
 arena.point(s,22);assert.deepEqual(arena.press(s,'choose'),{ask:'move',id:ID,cell:22,revision:2});
 const board=moveBoard('reversi',initialBoard('reversi'),22);
 const changed=arena.heard(s,{match:snapshot('reversi',board,{revision:3,moves:1})});
 assert.equal(changed.moved,true);assert.deepEqual(s.match.flipped,[21]);
 assert.equal(arena.say(s).host,'sophia  (you)   4');assert.equal(arena.say(s).guest,'barbelo   1');
});

test('the pointer aims at a cell; Flux Four takes its column; its press is the logic\'s to read',()=>{
 const s=playing('flux');arena.point(s,40);assert.equal(s.cursor,5);arena.point(s,99);assert.equal(s.cursor,5);
 assert.equal(arena.press(s,'pick'),null);
});

test('conceding asks to be sure; Space that pressed a button is the button\'s',()=>{
 const s=playing('flux');
 assert.equal(arena.click(s,'Concede'),null);assert.equal(s.confirming,true);assert.equal(arena.press(s,'choose'),null);
 arena.click(s,'Keep playing');assert.equal(s.confirming,false);
 assert.equal(arena.press(s,'choose'),null,'the same tick as the click');
 arena.step(s);assert.deepEqual(arena.press(s,'choose'),{ask:'move',id:ID,cell:3,revision:2});
 arena.heard(s,{refused:'Wait for your turn.'});arena.click(s,'Concede');
 assert.deepEqual(arena.click(s,'Concede match'),{ask:'resign',id:ID,revision:2});
});

test('a refusal is a notice that fades, and asking may begin again',()=>{
 const s=arena.create('flux');arena.heard(s,lobby([]));arena.newMatch(s);
 arena.heard(s,{refused:'Too many open lobbies.'});
 assert.equal(s.waiting,'');assert.equal(arena.say(s).notice,'Too many open lobbies.');assert.equal(arena.screen(s),'lobby');
 for(let i=0;i<arena.NOTICE_STEPS-1;i++)arena.step(s);assert.equal(s.notice,'Too many open lobbies.');
 arena.step(s);assert.equal(s.notice,'');assert.deepEqual(arena.newMatch(s),{ask:'create'});
});

test('the clock counts down from the server\'s answer, a step at a time',()=>{
 const s=playing('reversi');arena.heard(s,{match:snapshot('reversi',initialBoard('reversi'),{revision:3,deadline:100500,serverNow:10000})});
 assert.equal(arena.secondsLeft(s),91);for(let i=0;i<30;i++)arena.step(s);assert.equal(arena.secondsLeft(s),90);
 for(let i=0;i<60*95;i++)arena.step(s);assert.equal(arena.secondsLeft(s),0);assert.equal(arena.say(s).clock,'0 s');
});

test('the match over says how, and the run\'s outcome is the server\'s verdict',()=>{
 let board=initialBoard('flux');for(const c of [0,1,0,1,0,1,0])board=moveBoard('flux',board,c);
 assert.equal(board.winner,1);
 const won=playing('flux');const done=arena.heard(won,{match:snapshot('flux',board,{status:'done',revision:9,moves:7})});
 assert.equal(done.ended,true);assert.equal(arena.screen(won),'result');assert.equal(arena.say(won).result,'VICTORY');
 assert.deepEqual(arena.outcome(won),{match:ID,seat:1,result:'won',moves:7});
 assert.deepEqual(arena.fourLine(board.cells,1),[14,21,28,35]);
 const lost=playing('flux',2);arena.heard(lost,{match:snapshot('flux',board,{status:'done',revision:9,moves:7,seat:2})});
 assert.equal(arena.say(lost).result,'MATCH COMPLETE');assert.equal(arena.say(lost).line,'Another world awaits.');
 assert.equal(arena.outcome(lost).result,'lost');
 const drawn=playing('reversi');arena.heard(drawn,{match:snapshot('reversi',{...initialBoard('reversi'),winner:0},{status:'done',revision:4})});
 assert.equal(arena.say(drawn).result,'DRAW');assert.equal(arena.say(drawn).line,'Balance restored.   2 to 2');
 assert.equal(arena.outcome(drawn).result,'drawn');
 const closed=following('flux');arena.heard(closed,{match:snapshot('flux',initialBoard('flux'),{status:'cancelled',revision:2,guest:null})});
 assert.equal(arena.screen(closed),'closed');assert.equal(arena.outcome(closed).result,'cancelled');
 assert.deepEqual(arena.click(won,'Rematch'),{ask:'create'});assert.equal(won.match,null);
});

test('leaving follows nothing, and a late answer for the old match is not drawn',()=>{
 const s=playing('flux');assert.deepEqual(arena.click(s,'Leave match'),{ask:'leave'});
 assert.equal(arena.screen(s),'lobby');
 assert.equal(arena.heard(s,{match:snapshot('flux',initialBoard('flux'),{revision:5})}).took,false);
 assert.deepEqual(arena.outcome(s),{match:'',seat:0,result:'',moves:0});
});

test('what is told is read, never trusted: a board of the wrong size or game is no match',()=>{
 const s=following('flux');
 for(const bad of [null,'x',{id:ID,game:'reversi',state:initialBoard('reversi')},{id:'../live',game:'flux',state:initialBoard('flux')},
  {id:ID,game:'flux',state:{cells:[1,2,3]}}])
  assert.equal(arena.heard(s,{match:bad}).took,false);
 const odd=arena.readMatch('flux',snapshot('flux',{...initialBoard('flux'),cells:Array(42).fill(7),placed:500,flipped:[3,'x',99]},{seat:5,status:'?'}));
 assert.equal(odd.cells.every(v=>v===0),true);assert.equal(odd.placed,-1);assert.deepEqual(odd.flipped,[3]);
 assert.equal(odd.seat,0);assert.equal(odd.status,'done');
 assert.throws(()=>arena.create('chess'));
});

test('a run replays: its clicks, then what the host told, then its presses, a tick at a time',()=>{
 const board=moveBoard('flux',initialBoard('flux'),3);
 const run={qqRun:1,seed:1,ticks:40,
  told:[[2,[lobby([])]],[6,[{match:snapshot('flux',initialBoard('flux'),{status:'waiting',guest:null})}]],
        [12,[{match:snapshot('flux',initialBoard('flux'),{revision:2})}]],[20,[{match:snapshot('flux',board,{revision:3,moves:1})}]],
        [30,[{match:snapshot('flux',board,{status:'done',revision:4,moves:1,state:{...board,winner:2}})}]]],
  clicks:[[4,['Create']]],edges:[[15,[['pressed','choose'],['released','choose']]]]};
 const {state,outcome}=arena.replay(run,'flux');
 assert.deepEqual(outcome,{match:ID,seat:1,result:'lost',moves:1});assert.equal(state.steps,40);
 assert.throws(()=>arena.replay({qqRun:2,ticks:1},'flux'));
});

const fixture=name=>JSON.parse(readFileSync(new URL(`./fixtures/${name}-run.json`,import.meta.url),'utf8'));
test('the runs QQ kept as ARQADE played the arenas replay here to the outcomes QQ reached, and an altered one does not',()=>{
 // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which plays both arenas in ARQADE's window against a
 // stand-in server and replays these same files with QQ's own replay.
 for(const [name,game,result] of [['flux-four','flux','won'],['rift-reversi','reversi','lost']]){
  const run=fixture(name);assert.equal(run.qqRun,1);assert.equal(run.scene,name);assert.equal(run.outcome.Arena.result,result);
  assert.deepEqual(arena.replay(run,game).outcome,run.outcome.Arena);
  // Without the server's last answer, the match is not over.
  assert.notDeepEqual(arena.replay({...run,told:run.told.slice(0,-1)},game).outcome,run.outcome.Arena);
 }
});
test('the arenas\' whole state at the end of each kept run is pinned, so no change to the conversation passes unseen',()=>{
 const digest=(name,game)=>createHash('sha256').update(JSON.stringify(arena.replay(fixture(name),game).state)).digest('hex');
 assert.equal(digest('flux-four','flux'),'46165b29772b8732c3b6b4d4c302618b2f8c8cb57e38cbb640afe85d5035fd1b');
 assert.equal(digest('rift-reversi','reversi'),'e17e9ae1646571c8f01ab1d4b5c1d9ed8f90c5b02f7ca57d2d0652d71fc8925b');
});
test('the conversation keeps no clock and draws no chance of its own: its steps are QQ\'s, its matches the server\'s',()=>{
 const source=readFileSync(new URL('../games/arenas/logic/arena.mjs',import.meta.url),'utf8').replace(/\/\*[\s\S]*?\*\//g,'').replace(/\/\/.*$/gm,'');
 assert.doesNotMatch(source,/Math\.(sin|cos|tan|asin|acos|atan2?|exp|log\w*|pow|sqrt|hypot|random)\b|\bDate\b|performance\./);
});
