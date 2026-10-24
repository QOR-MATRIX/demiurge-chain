// Orbital (DIRECTION P7.20, ADR-102 decision 2), rebuilt in QQ from ARQADE's 2D game: a satellite circles a gilt core,
// and a lock holds while it crosses the amber capture arc. Ten locks complete the orbit; three misses lose it.
//
// The rules are logic/rules.mjs, the one place they live: ARQADE's service runs the same file to replay a run. This
// logic hands them every event as QQ's tick delivers it (a screen button's click, an action's press, a step) and draws
// what they say: the satellite where the rules put it, the arc where its middle is, the screens for the rules' phase.
// So a run QQ records is the rules' input exactly, and its outcome is theirs (`outcome`). Nothing drawn here decides
// anything, which is why it may use sines and cosines the rules may not.
//
// The orbit is laid in a group turned toward where the eye looks from, its beads, arc and the satellite's trail made
// here as play starts. QQ's world is held still only while the player pauses; on the title and at the end the core
// still glows and its embers rise, and the rules, outside play, take no step. With less motion asked for (Look), the
// core does not turn and the words do not swell; the satellite still moves, since it is the game.
//
// A lock and a miss are heard from the satellite, through QQ's spatial audio (`at`): panned to its side of the orbit and
// heard through the eye. The ends of the orbit and the music are heard everywhere.

import QQ
import "rules.mjs" as Rules

Logic {
    id: orbital

    /// The rules' game: their state, which only they change.
    property var game: null

    /// The orbit as drawn: its radius in the group's units, its beads, the arc's beads and the satellite's trail.
    readonly property real radius: 3.2
    readonly property int beads: 96
    readonly property int arcBeads: 13
    readonly property int trailLength: 12
    property var arc: []
    property var trail: []
    /// The arc's glow at rest: much past this, bloom washes its amber to the satellite's white.
    property real arcGlow: 1

    /// A point of the orbit at `angle`, in the group's space: the orbit stands upright, facing the eye.
    function at(angle, out) {
        return Qt.vector3d(Math.cos(angle) * (radius + (out || 0)), Math.sin(angle) * (radius + (out || 0)), 0)
    }

    /// A bead of light: only looked at, so it has no body (a Shape's is static unless it says otherwise).
    function bead(name, angle, size, colour, glow) {
        return spawn("Shape", { name: name, position: at(angle), scale: Qt.vector3d(size, size, size), form: "sphere",
                                colour: colour, metalness: 0, roughness: 0.4, emissive: colour, emissivePower: glow,
                                body: "none" },
                     entity("Orbit"))
    }

    function text(name, words) {
        const shown = entity(name)
        if (shown)
            shown.text = words
    }

    function button(name, then) {
        const b = entity(name)
        if (b)
            b.clicked.connect(() => { Rules.click(game, name); then(); show() })
    }

    /// The arc where its middle is now.
    function placeArc() {
        for (let i = 0; i < arc.length; i++) {
            const along = game.target + (i / (arc.length - 1) * 2 - 1) * Rules.ARC
            arc[i].position = at(along)
        }
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
        text("Locks", "LOCKS " + game.locks + " / " + Rules.LOCKS_TO_WIN)
        text("Misses", "MISSES " + game.misses + " / " + Rules.MISSES_TO_LOSE)
        if (phase === "over") {
            const won = game.locks >= Rules.LOCKS_TO_WIN
            text("Result", won ? "ORBIT COMPLETE" : "ORBIT LOST")
            text("Score", "SCORE " + (game.locks * Rules.POINTS).toLocaleString(Qt.locale("en_GB"), "f", 0))
        }
    }

    function said(words, colour) {
        const message = entity("Message")
        if (!message)
            return
        message.text = words
        message.colour = colour
        // It swells as it is said, unless the person asked for less motion.
        if (!Look.reducedMotion) {
            message.size = 30
            tween(message, "size", 24, 0.35, "outCubic")
        }
    }

    /// The stars: points far round the scene, each glowing a little differently, placed from a fixed sequence of
    /// their own (never the rules' numbers, so drawing them changes nothing a run reaches).
    function placeStars(count) {
        let s = 0x5f3759df
        const next = () => {
            s = (Math.imul(s, 1103515245) + 12345) >>> 0
            return s / 4294967296
        }
        for (let i = 0; i < count; i++) {
            const turn = next() * Rules.TURN
            const up = next() * 2 - 1
            const across = Math.sqrt(1 - up * up)
            const far = 42 + next() * 14
            const size = 0.06 + 0.12 * next() * next()
            const warm = next()
            spawn("Shape", { name: "Star " + i, form: "sphere",
                             position: Qt.vector3d(Math.cos(turn) * across * far, up * far, Math.sin(turn) * across * far),
                             scale: Qt.vector3d(size, size, size), metalness: 0, roughness: 1,
                             colour: warm > 0.7 ? "#ffd9a8" : "#e8eeff", emissive: warm > 0.7 ? "#ffd9a8" : "#e8eeff",
                             emissivePower: 1.5 + 7 * next() * next(), body: "none" })
        }
    }

    onStarted: {
        game = Rules.create(scene.playSeed)
        placeStars(260)
        // The orbit's beads, faint gilt; the arc's, ember; the satellite's trail, fading behind it.
        for (let i = 0; i < beads; i++)
            bead("Bead " + i, i / beads * Rules.TURN, 0.045, "#8c7a52", 0.6)
        const beadsOfArc = []
        for (let i = 0; i < arcBeads; i++)
            beadsOfArc.push(bead("Arc " + i, 0, 0.12, "#ff7a1a", arcGlow))
        arc = beadsOfArc
        const behind = []
        for (let i = 0; i < trailLength; i++)
            behind.push(bead("Trail " + i, 0, 0.17 * (1 - i / trailLength), "#ffe7bd", 2.5 * (1 - i / trailLength)))
        trail = behind
        placeArc()
        button("Play", () => {})
        button("Resume", () => {})
        button("Play again", () => placeArc())
        show()
    }

    // P, the pad's Start or the Pause button; Space, Enter, the pad's A or the Lock button under a finger.
    onPressed: (action) => {
        Rules.press(game, action)
        show()
    }

    onStep: {
        const before = { locks: game.locks, misses: game.misses, phase: game.phase }
        Rules.step(game)
        const satellite = entity("Satellite")
        const there = satellite ? satellite.scenePosition : null
        if (game.said === "locked") {
            placeArc()
            play("../assets/lock.wav", { volume: 0.9, at: there })
            said("ORBIT LOCKED", "#ffd08a")
            for (const b of arc) {
                b.emissivePower = 6
                tween(b, "emissivePower", arcGlow, 0.5, "outCubic")
            }
        } else if (game.said === "missed") {
            play("../assets/miss.wav", { volume: 0.8, at: there })
            said("SIGNAL MISSED", "#ff7a5c")
            if (satellite) {
                satellite.emissive = "#ff5a3a"
                tween(satellite, "emissive", "#fff3d6", 0.6, "outQuad")
            }
        }
        if (before.phase !== "over" && game.phase === "over")
            play(game.locks >= Rules.LOCKS_TO_WIN ? "../assets/complete.wav" : "../assets/lost.wav", { volume: 0.9 })
        if (game.said !== "" || before.phase !== game.phase)
            show()
    }

    // Only looked at: the satellite and its trail where the rules put it, and the core turning.
    onFrame: (dt) => {
        const satellite = entity("Satellite")
        if (satellite)
            satellite.position = at(game.angle)
        const step = Rules.speed(game.locks) * Rules.STEP * 1.5
        for (let i = 0; i < trail.length; i++)
            trail[i].position = at(game.angle - (i + 1) * step)
        const core = entity("Core")
        if (core && !Look.reducedMotion)
            core.eulerRotation.y += 12 * dt
    }
}
