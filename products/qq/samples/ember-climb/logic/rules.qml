// Ember Climb's rules: eight embers wait around the forge, the hero gathers them by running and jumping through them,
// and the forge is lit when the last is taken. A fall into a pit puts the hero back where it began. Everything here is
// QQ's logic API, over a scene made of QQ's 2D kinds (DIRECTION P3.10): a level drawn in Tiled (TileMap), sprites
// played from sheets, a Player2D, Triggers spawned here, and the scene's screens. Nothing in the engine knows this game.
//
// It keeps its time in steps (P3.11), and its outcome is how many embers were gathered and in how many steps: a run
// kept with the Player's --record is played again with no window to the same outcome.

import QQ

Logic {
    id: rules

    /// Embers gathered, the steps the climb has taken (60 a second), and whether it is under way.
    property int gathered: 0
    property int took: 0
    property bool climbing: false

    /// What the run came to, for a replay to match.
    outcome: ({ gathered: gathered, lit: gathered === spots.length, steps: took })

    /// Where the embers wait, at the hero's middle: over the floor and over each platform, a little in front of the level.
    readonly property var spots: [Qt.vector3d(4.5, 2.5, 0.05), Qt.vector3d(7.5, 5.5, 0.05), Qt.vector3d(13.5, 8.5, 0.05),
                                  Qt.vector3d(18.5, 11.5, 0.05), Qt.vector3d(25.5, 5.5, 0.05), Qt.vector3d(32.5, 8.5, 0.05),
                                  Qt.vector3d(26.5, 11.5, 0.05), Qt.vector3d(45.5, 13.5, 0.05)]

    /// The hero's look for each thing it does (Player2D's `activity`): its first frame on the sheet, how many, how fast.
    readonly property var looks: ({ idle: [0, 2, 3], run: [4, 4, 12], jump: [2, 1, 0], fall: [3, 1, 0] })
    property string shown: ""

    /// Each ember: a sprite turning in place, and a trigger around it that hears the hero.
    function placeEmbers() {
        for (let i = 0; i < spots.length; ++i) {
            const name = "Ember " + (i + 1)
            const glow = spawn("Sprite", { name: name + " glow", source: "../assets/ember.png", columns: 4, count: 4,
                                           fps: 8, frame: i % 4, position: spots[i] })
            const ember = spawn("Trigger", { name: name, position: spots[i], scale: Qt.vector3d(0.6, 0.6, 1) })
            if (!glow || !ember)
                continue
            ember.entered.connect((other) => {
                if (!other || other.kind !== "Player2D" || removing(ember))
                    return
                remove(ember)
                remove(glow)
                gathered += 1
                tell()
                if (gathered === spots.length)
                    light()
            })
        }
    }

    function tell() {
        const count = entity("Embers")
        if (count)
            count.text = "EMBERS " + gathered + " / " + spots.length
    }

    /// The last ember: the forge is lit, and the time it took is shown.
    function light() {
        climbing = false
        const time = entity("Took")
        if (time)
            time.text = "Climbed in " + Math.floor(took / 3600) + ":" + String(Math.floor(took / 60) % 60).padStart(2, "0")
        onlyScreen("won")
        pause()
    }

    function climb() {
        onlyScreen("hud")
        resume()
        climbing = true
    }

    function button(name, pressed) {
        const b = entity(name)
        if (b)
            b.clicked.connect(pressed)
    }

    onStarted: {
        // The title first, the forge held still behind it.
        pause()
        placeEmbers()
        tell()
        button("Play", climb)
        button("Resume", () => { hideScreen("pause"); resume() })
        button("Again", () => {
            gathered = 0
            took = 0
            tell()
            // The hero back first, so the last ember is not placed where it still stands.
            if (scene.player)
                scene.player.respawn()
            placeEmbers()
            climb()
        })
    }

    // Pause: P, the pad's Start, or the Pause button under a finger; and again to go on.
    onPressed: (action) => {
        if (action !== "pause" || !climbing)
            return
        if (paused) {
            hideScreen("pause")
            resume()
        } else {
            showScreen("pause")
            pause()
        }
    }

    onStep: (dt) => {
        const hero = scene.player
        if (!hero)
            return
        if (climbing)
            took += 1
        // Into a pit, and below the level: back to the start.
        if (hero.feet.y < -1)
            hero.respawn()
        // The look plays what the hero is doing.
        const look = entity("Hero look")
        if (look && hero.activity !== shown) {
            shown = hero.activity
            const frames = looks[shown] ?? looks.idle
            look.count = frames[1]
            look.fps = frames[2]
            look.first = frames[0]
            look.frame = frames[0]
        }
    }
}
