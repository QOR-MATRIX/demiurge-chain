// The logic editor (DIRECTION P3.15): one logic file, logic/<name>.qml in the project, written in the Studio, under the
// world. Lines are numbered and coloured (LogicHighlighter); every half second of quiet the text is built as QML where
// the file is (SceneIO.checkLogic), and what does not build is listed by line, a click going to that line. Saving
// (its button, or Ctrl+S while it has the keys) writes the file whole, and a scene that is playing picks it up at once,
// as it picks up any save of its logic.

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import QQ
import QQ.Studio

Pane {
    id: editor

    /// The Studio's window: its project.
    required property QtObject studio
    /// The file open, by name (logic/<name>.qml), or "".
    property string name: ""
    readonly property url file: name === "" ? "" : String(studio.project) + "/logic/" + name + ".qml"
    /// The text as last read or written, and what the text does not build for: [{ line, column, message }].
    property string saved: ""
    property var problems: []
    readonly property bool dirty: name !== "" && code.text !== saved
    /// Whether the text has the keys: Ctrl+S saves it rather than the scene.
    readonly property bool typing: code.activeFocus
    readonly property alias text: code.text

    signal closed()

    Material.background: Look.well
    padding: 0

    /// Open logic/<name>.qml: false (said) if it cannot be read.
    function open(logicName) {
        const url = String(studio.project) + "/logic/" + logicName + ".qml"
        const text = SceneIO.readText(url)
        if (text === undefined || text === null) {
            studio.say(SceneIO.lastError, true)
            return false
        }
        name = logicName
        saved = text
        code.text = text
        code.cursorPosition = 0
        check()
        code.forceActiveFocus()
        return true
    }

    /// Write the text to its file, whole: true if written.
    function save() {
        if (name === "")
            return false
        const written = SceneIO.writeLogic(studio.project, name, code.text)
        if (String(written) === "") {
            studio.say(SceneIO.lastError, true)
            return false
        }
        saved = code.text
        check()
        studio.say(problems.length === 0 ? qsTr("Saved logic/%1.qml.").arg(name)
                                         : qsTr("Saved logic/%1.qml; it does not build yet (line %2).")
                                               .arg(name).arg(problems[0].line), problems.length > 0)
        return true
    }

    function check() {
        problems = name === "" ? [] : SceneIO.checkLogic(code.text, file)
    }

    /// Put the cursor at the start of line `n` (from 1).
    function goTo(n) {
        const lines = code.text.split("\n")
        let at = 0
        for (let i = 0; i < Math.min(n - 1, lines.length); ++i)
            at += lines[i].length + 1
        code.cursorPosition = at
        code.forceActiveFocus()
    }

    Timer {
        id: quiet
        interval: 500
        onTriggered: editor.check()
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        // Its name, whether it is saved, and what it does not build for.
        RowLayout {
            Layout.fillWidth: true
            Layout.leftMargin: 12
            Layout.rightMargin: 6
            spacing: 10
            Label {
                text: "logic/" + editor.name + ".qml"
                font.pixelSize: 13
                font.weight: Font.DemiBold
            }
            Label {
                visible: editor.dirty
                text: qsTr("Unsaved")
                color: Material.accent
                font.pixelSize: 12
            }
            Label {
                objectName: "logicStatus"
                text: editor.problems.length === 0 ? qsTr("Builds") : qsTr("%n problem(s)", "", editor.problems.length)
                color: editor.problems.length === 0 ? Look.ok : Look.bad
                font.pixelSize: 12
            }
            Item { Layout.fillWidth: true }
            StudioButton {
                objectName: "logicSave"
                kind: "ghost"
                text: qsTr("Save")
                highlighted: editor.dirty
                enabled: editor.name !== ""
                onClicked: editor.save()
            }
            StudioButton {
                text: qsTr("Close")
                onClicked: {
                    editor.name = ""
                    editor.closed()
                }
            }
        }

        Flickable {
            id: flick
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            ScrollBar.vertical: ScrollBar {}
            ScrollBar.horizontal: ScrollBar {}

            TextArea.flickable: TextArea {
                id: code
                objectName: "logicText"
                wrapMode: TextEdit.NoWrap
                selectByMouse: true
                persistentSelection: true
                font.family: Look.mono
                font.pixelSize: 13
                leftPadding: gutter.width + 12
                color: Look.inkBody
                background: null
                onTextChanged: quiet.restart()
                // Four spaces for a tab, as the files are written.
                Keys.onTabPressed: (event) => {
                    code.insert(code.cursorPosition, "    ")
                    event.accepted = true
                }

                // Line numbers, in the same type, beside the text.
                Text {
                    id: gutter
                    x: 6
                    y: code.topPadding
                    width: Math.max(28, implicitWidth)
                    horizontalAlignment: Text.AlignRight
                    font: code.font
                    color: Look.inkFaint
                    text: Array.from({ length: Math.max(1, code.lineCount) }, (_, i) => i + 1).join("\n")
                }
            }
        }

        LogicHighlighter { document: code.textDocument }

        // What does not build, by line: a click goes there.
        ListView {
            objectName: "logicProblems"
            visible: editor.problems.length > 0
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(3, editor.problems.length) * 24 + 4
            clip: true
            model: editor.problems
            delegate: ItemDelegate {
                required property var modelData
                width: ListView.view.width
                height: 24
                font.pixelSize: 12
                text: qsTr("Line %1: %2").arg(modelData.line).arg(modelData.message)
                Material.foreground: Look.bad
                onClicked: editor.goTo(modelData.line)
            }
        }
    }
}
