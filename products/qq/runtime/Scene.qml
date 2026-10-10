// A QQ scene: the root of everything a creator places, and the look of the world it is shown in. It is what a scene
// file holds (`scenes/<name>.qml`, ADR-083 decision 3): the Studio writes it with SceneIO in one canonical layout, one
// property per line, so a change is a small true diff in Projects.
//
// Every entity in a scene declares `kind` (its type's name, as written in the file) and `fields` (the properties a
// file keeps, in the order it keeps them). The writer needs nothing else to know about a type.
//
// A scene plays when `playing` is set (DIRECTION P3.2): a physics world starts, in metres and seconds, and each entity
// becomes what it is in play (a body, a character, a sound). The Studio plays a copy built from the scene's text, so the
// scene being edited is never touched by play.

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.Physics

Node {
    id: root

    readonly property string kind: "Scene"
    readonly property var fields: ["name", "skyTop", "skyHorizon", "groundHorizon", "groundBottom", "skyLight",
                                   "exposure", "bloom", "gravity"]
    readonly property var ranges: ({ skyLight: [0, 3], exposure: [0.2, 3], gravity: [0, 30] })

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

    /// Whether the scene plays: not a field, so never written to a file.
    property bool playing: false

    /// The player, once play has made one: what a camera follows.
    property QtObject player: null

    /// The physics world while playing, or null.
    readonly property QtObject physicsWorld: physics.object

    // One physics world, made when play starts and gone when it stops. QQ's unit is the metre.
    Instantiator {
        id: physics
        active: root.playing
        delegate: PhysicsWorld {
            scene: root
            running: true
            gravity: Qt.vector3d(0, -root.gravity, 0)
            typicalLength: 1
            typicalSpeed: 10
            defaultDensity: 1000
        }
    }
}
