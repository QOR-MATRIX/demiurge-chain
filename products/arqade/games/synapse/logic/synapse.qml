// Synapse (DIRECTION P7.20, ADR-102 decision 2), rebuilt in QQ from ARQADE's 2D game: four nodes of a mind, Origin,
// Echo, Pulse and Aether, light in a signal that the player repeats; each signal repeated grows by one node, and eight
// master it. One node out of turn loses the signal.
//
// The rules are logic/rules.mjs, the one place they live: ARQADE's service runs the same file to replay a run. This
// logic hands them every event as QQ's tick delivers it (a screen button's click, an action's press, a step) and draws
// what they say: the node the signal shows, the node a player presses, the screens for the rules' phase. So a run QQ
// records is the rules' input exactly, and its outcome is theirs (`outcome`). Nothing drawn here decides anything.
//
// The nodes stand in a gentle arc in a group turned toward where the eye looks from, each on a ring of its own colour,
// with dendrites reaching out from it and filaments joining it to the next, all made here as play starts. A node lit by
// the signal, or pressed, glows, swells and throws sparks, and sounds its note. With less motion asked for (Look), it
// glows and sounds but does not swell, and the words do not swell.
//
// Each node's note is heard from the node, through QQ's spatial audio (`at`), so the signal moves from left to right as
// it is heard as well as seen; a node out of turn is heard from where it was pressed. A signal repeated, the game
// mastered and the music are heard everywhere.

import QQ
import "rules.mjs" as Rules

Logic {
    id: synapse

    /// The rules' game: their state, which only they change.
    property var game: null

    /// Each node's colour, as its button wears it.
    readonly property var colours: ["#e8b860", "#ff7a3a", "#4fc8c0", "#9a86ff"]
    readonly property real spacing: 2.6
    readonly property real size: 1.25
    property var nodes: []
    property var rings: []
    property var sparks: []
    /// How lit each node is as drawn, from 0 to 1: the signal holds it at 1, and a press sets it there to fade.
    property var glow: [0, 0, 0, 0]
    /// The node the signal showed at the last step, so each lighting sounds once.
    property int sounding: -1

    /// Node `i`'s place in the group: an arc, its ends a little lower.
    function at(i) {
        const k = i - 1.5
        return Qt.vector3d(k * spacing, 0.4 - 0.12 * k * k, 0)
    }

    /// Something of the scene's look, made here: only looked at, so it has no body.
    function piece(fields) {
        fields.body = "none"
        return spawn("Shape", fields, entity("Mind"))
    }

    /// A thin rod from `from` to `to`, in the group's plane: a dendrite or a filament.
    function rod(name, from, to, width, colour, power) {
        const dx = to.x - from.x, dy = to.y - from.y
        const length = Math.sqrt(dx * dx + dy * dy)
        return piece({ name: name, form: "cylinder", position: Qt.vector3d((from.x + to.x) / 2, (from.y + to.y) / 2, 0),
                       eulerRotation: Qt.vector3d(0, 0, Math.atan2(dy, dx) * 180 / Math.PI - 90),
                       scale: Qt.vector3d(width, length, width), colour: "#141822", metalness: 0.2, roughness: 0.4,
                       emissive: colour, emissivePower: power })
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
        text("Length", "SIGNAL " + game.signal.length + " / " + Rules.LONGEST)
        text("Stage", game.stage === "watch" ? "OBSERVE THE SIGNAL"
                                             : "REPEAT " + game.cursor + " / " + game.signal.length)
        if (phase === "over") {
            text("Result", game.mastered ? "SIGNAL MASTERED" : "SIGNAL LOST")
            text("Final", "SCORE " + game.score.toLocaleString(Qt.locale("en_GB"), "f", 0))
        }
    }

    /// Node `node` glows at once and fades, and sounds its note.
    function light(node) {
        const now = glow.slice()
        now[node] = 1
        glow = now
        play("../assets/node" + (node + 1) + ".wav", { volume: 0.8, at: nodes[node] ? nodes[node].scenePosition : null })
    }

    /// The stars: points far round the scene, placed from a fixed sequence of their own (never the rules' numbers).
    function placeStars(count, next) {
        for (let i = 0; i < count; i++) {
            const turn = next() * 6.283185307179586
            const up = next() * 1.6 - 0.6
            const across = Math.sqrt(1 - up * up)
            const away = 60 + next() * 20
            const size = 0.07 + 0.14 * next() * next()
            spawn("Shape", { name: "Star " + i, form: "sphere", body: "none",
                             position: Qt.vector3d(Math.cos(turn) * across * away, up * away, Math.sin(turn) * across * away),
                             scale: Qt.vector3d(size, size, size), metalness: 0, roughness: 1,
                             colour: "#dfe8ff", emissive: "#dfe8ff", emissivePower: 1 + 4 * next() * next() })
        }
    }

    onStarted: {
        game = Rules.create(scene.playSeed)
        let s = 0x6d2b79f5
        const next = () => {
            s = (Math.imul(s, 1103515245) + 12345) >>> 0
            return s / 4294967296
        }
        placeStars(160, next)
        const made = [], under = [], thrown = []
        for (let i = 0; i < Rules.NODES.length; i++) {
            const p = at(i)
            made.push(piece({ name: Rules.NODES[i], form: "sphere", position: p, scale: Qt.vector3d(size, size, size),
                              colour: "#141822", metalness: 0.3, roughness: 0.15, emissive: colours[i],
                              emissivePower: 0.12 }))
            under.push(piece({ name: Rules.NODES[i] + " ring", form: "cylinder", position: Qt.vector3d(p.x, p.y - 1.05, 0),
                               scale: Qt.vector3d(1.5, 0.04, 1.5), colour: "#0c0e14", metalness: 0.5, roughness: 0.3,
                               emissive: colours[i], emissivePower: 0.3 }))
            thrown.push(spawn("Emitter", { name: Rules.NODES[i] + " sparks", position: p, colour: colours[i], rate: 2,
                                           life: 1.2, size: 0.06, speed: 0.9, spread: 1.2 }, entity("Mind")))
            // Its dendrites, reaching out and away from the others.
            for (let d = 0; d < 6; d++) {
                const turn = (d / 6 + next() * 0.1) * 6.283185307179586
                const reach = 0.9 + 0.7 * next()
                rod(Rules.NODES[i] + " dendrite " + d,
                    Qt.vector3d(p.x + Math.cos(turn) * 0.6, p.y + Math.sin(turn) * 0.6, 0),
                    Qt.vector3d(p.x + Math.cos(turn) * (0.6 + reach), p.y + Math.sin(turn) * (0.6 + reach), 0),
                    0.025, colours[i], 0.25)
            }
            if (i > 0)
                rod("Filament " + i, at(i - 1), p, 0.04, "#c9a45c", 0.35)
        }
        nodes = made
        rings = under
        sparks = thrown
        button("Play")
        button("Resume")
        button("Play again")
        show()
    }

    // 1 to 4, the pad's X, Y, B and A or its d-pad, or the buttons under a finger; P or Start pauses.
    onPressed: (action) => {
        const from = game.said.length
        Rules.press(game, action)
        for (const said of game.said.slice(from)) {
            light(said.node)
            if (said.what === "wrong")
                play("../assets/wrong.wav", { volume: 0.8, at: nodes[said.node] ? nodes[said.node].scenePosition : null })
            else if (said.what === "signal")
                play("../assets/signal.wav", { volume: 0.6 })
            else if (said.what === "mastered")
                play("../assets/mastered.wav", { volume: 0.8 })
        }
        show()
    }

    onStep: {
        const stage = game.stage
        Rules.step(game)
        const lit = Rules.lit(game)
        if (lit !== -1 && lit !== sounding)
            light(lit)
        sounding = lit
        if (stage !== game.stage)
            show()
    }

    // Only looked at: each node's glow, held while the signal shows it and fading after a press.
    onFrame: (dt) => {
        const lit = Rules.lit(game)
        const now = glow.slice()
        for (let i = 0; i < nodes.length; i++) {
            now[i] = lit === i ? 1 : Math.max(0, now[i] - dt * 2.5)
            nodes[i].emissivePower = 0.12 + 0.65 * now[i]
            const swell = Look.reducedMotion ? 1 : 1 + 0.14 * now[i]
            nodes[i].scale = Qt.vector3d(size * swell, size * swell, size * swell)
            rings[i].emissivePower = 0.3 + 0.4 * now[i]
            sparks[i].rate = 2 + 70 * now[i]
        }
        glow = now
    }
}
