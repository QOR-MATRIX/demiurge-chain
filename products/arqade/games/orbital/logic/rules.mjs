// Orbital's rules (DIRECTION P7.20, ADR-102 decision 2): a satellite circles a core, and a lock is made while it is
// inside the amber capture arc. Ten locks complete the orbit; three misses lose it. Each lock moves the arc and makes
// the next orbit faster.
//
// The rules live here alone, so the game and ARQADE's service run the same ones (ADR-102's consequences): the QQ game's
// logic (logic/orbital.qml) hands each event to them and draws what they say, and the service replays a run, the very
// file QQ records (`qqRun` 1), with `replay` below, to reach the outcome the player's game reached. Nothing here uses
// QQ or Qt.
//
// They keep to what every machine computes alike: adding, subtracting, multiplying, dividing, `Math.floor` and whole
// numbers. No sine, cosine or arctangent decides anything, since each system's library may differ in its last bits
// (QQ's README, "What a replay reaches"); where the satellite is drawn is the game's own business. Random numbers come
// from the run's seed through xoshiro128**, its state filled by splitmix32, in 32-bit whole numbers.
//
// A game is a plain object; each function takes it and changes it:
//   create(seed)          a game on its title, from a seed (a whole number)
//   click(game, button)   a screen button: "Play", "Play again", "Resume"; anything else is not the rules'
//   press(game, action)   an action pressed: "lock", judged on the next step, or "pause", which pauses and resumes
//   step(game)            one step of 1/60 s, which moves only while the game plays
//   outcome(game)         { locks, misses, score, over }: what a run came to
//   replay(run)           a QQ run (seed, ticks, edges, clicks) played again: its outcome

/** One step's length in seconds: QQ steps 60 times a second whatever the frame rate. */
export const STEP = 1 / 60
/** A whole turn, in radians. */
export const TURN = 6.283185307179586
/** Half the capture arc's width, in radians: a lock within this of the arc's middle holds. */
export const ARC = 0.29
/** Locks that complete the orbit, and misses that lose it. */
export const LOCKS_TO_WIN = 10
export const MISSES_TO_LOSE = 3
/** Points for each lock. */
export const POINTS = 200

/** How fast the satellite turns, in radians a second, after `locks` locks. */
export function speed(locks) {
    return 1.4 + locks * 0.22
}

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

/** Where the arc's middle goes next: one of 628 places round the orbit, a hundredth of a radian apart. */
function nextTarget(game) {
    return between(game.random, 0, 627) / 100
}

/** Whether a lock at `angle` holds on an arc whose middle is at `target`: how far apart they are round the orbit. */
export function within(angle, target) {
    let apart = angle - target
    apart -= Math.floor(apart / TURN) * TURN
    return Math.min(apart, TURN - apart) < ARC
}

function begin(game) {
    game.angle = 0
    game.locks = 0
    game.misses = 0
    game.steps = 0
    game.lockAsked = false
    game.said = ""
    game.target = nextTarget(game)
    game.phase = "playing"
}

/** A game on its title screen, from `seed`. */
export function create(seed) {
    const game = { seed: seed >>> 0, random: seeded(seed), phase: "title" }
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
}

/** An action pressed. */
export function press(game, action) {
    if (action === "lock" && game.phase === "playing")
        game.lockAsked = true
    else if (action === "pause" && game.phase === "playing")
        game.phase = "paused"
    else if (action === "pause" && game.phase === "paused")
        game.phase = "playing"
}

/** One step: a lock asked for is judged where the satellite is, and then it moves on. */
export function step(game) {
    if (game.phase !== "playing")
        return
    game.steps += 1
    game.said = ""
    if (game.lockAsked) {
        game.lockAsked = false
        if (within(game.angle, game.target)) {
            game.locks += 1
            game.target = nextTarget(game)
            game.said = "locked"
        } else {
            game.misses += 1
            game.said = "missed"
        }
        if (game.locks >= LOCKS_TO_WIN || game.misses >= MISSES_TO_LOSE) {
            game.phase = "over"
            return
        }
    }
    game.angle += speed(game.locks) * STEP
    if (game.angle >= TURN)
        game.angle -= TURN
}

/** What the game came to. */
export function outcome(game) {
    return { locks: game.locks, misses: game.misses, score: game.locks * POINTS, over: game.phase === "over" }
}

/**
 * A QQ run played again: { seed, ticks, edges: [[tick, [["pressed" or "released", action], ...]], ...],
 * clicks: [[tick, [button, ...]], ...] }, as QQ writes it (format 1). Each tick takes its clicks, then its presses,
 * then steps, as QQ's scene does. Its outcome, which is the one the game reached if the run is the game's.
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
        for (const [what, action] of edges.get(tick) || [])
            if (what === "pressed")
                press(game, action)
        step(game)
    }
    return outcome(game)
}
