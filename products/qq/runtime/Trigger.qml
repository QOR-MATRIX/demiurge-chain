// A trigger: a volume that reports what enters it and what leaves (DIRECTION P3.7, ADR-105). A pickup, a goal line, a
// door's sensor, a checkpoint. Nothing collides with it: things pass through, and it says so. A box or a sphere, one
// unit across before its scale, like a Shape.
//
// While editing it is drawn as a faint volume, so it can be seen and picked; in play it is not drawn. Logic hears it
// through a Behaviour that targets it (`onEntered`, `onExited`), or through its own signals.

import QtQuick
import QtQuick3D
import QtQuick3D.Physics

Node {
    id: trigger

    readonly property string kind: "Trigger"
    readonly property var fields: ["name", "position", "eulerRotation", "scale", "form"]
    readonly property var choices: ({ form: ["box", "sphere"] })

    property string name: "Trigger"
    /// box or sphere.
    property string form: "box"

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// The volume play made, or null.
    property Node physics: null

    /// An entity came in, or left (null for something that is not an entity).
    signal entered(QtObject other)
    signal exited(QtObject other)

    /// How many bodies are inside now.
    readonly property int inside: physics ? physics.collisionCount : 0

    function makeBody() {
        if (!live || physics)
            return
        const made = volume.createObject(parent, { position: trigger.position, rotation: trigger.rotation })
        made.collisionShapes = [(form === "sphere" ? ball : box).createObject(made)]
        physics = made
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    Component {
        id: volume
        TriggerBody {
            readonly property QtObject entity: trigger
            onBodyEntered: (body) => trigger.entered(body && body.entity !== undefined ? body.entity : null)
            onBodyExited: (body) => trigger.exited(body && body.entity !== undefined ? body.entity : null)
        }
    }
    Component {
        id: box
        BoxShape { extents: trigger.scale }
    }
    Component {
        id: ball
        SphereShape { diameter: Math.max(trigger.scale.x, trigger.scale.y, trigger.scale.z) }
    }

    // The faint volume the editor shows. Qt's built-in meshes are a hundred units across, and a QQ unit is one.
    Model {
        visible: !trigger.live
        pickable: !trigger.live
        source: trigger.form === "sphere" ? "#Sphere" : "#Cube"
        scale: Qt.vector3d(0.01, 0.01, 0.01)
        castsShadows: false
        receivesShadows: false
        materials: PrincipledMaterial {
            baseColor: "#5ad1ff"
            emissiveFactor: Qt.vector3d(0.05, 0.25, 0.35)
            alphaMode: PrincipledMaterial.Blend
            opacity: 0.18
            depthDrawMode: Material.NeverDepthDraw
            cullMode: Material.NoCulling
            lighting: PrincipledMaterial.NoLighting
        }
    }
}
