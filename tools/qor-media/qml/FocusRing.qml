import QtQuick

// The focus ring every surface draws (ADR-108 decision 5), for the keyboard only: a soft halo, and 2 pixels of bright
// ember just outside what has focus. It fills its parent; a control shows it while visualFocus.
Item {
    id: ring
    objectName: "focusRing"
    property real radius: Theme.radiusControl
    anchors.fill: parent

    Rectangle {
        anchors.fill: parent
        anchors.margins: -Theme.focusHalo
        radius: ring.radius + Theme.focusHalo
        color: "transparent"
        border.width: Theme.focusHalo
        border.color: Qt.alpha(Theme.focus, 0.22)
    }
    Rectangle {
        anchors.fill: parent
        anchors.margins: -(Theme.focusWidth + 2)
        radius: ring.radius + Theme.focusWidth + 2
        color: "transparent"
        border.width: Theme.focusWidth
        border.color: Theme.focus
    }
}
