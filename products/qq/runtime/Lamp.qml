// A point of light: a lamp, a torch, a glowing crystal. Light falls off with distance from it.

import QtQuick
import QtQuick3D

PointLight {
    readonly property string kind: "Lamp"
    readonly property var fields: ["name", "position", "color", "brightness", "reach"]
    readonly property var ranges: ({ brightness: [0, 20], reach: [0.5, 50] })

    property string name: "Lamp"
    /// Roughly how far, in units, the lamp lights.
    property real reach: 6

    color: "#ffd9a8"
    brightness: 4
    castsShadow: true
    shadowMapQuality: Light.ShadowMapQualityHigh
    constantFade: 1.0
    linearFade: 0.0
    quadraticFade: 4.0 / (reach * reach)
}
