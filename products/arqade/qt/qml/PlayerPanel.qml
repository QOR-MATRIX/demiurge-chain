import QtQuick
import QtQuick.Layouts
import ARQADE

// Who is playing, as the launcher says: their QOR ID, their level, and their wallet's balance in Sparks (ADR-101), with
// what rewards are on the devnet said plainly (ADR-102 decision 7).
Glass {
    id: panel
    property var player: ({ signed_in: false })
    property bool launched: false

    readonly property bool signedIn: player.signed_in === true
    // "37,000,000 Sparks" as the number and its unit; an amount finer than a Spark arrives as CGT and is shown whole.
    readonly property string sparks: player.sparks ?? ""
    readonly property bool inSparks: /\sSparks?$/.test(sparks)
    readonly property string amount: inSparks ? sparks.replace(/\sSparks?$/, "") : sparks
    readonly property string unit: inSparks ? sparks.slice(amount.length + 1) : ""

    implicitWidth: 360
    implicitHeight: body.implicitHeight + 48

    ColumnLayout {
        id: body
        anchors.fill: parent
        anchors.margins: 24
        spacing: 16

        RowLayout {
            visible: panel.signedIn
            spacing: 14
            Layout.fillWidth: true

            Rectangle {
                Layout.preferredWidth: 48
                Layout.preferredHeight: 48
                radius: width / 2
                color: Theme.raised
                border.width: 1
                border.color: Theme.gilt
                Text {
                    anchors.centerIn: parent
                    text: (panel.player.name ?? "?").charAt(0).toUpperCase()
                    font.family: Theme.display
                    font.pixelSize: Theme.sizeDisplay
                    font.weight: Font.Bold
                    color: Theme.giltBright
                }
            }
            ColumnLayout {
                spacing: 2
                Layout.fillWidth: true
                Text {
                    text: panel.player.name ?? ""
                    font.family: Theme.display
                    font.pixelSize: Theme.sizeHeading
                    font.weight: Font.DemiBold
                    color: Theme.ink
                    elide: Text.ElideRight
                    Layout.fillWidth: true
                }
                Text {
                    visible: (panel.player.level ?? 0) > 0
                    text: qsTr("Level %1").arg(panel.player.level ?? 0)
                    font.family: Theme.ui
                    font.pixelSize: Theme.sizeUi
                    font.weight: Font.DemiBold
                    font.letterSpacing: Theme.sizeUi * Theme.trackingLabel
                    color: Theme.gilt
                }
            }
        }

        Rectangle {
            visible: panel.signedIn
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: Theme.line
        }

        ColumnLayout {
            visible: panel.signedIn && panel.sparks.length > 0
            spacing: 4
            Layout.fillWidth: true

            Text {
                text: qsTr("YOUR SPARKS")
                font.family: Theme.ui
                font.pixelSize: Theme.sizeUi
                font.weight: Font.DemiBold
                font.letterSpacing: Theme.sizeUi * Theme.trackingEyebrow
                color: Theme.gilt
            }
            RowLayout {
                spacing: 10
                Layout.fillWidth: true
                Text {
                    text: panel.amount
                    font.family: Theme.display
                    font.pixelSize: Theme.sizeFigure
                    fontSizeMode: Text.HorizontalFit
                    minimumPixelSize: Theme.sizeDisplay
                    font.weight: Font.Bold
                    color: Theme.giltBright
                    Layout.maximumWidth: panel.width - 64
                    Accessible.name: panel.sparks
                }
                Text {
                    visible: panel.unit.length > 0
                    text: panel.unit
                    font.family: Theme.ui
                    font.pixelSize: Theme.sizeTitle
                    font.weight: Font.Medium
                    color: Theme.inkMuted
                    Layout.alignment: Qt.AlignBaseline
                }
            }
            Text {
                visible: (panel.player.cgt ?? "").length > 0
                text: qsTr("%1 CGT in your wallet. A Spark is a thousandth of a CGT.").arg(panel.player.cgt ?? "")
                font.family: Theme.body
                font.pixelSize: Theme.sizeBody
                color: Theme.inkMuted
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        Text {
            visible: !panel.signedIn
            text: panel.launched
                ? qsTr("Sign in to the launcher with your QOR ID to play signed in and see your Sparks here.")
                : qsTr("Open ARQADE from the QOR Launcher to play signed in and see your Sparks here.")
            font.family: Theme.body
            font.pixelSize: Theme.sizeTitle
            color: Theme.inkBody
            wrapMode: Text.WordWrap
            Layout.fillWidth: true
        }

        // What rewards are, today: said wherever they are shown (ADR-102 decision 7).
        Rectangle {
            Layout.fillWidth: true
            implicitHeight: devnet.implicitHeight + 24
            radius: Theme.radiusControl
            color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.07)
            border.width: 1
            border.color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.28)
            Text {
                id: devnet
                anchors.fill: parent
                anchors.margins: 12
                text: qsTr("Rewards for play arrive with the rebuilt games: Sparks from ARQADE's reward pool, for runs its server confirms. On the devnet they are test CGT, with no value.")
                font.family: Theme.body
                font.pixelSize: Theme.sizeBody
                color: Theme.inkBody
                wrapMode: Text.WordWrap
                lineHeight: 1.2
            }
        }
    }
}
