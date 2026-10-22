import QtQuick
import QtQuick.Controls.Basic

// A button with words, in the tool register (ADR-108): Barlow Semi Condensed in tracked capitals, a control's corners,
// every colour from Theme. `primary` is the one action a moment is for: ember, with dark ink on it. Otherwise it is quiet,
// a hairline that lights under the pointer. Tab reaches it, and Return or Space presses it.
Button {
    id: control
    property bool primary: false
    readonly property bool lit: control.enabled && (control.hovered || control.visualFocus)

    implicitHeight: 36
    leftPadding: 16
    rightPadding: 16
    focusPolicy: Qt.StrongFocus
    opacity: enabled ? 1 : 0.4

    font.family: Theme.ui
    font.weight: Font.DemiBold
    font.pixelSize: Theme.sizeBody
    font.letterSpacing: Theme.sizeBody * Theme.trackingLabel
    font.capitalization: Font.AllUppercase

    contentItem: Text {
        text: control.text
        font: control.font
        color: control.primary ? Theme.inkOnAccent : control.lit ? Theme.ink : Theme.inkBody
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        implicitWidth: 64
        implicitHeight: 36
        radius: Theme.radiusControl
        color: control.primary ? (control.lit && !control.down ? Theme.accentBright : Theme.accent)
                               : control.down ? Theme.well : control.lit ? Theme.raised : "transparent"
        border.width: 1
        border.color: control.primary ? Theme.accentBright : control.lit ? Theme.inkMuted : Theme.edge
        Behavior on color { ColorAnimation { duration: Theme.quick } }
        FocusRing { visible: control.visualFocus }
    }

    Keys.onReturnPressed: clicked()
    Keys.onEnterPressed: clicked()
}
