// Rift Survivor's rules (DIRECTION P7.20, ADR-102 decision 2): one signal in an arena for ninety seconds. Enemies come in
// from its edges and close on the ship; the ship fires on the nearest by itself, and the player moves it. Each kill
// scores, more in a chain of kills, and leaves a fragment to collect. Rift pulses open where the ship was, arm after a
// moment and breach the hull of a ship still inside. Every so many kills the signal evolves: the world holds while the
// player chooses one of three perks. Four breaches end the run; ninety seconds survived completes it.
//
// The rules live here alone, so the game and ARQADE's service run the same ones (ADR-102's consequences): the QQ game's
// logic (logic/rift-survivor.qml) hands each event to them and draws what they say, and the service replays a run, the
// very file QQ records (`qqRun` 1), with `replay` below, to reach the outcome the player's game reached. Nothing here
// uses QQ or Qt. Each game's package keeps its own rules whole, random numbers and all, since a game reaches nothing
// outside its package (ADR-087 decision 2).
//
// They keep to what every machine computes alike: adding, subtracting, multiplying, dividing, `Math.floor`, whole
// numbers, and `Math.sqrt`, which ECMAScript specifies exactly (IEEE 754's correctly rounded square root), unlike
// `Math.hypot`, the trigonometric functions, powers and logarithms, which each system's library may round differently
// (QQ's README, "What a replay reaches"). A shot's spread turns by angles whose cosines and sines are written here as
// numbers. Random numbers come from the run's seed through xoshiro128**, its state filled by splitmix32. The arena is
// measured as the website's was, 1000 by 620, with y downward.
//
// A game is a plain object; each function takes it and changes it:
//   create(seed)          a game on its title, from a seed (a whole number)
//   click(game, button)   a screen button: "Play", "Play again", "Resume", and the perks "Overclock", "Fracture",
//                         "Second wind" while the signal evolves; anything else is not the rules'
//   press(game, action)   an action pressed: "up", "down", "left", "right" held to move; "perk1" to "perk3" choose a
//                         perk; "pause" pauses and resumes
//   release(game, action) an action let go
//   step(game)            one step of 1/60 s, which moves only while the game plays
//   outcome(game)         { score, kills, level, hull, seconds, survived, over }: what a run came to
//   replay(run)           a QQ run (seed, ticks, edges, clicks) played again: its outcome

/** One step's length in seconds: QQ steps 60 times a second whatever the frame rate. */
export const STEP = 1 / 60
/** The arena, and how near its edges the ship may go. */
export const WIDTH = 1000
export const HEIGHT = 620
export const MARGIN = 20
/** Steps a run lasts: ninety seconds. Breaches the hull takes. */
export const RUN_STEPS = 5400
export const HULL = 4
/** The ship's speed, in the arena's units a second; the steps after a breach in which the hull holds. */
export const SHIP_SPEED = 240
export const HOLD_STEPS = 96
/** A chain of kills breaks after this many steps without one, and counts up to this many. */
export const CHAIN_STEPS = 180
export const CHAIN_MOST = 20
/** At most this many enemies, shots and fragments at once. */
export const MOST_ENEMIES = 65
export const MOST_SHOTS = 180
export const MOST_FRAGMENTS = 100
/** A shot's speed, a second, and its life in steps; how near it must come to an enemy's edge to hit. */
export const SHOT_SPEED = 550
export const SHOT_STEPS = 108
/** A rift pulse: its reach, and when it arms, stops and is gone, in steps after it opens. */
export const PULSE_REACH = 90
export const PULSE_ARMS = 81
export const PULSE_ENDS = 102
export const PULSE_GONE = 111
/** The perks, as their buttons are named. */
export const PERKS = ["Overclock", "Fracture", "Second wind"]

/** cos and sin of 0.14, 0.28 and 0.42 radians: a shot's spread turns by these. */
const TURNS = [[1, 0], [0.9902159962126371, 0.1395431146442365], [0.9610554383107709, 0.27635564856411376],
               [0.9130889403123083, 0.4077604530595702]]
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

/** Steps between one enemy and the next, `steps` into the run: from 48 down to 11. */
export function spawnEvery(steps) {
    return Math.max(11, 48 - Math.floor((steps * 6) / 1000))
}

/** Steps between one rift pulse and the next: 330, and 228 after forty-five seconds. */
export function pulseEvery(steps) {
    return steps > 2700 ? 228 : 330
}

/** An enemy's speed, in the arena's units a second, `steps` into the run. */
export function enemySpeed(kind, steps) {
    return (kind === 1 ? 44 : 67) + (steps / 60) * 0.55
}

function begin(game) {
    game.x = 500
    game.y = 310
    game.facing = [0, -1]
    game.held = { up: false, down: false, left: false, right: false }
    game.steps = 0
    game.spawn = 0
    game.shot = 0
    game.pulse = 0
    game.score = 0
    game.hull = HULL
    game.hold = 0
    game.chain = 0
    game.chainSteps = 0
    game.kills = 0
    game.level = 1
    game.nextLevel = 10
    game.fireEvery = 23
    game.damage = 1
    game.magnet = 90
    game.spread = 0
    game.enemies = []
    game.shots = []
    game.fragments = []
    game.pulses = []
    game.made = 0
    game.said = []
    game.phase = "playing"
}

/** A game on its title screen, from `seed`. */
export function create(seed) {
    const game = { seed: seed >>> 0, random: seeded(seed), phase: "title" }
    begin(game)
    game.phase = "title"
    return game
}

/** The perk chosen while the signal evolves, by its index. */
function choose(game, perk) {
    if (game.phase !== "choosing" || perk < 0 || perk >= PERKS.length)
        return
    if (perk === 0)
        game.fireEvery = Math.max(6, Math.floor((game.fireEvery * 3) / 4))
    else if (perk === 1)
        game.spread = Math.min(3, game.spread + 1)
    else {
        game.hull = Math.min(HULL, game.hull + 2)
        game.magnet += 40
    }
    game.said.push({ what: "perk", perk: perk })
    game.phase = "playing"
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
        choose(game, PERKS.indexOf(button))
}

/** An action pressed. Moving is held whatever the phase, so a key held through a pause still moves after it. */
export function press(game, action) {
    if (action in game.held)
        game.held[action] = true
    else if (action === "perk1" || action === "perk2" || action === "perk3")
        choose(game, Number(action.slice(4)) - 1)
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

function breach(game) {
    game.hull -= 1
    game.hold = HOLD_STEPS
    game.chain = 0
    game.said.push({ what: "hit", x: game.x, y: game.y })
}

function near(ax, ay, bx, by, reach) {
    const dx = ax - bx, dy = ay - by
    return dx * dx + dy * dy < reach * reach
}

/**
 * One step, in the website's order: the hold and the chain wear off; the ship moves; rift pulses open, age and breach;
 * an enemy may come; the ship fires on the nearest; enemies close and breach; shots fly and hit; fragments are drawn in
 * and collected; the signal may evolve; and the run may end. What happened is in `said`.
 */
export function step(game) {
    if (game.phase !== "playing")
        return
    game.steps += 1
    game.said = []
    if (game.hold > 0)
        game.hold -= 1
    if (game.chainSteps > 0) {
        game.chainSteps -= 1
        if (game.chainSteps === 0)
            game.chain = 0
    }

    // The ship.
    let vx = (game.held.right ? 1 : 0) - (game.held.left ? 1 : 0)
    let vy = (game.held.down ? 1 : 0) - (game.held.up ? 1 : 0)
    if (vx !== 0 && vy !== 0) {
        vx *= HALF_ROOT
        vy *= HALF_ROOT
    }
    if (vx !== 0 || vy !== 0)
        game.facing = [vx, vy]
    const move = SHIP_SPEED * STEP
    game.x = Math.max(MARGIN, Math.min(WIDTH - MARGIN, game.x + vx * move))
    game.y = Math.max(MARGIN, Math.min(HEIGHT - MARGIN, game.y + vy * move))

    // Rift pulses, after five seconds: one opens where the ship is, arms, and breaches a ship still inside.
    game.pulse += 1
    if (game.steps > 300 && game.pulse >= pulseEvery(game.steps)) {
        game.pulse = 0
        game.pulses.push({ id: game.made++, x: game.x, y: game.y, age: 0 })
        game.said.push({ what: "pulse" })
    }
    for (const p of game.pulses) {
        p.age += 1
        if (p.age >= PULSE_ARMS && p.age < PULSE_ENDS && game.hold === 0 && near(game.x, game.y, p.x, p.y, PULSE_REACH))
            breach(game)
    }
    game.pulses = game.pulses.filter((p) => p.age < PULSE_GONE)

    // An enemy from an edge.
    game.spawn += 1
    if (game.spawn >= spawnEvery(game.steps) && game.enemies.length < MOST_ENEMIES) {
        game.spawn = 0
        const edge = between(game.random, 0, 3)
        const kind = between(game.random, 0, 9999) < Math.min(4000, Math.floor((game.steps * 10) / 9)) ? 1 : 0
        const along = between(game.random, 0, 9999) / 10000
        game.enemies.push({ id: game.made++, kind: kind, hp: kind === 1 ? 3 : 1, r: kind === 1 ? 18 : 11,
                            x: edge === 0 ? -20 : edge === 1 ? WIDTH + 20 : along * WIDTH,
                            y: edge === 2 ? -20 : edge === 3 ? HEIGHT + 20 : along * HEIGHT })
    }

    // The ship fires on the nearest enemy, a spread of shots if it has one.
    game.shot += 1
    if (game.enemies.length > 0 && game.shot >= game.fireEvery) {
        game.shot = 0
        let target = game.enemies[0], best = Infinity
        for (const e of game.enemies) {
            const dx = e.x - game.x, dy = e.y - game.y, d = dx * dx + dy * dy
            if (d < best) {
                best = d
                target = e
            }
        }
        const dx = target.x - game.x, dy = target.y - game.y
        const length = Math.sqrt(dx * dx + dy * dy) || 1
        const ux = dx / length, uy = dy / length
        const speed = SHOT_SPEED * STEP
        for (let i = -game.spread; i <= game.spread; i++) {
            const [c, s] = TURNS[Math.abs(i)]
            const sn = i < 0 ? -s : s
            const sx = ux * c - uy * sn, sy = ux * sn + uy * c
            game.shots.push({ id: game.made++, x: game.x, y: game.y, vx: sx * speed, vy: sy * speed, life: SHOT_STEPS })
        }
        game.said.push({ what: "shot" })
    }

    // Enemies close on the ship, and breach it on touching.
    for (const e of game.enemies) {
        const dx = game.x - e.x, dy = game.y - e.y
        const length = Math.sqrt(dx * dx + dy * dy) || 1
        const speed = enemySpeed(e.kind, game.steps) * STEP
        e.x += (dx / length) * speed
        e.y += (dy / length) * speed
        if (length < e.r + 12 && game.hold === 0)
            breach(game)
    }

    // Shots fly, and hit.
    for (const b of game.shots) {
        b.x += b.vx
        b.y += b.vy
        b.life -= 1
        for (const e of game.enemies) {
            if (e.hp <= 0 || b.life <= 0 || !near(b.x, b.y, e.x, e.y, e.r + 5))
                continue
            e.hp -= game.damage
            b.life = 0
            if (e.hp <= 0) {
                game.kills += 1
                game.chain = Math.min(CHAIN_MOST, game.chain + 1)
                game.chainSteps = CHAIN_STEPS
                game.score += Math.floor(((e.kind === 1 ? 45 : 20) * (10 + game.chain) + 5) / 10)
                if (game.fragments.length < MOST_FRAGMENTS)
                    game.fragments.push({ id: game.made++, x: e.x, y: e.y })
                game.said.push({ what: "kill", kind: e.kind, x: e.x, y: e.y })
            } else {
                game.said.push({ what: "struck", x: e.x, y: e.y })
            }
        }
    }
    game.shots = game.shots.filter((b) => b.life > 0).slice(-MOST_SHOTS)
    game.enemies = game.enemies.filter((e) => e.hp > 0)

    // Fragments within reach are drawn in, and collected at the ship.
    game.fragments = game.fragments.filter((f) => {
        const dx = game.x - f.x, dy = game.y - f.y, d = dx * dx + dy * dy
        if (d < game.magnet * game.magnet) {
            f.x += dx / 12
            f.y += dy / 12
        }
        if (d < 19 * 19) {
            game.score += 15
            game.said.push({ what: "fragment" })
            return false
        }
        return true
    })

    if (game.kills >= game.nextLevel && game.hull > 0) {
        game.level += 1
        game.nextLevel += 10 + game.level * 3
        game.phase = "choosing"
        game.said.push({ what: "evolved", level: game.level })
    }
    if (game.hull <= 0 || game.steps >= RUN_STEPS)
        game.phase = "over"
}

/** What the game came to: the score with five a second survived, counted each twelfth of a second. */
export function outcome(game) {
    return {
        score: game.score + Math.floor(game.steps / 12),
        kills: game.kills,
        level: game.level,
        hull: game.hull,
        seconds: Math.floor(game.steps / 60),
        survived: game.steps >= RUN_STEPS && game.hull > 0,
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
