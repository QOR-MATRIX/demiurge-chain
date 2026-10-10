// The QQ Player's window: one game, full bleed, with a line of help that fades once play is under way. F11 fills the
// screen; Ctrl+R plays the scene again from the start.

import QtQuick
import QtQuick3D
import QQ

Window {
    id: window

    /// The scene file to play.
    required property url scene

    width: 1280
    height: 720
    visible: true
    color: "#07080c"
    title: (game.scene ? game.scene.name : "QQ") + " — QQ Player"

    /// What a capture or a measurement waits for: the game's first frame proper.
    readonly property bool settled: game.settled
    readonly property string problem: game.problem

    Game {
        id: game
        objectName: "game"
        anchors.fill: parent
        file: window.scene
    }

    function release() { game.release() }

    Shortcut { sequence: "F11"; onActivated: window.visibility = window.visibility === Window.FullScreen ? Window.Windowed : Window.FullScreen }
    Shortcut { sequence: "Ctrl+R"; onActivated: game.restart() }

    Text {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: 22
        color: "#e6e8ee"
        font.pixelSize: 13
        font.letterSpacing: 0.4
        text: game.follow ? qsTr("WASD or the left stick to move  ·  Space or A to jump  ·  Shift to run  ·  drag or the right stick to look")
                          : qsTr("Drag to turn  ·  scroll to come closer")
        opacity: game.settled && hint.running ? 0.7 : 0
        Behavior on opacity { NumberAnimation { duration: 900; easing.type: Easing.OutCubic } }
        Timer { id: hint; interval: 7000; running: game.settled }
    }

    Rectangle {
        visible: game.problem !== ""
        anchors.centerIn: parent
        width: Math.min(parent.width - 48, problemText.implicitWidth + 40)
        height: problemText.implicitHeight + 28
        radius: 10
        color: Qt.rgba(0.05, 0.06, 0.08, 0.9)
        border.color: "#d57889"
        Text {
            id: problemText
            anchors.centerIn: parent
            width: Math.min(implicitWidth, window.width - 88)
            wrapMode: Text.Wrap
            color: "#f2c4cc"
            font.pixelSize: 14
            text: qsTr("This game could not start: %1").arg(game.problem)
        }
    }
}
