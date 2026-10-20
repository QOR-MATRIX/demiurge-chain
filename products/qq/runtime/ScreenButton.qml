// A button on a game's screen (P3.8): Play, Resume, Try again, Quit to title. Pressed with the pointer or by touch while
// the world plays and its screen is shown, it sends `clicked`, which a Behaviour that targets it hands its logic
// (`onClicked`). A target of 48 points tall or more, so a finger finds it; it lifts under the pointer, unless the person
// asked for less motion.
//
// With an `action` (P3.9), it is also a touch control: the action is held while a finger, or the pointer, is on it, so
// a phone plays with the same actions a keyboard and a gamepad have.

import QtQuick
import QQ

ScreenPart {
    id: button

    readonly property string kind: "ScreenButton"
    readonly property var fields: ["name", "screen", "anchor", "insetX", "insetY", "width", "text", "colour", "action"]
    readonly property var ranges: ({ insetX: [-2000, 2000], insetY: [-2000, 2000], width: [0, 1280] })
    readonly property var choices: ({ anchor: places })

    name: "Button"
    /// Points; 0 fits the text.
    property real width: 0
    property string text: "Play"
    /// Its light: the edge and the glow when it is pointed at.
    property color colour: "#c9a45c"
    /// An action it holds while pressed (InputAction, P3.9), or nothing.
    property string action: ""

    /// Pressed, while the world plays and its screen is shown.
    signal clicked()

    /// Whether it answers now.
    readonly property bool active: live && shown

    drawing: Component {
        Rectangle {
            id: face
            readonly property bool lit: area.containsMouse || area.pressed
            width: (button.width > 0 ? button.width : label.implicitWidth / button.unit + 56) * button.unit
            height: 48 * button.unit
            radius: 6 * button.unit
            color: lit ? Qt.rgba(Look.surface.r * 1.6, Look.surface.g * 1.6, Look.surface.b * 1.6, 0.94)
                       : Qt.rgba(Look.surface.r, Look.surface.g, Look.surface.b, 0.88)
            border.width: Math.max(1, button.unit)
            border.color: lit ? Qt.lighter(button.colour, 1.25) : Qt.darker(button.colour, 1.35)
            scale: area.pressed && Look.fade > 0 ? 0.97 : 1
            transform: Translate { y: face.lit && Look.fade > 0 ? -2 * button.unit : 0 }

            Behavior on color { enabled: Look.fade > 0; ColorAnimation { duration: 120 } }

            // A line of light along the foot, brighter under the pointer.
            Rectangle {
                anchors.bottom: parent.bottom
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.bottomMargin: Math.max(1, button.unit)
                width: parent.width * (face.lit ? 0.72 : 0.4)
                height: Math.max(1, 2 * button.unit)
                radius: height / 2
                color: button.colour
                opacity: face.lit ? 0.95 : 0.45
                Behavior on width { enabled: Look.fade > 0; NumberAnimation { duration: 160; easing.type: Easing.OutCubic } }
            }

            Text {
                id: label
                anchors.centerIn: parent
                text: button.text.toUpperCase()
                color: Look.ink
                font.family: Look.ui
                font.weight: Font.DemiBold
                font.pixelSize: 18 * button.unit
                font.letterSpacing: 0.12 * 18 * button.unit
            }

            MouseArea {
                id: area
                anchors.fill: parent
                enabled: button.active
                hoverEnabled: true
                cursorShape: enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                // Delivered at the scene's next tick, so a replay clicks it at the same moment (P3.11).
                onClicked: if (button.parent && button.parent._click) button.parent._click(button); else button.clicked()
                onPressed: if (button.action !== "") Input.hold(button.action, true)
                onReleased: if (button.action !== "") Input.hold(button.action, false)
                onCanceled: if (button.action !== "") Input.hold(button.action, false)
            }
        }
    }
}
