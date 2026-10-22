import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtMultimedia
import QOR.Media

// The player (ADR-094 decision 2): Qt Multimedia on its FFmpeg backend, so every container and codec FFmpeg reads plays.
// Controls fade in with the pointer and out while it rests; the keyboard does everything: Space or K play and pause,
// arrows seek five seconds and change the volume, J and L ten seconds, M mutes, F fills the screen, [ and ] change speed,
// C and A cycle subtitles and audio tracks, Esc goes back. It resumes where the Library says it stopped, and tells the
// Library where it stopped and how long the file is.
FocusScope {
    id: player
    objectName: "player"
    required property string path
    required property string kind
    required property real resumeAt
    required property var link
    signal back()
    signal fullScreenToggled()
    focus: true

    readonly property bool playing: media.playbackState === MediaPlayer.PlayingState
    readonly property real seconds: media.position / 1000
    readonly property real length: media.duration / 1000
    property bool chrome: true
    property bool learned: false
    property bool consumesEscape: false
    readonly property var rates: [0.5, 0.75, 1, 1.25, 1.5, 2]

    function leaving() {
        if (length > 0)
            link.reportPosition(path, seconds >= length - 5 ? 0 : seconds)
        media.stop()
    }
    function pauseForHide() {
        media.pause()
        if (length > 0)
            link.reportPosition(path, seconds)
    }
    function wake() {
        chrome = true
        rest.restart()
    }
    // Whether the keyboard is on one of the controls: they stay while it is, so Tab never lands on something hidden.
    function steering() {
        for (let item = player.Window.activeFocusItem; item; item = item.parent)
            if (item === controls || item === top)
                return true
        return false
    }
    // The keyboard back from a control to the player, so its keys work and the controls rest again. Focusing the scope
    // alone would hand it back to the control that had it last.
    function release() {
        const item = player.Window.activeFocusItem
        if (item && item !== player && steering())
            item.focus = false
        player.forceActiveFocus()
    }
    function seekBy(delta) {
        media.position = Math.max(0, Math.min(media.duration, media.position + delta * 1000))
        wake()
    }
    // Subtitles go round to off after the last; audio goes round to the first.
    function nextSubtitle() {
        const n = media.subtitleTracks.length
        if (n === 0) return
        media.activeSubtitleTrack = media.activeSubtitleTrack + 1 >= n ? -1 : media.activeSubtitleTrack + 1
        wake()
    }
    function nextAudio() {
        const n = media.audioTracks.length
        if (n < 2) return
        media.activeAudioTrack = (Math.max(0, media.activeAudioTrack) + 1) % n
        wake()
    }
    function trackName(track, i) {
        const lang = track.stringValue(MediaMetaData.Language)
        const title = track.stringValue(MediaMetaData.Title)
        return title || lang || qsTr("Track %1").arg(i + 1)
    }

    MediaPlayer {
        id: media
        source: Files.urlOf(player.path)
        audioOutput: AudioOutput {
            id: speaker
            volume: settings.volume
            muted: settings.muted
        }
        videoOutput: screen
        onMediaStatusChanged: {
            if (mediaStatus === MediaPlayer.LoadedMedia && !player.learned) {
                player.learned = true
                const size = metaData.value(MediaMetaData.Resolution)
                link.reportLearned(player.path, duration / 1000, size ? size.width : 0, size ? size.height : 0)
                if (player.resumeAt > 5 && player.resumeAt < duration / 1000 - 5)
                    position = player.resumeAt * 1000
                play()
            }
            if (mediaStatus === MediaPlayer.EndOfMedia)
                player.chrome = true
        }
        onErrorOccurred: (error, message) => problem.text = message || qsTr("This file could not be played.")
    }

    QtObject {
        id: settings
        property real volume: 0.9
        property bool muted: false
    }

    Timer {
        // Where it stopped, every few seconds while it plays, for the Library's Continue shelf.
        interval: 5000
        repeat: true
        running: player.playing
        onTriggered: player.link.reportPosition(player.path, player.seconds)
    }
    Timer {
        id: rest
        interval: 2600
        onTriggered: if (player.playing && !player.steering()) player.chrome = false
    }
    Connections {
        target: player.Window
        // On a control, they show and stay; leaving them, they rest as after the pointer stops.
        function onActiveFocusItemChanged() { player.steering() ? player.wake() : rest.restart() }
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.void_
    }

    VideoOutput {
        id: screen
        anchors.fill: parent
        fillMode: VideoOutput.PreserveAspectFit
        visible: media.hasVideo
    }

    // Music: its title, its artist, and a slow glow in the theme's accent, which stays still for less motion.
    Item {
        anchors.fill: parent
        visible: !media.hasVideo
        Rectangle {
            anchors.centerIn: parent
            width: Math.min(parent.width, parent.height) * 0.62
            height: width
            radius: width / 2
            gradient: Gradient {
                GradientStop { position: 0; color: Qt.alpha(Theme.accentBright, 0.35) }
                GradientStop { position: 1; color: "transparent" }
            }
            SequentialAnimation on scale {
                running: player.playing && !Theme.reducedMotion
                loops: Animation.Infinite
                NumberAnimation { to: 1.08; duration: 1800; easing.type: Easing.InOutSine }
                NumberAnimation { to: 0.96; duration: 1800; easing.type: Easing.InOutSine }
            }
        }
        ColumnLayout {
            anchors.centerIn: parent
            spacing: 6
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: media.metaData.stringValue(MediaMetaData.Title) || Files.nameOf(player.path)
                color: Theme.ink
                font.family: Theme.ui
                font.pixelSize: Theme.sizeFigure
                font.weight: Font.DemiBold
            }
            Text {
                Layout.alignment: Qt.AlignHCenter
                text: [media.metaData.stringValue(MediaMetaData.ContributingArtist), media.metaData.stringValue(MediaMetaData.AlbumTitle)].filter(Boolean).join(" · ")
                color: Theme.inkMuted
                font.family: Theme.text
                font.pixelSize: Theme.sizeTitle
            }
        }
    }

    Text {
        id: problem
        anchors.centerIn: parent
        visible: text.length > 0
        color: Theme.inkBody
        font.family: Theme.text
        font.pixelSize: Theme.sizeTitle
    }

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: player.chrome ? Qt.ArrowCursor : Qt.BlankCursor
        onPositionChanged: player.wake()
        onClicked: { player.release(); player.playing ? media.pause() : media.play(); player.wake() }
        onDoubleClicked: player.fullScreenToggled()
    }

    TopBar {
        id: top
        anchors { left: parent.left; right: parent.right; top: parent.top }
        title: media.metaData.stringValue(MediaMetaData.Title) || Files.nameOf(player.path)
        subtitle: [player.length > 0 ? Theme.clock(player.length) : "", media.metaData.value(MediaMetaData.Resolution) ? media.metaData.value(MediaMetaData.Resolution).height + "p" : ""].filter(Boolean).join(" · ")
        opacity: player.chrome ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: Theme.standard; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }
        onBack: player.back()
    }

    // The controls, over a scrim at the bottom.
    Rectangle {
        id: controls
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 118
        opacity: player.chrome ? 1 : 0
        visible: opacity > 0
        Behavior on opacity { NumberAnimation { duration: Theme.standard; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }
        gradient: Gradient {
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 1; color: Qt.alpha(Theme.void_, 0.94) }
        }

        // The scrubbing bar, with the time under the pointer.
        Item {
            id: scrub
            anchors { left: parent.left; right: parent.right; bottom: buttons.top; leftMargin: 24; rightMargin: 24; bottomMargin: 6 }
            height: 22
            property real hoverAt: -1
            Rectangle {
                id: groove
                anchors.verticalCenter: parent.verticalCenter
                width: parent.width
                height: scrubArea.containsMouse || scrubArea.pressed ? 6 : 4
                radius: height / 2
                color: Qt.alpha(Theme.ink, 0.18)
                Behavior on height { NumberAnimation { duration: Theme.quick; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }
                Rectangle {
                    width: media.duration > 0 ? parent.width * media.position / media.duration : 0
                    height: parent.height
                    radius: height / 2
                    color: Theme.accent
                }
                Rectangle {
                    x: (media.duration > 0 ? parent.width * media.position / media.duration : 0) - width / 2
                    anchors.verticalCenter: parent.verticalCenter
                    width: 14
                    height: 14
                    radius: width / 2
                    color: Theme.accentBright
                    visible: scrubArea.containsMouse || scrubArea.pressed
                }
            }
            Rectangle {
                visible: scrub.hoverAt >= 0
                x: Math.max(0, Math.min(parent.width - width, scrub.hoverAt * parent.width - width / 2))
                y: -30
                width: hoverText.implicitWidth + 14
                height: 24
                radius: Theme.radiusControl
                color: Theme.raised
                border.color: Theme.edge
                Text {
                    id: hoverText
                    anchors.centerIn: parent
                    text: Theme.clock(scrub.hoverAt * player.length)
                    color: Theme.ink
                    font.family: Theme.mono
                    font.pixelSize: Theme.sizeUi
                }
            }
            MouseArea {
                id: scrubArea
                anchors.fill: parent
                hoverEnabled: true
                onPositionChanged: (mouse) => {
                    scrub.hoverAt = Math.max(0, Math.min(1, mouse.x / width))
                    if (pressed) media.position = scrub.hoverAt * media.duration
                    player.wake()
                }
                onExited: scrub.hoverAt = -1
                onPressed: (mouse) => media.position = Math.max(0, Math.min(1, mouse.x / width)) * media.duration
            }
        }

        RowLayout {
            id: buttons
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom; leftMargin: 18; rightMargin: 18; bottomMargin: 14 }
            spacing: 6
            IconButton {
                iconName: player.playing ? "pause" : "play"
                size: 48
                ToolTip.text: player.playing ? qsTr("Pause (Space)") : qsTr("Play (Space)")
                onClicked: player.playing ? media.pause() : media.play()
            }
            IconButton { iconName: "prev"; ToolTip.text: qsTr("Back 10 s (J)"); onClicked: player.seekBy(-10) }
            IconButton { iconName: "next"; ToolTip.text: qsTr("Forward 10 s (L)"); onClicked: player.seekBy(10) }
            IconButton {
                iconName: settings.muted || settings.volume === 0 ? "mute" : "volume"
                ToolTip.text: qsTr("Mute (M)")
                onClicked: settings.muted = !settings.muted
            }
            Slider {
                id: volume
                Layout.preferredWidth: 110
                from: 0; to: 1
                value: settings.volume
                onMoved: { settings.volume = value; settings.muted = false }
                focusPolicy: Qt.TabFocus
                Accessible.name: qsTr("Volume")
                background: Rectangle {
                    x: volume.leftPadding; y: volume.topPadding + volume.availableHeight / 2 - height / 2
                    width: volume.availableWidth; height: 4; radius: 2
                    color: Qt.alpha(Theme.ink, 0.18)
                    Rectangle { width: volume.visualPosition * parent.width; height: parent.height; radius: 2; color: Theme.inkBody }
                }
                handle: Rectangle {
                    x: volume.leftPadding + volume.visualPosition * (volume.availableWidth - width)
                    y: volume.topPadding + volume.availableHeight / 2 - height / 2
                    width: 12; height: 12; radius: width / 2; color: Theme.ink
                    FocusRing { visible: volume.visualFocus; radius: parent.radius }
                }
            }
            Text {
                text: Theme.clock(player.seconds) + "  /  " + Theme.clock(player.length)
                color: Theme.inkBody
                font.family: Theme.mono
                font.pixelSize: Theme.sizeBody
                Layout.leftMargin: 10
            }
            Item { Layout.fillWidth: true }
            Button {
                id: speed
                flat: true
                focusPolicy: Qt.TabFocus
                text: media.playbackRate + "×"
                Accessible.name: qsTr("Speed %1").arg(text)
                ToolTip.visible: (hovered || visualFocus) && ToolTip.text.length > 0
                ToolTip.text: qsTr("Speed ([ and ])")
                contentItem: Text { text: speed.text; color: media.playbackRate === 1 ? Theme.inkBody : Theme.accentBright; font.family: Theme.mono; font.pixelSize: Theme.sizeBody; horizontalAlignment: Text.AlignHCenter; verticalAlignment: Text.AlignVCenter }
                background: Rectangle {
                    radius: Theme.radiusControl
                    color: speed.hovered || speed.visualFocus ? Qt.alpha(Theme.ink, 0.12) : "transparent"
                    FocusRing { visible: speed.visualFocus }
                }
                Keys.onReturnPressed: clicked()
                Keys.onEnterPressed: clicked()
                onClicked: { const i = player.rates.indexOf(media.playbackRate); media.playbackRate = player.rates[(i + 1) % player.rates.length] }
            }
            IconButton {
                iconName: "subtitles"
                visible: media.subtitleTracks.length > 0
                active: media.activeSubtitleTrack >= 0
                ToolTip.text: media.activeSubtitleTrack >= 0 ? qsTr("Subtitles: %1 (C)").arg(player.trackName(media.subtitleTracks[media.activeSubtitleTrack], media.activeSubtitleTrack)) : qsTr("Subtitles off (C)")
                onClicked: player.nextSubtitle()
            }
            IconButton {
                iconName: "tracks"
                visible: media.audioTracks.length > 1
                ToolTip.text: qsTr("Audio: %1 (A)").arg(media.audioTracks.length ? player.trackName(media.audioTracks[Math.max(0, media.activeAudioTrack)], Math.max(0, media.activeAudioTrack)) : "")
                onClicked: player.nextAudio()
            }
            IconButton { iconName: "full"; ToolTip.text: qsTr("Full screen (F)"); onClicked: player.fullScreenToggled() }
        }
    }

    Keys.onPressed: (event) => {
        switch (event.key) {
        case Qt.Key_Space: case Qt.Key_K: player.playing ? media.pause() : media.play(); break
        case Qt.Key_Left: player.seekBy(-5); break
        case Qt.Key_Right: player.seekBy(5); break
        case Qt.Key_J: player.seekBy(-10); break
        case Qt.Key_L: player.seekBy(10); break
        case Qt.Key_Up: settings.volume = Math.min(1, settings.volume + 0.05); settings.muted = false; break
        case Qt.Key_Down: settings.volume = Math.max(0, settings.volume - 0.05); break
        case Qt.Key_M: settings.muted = !settings.muted; break
        case Qt.Key_F: player.fullScreenToggled(); break
        case Qt.Key_BracketLeft: { const i = player.rates.indexOf(media.playbackRate); media.playbackRate = player.rates[Math.max(0, i - 1)]; break }
        case Qt.Key_BracketRight: { const i = player.rates.indexOf(media.playbackRate); media.playbackRate = player.rates[Math.min(player.rates.length - 1, i + 1)]; break }
        case Qt.Key_C: player.nextSubtitle(); break
        case Qt.Key_A: player.nextAudio(); break
        default: return
        }
        player.wake()
        event.accepted = true
    }
}
