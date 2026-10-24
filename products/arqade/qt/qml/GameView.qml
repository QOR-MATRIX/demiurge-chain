pragma ComponentBehavior: Bound
import QtQuick
import QQ
import ARQADE

// A game rebuilt in QQ (DIRECTION P7.20; ADR-102 decision 2), played in ARQADE's window by QQ's runtime: a bar with the
// way back and the game's name, and the game beneath it. Esc, or Back, puts the game away and returns to the home.
//
// The games here are ARQADE's own, written and reviewed in this repository and carried in the executable
// (qrc:/games/<kind>/), so they play in ARQADE's engine as the QQ Player plays a person's own project. QQ's lockdown
// (ADR-087 decision 2) is the second wall around a stranger's game, and a stranger's game never plays here: it plays in
// the browser (ADR-087 decision 1).
//
// Each play is kept as a run (QQ's RunRecord): the seed it started from, what every tick took, and what it came to. A
// game's rules replay that run to the same outcome, in QQ and in Node, which is what ARQADE's service will do before it
// confirms a run (P7.22).
Item {
    id: view

    /// Which game: its card's kind.
    required property string kind

    /// The games played here, by kind: their names and scene files.
    readonly property var games: ({
        orbital: { name: "Orbital", file: "qrc:/games/orbital/scenes/orbital.qml" },
        "void": { name: "Void Runner", file: "qrc:/games/void-runner/scenes/void-runner.qml" },
        synapse: { name: "Synapse", file: "qrc:/games/synapse/scenes/synapse.qml" },
    })
    readonly property var game: games[kind] ?? null

    /// The run so far.
    readonly property alias record: run
    /// The scene playing, or null.
    readonly property alias playing: play.scene

    /// The game has been put away, and the home should show.
    signal closed()

    /// Put the game away in a safe order, its bodies before their world.
    function release() {
        play.release()
    }

    function leave() {
        release()
        closed()
    }

    RunRecord {
        id: run
    }

    Rectangle {
        id: bar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 56
        color: Theme.void_

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: Theme.line
        }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 18

            ArqButton {
                objectName: "back"
                text: qsTr("Back")
                implicitHeight: 34
                // Never the keyboard's: Space and Enter are the game's, and Esc is the way back.
                focusPolicy: Qt.NoFocus
                onClicked: view.leave()
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: view.game ? view.game.name : ""
                font.family: Theme.display
                font.pixelSize: Theme.sizeHeading
                font.weight: Font.DemiBold
                font.letterSpacing: Theme.sizeHeading * Theme.trackingHeading
                color: Theme.ink
            }
        }

        Text {
            anchors.right: parent.right
            anchors.rightMargin: 20
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Esc returns to ARQADE  ·  F11 fills the screen")
            font.family: Theme.ui
            font.pixelSize: Theme.sizeUi
            font.letterSpacing: Theme.sizeUi * Theme.trackingLabel
            color: Theme.inkMuted
        }
    }

    Game {
        id: play
        objectName: "game"
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: bar.bottom
        anchors.bottom: parent.bottom
        recorder: run
        file: view.game ? view.game.file : ""
    }

    Text {
        visible: play.problem !== ""
        anchors.centerIn: play
        width: Math.min(implicitWidth, play.width - 48)
        wrapMode: Text.WordWrap
        text: qsTr("This game could not start: %1").arg(play.problem)
        font.family: Theme.body
        font.pixelSize: Theme.sizeBody
        color: Theme.bad
    }

    Shortcut {
        sequence: "Esc"
        // Full screen, Esc first puts the window back (Main).
        enabled: !Dock.fullScreen
        onActivated: view.leave()
    }
}
