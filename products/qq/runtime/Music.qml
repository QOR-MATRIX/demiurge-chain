// A scene's music (DIRECTION P3.13, ADR-105): a track heard everywhere alike, from when play starts, in its volume
// group ("music" unless named otherwise, so a player's music level turns it down). Logic changes the track by setting
// `source`: the old one fades out as the new one fades in, over `fade` seconds. It plays on while the world is paused,
// as a menu's music would, and never in a replay, which has no ears.
//
// In a browser its two voices are WebVoices, as a Sound's are there.

import QtQml
import QtQuick
import QtQuick3D
import QtQuick3D.SpatialAudio
import QQ

Node {
    id: music

    readonly property string kind: "Music"
    readonly property var fields: ["name", "source", "volume", "fade", "loops", "group"]
    readonly property var ranges: ({ volume: [0, 2], fade: [0, 10] })
    readonly property var fileTypes: ({ source: ["Sound (*.wav *.mp3 *.ogg *.flac)"] })

    property string name: "Music"
    /// The track, kept relative to the scene file.
    property url source
    property real volume: 0.8
    /// Seconds over which one track gives way to the next.
    property real fade: 1.5
    /// Over and over, or once.
    property bool loops: true
    /// The volume group it plays in (Mixer).
    property string group: "music"

    /// True while the scene this belongs to plays, and whether it is heard: not in a replay.
    readonly property bool live: parent !== null && parent.playing === true
    readonly property bool sounding: live && !(parent && parent.replaying)
    /// Whether this is a web page, where its voices are WebVoices. Set by a check, to hear as a browser would.
    property bool web: Qt.platform.os === "wasm"

    /// Its two voices' tracks (whole URLs, or empty), which of them plays `source`, and how far it has faded in.
    property var tracks: ["", ""]
    property int current: 0
    property real mix: 1
    /// One of its two voices (0 or 1), for what wants to see it (a check), or null when it has no track.
    function voice(index) {
        return _voices[index]
    }

    // A voice is made when it is given a track, and let go with it (one given no file would try to load one). Each is a
    // child of this, so goes with it.
    property var _voices: [null, null]
    function _sync() {
        const voices = _voices.slice()
        for (let i = 0; i < 2; ++i) {
            const track = sounding ? tracks[i] : ""
            const was = voices[i]
            if (was && String(was.source) === track)
                continue
            if (was)
                was.destroy()
            voices[i] = track === "" ? null : (web ? webVoice : ambientVoice).createObject(music, { index: i,
                                                                                                    source: track })
        }
        _voices = voices
    }
    onTracksChanged: _sync()

    function _start() {
        tracks = current === 0 ? [String(SceneIO.resolvedUrl(music, source)), ""]
                               : ["", String(SceneIO.resolvedUrl(music, source))]
        mix = 1
    }
    onSoundingChanged: sounding ? _start() : _sync()
    Component.onCompleted: if (sounding) _start()
    onSourceChanged: {
        if (!sounding)
            return
        // The new track on the other voice, fading in while this one fades out.
        const next = 1 - current
        const both = tracks.slice()
        both[next] = String(SceneIO.resolvedUrl(music, source))
        tracks = both
        current = next
        mix = fade > 0 ? 0 : 1
        if (mix === 1)
            _letGo()
    }

    /// The voice faded out is let go.
    function _letGo() {
        const both = tracks.slice()
        both[1 - current] = ""
        tracks = both
    }

    // The fade moves with the frames, by their time: music fades as heard, paused or not.
    FrameAnimation {
        running: music.sounding && music.mix < 1
        onTriggered: {
            music.mix = Math.min(1, music.mix + frameTime / Math.max(0.001, music.fade))
            if (music.mix >= 1)
                music._letGo()
        }
    }

    /// A voice's volume: the track's, its group's level, and how far it has faded.
    function _level(index) {
        const levels = Mixer.levels
        const gain = (levels.master ?? 1) * (group === "master" ? 1 : levels[group] ?? 1)
        return volume * gain * (index === current ? mix : 1 - mix)
    }

    readonly property Component ambientVoice: Component {
        AmbientSound {
            property int index
            volume: music._level(index)
            loops: music.loops ? AmbientSound.Infinite : 1
            autoPlay: true
        }
    }

    readonly property Component webVoice: Component {
        WebVoice {
            property int index
            volume: music._level(index)
            loops: music.loops ? -2 : 1
            Component.onCompleted: play()
        }
    }

    // What a click in the Studio lands on: a small mark, hidden in play.
    Model {
        source: "#Cone"
        scale: Qt.vector3d(0.003, 0.003, 0.003)
        pickable: true
        visible: !music.live
        castsShadows: false
        receivesShadows: false
        materials: PrincipledMaterial {
            baseColor: "#000000"
            emissiveFactor: Qt.vector3d(1.6, 1.2, 0.4)
        }
    }
}
