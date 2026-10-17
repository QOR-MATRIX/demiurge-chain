import QtQuick
import QtQuick.Controls.Basic
import QOR.Media

// The viewer (ADR-094 decision 2): a picture fitted to the window, zoomed around the pointer with the wheel, dragged when
// larger than the window, 1:1 on a double click; the folder's other pictures next and previous with the arrow keys, the
// picture turned with R, a slideshow with S. Turned as the camera wrote it.
FocusScope {
    id: viewer
    required property string path
    required property var link
    signal back()
    signal fullScreenToggled()
    focus: true

    property var pictures: Files.siblings(path)
    property int index: Math.max(0, pictures.indexOf(path))
    readonly property string current: pictures.length ? pictures[index] : path
    property real zoom: 1
    property int turn: 0
    property bool sliding: false

    function go(delta) {
        if (pictures.length < 2) return
        index = (index + delta + pictures.length) % pictures.length
        zoom = 1; turn = 0
        flick.contentX = 0; flick.contentY = 0
    }
    function fit() { zoom = 1; flick.contentX = 0; flick.contentY = 0 }

    Rectangle { anchors.fill: parent; color: Theme.void_ }

    Flickable {
        id: flick
        anchors.fill: parent
        contentWidth: Math.max(width, picture.paintedWidth * viewer.zoom)
        contentHeight: Math.max(height, picture.paintedHeight * viewer.zoom)
        clip: true
        interactive: viewer.zoom > 1
        boundsBehavior: Flickable.StopAtBounds

        Image {
            id: picture
            width: flick.width
            height: flick.height
            anchors.centerIn: parent
            source: Files.urlOf(viewer.current)
            fillMode: Image.PreserveAspectFit
            asynchronous: true
            autoTransform: true
            smooth: true
            mipmap: true
            scale: viewer.zoom
            rotation: viewer.turn
            Behavior on scale { NumberAnimation { duration: 140; easing.type: Easing.OutCubic } }
            Behavior on rotation { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
            onStatusChanged: if (status === Image.Ready) viewer.link.reportLearned(viewer.current, 0, sourceSize.width, sourceSize.height)
        }

        WheelHandler {
            onWheel: (event) => {
                const before = viewer.zoom
                viewer.zoom = Math.max(1, Math.min(12, viewer.zoom * (event.angleDelta.y > 0 ? 1.15 : 1 / 1.15)))
                // Keep the point under the pointer under it.
                const ratio = viewer.zoom / before
                flick.contentX = Math.max(0, (flick.contentX + event.x) * ratio - event.x)
                flick.contentY = Math.max(0, (flick.contentY + event.y) * ratio - event.y)
            }
        }
        TapHandler {
            onDoubleTapped: viewer.zoom > 1 ? viewer.fit() : (viewer.zoom = Math.max(1, picture.sourceSize.width / Math.max(1, picture.paintedWidth)))
        }
    }

    Text {
        anchors.centerIn: parent
        visible: picture.status === Image.Error
        text: qsTr("This picture's format is not one QOR Media reads yet.")
        color: Theme.inkBody
        font.family: Theme.text
        font.pixelSize: 16
    }

    TopBar {
        anchors { left: parent.left; right: parent.right; top: parent.top }
        title: Files.nameOf(viewer.current)
        subtitle: [picture.sourceSize.width > 0 ? picture.sourceSize.width + " × " + picture.sourceSize.height : "", viewer.pictures.length > 1 ? (viewer.index + 1) + " of " + viewer.pictures.length : "", viewer.zoom > 1 ? Math.round(viewer.zoom * 100) + "%" : ""].filter(Boolean).join(" · ")
        onBack: viewer.back()
        IconButton { iconName: "prev"; visible: viewer.pictures.length > 1; ToolTip.text: qsTr("Previous (←)"); onClicked: viewer.go(-1) }
        IconButton { iconName: "next"; visible: viewer.pictures.length > 1; ToolTip.text: qsTr("Next (→)"); onClicked: viewer.go(1) }
        IconButton { iconName: "rotate"; ToolTip.text: qsTr("Turn (R)"); onClicked: viewer.turn = (viewer.turn + 90) % 360 }
        IconButton { iconName: "play"; active: viewer.sliding; visible: viewer.pictures.length > 1; ToolTip.text: qsTr("Slideshow (S)"); onClicked: viewer.sliding = !viewer.sliding }
        IconButton { iconName: "full"; ToolTip.text: qsTr("Full screen (F)"); onClicked: viewer.fullScreenToggled() }
    }

    Timer {
        interval: 4000
        repeat: true
        running: viewer.sliding
        onTriggered: viewer.go(1)
    }

    Keys.onPressed: (event) => {
        switch (event.key) {
        case Qt.Key_Right: case Qt.Key_Space: viewer.go(1); break
        case Qt.Key_Left: viewer.go(-1); break
        case Qt.Key_R: viewer.turn = (viewer.turn + 90) % 360; break
        case Qt.Key_S: viewer.sliding = !viewer.sliding; break
        case Qt.Key_F: viewer.fullScreenToggled(); break
        case Qt.Key_0: viewer.fit(); break
        case Qt.Key_Plus: case Qt.Key_Equal: viewer.zoom = Math.min(12, viewer.zoom * 1.25); break
        case Qt.Key_Minus: viewer.zoom = Math.max(1, viewer.zoom / 1.25); break
        default: return
        }
        event.accepted = true
    }
}
