import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QOR.Media

// The editor (ADR-094 decision 3): plain text, Markdown and code, read and saved as they were written (TextFile), coloured
// in the theme's colours, with line numbers for code, a Markdown preview beside the words, and find. Ctrl+S saves, Ctrl+F
// finds, F3 finds the next, Ctrl+P shows the preview, Alt+Z wraps lines. Nothing is lost on the way out: with unsaved
// changes it asks before it goes back to the Library or opens something else.
FocusScope {
    id: editor
    required property string path
    signal back()
    focus: true

    property bool dirty: false
    property bool loading: true
    property bool preview: false
    property bool wrap: file.language !== "code"
    property string notice
    property var afterAsk: null
    readonly property bool consumesEscape: dirty || finder.visible || asking.visible

    // Called by the window before it goes back or opens another file.
    function ask(then) {
        afterAsk = then
        asking.visible = true
    }
    function save() {
        if (file.save(area.text)) {
            dirty = false
            notice = qsTr("Saved")
            noticeTimer.restart()
            return true
        }
        notice = file.error
        noticeTimer.restart()
        return false
    }
    function goBack() {
        if (dirty)
            ask(() => editor.back())
        else
            editor.back()
    }
    function find(forward) {
        const needle = query.text.toLowerCase()
        if (!needle.length) return
        const hay = area.text.toLowerCase()
        let at = forward ? hay.indexOf(needle, area.selectionEnd) : hay.lastIndexOf(needle, Math.max(0, area.selectionStart - 1))
        if (at < 0) at = forward ? hay.indexOf(needle) : hay.lastIndexOf(needle)
        if (at < 0) {
            matches.text = qsTr("Not found")
            return
        }
        area.select(at, at + needle.length)
        const r = area.positionToRectangle(at)
        if (r.y < flick.contentY || r.y > flick.contentY + flick.height - 40)
            flick.contentY = Math.max(0, r.y - flick.height / 3)
        let count = 0, i = -1
        while ((i = hay.indexOf(needle, i + 1)) >= 0 && count < 10000) count++
        matches.text = count === 1 ? qsTr("1 match") : qsTr("%1 matches").arg(count)
    }
    function caret() {
        const before = area.text.slice(0, area.cursorPosition)
        const line = before.split("\n").length
        return qsTr("Line %1, column %2").arg(line).arg(area.cursorPosition - before.lastIndexOf("\n"))
    }

    TextFile { id: file }

    Component.onCompleted: {
        const words = file.load(path)
        area.text = words
        file.highlight(area.textDocument, Theme.tokens)
        loading = false
        dirty = false
        area.cursorPosition = 0
        area.forceActiveFocus()
    }
    Connections {
        target: Theme
        function onTokensChanged() { file.highlight(area.textDocument, Theme.tokens) }
    }

    Rectangle { anchors.fill: parent; color: Theme.base }

    TopBar {
        id: bar
        anchors { left: parent.left; right: parent.right; top: parent.top }
        title: (editor.dirty ? "● " : "") + Files.nameOf(editor.path)
        subtitle: file.error.length ? file.error : [file.language === "markdown" ? "Markdown" : file.language === "code" ? qsTr("Code") : qsTr("Text"), file.crlf ? "CRLF" : "LF", editor.notice].filter(Boolean).join(" · ")
        color: Theme.base
        onBack: editor.goBack()
        IconButton { iconName: "preview"; visible: file.language === "markdown"; active: editor.preview; ToolTip.text: qsTr("Preview (Ctrl+P)"); onClicked: editor.preview = !editor.preview }
        IconButton { iconName: "save"; enabled: editor.dirty; opacity: enabled ? 1 : 0.4; ToolTip.text: qsTr("Save (Ctrl+S)"); onClicked: editor.save() }
        IconButton { iconName: "open"; ToolTip.text: qsTr("Open with another program"); onClicked: Files.openElsewhere(editor.path) }
    }

    Timer { id: noticeTimer; interval: 2400; onTriggered: editor.notice = "" }

    // What could not be opened here, and why.
    ColumnLayout {
        anchors.centerIn: parent
        visible: file.error.length > 0 && editor.loading === false && area.text.length === 0
        spacing: 14
        Text { Layout.alignment: Qt.AlignHCenter; text: file.error; color: Theme.inkBody; font.family: Theme.text; font.pixelSize: 16 }
        Button {
            Layout.alignment: Qt.AlignHCenter
            text: qsTr("Open with another program")
            onClicked: Files.openElsewhere(editor.path)
        }
    }

    RowLayout {
        anchors { left: parent.left; right: parent.right; top: bar.bottom; bottom: status.top }
        spacing: 0
        visible: !(file.error.length > 0 && area.text.length === 0)

        // The words, with line numbers beside code.
        Item {
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredWidth: 1

            Rectangle {
                id: gutter
                visible: !editor.wrap
                width: visible ? numbers.implicitWidth + 28 : 0
                anchors { left: parent.left; top: parent.top; bottom: parent.bottom }
                color: Theme.well
                clip: true
                Text {
                    id: numbers
                    x: 14
                    y: area.topPadding - flick.contentY
                    text: Array.from({ length: area.lineCount }, (_, i) => i + 1).join("\n")
                    color: Theme.inkFaint
                    font: area.font
                    horizontalAlignment: Text.AlignRight
                    lineHeight: 1
                }
            }

            Flickable {
                id: flick
                anchors { left: gutter.right; right: parent.right; top: parent.top; bottom: parent.bottom }
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                ScrollBar.vertical: ScrollBar { }
                ScrollBar.horizontal: ScrollBar { }

                TextArea.flickable: TextArea {
                    id: area
                    objectName: "editorArea"
                    font.family: file.language === "code" ? Theme.mono : Theme.text
                    font.pixelSize: file.language === "code" ? 14 : 16
                    color: Theme.inkBody
                    selectionColor: Qt.alpha(Theme.accent, 0.45)
                    selectedTextColor: Theme.ink
                    wrapMode: editor.wrap ? TextEdit.Wrap : TextEdit.NoWrap
                    textFormat: TextEdit.PlainText
                    persistentSelection: true
                    selectByMouse: true
                    leftPadding: 18
                    rightPadding: 18
                    topPadding: 14
                    bottomPadding: 120
                    tabStopDistance: metrics.averageCharacterWidth * 4
                    background: null
                    onTextChanged: if (!editor.loading) editor.dirty = true
                    FontMetrics { id: metrics; font: area.font }
                }
            }
        }

        Rectangle { visible: editor.preview; Layout.fillHeight: true; width: 1; color: Theme.edge }

        // The Markdown as it reads.
        ScrollView {
            visible: editor.preview
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredWidth: 1
            contentWidth: availableWidth
            TextEdit {
                width: parent.width
                readOnly: true
                selectByMouse: true
                textFormat: TextEdit.MarkdownText
                text: editor.preview ? area.text : ""
                wrapMode: TextEdit.Wrap
                color: Theme.inkBody
                font.family: Theme.text
                font.pixelSize: 16
                leftPadding: 28
                rightPadding: 28
                topPadding: 18
                bottomPadding: 60
                onLinkActivated: (link) => Qt.openUrlExternally(link)
            }
        }
    }

    // Find.
    Rectangle {
        id: finder
        visible: false
        anchors { right: parent.right; top: bar.bottom; rightMargin: 24; topMargin: 8 }
        width: 420
        height: 48
        radius: 10
        color: Theme.raised
        border.color: Theme.edge
        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 8
            TextField {
                id: query
                Layout.fillWidth: true
                placeholderText: qsTr("Find")
                color: Theme.ink
                placeholderTextColor: Theme.inkFaint
                background: null
                font.family: Theme.text
                font.pixelSize: 14
                onTextEdited: { area.select(area.selectionStart, area.selectionStart); editor.find(true) }
                onAccepted: editor.find(true)
                Keys.onEscapePressed: { finder.visible = false; area.forceActiveFocus() }
            }
            Text { id: matches; color: Theme.inkMuted; font.family: Theme.text; font.pixelSize: 12 }
            IconButton { iconName: "prev"; size: 32; ToolTip.text: qsTr("Previous (Shift+F3)"); onClicked: editor.find(false) }
            IconButton { iconName: "next"; size: 32; ToolTip.text: qsTr("Next (F3)"); onClicked: editor.find(true) }
        }
    }

    // Unsaved changes, before going.
    Rectangle {
        id: asking
        visible: false
        anchors.centerIn: parent
        width: 460
        height: 168
        radius: 14
        color: Theme.raised
        border.color: Theme.edge
        ColumnLayout {
            anchors.fill: parent
            anchors.margins: 22
            spacing: 8
            Text { text: qsTr("Save the changes to %1?").arg(Files.nameOf(editor.path)); color: Theme.ink; font.family: Theme.display; font.pixelSize: 17; font.weight: Font.DemiBold; Layout.fillWidth: true; elide: Text.ElideMiddle }
            Text { text: qsTr("They are lost if you don't."); color: Theme.inkMuted; font.family: Theme.text; font.pixelSize: 13 }
            Item { Layout.fillHeight: true }
            RowLayout {
                Layout.alignment: Qt.AlignRight
                spacing: 8
                Button { text: qsTr("Cancel"); onClicked: { asking.visible = false; editor.afterAsk = null; area.forceActiveFocus() } }
                Button { text: qsTr("Don't save"); onClicked: { asking.visible = false; editor.dirty = false; const then = editor.afterAsk; editor.afterAsk = null; if (then) then() } }
                Button {
                    text: qsTr("Save")
                    highlighted: true
                    onClicked: {
                        if (!editor.save()) return
                        asking.visible = false
                        const then = editor.afterAsk
                        editor.afterAsk = null
                        if (then) then()
                    }
                }
            }
        }
    }

    // The bottom line: where the caret is.
    Rectangle {
        id: status
        anchors { left: parent.left; right: parent.right; bottom: parent.bottom }
        height: 28
        color: Theme.well
        Text {
            anchors { left: parent.left; leftMargin: 16; verticalCenter: parent.verticalCenter }
            text: area.text.length < 4 * 1024 * 1024 ? editor.caret() : ""
            color: Theme.inkFaint
            font.family: Theme.text
            font.pixelSize: 12
        }
        Text {
            anchors { right: parent.right; rightMargin: 16; verticalCenter: parent.verticalCenter }
            text: qsTr("%1 lines").arg(area.lineCount)
            color: Theme.inkFaint
            font.family: Theme.text
            font.pixelSize: 12
        }
    }

    Shortcut { sequences: [StandardKey.Save]; onActivated: editor.save() }
    Shortcut { sequences: [StandardKey.Find]; onActivated: { finder.visible = true; query.selectAll(); query.forceActiveFocus() } }
    Shortcut { sequence: "F3"; onActivated: editor.find(true) }
    Shortcut { sequence: "Shift+F3"; onActivated: editor.find(false) }
    Shortcut { sequence: "Ctrl+P"; enabled: file.language === "markdown"; onActivated: editor.preview = !editor.preview }
    Shortcut { sequence: "Alt+Z"; onActivated: editor.wrap = !editor.wrap }
    Shortcut {
        // Only when there is something to close first; otherwise the window's Esc goes back.
        sequence: "Esc"
        enabled: editor.consumesEscape
        onActivated: {
            if (asking.visible) { asking.visible = false; editor.afterAsk = null; area.forceActiveFocus() }
            else if (finder.visible) { finder.visible = false; area.forceActiveFocus() }
            else editor.goBack()
        }
    }
}
