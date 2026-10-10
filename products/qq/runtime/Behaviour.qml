// Game logic in the scene: a Logic file from the project (`source`, see Logic.qml), driving the entity named `target`
// (or nothing, for logic that looks after the whole scene). It runs only while the world plays, and a save of the file
// while it plays is picked up at once, without stopping.

import QtQuick
import QtQuick3D
import QQ

Node {
    id: behaviour

    readonly property string kind: "Behaviour"
    readonly property var fields: ["name", "source", "target"]
    readonly property var fileTypes: ({ source: ["QQ logic (*.qml)"] })

    property string name: "Behaviour"
    /// The logic file, `logic/<name>.qml` in the project; kept relative to the scene file.
    property url source
    /// The name of the entity the logic drives.
    property string target: ""

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true

    readonly property QtObject logic: file.object
    readonly property string error: file.error
    readonly property int builds: file.builds

    readonly property QtObject targetEntity: live && target !== "" && parent
                                             ? SceneIO.entities(parent).find(e => e.name === behaviour.target) ?? null
                                             : null

    LogicFile {
        id: file
        active: behaviour.live
        source: SceneIO.resolvedUrl(behaviour, behaviour.source)
        properties: ({ target: behaviour.targetEntity, scene: behaviour.parent })
    }

    FrameAnimation {
        running: behaviour.live && file.object !== null
        onTriggered: {
            // A long frame (a window dragged, a breakpoint) is not a long step: logic never jumps more than 1/20 s.
            const dt = Math.min(frameTime, 0.05)
            file.object.elapsed += dt
            file.object.frame(dt)
        }
    }
}
