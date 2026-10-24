// Void Runner (DIRECTION P7.20, ADR-102 decision 2), rebuilt in QQ from ARQADE's 2D game: a ship runs three lanes into
// the collapse on the horizon, and hazards and gems rush down the lanes at it. Forty-five seconds survived completes the
// run; three hits end it.
//
// The rules are logic/rules.mjs, the one place they live: ARQADE's service runs the same file to replay a run. This
// logic hands them every event as QQ's tick delivers it (a screen button's click, an action's press, a step) and draws
// what they say: the ship in its lane, what is coming where it has come to, the screens for the rules' phase. So a run
// QQ records is the rules' input exactly, and its outcome is theirs (`outcome`). Nothing drawn here decides anything.
//
// The track is laid in a group turned toward where the eye looks from, so its lanes run straight away from it. What is
// thrown is drawn from two pools, made as play starts, so nothing is made or removed while the run goes on. The track's
// bars run toward the eye as fast as what is thrown, slowly on the title; QQ's world is held still only while the player
// pauses. With less motion asked for (Look), the gems do not turn, the collapse does not pulse, the hull's warning is
// steady rather than blinking, and the words do not swell.
//
// What happens at the ship (a change of lane, a gem, a breach) is heard from the ship, through QQ's spatial audio
// (`at`), so it sits in the ship's lane, left, middle or right. The run's end and the music are heard everywhere.

import QQ
import "rules.mjs" as Rules

Logic {
    id: runner

    /// The rules' game: their state, which only they change.
    property var game: null

    /// The track as drawn, in the group's units: where a lane's middle is, where things are thrown from and where the
    /// ship meets them.
    readonly property real laneGap: 1.7
    readonly property real far: -46
    readonly property real near: 4.5
    /// The pools, and the track's bars.
    readonly property int pool: 12
    readonly property int barCount: 26
    readonly property real barGap: 2.2
    property var hazards: []
    property var gems: []
    property var bars: []
    /// The bars' run, and the ship's drawn place across the track.
    property real travel: 0
    property real shipX: 0
    property real pulse: 0

    function laneX(lane) {
        return (lane - 1) * laneGap
    }

    /// Where a sound at the ship is heard from: its lane, where it meets what is thrown.
    function atShip() {
        const track = entity("Track")
        return track ? track.mapPositionToScene(Qt.vector3d(laneX(game.lane), 0.35, near)) : null
    }

    /// Something of the scene's look, made here: only looked at, so it has no body.
    function piece(fields) {
        fields.body = "none"
        return spawn("Shape", fields, entity("Track"))
    }

    function text(name, words) {
        const shown = entity(name)
        if (shown)
            shown.text = words
    }

    function button(name) {
        const b = entity(name)
        if (b)
            b.clicked.connect(() => { Rules.click(game, name); show() })
    }

    /// The screens, the world's hold and the words, for the rules' phase; and what the run came to.
    function show() {
        outcome = Rules.outcome(game)
        const phase = game.phase
        if (phase === "paused" && !paused)
            pause()
        else if (phase !== "paused" && paused)
            resume()
        onlyScreen(phase === "title" ? "title" : phase === "over" ? "result" : "hud")
        if (phase === "paused")
            showScreen("pause")
        text("Score", "SCORE " + outcome.score.toLocaleString(Qt.locale("en_GB"), "f", 0))
        text("Hull", "HULL " + game.hull + " / " + Rules.HULL)
        text("Clock", String(Math.max(0, Math.ceil((Rules.RUN_STEPS - game.steps) / 60))))
        if (phase === "over") {
            text("Result", outcome.survived ? "RUN SURVIVED" : "HULL BREACHED")
            text("Final", "SCORE " + outcome.score.toLocaleString(Qt.locale("en_GB"), "f", 0))
        }
    }

    function said(words, colour) {
        const message = entity("Message")
        if (!message)
            return
        message.text = words
        message.colour = colour
        if (!Look.reducedMotion) {
            message.size = 28
            tween(message, "size", 22, 0.3, "outCubic")
        }
    }

    /// The stars over the collapse: points far round the scene, placed from a fixed sequence of their own (never the
    /// rules' numbers, so drawing them changes nothing a run reaches), and none below the horizon.
    function placeStars(count) {
        let s = 0x2f6b1c3d
        const next = () => {
            s = (Math.imul(s, 1103515245) + 12345) >>> 0
            return s / 4294967296
        }
        for (let i = 0; i < count; i++) {
            const turn = next() * 6.283185307179586
            const up = 0.08 + 0.9 * next()
            const across = Math.sqrt(1 - up * up)
            const away = 70 + next() * 20
            const size = 0.08 + 0.16 * next() * next()
            spawn("Shape", { name: "Star " + i, form: "sphere", body: "none",
                             position: Qt.vector3d(Math.cos(turn) * across * away, up * away, Math.sin(turn) * across * away),
                             scale: Qt.vector3d(size, size, size), metalness: 0, roughness: 1,
                             colour: "#e8eeff", emissive: "#e8eeff", emissivePower: 1.2 + 5 * next() * next() })
        }
    }

    onStarted: {
        game = Rules.create(scene.playSeed)
        placeStars(180)
        // The lanes' lines and the track's edges, faint ember and brighter.
        for (const x of [-1.5, -0.5, 0.5, 1.5])
            piece({ name: "Line " + x, form: "cube", position: Qt.vector3d(x * laneGap, 0.01, -40),
                    scale: Qt.vector3d(Math.abs(x) > 1 ? 0.09 : 0.04, 0.02, 100), colour: "#a65d35", metalness: 0,
                    roughness: 0.6, emissive: Math.abs(x) > 1 ? "#ff5a14" : "#a65d35",
                    emissivePower: Math.abs(x) > 1 ? 0.9 : 0.55 })
        const made = []
        for (let i = 0; i < barCount; i++)
            made.push(piece({ name: "Bar " + i, form: "cube", position: Qt.vector3d(0, 0.012, near - i * barGap),
                              scale: Qt.vector3d(laneGap * 3, 0.015, 0.05), colour: "#a65d35", metalness: 0,
                              roughness: 0.6, emissive: "#a65d35", emissivePower: 0.35 }))
        bars = made
        // What is thrown: red slabs and turning gold, waiting out of sight until the rules throw them.
        const red = [], gold = []
        for (let i = 0; i < pool; i++) {
            red.push(piece({ name: "Hazard " + i, form: "cube", position: Qt.vector3d(0, -5, far),
                             scale: Qt.vector3d(1.25, 0.7, 0.5), colour: "#3a0c08", metalness: 0.5, roughness: 0.35,
                             emissive: "#ff2a14", emissivePower: 0.9 }))
            gold.push(piece({ name: "Gem " + i, form: "cube", position: Qt.vector3d(0, -5, far),
                              eulerRotation: Qt.vector3d(45, 0, 45), scale: Qt.vector3d(0.42, 0.42, 0.42),
                              colour: "#ffcf5a", metalness: 0.8, roughness: 0.2, emissive: "#ffa818",
                              emissivePower: 0.9 }))
        }
        hazards = red
        gems = gold
        button("Play")
        button("Resume")
        button("Play again")
        show()
    }

    // A, D, the arrows, the pad's d-pad and shoulders, or the Left and Right buttons under a finger; P or Start pauses.
    onPressed: (action) => {
        const lane = game.lane
        Rules.press(game, action)
        if (game.lane !== lane)
            play("../assets/shift.wav", { volume: 0.5, at: atShip() })
        show()
    }

    onStep: {
        const before = game.phase
        const hull = game.hull
        Rules.step(game)
        for (const what of game.said) {
            if (what === "gem") {
                play("../assets/gem.wav", { volume: 0.8, at: atShip() })
                said("GEM CAUGHT  +" + Rules.GEM_POINTS, "#ffd08a")
            } else if (what === "hit") {
                play("../assets/hit.wav", { volume: 0.9, at: atShip() })
                said("HULL BREACHED", "#ff7a5c")
            } else if (what === "held") {
                play("../assets/held.wav", { volume: 0.6, at: atShip() })
            }
        }
        if (before !== "over" && game.phase === "over")
            play(Rules.outcome(game).survived ? "../assets/complete.wav" : "../assets/lost.wav", { volume: 0.9 })
        // The score and the clock change every step: the words follow ten times a second.
        if (game.said.length > 0 || before !== game.phase || hull !== game.hull || game.steps % 6 === 0)
            show()
    }

    // Only looked at: the ship where the rules put it, what is coming where it has come to, and the track running.
    onFrame: (dt) => {
        const shipTo = laneX(game.lane)
        shipX += (shipTo - shipX) * Math.min(1, dt * 14)
        const ship = entity("Ship")
        const engine = entity("Engine")
        const wake = entity("Wake")
        const breached = game.shield > 0 && (Look.reducedMotion || game.shield % 12 < 6)
        if (ship) {
            ship.position.x = shipX
            // It banks into a change of lane, a little.
            ship.eulerRotation.z = Math.max(-10, Math.min(10, (shipTo - shipX) * -7))
            ship.emissive = breached ? "#ff4f36" : "#ffb070"
            ship.emissivePower = breached ? 1.2 : 0.45
        }
        if (engine)
            engine.position.x = shipX
        if (wake)
            wake.position.x = shipX

        // Each thing thrown keeps one piece of its pool, by its number, for as long as it is coming.
        const shown = new Set()
        for (const thing of game.coming) {
            const drawn = (thing.gem ? gems : hazards)[thing.id % pool]
            shown.add(drawn)
            drawn.position = Qt.vector3d(laneX(thing.lane), thing.gem ? 0.55 : 0.36, far + (near - far) * thing.z)
            if (thing.gem && !Look.reducedMotion)
                drawn.eulerRotation.y += 120 * dt
        }
        for (const drawn of hazards.concat(gems))
            if (!shown.has(drawn))
                drawn.position = Qt.vector3d(0, -5, far)

        // The bars run toward the eye with what is thrown; slowly on the title, and not once the run is over.
        const rate = game.phase === "playing" ? Rules.speed(game.steps) * (near - far)
                   : game.phase === "title" && !Look.reducedMotion ? 3 : 0
        travel = (travel + rate * dt) % barGap
        for (let i = 0; i < bars.length; i++)
            bars[i].position.z = near + 1.5 - i * barGap + travel

        const collapse = entity("Collapse")
        if (collapse && !Look.reducedMotion) {
            pulse += dt
            collapse.emissivePower = 0.75 + 0.08 * Math.sin(pulse * 1.3)
        }
    }
}
