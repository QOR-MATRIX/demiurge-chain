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
        // A paused world's logic holds still too: its clock, its timers and its frames (P3.8).
        running: behaviour.live && file.object !== null && !behaviour.parent.paused
        onTriggered: {
            // A long frame (a window dragged, a breakpoint) is not a long step: logic never jumps more than 1/20 s.
            // The step moves the clock, runs the timers that are due, then `frame`.
            file.object._step(Math.min(frameTime, 0.05))
        }
    }

    // What happens to the target reaches its logic (P3.7, P3.8): a contact on its body, what enters or leaves it when it
    // is a Trigger, and a press when it is a ScreenButton. A kind without one of these signals simply never sends it.
    Connections {
        target: behaviour.live ? behaviour.targetEntity : null
        ignoreUnknownSignals: true
        function onContact(other, point, normal) {
            if (file.object)
                file.object.contact(other, point, normal)
        }
        function onEntered(other) {
            if (file.object)
                file.object.entered(other)
        }
        function onExited(other) {
            if (file.object)
                file.object.exited(other)
        }
        function onClicked() {
            if (file.object)
                file.object.clicked()
        }
    }
}
