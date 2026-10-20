// A model brought into a world from a glTF 2.0 file at runtime: no import step, no conversion, so a creator's own asset
// appears as soon as it is chosen. A file that cannot be read is an error the prop reports, never a crash.
//
// Its animations (P3.12, ADR-105): a glTF file's animations, skeletal (joints turned and moved) and morph (target
// weights) alike, are its clips, by name. It plays `clip` at `clipSpeed`, looping or holding its last pose, on the game's
// clock: a step at a time, held while the world is paused, and drawn between steps so it moves smoothly at any frame
// rate. Logic changes clips by setting `clip`; `clipTime` says how far in it is, and `clipDone` when one that does not
// loop has reached its end; part(name) is a joint or a mesh of it. While editing, it shows the clip's first pose.

import QtQuick
import QtQuick3D
import QtQuick3D.AssetUtils
import QQ

Node {
    id: prop

    readonly property string kind: "Prop"
    readonly property var fields: ["name", "position", "eulerRotation", "scale", "source", "clip", "clipSpeed",
                                   "clipLoops"]
    readonly property var ranges: ({ clipSpeed: [-4, 4] })
    readonly property var choices: ({ clip: [""].concat(clips) })
    readonly property var fileTypes: ({ source: ["glTF 2.0 (*.gltf *.glb)"] })

    property string name: "Model"

    /// The glTF 2.0 file (.gltf or .glb), as a URL. A scene file keeps it relative to itself; declared here rather than
    /// aliased to the loader so QML resolves it against the scene file, and it is always a whole URL once read.
    property url source

    /// The animation it plays, by name (one of `clips`); empty for none.
    property string clip: ""
    /// How fast it plays: 1 as made, 2 twice as fast, -1 backwards.
    property real clipSpeed: 1
    /// Whether it starts again at its end, or holds its last pose there.
    property bool clipLoops: true

    /// Its animations' names, how far into the clip it is (seconds), and the clip's length.
    readonly property var clips: animations.names
    property real clipTime: 0
    readonly property real clipLength: animations.names.length >= 0 ? animations.lengthOf(clip) : 0
    /// Whether a clip that does not loop has reached its end (its start, played backwards).
    readonly property bool clipDone: !clipLoops && clipLength > 0
                                     && (clipSpeed >= 0 ? clipTime >= clipLength : clipTime <= 0)

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true

    /// Empty, Ready or Failed, as the loader reports it.
    readonly property int status: loader.status === RuntimeLoader.Success ? Prop.Ready
                                : loader.status === RuntimeLoader.Error ? Prop.Failed
                                : Prop.Empty
    readonly property string errorString: loader.errorString
    readonly property var bounds: loader.bounds
    /// Whether play can tick (P3.11): its file has loaded, or failed, or there is none; so a clip's steps never depend on
    /// how quickly a file loads.
    readonly property bool ready: String(source) === "" || status !== Prop.Empty

    enum Status { Empty, Ready, Failed }

    RuntimeLoader {
        id: loader
        // Whole, against the scene file this entity came from: a relative path given to the loader as written would be
        // taken against QQ's own module, and a confined game's loads are checked whole (ADR-087).
        source: SceneIO.resolvedUrl(prop, prop.source)
        onStatusChanged: {
            if (status === RuntimeLoader.Success)
                animations.scan()
            prop._pose(0)
        }
    }

    Clips {
        id: animations
        model: loader
    }

    onClipChanged: {
        clipTime = clipSpeed >= 0 ? 0 : clipLength
        _pose(0)
    }

    /// A part of the model by its name in the file (a joint, a mesh), or null: to read where a hand is, or to hide a part.
    function part(name) {
        return loader.query(name)
    }

    /// One step of the game (Scene.tick): the clip moves on, wrapping or holding at its ends, and the model is posed
    /// exactly there, so logic that reads a joint in its step reads the same in play and in a replay.
    function _stepAnimate(dt) {
        if (clip === "" || clipLength <= 0)
            return
        let t = clipTime + dt * clipSpeed
        if (clipLoops)
            t = ((t % clipLength) + clipLength) % clipLength
        else
            t = Math.max(0, Math.min(clipLength, t))
        clipTime = t
        _pose(0)
    }

    /// The pose `ahead` seconds of play past the last step: between steps, so it moves smoothly.
    function _pose(ahead) {
        if (clip === "" || clipLength <= 0) {
            animations.rest()
            return
        }
        let t = clipTime + ahead * clipSpeed
        t = clipLoops ? ((t % clipLength) + clipLength) % clipLength : Math.max(0, Math.min(clipLength, t))
        animations.pose(clip, t)
    }

    FrameAnimation {
        running: prop.live && prop.clip !== "" && !prop.parent.replaying
        onTriggered: prop._pose(prop.parent.paused ? 0 : prop.parent._owed)
    }

    // What a click lands on: an invisible box over the model's bounds. The meshes the loader makes cannot be marked
    // pickable from here, and a box is what selecting a whole model needs anyway.
    Model {
        visible: prop.status === Prop.Ready
        source: "#Cube"
        pickable: true
        castsShadows: false
        receivesShadows: false
        position: loader.bounds.minimum.plus(loader.bounds.maximum).times(0.5)
        scale: loader.bounds.maximum.minus(loader.bounds.minimum).times(0.01)
        materials: PrincipledMaterial {
            alphaMode: PrincipledMaterial.Blend
            opacity: 0.0
            depthDrawMode: Material.NeverDepthDraw
        }
    }
}
