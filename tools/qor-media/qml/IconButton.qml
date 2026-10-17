import QtQuick
import QtQuick.Controls.Basic

// A round, quiet button with a line icon, lit by the accent when hovered or held.
Button {
    id: control
    property string iconName
    property int size: 40
    property bool active: false
    implicitWidth: size
    implicitHeight: size
    flat: true
    display: AbstractButton.IconOnly
    focusPolicy: Qt.NoFocus
    icon.source: iconName ? "icons/" + iconName + ".svg" : ""
    icon.width: Math.round(size * 0.5)
    icon.height: Math.round(size * 0.5)
    icon.color: control.active ? Theme.accentBright : (control.hovered ? Theme.ink : Theme.inkBody)
    ToolTip.visible: hovered && ToolTip.text.length > 0
    ToolTip.delay: 600
    background: Rectangle {
        radius: width / 2
        color: control.down ? Qt.alpha(Theme.accent, 0.35) : control.hovered ? Qt.alpha(Theme.ink, 0.12) : "transparent"
        border.color: control.active ? Theme.accent : "transparent"
        Behavior on color { ColorAnimation { duration: 120 } }
    }
}
