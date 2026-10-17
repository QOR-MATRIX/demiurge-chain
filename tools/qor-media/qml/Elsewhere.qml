import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QOR.Media

// What QOR Media does not open yet: said plainly, with the way to open it now. PDF comes with Qt PDF (L9.5), Office
// documents with the second release (L9.7); until then the program the system chooses opens them.
FocusScope {
    id: elsewhere
    required property string path
    signal back()
    focus: true

    readonly property string extension: path.slice(path.lastIndexOf(".") + 1).toLowerCase()
    readonly property string why: extension === "pdf"
        ? qsTr("PDFs open here once QOR Media's PDF viewer arrives.")
        : ["doc", "docx", "odt", "rtf", "xls", "xlsx", "ods", "ppt", "pptx", "odp"].indexOf(extension) >= 0
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
            font.family: Theme.display
            font.pixelSize: 44
            font.weight: Font.Bold
        }
        Text {
            Layout.fillWidth: true
            horizontalAlignment: Text.AlignHCenter
            text: elsewhere.why
            wrapMode: Text.WordWrap
            color: Theme.inkBody
            font.family: Theme.text
            font.pixelSize: 16
        }
        Button {
            id: open
            Layout.alignment: Qt.AlignHCenter
            Layout.topMargin: 8
            focusPolicy: Qt.StrongFocus
            focus: true
            text: qsTr("Open with another program")
            contentItem: Text { text: open.text; color: Theme.ink; font.family: Theme.text; font.pixelSize: 15; font.weight: Font.DemiBold; horizontalAlignment: Text.AlignHCenter }
            background: Rectangle { implicitHeight: 44; implicitWidth: 260; radius: 22; color: open.down ? Theme.accentDim : open.hovered ? Theme.accentBright : Theme.accent }
            onClicked: Files.openElsewhere(elsewhere.path)
            Keys.onReturnPressed: clicked()
        }
    }
}
