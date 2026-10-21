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
//
// Its sound (P3.13): the title's music, giving way to the arena's when play starts; a chime where an orb is taken and a
// rumble where the boulder is rolled; and the pause screen turns the music off and on again (its volume group).
//
// Its orbs are a prefab (P3.15), prefabs/orb.qml: a glow and the trigger that hears the player, placed whole and taken
// away whole; the arena's walls are a group.

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

    /// An orb: the prefab, a glowing sphere that nothing touches and a trigger around it that hears the player.
    function placeOrb() {
        const at = spots[placed % spots.length]
        placed += 1
        const orb = spawn("Prefab", { name: "Orb " + placed, source: "../prefabs/orb.qml", position: at })
        const glow = orb ? orb.part("Glow") : null
        const pickup = orb ? orb.part("Pickup") : null
        if (!glow || !pickup)
            return
        tween(glow, "scale", Qt.vector3d(0.6, 0.6, 0.6), 0.45, "outBack")
        orbsOut += 1
        pickup.entered.connect((other) => {
            if (!other || other.kind !== "Player" || removing(pickup))
                return
            score += 1
            orbsOut -= 1
            const shown = entity("Score")
            if (shown) {
                shown.text = "ORBS " + score
                tween(shown, "size", 44, 0.12, "outQuad", () => tween(shown, "size", 34, 0.25, "inOutQuad"))
            }
            // Heard no more at once; seen shrinking away, and then gone whole.
            remove(pickup)
            play("../assets/chime.wav", { at: orb.position })
            tween(glow, "scale", Qt.vector3d(0.01, 0.01, 0.01), 0.25, "inCubic", () => remove(orb))
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
                const from = rock.physics ? rock.physics.scenePosition : rock.position
                const toward = player.feet.minus(from)
                rock.push(Qt.vector3d(toward.x, 0, toward.z).normalized().times(6))
                play("../assets/rumble.wav", { at: from, volume: 0.9 })
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
        button("Play", () => {
            onlyScreen("hud")
            resume()
            const music = entity("Music")
            if (music)
                music.source = "../assets/arena.wav"  // the title's track fades out as this one fades in
        })
        button("Resume", () => { hideScreen("pause"); resume() })
        button("Music switch", () => {
            const on = volumeOf("music") > 0
            setVolume("music", on ? 0 : 1)
            entity("Music switch").text = on ? "Music: off" : "Music: on"
        })
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
            if (e.kind === "Prefab" && e.name.startsWith("Orb "))
                e.eulerRotation.y += 60 * dt
    }
}
