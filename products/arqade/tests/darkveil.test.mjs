// SPDX-License-Identifier: Apache-2.0
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as R from '../games/darkveil/logic/rules.mjs';

// Darkveil's rules (QOR OS's QOR-0017): the same module the QQ game runs, here as ARQADE's service will run it to replay
// a run. A run QQ itself recorded is replayed below to the outcome QQ reached.
const {COLS, ROWS, TILE} = R;
const newPilot = () => (0, eval)(readFileSync(new URL('./darkveil-pilot.js', import.meta.url), 'utf8'));

const started = (seed, count) => { const g = R.create(seed, count); R.click(g, 'Play'); return g; };
const hero = (g, who = 0) => g.players[who];
const toward = (g, [dx, dy], into, who = 0) => {
  const p = hero(g, who), want = {right: dx > 0, left: dx < 0, down: dy > 0, up: dy < 0};
  for (const a of ['up', 'down', 'left', 'right'])
    if (want[a] !== p.held[a]) { (want[a] ? R.press : R.release)(g, a, who); into?.push([want[a] ? 'pressed' : 'released', a]); }
};
// Played by the pilot for up to `steps`; each step's presses and releases go to `into`, and `each` looks after each step.
const played = (g, steps, {pilot = newPilot(), into = () => {}, each = () => {}} = {}) => {
  for (let i = 0; i < steps && g.phase === 'playing'; i++) {
    const c = pilot(R, g, false), list = [];
    toward(g, c.way, list);
    if (c.attack !== hero(g).held.attack) { (c.attack ? R.press : R.release)(g, 'attack'); list.push([c.attack ? 'pressed' : 'released', 'attack']); }
    for (const a of ['roll', 'bolt', 'nova', 'potion', 'talk'])
      if (c[a]) { R.press(g, a); R.release(g, a); list.push(['pressed', a], ['released', a]); }
    if (c.say) { R.press(g, 'say' + c.say); R.release(g, 'say' + c.say); list.push(['pressed', 'say' + c.say], ['released', 'say' + c.say]); }
    into(list);
    R.step(g);
    each(g);
  }
  return g;
};
const steps = (g, n) => { for (let i = 0; i < n; i++) R.step(g); return g; };
// The valley with nothing in it but one enemy of `kind`, at `dx`, `dy` from the hero, awake if asked.
const alone = (seed, kind, dx, dy, awake = true) => {
  const g = started(seed), like = g.world.enemies[0], p = hero(g);
  g.world.enemies = [Object.assign({}, like, {id: 999, kind, x: p.x + dx, y: p.y + dy, homeX: p.x + dx, homeY: p.y + dy,
                                              health: R.KINDS[kind].health, most: R.KINDS[kind].health, wound: R.KINDS[kind].wound,
                                              awake, woke: awake, region: 'meadow', hold: 0})];
  return g;
};
const person = name => R.placeAt(R.PEOPLE[name].place);
const talkTo = (g, name, who = 0) => {
  const p = hero(g, who), at = person(name);
  p.x = at.x; p.y = at.y;
  R.press(g, 'talk', who);
  R.step(g);
  assert.equal(p.talk.who, name);
};
const sayId = (g, id, who = 0) => {
  const n = R.offers(g, who).findIndex(o => o.id === id);
  assert.ok(n >= 0, `${id} is offered`);
  R.press(g, 'say' + (n + 1), who);
  R.step(g);
};
const walkFrom = (g, [c, r]) => {
  const seen = new Set([r * COLS + c]), queue = [[c, r]];
  for (const [ac, ar] of queue)
    for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const key = (ar + dr) * COLS + ac + dc;
      if (R.walkable(g, ac + dc, ar + dr) && !seen.has(key)) { seen.add(key); queue.push([ac + dc, ar + dr]); }
    }
  return seen;
};
const atPlace = ([c, r]) => r * COLS + c;

test('a seed places the same enemies and plays the same run every time, and another seed others', () => {
  const run = seed => JSON.stringify(played(started(seed), 3000));
  assert.equal(run(4), run(4));
  assert.notEqual(run(4), run(5));
  assert.equal(R.create(9).phase, 'title');
  const g = R.create(9), before = JSON.stringify(g);
  steps(g, 100);
  assert.equal(JSON.stringify(g), before, 'nothing moves on the title');
});

test('the valley holds together: every place is walked to from Hearth, and the Crown only through its gate', () => {
  const g = started(1);
  assert.equal(R.WORLD.length, ROWS);
  assert.ok(R.WORLD.every(row => row.length === COLS));
  const shut = walkFrom(g, R.PLACES.hearth);
  for (const name of ['crossroads', 'den', 'mireBeacon', 'ashBeacon', 'highBeacon', 'gate'])
    assert.ok(shut.has(atPlace(R.PLACES[name])), name);
  assert.ok(!shut.has(atPlace(R.PLACES.crown)));
  for (const [c, r] of [...R.HERBS, ...R.ORE])
    assert.ok(shut.has(r * COLS + c), `what is to be found at ${c},${r}`);
  g.world.gate = true;
  assert.ok(walkFrom(g, R.PLACES.hearth).has(atPlace(R.PLACES.crown)));
  // Everything that waits for a hero stands on open ground, and can be walked to once the gate is open.
  const open = walkFrom(g, R.PLACES.hearth);
  for (const e of g.world.enemies)
    assert.ok(open.has(Math.floor(e.y / TILE) * COLS + Math.floor(e.x / TILE)), R.KINDS[e.kind].name);
});

test('the hero moves as the view faces: a whole step each way, a diagonal by the half root, and held while still', () => {
  const g = started(1), p = hero(g);
  g.world.enemies = [];
  const moved = (way, view) => {
    p.view = view;
    const x = p.x, y = p.y;
    toward(g, way);
    R.step(g);
    toward(g, [0, 0]);
    return [p.x - x, p.y - y];
  };
  const close = ([ax, ay], [bx, by]) => assert.ok(Math.abs(ax - bx) < 1e-9 && Math.abs(ay - by) < 1e-9, `${ax},${ay} is not ${bx},${by}`);
  const s = R.HERO_SPEED, h = s * Math.SQRT1_2;
  close(moved([0, -1], 0), [0, -s]);
  close(moved([1, 0], 0), [s, 0]);
  close(moved([1, -1], 0), [h, -h]);
  close(moved([0, -1], 2), [s, 0]);   // the view turned a quarter east: ahead is east
  close(moved([1, 0], 2), [0, s]);    // and its right is south
  close(moved([0, -1], 1), [h, -h]);  // an eighth: ahead is north-east
  p.view = 0;
  R.press(g, 'turnRight'); R.step(g);
  assert.equal(p.view, 1);
  R.press(g, 'turnLeft'); R.press(g, 'turnLeft'); R.step(g);
  assert.equal(p.view, 0, 'a press is kept once, however often it came before the step');
  // Nothing goes through what stands: it walks into a tree or a wall and stays out.
  p.view = 0;
  const [c, r] = [31, 35];
  p.x = (c + 0.5) * TILE; p.y = (r + 0.5) * TILE;
  toward(g, [0, 1]);
  for (let i = 0; i < 2000; i++) {
    R.step(g);
    assert.ok(R.walkable(g, Math.floor(p.x / TILE), Math.floor(p.y / TILE)));
  }
  toward(g, [0, 0]);
  // Mud slows it, and a pause holds it, a key held through the pause still counting after.
  const mire = R.WORLD.findIndex(row => row.includes(':::::::'));
  p.x = (R.WORLD[mire].indexOf(':::::::') + 3.5) * TILE; p.y = (mire + 0.5) * TILE;
  close(moved([1, 0], 0), [s * 0.85, 0]);
  R.press(g, 'pause');
  assert.equal(g.phase, 'paused');
  toward(g, [1, 0]);
  const x = p.x;
  steps(g, 20);
  assert.equal(p.x, x);
  R.click(g, 'Resume');
  R.step(g);
  assert.ok(p.x > x);
});

test('a swing strikes what is in its arc and reach, never what is behind; the third of a chain is the heavy one', () => {
  const g = alone(3, R.REVENANT, 40, 0), p = hero(g), e = g.world.enemies[0];
  e.kind = R.REVENANT;
  e.awake = false; e.woke = true; // it holds still
  e.act = 'recover'; e.timer = 1e9;
  p.facing = [1, 0];
  const most = e.health;
  R.press(g, 'attack'); R.release(g, 'attack');
  R.step(g);
  assert.deepEqual(p.facing, [1, 0]);
  assert.equal(p.swing, R.SWINGS[0].steps);
  while (p.swing > R.SWINGS[0].strikes + 1) { R.step(g); assert.equal(e.health, most, 'nothing lands while it winds up'); }
  R.step(g);
  assert.ok(e.health < most, 'it lands');
  // Behind the hero, it is untouched; and a swing turns only toward what is awake.
  const b = alone(3, R.REVENANT, -40, 0, false), q = hero(b), f = b.world.enemies[0];
  q.facing = [1, 0];
  R.press(b, 'attack'); R.release(b, 'attack');
  steps(b, R.SWINGS[0].steps);
  assert.equal(f.health, f.most);
  // Awake, the swing turns to it.
  const a = alone(3, R.REVENANT, -40, 0), r = hero(a);
  r.facing = [1, 0];
  R.press(a, 'attack'); R.release(a, 'attack');
  R.step(a);
  assert.deepEqual(r.facing, [-1, 0]);
  // Held, one swing follows another along the chain, the third the longer, and round again.
  const c = started(5), s = hero(c);
  c.world.enemies = [];
  R.press(c, 'attack');
  const chains = [];
  let last = 0;
  for (let i = 0; i < 400 && chains.length < 5; i++) {
    R.step(c);
    if (s.swing > last) chains.push([s.chain, s.swing]);
    last = s.swing;
  }
  R.release(c, 'attack');
  assert.deepEqual(chains.map(([n]) => n), [0, 1, 2, 0, 1]);
  // Pressed in time, the same; left to lapse, it starts over.
  const d = started(5), t = hero(d);
  d.world.enemies = [];
  const swingNow = () => { R.press(d, 'attack'); R.release(d, 'attack'); R.step(d); };
  swingNow();
  assert.equal(t.chain, 0);
  while (t.swing > 0) R.step(d);
  swingNow();
  assert.equal(t.chain, 1);
  while (t.swing > 0) R.step(d);
  swingNow();
  assert.equal(t.chain, 2);
  assert.equal(t.swing, R.SWINGS[2].steps);
  while (t.swing > 0) R.step(d);
  steps(d, R.CHAIN_STEPS + 1);
  swingNow();
  assert.equal(t.chain, 0, 'left to lapse, it starts over');
});

test('a roll crosses ground unwounded; a bolt costs mana, turns to its mark and strikes it; a nova waits for its level', () => {
  const g = alone(6, R.REVENANT, 30, 0), p = hero(g), e = g.world.enemies[0];
  e.act = 'windup'; e.timer = 6;
  p.facing = [-1, 0];
  const x = p.x;
  R.press(g, 'roll');
  const health = p.health;
  steps(g, R.ROLL_STEPS + 1);
  assert.equal(p.health, health, 'the blow lands on nothing');
  assert.ok(x - p.x > R.ROLL_SPEED * (R.ROLL_STEPS - 2));
  assert.equal(p.roll, 0);
  R.press(g, 'roll');
  R.step(g);
  assert.equal(p.roll, 0, 'not again until it has cooled');
  steps(g, R.ROLL_COOL);
  R.press(g, 'roll');
  R.step(g);
  assert.equal(p.roll, R.ROLL_STEPS);

  const b = alone(6, R.SHADE, -160, 0, false), q = hero(b), f = b.world.enemies[0];
  q.facing = [1, 0];
  f.health = f.most = 500;
  f.act = 'recover'; f.timer = 1e9;
  R.press(b, 'bolt');
  R.step(b);
  assert.equal(q.mana, R.MANA - R.BOLT_COST);
  assert.deepEqual(q.facing, [-1, 0]);
  for (let i = 0; i < 60 && f.health === 500; i++) R.step(b);
  assert.ok(f.health < 500, 'it flies and strikes');
  assert.equal(b.world.shots.length, 0);
  q.mana = R.BOLT_COST - 1;
  while (q.cast > 0) R.step(b);
  const stillMana = q.mana;
  R.press(b, 'bolt');
  R.step(b);
  assert.equal(q.cast, 0, 'not without the mana');
  assert.ok(q.mana >= stillMana);

  const n = alone(6, R.REVENANT, 60, 0, false), h = hero(n), o = n.world.enemies[0];
  o.health = o.most = 900;
  o.act = 'recover'; o.timer = 1e9;
  R.press(n, 'nova');
  R.step(n);
  assert.equal(h.cast, 0, 'not before its level');
  h.level = R.NOVA_LEVEL;
  R.press(n, 'nova');
  steps(n, R.NOVA_CAST);
  assert.ok(o.health < 900, 'it strikes all round');
  assert.equal(h.mana, R.MANA - R.NOVA_COST + Math.floor(R.NOVA_CAST / R.MANA_EVERY));
});

test('potions heal a share, one at a time; a wound holds off the next, and the ward takes its part', () => {
  const g = started(7), p = hero(g);
  g.world.enemies = [];
  R.press(g, 'potion');
  R.step(g);
  assert.equal(p.potions, R.POTIONS, 'not drunk while whole');
  p.health = 10;
  R.press(g, 'potion');
  R.step(g);
  assert.equal(p.health, 10 + Math.floor(R.HEALTH * R.POTION_HEALS / 100));
  assert.equal(p.potions, R.POTIONS - 1);
  R.press(g, 'potion');
  R.step(g);
  assert.equal(p.potions, R.POTIONS - 1, 'not again so soon');

  const w = alone(8, R.REVENANT, 30, 0), q = hero(w), e = w.world.enemies[0];
  q.ward = 2;
  e.act = 'windup'; e.timer = 1;
  const before = q.health;
  R.step(w);
  assert.equal(before - q.health, R.KINDS[R.REVENANT].wound - R.WARDS[2].ward);
  assert.equal(q.hurt, R.HURT_HOLD);
  e.act = 'windup'; e.timer = 1;
  const once = q.health;
  R.step(w);
  assert.equal(q.health, once, 'still reeling: no other wounds');
});

test('talking holds the valley; the people rest, sell and hand out errands, and nothing is sold for too little', () => {
  const g = started(10), p = hero(g), e = g.world.enemies[0];
  // Too far off, nothing happens.
  R.press(g, 'talk');
  R.step(g);
  assert.equal(p.talk, null);

  const x = e.x;
  e.awake = true;
  talkTo(g, 'healer');
  steps(g, 30);
  assert.equal(e.x, x, 'the valley holds while the only hero talks');
  assert.ok(p.talk.line.length > 0);
  assert.equal(R.offers(g).at(-1).id, 'leave');

  p.health = 1; p.mana = 1;
  sayId(g, 'rest');
  assert.equal(p.health, p.most);
  assert.equal(p.mana, p.manaMost);
  // A potion costs what it costs.
  p.gold = R.POTION_COST - 1;
  const potions = p.potions;
  assert.equal(R.offers(g).find(o => o.id === 'potion').able, false);
  sayId(g, 'potion');
  assert.equal(p.potions, potions);
  assert.equal(p.gold, R.POTION_COST - 1);
  p.gold = R.POTION_COST;
  sayId(g, 'potion');
  assert.equal(p.potions, potions + 1);
  assert.equal(p.gold, 0);
  // Up and down move the choice, the attack button says it, and the roll button walks away.
  const cursor = p.talk.cursor;
  R.press(g, 'down');
  R.step(g);
  assert.equal(p.talk.cursor, cursor + 1);
  R.press(g, 'up'); R.press(g, 'up');
  R.step(g);
  assert.equal(p.talk.cursor, cursor);
  R.press(g, 'attack'); R.release(g, 'attack');
  R.step(g);
  assert.equal(p.health, p.most);
  R.press(g, 'roll');
  R.step(g);
  assert.equal(p.talk, null);

  // The smith: the blade and the ward, in order, each for its price, and the blade's power counts.
  talkTo(g, 'smith');
  p.gold = R.BLADES[1].cost - 1;
  assert.equal(R.offers(g).find(o => o.id === 'blade').able, false);
  p.gold = R.BLADES[1].cost + R.WARDS[1].cost;
  const strength = R.power(p);
  sayId(g, 'blade');
  assert.equal(p.blade, 1);
  assert.equal(R.power(p), strength + R.BLADES[1].power);
  sayId(g, 'ward');
  assert.equal(p.ward, 1);
  assert.equal(p.gold, 0);
  sayId(g, 'leave');
  assert.equal(p.talk, null);
  // A screen button says it too.
  talkTo(g, 'elder');
  assert.equal(p.errands.king, 0);
  R.click(g, 'Say 1');
  R.step(g);
  assert.equal(p.errands.king, 1);
});

test('the errands: found things counted once, given over for their rewards; the wolf wants its death first', () => {
  const g = started(11), p = hero(g);
  g.world.enemies = [];
  talkTo(g, 'healer');
  sayId(g, 'herbs');
  sayId(g, 'leave');
  assert.equal(R.offers(g).length, 0);
  for (let i = 0; i < R.HERBS_WANTED; i++) {
    const [c, r] = R.HERBS[i];
    p.x = (c + 0.5) * TILE; p.y = (r + 0.5) * TILE;
    R.step(g);
    R.step(g);
    assert.equal(p.herbs.filter(x => x).length, i + 1);
  }
  talkTo(g, 'healer');
  const score = p.score, gold = p.gold, most = p.potionsMost;
  sayId(g, 'herbs done');
  assert.equal(p.errands.herbs, 2);
  assert.equal(p.score, score + 400);
  assert.equal(p.gold, gold + 100);
  assert.equal(p.potionsMost, most + 1);
  assert.ok(!R.offers(g).some(o => o.id.startsWith('herbs')));
  sayId(g, 'leave');

  talkTo(g, 'smith');
  sayId(g, 'ore');
  sayId(g, 'leave');
  for (let i = 0; i < R.ORE_WANTED; i++) {
    const [c, r] = R.ORE[i];
    p.x = (c + 0.5) * TILE; p.y = (r + 0.5) * TILE;
    R.step(g);
  }
  talkTo(g, 'smith');
  sayId(g, 'ore done');
  assert.equal(p.blade, 1);
  assert.equal(R.outcome(g).errands, 2);
  sayId(g, 'leave');

  talkTo(g, 'hunter');
  sayId(g, 'wolf');
  assert.equal(R.offers(g).find(o => o.id === 'wolf done').able, false);
  g.world.wolfDown = true;
  assert.equal(R.offers(g).find(o => o.id === 'wolf done').able, true);
  const whole = p.most;
  sayId(g, 'wolf done');
  assert.equal(p.level, 2, 'what it pays in experience is a level');
  assert.equal(p.most, whole + 20 + 12);
  assert.equal(R.outcome(g).errands, 3);
});

test('enemies sleep until they see a hero, wake their pack, wind up a blow that misses one who steps away, and go home', () => {
  const g = started(12), p = hero(g), like = g.world.enemies[0];
  const put = (id, x, y) => Object.assign({}, like, {id, kind: R.WOLF, x, y, homeX: x, homeY: y, health: 30, most: 30, wound: 7, awake: false, woke: false, region: 'meadow', hold: -1, act: 'move', timer: 0, cool: 0});
  // Hearth's clearing is open ground: well out of reach it sleeps, in reach it wakes, and its neighbour with it.
  const a = put(1, p.x + R.WAKE_REACH + 40, p.y), b = put(2, p.x + R.WAKE_REACH + 40 + R.PACK_REACH - 20, p.y);
  g.world.enemies = [a, b];
  steps(g, 20);
  assert.ok(!a.awake && !b.awake);
  a.x = a.homeX = p.x + R.WAKE_REACH - 20;
  b.x = b.homeX = a.x + R.PACK_REACH - 20;
  R.step(g);
  assert.ok(a.awake);
  assert.ok(b.awake, 'its neighbour woke with it');

  // It winds up in reach, and a hero who steps away is not there when it lands.
  const m = alone(12, R.REVENANT, 40, 0), q = hero(m), e = m.world.enemies[0];
  const reach = R.KINDS[R.REVENANT].reach + R.HERO_R + R.KINDS[R.REVENANT].r;
  assert.ok(40 < reach);
  R.step(m);
  assert.equal(e.act, 'windup');
  q.x -= 200;
  const health = q.health;
  steps(m, R.KINDS[R.REVENANT].windup);
  assert.equal(q.health, health);
  assert.equal(e.act, 'recover');

  // Led too far from home it goes back, whole, and sleeps again.
  const h = alone(13, R.WOLF, 40, 0), r = hero(h), w = h.world.enemies[0];
  w.health = 5;
  w.x = w.homeX + R.LEASH + 10;
  r.x = w.x + 5;
  R.step(h);
  assert.equal(w.act, 'home');
  r.x = hero(started(13)).x; // well away
  for (let i = 0; i < 2000 && w.awake; i++) R.step(h);
  assert.ok(!w.awake);
  assert.equal(w.health, w.most);
  assert.equal(w.x, w.homeX);

  // A shot from afar is stopped by what stands, and wounds a hero it reaches.
  const s = alone(14, R.THORNLING, 120, 0), t = hero(s), f = s.world.enemies[0];
  f.act = 'windup'; f.timer = 1; f.aimX = t.x; f.aimY = t.y;
  R.step(s);
  assert.equal(s.world.bolts.length, 1);
  const whole = t.health;
  for (let i = 0; i < 40 && t.health === whole; i++) R.step(s);
  assert.equal(whole - t.health, R.KINDS[R.THORNLING].wound);
  assert.equal(s.world.bolts.length, 0);
});

test('the fallen give experience, score and what they leave, on open ground, and each level brings health and power', () => {
  let items = 0;
  for (let seed = 21; seed < 29; seed++) {
    const g = started(seed), p = hero(g);
    for (const e of g.world.enemies) {
      if (e.kind <= R.GREAT_WOLF) { e.health = 0; e.by = 0; }
    }
    R.step(g);
    assert.ok(p.kills > 80);
    assert.ok(p.level > 1);
    assert.ok(p.power > R.POWER);
    assert.ok(p.most > R.HEALTH);
    assert.ok(g.world.wolfDown);
    assert.ok(g.world.items.length <= R.MOST_ITEMS);
    assert.ok(g.world.items.length > 0);
    items += g.world.items.length;
  }
  assert.ok(items > 100);
  // Whatever it leaves, a hero can reach, never inside a tree or a wall: great ones felled hard against what stands, each
  // of them leaving its six piles and a potion within 34 units.
  const g = started(29), like = g.world.enemies.find(e => e.kind === R.GREAT_WOLF);
  const against = [];
  for (let r = 1; r < ROWS - 1 && against.length < 40; r++)
    for (let c = 1; c < COLS - 1 && against.length < 40; c++)
      if (R.walkable(g, c, r) && !R.walkable(g, c + 1, r)) against.push([(c + 0.7) * TILE, (r + 0.5) * TILE]);
  assert.equal(against.length, 40);
  let lying = 0;
  for (const [x, y] of against) {
    g.world.items = [];
    g.world.enemies = [{...like, x, y, health: 0, by: 0}];
    R.step(g);
    assert.ok(g.world.items.length >= 6);
    for (const item of g.world.items) {
      const c = Math.floor(item.x / TILE), r = Math.floor(item.y / TILE);
      assert.ok(R.walkable(g, c, r), `${item.what} at ${c},${r} lies where nobody can reach it`);
      lying += 1;
    }
    g.phase = 'playing';
  }
  assert.ok(lying > 200);
  assert.equal(R.nextXp(1), 50);
  assert.equal(R.nextXp(4), 500);
});

test('a warden fallen lets its beacon be lit; all three open the gate; and the objective keeps up', () => {
  const g = started(30), p = hero(g);
  assert.match(R.objective(g).words, /Elder/);
  talkTo(g, 'elder');
  sayId(g, 'beacons');
  sayId(g, 'leave');
  assert.match(R.objective(g).words, /warden of the Mirewood/);
  g.world.enemies = g.world.enemies.filter(e => e.kind !== R.WOLF && e.kind !== R.SHADE && e.kind !== R.THORNLING && e.kind !== R.REVENANT && e.kind !== R.BONECASTER);
  const at = R.placeAt('mireBeacon');
  p.x = at.x; p.y = at.y;
  R.step(g);
  assert.equal(g.world.beacons[0], false, 'not while its warden stands');
  const warden = g.world.enemies.find(e => e.kind === R.WARDEN && e.hold === 0);
  warden.health = 0; warden.by = 0;
  R.step(g);
  assert.ok(g.world.wardens[0]);
  assert.match(R.objective(g).words, /Light the Mirewood/);
  R.step(g);
  assert.ok(g.world.beacons[0]);
  assert.match(R.objective(g).words, /Ashen Hold/);
  assert.equal(g.world.gate, false);
  for (const i of [1, 2]) {
    g.world.wardens[i] = true;
    g.world.enemies = g.world.enemies.filter(e => !(e.kind === R.WARDEN && e.hold === i));
    const b = R.placeAt(R.BEACONS[i].place);
    p.x = b.x; p.y = b.y;
    R.step(g);
    assert.ok(g.world.beacons[i]);
    assert.equal(g.world.gate, i === 2, 'the gate opens at the third beacon, and not before');
  }
  assert.match(R.objective(g).words, /Hollow King/);
  assert.equal(R.outcome(g).beacons, 3);
  // The king ended wins, once the valley has lit up, with a bonus for the time left.
  const king = g.world.enemies.find(e => e.kind === R.KING);
  king.health = 0; king.by = 0;
  R.step(g);
  assert.ok(g.world.kingDown);
  assert.equal(R.outcome(g).won, false);
  steps(g, R.CLEARING_STEPS);
  assert.equal(g.phase, 'over');
  assert.ok(R.outcome(g).won);
});

test('a hero who falls wakes in Hearth lighter of gold, what chased them goes home, and the third fall ends the run', () => {
  const g = alone(40, R.WOLF, 40, 0), p = hero(g), e = g.world.enemies[0];
  const hearth = R.placeAt('hearth');
  p.x += 400;
  e.x = p.x + 40;
  p.gold = 100;
  p.health = 0;
  R.step(g);
  assert.equal(p.falls, 1);
  assert.equal(p.gold, 75);
  assert.equal(p.health, p.most);
  assert.deepEqual([p.x, p.y], [hearth.x, hearth.y]);
  assert.equal(e.act, 'home');
  assert.equal(g.phase, 'playing');
  p.health = 0;
  R.step(g);
  assert.equal(g.phase, 'playing');
  p.health = 0;
  R.step(g);
  assert.equal(p.falls, R.LIVES);
  assert.equal(g.phase, 'over');
  assert.equal(R.outcome(g).won, false);
  assert.ok(R.outcome(g).over);
  // Play again starts afresh, from the same run of chance onward.
  R.click(g, 'Play again');
  assert.equal(g.phase, 'playing');
  assert.equal(hero(g).falls, 0);
  assert.ok(g.world.enemies.length > 80);
});

test('a hero who plays it through wins; one who goes straight for the wardens with a rusted blade falls', () => {
  for (const seed of [1, 2, 3]) {
    const g = played(started(seed), R.RUN_STEPS);
    const o = R.outcome(g);
    assert.ok(o.won, `seed ${seed}: ${JSON.stringify(o)}`);
    assert.equal(o.beacons, 3);
    assert.ok(o.errands >= 2);
    assert.ok(o.level >= 8);
    assert.ok(o.kills > 100);
    assert.ok(o.score > 20000);
  }
  const g = started(4), p = hero(g), pilot = newPilot();
  for (let i = 0; i < R.RUN_STEPS && g.phase === 'playing'; i++) {
    const warden = g.world.enemies.find(e => e.kind === R.WARDEN);
    if (!warden) break;
    // The pilot's own way there, but nothing bought, nothing drunk, nothing dodged: only swings.
    const near = Math.hypot(warden.x - p.x, warden.y - p.y) < 90;
    if (near !== p.held.attack) (near ? R.press : R.release)(g, 'attack');
    toward(g, near ? [0, 0] : pilot(R, g, 'calm-to-warden').way);
    R.step(g);
  }
  assert.equal(p.falls, R.LIVES);
  assert.equal(R.outcome(g).won, false);
});

test('a still hero is never found in Hearth, and the veil closes on them', () => {
  const g = steps(started(2), R.RUN_STEPS + 10);
  assert.equal(g.steps, R.RUN_STEPS);
  assert.deepEqual(R.outcome(g), {score: 0, kills: 0, gold: 0, level: 1, health: R.HEALTH, falls: 0, beacons: 0, errands: 0, seconds: 2700, won: false, over: true});
});

test('the world is one and each player their own: two heroes step in one valley by their own inputs', () => {
  const g = started(50, 2), a = hero(g, 0), b = hero(g, 1);
  assert.equal(g.players.length, 2);
  g.world.enemies = g.world.enemies.filter(e => e.kind === R.WARDEN);
  // Each moves by their own keys.
  const ax = a.x, bx = b.x;
  R.press(g, 'right', 1);
  steps(g, 5);
  assert.equal(a.x, ax);
  assert.ok(Math.abs(b.x - (bx + 5 * R.HERO_SPEED)) < 1e-9);
  R.release(g, 'right', 1);
  // One talking does not hold the valley; both do.
  const warden = g.world.enemies[0];
  warden.awake = true; warden.woke = true;
  a.x = warden.x; a.y = warden.y + 200;
  talkTo(g, 'elder', 1);
  assert.equal(a.talk, null);
  const y = warden.y;
  steps(g, 5);
  assert.notEqual(warden.y, y, 'the warden comes on');
  // A wound lands on the one it was meant for.
  warden.x = a.x; warden.y = a.y; warden.act = 'windup'; warden.timer = 1;
  const health = a.health;
  R.step(g);
  assert.ok(a.health < health);
  assert.equal(b.health, R.HEALTH);
  // Nothing wounds a hero while they talk.
  const talker = b.health, [homeX, homeY] = [warden.homeX, warden.homeY];
  a.x = b.x + 2000; a.y = b.y;
  warden.x = warden.homeX = b.x; warden.y = warden.homeY = b.y; warden.act = 'windup'; warden.timer = 1;
  R.step(g);
  assert.equal(warden.act, 'recover', 'its blow has landed');
  assert.equal(b.health, talker);
  assert.ok(b.talk);
  a.x = homeX; a.y = homeY;
  warden.homeX = homeX; warden.homeY = homeY; warden.x = a.x; warden.y = a.y + 200; warden.act = 'move'; warden.timer = 0;
  talkTo(g, 'elder', 0);
  const y2 = warden.y;
  steps(g, 5);
  assert.equal(warden.y, y2, 'both talking: the valley holds');
  sayId(g, 'beacons', 0);
  assert.equal(a.errands.king, 1);
  assert.equal(b.errands.king, 0, 'errands are their own');
  sayId(g, 'leave', 0);
  sayId(g, 'leave', 1);
  // A beacon lit by one is lit for both, and scores for the one who lit it.
  g.world.enemies = g.world.enemies.filter(e => e.hold !== 0);
  g.world.wardens[0] = true;
  const at = R.placeAt('mireBeacon');
  b.x = at.x; b.y = at.y;
  const score = a.score;
  R.step(g);
  assert.ok(g.world.beacons[0]);
  assert.equal(R.outcome(g, 1).beacons, 1);
  assert.equal(R.outcome(g, 0).beacons, 1);
  assert.equal(a.score, score);
  assert.equal(b.score, 800);
  // One out does not end the run while the other stands.
  a.falls = R.LIVES - 1; a.health = 0;
  R.step(g);
  assert.ok(a.out);
  assert.equal(g.phase, 'playing');
  b.falls = R.LIVES - 1; b.health = 0;
  R.step(g);
  assert.equal(g.phase, 'over');
});

test('a run replays to the outcome of the same events played live', () => {
  const live = R.create(99), edges = [], clicks = [[3, ['Play']]], pilot = newPilot();
  let tick = 0;
  for (; tick < 12000 && live.phase !== 'over'; tick++) {
    if (tick === 3) R.click(live, 'Play');
    if (live.phase === 'playing') played(live, 1, {pilot, into: list => { if (list.length) edges.push([tick, list]); }});
    else R.step(live);
  }
  const run = {qqRun: 1, seed: 99, ticks: tick, edges, clicks};
  assert.deepEqual(R.replay(run), R.outcome(live));
  assert.ok(R.outcome(live).kills > 20);
  assert.throws(() => R.replay({qqRun: 2, seed: 1, ticks: 1}));
  assert.throws(() => R.replay(null));
});

const fixture = new URL('./fixtures/darkveil-run.json', import.meta.url);
test('the run QQ kept as ARQADE played it replays here to the outcome QQ reached, and an altered one does not', () => {
  // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which plays Darkveil in ARQADE's window to its first beacon
  // and on to a third fall, and replays this same file with QQ's own replay.
  const run = JSON.parse(readFileSync(fixture, 'utf8'));
  assert.equal(run.qqRun, 1); assert.equal(run.scene, 'darkveil');
  assert.deepEqual(run.outcome['Darkveil'], {beacons: 1, errands: 0, falls: 3, gold: 130, health: 0, kills: 34, level: 4, over: true, score: 2480, seconds: 217, won: false});
  assert.deepEqual(R.replay(run), run.outcome['Darkveil']);
  // Another seed, or the same run never swinging, is another run with another outcome.
  assert.notDeepEqual(R.replay({...run, seed: run.seed + 1}), run.outcome['Darkveil']);
  const calm = run.edges.map(([tick, list]) => [tick, list.filter(([, action]) => action !== 'attack')]);
  assert.notDeepEqual(R.replay({...run, edges: calm}), run.outcome['Darkveil']);
});

test('the rules\' whole state at the end of the kept run is pinned, so no change to the rules passes unseen', () => {
  const run = JSON.parse(readFileSync(fixture, 'utf8'));
  const edges = new Map(run.edges.map(([t, l]) => [t, l])), clicks = new Map(run.clicks.map(([t, l]) => [t, l])), g = R.create(run.seed);
  for (let t = 0; t < run.ticks; t++) {
    for (const b of clicks.get(t) || []) R.click(g, b);
    for (const [w, a] of edges.get(t) || []) (w === 'pressed' ? R.press : R.release)(g, a);
    R.step(g);
  }
  assert.equal(createHash('sha256').update(JSON.stringify(g)).digest('hex'), 'ee421102b714e9694058fcb42c11a7fb7b5882808c956c6039dba32fb2298348');
});

test('the rules keep to arithmetic every machine computes alike: a square root, but no sine, cosine, power or logarithm', () => {
  const source = readFileSync(new URL('../games/darkveil/logic/rules.mjs', import.meta.url), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
  assert.doesNotMatch(source, /Math\.(sin|cos|tan|asin|acos|atan2?|sinh|cosh|tanh|exp|expm1|log\w*|pow|cbrt|hypot|random)\b/);
  assert.doesNotMatch(source, /\*\*/);
  assert.doesNotMatch(source, /\b(Date|performance)\b/);
});
