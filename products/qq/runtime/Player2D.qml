// The player of a 2D game (DIRECTION P3.10, ADR-105): a figure on the plane z = 0 under the player's hands (Input), as
// a platformer's hero or a top-down one. In play it is a character controller, `width` by `height` units, stopped by
// tile maps and bodies:
//
//   platform   move left and right, jump (the jump action), fall under the scene's gravity, run with the run action
//   top-down   move freely across the plane, run with the run action; no gravity
//
// Its look is a Sprite in the scene named by `look`, carried with its feet at the player's and turned to face where it
// goes; without one, a plain figure in `colour`. `activity` says what it is doing (idle, run, jump or fall), for logic
// to play the look's frames by. A 2D scene's eye follows it. `position` is where its feet are.

import QtQuick
import QtQuick3D
import QtQuick3D.Physics
import QQ

Node {
    id: player

    readonly property string kind: "Player2D"
    readonly property var fields: ["name", "position", "style", "width", "height", "speed", "jumpHeight", "colour", "look"]
    readonly property var ranges: ({ width: [0.1, 20], height: [0.1, 20], speed: [0.1, 40], jumpHeight: [0, 40] })
    readonly property var choices: ({ style: ["platform", "top-down"] })

    property string name: "Player"
    /// platform or top-down.
    property string style: "platform"
    /// Units: the figure's size, which is what collides.
    property real width: 0.75
    property real height: 1.0
    /// Units a second, walking; running is faster.
    property real speed: 5
    /// How high a jump lifts the feet, in units.
    property real jumpHeight: 3
    property color colour: "#ff6a00"
    /// The Sprite it carries as its look, by name; empty for a plain figure.
    property string look: ""

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// The character controller play made, or null.
    property Node physics: null

    /// idle, run, jump or fall: what it is doing, for its look's frames.
    property string activity: "idle"
    /// Whether it faces left.
    property bool facingLeft: false

    readonly property real runFactor: 1.6
    /// How far PhysX keeps a character from what it touches (its contact offset, which Qt Quick 3D Physics leaves at
    /// PhysX's 0.1). The capsule is built that much smaller all round, so what collides is `width` by `height` exactly
    /// and the feet are on the floor, not a pixel or two above it.
    readonly property real skin: 0.1
    property real verticalSpeed: 0
    property bool jumpHeld: false
    /// Where it is put back by respawn(), and when it falls out of the world: its centre at the start of play.
    property vector3d start

    /// Where its feet are, and what the eye centres on.
    readonly property vector3d feet: physics ? physics.scenePosition.minus(Qt.vector3d(0, height / 2, 0)) : position
    readonly property vector3d lookAt: feet.plus(Qt.vector3d(0, height / 2, 0))
    readonly property vector3d eye: lookAt

    readonly property QtObject sprite: live && look !== "" && parent
                                       ? SceneIO.entities(parent).find(e => e.name === player.look) ?? null : null

    /// Back to `start` (where it began, unless logic has moved it there since, a checkpoint), standing still.
    function respawn() {
        if (!physics)
            return
        verticalSpeed = 0
        physics.teleport(start)
    }

    function makeBody() {
        if (!live || physics)
            return
        start = Qt.vector3d(position.x, position.y + height / 2, 0)
        physics = character.createObject(parent, { position: start })
        if (!parent.player)
            parent.player = player
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    Component {
        id: character
        CharacterController {
            readonly property QtObject entity: player
            sendContactReports: true
            sendTriggerReports: true
            // Upright: as wide as the figure and as tall, with the skin PhysX adds around it.
            collisionShapes: CapsuleShape {
                diameter: Math.max(0.05, player.width - 2 * player.skin)
                height: Math.max(0, player.height - 2 * player.skin - diameter)
            }
            gravity: Qt.vector3d(0, 0, 0)  // the fall and the jump are the player's own, below
            midAirControl: true
        }
    }

    FrameAnimation {
        running: player.live && player.physics !== null && !player.parent.paused
        onTriggered: {
            const body = player.physics
            const dt = Math.min(frameTime, 0.05)
            const v = player.speed * (Input.run ? player.runFactor : 1)
            const move = Input.move
            let across = move.x * v
            let up = 0
            if (player.style === "top-down") {
                up = move.y * v
                player.verticalSpeed = 0
            } else {
                const g = player.parent.gravity
                const grounded = (body.collisions & CharacterController.Down) !== 0
                if (grounded) {
                    if (player.verticalSpeed < 0)
                        player.verticalSpeed = -2  // pressed to the ground, so a step down is followed
                    if (Input.jump && !player.jumpHeld && g > 0)
                        player.verticalSpeed = Math.sqrt(2 * g * player.jumpHeight)
                } else {
                    player.verticalSpeed -= g * dt
                    if ((body.collisions & CharacterController.Up) !== 0 && player.verticalSpeed > 0)
                        player.verticalSpeed = 0
                }
                up = player.verticalSpeed
                player.activity = !grounded ? (player.verticalSpeed > 0 ? "jump" : "fall")
                             : Math.abs(across) > 0.01 ? "run" : "idle"
            }
            player.jumpHeld = Input.jump
            if (player.style === "top-down")
                player.activity = Math.abs(across) + Math.abs(up) > 0.01 ? "run" : "idle"
            if (across < -0.01)
                player.facingLeft = true
            else if (across > 0.01)
                player.facingLeft = false
            body.movement = Qt.vector3d(across, up, 0)

            // On the plane, always; and back to the start if fallen out of the world.
            const at = body.scenePosition
            if (Math.abs(at.z) > 0.001)
                body.teleport(Qt.vector3d(at.x, at.y, 0))
            if (at.y < -60) {
                player.verticalSpeed = 0
                body.teleport(player.start)
            }

            // The look goes where the feet go, at its own depth (in front of the level, or behind it).
            const s = player.sprite
            if (s) {
                s.position = Qt.vector3d(player.feet.x, player.feet.y + s.size.y * s.scale.y / 2, s.position.z)
                s.flip = player.facingLeft
            }
        }
    }

    // A plain figure, when it carries no Sprite: a rounded block in its colour with a lit band where it faces.
    Node {
        visible: player.look === ""
        parent: player.physics ?? player
        position: Qt.vector3d(0, player.physics ? 0 : player.height / 2, 0)
        Model {
            source: "#Cube"
            scale: Qt.vector3d(player.width / 100, player.height / 100, player.width / 100)
            materials: PrincipledMaterial { baseColor: player.colour; roughness: 0.4; lighting: PrincipledMaterial.NoLighting }
            pickable: true
        }
        Model {
            source: "#Cube"
            position: Qt.vector3d((player.facingLeft ? -1 : 1) * player.width * 0.25, player.height * 0.22, 0.01)
            scale: Qt.vector3d(player.width * 0.3 / 100, player.height * 0.12 / 100, player.width / 100)
            materials: PrincipledMaterial { baseColor: "#0a1a24"; emissiveFactor: Qt.vector3d(0.25, 1.6, 3.0); lighting: PrincipledMaterial.NoLighting }
            pickable: true
        }
    }
}
