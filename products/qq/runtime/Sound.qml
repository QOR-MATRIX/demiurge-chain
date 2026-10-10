// A sound in the world: a waterfall, a hum, a bird. It is heard from where it is, louder nearer, panned to the side it
// is on and silent beyond `reach`, through the ears of the camera (World's listener). It sounds only in play.
//
// In a browser it is quieter with distance from the scene's player and silent beyond `reach`, but not panned: Qt's
// spatial audio makes its sound on a thread of its own, and a web page can make sound only on its main thread (Web
// Audio is not there in a worker), so there it is a plain sound effect (WebVoice, QQ's own, so a game never needs Qt
// Multimedia's QML, which also offers the camera) whose volume follows the player.

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.SpatialAudio
import QQ

Node {
    id: sound

    readonly property string kind: "Sound"
    readonly property var fields: ["name", "position", "source", "volume", "reach", "loops"]
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

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// Whether this is a web page, where sounds are not placed in space (above).
    readonly property bool web: Qt.platform.os === "wasm"
    readonly property bool sounding: live && String(source) !== ""
    /// The sound playing, or null.
    readonly property QtObject playing: web ? webVoice.object : voice.item

    Loader3D {
        id: voice
        active: sound.sounding && !sound.web
        sourceComponent: SpatialSound {
            source: SceneIO.resolvedUrl(sound, sound.source)
            volume: sound.volume
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
            readonly property QtObject listener: sound.parent ? sound.parent.player : null
            readonly property real distance: listener ? listener.feet.minus(sound.scenePosition).length() : 0
            source: SceneIO.resolvedUrl(sound, sound.source)
            // As SpatialSound's logarithmic model: full within a metre, half at each doubling, none beyond the reach.
            volume: distance > sound.reach ? 0 : Math.min(1, sound.volume / Math.max(1, distance))
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
