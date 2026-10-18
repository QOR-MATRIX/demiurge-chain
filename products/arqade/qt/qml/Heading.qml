import QtQuick
import QtQuick.Layouts
import ARQADE

// A section's heading: a gilt eyebrow on a rule that fades out, and a line under it.
ColumnLayout {
    id: heading
    property string eyebrow
    property string title
    property string line

    spacing: 8

    RowLayout {
        spacing: 14
        Layout.fillWidth: true
        Text {
            text: heading.eyebrow.toUpperCase()
            font.family: Theme.ui
            font.pixelSize: 12
            font.weight: Font.DemiBold
            font.letterSpacing: 3
            color: Theme.gilt
        }
        Rectangle {
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter
            Layout.preferredHeight: 1
            gradient: Gradient {
                orientation: Gradient.Horizontal
                GradientStop { position: 0; color: Theme.lineStrong }
                GradientStop { position: 1; color: "transparent" }
            }
        }
    }
    Text {
        visible: heading.title.length > 0
        text: heading.title
        font.family: Theme.display
        font.pixelSize: 28
        font.weight: Font.DemiBold
        font.letterSpacing: 1
        color: Theme.ink
        Layout.fillWidth: true
        wrapMode: Text.WordWrap
        Accessible.role: Accessible.Heading
        Accessible.name: heading.title
    }
    Text {
        visible: heading.line.length > 0
        text: heading.line
        font.family: Theme.ui
        font.pixelSize: 16
        color: Theme.inkMuted
        Layout.fillWidth: true
        Layout.maximumWidth: 720
        wrapMode: Text.WordWrap
        lineHeight: 1.15
    }
}
