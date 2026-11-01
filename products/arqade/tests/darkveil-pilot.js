// SPDX-License-Identifier: Apache-2.0
// Darkveil's pilot: what a person would do next, read from the rules' game as it stands. Node's test
// (darkveil.test.mjs) and ARQADE's games suite (qt/tests/tst_games.cpp) both evaluate this and call the function it gives
// with the rules, the game and whether to play calm. The suite, in the game's own engine, plays what it says by the keys,
// the pad and the screen's buttons. The view stays where it starts, so a way in the valley's axes is the same on the keys.
//
// It takes the people's errands, rests and buys in Hearth each time a beacon is lit, clears what stands between it and the
// next warden, breaks the warden and lights the beacon, runs its errands as it passes them, then goes through the gate.
// In a fight it drinks when wounded, goes home to rest with nothing left to drink, rolls out from under a great one's slam,
// bursts among three, shoots what stands off, and strikes at what is near. Calm, it never strikes or uses a skill, and walks
// to the nearest enemy, awake or not; asked for "calm-to-warden", to the next warden.
(function () {
    const TILE = 40, COLS = 64, ROWS = 64
    const DIRS = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [1, -1], [-1, 1], [-1, -1]]
    const apart = (a, b) => Math.sqrt((a.x - b.x) * (a.x - b.x) + (a.y - b.y) * (a.y - b.y))
    // How far each tile is, walking, from the goal and from the hero; each worked out again only when its tile changes. The
    // suite calls this at every look, on the thread that draws the game, so it must cost next to nothing.
    let ways = { key: "", far: null }, around = { key: "", far: null }
    const memo = { run: -1, shopped: -1 }

    function walking(R, g, tc, tr) {
        const far = new Array(COLS * ROWS).fill(-1)
        far[tr * COLS + tc] = 0
        const queue = [tr * COLS + tc]
        for (let head = 0; head < queue.length; head++) {
            const at = queue[head], c = at % COLS, r = (at - c) / COLS
            for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
                const nc = c + dc, nr = r + dr
                if (R.walkable(g, nc, nr) && far[nr * COLS + nc] < 0) {
                    far[nr * COLS + nc] = far[at] + 1
                    queue.push(nr * COLS + nc)
                }
            }
        }
        return far
    }

    // How many tiles the place (x, y) is from the hero, walking, or -1: one walk from the hero answers for every place.
    function tilesTo(R, g, p, x, y) {
        const c = Math.floor(p.x / TILE), r = Math.floor(p.y / TILE), key = g.world.gate + ":" + c + "," + r
        if (around.key !== key)
            around = { key: key, far: walking(R, g, c, r) }
        return around.far[Math.floor(y / TILE) * COLS + Math.floor(x / TILE)]
    }

    // The way to (x, y): toward the middle of the next tile nearer it, or straight there once beside it.
    function wayTo(R, g, p, x, y) {
        const goalC = Math.floor(x / TILE), goalR = Math.floor(y / TILE), key = g.world.gate + ":" + goalC + "," + goalR
        if (ways.key !== key)
            ways = { key: key, far: walking(R, g, goalC, goalR) }
        const here = ways.far[Math.floor(p.y / TILE) * COLS + Math.floor(p.x / TILE)]
        let tx = x, ty = y
        if (here > 1) {
            const c = Math.floor(p.x / TILE), r = Math.floor(p.y / TILE)
            let best = here
            for (const [dc, dr] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
                const v = R.walkable(g, c + dc, r + dr) ? ways.far[(r + dr) * COLS + c + dc] : -1
                if (v >= 0 && v < best) {
                    best = v
                    tx = (c + dc + 0.5) * TILE
                    ty = (r + dr + 0.5) * TILE
                }
            }
        }
        const dx = tx - p.x, dy = ty - p.y
        if (dx * dx + dy * dy < 9)
            return [0, 0]
        let way = [0, 0], score = -Infinity
        for (const [a, b] of DIRS) {
            const s = (a * dx + b * dy) * (a && b ? 0.7071 : 1)
            if (s > score) {
                score = s
                way = [a, b]
            }
        }
        return way
    }

    // What it wants to say, in order: the first of these that it can.
    const wants = ["beacons", "wolf done", "herbs done", "ore done", "wolf", "herbs", "ore", "blade", "ward", "rest", "potion"]

    function answer(R, g, p, out) {
        const list = R.offers(g)
        for (const id of wants) {
            const n = list.findIndex((o) => o.id === id && o.able)
            if (n >= 0 && !(id === "potion" && p.potions >= 4) && !(id === "rest" && p.health === p.most)) {
                out.say = n + 1
                return out
            }
        }
        out.say = list.length
        return out
    }

    // The next person with something to say to the hero, if the hero is in Hearth's way.
    function errandFor(R, g, p, lit) {
        const e = p.errands
        const herbs = p.herbs.filter((x) => x).length, ore = p.ore.filter((x) => x).length
        if (e.king === 0)
            return "elder"
        if (e.wolf === 0)
            return "hunter"
        if (e.herbs === 0)
            return "healer"
        if (e.ore === 0)
            return "smith"
        if (e.herbs === 1 && herbs >= R.HERBS_WANTED)
            return "healer"
        if (e.ore === 1 && ore >= R.ORE_WANTED)
            return "smith"
        if (e.wolf === 1 && g.world.wolfDown)
            return "hunter"
        if (memo.shopped !== lit) {
            const next = R.BLADES[p.blade + 1], ward = R.WARDS[p.ward + 1]
            if (p.gold >= R.POTION_COST && p.potions < 2)
                return "healer"
            if ((next && p.gold >= next.cost) || (ward && p.gold >= ward.cost))
                return "smith"
            if (p.health < p.most || (p.gold >= R.POTION_COST && p.potions < 4))
                return "healer"
            memo.shopped = lit
        }
        return null
    }

    return function (R, g, calm) {
        const out = { way: [0, 0], attack: false, roll: false, bolt: false, nova: false, potion: false, talk: false, say: 0 }
        const p = g.players[0], world = g.world
        if (memo.run !== g.runs) {
            memo.run = g.runs
            memo.shopped = -1
        }
        const nearest = (list) => list.reduce((a, b) => apart(a, p) < apart(b, p) ? a : b)
        if (calm === "calm-to-warden") {
            const warden = world.enemies.find((e) => e.kind === R.WARDEN)
            if (warden)
                out.way = wayTo(R, g, p, warden.x, warden.y)
            return out
        }
        if (calm) {
            if (world.enemies.length) {
                const e = nearest(world.enemies.filter((o) => tilesTo(R, g, p, o.x, o.y) >= 0))
                out.way = wayTo(R, g, p, e.x, e.y)
            }
            return out
        }
        if (p.talk)
            return answer(R, g, p, out)

        const k = R.KINDS
        const awake = world.enemies.filter((e) => e.awake && e.act !== "home" && apart(e, p) < 420)
        const close = awake.filter((e) => apart(e, p) < R.SWINGS[0].reach + k[e.kind].r - 4)
        const lit = world.beacons.filter((x) => x).length
        out.potion = p.health < p.most * 0.5 && p.potions > 0 && p.potionCool === 0
        // Nothing left to drink and nearly spent: home, rolling when it can, to rest and to buy.
        if (p.health < p.most * 0.4 && p.potions === 0) {
            const at = R.placeAt("healer")
            if (apart(at, p) < R.TALK_REACH - 12) {
                out.talk = true
                return out
            }
            out.roll = p.rollCool === 0
            out.way = wayTo(R, g, p, at.x, at.y)
            return out
        }
        const slam = awake.find((e) => e.kind >= R.WARDEN && e.act === "windup" && apart(e, p) < k[e.kind].reach + 40)
        if (slam && p.rollCool === 0 && slam.timer < 24) {
            out.roll = true
            out.way = wayTo(R, g, p, 2 * p.x - slam.x, 2 * p.y - slam.y)
            return out
        }
        if (slam && p.rollCool > 0 && apart(slam, p) < k[slam.kind].reach + 24) {
            out.way = wayTo(R, g, p, 2 * p.x - slam.x, 2 * p.y - slam.y)
            return out
        }
        if (p.level >= R.NOVA_LEVEL && p.mana >= R.NOVA_COST && (close.length >= 3 || close.some((e) => e.kind >= R.GREAT_WOLF)))
            out.nova = true
        if (close.length) {
            out.attack = true
            return out
        }
        if (awake.length) {
            const e = nearest(awake)
            if (p.mana >= R.BOLT_COST && apart(e, p) < 280) {
                out.bolt = true
                return out
            }
            out.way = wayTo(R, g, p, e.x, e.y)
            return out
        }

        const goTo = (x, y) => {
            out.way = wayTo(R, g, p, x, y)
            return out
        }
        const person = errandFor(R, g, p, lit)
        if (person) {
            const at = R.placeAt(R.PEOPLE[person].place)
            if (apart(at, p) < R.TALK_REACH - 12) {
                out.talk = true
                return out
            }
            return goTo(at.x, at.y)
        }
        // By tiles walked, not by the straight line: the way to a thing can lead away from it for a while.
        const items = world.items.filter((i) => {
            const tiles = i.what !== "potion" || p.potions < p.potionsMost ? tilesTo(R, g, p, i.x, i.y) : -1
            return tiles >= 0 && tiles <= 9
        })
        if (items.length) {
            const i = nearest(items)
            return goTo(i.x, i.y)
        }
        // Whatever the healer wants, once the Mirewood is safe; what the smith wants, once the Hold is.
        const finds = []
        if (p.errands.herbs === 1 && p.herbs.filter((x) => x).length < R.HERBS_WANTED)
            p.herbs.forEach((had, i) => { if (!had) finds.push(R.HERBS[i]) })
        if (p.errands.ore === 1 && p.ore.filter((x) => x).length < R.ORE_WANTED && lit >= 1)
            p.ore.forEach((had, i) => { if (!had) finds.push(R.ORE[i]) })
        if (finds.length) {
            const places = finds.map(([c, r]) => ({ x: (c + 0.5) * TILE, y: (r + 0.5) * TILE }))
            const at = nearest(places)
            return goTo(at.x, at.y)
        }
        // The next beacon's region, and the meadow first; then its warden, and its light; the wolf after the first.
        const region = lit < 3 ? R.BEACONS[world.beacons.indexOf(false)].region : "crown"
        const standing = (name) => world.enemies.filter((e) => e.region === name && e.kind < R.GREAT_WOLF
                                                               && tilesTo(R, g, p, e.x, e.y) >= 0)
        for (const name of ["meadow", region]) {
            const left = standing(name)
            if (left.length) {
                const e = nearest(left)
                return goTo(e.x, e.y)
            }
        }
        if (lit >= 1 && !world.wolfDown) {
            const e = world.enemies.find((o) => o.kind === R.GREAT_WOLF)
            return goTo(e.x, e.y)
        }
        if (lit < 3) {
            const i = world.beacons.indexOf(false), at = R.placeAt(R.BEACONS[i].place)
            const warden = world.enemies.find((e) => e.kind === R.WARDEN && e.hold === i)
            return warden ? goTo(warden.x, warden.y) : goTo(at.x, at.y)
        }
        const king = world.enemies.find((e) => e.kind === R.KING)
        return king ? goTo(king.x, king.y) : out
    }
})()
