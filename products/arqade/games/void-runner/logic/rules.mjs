// Void Runner's rules (DIRECTION P7.20, ADR-102 decision 2): a ship runs a track of three lanes into the collapse, and
// what the collapse throws rushes down the lanes at it. A hazard that reaches it in its lane breaches its hull; a gem
// that does is caught. Forty-five seconds survived completes the run, and three breaches end it. The rush quickens as
// the run goes on, and after a breach the hull holds against the next for a moment.
//
// The rules live here alone, so the game and ARQADE's service run the same ones (ADR-102's consequences): the QQ game's
// logic (logic/void-runner.qml) hands each event to them and draws what they say, and the service replays a run, the
// very file QQ records (`qqRun` 1), with `replay` below, to reach the outcome the player's game reached. Nothing here
// uses QQ or Qt. Each game's package keeps its own rules whole, random numbers and all, since a game reaches nothing
// outside its package (ADR-087 decision 2).
//
// They keep to what every machine computes alike: adding, subtracting, multiplying, dividing, `Math.floor` and whole
// numbers. No sine, cosine or power decides anything, since each system's library may differ in its last bits (QQ's
// README, "What a replay reaches"). Random numbers come from the run's seed through xoshiro128**, its state filled by
// splitmix32, in 32-bit whole numbers.
//
// A game is a plain object; each function takes it and changes it:
//   create(seed)          a game on its title, from a seed (a whole number)
//   click(game, button)   a screen button: "Play", "Play again", "Resume"; anything else is not the rules'
//   press(game, action)   an action pressed: "left" and "right" change lane at once; "pause" pauses and resumes
//   step(game)            one step of 1/60 s, which moves only while the game plays
//   outcome(game)         { score, gems, hull, seconds, survived, over }: what a run came to
//   replay(run)           a QQ run (seed, ticks, edges, clicks) played again: its outcome

/** One step's length in seconds: QQ steps 60 times a second whatever the frame rate. */
export const STEP = 1 / 60
/** The lanes, numbered 0 to 2 from the left; the ship starts in the middle one. */
export const LANES = 3
/** Steps a run lasts if the hull holds: forty-five seconds. */
export const RUN_STEPS = 2700
/** Breaches the hull takes before the run ends. */
export const HULL = 3
/** Steps between one thing thrown and the next: a little over 0.62 s. */
export const SPAWN_EVERY = 38
/** Steps after a breach in which the hull holds against another: 0.8 s. */
export const SHIELD_STEPS = 48
/** Points for each gem; a run also scores ten a second, counted each sixth of a second. */
export const GEM_POINTS = 100

/** How fast what is thrown comes, in lengths of the track a second, `steps` into the run. */
export function speed(steps) {
    return 0.29 + steps * STEP * 0.004
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

function begin(game) {
    game.lane = 1
    game.steps = 0
    game.spawnSteps = 0
    game.hull = HULL
    game.gems = 0
    game.shield = 0
    game.coming = []
    game.thrown = 0
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
    if (action === "left" && game.phase === "playing")
        game.lane = Math.max(0, game.lane - 1)
    else if (action === "right" && game.phase === "playing")
        game.lane = Math.min(LANES - 1, game.lane + 1)
    else if (action === "pause" && game.phase === "playing")
        game.phase = "paused"
    else if (action === "pause" && game.phase === "paused")
        game.phase = "playing"
}

/**
 * One step: the hull's moment of holding wears off, something new may be thrown, everything thrown comes on, and what
 * arrives meets the ship if it is in the ship's lane. What happened is in `said`: "gem", "hit" or "held" (a hazard met
 * while the hull holds), once for each.
 */
export function step(game) {
    if (game.phase !== "playing")
        return
    game.steps += 1
    game.said = []
    if (game.shield > 0)
        game.shield -= 1
    game.spawnSteps += 1
    if (game.spawnSteps >= SPAWN_EVERY) {
        game.spawnSteps = 0
        const lane = between(game.random, 0, LANES - 1)
        const gem = between(game.random, 0, 3) === 0
        game.coming.push({ id: game.thrown, lane: lane, gem: gem, z: 0 })
        game.thrown += 1
    }
    const on = speed(game.steps) * STEP
    const still = []
    for (const thing of game.coming) {
        thing.z += on
        if (thing.z <= 1) {
            still.push(thing)
            continue
        }
        if (thing.lane !== game.lane)
            continue
        if (thing.gem) {
            game.gems += 1
            game.said.push("gem")
        } else if (game.shield === 0) {
            game.hull -= 1
            game.shield = SHIELD_STEPS
            game.said.push("hit")
        } else {
            game.said.push("held")
        }
    }
    game.coming = still
    if (game.steps >= RUN_STEPS || game.hull <= 0)
        game.phase = "over"
}

/** What the game came to. */
export function outcome(game) {
    return {
        score: game.gems * GEM_POINTS + Math.floor(game.steps / 6),
        gems: game.gems,
        hull: game.hull,
        seconds: Math.floor(game.steps / 60),
        survived: game.steps >= RUN_STEPS && game.hull > 0,
        over: game.phase === "over",
    }
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
