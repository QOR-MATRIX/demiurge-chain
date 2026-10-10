// A spring of particles: sparks, embers, mist, magic. Soft points of light rise from it at `rate` a second, spread as
// they go and fade out after `life` seconds. It runs while the scene is edited as well as in play, so it is placed by
// how it looks.

import QtQuick
import QtQuick3D
import QtQuick3D.Particles3D
import QQ

Node {
    id: emitter

    readonly property string kind: "Emitter"
    readonly property var fields: ["name", "position", "colour", "rate", "life", "size", "speed", "spread"]
    readonly property var ranges: ({ rate: [1, 500], life: [0.2, 10], size: [0.01, 2], speed: [0, 20], spread: [0, 10] })

    property string name: "Emitter"
    property color colour: "#ffb15c"
    /// Particles a second.
    property real rate: 60
    /// Seconds each one lasts.
    property real life: 2.0
    /// Metres across, as each one starts.
    property real size: 0.12
    /// Metres a second, upward, as each one starts.
    property real speed: 1.5
    /// How far, in metres a second, each one wanders from straight up.
    property real spread: 0.6

    /// The particles alive now: for a check, or for the agent judging a scene (P3.3).
    readonly property int alive: system.loggingData ? system.loggingData.particlesUsed : 0

    ParticleSystem3D {
        id: system
        logging: true
        loggingData.loggingInterval: 250

        SpriteParticle3D {
            id: spark
            sprite: Texture { textureData: SoftDot {} }
            color: emitter.colour
            colorVariation: Qt.vector4d(0.08, 0.08, 0.08, 0)
            maxAmount: Math.ceil(emitter.rate * emitter.life * 1.2) + 16
            particleScale: 1
            billboard: true
            blendMode: SpriteParticle3D.Screen
            fadeInDuration: 120
            fadeOutDuration: Math.min(600, emitter.life * 400)
        }

        ParticleEmitter3D {
            particle: spark
            emitRate: emitter.rate
            lifeSpan: emitter.life * 1000
            lifeSpanVariation: emitter.life * 200
            // In metres: the sprite's own scale is set to 1, so the emitter's is the particle's size.
            particleScale: emitter.size
            particleEndScale: emitter.size * 0.4
            particleScaleVariation: emitter.size * 0.3
            velocity: VectorDirection3D {
                direction: Qt.vector3d(0, emitter.speed, 0)
                directionVariation: Qt.vector3d(emitter.spread, emitter.spread * 0.3, emitter.spread)
            }
        }
    }

    // What a click in the Studio lands on: particles cannot be picked.
    Model {
        source: "#Sphere"
        scale: Qt.vector3d(0.003, 0.003, 0.003)
        pickable: true
        castsShadows: false
        receivesShadows: false
        visible: !(emitter.parent && emitter.parent.playing === true)
        materials: PrincipledMaterial {
            baseColor: "#000000"
            emissiveFactor: Qt.vector3d(emitter.colour.r * 2, emitter.colour.g * 2, emitter.colour.b * 2)
        }
    }
}
