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
    /// Whether play can tick (P3.11): its logic is built, or could not be, or there is none.
    readonly property bool ready: !live || file.object !== null || file.error !== "" || String(source) === ""

    readonly property QtObject targetEntity: live && target !== "" && parent ? _find(target) : null

    /// The entity called `name`: in a prefab, among its own parts first (P3.15), so each one placed drives its own; then
    /// the scene's own entities, then any prefab's part.
    function _find(name) {
        for (let p = parent; p; p = p.parent) {
            if (p.kind === "Prefab") {
                const own = SceneIO.everyEntity(p).find(e => e.name === name)
                if (own)
                    return own
                break
            }
        }
        const scene = SceneIO.sceneOf(behaviour)
        if (!scene)
            return null
        return SceneIO.entities(scene).find(e => e.name === name) ?? SceneIO.everyEntity(scene).find(e => e.name === name)
               ?? null
    }

    // Said where it can be read, in the Player as in the Studio: a game whose logic does not build plays without it.
    onErrorChanged: if (error !== "") console.warn("QQ logic: " + name + ": " + error)
    LogicFile {
        id: file
        active: behaviour.live
        source: SceneIO.resolvedUrl(behaviour, behaviour.source)
        properties: ({ target: behaviour.targetEntity, scene: behaviour.parent })
    }

    // Steps come from the scene's clock (Scene.tick, P3.11): the logic's clock, its timers and onStep. A paused world
    // has no steps, so its logic holds still (P3.8).
    function _stepLogic(dt) {
        if (file.object)
            file.object._step(dt)
    }
    // Presses and releases come from the tick too, the same in play and in a replay.
    // What the game's host told it comes from the tick too (ADR-117).
    function _told(message) {
        if (file.object)
            file.object._told(message)
    }

    function _hear(kind, action) {
        if (file.object)
            file.object._hear(kind, action)
    }

    // Frames, for what is only looked at: not in a paused world, and never more than 1/20 s at once.
    FrameAnimation {
        running: behaviour.live && file.object !== null && !behaviour.parent.paused && !behaviour.parent.replaying
        onTriggered: file.object._frame(Math.min(frameTime, 0.05))
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
