pragma ComponentBehavior: Bound
import QtQuick
import QtQuick3D
import QtQuick3D.Helpers
import ARQADE

// A game's emblem: a small 3D scene of what the game is about, in the portal's gilt and ember. It turns slowly, faster
// when its card is hovered, and holds still with reduced motion.
View3D {
    id: emblem

    // "rift", "void", "synapse", "orbital", "flux" or "reversi".
    required property string kind
    property bool lively: false
    readonly property bool moving: !Theme.reducedMotion && emblem.visible

    property real t: 0
    property real speed: lively ? 2.4 : 1
    Behavior on speed { NumberAnimation { duration: 500; easing.type: Easing.OutCubic } }

    FrameAnimation {
        running: emblem.moving
        onTriggered: emblem.t += Math.min(frameTime, 0.05) * emblem.speed
    }

    environment: ExtendedSceneEnvironment {
        backgroundMode: SceneEnvironment.Transparent
        lightProbe: Texture { source: Theme.sky }
        probeExposure: 1.3
        tonemapMode: SceneEnvironment.TonemapModeFilmic
        glowEnabled: true
        glowStrength: 1.0
        glowIntensity: 0.45
        glowBloom: 0.2
        glowHDRMinimumValue: 1.2
        glowBlendMode: ExtendedSceneEnvironment.GlowBlendMode.Additive
        antialiasingMode: SceneEnvironment.MSAA
        antialiasingQuality: SceneEnvironment.High
        specularAAEnabled: true
    }

    PerspectiveCamera {
        position: Qt.vector3d(0, 18, 230)
        eulerRotation.x: -4
        fieldOfView: 30
    }
    DirectionalLight {
        eulerRotation: Qt.vector3d(-40, -35, 0)
        color: "#ffe2b8"
        brightness: 1.2
    }
    DirectionalLight {
        eulerRotation: Qt.vector3d(15, 150, 0)
        color: "#7f9bff"
        brightness: 0.3
    }

    Loader3D {
        sourceComponent: ({ "rift": rift, "void": runner, "synapse": synapse, "orbital": orbital, "flux": flux, "reversi": reversi })[emblem.kind] ?? null
    }

    component Gilt: PrincipledMaterial {
        baseColor: Theme.gilt
        metalness: 1.0
        roughness: 0.26
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

    // Rift Survivor: one signal, held in a ring, with the rift's shards turning about it.
    Component {
        id: rift
        Node {
            Model {
                source: "#Sphere"
                readonly property real beat: 0.3 + 0.03 * Math.sin(emblem.t * 3)
                scale: Qt.vector3d(beat, beat, beat)
                materials: Ember { strength: 1.8 }
            }
            Node {
                eulerRotation: Qt.vector3d(70, emblem.t * 30, 0)
                Model {
                    geometry: TorusGeometry { radius: 52; tubeRadius: 2.6; rings: 96; segments: 16 }
                    materials: Gilt { }
                }
            }
            Repeater3D {
                model: 5
                Model {
                    required property int index
                    readonly property real a: emblem.t * 0.9 + index * Math.PI * 2 / 5
                    source: "#Cone"
                    position: Qt.vector3d(Math.cos(a) * 78, Math.sin(a * 2) * 10, Math.sin(a) * 78)
                    eulerRotation: Qt.vector3d(emblem.t * 80 + index * 40, index * 70, 30)
                    scale: Qt.vector3d(0.09, 0.16, 0.09)
                    materials: index % 2 ? gilded : embered
                }
            }
            Gilt { id: gilded }
            Ember { id: embered; strength: 1.2; tint: Theme.accent }
        }
    }

    // Void Runner: a ship on three lanes, the collapse coming at it.
    Component {
        id: runner
        Node {
            eulerRotation: Qt.vector3d(14, -18 + Math.sin(emblem.t * 0.6) * 6, 0)
            Repeater3D {
                model: 4
                Model {
                    required property int index
                    source: "#Cube"
                    position: Qt.vector3d((index - 1.5) * 36, -24, -60)
                    scale: Qt.vector3d(0.012, 0.012, 3.4)
                    materials: Gilt { }
                }
            }
            Model {
                source: "#Cone"
                position: Qt.vector3d(Math.sin(emblem.t * 1.4) * 36, -12, 40)
                eulerRotation: Qt.vector3d(-90, 0, Math.sin(emblem.t * 1.4) * -18)
                scale: Qt.vector3d(0.22, 0.42, 0.12)
                materials: Gilt { baseColor: Theme.giltBright; roughness: 0.18 }
            }
            Model {
                source: "#Sphere"
                position: Qt.vector3d(Math.sin(emblem.t * 1.4) * 36, -12, 64)
                scale: Qt.vector3d(0.07, 0.07, 0.07)
                materials: Ember { strength: 2 }
            }
            Repeater3D {
                model: 3
                Model {
                    id: barrier
                    required property int index
                    readonly property real run: ((emblem.t * 0.45 + index / 3) % 1)
                    source: "#Cube"
                    position: Qt.vector3d((index - 1) * 36, -14, -230 + run * 300)
                    scale: Qt.vector3d(0.26, 0.16, 0.05)
                    materials: Ember { strength: 0.35 + barrier.run * 1.1; tint: Theme.accent }
                }
            }
        }
    }

    // Synapse: four nodes on a square, lit in turn, a signal to remember.
    Component {
        id: synapse
        Node {
            eulerRotation: Qt.vector3d(18, emblem.t * 14, 0)
            Node {
                eulerRotation.z: 45
                Ring { radius: 62; tubeRadius: 1.2; rings: 4; segments: 10 }
            }
            Repeater3D {
                model: 4
                Model {
                    required property int index
                    readonly property bool lit: Math.floor(emblem.t * 1.6) % 4 === [0, 2, 1, 3][index]
                    source: "#Sphere"
                    position: Qt.vector3d([1, -1, -1, 1][index] * 44, [1, 1, -1, -1][index] * 44, 0)
                    scale: Qt.vector3d(lit ? 0.26 : 0.2, lit ? 0.26 : 0.2, lit ? 0.26 : 0.2)
                    materials: lit ? litNode : darkNode
                }
            }
            Ember { id: litNode; strength: 2.2 }
            Gilt { id: darkNode; roughness: 0.34 }
        }
    }

    // Orbital: a world, its orbit, and the satellite to lock in the amber arc.
    Component {
        id: orbital
        Node {
            eulerRotation.x: 22
            Model {
                source: "#Sphere"
                scale: Qt.vector3d(0.46, 0.46, 0.46)
                eulerRotation.y: emblem.t * 20
                materials: Gilt { roughness: 0.42 }
            }
            Model {
                // Flat, as Qt makes it: the orbit the satellite runs on.
                geometry: TorusGeometry { radius: 82; tubeRadius: 0.9; rings: 120; segments: 8 }
                materials: Gilt { baseColor: Theme.giltDim }
            }
            Repeater3D {
                // The amber arc: a short run of embers on the orbit.
                model: 7
                Model {
                    required property int index
                    readonly property real a: 0.6 + index * 0.07
                    source: "#Sphere"
                    position: Qt.vector3d(Math.cos(a) * 82, 0, Math.sin(a) * 82)
                    scale: Qt.vector3d(0.045, 0.045, 0.045)
                    materials: Ember { strength: 1.0; tint: Theme.accent }
                }
            }
            Model {
                readonly property real a: emblem.t * 1.3
                source: "#Sphere"
                position: Qt.vector3d(Math.cos(a) * 82, 0, Math.sin(a) * 82)
                scale: Qt.vector3d(0.1, 0.1, 0.1)
                materials: Ember { strength: 2 }
            }
        }
    }

    // Flux Four: pulses dropped into columns, four in a line.
    Component {
        id: flux
        Node {
            scale: Qt.vector3d(0.62, 0.62, 0.62)
            eulerRotation: Qt.vector3d(6, -22 + Math.sin(emblem.t * 0.5) * 12, 0)
            // The board's columns, and its floor.
            Repeater3D {
                model: 5
                Model {
                    required property int index
                    source: "#Cube"
                    position: Qt.vector3d((index - 2) * 34, 0, 0)
                    scale: Qt.vector3d(0.014, 1.4, 0.014)
                    materials: Gilt { }
                }
            }
            Model {
                source: "#Cube"
                position.y: -70
                scale: Qt.vector3d(1.4, 0.02, 0.14)
                materials: Gilt { baseColor: Theme.giltDim }
            }
            // Four pulses dropped in turn, each settling one row higher: four on the diagonal. Then again.
            Repeater3D {
                model: 4
                Model {
                    id: pulse
                    required property int index
                    readonly property real cycle: (emblem.t * 0.3) % 1
                    readonly property real fall: Math.max(0, Math.min(1, cycle * 7 - index))
                    source: "#Cylinder"
                    position: Qt.vector3d((index - 1.5) * 34, -53 + index * 34 + (1 - fall) * (1 - fall) * 150, 0)
                    eulerRotation.x: 90
                    scale: Qt.vector3d(0.3, 0.07, 0.3)
                    opacity: fall > 0 ? 1 : 0
                    materials: Ember { strength: 0.6 + 1.0 * pulse.fall; tint: Theme.accent }
                }
            }
        }
    }

    // Rift Reversi: a disc turned over, gilt to obsidian and back.
    Component {
        id: reversi
        Node {
            eulerRotation.x: 24
            Repeater3D {
                // The board under it: a few gilt lines, fading out.
                model: 5
                Node {
                    id: line
                    required property int index
                    Model {
                        source: "#Cube"
                        position: Qt.vector3d((line.index - 2) * 36, -34, 0)
                        scale: Qt.vector3d(0.008, 0.008, 1.5)
                        materials: Gilt { baseColor: Theme.giltDim }
                    }
                    Model {
                        source: "#Cube"
                        position: Qt.vector3d(0, -34, (line.index - 2) * 36)
                        scale: Qt.vector3d(1.5, 0.008, 0.008)
                        materials: Gilt { baseColor: Theme.giltDim }
                    }
                }
            }
            Node {
                readonly property real phase: (emblem.t * 0.4) % 2
                readonly property real turn: phase < 1 ? Math.min(1, phase * 2.5) : Math.min(1, (phase - 1) * 2.5)
                position.y: Math.sin(turn * Math.PI) * 36
                eulerRotation.x: (phase < 1 ? 0 : 180) + turn * 180
                Model {
                    source: "#Cylinder"
                    position.y: 3
                    scale: Qt.vector3d(0.8, 0.055, 0.8)
                    materials: Gilt { baseColor: Theme.giltBright; roughness: 0.2 }
                }
                Model {
                    source: "#Cylinder"
                    position.y: -3
                    scale: Qt.vector3d(0.8, 0.055, 0.8)
                    materials: PrincipledMaterial { baseColor: "#0b0c10"; metalness: 0.6; roughness: 0.3 }
                }
                Model {
                    geometry: TorusGeometry { radius: 40; tubeRadius: 1.4; rings: 72; segments: 8 }
                    materials: Ember { strength: 0.9; tint: Theme.accent }
                }
            }
        }
    }
}
