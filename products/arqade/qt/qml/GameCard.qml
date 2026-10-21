pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Effects
import ARQADE

// One game: its emblem in 3D, its name, what kind of game it is, and where its rebuild stands. Today's version plays in
// the launcher's ARQADE window, which a press opens.
AbstractButton {
    id: card

    required property string kind
    required property string number
    required property string name
    required property string tag
    required property string line
    property string status: qsTr("Being rebuilt in 3D")

    implicitWidth: 280
    implicitHeight: 360
    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    readonly property bool lit: hovered || visualFocus

    Accessible.role: Accessible.Button
    Accessible.name: name
    Accessible.description: line + ". " + status + ". " + qsTr("Opens today's version in the launcher's ARQADE window.")

    transform: Translate {
        y: card.lit && !Theme.reducedMotion ? -6 : 0
        Behavior on y { NumberAnimation { duration: Theme.enter; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }
    }
    scale: pressed ? 0.985 : 1
    Behavior on scale { NumberAnimation { duration: Theme.quick; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }

    background: Item {
        Rectangle {
            id: plate
            anchors.fill: parent
            radius: Theme.radiusPanel
            gradient: Gradient {
                GradientStop { position: 0; color: Qt.rgba(Theme.surface.r, Theme.surface.g, Theme.surface.b, 0.86) }
                GradientStop { position: 1; color: Qt.rgba(Theme.void_.r, Theme.void_.g, Theme.void_.b, 0.94) }
            }
            border.width: 1
            border.color: card.lit ? Theme.lineStrong : Theme.line
            Behavior on border.color { ColorAnimation { duration: Theme.standard } }

            layer.enabled: true
            layer.effect: MultiEffect {
                shadowEnabled: true
                shadowColor: Theme.accent
                shadowBlur: 1.0
                shadowOpacity: card.lit ? 0.28 : 0.0
                shadowVerticalOffset: 10
                Behavior on shadowOpacity { NumberAnimation { duration: Theme.enter; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }
            }
        }
        // An ember glow rising from the card's foot when it is hovered.
        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.margins: 1
            height: parent.height * 0.55
            radius: Theme.radiusPanel - 1
            opacity: card.lit ? 1 : 0
            Behavior on opacity { NumberAnimation { duration: Theme.enter; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }
            gradient: Gradient {
                GradientStop { position: 0; color: "transparent" }
                GradientStop { position: 1; color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, 0.14) }
            }
        }
        // A gilt hairline across the top, brighter at its middle.
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.top: parent.top
            width: parent.width * 0.7
            height: 1
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0; color: "transparent" }
                GradientStop { position: 0.5; color: card.lit ? Theme.giltBright : Theme.gilt }
                GradientStop { position: 1; color: "transparent" }
            }
        }
        // The focus ring, for the keyboard only.
        Rectangle {
            anchors.fill: plate
            anchors.margins: -(Theme.focusWidth + 2)
            radius: plate.radius + Theme.focusWidth + 2
            color: "transparent"
            visible: card.visualFocus
            border.width: Theme.focusWidth
            border.color: Theme.focus
        }
    }

    contentItem: Item {
        Emblem {
            id: emblem
            kind: card.kind
            lively: card.lit
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            height: parent.height * 0.5
        }

        Text {
            anchors.left: parent.left
            anchors.top: parent.top
            anchors.margins: 16
            text: card.number
            font.family: Theme.display
            font.pixelSize: Theme.sizeBody
            font.weight: Font.DemiBold
            color: Theme.gilt
            opacity: 0.8
        }

        Column {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: emblem.bottom
            anchors.leftMargin: 20
            anchors.rightMargin: 20
            spacing: 6

            Text {
                text: card.tag.toUpperCase()
                font.family: Theme.ui
                font.pixelSize: Theme.sizeCaption
                font.weight: Font.DemiBold
                font.letterSpacing: Theme.sizeCaption * Theme.trackingEyebrow
                color: Theme.gilt
            }
            Text {
                width: parent.width
                text: card.name
                font.family: Theme.display
                font.pixelSize: 21
                font.weight: Font.DemiBold
                font.letterSpacing: 1
                color: Theme.ink
                elide: Text.ElideRight
            }
            Text {
                width: parent.width
                text: card.line
                font.family: Theme.body
                font.pixelSize: Theme.sizeBody
                color: Theme.inkMuted
                wrapMode: Text.WordWrap
                maximumLineCount: 2
                elide: Text.ElideRight
            }
        }

        Row {
            anchors.left: parent.left
            anchors.bottom: parent.bottom
            anchors.margins: 20
            spacing: 8
            Rectangle {
                anchors.verticalCenter: parent.verticalCenter
                width: 6
                height: 6
                radius: width / 2
                color: Theme.accent
                SequentialAnimation on opacity {
                    running: !Theme.reducedMotion && card.visible
                    loops: Animation.Infinite
                    NumberAnimation { to: 0.3; duration: 1100; easing.type: Easing.InOutSine }
                    NumberAnimation { to: 1; duration: 1100; easing.type: Easing.InOutSine }
                }
            }
            Text {
                text: card.lit ? qsTr("Play today's version") : card.status
                font.family: Theme.ui
                font.pixelSize: Theme.sizeUi
                font.weight: Font.Medium
                font.letterSpacing: Theme.sizeUi * Theme.trackingLabel
                color: card.lit ? Theme.accentBright : Theme.inkMuted
                Behavior on color { ColorAnimation { duration: Theme.standard } }
            }
        }
    }
}
