import QtQuick
import QtQuick.Controls.Basic
import ARQADE

// ARQADE's button: a gilt outline, or an ember fill for the one thing a screen asks first.
Button {
    id: button
    property bool primary: false

    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    padding: 0
    leftPadding: 20
    rightPadding: 20
    implicitHeight: 40
    font.family: Theme.ui
    font.pixelSize: 14
    font.weight: Font.DemiBold
    font.letterSpacing: 1.6

    contentItem: Text {
        text: button.text.toUpperCase()
        font: button.font
        color: button.primary ? "#120a04" : (button.hovered ? Theme.giltBright : Theme.ink)
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        Behavior on color { ColorAnimation { duration: 200 } }
    }

    background: Rectangle {
        radius: height / 2
        color: button.primary ? (button.hovered ? Theme.accentBright : Theme.accent) : (button.hovered ? Qt.rgba(Theme.gilt.r, Theme.gilt.g, Theme.gilt.b, 0.1) : "transparent")
        border.width: button.primary ? 0 : 1
        border.color: button.hovered || button.visualFocus ? Theme.giltBright : Theme.lineStrong
        opacity: button.enabled ? (button.down ? 0.85 : 1) : 0.45
        Behavior on color { ColorAnimation { duration: 200 } }

        // The focus ring keyboard users follow.
        Rectangle {
            anchors.fill: parent
            anchors.margins: -4
            radius: height / 2
            color: "transparent"
            border.width: 2
            border.color: Theme.giltBright
            visible: button.visualFocus
        }
    }
}
