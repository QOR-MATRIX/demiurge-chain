// Orb Run's rules: orbs appear around the arena, the player gathers them by walking through, and a boulder rolls at the
// player now and then. Everything here is QQ's logic API (DIRECTION P3.7): spawn and remove, a Trigger's `entered`,
// timers on the game's clock, and a push on a dynamic body. Nothing in the engine knows this game.
//
// Its interface is the scene's screen parts (P3.8): it opens on its title with the world paused, Play starts it, the
// score is on the HUD, and Pause and Resume hold and release the world.

import QQ

Logic {
    id: rules

    /// Orbs gathered, and the most there have been at once.
    property int score: 0
    property int orbsOut: 0

    /// Where orbs appear, in turn: around the arena, a metre up.
    readonly property var spots: [Qt.vector3d(0, 1, -4), Qt.vector3d(5, 1, -2), Qt.vector3d(-5, 1, -2),
                                  Qt.vector3d(4, 1, 4), Qt.vector3d(-4, 1, 4), Qt.vector3d(0, 1, -8)]
    property int placed: 0

    /// An orb: a glowing sphere that nothing touches, and a trigger around it that hears the player.
    function placeOrb() {
        const at = spots[placed % spots.length]
        placed += 1
        const name = "Orb " + placed
        const glow = spawn("Shape", { name: name + " glow", form: "sphere", body: "none", position: at,
                                      scale: Qt.vector3d(0.6, 0.6, 0.6), colour: "#ffd27a", emissive: "#ffb340",
                                      emissivePower: 4, roughness: 0.2 })
        const orb = spawn("Trigger", { name: name, form: "sphere", position: at, scale: Qt.vector3d(1.4, 1.4, 1.4) })
        if (!glow || !orb)
            return
        orbsOut += 1
        orb.entered.connect((other) => {
            if (!other || other.kind !== "Player" || removing(orb))
                return
            score += 1
            orbsOut -= 1
            const shown = entity("Score")
            if (shown)
                shown.text = "ORBS " + score
            remove(orb)
            remove(glow)
            after(1.5, placeOrb)
        })
    }

    function button(name, pressed) {
        const b = entity(name)
        if (b)
            b.clicked.connect(pressed)
    }

    onStarted: {
        // The title first, the world held still behind it.
        pause()
        button("Play", () => { onlyScreen("hud"); resume() })
        button("Resume", () => { hideScreen("pause"); resume() })
        placeOrb()
        placeOrb()
        // The boulder rolls at the player every few seconds.
        every(4, () => {
            const boulder = entity("Boulder")
            const player = scene ? scene.player : null
            if (!boulder || !player)
                return
            const toward = player.feet.minus(boulder.physics ? boulder.physics.scenePosition : boulder.position)
            boulder.push(Qt.vector3d(toward.x, 0, toward.z).normalized().times(6))
        })
    }

    // Pause: P, the pad's Start, or the Pause button under a finger (its action), and again to go on (P3.9).
    onPressed: (action) => {
        if (action !== "pause" || scene.shown.indexOf("title") >= 0)
            return
        if (paused) {
            hideScreen("pause")
            resume()
        } else {
            showScreen("pause")
            pause()
        }
    }

    // The orbs turn slowly, so they read as things to take.
    onFrame: (dt) => {
        for (const e of SceneIO.entities(scene))
            if (e.name.endsWith(" glow"))
                e.eulerRotation.y += 60 * dt
    }
}
