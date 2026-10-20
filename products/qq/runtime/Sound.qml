// A sound in the world: a waterfall, a hum, a bird. It is heard from where it is, louder nearer, panned to the side it
// is on and silent beyond `reach`, through the ears of the camera (World's listener). It sounds only in play, and never
// in a replay, which has no ears.
//
// It belongs to a volume group (P3.13: "effects" unless named otherwise), so what is heard is its volume times the
// group's level and the master level (Mixer).
//
// In a browser Qt's spatial audio cannot sound: it makes its sound on a thread of its own, and a web page can make sound
// only on its main thread (Web Audio is not there in a worker). There it is a WebVoice, QQ's own, on the page's Web
// Audio: quieter with distance from the eye, silent beyond `reach`, and panned to the side of the eye it is on.

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.SpatialAudio
import QQ

Node {
    id: sound

    readonly property string kind: "Sound"
    readonly property var fields: ["name", "position", "source", "volume", "reach", "loops", "group"]
    readonly property var ranges: ({ volume: [0, 2], reach: [1, 200] })
    readonly property var fileTypes: ({ source: ["Sound (*.wav *.mp3 *.ogg *.flac)"] })

    property string name: "Sound"
    /// The sound file (.wav, or anything the platform plays); kept relative to the scene file.
    property url source
    property real volume: 1.0
    /// Metres beyond which it is not heard.
    property real reach: 25
    /// Over and over, or once when play starts.
    property bool loops: true
    /// The volume group it plays in (Mixer).
    property string group: "effects"

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// Whether this is a web page, where sounds are WebVoices (above). Set by a check, to hear as a browser would.
    property bool web: Qt.platform.os === "wasm"
    readonly property bool sounding: live && String(source) !== "" && !(parent && parent.replaying)
    /// The sound playing, or null.
    readonly property QtObject playing: web ? webVoice.object : voice.item

    /// Its group's level times the master level, read so that a change of level is heard at once.
    readonly property real gain: (Mixer.levels.master ?? 1) * (group === "master" ? 1 : Mixer.levels[group] ?? 1)

    Loader3D {
        id: voice
        active: sound.sounding && !sound.web
        sourceComponent: SpatialSound {
            source: SceneIO.resolvedUrl(sound, sound.source)
            volume: sound.volume * sound.gain
            distanceModel: SpatialSound.Logarithmic
            size: 0.5
            distanceCutoff: sound.reach
            loops: sound.loops ? SpatialSound.Infinite : 1
            autoPlay: true
        }
    }

    Instantiator {
        id: webVoice
        active: sound.sounding && sound.web
        delegate: WebVoice {
            // Heard through the eye, as natively.
            readonly property QtObject eye: sound.parent && SceneIO.viewOf(sound.parent)
                                            ? SceneIO.viewOf(sound.parent).activeCamera : null
            readonly property real distance: eye ? eye.scenePosition.minus(sound.scenePosition).length() : 0
            source: SceneIO.resolvedUrl(sound, sound.source)
            // As SpatialSound's logarithmic model: full within a metre, half at each doubling, none beyond the reach.
            volume: distance > sound.reach ? 0 : Math.min(1, sound.volume * sound.gain / Math.max(1, distance))
            pan: eye && distance > 0.01
                 ? Math.max(-1, Math.min(1, sound.scenePosition.minus(eye.scenePosition).normalized().dotProduct(eye.right)))
                 : 0
            loops: sound.loops ? -2 : 1  // -2: for ever
            Component.onCompleted: play()
        }
    }

    // What a click in the Studio lands on, and what shows where the sound is: hidden in play.
    Model {
        source: "#Sphere"
        scale: Qt.vector3d(0.0025, 0.0025, 0.0025)
        pickable: true
        visible: !sound.live
        castsShadows: false
        receivesShadows: false
        materials: PrincipledMaterial {
            baseColor: "#000000"
            emissiveFactor: Qt.vector3d(0.3, 1.8, 1.4)
        }
    }
}
