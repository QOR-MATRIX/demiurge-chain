pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import ARQADE

// The arenas' rankings, from ARQADE's server: all games, or one arena. The person's own row is marked, and shown under
// the list when it is not in it.
Glass {
    id: rankings
    required property var server
    property bool launcherSignedIn: false

    property string game: "all"
    readonly property var rows: (server.standings[game] ?? [])
    readonly property var leaders: rows.slice(0, 10)
    readonly property var mine: rows.find(r => r.me) ?? null
    readonly property bool mineShown: leaders.some(r => r.me)
    readonly property int phase: server.state

    implicitWidth: 420
    implicitHeight: body.implicitHeight + 48

    ColumnLayout {
        id: body
        anchors.fill: parent
        anchors.margins: 24
        spacing: 14

        RowLayout {
            Layout.fillWidth: true
            Text {
                text: qsTr("RANKINGS")
                font.family: Theme.ui
                font.pixelSize: 12
                font.weight: Font.DemiBold
                font.letterSpacing: 3
                color: Theme.gilt
                Layout.fillWidth: true
            }
            Row {
                visible: rankings.phase === ArqadeServer.SignedIn
                spacing: 6
                Rectangle {
                    anchors.verticalCenter: parent.verticalCenter
                    width: 7
                    height: 7
                    radius: 3.5
                    color: "#5fd38d"
                }
                Text {
                    text: qsTr("%n online", "", rankings.server.online)
                    font.family: Theme.ui
                    font.pixelSize: 13
                    color: Theme.inkMuted
                }
            }
        }

        // All, or one arena.
        Row {
            spacing: 6
            Layout.fillWidth: true
            Repeater {
                model: [
                    { id: "all", label: qsTr("All") },
                    { id: "flux", label: qsTr("Flux Four") },
                    { id: "reversi", label: qsTr("Rift Reversi") },
                ]
                AbstractButton {
                    id: tab
                    required property var modelData
                    readonly property bool chosen: rankings.game === modelData.id
                    hoverEnabled: true
                    focusPolicy: Qt.StrongFocus
                    implicitHeight: 30
                    implicitWidth: label.implicitWidth + 24
                    onClicked: rankings.game = modelData.id
                    Accessible.role: Accessible.PageTab
                    Accessible.name: modelData.label
                    Accessible.checked: chosen
                    background: Rectangle {
                        radius: 15
                        color: tab.chosen ? Qt.rgba(Theme.gilt.r, Theme.gilt.g, Theme.gilt.b, 0.16) : "transparent"
                        border.width: 1
                        border.color: tab.chosen || tab.visualFocus ? Theme.gilt : (tab.hovered ? Theme.lineStrong : Theme.line)
                    }
                    contentItem: Text {
                        id: label
                        text: tab.modelData.label
                        font.family: Theme.ui
                        font.pixelSize: 13
                        font.weight: Font.DemiBold
                        font.letterSpacing: 0.8
                        color: tab.chosen ? Theme.giltBright : Theme.inkBody
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                    }
                }
            }
        }

        // The list, signed in.
        ColumnLayout {
            visible: rankings.phase === ArqadeServer.SignedIn && rankings.rows.length > 0
            spacing: 2
            Layout.fillWidth: true
            Repeater {
                model: rankings.leaders
                Ranked { required property var modelData; row: modelData }
            }
            Text {
                visible: rankings.mine !== null && !rankings.mineShown
                text: "···"
                color: Theme.inkFaint
                font.pixelSize: 14
                Layout.alignment: Qt.AlignHCenter
            }
            Ranked {
                visible: rankings.mine !== null && !rankings.mineShown
                row: rankings.mine ?? ({})
            }
        }

        Text {
            visible: rankings.phase === ArqadeServer.SignedIn && rankings.rows.length === 0
            text: qsTr("No ranked matches yet. The first win ranks.")
            font.family: Theme.ui
            font.pixelSize: 15
            color: Theme.inkMuted
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        // Signing in: a light running along a rule.
        ColumnLayout {
            visible: rankings.phase === ArqadeServer.SigningIn || rankings.phase === ArqadeServer.Idle
            spacing: 10
            Layout.fillWidth: true
            Text {
                text: qsTr("Signing in to ARQADE with your QOR ID…")
                font.family: Theme.ui
                font.pixelSize: 15
                color: Theme.inkMuted
            }
            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 2
                radius: 1
                color: Theme.line
                clip: true
                Rectangle {
                    id: light
                    width: parent.width * 0.3
                    height: parent.height
                    radius: 1
                    color: Theme.gilt
                    NumberAnimation on x {
                        running: !Theme.reducedMotion && rankings.visible
                        from: -light.width; to: rankings.width
                        duration: 1400
                        loops: Animation.Infinite
                        easing.type: Easing.InOutSine
                    }
                }
            }
        }

        // Not signed in, or not reached: why, and a way on when there is one.
        ColumnLayout {
            visible: rankings.phase === ArqadeServer.SignedOut || rankings.phase === ArqadeServer.Unreachable
            spacing: 12
            Layout.fillWidth: true
            Text {
                text: rankings.server.message
                font.family: Theme.ui
                font.pixelSize: 15
                color: Theme.inkBody
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
            ArqButton {
                visible: rankings.launcherSignedIn
                text: qsTr("Try again")
                onClicked: rankings.server.signIn()
            }
        }
    }

    // One ranked player.
    component Ranked: Rectangle {
        id: ranked
        property var row: ({})
        readonly property bool me: row.me === true
        Layout.fillWidth: true
        implicitHeight: 38
        radius: 8
        color: me ? Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.1) : "transparent"
        border.width: me ? 1 : 0
        border.color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.4)

        Accessible.role: Accessible.ListItem
        Accessible.name: qsTr("%1. %2, %3 points, %4 wins, %5 losses, %6 draws")
            .arg(row.rank ?? "").arg(row.alias ?? "").arg(row.points ?? 0).arg(row.wins ?? 0).arg(row.losses ?? 0).arg(row.draws ?? 0)

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 12
            spacing: 12
            Text {
                text: ranked.row.rank ?? ""
                font.family: Theme.display
                font.pixelSize: 17
                font.weight: Font.Bold
                color: (ranked.row.rank ?? 99) <= 3 ? Theme.giltBright : Theme.inkMuted
                horizontalAlignment: Text.AlignRight
                Layout.preferredWidth: 26
            }
            Text {
                text: ranked.row.alias ?? ""
                font.family: Theme.ui
                font.pixelSize: 16
                font.weight: Font.Medium
                color: Theme.ink
                elide: Text.ElideRight
                Layout.fillWidth: true
            }
            Text {
                visible: ranked.me
                text: qsTr("YOU")
                font.family: Theme.ui
                font.pixelSize: 11
                font.weight: Font.DemiBold
                font.letterSpacing: 1.6
                color: Theme.accentBright
            }
            Text {
                text: "%1–%2–%3".arg(ranked.row.wins ?? 0).arg(ranked.row.losses ?? 0).arg(ranked.row.draws ?? 0)
                font.family: Theme.ui
                font.pixelSize: 13
                color: Theme.inkMuted
            }
            Text {
                text: ranked.row.points ?? 0
                font.family: Theme.display
                font.pixelSize: 16
                font.weight: Font.DemiBold
                color: Theme.giltBright
                horizontalAlignment: Text.AlignRight
                Layout.preferredWidth: 56
            }
        }
    }
}
