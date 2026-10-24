// Rift Survivor (DIRECTION P7.20, ADR-102 decision 2), rebuilt in QQ from ARQADE's 2D game: one ship in an arena for
// ninety seconds. Enemies close from the edges and the ship fires on the nearest by itself; the player moves it, keeps
// out of the amber rift pulses, collects the fragments kills leave, and every so many kills chooses a perk.
//
// The rules are logic/rules.mjs, the one place they live: ARQADE's service runs the same file to replay a run. This
// logic hands them every event as QQ's tick delivers it (a screen button's click, an action's press and release, a
// step) and draws what they say: the ship, enemies, shots, fragments and pulses where the rules put them, the screens for
// the rules' phase. So a run QQ records is the rules' input exactly, and its outcome is theirs (`outcome`). Nothing drawn
// here decides anything, which is why it may use trigonometry the rules may not.
//
// The arena is a board tilted toward where the eye looks from, so it is seen from above at an angle. What the rules hold
// is drawn from pools made as play starts, each thing keeping one piece by its number while it lasts; nothing is made
// or removed during a run. The world is held while the player pauses and while a perk is chosen. With less motion asked
// for (Look), enemies and fragments do not turn, the hull's warning is steady rather than blinking, and the words do not
// swell.

import QQ
import "rules.mjs" as Rules

Logic {
    id: rift

    /// The rules' game: their state, which only they change.
    property var game: null

    /// The arena's units to the board's: the website's 1000 by 620 drawn 16 by 9.92.
    readonly property real k: 0.016
    readonly property var away: Qt.vector3d(0, -30, 0)
    property var pools: ({})
    /// The pieces each pool showed last frame, so only those that stop showing are put away.
    property var showing: ({})
    property real turn: 0

    function at(x, y, lift) {
        return Qt.vector3d((x - 500) * k, lift, (y - 310) * k)
    }

    /// Something of the scene's look, made here: only looked at, so it has no body.
    function piece(fields) {
        fields.body = "none"
        fields.position = away
        return spawn("Shape", fields, entity("Board"))
    }

    function pool(name, count, fields) {
        const made = []
        for (let i = 0; i < count; i++) {
            const f = Object.assign({}, fields)
            f.name = name + " " + i
            made.push(piece(f))
        }
        return made
    }

    function text(name, words) {
        const shown = entity(name)
        if (shown)
            shown.text = words
    }

    function button(name) {
        const b = entity(name)
        if (b)
            b.clicked.connect(() => {
                const before = game.phase
                Rules.click(game, name)
                if (before === "choosing" && game.phase === "playing")
                    play("../assets/perk.wav", { volume: 0.7 })
                show()
            })
    }

    function said(words, colour) {
        const message = entity("Level")
        if (!message)
            return
        message.colour = colour
        if (!Look.reducedMotion) {
            message.size = 20
            tween(message, "size", 16, 0.4, "outCubic")
        }
    }

    /// The screens, the world's hold and the words, for the rules' phase; and what the run came to.
    function show() {
        outcome = Rules.outcome(game)
        const phase = game.phase
        const hold = phase === "paused" || phase === "choosing"
        if (hold && !paused)
            pause()
        else if (!hold && paused)
            resume()
        onlyScreen(phase === "title" ? "title" : phase === "over" ? "result" : "hud")
        if (phase === "paused")
            showScreen("pause")
        if (phase === "choosing")
            showScreen("evolve")
        text("Score", "SCORE " + outcome.score.toLocaleString(Qt.locale("en_GB"), "f", 0))
        text("Chain", "CHAIN ×" + ((10 + game.chain) / 10).toFixed(1))
        text("Hull", "HULL " + game.hull + " / " + Rules.HULL)
        text("Clock", String(Math.max(0, Math.ceil((Rules.RUN_STEPS - game.steps) / 60))))
        text("Level", "LEVEL " + game.level + "  ·  " + game.kills + " / " + game.nextLevel)
        text("Evolved", "SIGNAL EVOLVED  ·  LEVEL " + game.level)
        if (phase === "over") {
            text("Result", outcome.survived ? "RIFT SURVIVED" : "HULL BROKEN")
            text("Final", "SCORE " + outcome.score.toLocaleString(Qt.locale("en_GB"), "f", 0))
        }
    }

    /// Draw `things` from pool `name`, each by its number, at `place(thing)`; turn each by `spin` degrees if it turns.
    function draw(name, things, place, spin) {
        const list = pools[name]
        const was = showing[name] || new Set()
        const now = new Set()
        for (const thing of things) {
            const drawn = list[thing.id % list.length]
            now.add(drawn)
            drawn.position = place(thing)
            if (spin)
                drawn.eulerRotation.y += spin
        }
        for (const drawn of was)
            if (!now.has(drawn))
                drawn.position = away
        const next = Object.assign({}, showing)
        next[name] = now
        showing = next
    }

    onStarted: {
        game = Rules.create(scene.playSeed)
        // The arena's grid every fifty units of the website's, faint teal, and its edge, brighter.
        for (let x = 0; x <= Rules.WIDTH; x += 50)
            spawn("Shape", { name: "Grid x " + x, form: "cube", body: "none", position: at(x, 310, 0),
                             scale: Qt.vector3d(x % 1000 === 0 ? 0.05 : 0.022, 0.01, Rules.HEIGHT * k),
                             colour: "#0a151d", metalness: 0, roughness: 0.8, emissive: "#80b6c4",
                             emissivePower: x % 1000 === 0 ? 0.55 : x % 250 === 0 ? 0.3 : 0.16 },
                  entity("Board"))
        for (const y of [0, 50, 100, 150, 200, 250, 300, 350, 400, 450, 500, 550, 600, Rules.HEIGHT])
            spawn("Shape", { name: "Grid y " + y, form: "cube", body: "none", position: at(500, y, 0),
                             scale: Qt.vector3d(Rules.WIDTH * k, 0.01, y === 0 || y === Rules.HEIGHT ? 0.05 : 0.022),
                             colour: "#0a151d", metalness: 0, roughness: 0.8, emissive: "#80b6c4",
                             emissivePower: y === 0 || y === Rules.HEIGHT ? 0.55 : 0.16 }, entity("Board"))
        const p = {}
        p.small = pool("Drone", Rules.MOST_ENEMIES, { form: "cube", scale: Qt.vector3d(0.4, 0.4, 0.4),
                                                      eulerRotation: Qt.vector3d(0, 45, 0), colour: "#0e2a2e",
                                                      metalness: 0.4, roughness: 0.3, emissive: "#74cdd3",
                                                      emissivePower: 0.7 })
        p.armoured = pool("Warden", Rules.MOST_ENEMIES, { form: "cylinder", scale: Qt.vector3d(0.7, 0.32, 0.7),
                                                          colour: "#3a140b", metalness: 0.5, roughness: 0.3,
                                                          emissive: "#ff9160", emissivePower: 0.7 })
        p.shots = pool("Shot", Rules.MOST_SHOTS, { form: "sphere", scale: Qt.vector3d(0.09, 0.09, 0.09),
                                                   colour: "#cffaff", metalness: 0, roughness: 0.5, emissive: "#cffaff",
                                                   emissivePower: 0.9 })
        p.fragments = pool("Fragment", Rules.MOST_FRAGMENTS, { form: "cube", scale: Qt.vector3d(0.12, 0.12, 0.12),
                                                               eulerRotation: Qt.vector3d(45, 0, 45), colour: "#fbd68d",
                                                               metalness: 0.8, roughness: 0.2, emissive: "#ffb83a",
                                                               emissivePower: 0.8 })
        p.rings = pool("Rift", 8, { form: "cylinder", scale: Qt.vector3d(Rules.PULSE_REACH * 2 * k, 0.012,
                                                                          Rules.PULSE_REACH * 2 * k),
                                    colour: "#1a0c06", metalness: 0, roughness: 0.6, emissive: "#c2541c",
                                    emissivePower: 0.15 })
        p.fills = pool("Rift fill", 8, { form: "cylinder", scale: Qt.vector3d(0.1, 0.014, 0.1), colour: "#1a0c06",
                                         metalness: 0, roughness: 0.6, emissive: "#d8742e", emissivePower: 0.3 })
        pools = p
        for (const name of ["Play", "Resume", "Play again"].concat(Rules.PERKS))
            button(name)
        show()
    }

    // W, A, S and D, the arrows, the pad's d-pad or the buttons under a finger move; 1 to 3, X, Y or B choose a perk.
    onPressed: (action) => {
        const before = game.phase
        Rules.press(game, action)
        if (before === "choosing" && game.phase === "playing")
            play("../assets/perk.wav", { volume: 0.7 })
        show()
    }
    onReleased: (action) => Rules.release(game, action)

    onStep: {
        const before = game.phase
        const hull = game.hull
        const kills = game.kills
        Rules.step(game)
        // One sound of each kind a step, however many happened.
        const heard = new Set(game.said.map((s) => s.what))
        if (heard.has("hit"))
            play("../assets/hit.wav", { volume: 0.9 })
        if (heard.has("kill"))
            play("../assets/kill.wav", { volume: 0.5 })
        if (heard.has("shot"))
            play("../assets/shot.wav", { volume: 0.25 })
        if (heard.has("pulse"))
            play("../assets/pulse.wav", { volume: 0.5 })
        if (heard.has("fragment"))
            play("../assets/fragment.wav", { volume: 0.4 })
        if (heard.has("evolved")) {
            play("../assets/evolve.wav", { volume: 0.7 })
            said("", "#ead39a")
        }
        if (before !== "over" && game.phase === "over")
            play(Rules.outcome(game).survived ? "../assets/complete.wav" : "../assets/lost.wav", { volume: 0.9 })
        if (before !== game.phase || hull !== game.hull || kills !== game.kills || game.steps % 6 === 0)
            show()
    }

    // Only looked at: everything where the rules put it.
    onFrame: (dt) => {
        if (!Look.reducedMotion)
            turn = (turn + 90 * dt) % 360
        const spin = Look.reducedMotion ? 0 : 90 * dt
        const ship = entity("Ship")
        const wake = entity("Wake")
        if (ship) {
            ship.position = at(game.x, game.y, 0.15)
            // The cone points along -z at rest: turned about the board's normal toward where the ship last moved.
            ship.eulerRotation = Qt.vector3d(-90, Math.atan2(-game.facing[0], -game.facing[1]) * 180 / Math.PI, 0)
            const breached = game.hold > 0 && (Look.reducedMotion || game.hold % 12 < 6)
            ship.emissive = breached ? "#ff704d" : "#ffd08a"
            ship.emissivePower = breached ? 1 : 0.5
        }
        if (wake)
            wake.position = at(game.x, game.y, 0.15)
        draw("small", game.enemies.filter((e) => e.kind === 0), (e) => at(e.x, e.y, 0.2), spin)
        draw("armoured", game.enemies.filter((e) => e.kind === 1), (e) => at(e.x, e.y, 0.16), spin)
        draw("shots", game.shots, (b) => at(b.x, b.y, 0.2), 0)
        draw("fragments", game.fragments, (f) => at(f.x, f.y, 0.14), spin * 2)
        draw("rings", game.pulses, (q) => at(q.x, q.y, 0.006), 0)
        draw("fills", game.pulses, (q) => at(q.x, q.y, 0.012), 0)
        // A rift fills as it charges, and glows bright while it is armed.
        for (const q of game.pulses) {
            const armed = q.age >= Rules.PULSE_ARMS && q.age < Rules.PULSE_ENDS
            const ring = pools.rings[q.id % pools.rings.length]
            const fill = pools.fills[q.id % pools.fills.length]
            ring.emissivePower = armed ? 0.5 : 0.15
            const r = Rules.PULSE_REACH * 2 * k * Math.min(1, q.age / Rules.PULSE_ARMS)
            fill.scale = Qt.vector3d(Math.max(0.05, r), 0.014, Math.max(0.05, r))
            fill.emissivePower = armed ? 0.55 : 0.3
        }
    }
}
