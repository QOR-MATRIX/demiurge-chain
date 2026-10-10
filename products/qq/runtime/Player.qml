// The player: a figure that walks, runs and jumps through the world under the player's hands (Input), with a camera
// that follows from behind. In play it is a character controller, a capsule 1.8 m tall that climbs low steps, is
// stopped by walls and falls off edges; the camera turns with a drag, the arrow keys or the right stick.
//
// `position` is where its feet are.

import QtQuick
import QtQuick3D
import QtQuick3D.Physics
import QQ

Node {
    id: player

    readonly property string kind: "Player"
    readonly property var fields: ["name", "position", "eulerRotation", "colour", "speed", "jumpHeight"]
    readonly property var ranges: ({ speed: [0.5, 15], jumpHeight: [0, 5] })

    property string name: "Player"
    property color colour: "#ff6a00"
    /// Walking speed, metres a second; running is faster.
    property real speed: 4.5
    /// How high a jump lifts the feet, metres.
    property real jumpHeight: 1.2

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// The character controller play made, or null.
    property Node physics: null

    // ── the camera that follows ──
    /// How far up the camera looks down on the player, in degrees, and how far behind it is.
    property real pitch: 18
    property real distance: 5.5
    readonly property vector3d feet: physics ? physics.scenePosition.minus(Qt.vector3d(0, halfHeight, 0)) : scenePosition
    readonly property real heading: physics ? physics.eulerRotation.y : eulerRotation.y
    readonly property vector3d eye: {
        const h = heading * Math.PI / 180
        const p = pitch * Math.PI / 180
        return feet.plus(Qt.vector3d(Math.sin(h) * distance * Math.cos(p), 1.5 + distance * Math.sin(p),
                                     Math.cos(h) * distance * Math.cos(p)))
    }
    readonly property vector3d lookAt: feet.plus(Qt.vector3d(0, 1.4, 0))

    readonly property real halfHeight: 0.9
    readonly property real runFactor: 1.7
    property real verticalSpeed: 0
    property bool jumpHeld: false
    property vector3d start

    function makeBody() {
        if (!live || physics)
            return
        start = position.plus(Qt.vector3d(0, halfHeight, 0))
        physics = character.createObject(parent, {
            position: start,
            eulerRotation: Qt.vector3d(0, eulerRotation.y, 0)
        })
        if (!parent.player)
            parent.player = player
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    Component {
        id: character
        CharacterController {
            collisionShapes: CapsuleShape { diameter: 0.6; height: 1.2 }
            gravity: Qt.vector3d(0, 0, 0)  // fall and jump are the player's own, below
            midAirControl: true
        }
    }

    FrameAnimation {
        running: player.live && player.physics !== null
        onTriggered: {
            const body = player.physics
            const dt = Math.min(frameTime, 0.05)
            const g = player.parent.gravity
            const pointer = Input.takePointer()

            body.eulerRotation.y -= Input.look.x * 150 * dt + pointer.x * 0.25
            player.pitch = Math.max(-10, Math.min(60, player.pitch + pointer.y * 0.2 - Input.look.y * 60 * dt))

            const grounded = (body.collisions & CharacterController.Down) !== 0
            if (grounded) {
                if (player.verticalSpeed < 0)
                    player.verticalSpeed = -2  // pressed to the ground, so a step down is followed, not fallen off
                if (Input.jump && !player.jumpHeld && g > 0)
                    player.verticalSpeed = Math.sqrt(2 * g * player.jumpHeight)
            } else {
                player.verticalSpeed -= g * dt
                if ((body.collisions & CharacterController.Up) !== 0 && player.verticalSpeed > 0)
                    player.verticalSpeed = 0
            }
            player.jumpHeld = Input.jump

            const v = player.speed * (Input.run ? player.runFactor : 1)
            body.movement = Qt.vector3d(Input.move.x * v, player.verticalSpeed, -Input.move.y * v)

            // Fallen out of the world: back to the start.
            if (body.scenePosition.y < -40) {
                player.verticalSpeed = 0
                body.teleport(player.start)
            }
        }
    }

    // The figure: a capsule with a lit visor on the side it faces (-z). It rides on the controller in play.
    Node {
        id: figure
        parent: player.physics ?? player
        position: Qt.vector3d(0, player.physics ? -player.halfHeight : 0, 0)

        PrincipledMaterial {
            id: skin
            baseColor: player.colour
            metalness: 0.15
            roughness: 0.35
        }
        Model {
            source: "#Cylinder"
            position: Qt.vector3d(0, 0.9, 0)
            scale: Qt.vector3d(0.006, 0.012, 0.006)
            materials: skin
            pickable: true
            castsShadows: true
        }
        Model {
            source: "#Sphere"
            position: Qt.vector3d(0, 0.3, 0)
            scale: Qt.vector3d(0.006, 0.006, 0.006)
            materials: skin
            pickable: true
            castsShadows: true
        }
        Model {
            source: "#Sphere"
            position: Qt.vector3d(0, 1.5, 0)
            scale: Qt.vector3d(0.006, 0.006, 0.006)
            materials: skin
            pickable: true
            castsShadows: true
        }
        Model {
            source: "#Cube"
            position: Qt.vector3d(0, 1.48, -0.27)
            scale: Qt.vector3d(0.0036, 0.0012, 0.0012)
            pickable: true
            materials: PrincipledMaterial {
                baseColor: "#0a1a24"
                emissiveFactor: Qt.vector3d(0.25, 1.6, 3.0)
                roughness: 0.2
            }
        }
    }
}
