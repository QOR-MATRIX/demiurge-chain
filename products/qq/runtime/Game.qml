// A game: a world that plays one scene file from the moment it opens (DIRECTION P3.2). It is what the QQ Player shows,
// natively and in a browser: the scene's sky and light, its physics, its player under the keyboard, mouse and gamepad,
// its particles, sound and logic.
//
// With a player in the scene the eye follows it, and a drag looks around; without one the eye orbits the scene, a drag
// turns it and scrolling comes closer. A 2D scene is seen flat (P3.10; pixel art pixel for pixel, ADR-106), its eye on
// the player and stopped at the level's edges, or with no player on the scene from the ground up. `restart()` plays the
// scene again from its file.

import QtQuick
import QtQuick3D
import QQ

World {
    id: game

    /// The scene file to play: on this computer, or bundled in qrc:.
    property url file

    /// The scene playing, or null; and why there is none, if there is not.
    property QtObject scene: null
    property string problem: ""
    property var entityList: []

    /// True once the scene plays and every model in it has loaded or failed: the first frame of the game proper.
    readonly property bool settled: scene !== null && entityList.every(e => e.kind !== "Prop" || e.status !== Prop.Empty)
    /// The models that failed to load, each "name: why". The game plays without them, so the Player reports them
    /// rather than stopping; a game that settles with one missing is not the game its creator made.
    readonly property list<string> unloaded: entityList.filter(e => e.kind === "Prop" && e.status === Prop.Failed)
                                                       .map(e => e.name + ": " + e.errorString)

    readonly property QtObject follow: scene ? scene.player : null

    // The eye with no player: on a sphere around the middle of the scene.
    property real yaw: 25
    property real pitch: 16
    property real distance: 10
    property vector3d middle: Qt.vector3d(0, 1, 0)

    eye: follow ? follow.eye : Qt.vector3d(
        middle.x + distance * Math.cos(pitch * Math.PI / 180) * Math.sin(yaw * Math.PI / 180),
        middle.y + distance * Math.sin(pitch * Math.PI / 180),
        middle.z + distance * Math.cos(pitch * Math.PI / 180) * Math.cos(yaw * Math.PI / 180))
    // Flat, the eye on the player stops at the level's edges (Scene.keptInView); with no player, it looks from the
    // ground up.
    target: follow ? (flat ? scene.keptInView(follow.lookAt, width / (2 * magnification), height / (2 * magnification))
                           : follow.lookAt)
                   : flat ? Qt.vector3d(0, viewHeight / 2, 0) : middle

    flat: scene ? scene.view === "2d" : false
    pixelsPerUnit: scene ? scene.pixelsPerUnit : 16
    viewHeight: scene ? scene.viewHeight : 12
    pixelArt: scene ? scene.pixelArt : true

    hearing: true
    exposure: scene ? scene.exposure : 1
    bloom: scene ? scene.bloom : true
    skyLight: scene ? scene.skyLight : 0.75
    skyTop: scene ? scene.skyTop : "#0b1430"
    skyHorizon: scene ? scene.skyHorizon : "#c7623a"
    groundHorizon: scene ? scene.groundHorizon : "#2a1a1a"
    groundBottom: scene ? scene.groundBottom : "#07080c"

    function restart() {
        if (scene)
            SceneIO.discard(scene)
        scene = null
        entityList = []
        Input.release()
        const next = SceneIO.load(file, holder)
        if (!next) {
            problem = SceneIO.lastError
            return false
        }
        next.playing = true
        problem = ""
        scene = next
        entityList = SceneIO.entities(next)
        return true
    }

    /// Put the scene away in a safe order (its bodies before their world): what the player does as it quits.
    function release() {
        if (scene)
            SceneIO.discard(scene)
        scene = null
        entityList = []
    }

    property bool started: false
    Component.onCompleted: {
        started = true
        if (String(file) !== "")
            restart()
    }
    onFileChanged: if (started) restart()

    Node { id: holder }

    DragHandler {
        target: null
        enabled: !game.flat || game.follow !== null
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        property point last
        property real startYaw
        property real startPitch
        onActiveChanged: {
            last = Qt.point(0, 0)
            startYaw = game.yaw
            startPitch = game.pitch
        }
        onTranslationChanged: {
            if (game.follow) {
                Input.pointerMoved(Qt.point(translation.x - last.x, translation.y - last.y))
                last = Qt.point(translation.x, translation.y)
                return
            }
            game.yaw = startYaw - translation.x * 0.3
            game.pitch = Math.max(-5, Math.min(85, startPitch + translation.y * 0.3))
        }
    }

    WheelHandler {
        enabled: !game.follow && !game.flat
        onWheel: (event) => game.distance = Math.max(1.5, Math.min(80, game.distance * Math.pow(0.9, event.angleDelta.y / 120)))
    }
}
