// The pages are a TableView's delegates, which in this Qt see the ids of this file only when bound to it.
pragma ComponentBehavior: Bound

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
    readonly property bool ready: doc.status === PdfDocument.Ready
    readonly property bool unreadable: doc.status === PdfDocument.Error && !locked
    readonly property int pageCount: ready ? doc.pageCount : 0
    readonly property int matchCount: finder.visible ? reader.matches : 0
    readonly property int match: search.currentResult
    // A hundred percent is the page at its printed size.
    readonly property real actualSize: 96 / 72
    readonly property real minZoom: actualSize * 0.1
    readonly property real maxZoom: actualSize * 8
    // Room either side of the widest page, and above the first.
    readonly property int margin: 24
    readonly property bool consumesEscape: finder.visible || pageField.activeFocus

    function fit() {
        if (!ready || pages.width <= 0) return
        const across = pages.width - 2 * margin
        const size = doc.pagePointSize(page)
        const z = fitting === "page"
            ? Math.min(across / size.width, (pages.height - 2 * margin) / size.height)
            : across / Math.max(1, doc.maxPageWidth)
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
        if (!ready) return
        target = Math.max(0, Math.min(pageCount - 1, target))
        const y = location ? location.y * zoom - margin : -1
        if (y > 0)
            pages.positionViewAtRow(target, TableView.AlignTop, 0, Qt.rect(0, y, 1, 1))
        else
            pages.positionViewAtRow(target, TableView.AlignTop, -margin)
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
        if (link.page >= 0)
            goTo(link.page, link.location)
        else if (followable(link.url))
            Qt.openUrlExternally(link.url)
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
            if (search.data(search.index(i, 0), 256) >= here) return i  // QPdfSearchModel::Role::Page
        return reader.matches ? 0 : -1
    }
    function step(delta) {
        if (!reader.matches) return
        search.currentResult = (search.currentResult + delta + reader.matches) % reader.matches
    }
    function showMatch() {
        const link = search.currentResultLink
        if (search.currentResult < 0 || !link || link.page < 0) return
        const r = search.currentResultBoundingRect
        pages.positionViewAtRow(link.page, TableView.AlignTop, 0, Qt.rect(0, Math.max(0, r.y * zoom - pages.height / 3), 1, 1))
        page = link.page
    }
    function openFinder() {
        finder.visible = true
        query.selectAll()
        query.forceActiveFocus()
    }
    // The keyboard back to the pages, from a field, a button or a list. Focusing the reader itself would hand it back to
    // whichever of them had it last, since the reader is a focus scope.
    function rest() {
        resting.forceActiveFocus()
    }
    Item { id: resting; focus: true }
    function closeFinder() {
        finder.visible = false
        reader.rest()
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
    // Open `path` afresh: nothing of the last PDF stays, its password least of all.
    function load() {
        finder.visible = false
        query.clear()
        if (selecting) selecting.clear()
        selecting = null
        selectedText = ""
        locked = false
        wrongPassword = false
        triedPassword = ""
        hasContents = false
        page = 0
        fitting = "width"
        doc.password = ""
        doc.source = Files.urlOf(path)
    }
    property bool started: false
    Component.onCompleted: { started = true; load() }
    onPathChanged: if (started) load()

    PdfDocument {
        id: doc
        onPasswordRequired: {
            reader.wrongPassword = reader.triedPassword.length > 0
            reader.locked = true
            password.forceActiveFocus()
        }
        onStatusChanged: {
            if (doc.status === PdfDocument.Ready) {
                reader.locked = false
                reader.wrongPassword = false
                reader.rest()
                Qt.callLater(reader.fit)
                Qt.callLater(reader.readContents)
            } else if (doc.status === PdfDocument.Error && reader.locked && reader.triedPassword.length) {
                reader.wrongPassword = true
            }
        }
    }

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

    Rectangle { anchors.fill: parent; color: Theme.void_ }

    TopBar {
        id: bar
        anchors { left: parent.left; right: parent.right; top: parent.top }
        title: Files.nameOf(reader.path)
        subtitle: reader.locked ? qsTr("Locked") : reader.unreadable ? "" : [doc.title.length && doc.title !== title ? doc.title : "", reader.pageCount ? qsTr("Page %1 of %2").arg(reader.page + 1).arg(reader.pageCount) : ""].filter(Boolean).join(" · ")
        color: Theme.base
        onBack: reader.back()
        IconButton { iconName: "pages"; visible: reader.ready; active: reader.panel; ToolTip.text: qsTr("Pages and contents (F9)"); onClicked: reader.panel = !reader.panel }
        IconButton { iconName: "find"; visible: reader.ready; active: finder.visible; ToolTip.text: qsTr("Find (Ctrl+F)"); onClicked: finder.visible ? reader.closeFinder() : reader.openFinder() }
        IconButton { iconName: "zoomout"; visible: reader.ready; ToolTip.text: qsTr("Zoom out (-)"); onClicked: reader.zoomBy(0.8, pages.width / 2, pages.height / 2) }
        IconButton { iconName: "zoomin"; visible: reader.ready; ToolTip.text: qsTr("Zoom in (+)"); onClicked: reader.zoomBy(1.25, pages.width / 2, pages.height / 2) }
        IconButton { iconName: "fit"; visible: reader.ready; ToolTip.text: reader.fitting === "width" ? qsTr("Whole page (0)") : qsTr("Fit the width (0)"); onClicked: reader.cycleFit() }
        IconButton { iconName: "full"; ToolTip.text: qsTr("Full screen (F)"); onClicked: reader.fullScreenToggled() }
        IconButton { iconName: "open"; ToolTip.text: qsTr("Open with another program"); onClicked: Files.openElsewhere(reader.path) }
    }

    // Beside the pages: each page small, and the PDF's contents when it has them.
    Rectangle {
        id: side
        visible: reader.panel && reader.ready
        width: visible ? 216 : 0
        anchors { left: parent.left; top: bar.bottom; bottom: status.top }
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
            anchors { left: parent.left; right: parent.right; top: tabs.bottom; bottom: parent.bottom; topMargin: 8 }
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
                readonly property size points: doc.pagePointSize(index)
                width: thumbnails.width
                height: sheet.height + label.height + 6
                // The paper is a Rectangle's own white, the colour a PDF is drawn on.
                Rectangle {
                    id: sheet
                    anchors.horizontalCenter: parent.horizontalCenter
                    width: 132
                    height: Math.round(width * thumb.points.height / Math.max(1, thumb.points.width))
                    border.width: thumb.index === reader.page || thumbnails.activeFocus && thumb.index === thumbnails.currentIndex ? 2 : 0
                    border.color: thumb.index === reader.page ? Theme.accent : Theme.focus
                    PdfPageImage {
                        anchors.fill: parent
                        anchors.margins: parent.border.width
                        document: doc
                        currentFrame: thumb.index
                        asynchronous: true
                        fillMode: Image.PreserveAspectFit
                        sourceSize.width: width * Screen.devicePixelRatio
                    }
                }
                Text {
                    id: label
                    anchors { top: sheet.bottom; topMargin: 6; horizontalCenter: parent.horizontalCenter }
                    text: doc.pageLabel(thumb.index) || String(thumb.index + 1)
                    color: thumb.index === reader.page ? Theme.ink : Theme.inkMuted
                    font.family: Theme.text
                    font.pixelSize: Theme.sizeCaption
                }
                TapHandler { onTapped: { thumbnails.currentIndex = thumb.index; reader.goTo(thumb.index) } }
            }
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
            ScrollBar.vertical: ScrollBar { }
            delegate: Item {
                id: entry
                required property TreeView treeView
                required property bool expanded
                required property bool hasChildren
                required property int depth
                required property int row
                required property string title
                required property int page
                required property point location
                implicitWidth: contents.width
                implicitHeight: 32
                Rectangle {
                    anchors.fill: parent
                    color: entry.page === reader.page ? Theme.raised : hover.hovered ? Theme.surface : "transparent"
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
                    color: entry.page === reader.page ? Theme.ink : Theme.inkBody
                    font.family: Theme.text
                    font.pixelSize: Theme.sizeUi
                }
                HoverHandler { id: hover }
                TapHandler { onTapped: reader.goTo(entry.page, entry.location) }
            }
        }

        Rectangle { anchors { right: parent.right; top: parent.top; bottom: parent.bottom } width: 1; color: Theme.edge }
    }

    TableView {
        id: pages
        objectName: "pages"
        visible: reader.ready
        anchors { left: side.right; right: parent.right; top: bar.bottom; bottom: status.top }
        clip: true
        model: reader.pageCount
        rowSpacing: 12
        topMargin: reader.margin
        bottomMargin: reader.margin
        reuseItems: false
        animate: false
        keyNavigationEnabled: false
        activeFocusOnTab: false
        pointerNavigationEnabled: false
        boundsBehavior: Flickable.StopAtBounds
        columnWidthProvider: () => Math.max(pages.width, doc.maxPageWidth * reader.zoom + 2 * reader.margin)
        rowHeightProvider: (row) => doc.pagePointSize(row).height * reader.zoom
        ScrollBar.vertical: ScrollBar { }
        ScrollBar.horizontal: ScrollBar { }
        onContentYChanged: reader.track()
        onWidthChanged: reader.fitting.length ? Qt.callLater(reader.fit) : Qt.callLater(pages.forceLayout)
        onHeightChanged: if (reader.fitting === "page") Qt.callLater(reader.fit)

        delegate: Item {
            id: holder
            required property int row
            readonly property size points: doc.pagePointSize(row)
            property alias selection: selection

            // The edge the page sits on, and the paper: a Rectangle's own white, the colour a PDF is drawn on.
            Rectangle {
                anchors.fill: paper
                anchors.margins: -1
                color: Theme.edge
            }
            Rectangle {
                id: paper
                anchors.centerIn: parent
                width: holder.points.width * reader.zoom
                height: holder.points.height * reader.zoom

                PdfPageImage {
                    id: image
                    anchors.fill: parent
                    document: doc
                    currentFrame: holder.row
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
                        PathMultiline { paths: finder.visible && reader.matches >= 0 ? search.boundingPolygonsOnPage(holder.row) : [] }
                    }
                    ShapePath {
                        strokeWidth: 2
                        strokeColor: Theme.accent
                        fillColor: "transparent"
                        scale: Qt.size(reader.zoom, reader.zoom)
                        PathMultiline { paths: finder.visible && search.currentResultLink.page === holder.row ? search.currentResultBoundingPolygons : [] }
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
                    acceptedDevices: PointerDevice.Mouse | PointerDevice.Stylus
                    target: null
                    onActiveChanged: if (active) {
                        if (reader.selecting && reader.selecting !== selection) reader.selecting.clear()
                        reader.selecting = selection
                        reader.rest()
                    }
                }
                TapHandler {
                    id: tap
                    acceptedDevices: PointerDevice.Mouse | PointerDevice.Stylus
                    onTapped: reader.rest()
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
                    page: holder.row
                    renderScale: reader.zoom
                    from: drag.centroid.pressPosition
                    to: drag.centroid.position
                    hold: !drag.active && !tap.pressed
                    onTextChanged: if (reader.selecting === selection) reader.selectedText = text
                }

                // Links: where they go is shown under the pointer; a click takes the person there.
                Repeater {
                    model: PdfLinkModel { document: doc; page: holder.row }
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
                        ToolTip.text: linkArea.page >= 0 ? qsTr("Page %1").arg(linkArea.page + 1)
                                                         : reader.followable(linkArea.url) ? String(linkArea.url) : qsTr("QOR Media does not follow this link")
                        HoverHandler { id: linkHover; cursorShape: Qt.PointingHandCursor }
                        TapHandler { onTapped: reader.follow(linkArea) }
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
        if (!reader.ready) return
        const wide = pages.contentWidth > pages.width + 1
        switch (event.key) {
        case Qt.Key_PageDown: reader.scrollBy(pages.height * 0.9); break
        case Qt.Key_PageUp: reader.scrollBy(-pages.height * 0.9); break
        case Qt.Key_Space: reader.scrollBy((event.modifiers & Qt.ShiftModifier ? -1 : 1) * pages.height * 0.9); break
        case Qt.Key_Down: reader.scrollBy(48); break
        case Qt.Key_Up: reader.scrollBy(-48); break
        case Qt.Key_Right: if (wide) { pages.contentX += 48; reader.clamp() } else reader.goTo(reader.page + 1); break
        case Qt.Key_Left: if (wide) { pages.contentX -= 48; reader.clamp() } else reader.goTo(reader.page - 1); break
        case Qt.Key_Home: reader.goTo(0); break
        case Qt.Key_End: reader.goTo(reader.pageCount - 1); break
        case Qt.Key_Plus: case Qt.Key_Equal: reader.zoomBy(1.25, pages.width / 2, pages.height / 2); break
        case Qt.Key_Minus: reader.zoomBy(0.8, pages.width / 2, pages.height / 2); break
        case Qt.Key_0: reader.cycleFit(); break
        case Qt.Key_F: reader.fullScreenToggled(); break
        case Qt.Key_F9: reader.panel = !reader.panel; break
        default: return
        }
        event.accepted = true
    }

    Shortcut { sequences: [StandardKey.Find]; enabled: reader.ready; onActivated: reader.openFinder() }
    Shortcut { sequence: "F3"; enabled: finder.visible; onActivated: reader.step(1) }
    Shortcut { sequence: "Shift+F3"; enabled: finder.visible; onActivated: reader.step(-1) }
    Shortcut { sequence: "Ctrl+G"; enabled: reader.ready; onActivated: pageField.forceActiveFocus() }
    Shortcut { sequences: [StandardKey.Copy]; enabled: reader.ready; onActivated: reader.copy() }
    Shortcut { sequences: [StandardKey.SelectAll]; enabled: reader.ready; onActivated: reader.selectPage() }
    Shortcut {
        // Only when there is something to close first; otherwise the window's Esc goes back.
        sequence: "Esc"
        enabled: reader.consumesEscape
        onActivated: finder.visible ? reader.closeFinder() : reader.rest()
    }
}
