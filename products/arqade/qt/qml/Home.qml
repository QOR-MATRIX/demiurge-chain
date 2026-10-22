pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import ARQADE

// ARQADE's home (DIRECTION P7.19): the portal behind everything, the name and the player's Sparks over it, then the
// games, the arenas and their rankings. The games are being rebuilt in 3D (P7.20); until each arrives, its card opens
// today's version in the launcher's ARQADE window.
Item {
    id: home
    required property var link
    required property var server
    required property var player
    required property bool launched

    readonly property bool wide: width >= 1180
    readonly property real gutter: Math.max(28, Math.min(72, width * 0.05))
    property string webError: ""

    Portal {
        anchors.fill: parent
        pointer: Qt.point(hover.point.position.x / Math.max(1, home.width) * 2 - 1,
                          hover.point.position.y / Math.max(1, home.height) * 2 - 1)
        scroll: page.contentY
        coreAt: home.wide ? 0.6 : 0.72
        coreDown: home.wide ? 0.36 : 0.3
    }

    // The page darkens below the hero, so what is read there reads against the void, not the scene.
    Rectangle {
        anchors.fill: parent
        gradient: Gradient {
            GradientStop { position: 0.0; color: "transparent" }
            GradientStop { position: Math.min(0.9, 520 / Math.max(1, home.height)); color: Qt.rgba(Theme.void_.r, Theme.void_.g, Theme.void_.b, 0.35) }
            GradientStop { position: 1.0; color: Qt.rgba(Theme.void_.r, Theme.void_.g, Theme.void_.b, 0.82) }
        }
    }

    // And the hero's words read against a shade the portal fades into from the left.
    Rectangle {
        anchors.left: parent.left
        anchors.top: parent.top
        width: parent.width * 0.62
        height: Math.max(520, parent.height * 0.7) - page.contentY
        visible: height > 0
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: Qt.rgba(Theme.void_.r, Theme.void_.g, Theme.void_.b, 0.72) }
            GradientStop { position: 0.55; color: Qt.rgba(Theme.void_.r, Theme.void_.g, Theme.void_.b, 0.4) }
            GradientStop { position: 1.0; color: "transparent" }
        }
    }

    HoverHandler { id: hover }

    Connections {
        target: home.link
        function onWebFailed(message) { home.webError = message }
    }

    Component.onCompleted: {
        // Started by hand there is no launcher to sign in with: the rankings say so rather than wait.
        if (!home.launched)
            home.server.signIn()
    }

    Flickable {
        id: page
        objectName: "page"
        anchors.fill: parent
        contentWidth: width
        contentHeight: column.implicitHeight + 96
        boundsBehavior: Flickable.StopAtBounds
        ScrollBar.vertical: ScrollBar { policy: ScrollBar.AsNeeded }

        ColumnLayout {
            id: column
            x: home.gutter
            y: 56
            width: page.width - home.gutter * 2
            spacing: 56

            // The hero: the name, what ARQADE is, and who is playing.
            // Wide, the name sits left, the portal's core right of it, and the player at the far right below the core.
            GridLayout {
                columns: home.wide ? 3 : 1
                columnSpacing: 32
                rowSpacing: 32
                Layout.fillWidth: true
                Layout.minimumHeight: home.wide ? Math.max(420, home.height * 0.56) : 0

                ColumnLayout {
                    spacing: 18
                    Layout.preferredWidth: home.wide ? Math.min(720, column.width * 0.5) : column.width
                    Layout.alignment: Qt.AlignTop

                    Text {
                        text: qsTr("THE QOR ARCADE")
                        font.family: Theme.ui
                        font.pixelSize: Theme.sizeBody
                        font.weight: Font.DemiBold
                        font.letterSpacing: Theme.sizeBody * Theme.trackingEyebrow
                        color: Theme.gilt
                    }
                    Wordmark {
                        size: Math.max(56, Math.min(112, home.width * 0.075))
                    }
                    Text {
                        text: qsTr("Play the games of the Demiurge devnet, climb the arenas' rankings, and be rewarded in Sparks for runs ARQADE confirms.")
                        font.family: Theme.body
                        font.pixelSize: Theme.sizeHeading
                        color: Theme.inkBody
                        wrapMode: Text.WordWrap
                        lineHeight: 1.25
                        Layout.fillWidth: true
                        Layout.maximumWidth: 560
                    }
                    RowLayout {
                        spacing: 12
                        ArqButton {
                            primary: true
                            text: qsTr("Play today's versions")
                            onClicked: { home.webError = ""; home.link.openWeb() }
                        }
                    }
                    Text {
                        visible: home.webError.length > 0
                        text: home.webError
                        font.family: Theme.body
                        font.pixelSize: Theme.sizeBody
                        color: Theme.bad
                        wrapMode: Text.WordWrap
                        Layout.fillWidth: true
                    }
                }

                Item {
                    visible: home.wide
                    Layout.fillWidth: true
                }

                PlayerPanel {
                    player: home.player
                    launched: home.launched
                    Layout.alignment: home.wide ? Qt.AlignBottom : Qt.AlignLeft
                    Layout.preferredWidth: 400
                    Layout.maximumWidth: column.width
                }
            }

            // The four games played alone.
            ColumnLayout {
                spacing: 24
                Layout.fillWidth: true

                Heading {
                    eyebrow: qsTr("Games")
                    title: qsTr("Four games, being rebuilt in 3D")
                    line: qsTr("Each is being rebuilt with physically based light, particles and sound, and with rules that replay exactly, so a run can be confirmed and rewarded. Until a game's rebuild arrives, its card opens today's version.")
                    Layout.fillWidth: true
                }
                GridLayout {
                    columns: Math.max(1, Math.min(4, Math.floor((column.width + 20) / 280)))
                    columnSpacing: 20
                    rowSpacing: 20
                    Layout.fillWidth: true
                    Repeater {
                        model: [
                            { kind: "void", number: "01", name: "Void Runner", tag: qsTr("Reflex / survival"), line: qsTr("Outrun the collapse.") },
                            { kind: "synapse", number: "02", name: "Synapse", tag: qsTr("Memory / pattern"), line: qsTr("Trust the signal. Question your mind.") },
                            { kind: "orbital", number: "03", name: "Orbital", tag: qsTr("Precision / timing"), line: qsTr("Find order in the chaos.") },
                            { kind: "rift", number: "04", name: "Rift Survivor", tag: qsTr("Survival / evolution"), line: qsTr("One signal. Ninety seconds. Evolve.") },
                        ]
                        GameCard {
                            required property var modelData
                            kind: modelData.kind
                            number: modelData.number
                            name: modelData.name
                            tag: modelData.tag
                            line: modelData.line
                            Layout.fillWidth: true
                            Layout.preferredHeight: 360
                            onClicked: { home.webError = ""; home.link.openWeb() }
                        }
                    }
                }
            }

            // The arenas, played against another person, and their rankings.
            ColumnLayout {
                spacing: 24
                Layout.fillWidth: true

                Heading {
                    eyebrow: qsTr("Arenas")
                    title: qsTr("Two arenas, one ranking")
                    line: qsTr("Played against another person on ARQADE's server. Wins, losses and draws rank you in each arena and across both.")
                    Layout.fillWidth: true
                }
                GridLayout {
                    columns: home.wide ? 3 : (column.width >= 640 ? 2 : 1)
                    columnSpacing: 20
                    rowSpacing: 20
                    Layout.fillWidth: true

                    GameCard {
                        kind: "flux"
                        number: "I"
                        name: "Flux Four"
                        tag: qsTr("Arena / strategy")
                        line: qsTr("Gravity is a weapon. Think four moves ahead.")
                        Layout.fillWidth: true
                        Layout.preferredHeight: 360
                        Layout.alignment: Qt.AlignTop
                        onClicked: { home.webError = ""; home.link.openWeb() }
                    }
                    GameCard {
                        kind: "reversi"
                        number: "II"
                        name: "Rift Reversi"
                        tag: qsTr("Arena / territory")
                        line: qsTr("Turn their territory into your advantage.")
                        Layout.fillWidth: true
                        Layout.preferredHeight: 360
                        Layout.alignment: Qt.AlignTop
                        onClicked: { home.webError = ""; home.link.openWeb() }
                    }
                    Rankings {
                        server: home.server
                        launcherSignedIn: home.player.signed_in === true
                        Layout.fillWidth: true
                        Layout.columnSpan: home.wide ? 1 : (column.width >= 640 ? 2 : 1)
                        Layout.minimumWidth: 300
                        Layout.preferredWidth: 420
                        Layout.alignment: Qt.AlignTop
                    }
                }
            }

            Text {
                text: qsTr("ARQADE runs on the Demiurge devnet. Sparks are thousandths of a CGT; on the devnet they are test CGT, with no value.")
                font.family: Theme.body
                font.pixelSize: Theme.sizeUi
                color: Theme.inkMuted
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }
    }
}
