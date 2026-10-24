// Synapse's rules (DIRECTION P7.20, ADR-102 decision 2): four nodes, Origin, Echo, Pulse and Aether, light one after
// another in a signal; then the player repeats it, node by node. Each signal repeated grows by one node, and a signal of
// eight repeated masters the game. One node out of turn loses the signal.
//
// The rules live here alone, so the game and ARQADE's service run the same ones (ADR-102's consequences): the QQ game's
// logic (logic/synapse.qml) hands each event to them and draws what they say, and the service replays a run, the very
// file QQ records (`qqRun` 1), with `replay` below, to reach the outcome the player's game reached. Nothing here uses QQ
// or Qt. Each game's package keeps its own rules whole, random numbers and all, since a game reaches nothing outside its
// package (ADR-087 decision 2).
//
// They keep to what every machine computes alike: whole numbers only, counted in steps. Random numbers come from the
// run's seed through xoshiro128**, its state filled by splitmix32, in 32-bit whole numbers.
//
// A game is a plain object; each function takes it and changes it:
//   create(seed)          a game on its title, from a seed (a whole number)
//   click(game, button)   a screen button: "Play", "Play again", "Resume"; anything else is not the rules'
//   press(game, action)   an action pressed: "node1" to "node4" while the signal is repeated; "pause" pauses and resumes
//   step(game)            one step of 1/60 s, which moves only while the game plays
//   lit(game)             the node the signal shows now (0 to 3), or -1
//   outcome(game)         { score, length, mastered, over }: what a run came to
//   replay(run)           a QQ run (seed, ticks, edges, clicks) played again: its outcome

/** One step's length in seconds: QQ steps 60 times a second whatever the frame rate. */
export const STEP = 1 / 60
/** The nodes, numbered 0 to 3, and their names. */
export const NODES = ["Origin", "Echo", "Pulse", "Aether"]
/** The signal's length that, repeated, masters the game. */
export const LONGEST = 8
/** Steps before a signal's first node lights; between one node's lighting and the next's; and how long each is lit. */
export const LEAD = 30
export const GAP = 42
export const SHOWN = 28
/** Points for mastering, and for a signal lost: each whole signal repeated, and each node of the last. */
export const MASTERED_POINTS = 1500
export const SIGNAL_POINTS = 150
export const NODE_POINTS = 20

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

/** The steps a signal of `length` takes to show, from its lead to the moment it is the player's to repeat. */
export function showing(length) {
    return LEAD + length * GAP
}

function watch(game) {
    game.stage = "watch"
    game.shown = 0
    game.cursor = 0
}

function begin(game) {
    game.signal = [between(game.random, 0, NODES.length - 1)]
    game.steps = 0
    game.said = []
    game.score = 0
    game.mastered = false
    watch(game)
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

/** The node an action names, or -1. */
function nodeOf(action) {
    return ["node1", "node2", "node3", "node4"].indexOf(action)
}

/**
 * An action pressed. A node pressed while the signal is the player's is judged at once: the next in turn moves the
 * cursor on, the last masters the signal (and the game, at eight) or grows it, and any other loses it. What happened is
 * in `said`: "right", "signal", "mastered" or "wrong", with the node pressed.
 */
export function press(game, action) {
    if (action === "pause" && game.phase === "playing") {
        game.phase = "paused"
        return
    }
    if (action === "pause" && game.phase === "paused") {
        game.phase = "playing"
        return
    }
    const node = nodeOf(action)
    if (node < 0 || game.phase !== "playing" || game.stage !== "repeat")
        return
    if (node !== game.signal[game.cursor]) {
        game.score = (game.signal.length - 1) * SIGNAL_POINTS + game.cursor * NODE_POINTS
        game.said.push({ what: "wrong", node: node })
        game.phase = "over"
        return
    }
    game.cursor += 1
    if (game.cursor < game.signal.length) {
        game.said.push({ what: "right", node: node })
        return
    }
    if (game.signal.length >= LONGEST) {
        game.score = MASTERED_POINTS
        game.mastered = true
        game.said.push({ what: "mastered", node: node })
        game.phase = "over"
        return
    }
    game.said.push({ what: "signal", node: node })
    game.signal.push(between(game.random, 0, NODES.length - 1))
    watch(game)
}

/**
 * One step: the signal shows itself, and once shown is the player's to repeat. `said` is emptied first, so what a press
 * says is read as it is pressed, before the step that follows it.
 */
export function step(game) {
    if (game.phase !== "playing")
        return
    game.steps += 1
    game.said = []
    if (game.stage === "watch") {
        game.shown += 1
        if (game.shown >= showing(game.signal.length)) {
            game.stage = "repeat"
            game.cursor = 0
        }
    }
}

/** The node the signal shows now, 0 to 3, or -1 between nodes and while it is the player's. */
export function lit(game) {
    if (game.stage !== "watch" || game.shown < LEAD)
        return -1
    const at = game.shown - LEAD
    const n = Math.floor(at / GAP)
    return n < game.signal.length && at - n * GAP < SHOWN ? game.signal[n] : -1
}

/** What the game came to. */
export function outcome(game) {
    return { score: game.score, length: game.signal.length, mastered: game.mastered, over: game.phase === "over" }
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
