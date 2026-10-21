// A button in QQ Studio's tool register (ADR-108): Barlow Semi Condensed in capitals, 8 pixel corners, every colour
// from Look. Four kinds:
//
//   primary   the one action a moment is for (Play, or Save while there is something to save): ember, lit from
//             above, with a glow and dark ink on it (7:1; white would read 2.9:1)
//   danger    an action that ends something (Stop): the status red
//   ghost     a secondary action (Publish): a hairline that turns gilt under the pointer
//   quiet     a tool (Undo, Stats): no frame until it is pointed at or on
//
// A ghost or quiet button that is `highlighted` (Save, with work unsaved) takes an ember edge and ink, so a moment never
// has two primaries.
//
// It lifts a pixel under the pointer and gives under a press, unless the person asked for less motion; the focus ring is
// the one every surface draws, for the keyboard only.

import QtQuick
import QtQuick.Controls
import QQ

Button {
    id: control

    /// "primary", "danger", "ghost" or "quiet".
    property string kind: "quiet"

    readonly property bool lit: control.enabled && (control.hovered || control.visualFocus)
    readonly property color fill: kind === "primary" ? Look.accent : kind === "danger" ? Look.bad : Look.raised
    readonly property color inkColour: kind === "primary" || kind === "danger" ? Look.onAccent
                                       : control.checked || control.highlighted ? Look.accentBright
                                       : control.lit ? Look.ink : Look.inkBody

    implicitHeight: 36
    leftPadding: kind === "quiet" ? 10 : 16
    rightPadding: kind === "quiet" ? 10 : 16
    focusPolicy: Qt.StrongFocus
    opacity: enabled ? 1 : 0.4

    font.family: Look.ui
    font.weight: Font.DemiBold
    font.pixelSize: Look.sizeBody
    font.letterSpacing: Look.sizeBody * Look.trackingLabel
    font.capitalization: Font.AllUppercase

    contentItem: Text {
        text: control.text
        font: control.font
        color: control.inkColour
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
        Behavior on color { ColorAnimation { duration: Look.quick } }
    }

    background: Item {
        implicitWidth: 64
        implicitHeight: 36
        transform: Translate { y: control.lit && !control.down && Look.quick > 0 ? -1 : 0 }
        scale: control.down && Look.quick > 0 ? 0.97 : 1
        Behavior on scale { NumberAnimation { duration: Look.quick; easing.type: Easing.BezierSpline; easing.bezierCurve: Look.ease } }

        // The glow of a filled button: its own colour, soft, a little stronger under the pointer.
        Rectangle {
            visible: control.kind === "primary" || control.kind === "danger"
            anchors.fill: face
            anchors.margins: -4
            radius: face.radius + 4
            color: "transparent"
            border.width: 4
            border.color: Qt.rgba(control.fill.r, control.fill.g, control.fill.b, control.lit ? 0.28 : 0.14)
            Behavior on border.color { ColorAnimation { duration: Look.standard } }
        }

        Rectangle {
            id: face
            anchors.fill: parent
            radius: Look.radiusControl
            color: control.kind === "primary" || control.kind === "danger"
                   ? (control.lit ? Qt.lighter(control.fill, 1.08) : control.fill)
                   : control.checked ? Qt.rgba(Look.accent.r, Look.accent.g, Look.accent.b, 0.14)
                   : control.lit ? Look.raised : "transparent"
            border.width: 1
            border.color: control.kind === "primary" ? Look.accentBright
                          : control.kind === "danger" ? Qt.lighter(Look.bad, 1.15)
                          : control.checked ? Look.accentDim
                          : control.highlighted ? Look.accent
                          : control.kind === "ghost" ? (control.lit ? Look.gilt : Look.edge)
                          : control.lit ? Look.edge : "transparent"
            Behavior on color { ColorAnimation { duration: Look.quick } }
            Behavior on border.color { ColorAnimation { duration: Look.quick } }

            // Lit from above: a band of light across the top of a filled button.
            Rectangle {
                visible: control.kind === "primary" || control.kind === "danger"
                anchors { left: parent.left; right: parent.right; top: parent.top; margins: 1 }
                height: parent.height / 2
                radius: face.radius - 1
                gradient: Gradient {
                    GradientStop { position: 0; color: Qt.rgba(1, 1, 1, 0.22) }
                    GradientStop { position: 1; color: Qt.rgba(1, 1, 1, 0) }
                }
            }
        }

        // The focus ring, for the keyboard only.
        Rectangle {
            anchors.fill: face
            anchors.margins: -(Look.focusWidth + 2)
            radius: face.radius + Look.focusWidth + 2
            color: "transparent"
            visible: control.visualFocus
            border.width: Look.focusWidth
            border.color: Look.focus
        }
    }
}
