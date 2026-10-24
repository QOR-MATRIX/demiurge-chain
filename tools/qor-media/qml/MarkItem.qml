import QtQuick
import QtQuick.Shapes
import QOR.Media

// A mark made in the reader and not yet saved, drawn on its page as PdfMarks will write it (src/pdfmarks.cpp), at the
// page's zoom: words highlighted, underlined or struck through, a note, ink, or a picture of a signature. It sits on the
// page's paper, in the page's points times `zoom`. A tap chooses it; a note or a signature is dragged to move it, and a
// chosen signature is sized by its corner. While it is dragged it shows where it is going (`shift`, `sizedTo`), and the
// reader changes the mark itself once, when the drag ends.
Item {
    id: item
    objectName: "mark"
    required property var mark
    required property real zoom
    property bool chosen: false
    property bool live: true
    property point shift: Qt.point(0, 0)
    property real sizedTo: 0
    readonly property bool placed: mark.kind === "note" || mark.kind === "signature"
    readonly property bool words: mark.kind === "highlight" || mark.kind === "underline" || mark.kind === "strikeout"
    // Where it is, in points, for choosing it and for the outline drawn while it is chosen.
    readonly property rect bounds: {
        if (mark.kind === "note") return Qt.rect(mark.x + shift.x, mark.y + shift.y, PdfMarks.noteSize, PdfMarks.noteSize)
        if (mark.kind === "signature") {
            const w = sizedTo > 0 ? sizedTo : mark.width
            return Qt.rect(mark.x + shift.x, mark.y + shift.y, w, w * mark.height / mark.width)
        }
        const rects = words ? mark.rects : []
        if (!words) for (const stroke of mark.strokes) for (const p of stroke) rects.push(Qt.rect(p.x, p.y, 0, 0))
        let l = Infinity, t = Infinity, r = -Infinity, b = -Infinity
        for (const q of rects) { l = Math.min(l, q.x); t = Math.min(t, q.y); r = Math.max(r, q.x + q.width); b = Math.max(b, q.y + q.height) }
        const pad = words ? 0 : mark.width / 2
        return Qt.rect(l - pad, t - pad, r - l + 2 * pad, b - t + 2 * pad)
    }
    signal picked()
    signal moveStarted()
    signal moved(real dx, real dy)
    signal sized(real width)
    signal moveEnded()

    anchors.fill: parent

    // Words: each run highlighted through, or a line under it or through its middle.
    Repeater {
        model: item.words ? item.mark.rects : []
        delegate: Rectangle {
            required property rect modelData
            readonly property real thickness: Math.min(3, Math.max(0.5, modelData.height / 14))
            x: modelData.x * item.zoom
            width: modelData.width * item.zoom
            y: (item.mark.kind === "highlight" ? modelData.y
                : item.mark.kind === "underline" ? modelData.y + modelData.height - thickness
                : modelData.y + modelData.height / 2 - thickness / 2) * item.zoom
            height: (item.mark.kind === "highlight" ? modelData.height : thickness) * item.zoom
            color: item.mark.kind === "highlight" ? Qt.alpha(item.mark.colour, PdfMarks.highlightOpacity) : item.mark.colour
        }
    }

    // A note: a sheet with its corner turned down and three lines on it.
    Shape {
        visible: item.mark.kind === "note"
        x: item.bounds.x * item.zoom
        y: item.bounds.y * item.zoom
        width: PdfMarks.noteSize * item.zoom
        height: width
        ShapePath {
            strokeColor: Qt.darker(item.mark.colour, 1.7)
            strokeWidth: item.zoom
            fillColor: item.mark.colour
            joinStyle: ShapePath.RoundJoin
            scale: Qt.size(PdfMarks.noteSize * item.zoom, PdfMarks.noteSize * item.zoom)
            PathPolyline { path: [Qt.point(0, 0), Qt.point(0.7, 0), Qt.point(1, 0.3), Qt.point(1, 1), Qt.point(0, 1), Qt.point(0, 0)] }
        }
        ShapePath {
            strokeColor: Qt.darker(item.mark.colour, 1.7)
            strokeWidth: item.zoom
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            scale: Qt.size(PdfMarks.noteSize * item.zoom, PdfMarks.noteSize * item.zoom)
            PathMultiline {
                paths: [[Qt.point(0.7, 0), Qt.point(0.7, 0.3), Qt.point(1, 0.3)], [Qt.point(0.2, 0.5), Qt.point(0.8, 0.5)],
                        [Qt.point(0.2, 0.65), Qt.point(0.8, 0.65)], [Qt.point(0.2, 0.8), Qt.point(0.8, 0.8)]]
            }
        }
    }

    // Ink, in the page's points.
    Shape {
        visible: item.mark.kind === "ink"
        anchors.fill: parent
        ShapePath {
            strokeColor: item.mark.colour
            strokeWidth: (item.mark.width || 1) * item.zoom
            fillColor: "transparent"
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            scale: Qt.size(item.zoom, item.zoom)
            PathMultiline { paths: item.mark.kind === "ink" ? item.mark.strokes : [] }
        }
    }

    // A signature, from 0 to 1 across and down its box: lines drawn, or a name's outlines filled.
    Shape {
        visible: item.mark.kind === "signature"
        x: item.bounds.x * item.zoom
        y: item.bounds.y * item.zoom
        width: item.bounds.width * item.zoom
        height: item.bounds.height * item.zoom
        ShapePath {
            strokeColor: item.mark.colour
            strokeWidth: item.mark.strokes ? (item.mark.line || 0) * item.bounds.height * item.zoom : -1
            fillColor: item.mark.shapes ? item.mark.colour : "transparent"
            fillRule: ShapePath.WindingFill
            capStyle: ShapePath.RoundCap
            joinStyle: ShapePath.RoundJoin
            scale: Qt.size(item.bounds.width * item.zoom, item.bounds.height * item.zoom)
            PathMultiline { paths: item.mark.kind !== "signature" ? [] : item.mark.strokes || item.mark.shapes }
        }
    }

    // Chosen: outlined, and a signature's corner to size it by.
    Rectangle {
        visible: item.chosen
        x: item.bounds.x * item.zoom - 3
        y: item.bounds.y * item.zoom - 3
        width: item.bounds.width * item.zoom + 6
        height: item.bounds.height * item.zoom + 6
        color: "transparent"
        border.width: 2
        border.color: Theme.accent
    }

    // What takes the pointer: the mark's own bounds, and the corner. A press here is the mark's alone, so the page under
    // it neither selects words nor lets go of what is chosen; moved past the drag distance, it is a drag, else a tap.
    Item {
        id: hit
        x: item.bounds.x * item.zoom
        y: item.bounds.y * item.zoom
        width: Math.max(8, item.bounds.width * item.zoom)
        height: Math.max(8, item.bounds.height * item.zoom)
        enabled: item.live
        MouseArea {
            anchors.fill: parent
            cursorShape: item.placed ? Qt.SizeAllCursor : Qt.PointingHandCursor
            // The pages scroll when dragged; a mark being dragged keeps its drag.
            preventStealing: true
            property point from
            property bool dragging: false
            onPressed: (mouse) => {
                from = mapToItem(item, mouse.x, mouse.y)
                dragging = false
            }
            onPositionChanged: (mouse) => {
                if (!item.placed || !pressed) return
                const at = mapToItem(item, mouse.x, mouse.y)
                if (!dragging && Math.hypot(at.x - from.x, at.y - from.y) < Application.styleHints.startDragDistance) return
                if (!dragging) {
                    dragging = true
                    item.moveStarted()
                }
                item.moved((at.x - from.x) / item.zoom, (at.y - from.y) / item.zoom)
            }
            onReleased: {
                if (dragging) item.moveEnded()
                else item.picked()
                dragging = false
            }
            onCanceled: {
                if (dragging) item.moveEnded()
                dragging = false
            }
        }
    }
    Rectangle {
        objectName: "signatureCorner"
        visible: item.chosen && item.mark.kind === "signature"
        x: (item.bounds.x + item.bounds.width) * item.zoom - 7
        y: (item.bounds.y + item.bounds.height) * item.zoom - 7
        width: 14
        height: 14
        radius: 2
        color: Theme.accent
        border.color: Theme.inkOnAccent
        MouseArea {
            anchors.fill: parent
            anchors.margins: -6
            cursorShape: Qt.SizeFDiagCursor
            preventStealing: true
            property point from
            property real start
            onPressed: (mouse) => {
                from = mapToItem(item, mouse.x, mouse.y)
                start = item.bounds.width
                item.moveStarted()
            }
            onPositionChanged: (mouse) => {
                if (pressed) item.sized(start + (mapToItem(item, mouse.x, mouse.y).x - from.x) / item.zoom)
            }
            onReleased: item.moveEnded()
            onCanceled: item.moveEnded()
        }
    }
}
