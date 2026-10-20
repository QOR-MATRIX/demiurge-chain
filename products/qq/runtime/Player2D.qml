// The player of a 2D game (DIRECTION P3.10, ADR-105): a figure on the plane z = 0 under the player's hands (Input), as
// a platformer's hero or a top-down one, `width` by `height` units, stopped by tile maps and static bodies:
//
//   platform   move left and right, jump (the jump action), fall under the scene's gravity, run with the run action
//   top-down   move freely across the plane, run with the run action; no gravity
//
// It moves by QQ's own kinematics (P3.11, kinematics.h), once a step of the scene's clock: the same hands give the same
// places on every machine and in a replay with no physics world, so where it goes can count toward a rewarded outcome.
// A kinematic body follows it in play, so PhysX's bodies are pushed aside by it and feel it; triggers hear it by QQ's own
// measure. Between steps it is drawn where it would be, so it moves smoothly at any frame rate.
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
    /// The kinematic body play made, which PhysX's bodies meet, or null.
    property Node physics: null

    /// idle, run, jump or fall: what it is doing, for its look's frames.
    property string activity: "idle"
    /// Whether it faces left.
    property bool facingLeft: false
    /// Whether it stood on something at the last step.
    property bool grounded: false

    readonly property real runFactor: 1.6
    /// Falls faster than this, in units a second, are held to it, so no step carries it through a tile.
    readonly property real fastestFall: 30

    /// Where its middle is in play, in doubles: what QQ's kinematics move, and what a replay reproduces exactly.
    property real atX: position.x
    property real atY: position.y + height / 2
    property real atZ: position.z
    property real verticalSpeed: 0
    property bool jumpHeld: false
    /// Its middle where respawn() puts it back: where it began, unless logic has moved it since (a checkpoint).
    property vector3d start

    /// Where its feet are, and what the eye centres on: the step's place, and the place drawn between steps.
    readonly property vector3d feet: Qt.vector3d(atX, atY - height / 2, atZ)
    property real _shownX: atX
    property real _shownY: atY
    property real _lastX: atX
    property real _lastY: atY
    readonly property vector3d lookAt: Qt.vector3d(_shownX, _shownY, atZ)
    readonly property vector3d eye: lookAt

    readonly property QtObject sprite: live && look !== "" && parent
                                       ? SceneIO.entities(parent).find(e => e.name === player.look) ?? null : null

    /// Put its middle at `centre`, still.
    function place(centre) {
        atX = centre.x
        atY = centre.y
        atZ = centre.z
        _lastX = atX
        _lastY = atY
        verticalSpeed = 0
        grounded = false
        _show(1)
    }

    /// Back to `start`, still.
    function respawn() {
        place(start)
    }

    function makeBody() {
        if (!live || physics)
            return
        start = Qt.vector3d(position.x, position.y + height / 2, position.z)
        place(start)
        physics = carrier.createObject(parent, { position: start })
        if (!parent.player)
            parent.player = player
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    Component {
        id: carrier
        DynamicRigidBody {
            readonly property QtObject entity: player
            isKinematic: true
            kinematicPosition: Qt.vector3d(player.atX, player.atY, player.atZ)
            sendContactReports: true
            receiveContactReports: true
            // Triggers hear it by QQ's own measure (Scene.tick), not PhysX, so a replay hears it too.
            sendTriggerReports: false
            collisionShapes: BoxShape { extents: Qt.vector3d(player.width, player.height, 1) }
        }
    }

    /// One step of the game (Scene.tick): the hands read, the box moved, what it is doing worked out.
    function _stepMove(dt) {
        if (!live || !physics)
            return
        _lastX = atX
        _lastY = atY
        const v = speed * (Input.run ? runFactor : 1)
        const move = Input.move
        const across = move.x * v
        let up = 0
        if (style === "top-down") {
            up = move.y * v
            verticalSpeed = 0
        } else {
            const g = parent.gravity
            if (grounded && Input.jump && !jumpHeld && g > 0)
                verticalSpeed = Math.sqrt(2 * g * jumpHeight)
            else
                verticalSpeed = Math.max(-fastestFall, verticalSpeed - g * dt)
            up = verticalSpeed
        }
        jumpHeld = Input.jump

        const moved = SceneIO.move2D(parent, player, atX, atY, atZ, width / 2, height / 2, across * dt, up * dt)
        atX = moved.x
        atY = moved.y
        if (style === "top-down") {
            activity = Math.abs(across) + Math.abs(up) > 0.01 ? "run" : "idle"
        } else {
            if (moved.up && verticalSpeed > 0)
                verticalSpeed = 0
            grounded = moved.down && verticalSpeed <= 0
            if (moved.down && verticalSpeed < 0)
                verticalSpeed = 0
            activity = !grounded ? (verticalSpeed > 0 ? "jump" : "fall") : Math.abs(across) > 0.01 ? "run" : "idle"
        }
        if (across < -0.01)
            facingLeft = true
        else if (across > 0.01)
            facingLeft = false
        // Out of the world: back to the start.
        if (atY < -60)
            respawn()
        if (parent.replaying)
            _show(1)
    }

    /// Drawn `part` of the way from the last step's place to this one's: its look, and the eye.
    function _show(part) {
        _shownX = _lastX + (atX - _lastX) * part
        _shownY = _lastY + (atY - _lastY) * part
        const s = sprite
        if (s) {
            s.position = Qt.vector3d(_shownX, _shownY - height / 2 + s.size.y * s.scale.y / 2, s.position.z)
            s.flip = facingLeft
        }
    }

    // Each drawn frame: as far between the last two steps as the scene's clock has got; where it stands, when paused.
    FrameAnimation {
        running: player.live && player.physics !== null && !player.parent.replaying
        onTriggered: player._show(player.parent.paused ? 1 : Math.min(1, player.parent._owed * player.parent.stepRate))
    }

    // A plain figure, when it carries no Sprite: a rounded block in its colour with a lit band where it faces.
    Node {
        visible: player.look === ""
        position: player.live ? Qt.vector3d(player._shownX, player._shownY, player.atZ).minus(player.position)
                              : Qt.vector3d(0, player.height / 2, 0)
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
