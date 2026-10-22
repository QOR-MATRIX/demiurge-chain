import QtQuick
import QtQuick.Controls.Basic
import ARQADE

// ARQADE's button, in the one look (ADR-108): Barlow Semi Condensed in tracked capitals, 8 pixel corners. The primary,
// the one thing a screen asks first, is ember, lit from above, with a glow and dark ink on it (7:1; white would read
// 2.9:1); any other is a gilt hairline. It lifts a pixel under the pointer and gives under a press, unless the person
// asked for less motion; the focus ring is the one every surface draws, for the keyboard only.
Button {
    id: button
    property bool primary: false

    readonly property bool lit: button.enabled && (button.hovered || button.visualFocus)

    hoverEnabled: true
    focusPolicy: Qt.StrongFocus
    padding: 0
    leftPadding: 22
    rightPadding: 22
    implicitHeight: 42
    opacity: enabled ? 1 : 0.45
    font.family: Theme.ui
    font.pixelSize: Theme.sizeBody
    font.weight: Font.DemiBold
    font.letterSpacing: Theme.sizeBody * Theme.trackingLabel
    font.capitalization: Font.AllUppercase

    contentItem: Text {
        text: button.text
        font: button.font
        color: button.primary ? Theme.inkOnAccent : (button.lit ? Theme.giltBright : Theme.ink)
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        Behavior on color { ColorAnimation { duration: Theme.quick } }
    }

    background: Item {
        implicitWidth: 96
        implicitHeight: 42
        transform: Translate { y: button.lit && !button.down && Theme.quick > 0 ? -1 : 0 }
        scale: button.down && Theme.quick > 0 ? 0.97 : 1
        Behavior on scale { NumberAnimation { duration: Theme.quick; easing.type: Easing.BezierSpline; easing.bezierCurve: Theme.ease } }

        // The primary's glow: its own ember, soft, a little stronger under the pointer.
        Rectangle {
            visible: button.primary
            anchors.fill: face
            anchors.margins: -4
            radius: face.radius + 4
            color: "transparent"
            border.width: 4
            border.color: Qt.rgba(Theme.accent.r, Theme.accent.g, Theme.accent.b, button.lit ? 0.3 : 0.15)
            Behavior on border.color { ColorAnimation { duration: Theme.standard } }
        }

        Rectangle {
            id: face
            anchors.fill: parent
            radius: Theme.radiusControl
            color: button.primary ? (button.lit ? Theme.accentBright : Theme.accent)
                                  : (button.lit ? Qt.rgba(Theme.gilt.r, Theme.gilt.g, Theme.gilt.b, 0.1) : "transparent")
            border.width: 1
            border.color: button.primary ? Theme.accentBright : (button.lit ? Theme.giltBright : Theme.lineStrong)
            Behavior on color { ColorAnimation { duration: Theme.quick } }
            Behavior on border.color { ColorAnimation { duration: Theme.quick } }

            // Lit from above: a band of warm light across the top of the primary.
            Rectangle {
                visible: button.primary
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: 1 }
                height: parent.height / 2
                radius: face.radius - 1
                gradient: Gradient {
                    GradientStop { position: 0; color: Qt.rgba(Theme.ink.r, Theme.ink.g, Theme.ink.b, 0.24) }
                    GradientStop { position: 1; color: Qt.rgba(Theme.ink.r, Theme.ink.g, Theme.ink.b, 0) }
                }
            }
        }

        // The focus ring, for the keyboard only.
        Rectangle {
            objectName: "focusRing"
            anchors.fill: face
            anchors.margins: -(Theme.focusWidth + 2)
            radius: face.radius + Theme.focusWidth + 2
            color: "transparent"
            visible: button.visualFocus
            border.width: Theme.focusWidth
            border.color: Theme.focus
        }
    }
}
