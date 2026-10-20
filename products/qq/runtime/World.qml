// A QQ world: the viewport, its sky, its light and its look. A scene's things are its children.
//
// The look is QQ's default, chosen to make a scene read well before anyone tunes it: a procedural sky that also lights
// the scene (image-based lighting, so materials have something to reflect), filmic tonemapping, bloom from anything
// bright, ambient occlusion in the creases, and multisampled edges. Each can be changed per world.
//
// Flat (a 2D scene, DIRECTION P3.10, ADR-105 and ADR-106): the same world through an orthographic camera looking down
// the depth axis at `target`, over a backdrop of the sky's colours, with no ambient occlusion. Pixel art is scaled by
// whole numbers of device pixels to an art pixel, as many as fit `viewHeight` units in the height (`pixelsPerUnit`
// pixels to a unit), and drawn in exactly the colours it was painted in: linear tonemapping, no bloom, no dithering.
// Art that is not pixel art fills exactly `viewHeight` units, with the 3D look's tonemapping and bloom.

import QtQuick
import QtQuick3D
import QtQuick3D.Helpers
import QtQuick3D.SpatialAudio

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
    /// Or a sky image (P3.14): an equirectangular HDR file, a whole URL, seen behind everything and lighting it, turned
    /// `skyTurn` degrees about the vertical. Flat, the backdrop stays the colours.
    property url sky
    property real skyTurn: 0
    readonly property bool skyImage: String(sky) !== ""

    default property alias content: scene.data

    /// Whether the world is heard: its sounds through the camera's ears, in metres. On while a scene plays; off while
    /// it is edited, and in a world that has nothing to hear, so no sound device is opened for nothing.
    property bool hearing: false

    /// The camera the world is seen through, for an overlay that must see it the same way.
    readonly property alias viewCamera: eyeCamera

    /// Seen flat, through an orthographic camera (a 2D scene), or in perspective.
    property bool flat: false
    /// Flat: pixels of art to a unit, and how many units tall the view is at the least.
    property real pixelsPerUnit: 16
    property real viewHeight: 12
    /// Flat: the art is pixel art (scaled by whole numbers, in its painted colours), or not.
    property bool pixelArt: true
    /// Flat: shows exactly `viewHeight` units at any scale, pixel art or not (the Studio while editing, zooming freely).
    property bool freeZoom: false
    /// Whole-number scaling, and art in exactly its painted colours.
    readonly property bool wholePixels: flat && pixelArt && !freeZoom
    readonly property bool asPainted: flat && pixelArt

    /// The device pixels to a logical one, and the whole number of device pixels each art pixel is drawn with.
    readonly property real dpr: Screen.devicePixelRatio > 0 ? Screen.devicePixelRatio : 1
    readonly property int zoom: Math.max(1, Math.floor(height * dpr / (viewHeight * pixelsPerUnit)))
    /// Logical pixels to a unit, flat (never 0, which a camera refuses, even before the view has a size).
    readonly property real magnification: Math.max(0.001, wholePixels ? pixelsPerUnit * zoom / dpr
                                                                      : height / Math.max(0.01, viewHeight))

    /// The camera in use.
    readonly property Camera activeCamera: flat ? flatCamera : eyeCamera

    camera: activeCamera

    environment: ExtendedSceneEnvironment {
        // Flat, the camera's rays are parallel and a sky box would be one colour: a 2D world draws its backdrop below.
        backgroundMode: world.flat ? SceneEnvironment.Transparent : SceneEnvironment.SkyBox
        lightProbe: world.skyImage ? photographed : made
        probeOrientation: Qt.vector3d(0, world.skyTurn, 0)
        probeExposure: world.skyLight
        // A made sky is soft; a sky image is seen as it is.
        skyboxBlurAmount: world.skyImage ? 0 : 0.12

        // Pixel art is shown in the colours it was painted in.
        tonemapMode: world.asPainted ? SceneEnvironment.TonemapModeLinear : SceneEnvironment.TonemapModeFilmic
        exposure: world.exposure

        // Nor with bloom, which would brighten every colour, not only the brightest.
        glowEnabled: world.bloom && !world.asPainted
        glowStrength: 1.0
        glowIntensity: 0.6
        // Bloom bleeds a blur of the whole frame back over it, which a made sky is dim enough to take; a sky image is
        // bright all over, and would be washed out. It keeps the glow of what is brighter than white (its sun).
        glowBloom: world.skyImage ? 0 : 0.35
        glowHDRMinimumValue: 1.1
        glowBlendMode: ExtendedSceneEnvironment.GlowBlendMode.Additive

        aoEnabled: !world.flat
        aoStrength: 45
        aoDistance: 6

        antialiasingMode: SceneEnvironment.MSAA
        antialiasingQuality: SceneEnvironment.High
        specularAAEnabled: true
        // Nor with the noise added against banding.
        ditheringEnabled: !world.asPainted
    }

    readonly property Texture made: Texture {
        textureData: ProceduralSkyTextureData {
            skyTopColor: world.skyTop
            skyHorizonColor: world.skyHorizon
            groundHorizonColor: world.groundHorizon
            groundBottomColor: world.groundBottom
            sunLatitude: 18
            sunLongitude: 210
        }
    }
    readonly property Texture photographed: Texture {
        source: world.sky
        mappingMode: Texture.LightProbe
    }

    PerspectiveCamera {
        id: eyeCamera
        clipNear: 0.05
        clipFar: 2000

        Loader3D {
            // Not in a browser, where Sound plays without spatial audio (Sound.qml says why).
            active: world.hearing && Qt.platform.os !== "wasm"
            sourceComponent: Node {
                // One engine serves every sound in the process; this sets it to QQ's unit, the metre.
                AudioEngine { distanceScale: 100 }
                AudioListener {}
            }
        }
    }

    // Flat: centred on `target`, moved to the nearest whole device pixel so art pixels never straddle two. A view an odd
    // number of device pixels across has its middle inside a pixel, so its camera sits half a pixel over.
    OrthographicCamera {
        id: flatCamera
        horizontalMagnification: world.magnification
        verticalMagnification: world.magnification
        clipNear: 1
        clipFar: 4000
        position: Qt.vector3d(world.snapped(world.target.x, Math.round(world.width * world.dpr)),
                              world.snapped(world.target.y, Math.round(world.height * world.dpr)), 2000)
    }

    function snapped(v, devicePixels) {
        if (!wholePixels)
            return v
        const step = 1 / (pixelsPerUnit * zoom)
        return Math.round(v / step) * step + (devicePixels % 2 === 1 ? step / 2 : 0)
    }

    // A flat world's backdrop: the sky's colours top to bottom, beneath the scene.
    Rectangle {
        anchors.fill: parent
        z: -1
        visible: world.flat
        gradient: Gradient {
            GradientStop { position: 0; color: world.skyTop }
            GradientStop { position: 0.62; color: world.skyHorizon }
            GradientStop { position: 1; color: world.groundHorizon }
        }
    }

    // Position, then aim: in one place, so the camera never looks from where it was toward where it is going.
    function placeEye() {
        eyeCamera.position = world.eye
        eyeCamera.lookAt(world.target)
    }
    onEyeChanged: placeEye()
    onTargetChanged: placeEye()

    /// What a scene's interface is drawn on (the screen kinds, P3.8): a layer over the world, filling the view, made as
    /// the world is, so a screen in any scene shown here finds it (SceneIO::viewOf).
    property Item overlay: null
    Component {
        id: overlayLayer
        Item {
            anchors.fill: parent
            z: 1
        }
    }

    Component.onCompleted: {
        placeEye()
        overlay = overlayLayer.createObject(world)
    }

    Node {
        id: scene
        // What a scene under this world is seen through: where a ray is cast (SceneIO::viewOf, Logic.ray).
        readonly property QtObject qqView: world
    }
}
