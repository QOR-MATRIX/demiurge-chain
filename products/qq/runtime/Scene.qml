// A QQ scene: the root of everything a creator places, and the look of the world it is shown in. It is what a scene
// file holds (`scenes/<name>.qml`, ADR-083 decision 3): the Studio writes it with SceneIO in one canonical layout, one
// property per line, so a change is a small true diff in Projects.
//
// Every entity in a scene declares `kind` (its type's name, as written in the file) and `fields` (the properties a
// file keeps, in the order it keeps them). The writer needs nothing else to know about a type.
//
// Its interface (P3.8): the screen kinds (ScreenText, ScreenBar, ScreenButton, ScreenImage, ScreenPanel) each belong to
// a named screen, and `screens` names those shown when play starts ("hud", or "title" for a game that opens on its
// title). Logic shows, hides and swaps them, and can pause the world beneath them.
//
// A scene plays when `playing` is set (DIRECTION P3.2): a physics world starts, in metres and seconds, and each entity
// becomes what it is in play (a body, a character, a sound). The Studio plays a copy built from the scene's text, so the
// scene being edited is never touched by play.
//
// Play moves in ticks of 1/60 s, whatever the frame rate (P3.11, ADR-105 decision 6). A frame's time is owed and paid
// in whole ticks, at most eight a frame: beyond that the game slows rather than leaps. Each tick takes the player's
// hands (Input.takeStep) and the screen buttons clicked, delivers the clicks and every press and release, and then,
// unless the world is paused, steps: 2D players move by QQ's own kinematics, triggers hear them, models' clips move on
// (P3.12), and logic steps (its timers, tweens and state machines, then onStep). What logic removes during a tick is
// gone when it ends. A run is its seed and what each tick took, so
// a RunRecord keeps it and a replay, with no window and no physics world, reaches the same outcome (run.h).

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.Physics

Node {
    id: root

    readonly property string kind: "Scene"
    readonly property var fields: ["name", "skyTop", "skyHorizon", "groundHorizon", "groundBottom", "skyLight",
                                   "exposure", "bloom", "gravity", "screens", "view", "pixelsPerUnit", "viewHeight",
                                   "pixelArt", "seed"]
    readonly property var ranges: ({ skyLight: [0, 3], exposure: [0.2, 3], gravity: [0, 30], pixelsPerUnit: [1, 512],
                                     viewHeight: [1, 200], seed: [0, 2147483647] })
    readonly property var choices: ({ view: ["3d", "2d"] })

    property string name: "untitled"
    property color skyTop: "#0b1430"
    property color skyHorizon: "#c7623a"
    property color groundHorizon: "#2a1a1a"
    property color groundBottom: "#07080c"
    property real skyLight: 0.75
    property real exposure: 1.0
    property bool bloom: true
    /// How fast things fall, in metres a second per second: 9.81 on Earth, 1.62 on the Moon.
    property real gravity: 9.81
    /// The screens shown when play starts, by name, separated by commas: "hud", or "title" for a title screen.
    property string screens: "hud"

    /// 3d, or 2d (P3.10): the same world seen flat, through an orthographic camera on the plane z = 0, scaled as
    /// `pixelArt` says. Things move on that plane, and the eye follows a Player2D.
    property string view: "3d"
    /// Pixels of art to a unit: a 16-pixel sprite is one unit across at 16. Used by a 2D view and by sprites.
    property real pixelsPerUnit: 16
    /// How many units tall a 2D view shows at the least: the art is drawn as large as fits.
    property real viewHeight: 12
    /// Whether a 2D scene's art is pixel art: scaled by whole numbers only, so every art pixel is the same square of
    /// screen pixels, and shown in exactly the colours it was painted in. Off, the view is exactly `viewHeight` units
    /// tall and has the 3D look's tonemapping and bloom, for art that is drawn or rendered rather than placed in pixels.
    property bool pixelArt: true
    /// The seed this scene's random numbers start from (P3.11): the same seed gives the same numbers on every machine. 0
    /// draws a new one each time play starts; a run started by ARQADE, or replayed, brings its own.
    property int seed: 0

    /// The screens shown now: what `screens` names until logic changes it. Not a field.
    property var shown: screens.split(",").map(s => s.trim()).filter(s => s !== "")
    onScreensChanged: shown = screens.split(",").map(s => s.trim()).filter(s => s !== "")

    function showScreen(name) {
        if (shown.indexOf(name) < 0)
            shown = shown.concat([name])
    }
    function hideScreen(name) {
        shown = shown.filter(s => s !== name)
    }
    /// Show `name` and nothing else: from a title to the game, from the game to its end.
    function onlyScreen(name) {
        shown = [name]
    }

    /// Where a flat view meant to centre on `point` centres so that, `halfWidth` and `halfHeight` units each way, it
    /// stays inside the tile map marked `bounds` (P3.10): the eye stops at a level's edges rather than show past them.
    /// A view wider or taller than the map is centred on it that way. With no such map, `point`.
    function keptInView(point, halfWidth, halfHeight) {
        const map = SceneIO.entities(root).find(e => e.kind === "TileMap" && e.bounds && e.count > 0)
        if (!map)
            return point
        const left = map.position.x
        const right = left + map.extent.width
        const top = map.position.y
        const bottom = top - map.extent.height
        const x = right - left <= 2 * halfWidth ? (left + right) / 2
                                                : Math.min(Math.max(point.x, left + halfWidth), right - halfWidth)
        const y = top - bottom <= 2 * halfHeight ? (top + bottom) / 2
                                                 : Math.min(Math.max(point.y, bottom + halfHeight), top - halfHeight)
        return Qt.vector3d(x, y, point.z)
    }

    /// Whether the world is paused: physics, logic and the player hold still, and screens still answer. Not a field.
    property bool paused: false

    // ── the game's clock (P3.11) ──

    /// Ticks a second; the ticks this play has had, and its steps (the ticks not paused). Not fields.
    readonly property int stepRate: 60
    property int ticks: 0
    property int steps: 0
    /// The seed this play's numbers come from: `seed`, a new one when that is 0, or one given before play (a replay's,
    /// or the Player's --seed).
    property int playSeed: 0
    /// Ticks come from frames (play); off, only tick() and advance() move the game (a replay, a check).
    property bool autoAdvance: true
    /// Played from `recorder`'s hands and clicks, with no physics world.
    property bool replaying: false
    /// Frames tick no further than this many ticks (the Player's --ticks); 0, no limit.
    property int tickLimit: 0
    /// What keeps this run (a RunRecord), or null.
    property QtObject recorder: null
    /// The play's random numbers (Logic.random, randomInt).
    readonly property QtObject random: rng

    property real _owed: 0
    property var _clicks: []
    property var _doomed: []
    property bool _ticking: false

    Random { id: rng }

    /// The sounds logic plays once (Logic.play, P3.13). Not an entity.
    readonly property QtObject _voices: voices
    Voices {
        id: voices
        scene: root
    }

    onPlayingChanged: {
        if (!playing)
            return
        ticks = 0
        steps = 0
        _owed = 0
        _clicks = []
        _doomed = []
        if (!replaying) {
            if (playSeed === 0)
                playSeed = seed !== 0 ? seed : SceneIO.freshSeed()
            Input.takeStep()  // presses from before play are not this run's
        }
        rng.reset(playSeed)
        if (recorder && !replaying) {
            recorder.clear()
            recorder.scene = name
            recorder.seed = playSeed
        }
    }

    /// Whether play can tick: every Behaviour's logic is built (or could not be), and every model has loaded (or failed).
    function ready() {
        return SceneIO.entities(root).every(e => (e.kind !== "Behaviour" && e.kind !== "Prop") || e.ready)
    }

    /// Every Behaviour whose logic could not be built, as "name: why".
    function logicErrors() {
        return SceneIO.entities(root).filter(e => e.kind === "Behaviour" && e.error !== "").map(e => e.name + ": " + e.error)
    }

    /// Owe `seconds` more and pay what is owed in whole ticks, eight at most: how many ticks ran.
    function advance(seconds) {
        if (!playing || !ready()) {
            _owed = 0
            return 0
        }
        _owed = Math.min(_owed + Math.max(0, seconds), 8 / stepRate)
        let ran = 0
        while (_owed >= 1 / stepRate - 1e-9) {
            if (tickLimit > 0 && ticks >= tickLimit) {
                _owed = 0
                break
            }
            _owed -= 1 / stepRate
            tick()
            ran += 1
        }
        return ran
    }

    /// One tick.
    function tick() {
        if (!playing)
            return
        const hands = replaying && recorder ? recorder.handsAt(ticks) : Input.takeStep()
        if (replaying)
            Input.replayHands(hands)
        const clicks = replaying && recorder ? recorder.clicksAt(ticks) : _clicks
        _clicks = []
        if (recorder && !replaying)
            recorder.capture(ticks, hands, clicks)
        ticks += 1
        _ticking = true
        try {
            let list = SceneIO.entities(root)
            for (const clicked of clicks) {
                const button = list.find(e => e.kind === "ScreenButton" && e.name === clicked)
                if (button)
                    button.clicked()
            }
            for (const edge of hands.edges)
                for (const e of list)
                    if (e._hear)
                        e._hear(edge[0], edge[1])
            if (!paused) {
                steps += 1
                const dt = 1 / stepRate
                list = SceneIO.entities(root)
                for (const e of list)
                    if (e._stepMove)
                        e._stepMove(dt)
                _stepTriggers(list)
                for (const e of list)
                    if (e._stepAnimate)
                        e._stepAnimate(dt)
                for (const e of list)
                    if (e._stepLogic)
                        e._stepLogic(dt)
            }
        } finally {
            _ticking = false
            const doomed = _doomed
            _doomed = []
            for (const e of doomed)
                SceneIO.remove(e)
        }
    }

    /// A screen button pressed in play: delivered at the next tick, and kept by the record.
    function _click(button) {
        if (playing && !replaying)
            _clicks = _clicks.concat([button.name])
    }

    /// Logic removed `entity` during a tick: it goes as the tick ends.
    function _doom(entity) {
        _doomed = _doomed.concat([entity])
    }

    // Triggers hear 2D players by QQ's own measure, the same in a replay with no physics world.
    function _stepTriggers(list) {
        const movers = list.filter(e => e.kind === "Player2D")
        if (movers.length === 0)
            return
        for (const t of list) {
            if (t.kind !== "Trigger")
                continue
            const before = t._qqInside
            const now = movers.filter(m => SceneIO.overlaps(t, m))
            t._qqInside = now
            for (const m of before)
                if (now.indexOf(m) < 0)
                    t.exited(m)
            for (const m of now)
                if (before.indexOf(m) < 0)
                    t.entered(m)
        }
    }

    /// What this run has come to: each Behaviour's logic's `outcome`, by the Behaviour's name, as JSON.
    function outcomeText() {
        const result = {}
        for (const e of SceneIO.entities(root))
            if (e.kind === "Behaviour" && e.logic && e.logic.outcome !== undefined)
                result[e.name] = e.logic.outcome
        return JSON.stringify(result)
    }

    /// Close `recorder` on the run so far: its length and its outcome.
    function finishRecord() {
        if (!recorder)
            return
        recorder.ticks = ticks
        recorder.outcome = JSON.parse(outcomeText())
    }

    FrameAnimation {
        running: root.playing && root.autoAdvance && !root.replaying
        onTriggered: root.advance(frameTime)
    }

    /// Whether the scene plays: not a field, so never written to a file.
    property bool playing: false

    /// The player, once play has made one: what a camera follows.
    property QtObject player: null

    /// The physics world while playing, or null.
    readonly property QtObject physicsWorld: physics.object

    // One physics world, made when play starts and gone when it stops. QQ's unit is the metre.
    Instantiator {
        id: physics
        active: root.playing && !root.replaying
        delegate: PhysicsWorld {
            scene: root
            running: !root.paused
            gravity: Qt.vector3d(0, -root.gravity, 0)
            typicalLength: 1
            typicalSpeed: 10
            defaultDensity: 1000
        }
    }
}
