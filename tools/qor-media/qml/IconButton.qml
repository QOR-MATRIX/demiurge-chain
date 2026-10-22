import QtQuick
import QtQuick.Controls.Basic

// A round, quiet button with a line icon, lit by the accent when hovered or held. Tab reaches it, and Return or Space
// presses it; a click leaves the keyboard where it was, so the player's and viewer's keys keep working.
Button {
    id: control
    property string iconName
    property int size: 40
    property bool active: false
    readonly property bool lit: control.hovered || control.visualFocus
    implicitWidth: size
    implicitHeight: size
    flat: true
    display: AbstractButton.IconOnly
    focusPolicy: Qt.TabFocus
    // Resolved here, beside the icons, whichever file the button is placed in.
    icon.source: iconName ? Qt.resolvedUrl("icons/" + iconName + ".svg") : ""
    icon.width: Math.round(size * 0.5)
    icon.height: Math.round(size * 0.5)
    icon.color: control.active ? Theme.accentBright : (control.lit ? Theme.ink : Theme.inkBody)
    ToolTip.visible: control.lit && ToolTip.text.length > 0
    ToolTip.delay: control.visualFocus ? 0 : 600
    background: Rectangle {
        radius: width / 2
        color: control.down ? Qt.alpha(Theme.accent, 0.35) : control.lit ? Qt.alpha(Theme.ink, 0.12) : "transparent"
        border.color: control.active ? Theme.accent : "transparent"
        Behavior on color { ColorAnimation { duration: Theme.quick } }
        FocusRing { visible: control.visualFocus; radius: parent.radius }
    }
    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()
}
