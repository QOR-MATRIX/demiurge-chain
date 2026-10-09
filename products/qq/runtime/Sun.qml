// The key light: a directional sun that casts soft shadows. Its direction is set by two angles, so a creator (or the
// agent) can say "late afternoon, from the left" without thinking in vectors.

import QtQuick
import QtQuick3D

DirectionalLight {
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
