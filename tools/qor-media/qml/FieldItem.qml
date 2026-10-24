import QtQuick
import QtQuick.Controls.Basic
import QOR.Media

// A field of a PDF's form on its page (PdfForms, L9.5), over the widget PDFium draws in the page's image. While it can be
// filled it is tinted, and a tap, Space or Return fills it: a check box is ticked or not, a radio button chosen, a list's
// options shown under it to choose from, a text field typed in where it is. The words are typed in the look's face, scaled
// with the page as everything on it is, in the field's own ink on the paper's white; Return (Ctrl+Return where it takes
// lines) or Tab keeps them, and Esc leaves them as they were (the reader's). Whatever is filled shows in the page's image
// at once, drawn by PDFium as it will be saved. It sits on the page's paper, in the page's points times `zoom`; the reader
// holds every value and decides each tap.
FocusScope {
    id: fieldItem
    objectName: "formField"
    required property var field
    required property real zoom
    // Whether a tap does anything here; whether the keyboard is on it, whether the keys brought it (which shows the focus
    // ring, as it shows for the keyboard alone), and whether it is typed in or its options shown.
    property bool fillable: false
    property bool current: false
    property bool byKeys: false
    property bool typing: false
    property bool listing: false
    // What it holds now, the person's or the file's: its words, and a list's options chosen.
    property string words
    property var chosen: []
    property bool checked: false
    signal activated(bool byKeys)
    signal edited(string text)
    signal kept()
    signal stepped(bool back)
    signal chose(var places)
    signal wrote(string text)
    signal listClosed()

    // A size of the look's, scaled to the size of the field's words at the page's zoom; 0 in the field is sized to fit.
    readonly property real pointSize: field.fontSize > 0 ? field.fontSize : Math.max(4, Math.min(12, field.rect.height * 0.66))
    readonly property real typeScale: Math.max(0.05, pointSize * zoom / Theme.sizeBody)
    // What the form calls it for a person, never its name inside the form, which is often a path no person reads.
    readonly property string tip: [field.label,
                                   field.required ? qsTr("Required") : "",
                                   field.kind === "signature" && fillable ? qsTr("Sign here: a picture of your signature, which certifies nothing.")
                                   : !fillable && field.why.length ? field.why : ""].filter(Boolean).join(" · ")

    x: field.rect.x * zoom
    y: field.rect.y * zoom
    width: field.rect.width * zoom
    height: field.rect.height * zoom

    Accessible.role: field.kind === "text" ? Accessible.EditableText : field.kind === "check" ? Accessible.CheckBox
                     : field.kind === "radio" ? Accessible.RadioButton : field.kind === "combo" ? Accessible.ComboBox
                     : field.kind === "list" ? Accessible.List : Accessible.Button
    Accessible.name: field.label.length ? field.label : field.name
    Accessible.checkable: field.kind === "check" || field.kind === "radio"
    Accessible.checked: checked

    // The keyboard comes here when the reader puts it here, and back here when typing ends by Return or Esc, or the options
    // close. Words kept because the keyboard went elsewhere leave it there.
    property bool editorHasKeys: false
    function claim() {
        if (current && !typing && !listing) fieldItem.forceActiveFocus()
    }
    onCurrentChanged: claim()
    onTypingChanged: if (!typing && editorHasKeys) claim()
    onListingChanged: {
        if (listing) {
            place()
            options.open()
        } else {
            options.close()
        }
        claim()
    }
    Component.onCompleted: {
        if (listing) {
            place()
            options.open()
        }
        claim()
    }

    // Tinted while it can be filled, a required one outlined; the focus ring while the keyboard is on it.
    Rectangle {
        anchors.fill: parent
        visible: fieldItem.fillable && !fieldItem.typing
        color: Qt.alpha(Theme.accent, hover.hovered ? 0.2 : 0.1)
        border.width: fieldItem.field.required ? 1 : 0
        border.color: Theme.accent
    }
    FocusRing { visible: fieldItem.activeFocus && fieldItem.byKeys && !fieldItem.typing; radius: 2 }

    HoverHandler {
        id: hover
        cursorShape: fieldItem.fillable ? (fieldItem.field.kind === "text" ? Qt.IBeamCursor : Qt.PointingHandCursor) : Qt.ArrowCursor
    }
    ToolTip.visible: hover.hovered && !fieldItem.typing && !fieldItem.listing && tip.length > 0
    ToolTip.delay: 600
    ToolTip.text: tip

    // Its own press: the page under it hears no tap, and no words are selected across it.
    MouseArea {
        anchors.fill: parent
        enabled: !fieldItem.typing
        onClicked: fieldItem.activated(false)
    }

    Keys.onPressed: (event) => {
        switch (event.key) {
        case Qt.Key_Space: case Qt.Key_Return: case Qt.Key_Enter: fieldItem.activated(true); break
        case Qt.Key_Tab: fieldItem.stepped((event.modifiers & Qt.ShiftModifier) !== 0); break
        case Qt.Key_Backtab: fieldItem.stepped(true); break
        // The field's, never the page's.
        case Qt.Key_Delete: break
        default: return
        }
        event.accepted = true
    }

    // Typing.
    Loader {
        id: editor
        active: fieldItem.typing
        sourceComponent: fieldItem.field.multiline ? lines : line
        onLoaded: {
            editor.item.forceActiveFocus()
            editor.item.cursorPosition = editor.item.length
        }
    }
    function editorKey(event, multiline) {
        if (event.key === Qt.Key_Tab || event.key === Qt.Key_Backtab) {
            fieldItem.stepped(event.key === Qt.Key_Backtab || (event.modifiers & Qt.ShiftModifier) !== 0)
            event.accepted = true
        } else if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && (!multiline || (event.modifiers & Qt.ControlModifier))) {
            fieldItem.kept()
            event.accepted = true
        }
    }
    function editorFocus(has) {
        fieldItem.editorHasKeys = has
        if (!has && fieldItem.typing) fieldItem.kept()
    }
    Component {
        id: line
        TextField {
            objectName: "fieldEditor"
            width: fieldItem.width / fieldItem.typeScale
            height: fieldItem.height / fieldItem.typeScale
            scale: fieldItem.typeScale
            transformOrigin: Item.TopLeft
            leftPadding: 2 * fieldItem.zoom / fieldItem.typeScale
            rightPadding: leftPadding
            topPadding: 0
            bottomPadding: 0
            verticalAlignment: TextInput.AlignVCenter
            horizontalAlignment: [TextInput.AlignLeft, TextInput.AlignHCenter, TextInput.AlignRight][fieldItem.field.align]
            maximumLength: fieldItem.field.maxLength > 0 ? fieldItem.field.maxLength : 32767
            echoMode: fieldItem.field.secret ? TextInput.Password : TextInput.Normal
            text: fieldItem.words
            color: fieldItem.field.colour
            selectionColor: Qt.alpha(Theme.accent, 0.35)
            selectedTextColor: fieldItem.field.colour
            font.family: Theme.text
            font.pixelSize: Theme.sizeBody
            // The paper's white, a Rectangle's own; an edge in the focus colour, two pixels on screen at any zoom.
            background: Rectangle { border.width: 2 / fieldItem.typeScale; border.color: Theme.focus }
            onTextEdited: fieldItem.edited(text)
            onActiveFocusChanged: fieldItem.editorFocus(activeFocus)
            Keys.onPressed: (event) => fieldItem.editorKey(event, false)
        }
    }
    Component {
        id: lines
        TextArea {
            objectName: "fieldEditor"
            width: fieldItem.width / fieldItem.typeScale
            height: fieldItem.height / fieldItem.typeScale
            scale: fieldItem.typeScale
            transformOrigin: Item.TopLeft
            leftPadding: 2 * fieldItem.zoom / fieldItem.typeScale
            rightPadding: leftPadding
            topPadding: leftPadding
            bottomPadding: 0
            clip: true
            wrapMode: TextEdit.Wrap
            horizontalAlignment: [TextEdit.AlignLeft, TextEdit.AlignHCenter, TextEdit.AlignRight][fieldItem.field.align]
            text: fieldItem.words
            color: fieldItem.field.colour
            selectionColor: Qt.alpha(Theme.accent, 0.35)
            selectedTextColor: fieldItem.field.colour
            font.family: Theme.text
            font.pixelSize: Theme.sizeBody
            background: Rectangle { border.width: 2 / fieldItem.typeScale; border.color: Theme.focus }
            onTextChanged: {
                // A field's limit holds here too, as a line's does.
                if (fieldItem.field.maxLength > 0 && length > fieldItem.field.maxLength) remove(fieldItem.field.maxLength, length)
                else if (text !== fieldItem.words) fieldItem.edited(text)
            }
            onActiveFocusChanged: fieldItem.editorFocus(activeFocus)
            Keys.onPressed: (event) => fieldItem.editorKey(event, true)
        }
    }

    // A list's options, under the field (or over it, near the window's foot): a tap, Return or Space chooses one; a list
    // that takes several ticks each and stays open. A drop-down list that takes words of its own has a line for them first.
    function place() {
        const r = fieldItem.mapToItem(null, 0, 0, fieldItem.width, fieldItem.height)
        const room = options.parent ? options.parent.height : 0
        const tall = Math.min(optionList.contentHeight, 280) + (own.visible ? own.implicitHeight + 6 : 0) + 2 * options.padding
        options.width = Math.max(220, r.width)
        options.x = Math.max(0, r.x)
        options.y = r.y + r.height + 4 + tall > room ? Math.max(0, r.y - tall - 4) : r.y + r.height + 4
    }
    function pick(index) {
        if (index < 0 || index >= fieldItem.field.options.length) return
        if (fieldItem.field.kind === "list" && fieldItem.field.multiple) {
            const now = fieldItem.chosen.slice()
            const at = now.indexOf(index)
            if (at >= 0) now.splice(at, 1)
            else now.push(index)
            fieldItem.chose(now.sort((a, b) => a - b))
        } else {
            fieldItem.chose([index])
        }
    }
    Popup {
        id: options
        objectName: "fieldOptions"
        parent: Overlay.overlay
        padding: 6
        focus: true
        closePolicy: Popup.CloseOnPressOutside
        onOpened: fieldItem.field.editable ? own.forceActiveFocus() : optionList.forceActiveFocus()
        onClosed: if (fieldItem.listing) fieldItem.listClosed()
        background: Rectangle { radius: Theme.radiusPanel; color: Theme.raised; border.color: Theme.edge }
        contentItem: Column {
            spacing: 6
            TextField {
                id: own
                objectName: "fieldOwnWords"
                visible: fieldItem.field.editable
                width: optionList.width
                text: fieldItem.words
                placeholderText: qsTr("Or type your own")
                color: Theme.ink
                placeholderTextColor: Theme.inkFaint
                font.family: Theme.text
                font.pixelSize: Theme.sizeBody
                background: Rectangle { radius: Theme.radiusControl; color: Theme.well; border.color: own.activeFocus ? Theme.focus : Theme.edge }
                onAccepted: fieldItem.wrote(text)
                Keys.onDownPressed: optionList.forceActiveFocus()
            }
            ListView {
                id: optionList
                objectName: "fieldOptionList"
                width: options.availableWidth
                height: Math.min(contentHeight, 280)
                clip: true
                model: fieldItem.field.options
                keyNavigationEnabled: true
                boundsBehavior: Flickable.StopAtBounds
                currentIndex: fieldItem.chosen.length ? fieldItem.chosen[0] : 0
                ScrollBar.vertical: ScrollBar { }
                Keys.onReturnPressed: fieldItem.pick(currentIndex)
                Keys.onEnterPressed: fieldItem.pick(currentIndex)
                Keys.onSpacePressed: fieldItem.pick(currentIndex)
                delegate: Rectangle {
                    id: option
                    objectName: "fieldOption"
                    required property int index
                    required property string modelData
                    readonly property bool picked: fieldItem.chosen.indexOf(index) >= 0
                    width: optionList.width
                    height: 32
                    radius: Theme.radiusControl
                    color: optionHover.hovered || optionList.activeFocus && optionList.currentIndex === index ? Theme.surface : "transparent"
                    border.width: optionList.activeFocus && optionList.currentIndex === index ? 1 : 0
                    border.color: Theme.focus
                    // A tick box where several are taken, a dot where one is.
                    Item {
                        id: mark
                        readonly property bool several: fieldItem.field.kind === "list" && fieldItem.field.multiple
                        anchors { left: parent.left; leftMargin: 8; verticalCenter: parent.verticalCenter }
                        width: 14
                        height: 14
                        Rectangle {
                            anchors.fill: parent
                            visible: mark.several
                            radius: 2
                            color: option.picked ? Theme.accent : "transparent"
                            border.width: 1
                            border.color: option.picked ? Theme.accent : Theme.inkMuted
                        }
                        Rectangle {
                            anchors.fill: parent
                            visible: !mark.several
                            radius: width / 2
                            color: option.picked ? Theme.accent : "transparent"
                            border.width: 1
                            border.color: option.picked ? Theme.accent : Theme.inkMuted
                        }
                    }
                    Text {
                        anchors { left: mark.right; leftMargin: 10; right: parent.right; rightMargin: 8; verticalCenter: parent.verticalCenter }
                        text: option.modelData
                        elide: Text.ElideRight
                        color: option.picked ? Theme.ink : Theme.inkBody
                        font.family: Theme.text
                        font.pixelSize: Theme.sizeBody
                    }
                    HoverHandler { id: optionHover; cursorShape: Qt.PointingHandCursor }
                    TapHandler { onTapped: fieldItem.pick(option.index) }
                }
            }
        }
    }
}
