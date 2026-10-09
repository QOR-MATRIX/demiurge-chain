// A QQ world: the viewport, its sky, its light and its look. A scene's things are its children.
//
// The look is QQ's default, chosen to make a scene read well before anyone tunes it: a procedural sky that also lights
// the scene (image-based lighting, so materials have something to reflect), filmic tonemapping, bloom from anything
// bright, ambient occlusion in the creases, and multisampled edges. Each can be changed per world.

import QtQuick
import QtQuick3D
import QtQuick3D.Helpers

View3D {
    id: world

    /// Where the eye is, and what it looks at.
    property vector3d eye: Qt.vector3d(0, 1.6, 6.5)
    property vector3d target: Qt.vector3d(0, 0.4, 0)

    /// Light falling on the scene from the sky, and the exposure it is seen at.
    property real skyLight: 0.75
    property real exposure: 1.0

    /// Bloom: bright light spills past its edges.
    property bool bloom: true

    /// The sky's colours.
    property color skyTop: "#0b1430"
    property color skyHorizon: "#c7623a"
    property color groundHorizon: "#2a1a1a"
    property color groundBottom: "#07080c"

    default property alias content: scene.data

    camera: eyeCamera

    environment: ExtendedSceneEnvironment {
        backgroundMode: SceneEnvironment.SkyBox
        lightProbe: Texture {
            textureData: ProceduralSkyTextureData {
                skyTopColor: world.skyTop
                skyHorizonColor: world.skyHorizon
                groundHorizonColor: world.groundHorizon
                groundBottomColor: world.groundBottom
                sunLatitude: 18
                sunLongitude: 210
            }
        }
        probeExposure: world.skyLight
        skyboxBlurAmount: 0.12

        tonemapMode: SceneEnvironment.TonemapModeFilmic
        exposure: world.exposure

        glowEnabled: world.bloom
        glowStrength: 1.0
        glowIntensity: 0.6
        glowBloom: 0.35
        glowHDRMinimumValue: 1.1
        glowBlendMode: ExtendedSceneEnvironment.GlowBlendMode.Additive

        aoEnabled: true
        aoStrength: 45
        aoDistance: 6

        antialiasingMode: SceneEnvironment.MSAA
        antialiasingQuality: SceneEnvironment.High
        specularAAEnabled: true
        ditheringEnabled: true
    }

    PerspectiveCamera {
        id: eyeCamera
        clipNear: 0.05
        clipFar: 2000
    }

    // Position, then aim: in one place, so the camera never looks from where it was toward where it is going.
    function placeEye() {
        eyeCamera.position = world.eye
        eyeCamera.lookAt(world.target)
    }
    onEyeChanged: placeEye()
    onTargetChanged: placeEye()
    Component.onCompleted: placeEye()

    Node {
        id: scene
    }
}
