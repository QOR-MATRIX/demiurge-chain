import QtQuick
import QtQuick.Window
import QOR.Media

// QOR Media's one window: the player, the viewer or the editor, by what is open (ADR-094; docs/blueprints/library.md
// section 4). Docked inside the launcher it is borderless and sits over the Library; started by hand it is a window.
Window {
    id: root
    required property var link
    required property bool launched
    required property string startPath

    property string path
    property string kind
    property real resumeAt: 0

    width: 1280
    height: 760
    minimumWidth: 480
    minimumHeight: 320
    visible: !launched && startPath.length > 0
    color: Theme.void_
    title: path ? Files.nameOf(path) + " — QOR Media" : "QOR Media"
    flags: launched ? Qt.FramelessWindowHint : Qt.Window

    // The stock controls (scroll bars, tool tips, the find field's caret and selection) take Theme's colours, never the
    // style's own.
    palette.window: Theme.void_
    palette.base: Theme.well
    palette.text: Theme.ink
    palette.windowText: Theme.ink
    palette.button: Theme.raised
    palette.buttonText: Theme.ink
    palette.mid: Theme.edge
    palette.dark: Theme.inkMuted
    palette.toolTipBase: Theme.raised
    palette.toolTipText: Theme.ink
    palette.highlight: Theme.accent
    palette.highlightedText: Theme.inkOnAccent
    palette.placeholderText: Theme.inkFaint

    function open(filePath, fileKind, position) {
        // Unsaved words are asked about first.
        if (stage.item && stage.item.dirty) {
            root.show()
            stage.item.ask(() => root.openNow(filePath, fileKind, position))
            return
        }
        openNow(filePath, fileKind, position)
    }

    function openNow(filePath, fileKind, position) {
        // What was open says where it stopped before it goes.
        if (stage.item && stage.item.leaving)
            stage.item.leaving()
        path = filePath
        kind = fileKind || Files.kindOf(filePath)
        resumeAt = position || 0
        stage.sourceComponent = viewFor(path, kind)
        root.show()
        root.raise()
        root.requestActivate()
    }

    function viewFor(filePath, fileKind) {
        if (fileKind === "video" || fileKind === "audio")
            return playerView
        if (fileKind === "picture")
            return viewerView
        if (fileKind === "document" && Files.isText(filePath))
            return editorView
        return elsewhereView
    }

    // Back to the Library: what is open says where it stopped, QOR Media hides, the launcher is told.
    function close() {
        if (stage.item && stage.item.dirty) {
            stage.item.ask(() => root.close())
            return
        }
        if (stage.item && stage.item.leaving)
            stage.item.leaving()
        stage.sourceComponent = null
        path = ""
        if (Dock.fullScreen)
            Dock.toggleFullScreen(root)
        if (launched) {
            link.reportClosed()
            root.hide()
        } else {
            Qt.quit()
        }
    }

    Connections {
        target: root.link
        function onOpenRequested(filePath, fileKind, position) { root.open(filePath, fileKind, position) }
        function onPlaceRequested(rect, parent) { Dock.place(root, rect, parent) }
        function onHideRequested() {
            if (stage.item && stage.item.pauseForHide)
                stage.item.pauseForHide()
            root.hide()
        }
        function onThemeRequested(colors) { Theme.apply(colors) }
    }

    Component.onCompleted: {
        if (!launched && startPath.length > 0)
            open(startPath, Files.kindOf(startPath), 0)
    }

    Rectangle {
        anchors.fill: parent
        color: Theme.void_
    }

    Loader {
        id: stage
        anchors.fill: parent
        focus: true
    }

    Shortcut {
        sequence: "Esc"
        enabled: root.path.length > 0 && !(stage.item && stage.item.consumesEscape)
        onActivated: Dock.fullScreen ? Dock.toggleFullScreen(root) : root.close()
    }

    Component {
        id: playerView
        Player {
            path: root.path
            kind: root.kind
            resumeAt: root.resumeAt
            link: root.link
            onBack: root.close()
            onFullScreenToggled: Dock.toggleFullScreen(root)
        }
    }
    Component {
        id: viewerView
        Viewer {
            path: root.path
            link: root.link
            onBack: root.close()
            onFullScreenToggled: Dock.toggleFullScreen(root)
        }
    }
    Component {
        id: editorView
        Editor {
            path: root.path
            onBack: root.close()
        }
    }
    Component {
        id: elsewhereView
        Elsewhere {
            path: root.path
            onBack: root.close()
        }
    }
}
