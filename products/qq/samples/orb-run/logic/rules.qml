// Orb Run's rules: orbs appear around the arena, the player gathers them by walking through, and a boulder rolls at the
// player now and then. Everything here is QQ's logic API (DIRECTION P3.7): spawn and remove, a Trigger's `entered`,
// timers on the game's clock, and a push on a dynamic body. Nothing in the engine knows this game.
//
// Its interface is the scene's screen parts (P3.8): it opens on its title with the world paused, Play starts it, the
// score is on the HUD, and Pause and Resume hold and release the world.
//
// Its motion (P3.12): orbs grow in with a spring and shrink away when taken, the score swells as it counts, the arena's
// lanterns (an animated glTF model, assets/lantern.gltf) flare as each orb is taken and then go back to swaying, and the
// boulder is a state machine: resting, then rolled at the player, then resting again.

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
                                      scale: Qt.vector3d(0.05, 0.05, 0.05), colour: "#ffd27a", emissive: "#ffb340",
                                      emissivePower: 4, roughness: 0.2 })
        const orb = spawn("Trigger", { name: name, form: "sphere", position: at, scale: Qt.vector3d(1.4, 1.4, 1.4) })
        if (!glow || !orb)
            return
        tween(glow, "scale", Qt.vector3d(0.6, 0.6, 0.6), 0.45, "outBack")
        orbsOut += 1
        orb.entered.connect((other) => {
            if (!other || other.kind !== "Player" || removing(orb))
                return
            score += 1
            orbsOut -= 1
            const shown = entity("Score")
            if (shown) {
                shown.text = "ORBS " + score
                tween(shown, "size", 44, 0.12, "outQuad", () => tween(shown, "size", 34, 0.25, "inOutQuad"))
            }
            remove(orb)
            tween(glow, "scale", Qt.vector3d(0.01, 0.01, 0.01), 0.25, "inCubic", () => remove(glow))
            flare()
            after(1.5, placeOrb)
        })
    }

    /// The lanterns flare once, then sway again.
    function flare() {
        for (const name of ["Lantern west", "Lantern east"]) {
            const lantern = entity(name)
            if (!lantern)
                continue
            lantern.clipLoops = false
            lantern.clip = "flare"
        }
        after(0.8, () => {
            for (const name of ["Lantern west", "Lantern east"]) {
                const lantern = entity(name)
                if (lantern) {
                    lantern.clipLoops = true
                    lantern.clip = "idle"
                }
            }
        })
    }

    /// The boulder: rests three and a half seconds, is rolled at the player, rolls half a second, and rests again.
    readonly property var boulder: machine({ start: "resting",
        resting: { step: () => { if (boulder.time >= 3.5) boulder.go("rolling") } },
        rolling: {
            enter: () => {
                const rock = entity("Boulder")
                const player = scene ? scene.player : null
                if (!rock || !player)
                    return
                const toward = player.feet.minus(rock.physics ? rock.physics.scenePosition : rock.position)
                rock.push(Qt.vector3d(toward.x, 0, toward.z).normalized().times(6))
            },
            step: () => { if (boulder.time >= 0.5) boulder.go("resting") }
        } })

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
