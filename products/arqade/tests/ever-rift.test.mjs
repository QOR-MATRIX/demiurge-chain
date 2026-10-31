// SPDX-License-Identifier: Apache-2.0
import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import * as everRift from '../games/ever-rift/logic/rules.mjs';

// Ever Rift's rules (QOR OS's QOR-0017): the same module the QQ game runs, here as ARQADE's service will run it to
// replay a run. A run QQ itself recorded is replayed below to the outcome QQ reached.
const {COLS, ROWS, TILE} = everRift;
const DIRS = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1]];
const apart = (a, b) => Math.sqrt((a.x - b.x) ** 2 + (a.y - b.y) ** 2);

// How far each floor tile is from a tile, walking: what the pilot below finds its way by.
const walking = (g, tc, tr) => {
  const far = new Array(COLS * ROWS).fill(-1);
  if (everRift.rock(g, tc, tr)) return far;
  const queue = [tr * COLS + tc];
  far[queue[0]] = 0;
  for (let head = 0; head < queue.length; head++) {
    const at = queue[head], c = at % COLS, r = (at - c) / COLS;
    for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const nc = c + dc, nr = r + dr;
      if (!everRift.rock(g, nc, nr) && far[nr * COLS + nc] < 0) {
        far[nr * COLS + nc] = far[at] + 1;
        queue.push(nr * COLS + nc);
      }
    }
  }
  return far;
};
let ways = {key: null, far: null};
// The middle of the next tile on the way to (x, y).
const nextTo = (g, x, y) => {
  const tc = Math.floor(x / TILE), tr = Math.floor(y / TILE), key = `${g.seed}:${g.floor}:${tc},${tr}`;
  if (ways.key !== key) ways = {key, far: walking(g, tc, tr)};
  const c = Math.floor(g.x / TILE), r = Math.floor(g.y / TILE), here = ways.far[r * COLS + c];
  if (here <= 1) return [x, y];
  let best = here, to = [x, y];
  for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
    const v = ways.far[(r + dr) * COLS + c + dc];
    if (!everRift.rock(g, c + dc, r + dr) && v >= 0 && v < best) {
      best = v;
      to = [(c + dc + 0.5) * TILE, (r + dr + 0.5) * TILE];
    }
  }
  return to;
};
const wayTo = (g, x, y) => {
  const dx = x - g.x, dy = y - g.y;
  if (dx * dx + dy * dy < 9) return [0, 0];
  let best = null, score = -Infinity;
  for (const [a, b] of DIRS) {
    const s = (a * dx + b * dy) * (a && b ? 0.7071 : 1);
    if (s > score) { score = s; best = [a, b]; }
  }
  return best;
};
// A delver as a person plays one: a potion when wounded, a whirl among three, a dash away from the warden's slam, a
// swing at what is near, and otherwise toward the nearest enemy awake, what lies on the floor once none is, or the
// stairs once strong enough for the floor below. With `skills` false it only swings.
const pilot = (g, skills = true) => {
  const out = {way: [0, 0], attack: false, whirl: false, dash: false, potion: false};
  const awake = g.enemies.filter(e => e.awake);
  const close = awake.filter(e => apart(e, g) < everRift.SWING_REACH + everRift.KINDS[e.kind].r - 4);
  if (skills) {
    out.potion = g.health < g.most * 0.45 && g.potions > 0 && g.potionCool === 0;
    out.whirl = close.length >= 3 && g.fury >= everRift.WHIRL_COST;
    const slam = awake.find(e => e.kind === everRift.WARDEN && e.act === 'windup' && apart(e, g) < 130);
    if (slam && g.dashCool === 0) {
      out.dash = true;
      out.way = wayTo(g, 2 * g.x - slam.x, 2 * g.y - slam.y);
      return out;
    }
  }
  if (close.length) { out.attack = true; return out; }
  const nearest = list => list.reduce((a, b) => apart(a, g) < apart(b, g) ? a : b);
  const items = g.items.filter(i => (i.what !== 'potion' || g.potions < everRift.POTIONS_MOST) && apart(i, g) < 220);
  let goal = null;
  if (items.length && !awake.length) goal = nearest(items);
  else if (awake.length) goal = nearest(awake);
  else if (g.enemies.length && (!g.stairs || g.level < g.floor + 1)) goal = nearest(g.enemies);
  else if (g.stairs) goal = g.stairs;
  if (goal) out.way = wayTo(g, ...nextTo(g, goal.x, goal.y));
  return out;
};
const hold = (g, [dx, dy], into) => {
  const want = {right: dx > 0, left: dx < 0, down: dy > 0, up: dy < 0};
  for (const a of ['up', 'down', 'left', 'right'])
    if (want[a] !== g.held[a]) { (want[a] ? everRift.press : everRift.release)(g, a); into?.push([want[a] ? 'pressed' : 'released', a]); }
};
// Played for up to `steps`, by the pilot or by `way` alone; each step's presses and releases go to `into`.
const play = (g, steps, {skills = true, way = null, each = () => {}, into = null} = {}) => {
  for (let i = 0; i < steps && g.phase === 'playing'; i++) {
    const p = way ? {way: way(g), attack: false} : pilot(g, skills), list = [];
    hold(g, p.way, list);
    if (p.attack !== g.held.attack) { (p.attack ? everRift.press : everRift.release)(g, 'attack'); list.push([p.attack ? 'pressed' : 'released', 'attack']); }
    for (const a of ['whirl', 'dash', 'potion'])
      if (p[a]) { everRift.press(g, a); everRift.release(g, a); list.push(['pressed', a], ['released', a]); }
    into?.(list);
    everRift.step(g);
    each(g);
  }
  return g;
};
const started = seed => { const g = everRift.create(seed); everRift.click(g, 'Play'); return g; };
// A floor with nothing on it but the delver and one enemy of `kind`, `away` units to the delver's right, in its room
// (each room is five tiles wide or more, and the delver starts in the middle of the first).
const alone = (seed, kind, away) => {
  const g = started(seed);
  g.enemies = [];
  everRift.step(g);
  assert.ok(!everRift.rock(g, Math.floor((g.x + away) / TILE), Math.floor(g.y / TILE)));
  g.enemies = [{id: 999, kind, x: g.x + away, y: g.y, health: everRift.KINDS[kind].health, most: everRift.KINDS[kind].health,
                awake: true, woke: true, act: 'move', timer: 0, cool: 0, fx: -1, fy: 0, kx: 0, ky: 0, struck: 0,
                aimX: 0, aimY: 0}];
  return g;
};

test('a seed carves the same floors and plays the same run every time, and another seed others', () => {
  const run = seed => { const g = started(seed); play(g, 2400); return JSON.stringify(everRift.outcome(g)) + g.x + ',' + g.y + g.map.join(''); };
  assert.equal(run(7), run(7));
  assert.notEqual(run(7), run(8));
  assert.notEqual(everRift.create(7).map.join(''), everRift.create(8).map.join(''));
});

test('every floor is whole: rooms apart, every tile reached from the way in, stairs in the furthest room, the warden on the last', () => {
  for (let seed = 1; seed <= 20; seed++) {
    const g = everRift.create(seed);
    for (const floor of [1, 2, 3]) {
      assert.ok(g.rooms.length >= 5, `seed ${seed} floor ${floor}: ${g.rooms.length} rooms`);
      const from = walking(g, Math.floor(g.x / TILE), Math.floor(g.y / TILE));
      const tiles = g.map.filter(t => t === 1).length, reached = from.filter(d => d >= 0).length;
      assert.equal(reached, tiles, `seed ${seed} floor ${floor}`);
      for (let c = 0; c < COLS; c++) assert.ok(everRift.rock(g, c, 0) && everRift.rock(g, c, ROWS - 1));
      const room = g.rooms[g.last], end = {x: (room.c + room.w / 2) * TILE, y: (room.r + room.h / 2) * TILE};
      const steps = r => from[Math.floor((r.r + r.h / 2)) * COLS + Math.floor(r.c + r.w / 2)];
      assert.ok(g.rooms.every(r => steps(r) <= steps(room)));
      if (floor < 3) {
        assert.deepEqual(g.stairs, end);
        assert.ok(!g.enemies.some(e => e.kind === everRift.WARDEN));
        g.x = g.stairs.x; g.y = g.stairs.y; g.phase = 'playing';
        everRift.step(g);
        assert.equal(g.floor, floor + 1);
        assert.ok(g.said.some(s => s.what === 'descend' && s.floor === floor + 1));
      } else {
        assert.equal(g.stairs, null);
        const warden = g.enemies.filter(e => e.kind === everRift.WARDEN);
        assert.equal(warden.length, 1);
        assert.deepEqual([warden[0].x, warden[0].y], [end.x, end.y]);
        assert.equal(warden[0].health, everRift.scaled(450, 3, 40));
      }
    }
  }
});

test('the delver moves three a step, a diagonal by the half root, slips round a corner, and holds still on the title and paused', () => {
  const g = everRift.create(5), x = g.x, y = g.y;
  everRift.press(g, 'right'); everRift.step(g);
  assert.equal(g.x, x); assert.equal(g.steps, 0);
  everRift.click(g, 'Play'); everRift.step(g);
  assert.equal(g.x, x + 3);
  everRift.press(g, 'down'); everRift.step(g);
  assert.equal(g.x, x + 3 + 3 * 0.7071067811865476); assert.equal(g.y, y + 3 * 0.7071067811865476);
  everRift.press(g, 'pause'); everRift.step(g); assert.equal(g.steps, 2);
  everRift.press(g, 'pause'); everRift.step(g); assert.equal(g.steps, 3);
  everRift.release(g, 'down');
  for (let i = 0; i < 400; i++) everRift.step(g);
  const right = Math.floor((g.x + everRift.HERO_R) / TILE);
  assert.ok(everRift.rock(g, right + 1, Math.floor(g.y / TILE)) || everRift.rock(g, right, Math.floor(g.y / TILE)), 'stopped at a wall');
  // Six units off a passage's line, a step along it slips round the corner and through.
  const h = started(7);
  h.enemies = [];
  h.x = 412; h.y = 368;
  assert.ok(everRift.rock(h, 9, 8) && !everRift.rock(h, 9, 9) && !everRift.rock(h, 9, 10), 'the passage seed 7 starts beside');
  everRift.press(h, 'left');
  for (let i = 0; i < 30; i++) everRift.step(h);
  assert.ok(h.x < 340, `went only to ${h.x}, ${h.y}`);
});

test('a swing strikes what is in its arc and reach, never what is behind, gains fury, and swings again while held', () => {
  const g = alone(3, everRift.HUSK, 40);
  g.enemies.push({...g.enemies[0], id: 998, x: g.x - 40});
  everRift.press(g, 'attack');
  everRift.step(g);
  assert.equal(g.swing, everRift.SWING_STEPS);
  assert.deepEqual(g.facing, [1, 0]);
  for (let i = 0; i < everRift.SWING_STEPS - everRift.SWING_STRIKES; i++) everRift.step(g);
  const [ahead, behind] = [g.enemies.find(e => e.id === 999), g.enemies.find(e => e.id === 998)];
  assert.ok(ahead.health < ahead.most, 'the husk ahead was struck');
  assert.equal(behind.health, behind.most);
  assert.equal(g.fury, everRift.FURY_HIT);
  assert.ok(g.said.some(s => s.what === 'hit' || s.what === 'crit'));
  for (let i = 0; i < everRift.SWING_STRIKES; i++) everRift.step(g);
  assert.equal(g.swing, everRift.SWING_STEPS, 'held, it swings again');
  everRift.release(g, 'attack');
  for (let i = 0; i < everRift.SWING_STEPS + 1; i++) everRift.step(g);
  assert.equal(g.swing, 0);
  // Out of reach, a swing strikes nothing.
  const far = alone(3, everRift.BRUTE, 90);
  everRift.press(far, 'attack'); everRift.release(far, 'attack');
  for (let i = 0; i < 7; i++) everRift.step(far);
  assert.equal(far.enemies[0].health, far.enemies[0].most);
});

test('a whirl costs fury and strikes everything round the delver three times; a dash crosses ninety units unwounded', () => {
  const g = alone(4, everRift.BRUTE, 50);
  g.enemies.push({...g.enemies[0], id: 998, x: g.x - 50}, {...g.enemies[0], id: 997, x: g.x, y: g.y + 50});
  everRift.press(g, 'whirl'); everRift.step(g);
  assert.equal(g.whirl, 0, 'no fury, no whirl');
  g.fury = 40;
  everRift.press(g, 'whirl'); everRift.step(g);
  assert.equal(g.whirl, everRift.WHIRL_STEPS); assert.equal(g.fury, 5);
  let blows = 0;
  for (let i = 0; i < everRift.WHIRL_STEPS; i++) { everRift.step(g); blows += g.said.filter(s => s.what === 'hit' || s.what === 'crit').length; }
  assert.equal(blows, 9);
  // A dash goes where the delver faces, nine units a step for ten steps, the first the step it is pressed; a blow
  // landing meanwhile does not wound.
  const h = alone(4, everRift.HUSK, 40), x = h.x;
  h.facing = [-1, 0];
  everRift.press(h, 'dash');
  everRift.step(h);
  assert.equal(h.dash, everRift.DASH_STEPS);
  for (let i = 1; i < everRift.DASH_STEPS; i++) everRift.step(h);
  assert.equal(h.dash, 1);
  everRift.step(h);
  assert.equal(h.dash, 0);
  assert.ok(x - h.x <= everRift.DASH_STEPS * everRift.DASH_SPEED + 0.001 && x - h.x > 40, `dashed ${h.x - x}`);
  const wounded = alone(4, everRift.HUSK, 40);
  wounded.enemies[0].act = 'windup';
  wounded.enemies[0].timer = 3;
  everRift.press(wounded, 'dash'); everRift.release(wounded, 'dash');
  wounded.facing = [0, 0];
  for (let i = 0; i < 4; i++) everRift.step(wounded);
  assert.equal(wounded.health, everRift.HEALTH, 'unwounded while it dashes');
  everRift.press(h, 'dash'); everRift.step(h);
  assert.equal(h.dash, 0, 'not again before it cools');
});

test('a potion heals nearly half, one at a time, and five at most are carried', () => {
  const g = alone(6, everRift.HUSK, 600);
  g.health = 20;
  everRift.press(g, 'potion'); everRift.step(g);
  assert.equal(g.health, 20 + 45); assert.equal(g.potions, 1);
  everRift.press(g, 'potion'); everRift.step(g);
  assert.equal(g.potions, 1, 'not again before it cools');
  g.potions = 5;
  g.items.push({id: 1, what: 'potion', x: g.x, y: g.y});
  everRift.step(g);
  assert.equal(g.potions, 5); assert.equal(g.items.length, 1, 'left lying');
  g.potions = 4;
  everRift.step(g);
  assert.equal(g.potions, 5); assert.equal(g.items.length, 0);
});

test('enemies sleep until they see the delver, wake their pack, wind up a blow, and miss a delver who steps away', () => {
  const g = started(9);
  assert.ok(g.enemies.every(e => !e.awake));
  everRift.step(g);
  assert.ok(g.enemies.every(e => !e.awake), 'nothing in the first room');
  // On a floor opened wide: one in sight within 260 wakes, and wakes one beyond sight within 170 of it, but not one
  // further.
  const open = started(9);
  open.map = open.map.map((t, i) => (i % COLS > 0 && i % COLS < COLS - 1 && i >= COLS && i < COLS * (ROWS - 1) ? 1 : 0));
  open.pathsFrom = -1;
  open.x = 400; open.y = 400;
  const sleeper = (id, x) => ({id, kind: everRift.HUSK, x, y: 400, health: 28, most: 28, awake: false, woke: false,
                               act: 'move', timer: 0, cool: 0, fx: 0, fy: 1, kx: 0, ky: 0, struck: 0, aimX: x, aimY: 400});
  open.enemies = [sleeper(1, 650), sleeper(2, 800), sleeper(3, 1100)];
  everRift.step(open);
  assert.deepEqual(open.enemies.map(e => e.awake), [true, true, false]);
  assert.ok(open.said.some(s => s.what === 'wake' && s.kind === everRift.HUSK));
  // A husk beside the delver winds up for 22 steps, and its blow lands on a delver still in reach.
  const h = alone(9, everRift.HUSK, 40);
  everRift.step(h);
  assert.equal(h.enemies[0].act, 'windup');
  for (let i = 0; i < everRift.KINDS[everRift.HUSK].windup - 1; i++) { everRift.step(h); assert.equal(h.health, everRift.HEALTH); }
  everRift.step(h);
  assert.equal(h.health, everRift.HEALTH - everRift.KINDS[everRift.HUSK].wound);
  assert.equal(h.hurt, everRift.HURT_HOLD);
  // The same blow, the delver stepping back as it winds up, misses.
  const m = alone(9, everRift.HUSK, 40);
  everRift.step(m);
  everRift.press(m, 'left');
  for (let i = 0; i < everRift.KINDS[everRift.HUSK].windup; i++) everRift.step(m);
  assert.equal(m.health, everRift.HEALTH);
  // A seer looses a bolt from afar, which wounds where it strikes.
  const s = alone(10, everRift.SEER, 90);
  let bolt = false;
  for (let i = 0; i < 120 && s.health === everRift.HEALTH; i++) { everRift.step(s); bolt = bolt || s.said.some(x => x.what === 'bolt'); }
  assert.ok(bolt);
  assert.equal(s.health, everRift.HEALTH - everRift.KINDS[everRift.SEER].wound);
});

test('the fallen give experience, score and what they leave, and each level brings health and power', () => {
  assert.deepEqual([1, 2, 3, 4].map(everRift.nextXp), [50, 150, 300, 500]);
  const g = started(12);
  let levels = 0, picked = 0;
  play(g, 3000, {each: x => { levels += x.said.filter(s => s.what === 'level').length; picked += x.said.filter(s => s.what === 'pick').length; }});
  assert.ok(g.kills > 10); assert.ok(levels >= 1); assert.ok(picked > 0);
  assert.equal(g.most, everRift.HEALTH + 12 * (g.level - 1) + 20 * g.relics[1]);
  assert.equal(g.power, everRift.POWER + 2 * (g.level - 1) + 3 * g.relics[0]);
  assert.ok(g.score >= g.gold + g.kills * 15);
});

test('the warden broken clears the rift: a delver who uses its skills clears it, one who only swings falls', () => {
  let cleared = 0, fell = 0;
  for (let seed = 1; seed <= 6; seed++) {
    const g = started(seed);
    let down = -1;
    play(g, everRift.RUN_STEPS, {each: x => { if (x.said.some(s => s.what === 'warden down')) down = x.steps; }});
    const o = everRift.outcome(g);
    if (o.cleared) {
      cleared++;
      assert.equal(g.steps - down, everRift.CLEARING_STEPS);
      assert.equal(o.floor, 3);
      assert.ok(o.score > 2500 + 600);
    }
    const s = play(started(seed), everRift.RUN_STEPS, {skills: false});
    if (everRift.outcome(s).health === 0) fell++;
  }
  assert.ok(cleared >= 5, `${cleared} of 6 cleared`);
  assert.equal(fell, 6);
});

test('a delver who never moves is never found, and the rift closes on it after ten minutes', () => {
  const g = play(started(2), everRift.RUN_STEPS + 10, {way: () => [0, 0]});
  assert.equal(g.steps, everRift.RUN_STEPS);
  assert.deepEqual(everRift.outcome(g), {score: 0, kills: 0, gold: 0, level: 1, health: everRift.HEALTH, floor: 1, seconds: 600, cleared: false, over: true});
});

test('a run replays to the outcome of the same events played live', () => {
  const live = everRift.create(99), edges = [], clicks = [[3, ['Play']]];
  let tick = 0;
  for (; tick < 20000 && live.phase !== 'over'; tick++) {
    if (tick === 3) everRift.click(live, 'Play');
    if (live.phase === 'playing') play(live, 1, {into: list => { if (list.length) edges.push([tick, list]); }});
    else everRift.step(live);
  }
  const run = {qqRun: 1, seed: 99, ticks: tick, edges, clicks};
  assert.deepEqual(everRift.replay(run), everRift.outcome(live));
  assert.ok(everRift.outcome(live).kills > 20);
  assert.throws(() => everRift.replay({qqRun: 2, seed: 1, ticks: 1}));
  assert.throws(() => everRift.replay(null));
});

const fixture = new URL('./fixtures/ever-rift-run.json', import.meta.url);
test('the run QQ kept as ARQADE played it replays here to the outcome QQ reached, and an altered one does not', () => {
  // Kept by the Qt app's games suite (qt/tests/tst_games.cpp), which plays Ever Rift in ARQADE's window through its
  // first floor and to a fall on the second, and replays this same file with QQ's own replay.
  const run = JSON.parse(readFileSync(fixture, 'utf8'));
  assert.equal(run.qqRun, 1); assert.equal(run.scene, 'ever-rift');
  assert.deepEqual(run.outcome['Ever Rift'], {cleared: false, floor: 2, gold: 77, health: 0, kills: 19, level: 2, over: true, score: 722, seconds: 40});
  assert.deepEqual(everRift.replay(run), run.outcome['Ever Rift']);
  // Another seed, or the same run never swinging, is another run with another outcome.
  assert.notDeepEqual(everRift.replay({...run, seed: run.seed + 1}), run.outcome['Ever Rift']);
  const calm = run.edges.map(([tick, list]) => [tick, list.filter(([, action]) => action !== 'attack')]);
  assert.notDeepEqual(everRift.replay({...run, edges: calm}), run.outcome['Ever Rift']);
});

test('the rules\' whole state at the end of the kept run is pinned, so no change to the rules passes unseen', () => {
  const run = JSON.parse(readFileSync(fixture, 'utf8'));
  const edges = new Map(run.edges.map(([t, l]) => [t, l])), clicks = new Map(run.clicks.map(([t, l]) => [t, l])), g = everRift.create(run.seed);
  for (let t = 0; t < run.ticks; t++) {
    for (const b of clicks.get(t) || []) everRift.click(g, b);
    for (const [w, a] of edges.get(t) || []) (w === 'pressed' ? everRift.press : everRift.release)(g, a);
    everRift.step(g);
  }
  assert.equal(createHash('sha256').update(JSON.stringify(g)).digest('hex'), '842bbdc2697aab4e386e784891521a8ebbd258aa0232cf90f9a683840221fc8b');
});

test('the rules keep to arithmetic every machine computes alike: a square root, but no sine, cosine, power or logarithm', () => {
  const source = readFileSync(new URL('../games/ever-rift/logic/rules.mjs', import.meta.url), 'utf8').replace(/\/\*[\s\S]*?\*\//g, '').replace(/\/\/.*$/gm, '');
  assert.doesNotMatch(source, /Math\.(sin|cos|tan|asin|acos|atan2?|sinh|cosh|tanh|exp|expm1|log\w*|pow|cbrt|hypot|random)\b/);
  assert.doesNotMatch(source, /\*\*/);
  assert.doesNotMatch(source, /\b(Date|performance)\b/);
});
