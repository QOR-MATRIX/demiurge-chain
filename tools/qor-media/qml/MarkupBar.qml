import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QOR.Media

// The reader's marks (L9.5), under its top bar while marking up: highlight, underline and strike through the words
// selected; a note; drawing by hand; a signature; the colour the next mark takes; and taking away the mark chosen. Each
// is a button Tab reaches, and most a key the reader answers.
Rectangle {
    id: bar
    objectName: "markupBar"
    // What the reader has: words selected, drawing on, a mark chosen, and the colours chosen for words and for the pen.
    property bool hasWords: false
    property bool drawing: false
    property bool hasChosen: false
    property string wordsColour
    property string penColour
    property string hint
    signal marked(string kind)
    signal noted()
    signal drew()
    signal signed()
    signal removed()
    signal coloured(string colour)

    height: 52
    color: Theme.base

    RowLayout {
        anchors { fill: parent; leftMargin: 14; rightMargin: 14 }
        spacing: 4

        IconButton { objectName: "markHighlight"; iconName: "highlight"; enabled: bar.hasWords; opacity: enabled ? 1 : 0.4; ToolTip.text: qsTr("Highlight the words selected (H)"); onClicked: bar.marked("highlight") }
        IconButton { objectName: "markUnderline"; iconName: "underline"; enabled: bar.hasWords; opacity: enabled ? 1 : 0.4; ToolTip.text: qsTr("Underline them (U)"); onClicked: bar.marked("underline") }
        IconButton { objectName: "markStrike"; iconName: "strike"; enabled: bar.hasWords; opacity: enabled ? 1 : 0.4; ToolTip.text: qsTr("Strike them through (X)"); onClicked: bar.marked("strikeout") }
        Rectangle { Layout.preferredWidth: 1; Layout.preferredHeight: 24; Layout.leftMargin: 6; Layout.rightMargin: 6; color: Theme.edge }
        IconButton { objectName: "markNote"; iconName: "note"; ToolTip.text: qsTr("A note (N)"); onClicked: bar.noted() }
        IconButton { objectName: "markDraw"; iconName: "draw"; active: bar.drawing; ToolTip.text: bar.drawing ? qsTr("Stop drawing (D)") : qsTr("Draw by hand (D)"); onClicked: bar.drew() }
        IconButton { objectName: "markSign"; iconName: "sign"; ToolTip.text: qsTr("Sign"); onClicked: bar.signed() }
        Rectangle { Layout.preferredWidth: 1; Layout.preferredHeight: 24; Layout.leftMargin: 6; Layout.rightMargin: 6; color: Theme.edge }

        // The colours a mark can be, which PdfMarks keeps: what the PDF holds, the same in every theme. One choice for
        // words and notes, one for the pen; drawing shows the pen's.
        Repeater {
            model: PdfMarks.colours
            delegate: Swatch {
                picked: modelData.colour === (bar.drawing ? bar.penColour : bar.wordsColour)
                onClicked: bar.coloured(modelData.colour)
            }
        }

        Text {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            text: bar.hint
            elide: Text.ElideRight
            color: Theme.inkMuted
            font.family: Theme.text
            font.pixelSize: Theme.sizeUi
        }
        IconButton { objectName: "markRemove"; iconName: "delete"; visible: bar.hasChosen; ToolTip.text: qsTr("Take the mark away (Delete)"); onClicked: bar.removed() }
    }

    Rectangle { anchors { left: parent.left; right: parent.right; bottom: parent.bottom } height: 1; color: Theme.edge }
}
