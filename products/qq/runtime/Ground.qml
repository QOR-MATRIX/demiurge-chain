// A ground plane that receives shadows: something for a world to stand on. In play it is solid, a slab as wide as it
// is drawn, so whatever walks or rolls off its edge falls.

import QtQuick
import QtQuick3D
import QtQuick3D.Physics

Model {
    id: ground

    readonly property string kind: "Ground"
    readonly property var fields: ["name", "extent", "colour"]
    readonly property var ranges: ({ extent: [1, 500] })

    property string name: "Ground"
    property real extent: 40
    property color colour: "#151821"

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    property Node physics: null

    function makeBody() {
        if (live && !physics)
            physics = slab.createObject(parent)
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    source: "#Rectangle"
    eulerRotation.x: -90
    scale: Qt.vector3d(extent, extent, 1)
    pickable: true
    receivesShadows: true
    castsShadows: false
    materials: PrincipledMaterial {
        baseColor: ground.colour
        roughness: 0.82
        metalness: 0.0
    }

    Component {
        id: slab
        StaticRigidBody {
            // Its top face is the drawn plane.
            position: Qt.vector3d(0, -0.5, 0)
            physicsMaterial: PhysicsMaterial { staticFriction: 0.8; dynamicFriction: 0.7; restitution: 0.1 }
            collisionShapes: BoxShape { extents: Qt.vector3d(ground.extent, 1, ground.extent) }
        }
    }
}
