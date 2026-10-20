// The sounds a scene's logic plays once (Logic.play, DIRECTION P3.13): a coin, a hit, a click. Each is a voice of its
// own, placed in the world where logic says (`at`, heard from there as a Sound is) or heard everywhere alike, in a
// volume group (Mixer), and put away when its file has played through. Not an entity and not in the scene's tree: a
// scene keeps one, nothing of it is written to the scene's file, and a placed voice is in the scene only while it sounds.
//
// In a browser each is a WebVoice, its volume following the distance from the eye and its side the eye's: panned.

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.SpatialAudio
import QQ

QtObject {
    id: voices

    /// The scene they play in.
    property QtObject scene: null
    /// The voices sounding now.
    property var active: []
    /// Whether this is a web page, where voices are WebVoices. Set by a check, to hear as a browser would.
    property bool web: Qt.platform.os === "wasm"

    /// Play `url` once with `options` { at, volume, group, reach }: the voice, or null if it could not be made.
    function oneShot(url, options) {
        const group = options.group ?? "effects"
        const own = Math.max(0, options.volume ?? 1)
        const reach = Math.max(1, options.reach ?? 25)
        const at = options.at ?? null
        const voice = web ? webShot.createObject(voices, { source: url, own: own, group: group, at: at, reach: reach })
                    : at !== null ? placed.createObject(voices.scene, { source: url, own: own, group: group, position: at,
                                                                        distanceCutoff: reach })
                    : everywhere.createObject(voices, { source: url, own: own, group: group })
        if (!voice)
            return null
        voices.active = voices.active.concat([voice])
        // Put away once played through: Qt's spatial sounds do not say when they have finished, so their files are
        // measured (Mixer.lengthOf), and one that cannot be is given ten seconds.
        const length = Mixer.lengthOf(url)
        later.createObject(voice, { voice: voice, interval: Math.ceil(((length > 0 ? length : 10) + 0.25) * 1000) })
        return voice
    }

    function _done(voice) {
        voices.active = voices.active.filter(v => v !== voice)
        voice.destroy()
    }

    /// The level a voice in `group` plays at: the Mixer's, read so that a change of level is heard at once.
    function _gain(group) {
        const levels = Mixer.levels
        return (levels.master ?? 1) * (group === "master" ? 1 : levels[group] ?? 1)
    }

    readonly property Component placed: Component {
        SpatialSound {
            property real own: 1
            property string group: "effects"
            volume: own * voices._gain(group)
            distanceModel: SpatialSound.Logarithmic
            size: 0.5
            loops: 1
            autoPlay: true
        }
    }
    readonly property Component everywhere: Component {
        AmbientSound {
            property real own: 1
            property string group: "effects"
            volume: own * voices._gain(group)
            loops: 1
            autoPlay: true
        }
    }
    readonly property Component webShot: Component {
        WebVoice {
            property real own: 1
            property string group: "effects"
            property var at: null
            property real reach: 25
            // Heard through the eye, as natively: louder nearer, and on the side of the eye it is on.
            readonly property QtObject eye: voices.scene && SceneIO.viewOf(voices.scene)
                                            ? SceneIO.viewOf(voices.scene).activeCamera : null
            readonly property real distance: eye && at !== null ? eye.scenePosition.minus(at).length() : 0
            volume: own * voices._gain(group) * (at === null ? 1 : distance > reach ? 0 : 1 / Math.max(1, distance))
            pan: eye && at !== null && distance > 0.01
                 ? Math.max(-1, Math.min(1, at.minus(eye.scenePosition).normalized().dotProduct(eye.right))) : 0
            loops: 1
            Component.onCompleted: play()
        }
    }
    readonly property Component later: Component {
        Timer {
            property QtObject voice: null
            running: true
            onTriggered: voices._done(voice)
        }
    }
}
