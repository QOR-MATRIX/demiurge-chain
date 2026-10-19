pragma ComponentBehavior: Bound
import QtQuick
import QtQuick3D
import QtQuick3D.Helpers
import QtQuick3D.Particles3D
import ARQADE

// ARQADE's portal: an astrolabe of gilt rings around an ember core, the homepage's triangle and square turning inside it,
// embers rising through it and leaning toward the pointer. Physically based metal lit by a warm sky, filmic tonemapping,
// bloom and a vignette (ADR-102 decision 1). With reduced motion it holds still, embers and all.
View3D {
    id: portal

    // Where the pointer is over the window, -1 to 1 on each axis; (0, 0) at rest.
    property point pointer: Qt.point(0, 0)
    // How far the page has scrolled, in pixels: the scene rises a little slower than the page.
    property real scroll: 0
    // Where the core sits in the view, 0 to 1 across and down.
    property real coreAt: 0.68
    property real coreDown: 0.36
    readonly property bool moving: !Theme.reducedMotion && portal.visible

    // The time the rings turn by, in seconds, and the pointer as the camera follows it, eased.
    property real t: 0
    property real lookX: 0
    property real lookY: 0

    FrameAnimation {
        running: portal.moving
        onTriggered: {
            portal.t += Math.min(frameTime, 0.05)
            const ease = 1 - Math.pow(0.04, Math.min(frameTime, 0.05))
            portal.lookX += (portal.pointer.x - portal.lookX) * ease
            portal.lookY += (portal.pointer.y - portal.lookY) * ease
        }
    }

    environment: ExtendedSceneEnvironment {
        backgroundMode: SceneEnvironment.Color
        clearColor: Theme.void_
        lightProbe: Texture { source: Theme.sky }
        probeExposure: 1.4

        tonemapMode: SceneEnvironment.TonemapModeFilmic
        exposure: 1.15

        // Bloom is added after the tonemapping, so it is kept gentle: strong, it turns ember to yellow.
        glowEnabled: true
        glowStrength: 1.0
        glowIntensity: 0.55
        glowBloom: 0.25
        glowHDRMinimumValue: 1.2
        glowQualityHigh: true
        glowUseBicubicUpscale: true
        glowBlendMode: ExtendedSceneEnvironment.GlowBlendMode.Additive

        vignetteEnabled: true
        vignetteStrength: 1.1
        vignetteRadius: 0.42
        vignetteColor: Theme.void_

        antialiasingMode: SceneEnvironment.MSAA
        antialiasingQuality: SceneEnvironment.High
        specularAAEnabled: true
        ditheringEnabled: true
    }

    camera: eye
    Node {
        id: rig
        eulerRotation: Qt.vector3d(-portal.lookY * 5, portal.lookX * 8, 0)
        PerspectiveCamera {
            id: eye
            position: Qt.vector3d(0, 0, 900)
            fieldOfView: 34
            clipNear: 1
            clipFar: 6000
        }
    }

    DirectionalLight {
        eulerRotation: Qt.vector3d(-35, -40, 0)
        color: "#ffe2b8"
        brightness: 1.1
        ambientColor: "#0a0806"
    }
    DirectionalLight {
        // A cool rim from behind, so the metal reads against the dark.
        eulerRotation: Qt.vector3d(20, 160, 0)
        color: "#7f9bff"
        brightness: 0.35
    }

    // One pixel of the view in the scene's units, where the core is.
    readonly property real unit: 2 * eye.z * Math.tan(eye.fieldOfView * Math.PI / 360) / Math.max(1, portal.height)

    // The scene, set to the side the page leaves open, rising a third as fast as the page scrolls.
    Node {
        id: astrolabe
        x: (portal.coreAt - 0.5) * portal.width * portal.unit
        y: ((0.5 - portal.coreDown) * portal.height + portal.scroll * 0.35) * portal.unit

        PointLight {
            color: Theme.accentBright
            brightness: 22
            quadraticFade: 2.2
            linearFade: 0
        }

        // The ember core: a white-hot heart seen through an orange shell, and its halo, light screened over everything
        // behind it, facing the eye.
        Model {
            source: "#Sphere"
            scale: Qt.vector3d(0.34, 0.34, 0.34)
            materials: Ember { strength: 5; tint: "#ffd9a0" }
        }
        Model {
            id: core
            source: "#Sphere"
            readonly property real beat: 1 + 0.035 * Math.sin(portal.t * 2.1) + 0.02 * Math.sin(portal.t * 5.3)
            scale: Qt.vector3d(0.62 * beat, 0.62 * beat, 0.62 * beat)
            materials: Ember {
                strength: 2.6
                alphaMode: PrincipledMaterial.Blend
                opacity: 0.55
            }
        }
        Model {
            source: "#Rectangle"
            eulerRotation: rig.eulerRotation
            scale: Qt.vector3d(4.2 * core.beat, 4.2 * core.beat, 1)
            materials: PrincipledMaterial {
                lighting: PrincipledMaterial.NoLighting
                baseColorMap: Texture { source: Theme.halo }
                alphaMode: PrincipledMaterial.Blend
                blendMode: PrincipledMaterial.Screen
                depthDrawMode: Material.NeverDepthDraw
            }
        }

        // The rings: three gilt circles on three axes, slowest outermost.
        Node {
            eulerRotation: Qt.vector3d(62, portal.t * 9, 8)
            Ring { radius: 175; tubeRadius: 3.4; rings: 160; segments: 24 }
        }
        Node {
            eulerRotation: Qt.vector3d(18 + portal.t * 6, -34, portal.t * -4)
            Ring { radius: 232; tubeRadius: 2.4; rings: 180; segments: 20; material: Gilt { roughness: 0.32 } }
        }
        Node {
            id: outer
            eulerRotation: Qt.vector3d(-8, 6, portal.t * 2.5)
            Ring {
                radius: 300; tubeRadius: 1.7; rings: 220; segments: 16
                material: Gilt { baseColor: Theme.giltDim; roughness: 0.4 }
            }
            // An astrolabe's degree marks: every fifteenth degree, the quarters longer.
            Repeater3D {
                model: 24
                Model {
                    id: mark
                    required property int index
                    readonly property real angle: index * Math.PI / 12
                    readonly property bool quarter: index % 6 === 0
                    source: "#Cube"
                    position: Qt.vector3d(Math.cos(angle) * 300, Math.sin(angle) * 300, 0)
                    eulerRotation: Qt.vector3d(0, 0, index * 15 + 90)
                    scale: Qt.vector3d(0.018, quarter ? 0.34 : 0.16, 0.018)
                    materials: Gilt { baseColor: mark.quarter ? Theme.giltBright : Theme.gilt }
                }
            }
        }

        // The homepage's triangle and square, turning against each other, faintly alight.
        Node {
            eulerRotation: Qt.vector3d(0, 0, 90 + portal.t * -7)
            Ring { radius: 128; tubeRadius: 1.6; rings: 3; segments: 12; material: Ember { strength: 3.2; tint: Theme.accent } }
        }
        Node {
            eulerRotation: Qt.vector3d(0, 0, 45 + portal.t * 5)
            Ring {
                radius: 128; tubeRadius: 1.3; rings: 4; segments: 12
                material: Gilt { baseColor: Theme.giltBright; emissiveFactor: Theme.glow(Theme.gilt, 0.6) }
            }
        }

        // Embers: rising from below the rings, drifting, leaning toward the pointer.
        ParticleSystem3D {
            id: embers
            startTime: 9000
            paused: !portal.moving

            ModelParticle3D {
                id: ember
                maxAmount: 320
                fadeInEffect: Particle3D.FadeScale
                fadeOutEffect: Particle3D.FadeScale
                fadeInDuration: 900
                fadeOutDuration: 1800
                delegate: Component {
                    Model {
                        source: "#Sphere"
                        scale: Qt.vector3d(0.028, 0.028, 0.028)
                        materials: Ember { strength: 2.2; tint: Theme.accent }
                    }
                }
            }
            ParticleEmitter3D {
                particle: ember
                position: Qt.vector3d(0, -330, 0)
                shape: ParticleShape3D { type: ParticleShape3D.Cube; extents: Qt.vector3d(460, 20, 200) }
                emitRate: 34
                lifeSpan: 8000
                lifeSpanVariation: 2500
                particleScaleVariation: 0.7
                velocity: VectorDirection3D {
                    direction: Qt.vector3d(0, 70, 0)
                    directionVariation: Qt.vector3d(18, 26, 12)
                }
            }
            Wander3D {
                globalAmount: Qt.vector3d(40, 0, 20)
                globalPace: Qt.vector3d(0.18, 0, 0.12)
                uniqueAmount: Qt.vector3d(20, 10, 10)
                uniquePace: Qt.vector3d(0.4, 0.3, 0.3)
            }
            Gravity3D {
                direction: Qt.vector3d(portal.lookX, -portal.lookY, 0)
                magnitude: 22 * Math.hypot(portal.lookX, portal.lookY)
            }
        }
    }

    component Gilt: PrincipledMaterial {
        baseColor: Theme.gilt
        metalness: 1.0
        roughness: 0.24
        specularAmount: 1.0
    }
    // Light, not lit: black with no reflection, so only what it gives off shows.
    component Ember: PrincipledMaterial {
        property real strength: 4
        property color tint: Theme.accent
        baseColor: "#000000"
        metalness: 0
        roughness: 1
        specularAmount: 0
        emissiveFactor: Theme.glow(tint, strength)
    }
    // Qt's torus lies flat, around the up axis; this one stands up to face the eye.
    component Ring: Model {
        property alias radius: torus.radius
        property alias tubeRadius: torus.tubeRadius
        property alias rings: torus.rings
        property alias segments: torus.segments
        property Material material: Gilt { }
        eulerRotation.x: 90
        geometry: TorusGeometry { id: torus }
        materials: [material]
    }
}
