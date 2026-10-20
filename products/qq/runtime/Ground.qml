// A ground plane that receives shadows: something for a world to stand on, `extent` units on a side. In play it is
// solid, a slab as wide as it is drawn, so whatever walks or rolls off its edge falls.
//
// Its surface can be a picture (P3.14): `image`, one copy every `imageSize` units, tinted by `colour` (white shows it
// as painted).

import QtQuick
import QtQuick3D
import QtQuick3D.Physics
import QQ

Model {
    id: ground

    readonly property string kind: "Ground"
    readonly property var fields: ["name", "extent", "colour", "image", "imageSize"]
    readonly property var ranges: ({ extent: [1, 500], imageSize: [0.05, 100] })
    readonly property var fileTypes: ({ image: ["Images (*.png *.jpg *.jpeg *.webp)"] })

    property string name: "Ground"
    property real extent: 40
    property color colour: "#151821"
    /// A picture on it, kept relative to the scene file, and the units one copy of it covers.
    property url image
    property real imageSize: 1

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    property Node physics: null

    function makeBody() {
        if (live && !physics)
            physics = slab.createObject(parent)
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    // Qt's built-in rectangle is a hundred units across, and a QQ unit is one.
    source: "#Rectangle"
    eulerRotation.x: -90
    scale: Qt.vector3d(extent * 0.01, extent * 0.01, 1)
    pickable: true
    receivesShadows: true
    castsShadows: false
    materials: PrincipledMaterial {
        baseColor: ground.colour
        roughness: 0.82
        metalness: 0.0
        baseColorMap: String(ground.image) !== "" ? picture : null
    }

    readonly property Texture picture: Texture {
        source: String(ground.image) !== "" ? SceneIO.resolvedUrl(ground, ground.image) : ""
        scaleU: ground.extent / Math.max(0.05, ground.imageSize)
        scaleV: ground.extent / Math.max(0.05, ground.imageSize)
        tilingModeHorizontal: Texture.Repeat
        tilingModeVertical: Texture.Repeat
        generateMipmaps: true
        mipFilter: Texture.Linear
    }

    Component {
        id: slab
        StaticRigidBody {
            // Named for contacts and rays (P3.7), and heard by what lands on it.
            readonly property QtObject entity: ground
            sendContactReports: true
            // Its top face is the drawn plane.
            position: Qt.vector3d(0, -0.5, 0)
            physicsMaterial: PhysicsMaterial { staticFriction: 0.8; dynamicFriction: 0.7; restitution: 0.1 }
            collisionShapes: BoxShape { extents: Qt.vector3d(ground.extent, 1, ground.extent) }
        }
    }
}
