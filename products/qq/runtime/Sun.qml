// The key light: a directional sun that casts soft shadows. Its direction is set by two angles, so a creator (or the
// agent) can say "late afternoon, from the left" without thinking in vectors.

import QtQuick
import QtQuick3D

DirectionalLight {
    readonly property string kind: "Sun"
    readonly property var fields: ["name", "elevation", "azimuth", "color", "brightness"]
    readonly property var ranges: ({ elevation: [0, 90], azimuth: [-180, 180], brightness: [0, 5] })

    property string name: "Sun"
    /// Degrees above the horizon.
    property real elevation: 42
    /// Degrees around the vertical axis.
    property real azimuth: -35

    eulerRotation: Qt.vector3d(-elevation, azimuth, 0)
    color: "#fff1dc"
    brightness: 1.15
    castsShadow: true
    shadowMapQuality: Light.ShadowMapQualityVeryHigh
    shadowFactor: 65
    shadowBias: 10
    softShadowQuality: Light.PCF16
    pcfFactor: 2
}
