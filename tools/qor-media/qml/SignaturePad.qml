import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Shapes
import QOR.Media

// A signature to place on a page (ADR-114 decision 1): drawn on the pad with the pointer, a pen or a finger, or a name
// typed and drawn in a handwriting face (PdfMarks.typed). It is a picture of a signature, kept only while QOR Media is
// open and sent nowhere, and the pad says plainly that it certifies nothing. `placed` hands it over: from 0 to 1 across
// and down its own box, with the box's width for each unit of its height, and the colour chosen.
Rectangle {
    id: pad
    objectName: "signaturePad"
    property string colour
    property string way: "draw"
    // The lines drawn, in the pad's pixels; the one being drawn.
    property var strokes: []
    property var current: null
    readonly property real line: 2.5
    readonly property var typedLook: way === "type" ? PdfMarks.typed(name.text) : ({})
    readonly property bool ready: way === "draw" ? strokes.length > 0 : !!typedLook.shapes
    signal placed(var signature)
    signal cancelled()

    function open() {
        strokes = []
        current = null
        name.clear()
        visible = true
        if (way === "type") name.forceActiveFocus()
        else drawTab.forceActiveFocus()
    }
    function place() {
        if (!ready) return
        let signature
        if (way === "type") {
            signature = { aspect: typedLook.aspect, shapes: typedLook.shapes }
        } else {
            // The lines' own bounds, with their width, are the box; a line drawn flat still has some height.
            let l = Infinity, t = Infinity, r = -Infinity, b = -Infinity
            for (const stroke of strokes) for (const p of stroke) { l = Math.min(l, p.x); t = Math.min(t, p.y); r = Math.max(r, p.x); b = Math.max(b, p.y) }
            let w = r - l + line, h = b - t + line
            const least = 12
            if (h < least) { t -= (least - h) / 2; h = least }
            if (w < least) { l -= (least - w) / 2; w = least }
            l -= line / 2
            t -= line / 2
            signature = { aspect: w / h, line: line / h,
                          strokes: strokes.map((stroke) => stroke.map((p) => Qt.point((p.x - l) / w, (p.y - t) / h))) }
        }
        signature.colour = colour
        visible = false
        placed(signature)
    }

    visible: false
    width: 560
    height: 376
    radius: Theme.radiusPanel
    color: Theme.raised
    border.color: Theme.edge

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 22
        spacing: 10
        Text { text: qsTr("Your signature"); color: Theme.ink; font.family: Theme.ui; font.pixelSize: Theme.sizeHeading; font.weight: Font.DemiBold }
        RowLayout {
            spacing: 8
            // The way showing is underlined in the accent, as the reader's tabs are.
            TextButton {
                id: drawTab
                objectName: "signDraw"
                text: qsTr("Draw")
                onClicked: pad.way = "draw"
                Rectangle { anchors { left: parent.left; right: parent.right; bottom: parent.bottom } height: 2; color: Theme.accent; visible: pad.way === "draw" }
            }
            TextButton {
                objectName: "signType"
                text: qsTr("Type")
                onClicked: { pad.way = "type"; name.forceActiveFocus() }
                Rectangle { anchors { left: parent.left; right: parent.right; bottom: parent.bottom } height: 2; color: Theme.accent; visible: pad.way === "type" }
            }
            Item { Layout.fillWidth: true }
            Repeater {
                model: PdfMarks.colours
                delegate: Swatch {
                    picked: modelData.colour === pad.colour
                    onClicked: pad.colour = modelData.colour
                }
            }
        }

        // The pad: paper, a Rectangle's own white, with a line to sign along.
        Rectangle {
            id: paper
            objectName: "signatureArea"
            visible: pad.way === "draw"
            Layout.fillWidth: true
            Layout.preferredHeight: 168
            radius: Theme.radiusControl
            Rectangle {
                anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: 28 }
                height: 1
                color: Theme.inkMuted
            }
            Shape {
                anchors.fill: parent
                ShapePath {
                    strokeColor: pad.colour
                    strokeWidth: pad.line
                    fillColor: "transparent"
                    capStyle: ShapePath.RoundCap
                    joinStyle: ShapePath.RoundJoin
                    PathMultiline { paths: pad.current ? pad.strokes.concat([pad.current]) : pad.strokes }
                }
            }
            DragHandler {
                target: null
                dragThreshold: 0
                acceptedDevices: PointerDevice.Mouse | PointerDevice.Stylus | PointerDevice.TouchScreen
                onActiveChanged: {
                    if (active) {
                        pad.current = [centroid.pressPosition]
                    } else if (pad.current) {
                        pad.strokes = pad.strokes.concat([pad.current.length > 1 ? pad.current : [pad.current[0], pad.current[0]]])
                        pad.current = null
                    }
                }
                onCentroidChanged: if (active && pad.current) pad.current = pad.current.concat([centroid.position])
            }
        }

        // Typed: the name, and how it will look.
        TextField {
            id: name
            objectName: "signatureName"
            visible: pad.way === "type"
            Layout.fillWidth: true
            placeholderText: qsTr("Your name")
            color: Theme.ink
            placeholderTextColor: Theme.inkFaint
            font.family: Theme.text
            font.pixelSize: Theme.sizeBody
            background: Rectangle { radius: Theme.radiusControl; color: Theme.well; border.color: name.activeFocus ? Theme.focus : Theme.edge }
            onAccepted: pad.place()
        }
        Rectangle {
            id: preview
            visible: pad.way === "type"
            Layout.fillWidth: true
            Layout.preferredHeight: 120
            radius: Theme.radiusControl
            readonly property real aspect: pad.typedLook.aspect || 1
            readonly property real across: Math.min(width - 40, (height - 24) * aspect)
            Shape {
                x: (preview.width - preview.across) / 2
                y: (preview.height - preview.across / preview.aspect) / 2
                width: preview.across
                height: preview.across / preview.aspect
                ShapePath {
                    strokeWidth: -1
                    strokeColor: "transparent"
                    fillColor: pad.colour
                    fillRule: ShapePath.WindingFill
                    scale: Qt.size(preview.across, preview.across / preview.aspect)
                    PathMultiline { paths: pad.typedLook.shapes || [] }
                }
            }
        }

        Text {
            Layout.fillWidth: true
            wrapMode: Text.WordWrap
            text: qsTr("A picture of your signature, placed on the page. It does not certify the PDF.")
            color: Theme.inkMuted
            font.family: Theme.text
            font.pixelSize: Theme.sizeUi
        }
        Item { Layout.fillHeight: true }
        RowLayout {
            Layout.alignment: Qt.AlignRight
            spacing: 8
            TextButton { objectName: "signClear"; text: qsTr("Clear"); visible: pad.way === "draw"; enabled: pad.strokes.length > 0; onClicked: pad.strokes = [] }
            TextButton { objectName: "signCancel"; text: qsTr("Cancel"); onClicked: pad.cancelled() }
            TextButton { objectName: "signPlace"; text: qsTr("Place"); primary: true; enabled: pad.ready; onClicked: pad.place() }
        }
    }
}
