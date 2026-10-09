// A ground plane that receives shadows: something for a world to stand on.

import QtQuick
import QtQuick3D

Model {
    id: ground
    property real extent: 40
    property color colour: "#151821"

    source: "#Rectangle"
    eulerRotation.x: -90
    scale: Qt.vector3d(extent, extent, 1)
    receivesShadows: true
    castsShadows: false
    materials: PrincipledMaterial {
        baseColor: ground.colour
        roughness: 0.82
        metalness: 0.0
    }
}
