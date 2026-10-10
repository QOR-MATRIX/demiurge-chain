// What a game's logic file is: a Logic, with handlers for what happens while the world plays. A Behaviour in the scene
// names the file and the entity it drives; the file is built when play starts and again each time it is saved.
//
//   // logic/spin.qml
//   import QQ
//
//   Logic {
//       property real speed: 90          // degrees a second
//       onFrame: (dt) => target.eulerRotation.y += speed * dt
//   }
//
// `target` is the entity the Behaviour names, `scene` the scene playing, `elapsed` the seconds since this logic was
// built. `Input` (move, look, jump, run) is there for any logic that reads the player's hands.

import QtQml
import QQ

QtObject {
    id: logic

    /// The entity this logic drives, and the scene it plays in.
    property QtObject target: null
    property QtObject scene: null

    /// Seconds since this logic was built.
    property real elapsed: 0

    /// Once a frame while the world plays, with the seconds since the last frame.
    signal frame(real dt)

    /// The entity in this scene called `name`, or null.
    function entity(name) {
        return scene ? SceneIO.entities(scene).find(e => e.name === name) ?? null : null
    }

    /// Marks a Logic: a file whose root is anything else is refused.
    readonly property bool qqLogic: true

    default property list<QtObject> data
}
