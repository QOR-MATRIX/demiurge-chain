// The pages are a TableView's delegates, which in this Qt see the ids of this file only when bound to it.
pragma ComponentBehavior: Bound

import QtQml.Models
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QtQuick.Pdf
import QtQuick.Shapes
import QOR.Media

// The reader (ADR-094 decision 3, L9.5): a PDF's pages one under another, drawn by Qt PDF on their own paper, everything
// around them in the theme's colours. It opens fitted to the width; 0 fits the width or the whole page in turn, and + and
// - or Ctrl with the wheel zoom around the pointer. Page Up, Page Down, Space, Home and End go through it, the arrows too,
// and Ctrl+G goes to a page by its number. Ctrl+F finds words on every page, F3 and Shift+F3 go from one to the next; text
// is selected by dragging across it, Ctrl+A selects a page's, and Ctrl+C copies it. A link goes to its place in the
// document, or out to the web or to mail, and nowhere else. F9 shows the pages and the contents beside it. A PDF with a
// password asks for it and keeps it only while it is open; one that cannot be read says so, with the way to open it now.
//
// Its pages change too: R turns the page in view (Shift+R the other way), Delete deletes it, Alt+Up and Alt+Down move it,
// Ctrl+Z and Ctrl+Y undo and redo, and Ctrl+S saves through PDFium (PdfPages). A change shows at once and reaches the file
// only when it is saved; going back with changes unsaved asks first. A locked or protected PDF is read, never changed.
//
// And it is marked up (PdfMarks, MarkupBar): M shows the marks. H, U and X highlight, underline and strike through the
// words selected; N writes a note; D draws by hand; Sign places a picture of the person's signature, drawn or typed,
// which certifies nothing (ADR-114 decision 1). A note or a signature is dragged into place, or moved by the arrows when
// chosen (Shift for ten points), and [ and ] size a signature. A tap chooses a mark, the PDF's own too, and Delete takes
// it away. A PDF's own note shows its words when tapped, even with the marks hidden. Marks are undone, redone and saved
// with the pages' changes, in one write.
//
// And a form is filled (PdfForms, FieldItem): its fields are tinted, and a tap fills one. A text field is typed in where
// it is, kept by Return or Tab and left by Esc; a check box is ticked, a radio button chosen, and a list's options chosen
// from under it. Tab goes from field to field, on to the next page's, and into the first on the page in view. A tap on a
// signature field places a picture of the person's signature in it, which certifies nothing. What is filled shows at once
// in the page's image, drawn by PDFium as it will be saved, and is undone, redone and saved with the rest.
FocusScope {
    id: reader
    objectName: "reader"
    required property string path
    signal back()
    signal fullScreenToggled()
    focus: true

    // Pixels for each of the PDF's points, and what they follow: "width", "page", or "" once zoomed by hand.
    property real zoom: 1
    property string fitting: "width"
    property int page: 0
    property bool locked: false
    property bool wrongPassword: false
    property string triedPassword
    property bool panel: false
    property string panelTab: "pages"
    property var selecting: null
    property string selectedText
    // The pages as they show, one entry for each: `page`, the page's index in the file, and `turn`, the quarter turns
    // clockwise added to it. The file is written to match only when it is saved.
    property var layout: []
    property var undoStack: []
    property var redoStack: []
    property bool dirty: false
    // Marks (PdfMarks): those made here and not yet saved, each naming its page in the file as it is now and an id of its
    // own; the PDF's own, read a page at a time as each is shown (heldOn), since reading a page parses all it draws; and
    // those of its own to take away when it is saved.
    property var added: []
    property var removed: []
    property var held: ({})
    property int markIds: 0
    property bool marking: false
    // "ink" while drawing by hand.
    property string tool: ""
    // The mark chosen: {from: "added", id} or {from: "held", page, index}.
    property var chosen: null
    property string wordsColour: PdfMarks.colours[0].colour
    property string penColour: PdfMarks.colours[PdfMarks.colours.length - 1].colour
    // A line being drawn, {page, points}; a note being written, {page, x, y, colour}; a mark being dragged, {id, dx, dy,
    // width}.
    property var inking: null
    property var noteDraft: null
    property var dragged: null
    // A form's fields (PdfForms): the file's, read a page at a time as each is shown (fieldsOn), afresh with each file and
    // each save; what the person has filled and not yet saved, by field, each entry naming the widget it was filled
    // through; and the field the keyboard is on, with whether it is typed in or its options shown: {page, index, mode: "",
    // "typing" or "listing"}. The words being typed are kept in their field only on Return, Tab or leaving it.
    property var fields: ({})
    property var fills: ({})
    property var fieldFocus: null
    property string typingWords
    // The filling the pages' images show: none (0), or the last one handed to PdfDrawing.
    property int shownFills: 0
    property int fillings: 0
    property bool formSaid: false
    // The signature field a signature being made goes into.
    property var signTarget: null
    property bool editable: false
    // The widest page as laid out, in points.
    property real widest: 1
    property string notice
    property var afterAsk: null
    // While saving, the reader shows a copy so the file can be replaced; then it opens the file again. No page is drawn
    // across that, so none outlives the file it was drawn from.
    property bool saving: false
    property bool swapping: false
    property bool keepLayout: false
    property int returnTo: -1
    readonly property bool ready: doc.status === PdfDocument.Ready
    readonly property bool unreadable: doc.status === PdfDocument.Error && !locked
    readonly property int pageCount: ready && !swapping ? layout.length : 0
    readonly property int currentSource: page < layout.length ? layout[page].page : -1
    readonly property int matchCount: finder.visible ? reader.matches : 0
    readonly property int match: search.currentResult
    // A hundred percent is the page at its printed size.
    readonly property real actualSize: 96 / 72
    readonly property real minZoom: actualSize * 0.1
    readonly property real maxZoom: actualSize * 8
    // Room either side of the widest page, and above the first.
    readonly property int margin: 24
    readonly property bool consumesEscape: dirty || asking.visible || finder.visible || pageField.activeFocus || signing.visible
                                           || noteEditor.visible || fieldFocus !== null || tool.length > 0 || chosen !== null || marking
    readonly property string markHint: tool === "ink" ? qsTr("Drag across a page to draw. D or Esc stops.")
        : chosen && chosen.from === "held" ? qsTr("One of the PDF's own marks. Delete takes it away.")
        : chosen ? qsTr("Drag it, or move it with the arrows. Delete takes it away.")
        : selectedText.length ? qsTr("H highlights the words selected, U underlines them, X strikes them through.")
        : qsTr("Select words to mark them, or add a note, a drawing or a signature.")

    function entryAt(row) {
        return layout[row] || ({ page: 0, turn: 0 })
    }
    // Where a page of the file shows now, or -1 when it is deleted.
    function rowOf(source) {
        return layout.findIndex((e) => e.page === source)
    }
    // A page's size as laid out, turned, in points.
    function laidOut(row) {
        const e = entryAt(row)
        const s = doc.pagePointSize(e.page)
        return e.turn % 2 ? Qt.size(s.height, s.width) : s
    }

    function fit() {
        if (!ready || pages.width <= 0) return
        const across = pages.width - 2 * margin
        const size = laidOut(page)
        const z = fitting === "page"
            ? Math.min(across / size.width, (pages.height - 2 * margin) / size.height)
            : across / Math.max(1, widest)
        setZoom(Math.max(minZoom, Math.min(maxZoom, z)), page)
    }
    function cycleFit() {
        fitting = fitting === "width" ? "page" : "width"
        fit()
    }
    // Zoom to `z`, keeping `keep` (a page) at the top of the view.
    function setZoom(z, keep) {
        zoom = z
        pages.forceLayout()
        goTo(keep)
    }
    // Zoom by `factor` around a point of the view, which stays under it.
    function zoomBy(factor, atX, atY) {
        const before = zoom
        const after = Math.max(minZoom, Math.min(maxZoom, before * factor))
        if (after === before || !ready) return
        fitting = ""
        const cell = pages.cellAtPosition(Qt.point(pages.contentX + atX, pages.contentY + atY), true)
        const item = cell.y >= 0 ? pages.itemAtCell(cell) : null
        const down = item ? (pages.contentY + atY - item.y) / before : 0
        const across = pages.contentX + atX
        zoom = after
        pages.forceLayout()
        const moved = item ? pages.itemAtCell(cell) : null
        if (moved)
            pages.contentY = moved.y + down * after - atY
        pages.contentX = Math.max(0, across * after / before - atX)
        clamp()
    }
    function clamp() {
        pages.contentX = Math.max(pages.originX, Math.min(pages.contentX, pages.originX + pages.contentWidth - pages.width))
        const top = pages.originY - pages.topMargin
        pages.contentY = Math.max(top, Math.min(pages.contentY, pages.originY + pages.contentHeight + pages.bottomMargin - pages.height))
    }
    // Go to a page, and to a place on it in the PDF's points when there is one.
    function goTo(target, location) {
        if (!ready || !pageCount) return
        target = Math.max(0, Math.min(pageCount - 1, target))
        const y = location ? location.y * zoom - margin : -1
        if (y > 0)
            pages.positionViewAtRow(target, TableView.AlignTop, 0, Qt.rect(0, y, 1, 1))
        else
            pages.positionViewAtRow(target, TableView.AlignTop, -margin)
        // A page not laid out yet is gone to at the view's next frame, which could land after a later jump and undo it.
        pages.forceLayout()
        page = target
    }
    function scrollBy(dy) {
        pages.contentY += dy
        clamp()
    }
    // The page under the top third of the view.
    function track() {
        const cell = pages.cellAtPosition(Qt.point(pages.contentX + pages.width / 2, pages.contentY + pages.height / 3), true)
        if (cell.y >= 0)
            page = cell.y
    }
    // Where a link may take the person: a place in the document, the web, or mail. Never a file or a program.
    function followable(url) {
        const s = String(url).toLowerCase()
        return s.startsWith("https://") || s.startsWith("http://") || s.startsWith("mailto:")
    }
    function follow(link) {
        if (link.page >= 0) {
            const row = rowOf(link.page)
            if (row >= 0) goTo(row, link.location)
            else say(qsTr("That page is no longer in this PDF."))
        } else if (followable(link.url)) {
            Qt.openUrlExternally(link.url)
        }
    }
    // A page's item, laid out now if the view has not yet made it (it makes them as it draws).
    function pageItem(row) {
        if (!pages.itemAtCell(Qt.point(0, row)))
            pages.forceLayout()
        return pages.itemAtCell(Qt.point(0, row))
    }
    function selectPage() {
        const item = pageItem(page)
        if (!item) return
        if (selecting && selecting !== item.selection) selecting.clear()
        selecting = item.selection
        selecting.selectAll()
        selectedText = selecting.text
    }
    function copy() {
        if (selecting) selecting.copyToClipboard()
    }
    // Search: the first match on or after the page in view, then each in turn.
    function findFrom(here) {
        for (let i = 0; i < reader.matches; ++i)
            if (rowOf(search.data(search.index(i, 0), 256)) >= here) return i  // QPdfSearchModel::Role::Page
        return reader.matches ? 0 : -1
    }
    function step(delta) {
        if (!reader.matches) return
        search.currentResult = (search.currentResult + delta + reader.matches) % reader.matches
    }
    function showMatch() {
        const link = search.currentResultLink
        if (search.currentResult < 0 || !link || link.page < 0) return
        const row = rowOf(link.page)
        if (row < 0) return
        const r = search.currentResultBoundingRect
        pages.positionViewAtRow(row, TableView.AlignTop, 0, Qt.rect(0, Math.max(0, r.y * zoom - pages.height / 3), 1, 1))
        pages.forceLayout()
        page = row
    }
    function openFinder() {
        finder.visible = true
        query.selectAll()
        query.forceActiveFocus()
    }
    // The key PdfDrawing opened this PDF under, for its pages' images; empty while none is open.
    property string drawingKey
    function pageImage(source) {
        if (!drawingKey.length) return ""
        const hidden = removed.filter((r) => r.page === source).map((r) => r.index)
        // A page with fields, once something is filled, is drawn with them filled, afresh for each filling.
        const filled = shownFills && fieldsOn(source).length ? "/" + shownFills : ""
        return "image://pdfpage/" + drawingKey + "/" + source + (hidden.length || filled.length ? "/" + hidden.join(",") : "") + filled
    }
    function draw() {
        stopDrawing()
        drawingKey = PdfDrawing.open(path, triedPassword)
        sendFills()
    }
    // What is filled, to PdfDrawing first and then to the pages' images, so none is drawn with less.
    function sendFills() {
        if (!drawingKey.length) return
        const list = Object.values(fills)
        PdfDrawing.fill(drawingKey, list)
        shownFills = list.length ? ++fillings : 0
    }
    onFillsChanged: sendFills()
    // Lets go of the file the pages are drawn from.
    function stopDrawing() {
        if (drawingKey.length) PdfDrawing.close(drawingKey)
        drawingKey = ""
    }
    Component.onDestruction: stopDrawing()
    // The keyboard back to the pages, from a field, a button or a list, what was typed in a form's field kept. Focusing
    // the reader itself would hand it back to whichever of them had it last, since the reader is a focus scope.
    function rest() {
        keepTyping()
        fieldFocus = null
        resting.forceActiveFocus()
    }
    Item { id: resting; focus: true }
    function closeFinder() {
        finder.visible = false
        reader.rest()
    }
    function say(words) {
        notice = words
        noticeTimer.restart()
    }
    function unlock() {
        const words = password.text
        password.clear()
        if (!words.length) return
        if (words === triedPassword) {
            wrongPassword = true
            return
        }
        triedPassword = words
        doc.password = words
    }

    // Changing the pages: each change is a new list, the one before it kept for undo.
    function identity() {
        const list = []
        for (let i = 0; i < doc.pageCount; ++i) list.push({ page: i, turn: 0 })
        return list
    }
    // Whether the pages, as they show, differ from the file's.
    property bool reordered: false
    function relayout() {
        let w = 1
        let same = layout.length === doc.pageCount
        for (let i = 0; i < layout.length; ++i) {
            const s = doc.pagePointSize(layout[i].page)
            w = Math.max(w, layout[i].turn % 2 ? s.height : s.width)
            same = same && layout[i].page === i && layout[i].turn === 0
        }
        widest = w
        reordered = !same
        updateDirty()
        pages.forceLayout()
        if (fitting.length) fit()
    }
    function updateDirty() {
        dirty = reordered || added.length > 0 || removed.length > 0 || Object.keys(fills).length > 0 || typedUnkept()
    }
    // What undo and redo step between: the pages, the marks and what is filled, together.
    function snapshot() {
        return { layout: layout, added: added, removed: removed, fills: fills }
    }
    function restore(state) {
        const moved = state.layout !== layout
        layout = state.layout
        added = state.added
        removed = state.removed
        fills = state.fills
        chosen = null
        relayout()
        if (moved) goTo(Math.min(page, layout.length - 1))
    }
    function record() {
        undoStack = undoStack.concat([snapshot()])
        redoStack = []
    }
    function change(next, then) {
        if (!editable) {
            say(locked || triedPassword.length ? qsTr("A locked PDF is read here, not changed.") : qsTr("This PDF's pages cannot be changed here."))
            return
        }
        record()
        layout = next
        relayout()
        goTo(then)
    }
    function turnPage(by) {
        if (!ready) return
        const next = layout.slice()
        next[page] = { page: layout[page].page, turn: (layout[page].turn + by + 4) % 4 }
        change(next, page)
    }
    function deletePage() {
        if (!ready) return
        if (layout.length === 1) {
            say(qsTr("A PDF keeps at least one page."))
            return
        }
        const gone = page
        change(layout.filter((_, i) => i !== gone), Math.min(gone, layout.length - 2))
    }
    function movePage(by) {
        const to = page + by
        if (!ready || to < 0 || to >= layout.length) return
        const next = layout.slice()
        next.splice(to, 0, next.splice(page, 1)[0])
        change(next, to)
    }
    function undo() {
        if (!undoStack.length) return
        redoStack = [snapshot()].concat(redoStack)
        const state = undoStack[undoStack.length - 1]
        undoStack = undoStack.slice(0, -1)
        restore(state)
    }
    function redo() {
        if (!redoStack.length) return
        undoStack = undoStack.concat([snapshot()])
        const state = redoStack[0]
        redoStack = redoStack.slice(1)
        restore(state)
    }

    // Marking up. Each change is a step undone and redone with the pages' own, written only when the PDF is saved.
    function canMark() {
        if (editable) return true
        say(locked || triedPassword.length ? qsTr("A locked PDF is read here, not marked.") : qsTr("This PDF cannot be marked here."))
        return false
    }
    function markChange(nextAdded, nextRemoved) {
        if (!canMark()) return false
        record()
        added = nextAdded
        removed = nextRemoved
        updateDirty()
        return true
    }
    function addMark(mark) {
        mark.id = ++markIds
        if (!markChange(added.concat([mark]), removed)) return
        chosen = mark.kind === "note" || mark.kind === "signature" ? { from: "added", id: mark.id } : null
    }
    function addedMark(id) {
        return added.find((m) => m.id === id)
    }
    function replaced(id, change) {
        return added.map((m) => m.id === id ? Object.assign({}, m, change) : m)
    }
    // The PDF's own marks on a page of the file, read the first time the page is asked for. `held` starts afresh with each
    // file and each save, and what reads it reads again.
    function heldOn(source) {
        if (!(source in held)) held[source] = PdfMarks.read(path, triedPassword, source)
        return held[source]
    }
    function isRemoved(own) {
        return removed.some((r) => r.page === own.page && r.index === own.index)
    }
    function isChosenAdded(id) {
        return chosen !== null && chosen.from === "added" && chosen.id === id
    }
    function isChosenHeld(own) {
        return chosen !== null && chosen.from === "held" && chosen.page === own.page && chosen.index === own.index
    }
    function toggleMarking() {
        if (!ready || !editable) return
        marking = !marking
        if (!marking) {
            tool = ""
            chosen = null
        }
    }
    function markWords(kind) {
        if (!ready) return
        const rects = selecting ? PdfMarks.rectsOf(selecting.geometry) : []
        if (!rects.length) {
            say(qsTr("Select words first, by dragging across them."))
            return
        }
        addMark({ kind: kind, page: selecting.page, colour: wordsColour, rects: rects })
        selecting.clear()
        selectedText = ""
    }
    // A point of the view (in its pixels) on the page in view, in the page's points, kept on the page.
    function onPage(viewX, viewY, width, height) {
        const item = pageItem(page)
        const size = doc.pagePointSize(currentSource)
        const at = item ? item.paper.mapFromItem(pages, viewX, viewY) : Qt.point(0, 0)
        return Qt.point(Math.max(0, Math.min(size.width - width, at.x / zoom)), Math.max(0, Math.min(size.height - height, at.y / zoom)))
    }
    // A mark placed by hand, kept on its page.
    function keptOn(mark, x, y, width) {
        const size = doc.pagePointSize(mark.page)
        const signature = mark.kind === "signature"
        const w = signature ? width : PdfMarks.noteSize
        const h = signature ? width * mark.height / mark.width : PdfMarks.noteSize
        const kept = { x: Math.max(0, Math.min(size.width - w, x)), y: Math.max(0, Math.min(size.height - h, y)) }
        if (signature) {
            kept.width = w
            kept.height = h
        }
        return kept
    }
    function placeNote() {
        if (!ready) return
        if (!canMark()) return
        const at = onPage(32, 32, PdfMarks.noteSize, PdfMarks.noteSize)
        noteDraft = { kind: "note", page: currentSource, x: at.x, y: at.y, colour: wordsColour, text: "" }
        openNote({ from: "draft" }, "")
    }
    function openNote(source, words) {
        noteEditor.source = source
        noteText.text = words
        noteEditor.visible = true
        if (source.from !== "held") noteText.forceActiveFocus()
    }
    function closeNote(keep) {
        const source = noteEditor.source
        const words = noteText.text
        const draft = noteDraft
        noteEditor.visible = false
        noteEditor.source = null
        noteDraft = null
        if (keep && source && source.from === "draft" && words.trim().length)
            addMark({ kind: "note", page: draft.page, x: draft.x, y: draft.y, colour: draft.colour, text: words })
        else if (keep && source && source.from === "added" && addedMark(source.id) && addedMark(source.id).text !== words)
            markChange(replaced(source.id, { text: words }), removed)
        rest()
    }
    function removeNote() {
        const source = noteEditor.source
        closeNote(false)
        if (!source) return
        chosen = source.from === "added" ? { from: "added", id: source.id } : { from: "held", page: source.mark.page, index: source.mark.index }
        removeChosen()
    }
    function toggleInk() {
        if (!ready) return
        if (!canMark()) return
        tool = tool === "ink" ? "" : "ink"
        chosen = null
        if (tool === "ink") marking = true
    }
    // A line drawn by hand: points in the page's points, one at least three quarters of a point from the last.
    function inkFrom(source, at) {
        inking = { page: source, points: [Qt.point(at.x / zoom, at.y / zoom)] }
    }
    function inkTo(at) {
        if (!inking) return
        const p = Qt.point(at.x / zoom, at.y / zoom)
        const last = inking.points[inking.points.length - 1]
        if (Math.hypot(p.x - last.x, p.y - last.y) < 0.75) return
        inking = { page: inking.page, points: inking.points.concat([p]) }
    }
    function inkDone() {
        const line = inking
        inking = null
        if (!line) return
        addMark({ kind: "ink", page: line.page, colour: penColour, width: 2,
                  strokes: [line.points.length > 1 ? line.points : [line.points[0], line.points[0]]] })
    }
    // A signature made on the pad, for the page in view or for the signature field `into`.
    function sign(into) {
        if (!ready) return
        if (!canMark()) return
        signTarget = into || null
        tool = ""
        signing.colour = penColour
        signing.open()
    }
    // Placed in the middle of the page in view, no wider than 180 points and no higher than 72; or in the signature field
    // it was made for, as large as the field takes, in its middle.
    function placeSignature(signature) {
        const into = signTarget
        signTarget = null
        let width = into ? Math.min(into.rect.width, into.rect.height * signature.aspect) : Math.min(180, doc.pagePointSize(currentSource).width * 0.35)
        if (!into && width / signature.aspect > 72) width = 72 * signature.aspect
        const height = width / signature.aspect
        const at = into ? Qt.point(into.rect.x + into.rect.width / 2, into.rect.y + into.rect.height / 2) : onPage(pages.width / 2, pages.height / 2, 0, 0)
        const placed = { kind: "signature", page: into ? into.page : currentSource, colour: signature.colour, x: at.x - width / 2,
                         y: at.y - height / 2, width: width, height: height }
        Object.assign(placed, keptOn(placed, placed.x, placed.y, width))
        if (signature.strokes) {
            placed.strokes = signature.strokes
            placed.line = signature.line
        } else {
            placed.shapes = signature.shapes
        }
        penColour = signature.colour
        addMark(placed)
        say(into ? qsTr("A picture of your signature, in its field: it does not certify the PDF.")
                 : qsTr("A picture of your signature: it does not certify the PDF. Drag it into place."))
        rest()
    }
    // Choosing a mark, moving it, sizing it and taking it away.
    function chooseAdded(id) {
        const mark = addedMark(id)
        if (!mark) return
        chosen = { from: "added", id: id }
        if (mark.kind === "note") openNote({ from: "added", id: id }, mark.text)
    }
    function chooseHeld(own) {
        chosen = { from: "held", page: own.page, index: own.index }
        if (own.kind === "note") openNote({ from: "held", mark: own }, own.text)
    }
    // A tap on a page, not on a mark (a mark keeps its taps to itself), lets go of what was chosen and of a note being
    // read; a note being written stays.
    function tappedPage() {
        if (noteEditor.visible && !noteEditor.writable) closeNote(false)
        if (!noteEditor.visible) chosen = null
    }
    function moveStart(id) {
        const mark = addedMark(id)
        if (!mark) return
        chosen = { from: "added", id: id }
        dragged = { id: id, dx: 0, dy: 0, width: 0 }
    }
    function moveBy(id, dx, dy) {
        if (dragged && dragged.id === id) dragged = { id: id, dx: dx, dy: dy, width: dragged.width }
    }
    function sizeTo(id, width) {
        if (dragged && dragged.id === id) dragged = { id: id, dx: dragged.dx, dy: dragged.dy, width: Math.max(24, width) }
    }
    function moveEnd(id) {
        const step = dragged
        dragged = null
        const mark = addedMark(id)
        if (!step || step.id !== id || !mark || (!step.dx && !step.dy && !step.width)) return
        markChange(replaced(id, keptOn(mark, mark.x + step.dx, mark.y + step.dy, step.width || mark.width)), removed)
    }
    // The arrows move a chosen note or signature by a point, or ten with Shift; [ and ] size a signature.
    function nudge(dx, dy) {
        const mark = chosen && chosen.from === "added" ? addedMark(chosen.id) : null
        if (!mark || (mark.kind !== "note" && mark.kind !== "signature")) return false
        markChange(replaced(mark.id, keptOn(mark, mark.x + dx, mark.y + dy, mark.width)), removed)
        return true
    }
    function resizeChosen(by) {
        const mark = chosen && chosen.from === "added" ? addedMark(chosen.id) : null
        if (!mark || mark.kind !== "signature") return
        const width = Math.max(24, mark.width * by)
        markChange(replaced(mark.id, keptOn(mark, mark.x + (mark.width - width) / 2, mark.y + (mark.height - width * mark.height / mark.width) / 2, width)), removed)
    }
    function removeChosen() {
        if (!chosen) return false
        const gone = chosen
        chosen = null
        if (noteEditor.visible) closeNote(false)
        if (gone.from === "added") markChange(added.filter((m) => m.id !== gone.id), removed)
        else if (!isRemoved(gone)) markChange(added, removed.concat([{ page: gone.page, index: gone.index }]))
        return true
    }

    // Filling a form. Each field has one entry at most, naming the widget it was filled through, and none while it holds
    // what the file has, so filling a field back as it was leaves nothing to save.
    function fieldsOn(source) {
        if (!(source in fields)) {
            const list = PdfForms.read(path, triedPassword, source)
            fields[source] = list
            // Said once, when a page with a form shows: later, since the pages' bindings read this.
            const xfa = list.find((f) => f.xfa)
            if (xfa) Qt.callLater(reader.sayForm, xfa.why)
            else if (list.some((f) => f.fillable)) Qt.callLater(reader.sayForm, qsTr("A form: tap a field to fill it, or Tab through its fields."))
        }
        return fields[source]
    }
    function sayForm(words) {
        if (formSaid || !editable) return
        formSaid = true
        say(words)
    }
    // The keyboard on a field: `mode` "", "typing" or "listing"; `keys` whether the keys brought it there, which shows
    // the focus ring, as it shows for the keyboard alone. Left out, it stays as it was.
    function focusOn(spot, mode, keys) {
        fieldFocus = { page: spot.page, index: spot.index, mode: mode, keys: keys === undefined ? fieldFocus !== null && fieldFocus.keys : keys }
    }
    function fieldKey(f) {
        return f.name.length ? f.name : f.page + "/" + f.index
    }
    function fieldAt(spot) {
        return spot ? fieldsOn(spot.page).find((f) => f.index === spot.index) : undefined
    }
    function isFieldFocus(f, mode) {
        return fieldFocus !== null && fieldFocus.page === f.page && fieldFocus.index === f.index && (mode === undefined || fieldFocus.mode === mode)
    }
    // What a field holds now: what the person filled in, or the file's.
    function wordsOf(f) {
        const entry = fills[fieldKey(f)]
        if (entry && entry.text !== undefined) return entry.text
        if (entry && entry.chosen !== undefined && entry.chosen.length) return f.options[entry.chosen[0]]
        return f.value
    }
    function chosenOf(f) {
        const entry = fills[fieldKey(f)]
        if (entry && entry.chosen !== undefined) return entry.chosen
        return entry && entry.text !== undefined ? [] : f.chosen
    }
    function checkedOf(f) {
        const entry = fills[fieldKey(f)]
        return entry && entry.checked !== undefined ? entry.checked && entry.state === f.state : f.checked
    }
    function canFill(f) {
        if (!editable) {
            say(locked || triedPassword.length ? qsTr("A locked PDF is read here, not filled.") : qsTr("This PDF cannot be filled here."))
            return false
        }
        if (f.fillable || f.kind === "signature") return true
        say(f.why)
        return false
    }
    function fill(f, value) {
        const key = fieldKey(f)
        const asFiled = value.text !== undefined ? value.text === f.value
            : value.checked !== undefined ? value.checked === f.checked
            : value.chosen.join(",") === f.chosen.join(",")
        const next = Object.assign({}, fills)
        if (asFiled) delete next[key]
        else next[key] = Object.assign({ page: f.page, index: f.index, state: f.state }, value)
        if (JSON.stringify(next) === JSON.stringify(fills)) return
        record()
        fills = next
        updateDirty()
    }
    // A tap on a field, or Space or Return while the keyboard is on it (`byKeys`).
    function activate(f, byKeys) {
        keepTyping()
        if (!canFill(f)) return
        if (f.kind === "signature") {
            sign(f)
            return
        }
        focusOn(f, f.kind === "combo" || f.kind === "list" ? "listing" : "", byKeys)
        if (f.kind === "text") startTyping(f)
        else if (f.kind === "check") fill(f, { checked: !checkedOf(f) })
        else if (f.kind === "radio" && !checkedOf(f)) fill(f, { checked: true })
    }
    function startTyping(f) {
        typingWords = wordsOf(f)
        focusOn(f, "typing")
    }
    function typedUnkept() {
        const f = fieldFocus !== null && fieldFocus.mode === "typing" ? fieldAt(fieldFocus) : undefined
        return f !== undefined && typingWords !== wordsOf(f)
    }
    function edited(text) {
        typingWords = text
        updateDirty()
    }
    // The words typed kept in their field, which keeps the keyboard; or, by Esc, left as they were.
    function keepTyping() {
        if (fieldFocus === null || fieldFocus.mode !== "typing") return
        const f = fieldAt(fieldFocus)
        focusOn(fieldFocus, "")
        if (f) fill(f, { text: typingWords })
        updateDirty()
    }
    function leaveTyping() {
        if (fieldFocus === null || fieldFocus.mode !== "typing") return
        focusOn(fieldFocus, "")
        updateDirty()
    }
    function chooseFrom(f, places) {
        fill(f, { chosen: places })
        // A list that takes several stays open for the next.
        if (!(f.kind === "list" && f.multiple)) closeOptions()
    }
    function writeChoice(f, text) {
        fill(f, { text: text })
        closeOptions()
    }
    function closeOptions() {
        if (fieldFocus !== null && fieldFocus.mode === "listing") focusOn(fieldFocus, "")
    }
    // Tab and Shift+Tab: the next field to fill on this page, or on the pages after it as they are laid out; past the
    // last, the keyboard goes back to the pages. What was typed is kept first.
    function stepField(from, back) {
        keepTyping()
        const start = rowOf(from.page)
        for (let row = start; row >= 0 && row < layout.length; row += back ? -1 : 1) {
            let list = fieldsOn(entryAt(row).page).filter((f) => f.fillable)
            if (back) list = list.reverse()
            const next = row !== start ? list[0] : list.find((f) => back ? f.index < from.index : f.index > from.index)
            if (next) {
                focusField(next, row)
                return
            }
        }
        rest()
    }
    // Tab with the keyboard on the pages: the first field to fill on the page in view.
    function enterFields() {
        const first = ready && editable ? fieldsOn(currentSource).find((f) => f.fillable) : undefined
        if (first) focusField(first, page)
        return first !== undefined
    }
    // The keyboard on a field, in view with room around it; a text field is typed in at once.
    function focusField(f, row) {
        let item = pageItem(row)
        if (!item) {
            goTo(row)
            item = pageItem(row)
        }
        if (item) {
            const r = item.paper.mapToItem(item, f.rect.x * zoom, f.rect.y * zoom, f.rect.width * zoom, f.rect.height * zoom)
            pages.positionViewAtCell(Qt.point(0, row), TableView.Contain, Qt.point(0, 0),
                                     Qt.rect(Math.max(0, r.x - 24), Math.max(0, r.y - 24), r.width + 48, r.height + 48))
            pages.forceLayout()
        }
        page = row
        focusOn(f, "", true)
        if (f.kind === "text") startTyping(f)
    }
    // Write the change to the file. Qt PDF holds the file it shows, so a copy is shown while the file is replaced; then
    // the file is opened again, at the same page. On failure the change stays, unsaved, and the file is as it was.
    //
    // In two steps: the pages are let go first, and the file is swapped on the next turn of the event loop, once they are
    // gone, since a page let go of lingers a moment and would otherwise follow the document to the copy. `then`, when
    // given, hears whether it was saved.
    property var saveJob: null
    function save(then) {
        const done = then || (() => {})
        if (saveJob) return
        keepTyping()
        if (!dirty) return done(true)
        if (!editable) {
            say(qsTr("This PDF's pages cannot be changed here."))
            return done(false)
        }
        const copy = writer.copyForSaving(path)
        if (!copy.toString().length) {
            say(qsTr("There was no room to save. The PDF is as it was."))
            return done(false)
        }
        saveJob = { copy: copy, entries: layout.map((e) => ({ page: e.page, turn: e.turn })), added: added, removed: removed,
                    filled: Object.values(fills), then: done }
        returnTo = page
        swapping = true
        // The view lets its pages go now rather than at its next frame, so they are gone by the next turn.
        pages.forceLayout()
        swapLater.restart()
    }
    function finishSaving() {
        const job = saveJob
        stopDrawing()
        saving = true
        doc.source = job.copy
        const saved = writer.save(path, job.entries, job.added, job.removed, job.filled)
        saving = false
        keepLayout = !saved
        if (saved) {
            added = []
            removed = []
            fills = ({})
            chosen = null
            reordered = false
            dirty = false
            undoStack = []
            redoStack = []
        }
        doc.source = Files.urlOf(path)
        swapping = false
        writer.discard(job.copy)
        say(saved ? qsTr("Saved") : writer.error)
        // What follows a save (going back, opening another file) waits for the pages to come back first.
        saveJob = { then: job.then, saved: saved }
        afterSaving.restart()
    }
    Timer { id: swapLater; interval: 0; onTriggered: reader.finishSaving() }
    Timer {
        id: afterSaving
        interval: 0
        onTriggered: {
            const job = reader.saveJob
            reader.saveJob = null
            job.then(job.saved)
        }
    }
    // Called by the window before it goes back or opens another file, and by the back button.
    function ask(then) {
        afterAsk = then
        asking.visible = true
        saveChoice.forceActiveFocus()
    }
    function goBack() {
        keepTyping()
        if (dirty) ask(() => reader.back())
        else reader.back()
    }
    function answered(choice) {
        // Saved first, then gone; a save that fails keeps the question, and the error, on screen.
        if (choice === "save") {
            save((saved) => { if (saved) reader.answered("saved") })
            return
        }
        if (choice === "discard") {
            layout = identity()
            added = []
            removed = []
            fills = ({})
            fieldFocus = null
            chosen = null
            undoStack = []
            redoStack = []
            relayout()
        }
        asking.visible = false
        const then = afterAsk
        afterAsk = null
        if (choice === "cancel") rest()
        else if (then) then()
    }

    // Open `path` afresh: nothing of the last PDF stays, its password least of all.
    function load() {
        stopDrawing()
        finder.visible = false
        query.clear()
        if (selecting) selecting.clear()
        selecting = null
        selectedText = ""
        locked = false
        wrongPassword = false
        triedPassword = ""
        hasContents = false
        layout = []
        undoStack = []
        redoStack = []
        added = []
        removed = []
        held = ({})
        fields = ({})
        fills = ({})
        fieldFocus = null
        typingWords = ""
        formSaid = false
        signTarget = null
        chosen = null
        tool = ""
        inking = null
        dragged = null
        noteDraft = null
        noteEditor.visible = false
        noteEditor.source = null
        signing.visible = false
        reordered = false
        dirty = false
        editable = false
        keepLayout = false
        returnTo = -1
        page = 0
        fitting = "width"
        doc.password = ""
        doc.source = Files.urlOf(path)
    }
    property bool started: false
    Component.onCompleted: { started = true; load() }
    onPathChanged: if (started) load()

    PdfPages { id: writer }

    PdfDocument {
        id: doc
        onPasswordRequired: {
            reader.wrongPassword = reader.triedPassword.length > 0
            reader.locked = true
            password.forceActiveFocus()
        }
        onStatusChanged: {
            // The copy shown while saving changes nothing.
            if (reader.saving) return
            if (doc.status === PdfDocument.Ready) {
                reader.locked = false
                reader.wrongPassword = false
                if (!reader.keepLayout) reader.layout = reader.identity()
                reader.keepLayout = false
                reader.editable = !reader.triedPassword.length && writer.changeable(reader.path)
                if (!reader.editable) reader.marking = false
                reader.held = ({})
                reader.fields = ({})
                reader.draw()
                reader.relayout()
                const back = reader.returnTo
                reader.returnTo = -1
                reader.rest()
                settling.at = Math.max(0, back)
                settling.restart()
            } else if (doc.status === PdfDocument.Error && reader.locked && reader.triedPassword.length) {
                reader.wrongPassword = true
            }
        }
    }
    function settle(at) {
        page = Math.min(at, Math.max(0, pageCount - 1))
        if (fitting.length) fit()
        else goTo(page)
    }
    // Work left for the next turn of the event loop, on timers of the reader's own, which go with it if it goes first.
    Timer { id: settling; interval: 0; property int at: 0; onTriggered: { reader.settle(at); reader.readContents() } }
    Timer { id: refitting; interval: 0; onTriggered: reader.fitting.length ? reader.fit() : pages.forceLayout() }

    // What follows the page in view: the page's number, unless it is being typed, and the page among the small ones.
    onPageChanged: {
        if (!pageField.activeFocus) pageField.text = String(page + 1)
        thumbnails.currentIndex = page
    }

    PdfBookmarkModel { id: bookmarks; document: doc }
    property bool hasContents: false
    function readContents() {
        hasContents = ready && bookmarks.rowCount() > 0
        if (hasContents) contents.expandRecursively()
    }

    PdfSearchModel {
        id: search
        document: doc
        searchString: finder.visible ? query.text : ""
        onCurrentResultChanged: reader.showMatch()
    }
    // The search model's own count is not offered to QML in this Qt (its revision is the base class's), so the matches
    // are counted from its rows as it finds them. A new search goes to its first match on or after the page in view.
    property int matches: 0
    property bool freshSearch: false
    function countMatches() {
        matches = search.rowCount()
        if (matches > 0 && freshSearch) {
            freshSearch = false
            const first = findFrom(page)
            if (search.currentResult === first) showMatch()
            else search.currentResult = first
        }
    }
    Connections {
        target: search
        function onRowsInserted() { reader.countMatches() }
        function onRowsRemoved() { reader.countMatches() }
        function onModelReset() { reader.countMatches() }
    }

    Timer { id: noticeTimer; interval: 2400; onTriggered: reader.notice = "" }

    Rectangle { anchors.fill: parent; color: Theme.void_ }

    TopBar {
        id: bar
        anchors { left: parent.left; right: parent.right; top: parent.top }
        title: (reader.dirty ? "● " : "") + Files.nameOf(reader.path)
        subtitle: reader.locked ? qsTr("Locked") : reader.unreadable ? "" : [doc.title.length && doc.title !== Files.nameOf(reader.path) ? doc.title : "", reader.pageCount ? qsTr("Page %1 of %2").arg(reader.page + 1).arg(reader.pageCount) : "", reader.notice].filter(Boolean).join(" · ")
        color: Theme.base
        onBack: reader.goBack()
        IconButton { iconName: "pages"; visible: reader.ready; active: reader.panel; ToolTip.text: qsTr("Pages and contents (F9)"); onClicked: reader.panel = !reader.panel }
        IconButton { iconName: "find"; visible: reader.ready; active: finder.visible; ToolTip.text: qsTr("Find (Ctrl+F)"); onClicked: finder.visible ? reader.closeFinder() : reader.openFinder() }
        IconButton { iconName: "zoomout"; visible: reader.ready; ToolTip.text: qsTr("Zoom out (-)"); onClicked: reader.zoomBy(0.8, pages.width / 2, pages.height / 2) }
        IconButton { iconName: "zoomin"; visible: reader.ready; ToolTip.text: qsTr("Zoom in (+)"); onClicked: reader.zoomBy(1.25, pages.width / 2, pages.height / 2) }
        IconButton { iconName: "fit"; visible: reader.ready; ToolTip.text: reader.fitting === "width" ? qsTr("Whole page (0)") : qsTr("Fit the width (0)"); onClicked: reader.cycleFit() }
        IconButton { objectName: "markUp"; iconName: "markup"; visible: reader.ready && reader.editable; active: reader.marking; ToolTip.text: qsTr("Mark up (M)"); onClicked: reader.toggleMarking() }
        IconButton { objectName: "turnPage"; iconName: "rotate"; visible: reader.ready && reader.editable; ToolTip.text: qsTr("Turn the page (R)"); onClicked: reader.turnPage(1) }
        IconButton { objectName: "deletePage"; iconName: "delete"; visible: reader.ready && reader.editable; ToolTip.text: qsTr("Delete the page (Delete)"); onClicked: reader.deletePage() }
        IconButton { iconName: "undo"; visible: reader.undoStack.length > 0; ToolTip.text: qsTr("Undo (Ctrl+Z)"); onClicked: reader.undo() }
        IconButton { iconName: "save"; visible: reader.dirty; ToolTip.text: qsTr("Save (Ctrl+S)"); onClicked: reader.save() }
        IconButton { iconName: "full"; ToolTip.text: qsTr("Full screen (F)"); onClicked: reader.fullScreenToggled() }
        IconButton { iconName: "open"; ToolTip.text: qsTr("Open with another program"); onClicked: Files.openElsewhere(reader.path) }
    }

    MarkupBar {
        id: markupBar
        visible: reader.marking && reader.ready && reader.editable
        height: visible ? implicitHeight : 0
        implicitHeight: 52
        anchors { left: parent.left; right: parent.right; top: bar.bottom }
        hasWords: reader.selectedText.length > 0
        drawing: reader.tool === "ink"
        hasChosen: reader.chosen !== null
        wordsColour: reader.wordsColour
        penColour: reader.penColour
        hint: reader.markHint
        onMarked: (kind) => reader.markWords(kind)
        onNoted: reader.placeNote()
        onDrew: reader.toggleInk()
        onSigned: reader.sign()
        onRemoved: reader.removeChosen()
        onColoured: (colour) => {
            if (reader.tool === "ink") reader.penColour = colour
            else reader.wordsColour = colour
        }
    }

    // Beside the pages: each page small, and the PDF's contents when it has them.
    Rectangle {
        id: side
        visible: reader.panel && reader.ready
        width: visible ? 216 : 0
        anchors { left: parent.left; top: markupBar.bottom; bottom: status.top }
        color: Theme.base

        RowLayout {
            id: tabs
            anchors { left: parent.left; right: parent.right; top: parent.top; margins: 8 }
            spacing: 4
            Repeater {
                model: reader.hasContents ? [["pages", qsTr("Pages")], ["contents", qsTr("Contents")]] : []
                delegate: TextButton {
                    id: tab
                    required property var modelData
                    objectName: modelData[0] + "Tab"
                    text: modelData[1]
                    Layout.fillWidth: true
                    onClicked: reader.panelTab = modelData[0]
                    // The tab that is showing is underlined in the accent.
                    Rectangle {
                        anchors { left: tab.left; right: tab.right; bottom: tab.bottom }
                        height: 2
                        color: Theme.accent
                        visible: reader.panelTab === tab.modelData[0]
                    }
                }
            }
        }

        ListView {
            id: thumbnails
            objectName: "thumbnails"
            visible: reader.panelTab === "pages" || !reader.hasContents
            anchors { left: parent.left; right: parent.right; top: tabs.bottom; bottom: moving.top; topMargin: 8 }
            clip: true
            model: reader.pageCount
            spacing: 12
            activeFocusOnTab: true
            keyNavigationEnabled: true
            highlightFollowsCurrentItem: false
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar { }
            onCurrentIndexChanged: positionViewAtIndex(currentIndex, ListView.Contain)
            Keys.onReturnPressed: reader.goTo(currentIndex)
            Keys.onEnterPressed: reader.goTo(currentIndex)
            delegate: Item {
                id: thumb
                required property int index
                readonly property var entry: reader.entryAt(index)
                readonly property size points: doc.pagePointSize(entry.page)
                readonly property bool sideways: entry.turn % 2 === 1
                width: thumbnails.width
                height: sheet.height + label.height + 6
                Item {
                    id: sheet
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 132
                    height: Math.round(width * (thumb.sideways ? thumb.points.width : thumb.points.height) / Math.max(1, thumb.sideways ? thumb.points.height : thumb.points.width))
                    // The paper is a Rectangle's own white, the colour a PDF is drawn on; turned as the page is.
                    Rectangle {
                        anchors.centerIn: parent
                        width: thumb.sideways ? sheet.height : sheet.width
                        height: thumb.sideways ? sheet.width : sheet.height
                        rotation: thumb.entry.turn * 90
                        Image {
                            anchors.fill: parent
                            source: reader.pageImage(thumb.entry.page)
                            asynchronous: true
                            fillMode: Image.PreserveAspectFit
                            sourceSize.width: width * Screen.devicePixelRatio
                        }
                    }
                    Rectangle {
                        anchors.fill: parent
                        color: "transparent"
                        border.width: thumb.index === reader.page || thumbnails.activeFocus && thumb.index === thumbnails.currentIndex ? 2 : 0
                        border.color: thumb.index === reader.page ? Theme.accent : Theme.focus
                    }
                }
                Text {
                    id: label
                    anchors { top: sheet.bottom; topMargin: 6; horizontalCenter: parent.horizontalCenter }
                    // A page's own label (iv, A-3) while the pages are as the file has them; its place once they change.
                    text: reader.dirty ? String(thumb.index + 1) : (doc.pageLabel(thumb.entry.page) || String(thumb.index + 1))
                    color: thumb.index === reader.page ? Theme.ink : Theme.inkMuted
                    font.family: Theme.text
                    font.pixelSize: Theme.sizeCaption
                }
                TapHandler { onTapped: { thumbnails.currentIndex = thumb.index; reader.goTo(thumb.index) } }
            }
        }

        // Moving the page in view, under the small pages.
        RowLayout {
            id: moving
            visible: reader.editable && thumbnails.visible
            height: visible ? implicitHeight : 0
            anchors { left: parent.left; right: parent.right; bottom: parent.bottom; margins: 8 }
            spacing: 4
            Text {
                Layout.fillWidth: true
                text: qsTr("Move the page")
                color: Theme.inkMuted
                font.family: Theme.text
                font.pixelSize: Theme.sizeUi
            }
            IconButton { objectName: "moveEarlier"; iconName: "up"; size: 32; enabled: reader.page > 0; ToolTip.text: qsTr("Earlier (Alt+Up)"); onClicked: reader.movePage(-1) }
            IconButton { objectName: "moveLater"; iconName: "down"; size: 32; enabled: reader.page < reader.pageCount - 1; ToolTip.text: qsTr("Later (Alt+Down)"); onClicked: reader.movePage(1) }
        }

        TreeView {
            id: contents
            objectName: "contents"
            visible: reader.panelTab === "contents" && reader.hasContents
            anchors { left: parent.left; right: parent.right; top: tabs.bottom; bottom: parent.bottom; topMargin: 8 }
            clip: true
            activeFocusOnTab: true
            boundsBehavior: Flickable.StopAtBounds
            model: bookmarks
            // The keyboard's place in the contents: the arrows move it (Left and Right close and open), Return goes there.
            selectionModel: ItemSelectionModel { model: bookmarks }
            onActiveFocusChanged: if (activeFocus && !selectionModel.currentIndex.valid) selectionModel.setCurrentIndex(index(0, 0), ItemSelectionModel.NoUpdate)
            function openCurrent() {
                const at = selectionModel.currentIndex
                // QPdfBookmarkModel::Role::Page and ::Location.
                if (at.valid) reader.follow({ page: bookmarks.data(at, 258), location: bookmarks.data(at, 259) })
            }
            Keys.onReturnPressed: openCurrent()
            Keys.onEnterPressed: openCurrent()
            ScrollBar.vertical: ScrollBar { }
            delegate: Item {
                id: entry
                required property TreeView treeView
                required property bool expanded
                required property bool hasChildren
                required property bool current
                required property int depth
                required property int row
                required property string title
                required property int page
                required property point location
                implicitWidth: contents.width
                implicitHeight: 32
                Rectangle {
                    anchors.fill: parent
                    color: entry.page === reader.currentSource ? Theme.raised : hover.hovered ? Theme.surface : "transparent"
                    border.width: entry.current && contents.activeFocus ? 2 : 0
                    border.color: Theme.focus
                }
                Text {
                    id: arrow
                    x: 8 + entry.depth * 14
                    anchors.verticalCenter: parent.verticalCenter
                    visible: entry.hasChildren
                    text: "›"
                    rotation: entry.expanded ? 90 : 0
                    color: Theme.inkMuted
                    font.family: Theme.ui
                    font.pixelSize: Theme.sizeUi
                    TapHandler { onTapped: entry.treeView.toggleExpanded(entry.row) }
                }
                Text {
                    anchors { left: arrow.right; leftMargin: 6; right: parent.right; rightMargin: 8; verticalCenter: parent.verticalCenter }
                    text: entry.title
                    elide: Text.ElideRight
                    color: entry.page === reader.currentSource ? Theme.ink : Theme.inkBody
                    font.family: Theme.text
                    font.pixelSize: Theme.sizeUi
                }
                HoverHandler { id: hover }
                TapHandler { onTapped: reader.follow({ page: entry.page, location: entry.location }) }
            }
        }

        Rectangle { anchors { right: parent.right; top: parent.top; bottom: parent.bottom } width: 1; color: Theme.edge }
    }

    TableView {
        id: pages
        objectName: "pages"
        visible: reader.ready
        anchors { left: side.right; right: parent.right; top: markupBar.bottom; bottom: status.top }
        clip: true
        model: reader.pageCount
        // Drawing by hand, a drag draws rather than scrolls.
        interactive: reader.tool !== "ink"
        rowSpacing: 12
        topMargin: reader.margin
        bottomMargin: reader.margin
        reuseItems: false
        animate: false
        keyNavigationEnabled: false
        activeFocusOnTab: false
        pointerNavigationEnabled: false
        boundsBehavior: Flickable.StopAtBounds
        columnWidthProvider: () => Math.max(pages.width, reader.widest * reader.zoom + 2 * reader.margin)
        rowHeightProvider: (row) => reader.laidOut(row).height * reader.zoom
        ScrollBar.vertical: ScrollBar { }
        ScrollBar.horizontal: ScrollBar { }
        onContentYChanged: reader.track()
        onWidthChanged: refitting.restart()
        onHeightChanged: if (reader.fitting === "page") refitting.restart()

        delegate: Item {
            id: holder
            required property int row
            readonly property var entry: reader.entryAt(row)
            readonly property size points: doc.pagePointSize(entry.page)
            property alias selection: selection
            property alias paper: paper

            // The paper: a Rectangle's own white, the colour a PDF is drawn on, on the edge it sits on; turned as the page is,
            // with everything on it.
            Rectangle {
                id: paper
                anchors.centerIn: parent
                width: holder.points.width * reader.zoom
                height: holder.points.height * reader.zoom
                rotation: holder.entry.turn * 90

                Rectangle {
                    z: -1
                    anchors.fill: parent
                    anchors.margins: -1
                    color: Theme.edge
                }

                // Drawn with its marks (PdfDrawing): Qt PDF's own page image leaves them out.
                Image {
                    id: image
                    objectName: "pageImage"
                    anchors.fill: parent
                    source: reader.pageImage(holder.entry.page)
                    asynchronous: true
                    fillMode: Image.PreserveAspectFit
                    sourceSize.width: paper.width * Screen.devicePixelRatio
                }

                // What a search found on this page, the one in view outlined; and what is selected.
                Shape {
                    anchors.fill: parent
                    visible: image.status === Image.Ready
                    ShapePath {
                        strokeWidth: -1
                        fillColor: Qt.alpha(Theme.accentBright, 0.3)
                        scale: Qt.size(reader.zoom, reader.zoom)
                        PathMultiline { paths: finder.visible && reader.matches >= 0 ? search.boundingPolygonsOnPage(holder.entry.page) : [] }
                    }
                    ShapePath {
                        strokeWidth: 2
                        strokeColor: Theme.accent
                        fillColor: "transparent"
                        scale: Qt.size(reader.zoom, reader.zoom)
                        PathMultiline { paths: finder.visible && search.currentResultLink.page === holder.entry.page ? search.currentResultBoundingPolygons : [] }
                    }
                    ShapePath {
                        strokeWidth: -1
                        fillColor: Qt.alpha(Theme.accent, 0.4)
                        scale: Qt.size(reader.zoom, reader.zoom)
                        PathMultiline { paths: selection.geometry }
                    }
                }

                DragHandler {
                    id: drag
                    enabled: reader.tool !== "ink"
                    acceptedDevices: PointerDevice.Mouse | PointerDevice.Stylus
                    target: null
                    onActiveChanged: if (active) {
                        if (reader.selecting && reader.selecting !== selection) reader.selecting.clear()
                        reader.selecting = selection
                        reader.chosen = null
                        reader.rest()
                    }
                }
                TapHandler {
                    id: tap
                    acceptedDevices: PointerDevice.Mouse | PointerDevice.Stylus
                    onTapped: {
                        reader.rest()
                        reader.tappedPage()
                    }
                }
                // Drawing by hand: with the mouse, a pen or a finger.
                DragHandler {
                    id: inkHandler
                    enabled: reader.tool === "ink"
                    acceptedDevices: PointerDevice.Mouse | PointerDevice.Stylus | PointerDevice.TouchScreen
                    target: null
                    dragThreshold: 0
                    onActiveChanged: active ? reader.inkFrom(holder.entry.page, centroid.pressPosition) : reader.inkDone()
                    onCentroidChanged: if (active) reader.inkTo(centroid.position)
                }
                WheelHandler {
                    acceptedModifiers: Qt.ControlModifier
                    target: null
                    onWheel: (event) => {
                        const at = pages.mapFromItem(paper, event.x, event.y)
                        reader.zoomBy(event.angleDelta.y > 0 ? 1.15 : 1 / 1.15, at.x, at.y)
                    }
                }

                PdfSelection {
                    id: selection
                    anchors.fill: parent
                    document: doc
                    page: holder.entry.page
                    renderScale: reader.zoom
                    from: drag.centroid.pressPosition
                    to: drag.centroid.position
                    hold: !drag.active && !tap.pressed
                    onTextChanged: if (reader.selecting === selection) reader.selectedText = text
                }

                // Links: where they go is shown under the pointer; a click takes the person there.
                Repeater {
                    model: PdfLinkModel { document: doc; page: holder.entry.page }
                    delegate: Item {
                        id: linkArea
                        objectName: "pdfLink"
                        required property rect rectangle
                        required property url url
                        required property int page
                        required property point location
                        x: rectangle.x * reader.zoom
                        y: rectangle.y * reader.zoom
                        width: rectangle.width * reader.zoom
                        height: rectangle.height * reader.zoom
                        ToolTip.visible: linkHover.hovered
                        ToolTip.delay: 400
                        ToolTip.text: linkArea.page >= 0 ? (reader.rowOf(linkArea.page) >= 0 ? qsTr("Page %1").arg(reader.rowOf(linkArea.page) + 1) : qsTr("That page is no longer in this PDF."))
                                                         : reader.followable(linkArea.url) ? String(linkArea.url) : qsTr("QOR Media does not follow this link")
                        HoverHandler { id: linkHover; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: reader.follow(linkArea) }
                    }
                }

                // A form's fields, over the widgets the page's image draws: tinted while they can be filled, and filled by
                // a tap or the keys (FieldItem). Drawing by hand goes over them.
                Repeater {
                    model: reader.fieldsOn(holder.entry.page)
                    delegate: FieldItem {
                        required property var modelData
                        field: modelData
                        zoom: reader.zoom
                        enabled: reader.tool !== "ink"
                        fillable: reader.editable && (modelData.fillable || modelData.kind === "signature")
                        current: reader.isFieldFocus(modelData)
                        typing: reader.isFieldFocus(modelData, "typing")
                        listing: reader.isFieldFocus(modelData, "listing")
                        words: typing ? reader.typingWords : reader.wordsOf(modelData)
                        chosen: reader.chosenOf(modelData)
                        checked: reader.checkedOf(modelData)
                        byKeys: current && reader.fieldFocus.keys
                        onActivated: (byKeys) => reader.activate(modelData, byKeys)
                        onEdited: (text) => reader.edited(text)
                        onKept: reader.keepTyping()
                        onStepped: (back) => reader.stepField(modelData, back)
                        onChose: (places) => reader.chooseFrom(modelData, places)
                        onWrote: (text) => reader.writeChoice(modelData, text)
                        onListClosed: reader.closeOptions()
                    }
                }

                // The PDF's own marks, drawn in the page's image: chosen by a tap while marking up, and a note's words read
                // by a tap at any time.
                Repeater {
                    model: reader.heldOn(holder.entry.page).filter((m) => !reader.isRemoved(m))
                    delegate: Item {
                        id: own
                        objectName: "ownMark"
                        required property var modelData
                        x: modelData.rect.x * reader.zoom
                        y: modelData.rect.y * reader.zoom
                        width: Math.max(8, modelData.rect.width * reader.zoom)
                        height: Math.max(8, modelData.rect.height * reader.zoom)
                        enabled: reader.tool !== "ink" && (reader.marking || modelData.kind === "note")
                        Rectangle {
                            anchors { fill: parent; margins: -3 }
                            visible: reader.isChosenHeld(own.modelData)
                            color: "transparent"
                            border.width: 2
                            border.color: Theme.accent
                        }
                        // Its own press, as a mark made here has (MarkItem): the page under it hears no tap.
                        MouseArea {
                            anchors.fill: parent
                            cursorShape: Qt.PointingHandCursor
                            onClicked: reader.chooseHeld(own.modelData)
                        }
                    }
                }
                // The marks made here, not yet saved, as they will be written; and a note being written.
                Repeater {
                    model: reader.added.filter((m) => m.page === holder.entry.page)
                    delegate: MarkItem {
                        required property var modelData
                        mark: modelData
                        zoom: reader.zoom
                        chosen: reader.isChosenAdded(modelData.id)
                        live: reader.tool !== "ink"
                        shift: reader.dragged && reader.dragged.id === modelData.id ? Qt.point(reader.dragged.dx, reader.dragged.dy) : Qt.point(0, 0)
                        sizedTo: reader.dragged && reader.dragged.id === modelData.id ? reader.dragged.width : 0
                        onPicked: reader.chooseAdded(modelData.id)
                        onMoveStarted: reader.moveStart(modelData.id)
                        onMoved: (dx, dy) => reader.moveBy(modelData.id, dx, dy)
                        onSized: (width) => reader.sizeTo(modelData.id, width)
                        onMoveEnded: reader.moveEnd(modelData.id)
                    }
                }
                MarkItem {
                    visible: reader.noteDraft !== null && reader.noteDraft.page === holder.entry.page
                    mark: reader.noteDraft || ({ kind: "note", x: 0, y: 0, colour: reader.wordsColour })
                    zoom: reader.zoom
                    chosen: true
                    live: false
                }
                // A line as it is drawn.
                Shape {
                    anchors.fill: parent
                    visible: reader.inking !== null && reader.inking.page === holder.entry.page
                    ShapePath {
                        strokeColor: reader.penColour
                        strokeWidth: 2 * reader.zoom
                        fillColor: "transparent"
                        capStyle: ShapePath.RoundCap
                        joinStyle: ShapePath.RoundJoin
                        scale: Qt.size(reader.zoom, reader.zoom)
                        PathPolyline { path: reader.inking && reader.inking.page === holder.entry.page ? reader.inking.points : [] }
                    }
                }
            }
        }
    }

    // A locked PDF: its password, kept only while it is open.
    Rectangle {
        id: unlocking
        visible: reader.locked
        anchors.centerIn: parent
        width: 460
        height: 214
        radius: Theme.radiusPanel
        color: Theme.raised
        border.color: Theme.edge
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 22
            spacing: 8
            Text { text: qsTr("This PDF is locked"); color: Theme.ink; font.family: Theme.ui; font.pixelSize: Theme.sizeHeading; font.weight: Font.DemiBold }
            Text {
                objectName: "passwordNote"
                Layout.fillWidth: true
                wrapMode: Text.WordWrap
                text: reader.wrongPassword ? qsTr("That is not its password. Try again.") : qsTr("Type its password to open it. It is kept only while the PDF is open.")
                color: reader.wrongPassword ? Theme.accentBright : Theme.inkMuted
                font.family: Theme.text
                font.pixelSize: Theme.sizeBody
            }
            TextField {
                id: password
                objectName: "password"
                Layout.fillWidth: true
                echoMode: TextInput.Password
                placeholderText: qsTr("Password")
                color: Theme.ink
                placeholderTextColor: Theme.inkFaint
                font.family: Theme.text
                font.pixelSize: Theme.sizeBody
                background: Rectangle { radius: Theme.radiusControl; color: Theme.well; border.color: password.activeFocus ? Theme.focus : Theme.edge }
                onAccepted: reader.unlock()
            }
            Item { Layout.fillHeight: true }
            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 8
                TextButton { text: qsTr("Back"); onClicked: reader.back() }
                TextButton { objectName: "unlock"; text: qsTr("Open"); primary: true; onClicked: reader.unlock() }
            }
        }
    }

    // A PDF that cannot be read, and why.
    ColumnLayout {
        anchors.centerIn: parent
        visible: reader.unreadable
        spacing: 14
        Text { Layout.alignment: Qt.AlignHCenter; text: qsTr("This PDF cannot be read here."); color: Theme.ink; font.family: Theme.text; font.pixelSize: Theme.sizeTitle }
        Text { Layout.alignment: Qt.AlignHCenter; text: doc.error; visible: text.length > 0; color: Theme.inkMuted; font.family: Theme.text; font.pixelSize: Theme.sizeBody }
        TextButton {
            objectName: "openElsewhere"
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("Open with another program")
            primary: true
            onClicked: Files.openElsewhere(reader.path)
        }
    }

    // Find.
    Rectangle {
        id: finder
        visible: false
        anchors { right: parent.right; top: bar.bottom; rightMargin: 24; topMargin: 8 }
        width: 420
        height: 48
        radius: Theme.radiusPanel
        color: Theme.raised
        border.color: Theme.edge
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 8
            TextField {
                id: query
                objectName: "readerFind"
                Layout.fillWidth: true
                placeholderText: qsTr("Find")
                color: Theme.ink
                placeholderTextColor: Theme.inkFaint
                background: null
                font.family: Theme.text
                font.pixelSize: Theme.sizeBody
                onTextEdited: reader.freshSearch = true
                onAccepted: reader.step(1)
            }
            Text {
                objectName: "readerMatches"
                text: !query.text.length ? "" : reader.matches === 0 ? qsTr("Not found") : qsTr("%1 of %2").arg(search.currentResult + 1).arg(reader.matches)
                color: Theme.inkMuted
                font.family: Theme.text
                font.pixelSize: Theme.sizeUi
            }
            IconButton { iconName: "prev"; size: 32; ToolTip.text: qsTr("Previous (Shift+F3)"); onClicked: reader.step(-1) }
            IconButton { iconName: "next"; size: 32; ToolTip.text: qsTr("Next (F3)"); onClicked: reader.step(1) }
        }
    }

    // A note's words: written for a new note or one made here, read for one the PDF holds. Ctrl+Return keeps them, Esc
    // leaves them as they were.
    Rectangle {
        id: noteEditor
        objectName: "noteEditor"
        // {from: "draft"}, {from: "added", id} or {from: "held", mark}.
        property var source: null
        readonly property bool writable: source !== null && source.from !== "held"
        visible: false
        anchors { right: parent.right; top: markupBar.bottom; rightMargin: 24; topMargin: 8 }
        width: 380
        height: 248
        radius: Theme.radiusPanel
        color: Theme.raised
        border.color: Theme.edge
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 16
            spacing: 8
            Text {
                Layout.fillWidth: true
                elide: Text.ElideRight
                text: noteEditor.source && noteEditor.source.from === "held" && noteEditor.source.mark.author.length
                      ? qsTr("Note from %1").arg(noteEditor.source.mark.author) : qsTr("Note")
                color: Theme.ink
                font.family: Theme.ui
                font.pixelSize: Theme.sizeHeading
                font.weight: Font.DemiBold
            }
            ScrollView {
                Layout.fillWidth: true
                Layout.fillHeight: true
                TextArea {
                    id: noteText
                    objectName: "noteText"
                    readOnly: !noteEditor.writable
                    wrapMode: TextEdit.Wrap
                    placeholderText: noteEditor.writable ? qsTr("What the note says") : ""
                    color: Theme.ink
                    placeholderTextColor: Theme.inkFaint
                    font.family: Theme.text
                    font.pixelSize: Theme.sizeBody
                    background: Rectangle { radius: Theme.radiusControl; color: Theme.well; border.color: noteText.activeFocus ? Theme.focus : Theme.edge }
                    Keys.onPressed: (event) => {
                        if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && (event.modifiers & Qt.ControlModifier)) {
                            reader.closeNote(true)
                            event.accepted = true
                        }
                    }
                }
            }
            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 8
                TextButton { objectName: "noteRemove"; text: qsTr("Take away"); visible: reader.editable && noteEditor.source !== null && noteEditor.source.from !== "draft"; onClicked: reader.removeNote() }
                TextButton { objectName: "noteClose"; text: noteEditor.writable ? qsTr("Cancel") : qsTr("Close"); onClicked: reader.closeNote(false) }
                TextButton { objectName: "noteDone"; text: qsTr("Done"); primary: true; visible: noteEditor.writable; onClicked: reader.closeNote(true) }
            }
        }
    }

    SignaturePad {
        id: signing
        anchors.centerIn: parent
        onPlaced: (signature) => reader.placeSignature(signature)
        onCancelled: {
            signing.visible = false
            reader.signTarget = null
            reader.rest()
        }
    }

    // Changes not yet saved, before going.
    Rectangle {
        id: asking
        objectName: "asking"
        visible: false
        anchors.centerIn: parent
        width: 460
        height: 168
        radius: Theme.radiusPanel
        color: Theme.raised
        border.color: Theme.edge
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 22
            spacing: 8
            Text { text: qsTr("Save the changes to %1?").arg(Files.nameOf(reader.path)); color: Theme.ink; font.family: Theme.ui; font.pixelSize: Theme.sizeHeading; font.weight: Font.DemiBold; Layout.fillWidth: true; elide: Text.ElideMiddle }
            Text { text: qsTr("Its pages, marks and form go back as they were if you don't."); color: Theme.inkMuted; font.family: Theme.text; font.pixelSize: Theme.sizeBody }
            Item { Layout.fillHeight: true }
            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 8
                TextButton { objectName: "askCancel"; text: qsTr("Cancel"); onClicked: reader.answered("cancel") }
                TextButton { objectName: "askDiscard"; text: qsTr("Don't save"); onClicked: reader.answered("discard") }
                TextButton { id: saveChoice; objectName: "askSave"; text: qsTr("Save"); primary: true; onClicked: reader.answered("save") }
            }
        }
    }

    // The bottom line: the page, which can be typed, and the zoom.
    Rectangle {
        id: status
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: reader.ready ? 32 : 0
        visible: reader.ready
        color: Theme.well
        RowLayout {
            anchors { left: parent.left; leftMargin: 16; verticalCenter: parent.verticalCenter }
            spacing: 6
            Text { text: qsTr("Page"); color: Theme.inkFaint; font.family: Theme.text; font.pixelSize: Theme.sizeUi }
            TextField {
                id: pageField
                objectName: "pageField"
                implicitWidth: 56
                implicitHeight: 24
                padding: 2
                horizontalAlignment: TextInput.AlignHCenter
                text: "1"
                validator: IntValidator { bottom: 1; top: Math.max(1, reader.pageCount) }
                color: Theme.ink
                font.family: Theme.text
                font.pixelSize: Theme.sizeUi
                background: Rectangle { radius: Theme.radiusControl; color: Theme.base; border.color: pageField.activeFocus ? Theme.focus : Theme.edge }
                onActiveFocusChanged: activeFocus ? selectAll() : (text = String(reader.page + 1))
                onAccepted: { reader.goTo(Number(text) - 1); reader.rest() }
            }
            Text { text: qsTr("of %1").arg(reader.pageCount); color: Theme.inkFaint; font.family: Theme.text; font.pixelSize: Theme.sizeUi }
        }
        Text {
            objectName: "zoomLabel"
            anchors { right: parent.right; rightMargin: 16; verticalCenter: parent.verticalCenter }
            text: (reader.fitting === "width" ? qsTr("Fit width") : reader.fitting === "page" ? qsTr("Whole page") : "") + (reader.fitting.length ? " · " : "") + Math.round(reader.zoom / reader.actualSize * 100) + "%"
            color: Theme.inkFaint
            font.family: Theme.text
            font.pixelSize: Theme.sizeUi
        }
    }

    Keys.onPressed: (event) => {
        if (!reader.ready || asking.visible || signing.visible || noteText.activeFocus) return
        // Typing in a field, or choosing from its options, the keys are theirs.
        if (reader.fieldFocus !== null && reader.fieldFocus.mode !== "") return
        const wide = pages.contentWidth > pages.width + 1
        const alt = event.modifiers & Qt.AltModifier
        const step = event.modifiers & Qt.ShiftModifier ? 10 : 1
        switch (event.key) {
        case Qt.Key_PageDown: reader.scrollBy(pages.height * 0.9); break
        case Qt.Key_PageUp: reader.scrollBy(-pages.height * 0.9); break
        case Qt.Key_Space: reader.scrollBy((event.modifiers & Qt.ShiftModifier ? -1 : 1) * pages.height * 0.9); break
        case Qt.Key_Down: if (alt) reader.movePage(1); else if (!reader.nudge(0, step)) reader.scrollBy(48); break
        case Qt.Key_Up: if (alt) reader.movePage(-1); else if (!reader.nudge(0, -step)) reader.scrollBy(-48); break
        case Qt.Key_Right: if (reader.nudge(step, 0)) break; if (wide) { pages.contentX += 48; reader.clamp() } else reader.goTo(reader.page + 1); break
        case Qt.Key_Left: if (reader.nudge(-step, 0)) break; if (wide) { pages.contentX -= 48; reader.clamp() } else reader.goTo(reader.page - 1); break
        case Qt.Key_Home: reader.goTo(0); break
        case Qt.Key_End: reader.goTo(reader.pageCount - 1); break
        case Qt.Key_Plus: case Qt.Key_Equal: reader.zoomBy(1.25, pages.width / 2, pages.height / 2); break
        case Qt.Key_Minus: reader.zoomBy(0.8, pages.width / 2, pages.height / 2); break
        case Qt.Key_0: reader.cycleFit(); break
        case Qt.Key_R: reader.turnPage(event.modifiers & Qt.ShiftModifier ? -1 : 1); break
        case Qt.Key_Delete: if (!reader.removeChosen()) reader.deletePage(); break
        case Qt.Key_M: reader.toggleMarking(); break
        case Qt.Key_H: reader.markWords("highlight"); break
        case Qt.Key_U: reader.markWords("underline"); break
        case Qt.Key_X: reader.markWords("strikeout"); break
        case Qt.Key_N: reader.placeNote(); break
        case Qt.Key_D: reader.toggleInk(); break
        case Qt.Key_BracketLeft: reader.resizeChosen(0.9); break
        case Qt.Key_BracketRight: reader.resizeChosen(1 / 0.9); break
        case Qt.Key_F: reader.fullScreenToggled(); break
        case Qt.Key_F9: reader.panel = !reader.panel; break
        // Into a form's fields; on a page without them, on to the next control.
        case Qt.Key_Tab: if (event.modifiers !== Qt.NoModifier || !reader.enterFields()) return; break
        default: return
        }
        event.accepted = true
    }

    Shortcut { sequences: [StandardKey.Find]; enabled: reader.ready && !asking.visible && !signing.visible; onActivated: reader.openFinder() }
    Shortcut { sequence: "F3"; enabled: finder.visible; onActivated: reader.step(1) }
    Shortcut { sequence: "Shift+F3"; enabled: finder.visible; onActivated: reader.step(-1) }
    Shortcut { sequence: "Ctrl+G"; enabled: reader.ready && !asking.visible && !signing.visible; onActivated: pageField.forceActiveFocus() }
    Shortcut { sequences: [StandardKey.Copy]; enabled: reader.ready; onActivated: reader.copy() }
    Shortcut { sequences: [StandardKey.SelectAll]; enabled: reader.ready && !asking.visible && !signing.visible; onActivated: reader.selectPage() }
    Shortcut { sequences: [StandardKey.Undo]; enabled: reader.ready && !asking.visible && !signing.visible; onActivated: reader.undo() }
    Shortcut { sequences: [StandardKey.Redo]; enabled: reader.ready && !asking.visible && !signing.visible; onActivated: reader.redo() }
    Shortcut { sequences: [StandardKey.Save]; enabled: reader.ready && !signing.visible; onActivated: asking.visible ? reader.answered("save") : reader.save() }
    Shortcut {
        // Only when there is something to close first; otherwise the window's Esc goes back.
        sequence: "Esc"
        enabled: reader.consumesEscape
        onActivated: {
            if (asking.visible) reader.answered("cancel")
            else if (signing.visible) signing.cancelled()
            else if (noteEditor.visible) reader.closeNote(false)
            else if (finder.visible) reader.closeFinder()
            else if (pageField.activeFocus) reader.rest()
            else if (reader.fieldFocus !== null && reader.fieldFocus.mode === "listing") reader.closeOptions()
            else if (reader.fieldFocus !== null && reader.fieldFocus.mode === "typing") reader.leaveTyping()
            else if (reader.fieldFocus !== null) reader.rest()
            else if (reader.tool.length) reader.tool = ""
            else if (reader.chosen !== null) reader.chosen = null
            else if (reader.marking) reader.toggleMarking()
            else reader.goBack()
        }
    }
}
