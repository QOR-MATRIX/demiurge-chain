// SPDX-License-Identifier: Apache-2.0
// Ever Rift's rules (QOR OS's QOR-0017): three floors down into the rift.
// Each floor is rooms and passages carved from the run's seed. Husks, wisps, brutes and seers wake as the delver comes
// near and close in; the delver cuts them down with a sword, a whirl that costs fury, a dash and potions, gathering
// gold, relics and experience. Stairs lead down from the room furthest from the way in, and the warden waits in the
// third floor's. The warden broken clears the rift; the delver's health spent ends the run, and so does the rift
// closing after ten minutes.
//
// The rules live here alone, so the game and ARQADE's service run the same ones (Demiurge's ADR-102): the QQ game's
// logic (logic/ever-rift.qml) hands each event to them and draws what they say, and the service replays a run, the very
// file QQ records (`qqRun` 1), with `replay` below, to reach the outcome the player's game reached. Nothing here uses QQ
// or Qt, and nothing here comes from outside this package (ADR-087 decision 2).
//
// They keep to what every machine computes alike: adding, subtracting, multiplying, dividing, `Math.floor`, whole
// numbers, and `Math.sqrt`, which ECMAScript specifies exactly (IEEE 754's correctly rounded square root). No sine,
// cosine, power or logarithm decides anything: a swing's arc is a dot product held against a cosine written here as a
// number. Random numbers come from the run's seed through xoshiro128**, its state filled by splitmix32. A floor is
// measured in tiles of 40 units, with y downward.
//
// A game is a plain object; each function takes it and changes it:
//   create(seed)          a game on its title, from a seed (a whole number)
//   click(game, button)   a screen button: "Play", "Play again", "Resume"; anything else is not the rules'
//   press(game, action)   an action pressed: "up", "down", "left", "right" held to move; "attack" (held, it swings
//                         again), "whirl", "dash", "potion"; "pause" pauses and resumes
//   release(game, action) an action let go
//   step(game)            one step of 1/60 s, which moves only while the game plays
//   outcome(game)         { score, kills, gold, level, health, floor, seconds, cleared, over }: what a run came to
//   replay(run)           a QQ run (seed, ticks, edges, clicks) played again: its outcome

/** One step's length in seconds: QQ steps 60 times a second whatever the frame rate. */
export const STEP = 1 / 60
/** A floor: its tiles, each this many units across. */
export const TILE = 40
export const COLS = 44
export const ROWS = 32
export const FLOORS = 3
/** Steps a run may last before the rift closes: ten minutes. */
export const RUN_STEPS = 36000
/** Steps from the warden's fall to the run's end, while the rift is cleared. */
export const CLEARING_STEPS = 120

/** The delver: half its width against walls, its speed a step, and how it starts. */
export const HERO_R = 12
export const HERO_SPEED = 3
export const HEALTH = 100
export const POWER = 10
/** A swing: its length in steps, the step of it that strikes (counted down), its reach and the arc it covers. */
export const SWING_STEPS = 18
export const SWING_STRIKES = 12
export const SWING_REACH = 56
/** cos 60°: an enemy is in the swing's arc while the angle to it from where the delver faces is no wider. */
const ARC_COS = 0.5
/** How near an enemy must be for a swing to turn toward it. */
export const AIM_REACH = 96
/** Fury: what a hit gains, its most, and what a whirl costs; a whirl's length, its reach, and the steps it strikes at. */
export const FURY_HIT = 6
export const FURY_MOST = 100
export const WHIRL_COST = 35
export const WHIRL_STEPS = 24
export const WHIRL_REACH = 84
const WHIRL_STRIKES = [20, 12, 4]
/** A dash: its length and speed a step, and the steps before the next. */
export const DASH_STEPS = 10
export const DASH_SPEED = 9
export const DASH_COOL = 75
/** Potions: how many at the start and at most, what one heals (a share of most health, in hundredths), and the steps
 * before the next. */
export const POTIONS = 2
export const POTIONS_MOST = 5
export const POTION_HEALS = 45
export const POTION_COOL = 60
/** The steps after a wound in which no other wounds. */
export const HURT_HOLD = 20
/** How near an enemy wakes, how near its wakening wakes others, and how near a thing on the floor is picked up. */
export const WAKE_REACH = 260
export const PACK_REACH = 170
export const PICK_REACH = 26
/** A seer's bolt: its speed a step, its life, and how near the delver's middle it must come to strike. */
export const BOLT_SPEED = 4.2
export const BOLT_STEPS = 150
/** At most this many enemies on a floor, bolts and things on the floor at once. */
export const MOST_ENEMIES = 48
export const MOST_BOLTS = 24
export const MOST_ITEMS = 40

/**
 * The enemies, by kind: health, speed a step, half their width (and against walls at most 18, so the warden fits a
 * passage), the wound they deal, how near a blow lands, the steps they wind up and recover, experience and score.
 * A seer strikes from afar with bolts; the warden's blow is a slam all round it.
 */
export const KINDS = [
    { name: "husk", health: 28, speed: 1.6, r: 13, wound: 8, reach: 30, windup: 22, recover: 30, xp: 6, score: 20 },
    { name: "wisp", health: 12, speed: 2.5, r: 10, wound: 5, reach: 24, windup: 12, recover: 26, xp: 4, score: 15 },
    { name: "brute", health: 90, speed: 1.1, r: 20, wound: 20, reach: 40, windup: 42, recover: 40, xp: 20, score: 60 },
    { name: "seer", health: 22, speed: 1.4, r: 12, wound: 10, reach: 0, windup: 36, recover: 90, xp: 10, score: 35 },
    { name: "warden", health: 450, speed: 1.2, r: 30, wound: 24, reach: 100, windup: 60, recover: 70, xp: 150,
      score: 2500 },
]
export const HUSK = 0, WISP = 1, BRUTE = 2, SEER = 3, WARDEN = 4
/** Which kinds wake on each floor, out of 100: husk, wisp, brute, seer. */
const MIX = [[60, 30, 0, 10], [40, 25, 15, 20], [35, 20, 25, 20]]
/** The relics, as the said list and the screens name them. */
export const RELICS = ["Might", "Vigor", "Haste", "Wrath"]

/** The square root of a half: a diagonal step's share of each axis. */
const HALF_ROOT = 0.7071067811865476

function rotl(x, k) {
    return ((x << k) | (x >>> (32 - k))) >>> 0
}

/** The four words of xoshiro128**'s state, from a seed, by splitmix32. */
function seeded(seed) {
    let s = seed >>> 0
    const state = []
    for (let i = 0; i < 4; i++) {
        s = (s + 0x9e3779b9) >>> 0
        let z = s
        z = Math.imul(z ^ (z >>> 16), 0x85ebca6b) >>> 0
        z = Math.imul(z ^ (z >>> 13), 0xc2b2ae35) >>> 0
        state.push((z ^ (z >>> 16)) >>> 0)
    }
    if (state.every((w) => w === 0))
        state[0] = 1
    return state
}

/** The next 32-bit word, xoshiro128**. */
function nextWord(state) {
    const result = Math.imul(rotl(Math.imul(state[1], 5) >>> 0, 7), 9) >>> 0
    const t = (state[1] << 9) >>> 0
    state[2] = (state[2] ^ state[0]) >>> 0
    state[3] = (state[3] ^ state[1]) >>> 0
    state[1] = (state[1] ^ state[2]) >>> 0
    state[0] = (state[0] ^ state[3]) >>> 0
    state[2] = (state[2] ^ t) >>> 0
    state[3] = rotl(state[3], 11)
    return result
}

/** A whole number from `low` to `high`, both included. */
function between(state, low, high) {
    return low + Math.floor((nextWord(state) / 4294967296) * (high - low + 1))
}

/** True `percent` times in a hundred. */
function chance(state, percent) {
    return between(state, 0, 99) < percent
}

/** The experience that reaches each level after the first: 50, 150, 300, 500 and on. */
export function nextXp(level) {
    return 25 * level * (level + 1)
}

/** An enemy's health and wound on a floor (1 to 3): two fifths more health and a quarter more wound a floor down. */
export function scaled(value, floor, more) {
    return Math.floor((value * (100 + more * (floor - 1))) / 100)
}

// ── The floor ────────────────────────────────────────────────────────────────────────────────────────────────────────

/** Whether the tile at column `c` and row `r` is rock (outside the floor counts as rock). */
export function rock(game, c, r) {
    return c < 0 || r < 0 || c >= COLS || r >= ROWS || game.map[r * COLS + c] === 0
}

function solidAt(game, x, y) {
    return rock(game, Math.floor(x / TILE), Math.floor(y / TILE))
}

/** Whether a square of half-width `h` at (x, y) touches rock. `h` is under half a tile, so its corners are enough. */
function blocked(game, x, y, h) {
    return solidAt(game, x - h, y - h) || solidAt(game, x + h, y - h) || solidAt(game, x - h, y + h)
           || solidAt(game, x + h, y + h)
}

/** How far round a corner a body slips: a step along a passage that a corner blocks by no more than this eases it
 * aside, toward the open side, as far as the step would have gone. */
const SLIP = 16

/** Moves `body` by (dx, dy), each axis on its own, so it slides along walls; a blocked step tries a half, then a
 * quarter, so it comes close to the wall. A step along one axis that a corner blocks slips round it. */
function moveBy(game, body, dx, dy, h) {
    const along = (ax, ay) => {
        if (ax === 0 && ay === 0)
            return
        for (const part of [1, 0.5, 0.25]) {
            if (!blocked(game, body.x + ax * part, body.y + ay * part, h)) {
                body.x += ax * part
                body.y += ay * part
                return
            }
        }
        if ((ax !== 0 && dy !== 0) || (ay !== 0 && dx !== 0))
            return
        const step = Math.abs(ax + ay)
        for (let k = 4; k <= SLIP; k += 4) {
            for (const side of [-1, 1]) {
                const sx = ay !== 0 ? side * k : 0, sy = ax !== 0 ? side * k : 0
                if (!blocked(game, body.x + sx, body.y + sy, h) && !blocked(game, body.x + ax + sx, body.y + ay + sy, h)) {
                    const by = Math.min(step, k) / k
                    body.x += sx * by
                    body.y += sy * by
                    return
                }
            }
        }
    }
    along(dx, 0)
    along(0, dy)
}

/** Whether nothing but floor lies between two places: the line between them looked along every ten units. */
function inSight(game, ax, ay, bx, by) {
    const dx = bx - ax, dy = by - ay
    const n = Math.floor(Math.sqrt(dx * dx + dy * dy) / 10) + 1
    for (let i = 1; i < n; i++) {
        if (solidAt(game, ax + (dx * i) / n, ay + (dy * i) / n))
            return false
    }
    return true
}

function carve(game, c, r) {
    if (c > 0 && r > 0 && c < COLS - 1 && r < ROWS - 1)
        game.map[r * COLS + c] = 1
}

/** A passage two tiles wide from one room's middle to another's, along one axis and then the other. */
function passage(game, a, b, acrossFirst) {
    const ac = a.c + Math.floor(a.w / 2), ar = a.r + Math.floor(a.h / 2)
    const bc = b.c + Math.floor(b.w / 2), br = b.r + Math.floor(b.h / 2)
    const run = (c0, c1, r) => {
        for (let c = Math.min(c0, c1); c <= Math.max(c0, c1) + 1; c++) {
            carve(game, c, r)
            carve(game, c, r + 1)
        }
    }
    const fall = (r0, r1, c) => {
        for (let r = Math.min(r0, r1); r <= Math.max(r0, r1) + 1; r++) {
            carve(game, c, r)
            carve(game, c + 1, r)
        }
    }
    if (acrossFirst) {
        run(ac, bc, ar)
        fall(ar, br, bc)
    } else {
        fall(ar, br, ac)
        run(ac, bc, br)
    }
}

/** How far each floor tile is from tile (c, r), in steps of one tile across or along: -1 for rock and what cannot be
 * reached. */
function distances(game, c, r) {
    const far = new Array(COLS * ROWS).fill(-1)
    if (rock(game, c, r))
        return far
    const queue = [r * COLS + c]
    far[queue[0]] = 0
    for (let head = 0; head < queue.length; head++) {
        const at = queue[head], ac = at % COLS, ar = Math.floor(at / COLS)
        for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
            const nc = ac + dc, nr = ar + dr
            if (!rock(game, nc, nr) && far[nr * COLS + nc] < 0) {
                far[nr * COLS + nc] = far[at] + 1
                queue.push(nr * COLS + nc)
            }
        }
    }
    return far
}

function middle(room) {
    return { x: (room.c + room.w / 2) * TILE, y: (room.r + room.h / 2) * TILE }
}

/** An enemy of `kind` at (x, y), its health and wound for the floor. */
function enemy(game, kind, x, y) {
    const k = KINDS[kind]
    const health = scaled(k.health, game.floor, 40)
    game.enemies.push({ id: game.made++, kind: kind, x: x, y: y, health: health, most: health, awake: false,
                        woke: false, act: "move", timer: 0, cool: 0, fx: 0, fy: 1, kx: 0, ky: 0, struck: 0, aimX: x,
                        aimY: y })
}

/**
 * The floor `game.floor`, carved from the seed (`carved` counts the floors carved, so a drawing knows a new one): up to nine rooms that keep apart, each joined to the one before by a
 * passage and one more pair joined for a loop; the delver in the first room, and in the room furthest from it by
 * walking, the stairs down, or on the last floor the warden. The other rooms hold enemies, more a floor down.
 */
function carveFloor(game) {
    game.carved += 1
    game.map = new Array(COLS * ROWS).fill(0)
    game.rooms = []
    for (let tries = 0; tries < 200 && game.rooms.length < 9; tries++) {
        const w = between(game.random, 5, 9), h = between(game.random, 4, 7)
        const c = between(game.random, 1, COLS - w - 1), r = between(game.random, 1, ROWS - h - 1)
        if (game.rooms.some((o) => c < o.c + o.w + 2 && o.c < c + w + 2 && r < o.r + o.h + 2 && o.r < r + h + 2))
            continue
        game.rooms.push({ c: c, r: r, w: w, h: h })
        if (tries > 120 && game.rooms.length >= 5)
            break
    }
    for (const room of game.rooms) {
        for (let r = room.r; r < room.r + room.h; r++) {
            for (let c = room.c; c < room.c + room.w; c++)
                carve(game, c, r)
        }
    }
    for (let i = 1; i < game.rooms.length; i++)
        passage(game, game.rooms[i - 1], game.rooms[i], between(game.random, 0, 1) === 0)
    if (game.rooms.length > 3) {
        const a = between(game.random, 0, game.rooms.length - 1)
        const b = (a + 2 + between(game.random, 0, game.rooms.length - 3)) % game.rooms.length
        passage(game, game.rooms[a], game.rooms[b], between(game.random, 0, 1) === 0)
    }

    const start = middle(game.rooms[0])
    game.x = start.x
    game.y = start.y
    const far = distances(game, Math.floor(start.x / TILE), Math.floor(start.y / TILE))
    let last = 1, most = -1
    for (let i = 1; i < game.rooms.length; i++) {
        const m = middle(game.rooms[i])
        const d = far[Math.floor(m.y / TILE) * COLS + Math.floor(m.x / TILE)]
        if (d > most) {
            most = d
            last = i
        }
    }
    game.last = last
    const end = middle(game.rooms[last])
    game.stairs = game.floor < FLOORS ? { x: end.x, y: end.y } : null

    game.enemies = []
    game.bolts = []
    game.items = []
    const mix = MIX[game.floor - 1]
    for (let i = 1; i < game.rooms.length; i++) {
        const room = game.rooms[i]
        const count = Math.min(MOST_ENEMIES - game.enemies.length, between(game.random, 2, 4) + game.floor)
        for (let n = 0; n < count; n++) {
            let roll = between(game.random, 0, 99), kind = 0
            while (roll >= mix[kind]) {
                roll -= mix[kind]
                kind += 1
            }
            const x = (room.c + 0.5 + between(game.random, 0, (room.w - 1) * 8) / 8) * TILE
            const y = (room.r + 0.5 + between(game.random, 0, (room.h - 1) * 8) / 8) * TILE
            enemy(game, kind, x, y)
        }
    }
    if (game.floor === FLOORS)
        enemy(game, WARDEN, end.x, end.y)
    game.paths = distances(game, Math.floor(game.x / TILE), Math.floor(game.y / TILE))
    game.pathsFrom = Math.floor(game.y / TILE) * COLS + Math.floor(game.x / TILE)
}

// ── A run ────────────────────────────────────────────────────────────────────────────────────────────────────────────

function begin(game) {
    game.held = { up: false, down: false, left: false, right: false, attack: false }
    game.asked = { attack: false, whirl: false, dash: false, potion: false }
    game.facing = [0, 1]
    game.steps = 0
    game.floor = 1
    game.health = HEALTH
    game.most = HEALTH
    game.power = POWER
    game.haste = 100
    game.wrath = 100
    game.fury = 0
    game.potions = POTIONS
    game.relics = [0, 0, 0, 0]
    game.xp = 0
    game.level = 1
    game.gold = 0
    game.kills = 0
    game.score = 0
    game.swing = 0
    game.whirl = 0
    game.dash = 0
    game.dashCool = 0
    game.potionCool = 0
    game.hurt = 0
    game.clearing = 0
    game.cleared = false
    game.wardenDown = false
    game.made = 0
    game.said = []
    carveFloor(game)
    game.phase = "playing"
}

/** A game on its title screen, from `seed`. */
export function create(seed) {
    const game = { seed: seed >>> 0, random: seeded(seed), phase: "title", carved: 0 }
    begin(game)
    game.phase = "title"
    return game
}

/** A screen button pressed. */
export function click(game, button) {
    if (button === "Play" && game.phase === "title")
        game.phase = "playing"
    else if (button === "Play again" && game.phase === "over")
        begin(game)
    else if (button === "Resume" && game.phase === "paused")
        game.phase = "playing"
    else
        return
    // What was pressed on a screen, the title's or the pause's, does nothing once play starts.
    game.asked = { attack: false, whirl: false, dash: false, potion: false }
}

/** An action pressed. Moving and attacking are held whatever the phase, so a key held through a pause still counts
 * after it; a press of the others is kept for the next step, so a press let go before it still counts. */
export function press(game, action) {
    if (action in game.held)
        game.held[action] = true
    if (action in game.asked)
        game.asked[action] = true
    else if (action === "pause" && game.phase === "playing")
        game.phase = "paused"
    else if (action === "pause" && game.phase === "paused")
        game.phase = "playing"
}

/** An action let go. */
export function release(game, action) {
    if (action in game.held)
        game.held[action] = false
}

function near(ax, ay, bx, by, reach) {
    const dx = ax - bx, dy = ay - by
    return dx * dx + dy * dy < reach * reach
}

/** The delver wounded, unless it dashes or is still reeling from the last wound. */
function wound(game, amount, from) {
    if (game.dash > 0 || game.hurt > 0 || game.health <= 0)
        return
    game.health = Math.max(0, game.health - amount)
    game.hurt = HURT_HOLD
    game.said.push({ what: "hurt", amount: amount, x: game.x, y: game.y, by: from })
}

/** An enemy struck for `amount`, knocked back from the delver, and woken if it slept. */
function strike(game, e, amount, crit) {
    e.health -= amount
    e.struck = 8
    e.awake = true
    const dx = e.x - game.x, dy = e.y - game.y, length = Math.sqrt(dx * dx + dy * dy) || 1
    const push = e.kind === WARDEN ? 0 : e.kind === BRUTE ? 2 : 5
    e.kx = (dx / length) * push
    e.ky = (dy / length) * push
    game.fury = Math.min(FURY_MOST, game.fury + Math.floor((FURY_HIT * game.wrath) / 100))
    game.said.push({ what: crit ? "crit" : "hit", id: e.id, amount: amount, x: e.x, y: e.y, kind: e.kind })
}

/** The swing's or whirl's damage for one blow: the delver's power and up to half again, doubled one time in ten. */
function blow(game, share) {
    const base = Math.floor((game.power * share) / 100)
    const amount = base + between(game.random, 0, Math.floor(base / 2))
    return chance(game.random, 10) ? [amount * 2, true] : [amount, false]
}

/** What an enemy leaves when it falls: gold more often than not, now and then a potion or a relic. */
function drop(game, e) {
    const at = (spread) => ({ x: e.x + between(game.random, -spread, spread), y: e.y + between(game.random, -spread, spread) })
    const add = (item) => {
        if (game.items.length < MOST_ITEMS)
            game.items.push(Object.assign({ id: game.made++ }, item, at(e.kind === WARDEN ? 30 : 10)))
    }
    const goldChance = [55, 40, 100, 60, 100][e.kind]
    if (chance(game.random, goldChance)) {
        const piles = e.kind === WARDEN ? 6 : 1
        for (let i = 0; i < piles; i++)
            add({ what: "gold", amount: between(game.random, 4, 12) * game.floor * (e.kind === BRUTE ? 2 : 1) })
    }
    if (chance(game.random, [9, 6, 30, 12, 0][e.kind]))
        add({ what: "potion" })
    if (e.kind === WARDEN || chance(game.random, [4, 3, 20, 6, 0][e.kind]))
        add({ what: "relic", relic: between(game.random, 0, RELICS.length - 1) })
}

function levelUp(game) {
    while (game.xp >= nextXp(game.level)) {
        game.level += 1
        game.most += 12
        game.power += 2
        game.health = Math.min(game.most, game.health + Math.floor((game.most * 30) / 100))
        game.said.push({ what: "level", level: game.level })
    }
}

function pick(game, item) {
    if (item.what === "gold") {
        game.gold += item.amount
        game.score += item.amount
    } else if (item.what === "potion") {
        if (game.potions >= POTIONS_MOST)
            return false
        game.potions += 1
    } else {
        game.relics[item.relic] += 1
        if (item.relic === 0)
            game.power += 3
        else if (item.relic === 1) {
            game.most += 20
            game.health = Math.min(game.most, game.health + 20)
        } else if (item.relic === 2)
            game.haste = Math.min(130, game.haste + 6)
        else
            game.wrath += 25
    }
    game.said.push({ what: "pick", item: item.what, relic: item.relic, amount: item.amount, x: item.x, y: item.y })
    return true
}

/** The delver: what it asked for, then its movement, the stairs, and what lies at its feet. */
function delver(game) {
    if (game.swing > 0)
        game.swing -= 1
    if (game.whirl > 0)
        game.whirl -= 1
    if (game.dash > 0)
        game.dash -= 1
    if (game.dashCool > 0)
        game.dashCool -= 1
    if (game.potionCool > 0)
        game.potionCool -= 1
    if (game.hurt > 0)
        game.hurt -= 1

    let vx = (game.held.right ? 1 : 0) - (game.held.left ? 1 : 0)
    let vy = (game.held.down ? 1 : 0) - (game.held.up ? 1 : 0)
    if (vx !== 0 && vy !== 0) {
        vx *= HALF_ROOT
        vy *= HALF_ROOT
    }
    if ((vx !== 0 || vy !== 0) && game.swing === 0 && game.dash === 0)
        game.facing = [vx, vy]

    const asked = game.asked
    game.asked = { attack: false, whirl: false, dash: false, potion: false }
    if (asked.potion && game.potions > 0 && game.potionCool === 0 && game.health < game.most) {
        game.potions -= 1
        game.potionCool = POTION_COOL
        game.health = Math.min(game.most, game.health + Math.floor((game.most * POTION_HEALS) / 100))
        game.said.push({ what: "potion" })
    }
    if (asked.dash && game.dashCool === 0 && game.dash === 0) {
        if (vx !== 0 || vy !== 0)
            game.facing = [vx, vy]
        game.dash = DASH_STEPS
        game.dashCool = DASH_COOL
        game.swing = 0
        game.said.push({ what: "dash", x: game.x, y: game.y })
    }
    if (asked.whirl && game.whirl === 0 && game.dash === 0 && game.fury >= WHIRL_COST) {
        game.fury -= WHIRL_COST
        game.whirl = WHIRL_STEPS
        game.swing = 0
        game.said.push({ what: "whirl", x: game.x, y: game.y })
    }
    if ((asked.attack || game.held.attack) && game.swing === 0 && game.whirl === 0 && game.dash === 0) {
        // The swing turns toward the nearest enemy awake within reach, so a pad aims it as well as a pointer.
        let target = null, best = AIM_REACH * AIM_REACH
        for (const e of game.enemies) {
            const dx = e.x - game.x, dy = e.y - game.y, d = dx * dx + dy * dy
            if (e.health > 0 && d < best) {
                best = d
                target = e
            }
        }
        if (target) {
            const dx = target.x - game.x, dy = target.y - game.y, length = Math.sqrt(dx * dx + dy * dy) || 1
            game.facing = [dx / length, dy / length]
        }
        game.swing = SWING_STEPS
        game.said.push({ what: "swing", x: game.x, y: game.y })
    }

    if (game.dash > 0) {
        moveBy(game, game, game.facing[0] * DASH_SPEED, game.facing[1] * DASH_SPEED, HERO_R)
    } else {
        // Planted while a swing winds up, slowed while it follows through or a whirl turns.
        const share = game.swing > SWING_STRIKES ? 0 : game.swing > 0 ? 40 : game.whirl > 0 ? 60 : 100
        const speed = (HERO_SPEED * game.haste * share) / 10000
        moveBy(game, game, vx * speed, vy * speed, HERO_R)
    }

    if (game.stairs && near(game.x, game.y, game.stairs.x, game.stairs.y, 22)) {
        game.floor += 1
        game.score += 300
        carveFloor(game)
        game.said.push({ what: "descend", floor: game.floor })
        return
    }
    game.items = game.items.filter((item) => !(near(game.x, game.y, item.x, item.y, PICK_REACH) && pick(game, item)))

    const at = Math.floor(game.y / TILE) * COLS + Math.floor(game.x / TILE)
    if (at !== game.pathsFrom) {
        game.paths = distances(game, at % COLS, Math.floor(at / COLS))
        game.pathsFrom = at
    }
}

/** The swing's and the whirl's blows, at the steps they strike. */
function blows(game) {
    if (game.swing === SWING_STRIKES) {
        const [fx, fy] = game.facing
        for (const e of game.enemies) {
            const dx = e.x - game.x, dy = e.y - game.y, d = Math.sqrt(dx * dx + dy * dy)
            const reach = SWING_REACH + KINDS[e.kind].r
            if (e.health > 0 && d < reach && (d < KINDS[e.kind].r || dx * fx + dy * fy >= ARC_COS * d)) {
                const [amount, crit] = blow(game, 100)
                strike(game, e, amount, crit)
            }
        }
    }
    if (game.whirl > 0 && WHIRL_STRIKES.includes(game.whirl)) {
        for (const e of game.enemies) {
            if (e.health > 0 && near(e.x, e.y, game.x, game.y, WHIRL_REACH + KINDS[e.kind].r)) {
                const [amount, crit] = blow(game, 80)
                strike(game, e, amount, crit)
            }
        }
    }
}

/** Where an enemy heads: straight for the delver while it sees it, otherwise one tile nearer it by the floor's ways. */
function heading(game, e) {
    if (inSight(game, e.x, e.y, game.x, game.y))
        return [game.x - e.x, game.y - e.y]
    const c = Math.floor(e.x / TILE), r = Math.floor(e.y / TILE)
    let best = game.paths[r * COLS + c], to = null
    for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        const d = game.paths[(r + dr) * COLS + c + dc]
        if (!rock(game, c + dc, r + dr) && d >= 0 && (best < 0 || d < best)) {
            best = d
            to = [(c + dc + 0.5) * TILE, (r + dr + 0.5) * TILE]
        }
    }
    return to ? [to[0] - e.x, to[1] - e.y] : [game.x - e.x, game.y - e.y]
}

function walk(game, e, dx, dy, share) {
    const length = Math.sqrt(dx * dx + dy * dy)
    if (length < 0.001)
        return
    const k = KINDS[e.kind], speed = (k.speed * share) / 100
    e.fx = dx / length
    e.fy = dy / length
    moveBy(game, e, e.fx * speed, e.fy * speed, Math.min(18, k.r))
}

/** The enemies: they wake, close in, wind up and strike, or for a seer keep away and loose bolts. */
function enemies(game) {
    for (const e of game.enemies) {
        if (e.health <= 0)
            continue
        const k = KINDS[e.kind]
        if (e.struck > 0)
            e.struck -= 1
        if (e.kx !== 0 || e.ky !== 0) {
            moveBy(game, e, e.kx, e.ky, Math.min(18, k.r))
            e.kx = Math.abs(e.kx) < 0.2 ? 0 : e.kx * 0.7
            e.ky = Math.abs(e.ky) < 0.2 ? 0 : e.ky * 0.7
        }
        if (!e.awake) {
            if (near(e.x, e.y, game.x, game.y, WAKE_REACH) && inSight(game, e.x, e.y, game.x, game.y))
                e.awake = true
            else
                continue
        }
        if (!e.woke) {
            e.woke = true
            game.said.push({ what: e.kind === WARDEN ? "warden" : "wake", x: e.x, y: e.y, kind: e.kind })
            for (const o of game.enemies) {
                if (!o.awake && near(o.x, o.y, e.x, e.y, PACK_REACH))
                    o.awake = true
            }
        }
        const dx = game.x - e.x, dy = game.y - e.y, d = Math.sqrt(dx * dx + dy * dy)
        if (e.cool > 0)
            e.cool -= 1

        if (e.act === "windup") {
            e.timer -= 1
            if (e.timer > 0)
                continue
            if (e.kind === SEER) {
                const ax = e.aimX - e.x, ay = e.aimY - e.y, length = Math.sqrt(ax * ax + ay * ay) || 1
                if (game.bolts.length < MOST_BOLTS)
                    game.bolts.push({ id: game.made++, x: e.x, y: e.y, vx: (ax / length) * BOLT_SPEED,
                                      vy: (ay / length) * BOLT_SPEED, life: BOLT_STEPS,
                                      wound: scaled(k.wound, game.floor, 25) })
                game.said.push({ what: "bolt", x: e.x, y: e.y })
            } else {
                if (e.kind === WARDEN)
                    game.said.push({ what: "slam", x: e.x, y: e.y })
                if (d < k.reach + HERO_R + (e.kind === WARDEN ? 0 : k.r))
                    wound(game, scaled(k.wound, game.floor, 25), e.kind)
            }
            e.act = "recover"
            e.timer = k.recover
            continue
        }
        if (e.act === "recover") {
            e.timer -= 1
            if (e.timer <= 0)
                e.act = "move"
            else if (e.kind !== SEER)
                walk(game, e, dx, dy, 25)
            continue
        }

        if (e.kind === SEER) {
            const sees = inSight(game, e.x, e.y, game.x, game.y)
            if (sees && d < 320 && e.cool === 0) {
                e.act = "windup"
                e.timer = k.windup
                e.cool = k.recover + k.windup
                e.aimX = game.x
                e.aimY = game.y
                e.fx = dx / (d || 1)
                e.fy = dy / (d || 1)
            } else if (sees && d < 150) {
                walk(game, e, -dx, -dy, 100)
            } else if (!sees || d > 260) {
                const [hx, hy] = heading(game, e)
                walk(game, e, hx, hy, 100)
            }
            continue
        }
        if (e.kind === WARDEN && e.cool === 0 && game.enemies.length < MOST_ENEMIES - 1) {
            // The warden calls two husks out of the rift every nine seconds.
            e.cool = 540
            for (const side of [-1, 1])
                enemy(game, HUSK, e.x + side * 50, e.y)
            for (const o of game.enemies.slice(-2))
                o.awake = true
            game.said.push({ what: "summon", x: e.x, y: e.y })
        }
        const reach = e.kind === WARDEN ? k.reach - 10 : k.reach + HERO_R + k.r
        if (d < reach) {
            e.act = "windup"
            e.timer = k.windup
            e.fx = dx / (d || 1)
            e.fy = dy / (d || 1)
            if (e.kind === WARDEN)
                game.said.push({ what: "rising", x: e.x, y: e.y })
            continue
        }
        let [hx, hy] = heading(game, e)
        if (e.kind === WISP) {
            // A wisp weaves as it comes, turning a little to one side and then the other.
            const side = Math.floor((game.steps + e.id * 13) / 40) % 2 === 0 ? 0.6 : -0.6
            const across = -hy * side, along = hx * side
            hx += across
            hy += along
        }
        walk(game, e, hx, hy, 100)
    }

    // Enemies awake keep apart from each other and from the delver.
    const awake = game.enemies.filter((e) => e.awake && e.health > 0)
    for (let i = 0; i < awake.length; i++) {
        const a = awake[i], ka = KINDS[a.kind]
        for (let j = i + 1; j < awake.length; j++) {
            const b = awake[j], kb = KINDS[b.kind]
            const dx = b.x - a.x, dy = b.y - a.y, d = Math.sqrt(dx * dx + dy * dy), apart = ka.r + kb.r
            if (d < apart && d > 0.001) {
                const push = (apart - d) / 2
                moveBy(game, a, (-dx / d) * push, (-dy / d) * push, Math.min(18, ka.r))
                moveBy(game, b, (dx / d) * push, (dy / d) * push, Math.min(18, kb.r))
            }
        }
        const dx = a.x - game.x, dy = a.y - game.y, d = Math.sqrt(dx * dx + dy * dy), apart = ka.r + HERO_R
        if (d < apart && d > 0.001)
            moveBy(game, a, (dx / d) * (apart - d), (dy / d) * (apart - d), Math.min(18, ka.r))
    }
}

function bolts(game) {
    for (const b of game.bolts) {
        b.x += b.vx
        b.y += b.vy
        b.life -= 1
        if (solidAt(game, b.x, b.y))
            b.life = 0
        else if (near(b.x, b.y, game.x, game.y, HERO_R + 6)) {
            wound(game, b.wound, SEER)
            b.life = 0
        }
    }
    game.bolts = game.bolts.filter((b) => b.life > 0)
}

/** The fallen: experience, score and what they leave; the warden's fall clears the rift. */
function fallen(game) {
    for (const e of game.enemies) {
        if (e.health > 0)
            continue
        const k = KINDS[e.kind]
        game.kills += 1
        game.score += k.score
        game.xp += k.xp
        game.said.push({ what: "kill", kind: e.kind, x: e.x, y: e.y })
        drop(game, e)
        if (e.kind === WARDEN) {
            game.wardenDown = true
            game.clearing = CLEARING_STEPS
            game.said.push({ what: "warden down", x: e.x, y: e.y })
        }
    }
    game.enemies = game.enemies.filter((e) => e.health > 0)
    levelUp(game)
}

/**
 * One step: the delver acts and moves, its blows land, the enemies act, bolts fly, the fallen leave what they leave,
 * and the run may end. What happened is in `said`.
 */
export function step(game) {
    if (game.phase !== "playing")
        return
    game.steps += 1
    game.said = []
    if (game.clearing > 0) {
        game.clearing -= 1
        if (game.clearing === 0) {
            game.cleared = true
            game.score += Math.floor((RUN_STEPS - game.steps) / 6)
            game.phase = "over"
            game.said.push({ what: "cleared" })
            return
        }
    }
    const floor = game.floor
    delver(game)
    if (game.floor === floor) {
        blows(game)
        enemies(game)
        bolts(game)
        fallen(game)
    }
    if (game.health <= 0) {
        game.phase = "over"
        game.said.push({ what: "fallen" })
    } else if (game.steps >= RUN_STEPS && game.phase === "playing") {
        game.phase = "over"
        game.said.push({ what: "closed" })
    }
}

/** What the game came to. */
export function outcome(game) {
    return {
        score: game.score,
        kills: game.kills,
        gold: game.gold,
        level: game.level,
        health: game.health,
        floor: game.floor,
        seconds: Math.floor(game.steps / 60),
        cleared: game.cleared,
        over: game.phase === "over",
    }
}

/**
 * A QQ run played again: { seed, ticks, edges: [[tick, [["pressed" or "released", action], ...]], ...],
 * clicks: [[tick, [button, ...]], ...] }, as QQ writes it (format 1). Each tick takes its clicks, then its presses and
 * releases, then steps, as QQ's scene does. Its outcome, which is the one the game reached if the run is the game's.
 */
export function replay(run) {
    if (!run || run.qqRun !== 1 || !Number.isInteger(run.seed) || !Number.isInteger(run.ticks) || run.ticks < 0)
        throw new Error("This replays QQ runs of format 1.")
    const edges = new Map((run.edges || []).map(([tick, list]) => [tick, list]))
    const clicks = new Map((run.clicks || []).map(([tick, list]) => [tick, list]))
    const game = create(run.seed)
    for (let tick = 0; tick < run.ticks; tick++) {
        for (const button of clicks.get(tick) || [])
            click(game, button)
        for (const [what, action] of edges.get(tick) || []) {
            if (what === "pressed")
                press(game, action)
            else if (what === "released")
                release(game, action)
        }
        step(game)
    }
    return outcome(game)
}
