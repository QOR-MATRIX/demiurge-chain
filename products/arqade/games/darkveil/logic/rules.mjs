// SPDX-License-Identifier: Apache-2.0
// Darkveil's rules (QOR OS's QOR-0017): a valley gone dark. Three beacons
// went out when the Hollow King rose behind his gate. Three wardens stand over them. A hero walks out of the village of
// Hearth with a rusted blade, takes errands from the people there, cuts through what waits in the meadow, the Mirewood,
// the Ashen Hold and the High Pass, breaks each warden and lights each beacon. With all three burning the gate opens,
// and the king waits behind it. The king ended wins the run; a third fall ends it, and so does the veil closing over
// the valley after three quarters of an hour.
//
// The rules live here alone, so the game and ARQADE's service run the same ones (Demiurge's ADR-102): the QQ game's logic
// (logic/darkveil.qml) hands each event to them and draws what they say, and the service replays a run, the very file
// QQ records (`qqRun` 1), with `replay` below, to reach the outcome the player's game reached. Nothing here uses QQ or
// Qt, and nothing here comes from outside this package (ADR-087 decision 2).
//
// They keep to what every machine computes alike: adding, subtracting, multiplying, dividing, `Math.floor`, whole
// numbers, and `Math.sqrt`, which ECMAScript specifies exactly (IEEE 754's correctly rounded square root). No sine,
// cosine, power or logarithm decides anything: the eight ways a view can face, and a swing's arc, are numbers written
// here. Random numbers come from the run's seed through xoshiro128**, its state filled by splitmix32. The valley is
// measured in tiles of 40 units, with y running south.
//
// The world's state is kept apart from each player's (QOR-0017 decision 4), so a server could step one world for many:
// `game.world` holds what is out there (enemies, what lies on the ground, the beacons, the gate), `game.players` holds
// each player's own (where they are, their health, their errands), and every input names its player. QQ plays one.
//
// A game is a plain object; each function takes it and changes it:
//   create(seed, players)      a game on its title, from a seed (a whole number), for one player unless told otherwise
//   click(game, button, who)   a screen button: "Play", "Play again", "Resume", "Say 1" to "Say 6"
//   press(game, action, who)   an action pressed: "up", "down", "left", "right" held to move, as the view faces;
//                              "attack" (held, it swings again), "roll", "bolt", "nova", "potion", "talk"; "turnLeft",
//                              "turnRight" turn the view an eighth; "say1" to "say6" answer; "pause" pauses and resumes
//   release(game, action, who) an action let go
//   step(game)                 one step of 1/60 s, which moves only while the game plays
//   offers(game, who)          what a player talking may say
//   objective(game, who)       what they should do next, and where
//   outcome(game, who)         { score, kills, gold, level, health, falls, beacons, errands, seconds, won, over }
//   replay(run)                a QQ run (seed, ticks, edges, clicks) played again: its outcome

/** One step's length in seconds: QQ steps 60 times a second whatever the frame rate. */
export const STEP = 1 / 60
/** The valley: its tiles, each this many units across. */
export const TILE = 40
export const COLS = 64
export const ROWS = 64
/** Steps a run may last before the veil closes over the valley: three quarters of an hour. */
export const RUN_STEPS = 162000
/** Steps from the king's end to the run's, while the valley lights up. */
export const CLEARING_STEPS = 180
/** Falls that end a run. Before the last, a hero wakes in Hearth, lighter of a quarter of their gold. */
export const LIVES = 3

/** The hero: half its width against what stands, its speed a step, and how it starts. */
export const HERO_R = 12
export const HERO_SPEED = 2.4
export const HEALTH = 120
export const MANA = 60
export const POWER = 10
/** A swing of the three in a chain: its length in steps, the step of it that strikes (counted down), its reach and its
 * share of the hero's power, in hundredths. The third is slow, wide and heavy. */
export const SWINGS = [
    { steps: 24, strikes: 16, reach: 54, share: 100 },
    { steps: 24, strikes: 16, reach: 54, share: 115 },
    { steps: 36, strikes: 18, reach: 68, share: 180 },
]
/** Steps after a swing in which the next carries on the chain. */
export const CHAIN_STEPS = 14
/** cos 60°: an enemy is in the swing's arc while the angle to it from where the hero faces is no wider. */
const ARC_COS = 0.5
/** How near an enemy must be for a swing to turn toward it. */
export const AIM_REACH = 110
/** A roll: its length and speed a step, and the steps before the next. Nothing wounds a hero rolling. */
export const ROLL_STEPS = 14
export const ROLL_SPEED = 6.5
export const ROLL_COOL = 55
/** A bolt: what it costs, the steps to cast it and the step that looses it (counted down), how fast and how long it flies,
 * its share of the hero's power, and how far it looks for something to fly at. */
export const BOLT_COST = 12
export const BOLT_CAST = 16
export const BOLT_LOOSES = 8
export const BOLT_SPEED = 7
export const BOLT_STEPS = 70
export const BOLT_SHARE = 130
export const BOLT_AIM = 340
/** A nova: what it costs, the level it comes at, the steps to cast it and the step that strikes, its reach and share. */
export const NOVA_COST = 36
export const NOVA_LEVEL = 3
export const NOVA_CAST = 24
export const NOVA_STRIKES = 10
export const NOVA_REACH = 112
export const NOVA_SHARE = 220
/** Mana comes back one every this many steps. */
export const MANA_EVERY = 10
/** Potions: how many at the start and at most, what one heals (a share of most health, in hundredths), and the steps
 * before the next. */
export const POTIONS = 3
export const POTIONS_MOST = 6
export const POTION_HEALS = 50
export const POTION_COOL = 60
/** The steps after a wound in which no other wounds. */
export const HURT_HOLD = 30
/** How near a hero must be to talk, to pick something up, and to light a beacon whose warden has fallen. */
export const TALK_REACH = 60
export const PICK_REACH = 30
export const BEACON_REACH = 70
/** How near an enemy wakes, how near its waking wakes others, how far it follows from home before going back, and how
 * far from every hero it stops altogether. */
export const WAKE_REACH = 230
export const PACK_REACH = 170
export const LEASH = 620
export const ACTIVE_REACH = 900
/** How many tiles an enemy that has lost sight of a hero looks along the valley's ways. */
const PATH_TILES = 26
/** A shot from afar: its speed a step and its life. */
export const THORN_SPEED = 4
export const THORN_STEPS = 160
/** At most this many enemies at once, shots of either side, and things on the ground. */
export const MOST_ENEMIES = 150
export const MOST_SHOTS = 40
export const MOST_ITEMS = 60
/** What the blade and the ward are, in order, what each adds, and what the smith asks for it. */
export const BLADES = [
    { name: "Rusted blade", power: 0, cost: 0 },
    { name: "Hearth steel", power: 5, cost: 150 },
    { name: "Tempered edge", power: 11, cost: 400 },
    { name: "Veilbane", power: 18, cost: 900 },
]
export const WARDS = [
    { name: "Worn leathers", ward: 0, cost: 0 },
    { name: "Ring mail", ward: 2, cost: 120 },
    { name: "Scaled plate", ward: 5, cost: 350 },
    { name: "Beacon plate", ward: 8, cost: 750 },
]
/** What the healer asks for a potion, and how much of each thing an errand wants. */
export const POTION_COST = 40
export const HERBS_WANTED = 5
export const ORE_WANTED = 4

/**
 * The enemies, by kind: health, speed a step, half their width (and against what stands at most 18, so all fit a way),
 * the wound they deal, how near a blow lands (0: from afar), the steps they wind up and recover, experience and score.
 * A warden's and the king's blow is a slam all round them.
 */
export const KINDS = [
    { name: "wolf", health: 30, speed: 2.0, r: 13, wound: 7, reach: 22, windup: 14, recover: 28, xp: 6, score: 15 },
    { name: "shade", health: 24, speed: 1.6, r: 11, wound: 9, reach: 24, windup: 18, recover: 32, xp: 8, score: 20 },
    { name: "thornling", health: 55, speed: 1.0, r: 14, wound: 11, reach: 0, windup: 30, recover: 70, xp: 10, score: 25 },
    { name: "revenant", health: 110, speed: 1.3, r: 15, wound: 18, reach: 28, windup: 28, recover: 36, xp: 16, score: 40 },
    { name: "bonecaster", health: 50, speed: 1.2, r: 13, wound: 15, reach: 0, windup: 36, recover: 80, xp: 14, score: 40 },
    { name: "great wolf", health: 420, speed: 2.2, r: 22, wound: 16, reach: 30, windup: 20, recover: 30, xp: 60, score: 300 },
    { name: "warden", health: 700, speed: 1.2, r: 26, wound: 20, reach: 84, windup: 50, recover: 60, xp: 80, score: 600 },
    { name: "hollow king", health: 3800, speed: 1.4, r: 30, wound: 34, reach: 110, windup: 56, recover: 56, xp: 400,
      score: 5000 },
]
export const WOLF = 0, SHADE = 1, THORNLING = 2, REVENANT = 3, BONECASTER = 4, GREAT_WOLF = 5, WARDEN = 6, KING = 7
/** Each region's enemies: their health and wound, in hundredths, and how much gold they leave. */
export const REGIONS = {
    meadow: { health: 80, wound: 85, gold: 1 },
    mire: { health: 125, wound: 115, gold: 2 },
    ash: { health: 165, wound: 140, gold: 3 },
    high: { health: 200, wound: 165, gold: 4 },
    crown: { health: 240, wound: 190, gold: 4 },
}
/** The three beacons, in the order the Elder names them: where each stands, and its region. */
export const BEACONS = [
    { name: "Mirewood", place: "mireBeacon", region: "mire", summons: THORNLING },
    { name: "Ashen Hold", place: "ashBeacon", region: "ash", summons: REVENANT },
    { name: "High Pass", place: "highBeacon", region: "high", summons: BONECASTER },
]
/** The people of Hearth: who they are and where they stand. */
export const PEOPLE = {
    elder: { name: "Elder Maren", place: "elder" },
    healer: { name: "Ise the healer", place: "healer" },
    smith: { name: "Tolv the smith", place: "smith" },
    hunter: { name: "Brann the hunter", place: "hunter" },
}

// WORLD BEGIN: written by make_world.py; change it there.
export const WORLD = [
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RR.RRR'''R'R'RRR''RR'''R'R'RR.''R'RR'R''R'R;R';R;RRRR;RRRRR.RRRR",
    "RRR''''TT'''R''''''''''''''''R.'T''''''''''RRRRRRRRR;T.......RRR",
    "RR..'T'''R''''''TR'''T''''T''T''''''R'''T'RRRRRRRRRRR;;;....T.RR",
    "RR.'R''''''TT'''''''''''''''R'R'R''''''''RRR''';;;;RRR;R;....RRR",
    "RRR'''''''''''T'..'.''''''''TR'''''''R''RRR';;;;;;;;RRR;.;....RR",
    "RRR'''''''''T''.''.'.''T'R'''''''''''''RRR';;;;;;;;;;RRR;;..T.RR",
    "RR'R''''R'''''''.'''.'''''''''''T''TT''RRR;;;;;;;;;;;;RR;.;...RR",
    "RRR'''T''''.'''''''''T'''''''T'''''''''RR';;;;;;;;;;;;RR;;....RR",
    "RRR'''''''.''.''''''TR'''''R''R'''R''''RR';;;;;;;;;;;;RR;;;...RR",
    "RR'T'''T'''''''''''T''.'''''R'R''''RT'RRR;;;;;;;;;;;;;RRR;;..RRR",
    "RR'T''R'''''',,''''''R'''''RRT''T''''''RR';;;;;;;;;;;;RR;.;...RR",
    "RRR''RR'''''',,'''''''''''.''''''R''T''RR';;;;;;;;;;;;RR;;....RR",
    "RRR'''''T'''',,.'''''''..''''''''''''R'RR';;;;;;;;;;;;RR;;..TRRR",
    "RRR''T''''''',,'''''''.''''''TR'''''R''RRR';;;;;;;;;;RRR;R...RRR",
    "RRR''''T''''',,,'''''R''''TT'''''T'R''''RRRR;;;;;;;;RRR;;....RRR",
    "RRR'''''.''''',,''''''T'''''T''RT'''''RRRRRR',,;;';RRR;;R....RRR",
    "RR'''''.'''T'',,'''''''R'RT'R'''''R'.'''''RRRGGRRRRRR;;.;....RRR",
    "RRR'''''.''''',,'''''''''R''''''''''''..'''RGGGRRRRR;R........RR",
    "RRR''''''''.'T,,,''''T'''T'R'''''''''''''''',,RRRR;;;.........RR",
    "RRR.''''.'''''',,.'''''''T'''''''RT'.''.'''',,.;;;;;.T.......RRR",
    "RR..''''.'.'''',,'''''''''~~~~~~~~~'.'''''.',,.;;T......~~~~~~RR",
    "RR.''''R''R'''R,,'.'''''~~~~~~~~~~~~~.T''..',,....T..~~~~~~~~RRR",
    "RRT.T.'''''T''',,,..''R~~~~~~~~~~~~~~~...'..,,.T...~~~~~...T.RRR",
    "RR..'..''''''''',,'''''~~~~~~~~~~~~~~~......,,;;.~~~~.;;.T.R.TRR",
    "RRR....'.''''''',,'''R'~~~~~~~~~~~~~~~~~...,,,.~~~~;;;;;.;.R.RRR",
    "RR......'''''''',,,'R''.~~~~~~~~~~~~~..~~~.,,..~~;;R;;;;...;..RR",
    "RRR......'.''.''',,'''...T~~~~~~~~~......~=,=~~R;;;;T;;;;;;...RR",
    "RRR........'..TR',,,''...................;,,R;;;.;;;;;;;R;;.;;RR",
    "RRR.....,,,,,T....,,.T...T...R...........,,,;;;;;;;;;;;;;;;;;RRR",
    "RR..R...,,,,,,,,,:,,,..............T...,,,,;;.;;;;;;;;;;;;;;;.RR",
    "RRTT......::,,,,,,,,,................,,,,,,;;;;.;;;;;;;;;;;RR;RR",
    "RR......:.::::::,,,,,,,,,,,R.T....,,,,,,T;;;T;;;;;;;;;;,,;;;;RRR",
    "RRR...:T:T:TTT::T:T:,,,,,,,,,,,,,,,,,,;.;;;;####.###..,,,..#;RRR",
    "RRR..T:::T:::::::::~:.::..,,,,,,,,,,,,,.;;;;.R;;;;;;;,,,;;..;RRR",
    "RRR.:::::T::::TTTTT:::.:...T,,,,,,,,,,,,,,,,,,;;;R;;,,,;;;;;;RRR",
    "RRRT:::T::::..:::::T.T::::.,,,,,......,,,,,,,,,,,,,,,,;;;;;.R;RR",
    "RRR:::::~::::::.:.T...::.,,,,.,,.....;;;;;;;.,,,,,,,,,;;;;;.;;RR",
    "RR.:::::::T~::.:::TT.::,,,,,..,,..T.....;;;;.;;;.;;;;;;;;;;.;RRR",
    "RR::T:::T:::.:.:::::TT,,,,:...,,......;;;;;;..;;;;;;;;;;;;;;;;RR",
    "RR~:::TTT::::::.:.:T,,,,......,,.......;;;;;.;;;;;;;;;;;.;;.;RRR",
    "RRTT::::.T:T::..:::,,,,:......,,.......;;;;;#;;;;;;;;;;;;;;.R;RR",
    "RRTT:::::.::T:::::,,,T:###....,,..###..;;;;;#;;;;;.;;;;;T;;#;RRR",
    "RRR::T:.::::TT:T:,,,::.###....,,..###..;;;;;#;;;;;;;;;;;;;;#;RRR",
    "RR::::.:::.:::T:,,,::::.......,,........;;;;#;;..;;;;;;;R;;#;RRR",
    "RR:T::.:.:..T::,,,:..::.......,,..........;;;;;;;R;;;;;;;;;#;RRR",
    "RRR:~T:..:.::T,,,:::::........,,..........;T######;#.#;.#;#R.RRR",
    "RRR:::.::T:.~,,,::.::::.......,,.........T.;;;;;;;;;;;;;;;.;.RRR",
    "RRR::::::T::,,,TT::.:T###...........###...T...;;;;;;;;;;..;;.TRR",
    "RR:::T.T:.:.,,:TT:::::###....###....###..T..;.T;;.;;;.;T......RR",
    "RRR::T::.::,,,:::::::T~T.....###......T....T.....;.;...;;.....RR",
    "RR..~::.:.,,,:::.::.::::......................................RR",
    "RRR::T::::,,.::.:::.:T::T....................................RRR",
    "RR..TT~.:.,,::::::.:TTT:.:...............T.................T..RR",
    "RR.:.:T:..:.:::::::.T:T:T....T.T.....T.............T....TT....RR",
    "RR...:.:::::.::T:::::.:::.....T............TT................RRR",
    "RR.....:.::TT::.:~::::................T.T...T.........T.......RR",
    "RR...T...T:::T:.:.:R.T.........T..........R..........R..T.....RR",
    "RR.....T....:::....T..............T.......T...............T..RRR",
    "RRR........T...T.............................T...........T...RRR",
    "RRRRR...RT..R..R.RRR.R.RRR...RRRRRRRRRRRRRRRRR.RR.RRR.....R..RRR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
    "RRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRRR",
]
export const PLACES = {"hearth": [30, 47], "elder": [30, 44], "healer": [25, 48], "smith": [36, 47], "hunter": [31, 53], "crossroads": [31, 35], "den": [8, 30], "mireBeacon": [10, 53], "ashBeacon": [55, 33], "highBeacon": [13, 12], "gate": [44, 20], "crown": [47, 11]}
export const CAMPS = [[24, 38, "meadow", 4, [0]], [39, 40, "meadow", 4, [0]], [38, 30, "meadow", 3, [0, 1]], [22, 33, "meadow", 3, [0, 1]], [20, 46, "mire", 6, [2, 1]], [15, 40, "mire", 6, [2, 1]], [8, 45, "mire", 5, [2]], [17, 53, "mire", 6, [2, 1]], [46, 39, "ash", 6, [3]], [52, 44, "ash", 6, [3, 4]], [57, 40, "ash", 6, [3, 4]], [48, 31, "ash", 6, [3, 4]], [17, 22, "high", 6, [0, 1, 4]], [8, 20, "high", 6, [1, 3]], [24, 13, "high", 7, [3, 4]], [18, 7, "high", 6, [1, 3, 4]], [44, 25, "crown", 6, [1, 3, 4]], [37, 20, "crown", 6, [3, 4]]]
export const HERBS = [[12, 37], [19, 50], [6, 49], [14, 57], [20, 38], [18, 42], [11, 47]]
export const ORE = [[47, 45], [55, 46], [59, 36], [50, 28], [43, 35], [54, 38]]
export const HOUSES = [[24, 44], [35, 44], [23, 50], [37, 50], [30, 51]]
// WORLD END

/** The square root of a half: a diagonal step's share of each axis. */
const HALF_ROOT = 0.7071067811865476
/** Where each of the eight views faces, in the valley's axes: a whole number of eighths clockwise from north. */
export const VIEWS = [[0, -1], [HALF_ROOT, -HALF_ROOT], [1, 0], [HALF_ROOT, HALF_ROOT], [0, 1], [-HALF_ROOT, HALF_ROOT],
                      [-1, 0], [-HALF_ROOT, -HALF_ROOT]]

/** What stands on each tile: 0 nothing, 1 water (which stops feet, not sight), 2 what stops both, 3 the gate. */
const GROUND = []
for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
        const tile = WORLD[r][c]
        GROUND.push(tile === "~" ? 1 : tile === "T" || tile === "R" || tile === "#" ? 2 : tile === "G" ? 3 : 0)
    }
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

/** True `percent` times in a hundred. */
function chance(state, percent) {
    return between(state, 0, 99) < percent
}

/** The experience that reaches each level after the first: 50, 150, 300, 500 and on. */
export function nextXp(level) {
    return 25 * level * (level + 1)
}

/** `value` raised by a share of hundredths, in whole numbers. */
export function scaled(value, share) {
    return Math.floor((value * share) / 100)
}

/** The middle of a place, in units. */
export function placeAt(name) {
    const [c, r] = PLACES[name]
    return { x: (c + 0.5) * TILE, y: (r + 0.5) * TILE }
}

// ── The valley ───────────────────────────────────────────────────────────────────────────────────────────────────────

/** Whether feet can stand on the tile at column `c` and row `r` (outside the valley they cannot). */
export function walkable(game, c, r) {
    if (c < 0 || r < 0 || c >= COLS || r >= ROWS)
        return false
    const at = GROUND[r * COLS + c]
    return at === 0 || (at === 3 && game.world.gate)
}

function standsAt(game, x, y) {
    return !walkable(game, Math.floor(x / TILE), Math.floor(y / TILE))
}

/** Whether what stands at (x, y) stops a line of sight, or a shot. Water does not. */
function hidesAt(game, x, y) {
    const c = Math.floor(x / TILE), r = Math.floor(y / TILE)
    if (c < 0 || r < 0 || c >= COLS || r >= ROWS)
        return true
    const at = GROUND[r * COLS + c]
    return at === 2 || (at === 3 && !game.world.gate)
}

/** Whether a square of half-width `h` at (x, y) touches what stands. `h` is under half a tile, so its corners are enough. */
function blocked(game, x, y, h) {
    return standsAt(game, x - h, y - h) || standsAt(game, x + h, y - h) || standsAt(game, x - h, y + h)
           || standsAt(game, x + h, y + h)
}

/** How far round a corner a body slips: a step along a way that a corner blocks by no more than this eases it aside,
 * toward the open side, as far as the step would have gone. */
const SLIP = 16

/** Moves `body` by (dx, dy), each axis on its own, so it slides along what stands; a blocked step tries a half, then a
 * quarter, so it comes close. A step along one axis that a corner blocks slips round it. */
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

/** Whether nothing that hides lies between two places: the line between them looked along every ten units. */
function inSight(game, ax, ay, bx, by) {
    const dx = bx - ax, dy = by - ay
    const n = Math.floor(Math.sqrt(dx * dx + dy * dy) / 10) + 1
    for (let i = 1; i < n; i++) {
        if (hidesAt(game, ax + (dx * i) / n, ay + (dy * i) / n))
            return false
    }
    return true
}

function near(ax, ay, bx, by, reach) {
    const dx = ax - bx, dy = ay - by
    return dx * dx + dy * dy < reach * reach
}

/** How many tiles each tile within `PATH_TILES` of a hero's is from it, walking: -1 further off, and for what stands.
 * Worked out when an enemy first needs it after the hero's tile, or the gate, has changed. */
function paths(game, p) {
    const c = Math.floor(p.x / TILE), r = Math.floor(p.y / TILE), from = r * COLS + c
    const key = game.world.gate ? from + COLS * ROWS : from
    if (p.pathsKey === key)
        return p.paths
    const far = new Array(COLS * ROWS).fill(-1)
    far[from] = 0
    const queue = [from]
    for (let head = 0; head < queue.length; head++) {
        const at = queue[head], ac = at % COLS, ar = (at - ac) / COLS
        if (far[at] >= PATH_TILES)
            break
        for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
            const nc = ac + dc, nr = ar + dr
            if (walkable(game, nc, nr) && far[nr * COLS + nc] < 0) {
                far[nr * COLS + nc] = far[at] + 1
                queue.push(nr * COLS + nc)
            }
        }
    }
    p.paths = far
    p.pathsKey = key
    return far
}

// ── A run ────────────────────────────────────────────────────────────────────────────────────────────────────────────

/** An enemy of `kind` from `region` at (x, y), which goes home there when it loses a hero. */
function enemy(game, kind, x, y, region, hold) {
    const world = game.world, k = KINDS[kind], r = REGIONS[region]
    const health = kind === KING ? k.health : scaled(k.health, r.health)
    world.enemies.push({ id: world.made++, kind: kind, x: x, y: y, homeX: x, homeY: y, region: region, hold: hold,
                         health: health, most: health, wound: kind === KING ? k.wound : scaled(k.wound, r.wound),
                         awake: false, woke: false, act: "move", timer: 0, cool: 0, fx: 0, fy: 1, kx: 0, ky: 0, struck: 0,
                         aimX: x, aimY: y, by: -1 })
}

/** The valley's enemies, from the seed: each camp's kinds and places, and the great ones where they stand. */
function populate(game) {
    const world = game.world
    for (const [c, r, region, count, kinds] of CAMPS) {
        for (let n = 0; n < count; n++) {
            const kind = kinds[between(game.random, 0, kinds.length - 1)]
            let x = (c + 0.5) * TILE, y = (r + 0.5) * TILE
            for (let tries = 0; tries < 8; tries++) {
                const tx = (c + 0.5 + between(game.random, -24, 24) / 8) * TILE
                const ty = (r + 0.5 + between(game.random, -24, 24) / 8) * TILE
                if (!blocked(game, tx, ty, 18)) {
                    x = tx
                    y = ty
                    break
                }
            }
            enemy(game, kind, x, y, region, -1)
        }
    }
    const den = placeAt("den")
    enemy(game, GREAT_WOLF, den.x, den.y, "mire", -1)
    BEACONS.forEach((beacon, i) => {
        const at = placeAt(beacon.place)
        enemy(game, WARDEN, at.x, at.y + TILE, beacon.region, i)
    })
    const crown = placeAt("crown")
    enemy(game, KING, crown.x, crown.y, "crown", -1)
}

function noAsks() {
    return { attack: false, roll: false, bolt: false, nova: false, potion: false, talk: false, turnLeft: false,
             turnRight: false, up: false, down: false, say1: false, say2: false, say3: false, say4: false, say5: false,
             say6: false }
}

/** A hero in Hearth, as they start. */
function newHero(id) {
    const at = placeAt("hearth")
    return { id: id, x: at.x + id * TILE, y: at.y, facing: [0, -1], view: 0, out: false,
             held: { up: false, down: false, left: false, right: false, attack: false }, asked: noAsks(),
             health: HEALTH, most: HEALTH, mana: MANA, manaMost: MANA, power: POWER, blade: 0, ward: 0,
             potions: POTIONS, potionsMost: POTIONS_MOST, gold: 0, xp: 0, level: 1, kills: 0, score: 0, falls: 0,
             swing: 0, chain: 0, chainLeft: 0, cast: 0, casting: "", aimX: 0, aimY: 0, roll: 0, rollCool: 0, potionCool: 0,
             hurt: 0, manaIn: MANA_EVERY, talk: null, errands: { king: 0, herbs: 0, ore: 0, wolf: 0 },
             herbs: new Array(HERBS.length).fill(false), ore: new Array(ORE.length).fill(false), paths: null, pathsKey: -1 }
}

function begin(game) {
    game.world = { enemies: [], shots: [], bolts: [], items: [], beacons: [false, false, false],
                   wardens: [false, false, false], gate: false, wolfDown: false, kingDown: false, made: 0 }
    game.players = []
    for (let i = 0; i < game.count; i++)
        game.players.push(newHero(i))
    game.steps = 0
    game.clearing = 0
    game.won = false
    game.said = []
    populate(game)
    game.phase = "playing"
}

/** A game on its title screen, from `seed`, for `count` players (one, as QQ plays it). */
export function create(seed, count) {
    const game = { seed: seed >>> 0, random: seeded(seed), phase: "title", count: count === undefined ? 1 : count, runs: 0 }
    begin(game)
    game.phase = "title"
    return game
}

/** A screen button pressed: the phase's, or one of what a player talking may say. */
export function click(game, button, who) {
    const p = game.players[who === undefined ? 0 : who]
    if (button === "Play" && game.phase === "title")
        game.phase = "playing"
    else if (button === "Play again" && game.phase === "over") {
        game.runs += 1
        begin(game)
    } else if (button === "Resume" && game.phase === "paused")
        game.phase = "playing"
    else if (button.startsWith("Say ") && game.phase === "playing" && p.talk) {
        p.asked["say" + button.substring(4)] = true
        return
    } else
        return
    // What was pressed on a screen, the title's or the pause's, does nothing once play starts.
    for (const hero of game.players)
        hero.asked = noAsks()
}

/** An action pressed. Moving and attacking are held whatever the phase, so a key held through a pause still counts after
 * it; a press of the others is kept for the next step, so a press let go before it still counts. */
export function press(game, action, who) {
    const p = game.players[who === undefined ? 0 : who]
    if (action in p.held)
        p.held[action] = true
    if (action in p.asked)
        p.asked[action] = true
    else if (action === "pause" && game.phase === "playing")
        game.phase = "paused"
    else if (action === "pause" && game.phase === "paused")
        game.phase = "playing"
}

/** An action let go. */
export function release(game, action, who) {
    const p = game.players[who === undefined ? 0 : who]
    if (action in p.held)
        p.held[action] = false
}

// ── Talking ──────────────────────────────────────────────────────────────────────────────────────────────────────────

function beaconsLit(game) {
    return game.world.beacons.filter((lit) => lit).length
}

function herbsHeld(p) {
    return p.herbs.filter((had) => had).length
}

function oreHeld(p) {
    return p.ore.filter((had) => had).length
}

/** What a person says as a hero comes to them. */
function greeting(game, p, who) {
    const lit = beaconsLit(game)
    if (who === "elder") {
        if (game.world.gate)
            return "All three burn. The gate in the north stands open, and he knows you are coming. End him."
        if (p.errands.king === 0)
            return "The beacons went dark when the Hollow King rose, and the veil has crept down the valley since. "
                   + "A warden stands over each. Will you break them, and give the valley its light?"
        return lit + " of three burn. The Mirewood lies west, the Ashen Hold east, the High Pass north. Rest with Ise, "
               + "and see Tolv about your blade, before you go."
    }
    if (who === "healer") {
        if (p.errands.herbs === 1 && herbsHeld(p) < HERBS_WANTED)
            return herbsHeld(p) + " of " + HERBS_WANTED + " moonherbs. They grow pale in the Mirewood's clearings."
        return "Sit a while. What the veil takes I can give back, and for the price of the herbs I can bottle it."
    }
    if (who === "smith") {
        if (p.errands.ore === 1 && oreHeld(p) < ORE_WANTED)
            return oreHeld(p) + " of " + ORE_WANTED + " lumps of ironroot. It comes up through the ash, in the Hold."
        return "That blade will not cut a warden. Bring me gold, or ironroot from the Hold, and I can mend that."
    }
    if (p.errands.wolf === 1 && game.world.wolfDown)
        return "I heard it fall from here. You have done what I could not."
    if (p.errands.wolf === 1)
        return "It denned up in the west, past the meadow. Bigger than the rest, and it does not run from a blade."
    return "A great wolf drives the packs down on us. I have lost three hounds to it, and two good friends."
}

function offer(id, words, able) {
    return { id: id, words: words, able: able === undefined ? true : able }
}

/** What a player talking may say: each with whether they can say it now. The last is always to leave. */
export function offers(game, who) {
    const p = game.players[who === undefined ? 0 : who]
    if (!p.talk)
        return []
    const list = []
    if (p.talk.who === "elder") {
        if (p.errands.king === 0)
            list.push(offer("beacons", "I will light them."))
    } else if (p.talk.who === "healer") {
        list.push(offer("rest", "Rest, and be whole."))
        list.push(offer("potion", "A potion (" + POTION_COST + " gold).", p.gold >= POTION_COST && p.potions < p.potionsMost))
        if (p.errands.herbs === 0)
            list.push(offer("herbs", "I will look for moonherbs."))
        else if (p.errands.herbs === 1)
            list.push(offer("herbs done", "Here are your moonherbs.", herbsHeld(p) >= HERBS_WANTED))
    } else if (p.talk.who === "smith") {
        if (p.blade + 1 < BLADES.length) {
            const next = BLADES[p.blade + 1]
            list.push(offer("blade", next.name + " (" + next.cost + " gold).", p.gold >= next.cost))
        }
        if (p.ward + 1 < WARDS.length) {
            const next = WARDS[p.ward + 1]
            list.push(offer("ward", next.name + " (" + next.cost + " gold).", p.gold >= next.cost))
        }
        if (p.errands.ore === 0)
            list.push(offer("ore", "I will bring you ironroot."))
        else if (p.errands.ore === 1)
            list.push(offer("ore done", "Here is your ironroot.", oreHeld(p) >= ORE_WANTED))
    } else if (p.talk.who === "hunter") {
        if (p.errands.wolf === 0)
            list.push(offer("wolf", "I will find it."))
        else if (p.errands.wolf === 1)
            list.push(offer("wolf done", "It is dead.", game.world.wolfDown))
    }
    list.push(offer("leave", "Farewell."))
    return list
}

/** A player says what they chose. */
function say(game, p, chosen) {
    const talk = p.talk
    if (!chosen.able) {
        game.said.push({ what: "no" })
        return
    }
    switch (chosen.id) {
    case "leave":
        p.talk = null
        game.said.push({ what: "leave" })
        return
    case "beacons":
        p.errands.king = 1
        talk.line = "Then go with what light we have left. The Mirewood is nearest. Keep to the paths."
        break
    case "rest":
        p.health = p.most
        p.mana = p.manaMost
        talk.line = "There. Whole again."
        break
    case "potion":
        p.gold -= POTION_COST
        p.potions += 1
        talk.line = "Drink it when the world goes grey at the edges."
        break
    case "herbs":
        p.errands.herbs = 1
        talk.line = "Five will do. They glow a little at dusk, which is all the time now."
        break
    case "herbs done":
        p.errands.herbs = 2
        p.gold += 100
        p.potionsMost += 1
        p.potions = Math.min(p.potionsMost, p.potions + 2)
        p.score += 400
        talk.line = "Bless you. Two potions, and a bigger satchel to carry them in."
        break
    case "blade":
        p.blade += 1
        p.gold -= BLADES[p.blade].cost
        talk.line = "Mind the edge. It remembers what it cuts."
        break
    case "ward":
        p.ward += 1
        p.gold -= WARDS[p.ward].cost
        talk.line = "Stand still. There. Now it will take a blow for you."
        break
    case "ore":
        p.errands.ore = 1
        talk.line = "Four lumps of it. It comes up through the ash in the Hold."
        break
    case "ore done":
        p.errands.ore = 2
        p.score += 400
        p.gold += 100
        if (p.blade + 1 < BLADES.length)
            p.blade += 1
        else
            p.gold += 200
        talk.line = "Good iron. Your blade is the better for it, and here is something for the walk."
        break
    case "wolf":
        p.errands.wolf = 1
        talk.line = "West, past the meadow. Do not let it get behind you."
        break
    case "wolf done":
        p.errands.wolf = 2
        p.score += 400
        p.gold += 200
        p.most += 20
        p.health = Math.min(p.most, p.health + 20)
        p.xp += 80
        levelUp(game, p)
        talk.line = "Take this, and my thanks. Wear its hide and you will stand the taller for it."
        break
    }
    game.said.push({ what: "yes", id: chosen.id })
}

/** A step of a hero in talk: the choice moves, or something is said. */
function talking(game, p, asked) {
    const list = offers(game, p.id)
    if (asked.roll) {
        p.talk = null
        game.said.push({ what: "leave" })
        return
    }
    if (asked.up)
        p.talk.cursor = (p.talk.cursor + list.length - 1) % list.length
    if (asked.down)
        p.talk.cursor = (p.talk.cursor + 1) % list.length
    p.talk.cursor = Math.min(p.talk.cursor, list.length - 1)
    let chosen = asked.attack || asked.talk ? p.talk.cursor : -1
    for (let n = 0; n < 6; n++) {
        if (asked["say" + (n + 1)] && n < list.length)
            chosen = n
    }
    if (chosen >= 0)
        say(game, p, list[chosen])
}

/** What a player should do next, and where: the words their screen shows, and the place it marks. */
export function objective(game, who) {
    const p = game.players[who === undefined ? 0 : who]
    const mark = (words, place) => {
        const at = placeAt(place)
        return { words: words, x: at.x, y: at.y }
    }
    if (game.world.kingDown)
        return { words: "The valley is yours again.", x: p.x, y: p.y }
    if (p.errands.king === 0)
        return mark("Speak to Elder Maren, in Hearth", "elder")
    if (game.world.gate)
        return mark("Through the gate: end the Hollow King", "crown")
    for (let i = 0; i < BEACONS.length; i++) {
        if (!game.world.beacons[i])
            return mark((game.world.wardens[i] ? "Light the " : "Break the warden of the ") + BEACONS[i].name
                        + (game.world.wardens[i] ? " beacon" : ""), BEACONS[i].place)
    }
    return mark("Speak to Elder Maren, in Hearth", "elder")
}

// ── Heroes ───────────────────────────────────────────────────────────────────────────────────────────────────────────

/** The hero's power with their blade. */
export function power(p) {
    return p.power + BLADES[p.blade].power
}

/** A hero wounded, unless they roll, are talking, or are still reeling from the last wound. */
function wound(game, p, amount, from) {
    if (p.roll > 0 || p.hurt > 0 || p.health <= 0 || p.talk || p.out)
        return
    const taken = Math.max(1, amount - WARDS[p.ward].ward)
    p.health = Math.max(0, p.health - taken)
    p.hurt = HURT_HOLD
    game.said.push({ what: "hurt", who: p.id, amount: taken, x: p.x, y: p.y, by: from })
}

/** An enemy struck for `amount` by a hero, knocked back from them, and woken if it slept. */
function strike(game, p, e, amount, crit) {
    e.health -= amount
    e.struck = 8
    e.awake = true
    e.by = p.id
    const dx = e.x - p.x, dy = e.y - p.y, length = Math.sqrt(dx * dx + dy * dy) || 1
    const push = e.kind >= WARDEN ? 0 : e.kind === GREAT_WOLF || e.kind === REVENANT ? 2 : 5
    e.kx = (dx / length) * push
    e.ky = (dy / length) * push
    game.said.push({ what: crit ? "crit" : "hit", id: e.id, who: p.id, amount: amount, x: e.x, y: e.y, kind: e.kind })
}

/** One blow's damage: a share of the hero's power and up to half again, doubled one time in ten. */
function blow(game, p, share) {
    const base = Math.floor((power(p) * share) / 100)
    const amount = base + between(game.random, 0, Math.floor(base / 2))
    return chance(game.random, 10) ? [amount * 2, true] : [amount, false]
}

function levelUp(game, p) {
    while (p.xp >= nextXp(p.level)) {
        p.level += 1
        p.most += 12
        p.manaMost += 4
        p.power += 2
        p.health = Math.min(p.most, p.health + Math.floor((p.most * 30) / 100))
        game.said.push({ what: "level", who: p.id, level: p.level })
    }
}

function pick(game, p, item) {
    if (item.what === "gold") {
        p.gold += item.amount
        p.score += item.amount
    } else {
        if (p.potions >= p.potionsMost)
            return false
        p.potions += 1
    }
    game.said.push({ what: "pick", who: p.id, item: item.what, amount: item.amount, x: item.x, y: item.y })
    return true
}

/** The nearest enemy alive within `reach` of a hero that they can see, or null. */
function targetFor(game, p, reach, awakeOnly) {
    let target = null, best = reach * reach
    for (const e of game.world.enemies) {
        const dx = e.x - p.x, dy = e.y - p.y, d = dx * dx + dy * dy
        if (e.health > 0 && d < best && (!awakeOnly || e.awake) && inSight(game, p.x, p.y, e.x, e.y)) {
            best = d
            target = e
        }
    }
    return target
}

function turnTo(p, e) {
    const dx = e.x - p.x, dy = e.y - p.y, length = Math.sqrt(dx * dx + dy * dy) || 1
    p.facing = [dx / length, dy / length]
}

/** One hero's step: what they asked for, then their movement, and what lies at their feet. */
function hero(game, p) {
    if (p.out)
        return
    const asked = p.asked
    p.asked = noAsks()
    if (p.talk) {
        talking(game, p, asked)
        return
    }
    if (p.swing > 0) {
        p.swing -= 1
        if (p.swing === 0)
            p.chainLeft = CHAIN_STEPS
    } else if (p.chainLeft > 0)
        p.chainLeft -= 1
    if (p.cast > 0)
        p.cast -= 1
    if (p.roll > 0)
        p.roll -= 1
    if (p.rollCool > 0)
        p.rollCool -= 1
    if (p.potionCool > 0)
        p.potionCool -= 1
    if (p.hurt > 0)
        p.hurt -= 1
    p.manaIn -= 1
    if (p.manaIn === 0) {
        p.manaIn = MANA_EVERY
        p.mana = Math.min(p.manaMost, p.mana + 1)
    }

    if (asked.turnLeft)
        p.view = (p.view + 7) % 8
    if (asked.turnRight)
        p.view = (p.view + 1) % 8
    const across = (p.held.right ? 1 : 0) - (p.held.left ? 1 : 0)
    const ahead = (p.held.up ? 1 : 0) - (p.held.down ? 1 : 0)
    const [ux, uy] = VIEWS[p.view]
    let vx = across * -uy + ahead * ux
    let vy = across * ux + ahead * uy
    if (across !== 0 && ahead !== 0) {
        vx *= HALF_ROOT
        vy *= HALF_ROOT
    }
    const moving = across !== 0 || ahead !== 0
    if (moving && p.swing === 0 && p.roll === 0 && p.cast === 0)
        p.facing = [vx, vy]

    const free = p.roll === 0 && p.cast === 0
    if (asked.potion && p.potions > 0 && p.potionCool === 0 && p.health < p.most) {
        p.potions -= 1
        p.potionCool = POTION_COOL
        p.health = Math.min(p.most, p.health + Math.floor((p.most * POTION_HEALS) / 100))
        game.said.push({ what: "potion", who: p.id })
    }
    if (asked.roll && p.rollCool === 0 && p.roll === 0) {
        if (moving)
            p.facing = [vx, vy]
        p.roll = ROLL_STEPS
        p.rollCool = ROLL_COOL
        p.swing = 0
        p.chainLeft = 0
        p.cast = 0
        game.said.push({ what: "roll", who: p.id, x: p.x, y: p.y })
    } else if (asked.nova && free && p.swing === 0 && p.level >= NOVA_LEVEL && p.mana >= NOVA_COST) {
        p.mana -= NOVA_COST
        p.cast = NOVA_CAST
        p.casting = "nova"
        game.said.push({ what: "nova", who: p.id, x: p.x, y: p.y })
    } else if (asked.bolt && free && p.swing === 0 && p.mana >= BOLT_COST) {
        p.mana -= BOLT_COST
        p.cast = BOLT_CAST
        p.casting = "bolt"
        const target = targetFor(game, p, BOLT_AIM, false)
        if (target)
            turnTo(p, target)
        game.said.push({ what: "cast", who: p.id, x: p.x, y: p.y })
    } else if ((asked.attack || p.held.attack) && free && p.swing === 0) {
        // The swing turns toward the nearest enemy awake within reach, so a pad aims it as well as a pointer.
        const target = targetFor(game, p, AIM_REACH, true)
        if (target)
            turnTo(p, target)
        p.chain = p.chainLeft > 0 ? (p.chain + 1) % SWINGS.length : 0
        p.chainLeft = 0
        p.swing = SWINGS[p.chain].steps
        game.said.push({ what: "swing", who: p.id, chain: p.chain, x: p.x, y: p.y })
    }

    if (p.roll > 0) {
        moveBy(game, p, p.facing[0] * ROLL_SPEED, p.facing[1] * ROLL_SPEED, HERO_R)
    } else if (p.cast === 0 && moving) {
        // Planted while a swing winds up, slowed while it follows through, and slower in the mire's mud.
        const strikes = SWINGS[p.chain].strikes
        const share = p.swing > strikes ? 0 : p.swing > 0 ? 40 : 100
        const mud = WORLD[Math.floor(p.y / TILE)][Math.floor(p.x / TILE)] === ":" ? 85 : 100
        const speed = (HERO_SPEED * share * mud) / 10000
        moveBy(game, p, vx * speed, vy * speed, HERO_R)
    }

    if (asked.talk) {
        let closest = null, best = TALK_REACH * TALK_REACH
        for (const who in PEOPLE) {
            const at = placeAt(PEOPLE[who].place)
            const dx = at.x - p.x, dy = at.y - p.y, d = dx * dx + dy * dy
            if (d < best) {
                best = d
                closest = who
            }
        }
        if (closest) {
            p.talk = { who: closest, cursor: 0, line: greeting(game, p, closest) }
            p.swing = 0
            p.chainLeft = 0
            p.cast = 0
            p.roll = 0
            game.said.push({ what: "talk", who: p.id, person: closest })
            return
        }
    }

    const world = game.world
    world.items = world.items.filter((item) => !(near(p.x, p.y, item.x, item.y, PICK_REACH) && pick(game, p, item)))
    HERBS.forEach(([c, r], i) => {
        if (!p.herbs[i] && near(p.x, p.y, (c + 0.5) * TILE, (r + 0.5) * TILE, PICK_REACH)) {
            p.herbs[i] = true
            game.said.push({ what: "herb", who: p.id, have: herbsHeld(p) })
        }
    })
    ORE.forEach(([c, r], i) => {
        if (!p.ore[i] && near(p.x, p.y, (c + 0.5) * TILE, (r + 0.5) * TILE, PICK_REACH)) {
            p.ore[i] = true
            game.said.push({ what: "ore", who: p.id, have: oreHeld(p) })
        }
    })
    BEACONS.forEach((beacon, i) => {
        const at = placeAt(beacon.place)
        if (world.wardens[i] && !world.beacons[i] && near(p.x, p.y, at.x, at.y, BEACON_REACH)) {
            world.beacons[i] = true
            p.score += 800
            game.said.push({ what: "beacon", who: p.id, beacon: i, x: at.x, y: at.y })
            if (beaconsLit(game) === BEACONS.length) {
                world.gate = true
                game.said.push({ what: "gate" })
            }
        }
    })
}

/** A hero's swing, bolt and nova, at the steps they land. */
function blows(game, p) {
    if (p.out || p.talk)
        return
    const world = game.world
    const swing = SWINGS[p.chain]
    if (p.swing === swing.strikes) {
        const [fx, fy] = p.facing
        for (const e of world.enemies) {
            const dx = e.x - p.x, dy = e.y - p.y, d = Math.sqrt(dx * dx + dy * dy), r = KINDS[e.kind].r
            if (e.health > 0 && d < swing.reach + r && (d < r || dx * fx + dy * fy >= ARC_COS * d)) {
                const [amount, crit] = blow(game, p, swing.share)
                strike(game, p, e, amount, crit)
            }
        }
    }
    if (p.casting === "bolt" && p.cast === BOLT_LOOSES) {
        if (world.shots.length < MOST_SHOTS)
            world.shots.push({ id: world.made++, who: p.id, x: p.x, y: p.y, vx: p.facing[0] * BOLT_SPEED,
                               vy: p.facing[1] * BOLT_SPEED, life: BOLT_STEPS })
        game.said.push({ what: "bolt", who: p.id, x: p.x, y: p.y })
    }
    if (p.casting === "nova" && p.cast === NOVA_STRIKES) {
        for (const e of world.enemies) {
            if (e.health > 0 && near(e.x, e.y, p.x, p.y, NOVA_REACH + KINDS[e.kind].r)) {
                const [amount, crit] = blow(game, p, NOVA_SHARE)
                strike(game, p, e, amount, crit)
            }
        }
        game.said.push({ what: "burst", who: p.id, x: p.x, y: p.y })
    }
}

/** What a hero's bolts strike. */
function shots(game) {
    const world = game.world
    for (const s of world.shots) {
        s.x += s.vx
        s.y += s.vy
        s.life -= 1
        if (hidesAt(game, s.x, s.y)) {
            s.life = 0
            continue
        }
        for (const e of world.enemies) {
            if (e.health > 0 && near(s.x, s.y, e.x, e.y, KINDS[e.kind].r + 6)) {
                const p = game.players[s.who]
                const [amount, crit] = blow(game, p, BOLT_SHARE)
                strike(game, p, e, amount, crit)
                s.life = 0
                break
            }
        }
    }
    world.shots = world.shots.filter((s) => s.life > 0)
}

/** A hero spent: they wake in Hearth, lighter of a quarter of their gold, with what chased them gone home; or, at the last
 * of their lives, they are out of the run. */
function fall(game, p) {
    p.falls += 1
    game.said.push({ what: "fallen", who: p.id, x: p.x, y: p.y })
    if (p.falls >= LIVES) {
        p.out = true
        return
    }
    const at = placeAt("hearth")
    p.x = at.x
    p.y = at.y
    p.health = p.most
    p.mana = p.manaMost
    p.gold -= Math.floor(p.gold / 4)
    p.swing = 0
    p.chainLeft = 0
    p.cast = 0
    p.roll = 0
    p.hurt = HURT_HOLD
    for (const e of game.world.enemies) {
        if (e.awake)
            e.act = "home"
    }
}

// ── The enemies ──────────────────────────────────────────────────────────────────────────────────────────────────────

/** The hero nearest an enemy still in the run, or null. */
function closest(game, e) {
    let found = null, best = Infinity
    for (const p of game.players) {
        const dx = p.x - e.x, dy = p.y - e.y, d = dx * dx + dy * dy
        if (!p.out && d < best) {
            best = d
            found = p
        }
    }
    return found
}

/** Where an enemy heads: straight for the hero while it sees them, otherwise one tile nearer by the valley's ways. */
function heading(game, e, p) {
    if (inSight(game, e.x, e.y, p.x, p.y))
        return [p.x - e.x, p.y - e.y]
    const way = paths(game, p)
    const c = Math.floor(e.x / TILE), r = Math.floor(e.y / TILE)
    let best = way[r * COLS + c], to = null
    for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        if (walkable(game, c + dc, r + dr)) {
            const d = way[(r + dr) * COLS + c + dc]
            if (d >= 0 && (best < 0 || d < best)) {
                best = d
                to = [(c + dc + 0.5) * TILE, (r + dr + 0.5) * TILE]
            }
        }
    }
    return to ? [to[0] - e.x, to[1] - e.y] : [p.x - e.x, p.y - e.y]
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

/** A shot from afar, at where `to` is now. */
function loose(game, e, x, y, vx, vy) {
    const world = game.world
    if (world.bolts.length < MOST_SHOTS)
        world.bolts.push({ id: world.made++, kind: e.kind, x: x, y: y, vx: vx, vy: vy, life: THORN_STEPS, wound: e.wound })
}

/** The king's ring: eight shots, one along each way a view can face. */
function ring(game, e) {
    for (const [ux, uy] of VIEWS)
        loose(game, e, e.x, e.y, ux * THORN_SPEED, uy * THORN_SPEED)
    game.said.push({ what: "ring", x: e.x, y: e.y })
}

/** A great one calls `count` of `kind` out of the ground beside it. */
function summon(game, e, kind, count) {
    const world = game.world
    for (let n = 0; n < count && world.enemies.length < MOST_ENEMIES; n++) {
        const side = n % 2 === 0 ? -1 : 1
        const x = e.x + side * (KINDS[e.kind].r + 24), y = e.y
        if (!blocked(game, x, y, 18)) {
            enemy(game, kind, x, y, e.region, -1)
            const made = world.enemies[world.enemies.length - 1]
            made.awake = true
            made.woke = true
        }
    }
    game.said.push({ what: "summon", x: e.x, y: e.y })
}

/** One enemy's step: it wakes, closes in, winds up and strikes, keeps away and shoots, or goes home. */
function enemyStep(game, e, p) {
    const k = KINDS[e.kind]
    const great = e.kind >= WARDEN
    if (e.struck > 0)
        e.struck -= 1
    if (e.kx !== 0 || e.ky !== 0) {
        moveBy(game, e, e.kx, e.ky, Math.min(18, k.r))
        e.kx = Math.abs(e.kx) < 0.2 ? 0 : e.kx * 0.7
        e.ky = Math.abs(e.ky) < 0.2 ? 0 : e.ky * 0.7
    }
    const dx = p.x - e.x, dy = p.y - e.y, d = Math.sqrt(dx * dx + dy * dy)
    if (!e.awake) {
        if (d < WAKE_REACH && inSight(game, e.x, e.y, p.x, p.y))
            e.awake = true
        else
            return
    }
    if (!e.woke) {
        e.woke = true
        game.said.push({ what: great ? (e.kind === KING ? "king" : "warden") : "wake", x: e.x, y: e.y, kind: e.kind })
        for (const o of game.world.enemies) {
            if (!o.awake && near(o.x, o.y, e.x, e.y, PACK_REACH))
                o.awake = true
        }
    }
    if (e.cool > 0)
        e.cool -= 1

    if (e.act === "home" || !near(e.x, e.y, e.homeX, e.homeY, LEASH)) {
        if (e.act !== "home") {
            e.act = "home"
            e.timer = 0
        }
        e.timer += 1
        // Home by the straight way, or at last by no way at all: nobody sees it arrive. It comes back whole.
        if (near(e.x, e.y, e.homeX, e.homeY, 12) || e.timer > 360) {
            e.x = e.homeX
            e.y = e.homeY
            e.health = e.most
            e.awake = false
            e.woke = false
            e.act = "move"
            e.timer = 0
        } else {
            walk(game, e, e.homeX - e.x, e.homeY - e.y, 130)
        }
        return
    }

    if (e.act === "windup") {
        e.timer -= 1
        if (e.timer > 0)
            return
        if (k.reach === 0) {
            const ax = e.aimX - e.x, ay = e.aimY - e.y, length = Math.sqrt(ax * ax + ay * ay) || 1
            loose(game, e, e.x, e.y, (ax / length) * THORN_SPEED, (ay / length) * THORN_SPEED)
            game.said.push({ what: e.kind === THORNLING ? "thorn" : "curse", x: e.x, y: e.y })
        } else {
            if (great)
                game.said.push({ what: "slam", x: e.x, y: e.y })
            if (d < k.reach + HERO_R + (great ? 0 : k.r))
                wound(game, p, e.wound, e.kind)
            if (e.kind === KING && e.health * 2 < e.most)
                ring(game, e)
        }
        e.act = "recover"
        e.timer = e.kind === KING && e.health * 2 < e.most ? Math.floor(k.recover / 2) : k.recover
        return
    }
    if (e.act === "recover") {
        e.timer -= 1
        if (e.timer <= 0)
            e.act = "move"
        else if (k.reach !== 0)
            walk(game, e, dx, dy, 25)
        return
    }

    if (k.reach === 0) {
        const sees = inSight(game, e.x, e.y, p.x, p.y)
        if (sees && d < 300 && e.cool === 0) {
            e.act = "windup"
            e.timer = k.windup
            e.cool = k.recover + k.windup
            e.aimX = p.x
            e.aimY = p.y
            e.fx = dx / (d || 1)
            e.fy = dy / (d || 1)
        } else if (sees && d < 130) {
            walk(game, e, -dx, -dy, 100)
        } else if (!sees || d > 240) {
            const [hx, hy] = heading(game, e, p)
            walk(game, e, hx, hy, 100)
        }
        return
    }
    if (great && e.cool === 0) {
        // A warden calls two of its own every ten seconds; the king, once he is hurt, two revenants every twelve.
        if (e.kind === WARDEN) {
            e.cool = 600
            summon(game, e, BEACONS[e.hold].summons, 2)
        } else if (e.health * 2 < e.most) {
            e.cool = 720
            summon(game, e, REVENANT, 2)
        }
    }
    const reach = great ? k.reach - 10 : k.reach + HERO_R + k.r
    if (d < reach) {
        e.act = "windup"
        e.timer = e.kind === KING && e.health * 2 < e.most ? Math.floor((k.windup * 3) / 4) : k.windup
        e.fx = dx / (d || 1)
        e.fy = dy / (d || 1)
        if (great)
            game.said.push({ what: "rising", x: e.x, y: e.y })
        return
    }
    let [hx, hy] = heading(game, e, p)
    if (e.kind === SHADE) {
        // A shade weaves as it comes, turning a little to one side and then the other.
        const side = Math.floor((game.steps + e.id * 13) / 40) % 2 === 0 ? 0.6 : -0.6
        const across = -hy * side, along = hx * side
        hx += across
        hy += along
    }
    walk(game, e, hx, hy, 100)
}

/** The enemies near a hero act, and keep apart from each other and from the hero. */
function enemies(game) {
    const world = game.world
    const near_ = []
    for (const e of world.enemies) {
        if (e.health <= 0)
            continue
        const p = closest(game, e)
        if (p && near(e.x, e.y, p.x, p.y, ACTIVE_REACH)) {
            enemyStep(game, e, p)
            if (e.awake && e.health > 0)
                near_.push(e)
        } else if (e.awake) {
            // Nobody near: it is home, as if it had walked.
            e.x = e.homeX
            e.y = e.homeY
            e.health = e.most
            e.awake = false
            e.woke = false
            e.act = "move"
        }
    }
    for (let i = 0; i < near_.length; i++) {
        const a = near_[i], ka = KINDS[a.kind]
        for (let j = i + 1; j < near_.length; j++) {
            const b = near_[j], kb = KINDS[b.kind]
            const dx = b.x - a.x, dy = b.y - a.y, d = Math.sqrt(dx * dx + dy * dy), apart = ka.r + kb.r
            if (d < apart && d > 0.001) {
                const push = (apart - d) / 2
                moveBy(game, a, (-dx / d) * push, (-dy / d) * push, Math.min(18, ka.r))
                moveBy(game, b, (dx / d) * push, (dy / d) * push, Math.min(18, kb.r))
            }
        }
        for (const p of game.players) {
            if (p.out)
                continue
            const dx = a.x - p.x, dy = a.y - p.y, d = Math.sqrt(dx * dx + dy * dy), apart = ka.r + HERO_R
            if (d < apart && d > 0.001)
                moveBy(game, a, (dx / d) * (apart - d), (dy / d) * (apart - d), Math.min(18, ka.r))
        }
    }
}

/** What the enemies shoot: hidden by what stands, and wounding a hero it reaches. */
function bolts(game) {
    const world = game.world
    for (const b of world.bolts) {
        b.x += b.vx
        b.y += b.vy
        b.life -= 1
        if (hidesAt(game, b.x, b.y)) {
            b.life = 0
            continue
        }
        for (const p of game.players) {
            if (!p.out && !p.talk && near(b.x, b.y, p.x, p.y, HERO_R + 6)) {
                wound(game, p, b.wound, b.kind)
                b.life = 0
                break
            }
        }
    }
    world.bolts = world.bolts.filter((b) => b.life > 0)
}

/** What an enemy leaves: gold more often than not, now and then a potion, and from the great ones both. */
function drop(game, e) {
    const world = game.world
    const add = (item, spread) => {
        const x = e.x + between(game.random, -spread, spread), y = e.y + between(game.random, -spread, spread)
        // Never inside a tree or a wall, where nobody could reach it: where the enemy stood instead.
        const clear = !blocked(game, x, y, 8)
        if (world.items.length < MOST_ITEMS)
            world.items.push(Object.assign({ id: world.made++, x: clear ? x : e.x, y: clear ? y : e.y }, item))
    }
    const worth = REGIONS[e.region].gold
    if (e.kind >= GREAT_WOLF) {
        for (let i = 0; i < 6; i++)
            add({ what: "gold", amount: between(game.random, 8, 16) * worth }, 34)
        add({ what: "potion", amount: 1 }, 34)
        return
    }
    if (chance(game.random, [50, 45, 60, 70, 65][e.kind]))
        add({ what: "gold", amount: between(game.random, 3, 9) * worth }, 10)
    if (chance(game.random, [6, 6, 10, 12, 12][e.kind]))
        add({ what: "potion", amount: 1 }, 10)
}

/** The fallen: experience and score to whoever felled them, what they leave, and what their fall changes. */
function fallen(game) {
    const world = game.world
    for (const e of world.enemies) {
        if (e.health > 0)
            continue
        const k = KINDS[e.kind], p = game.players[e.by]
        p.kills += 1
        p.score += k.score
        p.xp += k.xp
        game.said.push({ what: "kill", kind: e.kind, who: p.id, x: e.x, y: e.y })
        drop(game, e)
        if (e.kind === WARDEN) {
            world.wardens[e.hold] = true
            game.said.push({ what: "warden down", hold: e.hold, x: e.x, y: e.y })
        } else if (e.kind === GREAT_WOLF) {
            world.wolfDown = true
        } else if (e.kind === KING) {
            world.kingDown = true
            game.clearing = CLEARING_STEPS
            game.said.push({ what: "king down", x: e.x, y: e.y })
        }
    }
    world.enemies = world.enemies.filter((e) => e.health > 0)
    for (const p of game.players)
        levelUp(game, p)
}

/**
 * One step: the heroes act and move, their blows land, the enemies act, shots fly, the fallen leave what they leave, and
 * the run may end. What happened is in `said`. While every hero talks the valley holds, though the veil's clock does not.
 */
export function step(game) {
    if (game.phase !== "playing")
        return
    game.steps += 1
    game.said = []
    if (game.clearing > 0) {
        game.clearing -= 1
        if (game.clearing === 0) {
            game.won = true
            for (const p of game.players)
                p.score += Math.floor((RUN_STEPS - game.steps) / 12)
            game.phase = "over"
            game.said.push({ what: "won" })
            return
        }
    }
    for (const p of game.players)
        hero(game, p)
    if (!game.players.every((p) => p.talk !== null || p.out)) {
        for (const p of game.players)
            blows(game, p)
        enemies(game)
        shots(game)
        bolts(game)
        fallen(game)
    }
    for (const p of game.players) {
        if (!p.out && p.health <= 0)
            fall(game, p)
    }
    if (game.players.every((p) => p.out)) {
        game.phase = "over"
        game.said.push({ what: "lost" })
    } else if (game.steps >= RUN_STEPS && game.phase === "playing") {
        game.phase = "over"
        game.said.push({ what: "closed" })
    }
}

/** What the game came to, for one player. */
export function outcome(game, who) {
    const p = game.players[who === undefined ? 0 : who]
    return {
        score: p.score,
        kills: p.kills,
        gold: p.gold,
        level: p.level,
        health: p.health,
        falls: p.falls,
        beacons: beaconsLit(game),
        errands: [p.errands.herbs, p.errands.ore, p.errands.wolf].filter((state) => state === 2).length,
        seconds: Math.floor(game.steps / 60),
        won: game.won,
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
