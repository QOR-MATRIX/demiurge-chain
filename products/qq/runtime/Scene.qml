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

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.Physics

Node {
    id: root

    readonly property string kind: "Scene"
    readonly property var fields: ["name", "skyTop", "skyHorizon", "groundHorizon", "groundBottom", "skyLight",
                                   "exposure", "bloom", "gravity", "screens"]
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
    /// The screens shown when play starts, by name, separated by commas: "hud", or "title" for a title screen.
    property string screens: "hud"

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

    /// Whether the world is paused: physics, logic and the player hold still, and screens still answer. Not a field.
    property bool paused: false

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
            running: !root.paused
            gravity: Qt.vector3d(0, -root.gravity, 0)
            typicalLength: 1
            typicalSpeed: 10
            defaultDensity: 1000
        }
    }
}
