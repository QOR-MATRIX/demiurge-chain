// A spot light (DIRECTION P3.14): a cone of light from a point, a stage light, a torch's beam, a lamp over a table.
// It shines along its own forward direction, down unless it is turned (`eulerRotation`), within `cone` degrees from edge
// to edge, softening toward the edge over `softness` of that, and falls off with distance as a Lamp does. It casts
// shadows. Logic turns it as it turns anything (a tween on `eulerRotation` sweeps it).

import QtQuick
import QtQuick3D

SpotLight {
    id: spot

    readonly property string kind: "Spotlight"
    readonly property var fields: ["name", "position", "eulerRotation", "color", "brightness", "reach", "cone",
                                   "softness"]
    readonly property var ranges: ({ brightness: [0, 50], reach: [0.5, 100], cone: [1, 170], softness: [0, 1] })

    property string name: "Spotlight"
    /// Roughly how far, in units, it lights.
    property real reach: 10
    /// The cone's width, in degrees from edge to edge.
    property real cone: 40
    /// How much of the cone fades toward its edge: 0 a hard edge, 1 soft from the middle out.
    property real softness: 0.3

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true

    eulerRotation: Qt.vector3d(-90, 0, 0)
    color: "#fff1dc"
    brightness: 8
    coneAngle: cone
    innerConeAngle: cone * (1 - softness)
    castsShadow: true
    shadowMapQuality: Light.ShadowMapQualityHigh
    shadowFactor: 75
    softShadowQuality: Light.PCF8
    constantFade: 1.0
    linearFade: 0.0
    quadraticFade: 4.0 / (reach * reach)

    // What shows where it is and which way it points while editing (a click lands on it): hidden in play.
    Model {
        source: "#Cone"
        // Qt's cone stands on its base with its tip up its y axis, a hundred units tall; the light shines down its -z.
        // Turned so its tip points back up +z, at the light, and its base faces where the light goes.
        eulerRotation: Qt.vector3d(90, 0, 0)
        position: Qt.vector3d(0, 0, -0.3)
        scale: Qt.vector3d(0.002, 0.003, 0.002)
        pickable: true
        visible: !spot.live
        castsShadows: false
        receivesShadows: false
        materials: PrincipledMaterial {
            baseColor: "#000000"
            emissiveFactor: Qt.vector3d(spot.color.r * 1.6, spot.color.g * 1.6, spot.color.b * 1.6)
        }
    }
}
