import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts

// The bar across the top: back to the Library, what is open, and the actions of whatever shows it.
Rectangle {
    id: bar
    property string title
    property string subtitle
    default property alias actions: tools.data
    signal back()
    height: 64
    gradient: Gradient {
        GradientStop { position: 0; color: Qt.alpha(Theme.void_, 0.92) }
        GradientStop { position: 1; color: "transparent" }
    }
    RowLayout {
        anchors.fill: parent
        anchors.leftMargin: 14
        anchors.rightMargin: 14
        spacing: 12
        IconButton {
            iconName: "back"
            ToolTip.text: qsTr("Back to the Library (Esc)")
            onClicked: bar.back()
        }
        ColumnLayout {
            Layout.fillWidth: true
            spacing: 0
            Text {
                Layout.fillWidth: true
                text: bar.title
                color: Theme.ink
                font.family: Theme.ui
                font.pixelSize: Theme.sizeHeading
                font.weight: Font.DemiBold
                elide: Text.ElideMiddle
            }
            Text {
                Layout.fillWidth: true
                visible: text.length > 0
                text: bar.subtitle
                color: Theme.inkMuted
                font.family: Theme.text
                font.pixelSize: Theme.sizeUi
                elide: Text.ElideRight
            }
        }
        Row {
            id: tools
            spacing: 4
        }
    }
}
