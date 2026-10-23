import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QOR.Media

// What QOR Media does not open yet: said plainly, with the way to open it now. Office documents come with the second
// release (L9.7); until then the program the system chooses opens them.
FocusScope {
    id: elsewhere
    required property string path
    signal back()
    focus: true

    readonly property string extension: path.slice(path.lastIndexOf(".") + 1).toLowerCase()
    readonly property string why: ["doc", "docx", "odt", "rtf", "xls", "xlsx", "ods", "ppt", "pptx", "odp"].indexOf(extension) >= 0
        ? qsTr("Office documents open here in a later release.")
        : qsTr("QOR Media does not open this kind of file.")

    Rectangle { anchors.fill: parent; color: Theme.base }

    TopBar {
        anchors { left: parent.left; right: parent.right; top: parent.top }
        title: Files.nameOf(elsewhere.path)
        onBack: elsewhere.back()
    }

    ColumnLayout {
        anchors.centerIn: parent
        spacing: 16
        width: Math.min(parent.width - 48, 520)

        Text {
            Layout.alignment: Qt.AlignHCenter
            text: elsewhere.extension.toUpperCase()
            color: Theme.accentBright
            font.family: Theme.ui
            font.pixelSize: Theme.sizeFigure
            font.weight: Font.DemiBold
            font.letterSpacing: Theme.sizeFigure * Theme.trackingHeading
        }
        Text {
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            text: elsewhere.why
            wrapMode: Text.WordWrap
            color: Theme.inkBody
            font.family: Theme.text
            font.pixelSize: Theme.sizeTitle
        }
        TextButton {
            id: open
            objectName: "openElsewhere"
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 8
            primary: true
            focus: true
            text: qsTr("Open with another program")
            onClicked: Files.openElsewhere(elsewhere.path)
        }
    }
}
