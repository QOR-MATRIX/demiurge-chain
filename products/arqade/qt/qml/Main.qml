import QtQuick
import QtQuick.Window
import ARQADE

// ARQADE's one window (ADR-102). Docked inside the launcher it is borderless, sits over the launcher's ARQADE screen and
// shows when the launcher first places it; started by hand it is an ordinary window.
Window {
    id: root
    required property var link
    required property var server
    required property bool launched

    // Who is playing, from the launcher: { signed_in, name, level, sparks, cgt }.
    property var player: ({ signed_in: false })

    width: 1360
    height: 860
    minimumWidth: 720
    minimumHeight: 480
    visible: !launched
    color: Theme.void_
    title: "ARQADE"
    flags: launched ? Qt.FramelessWindowHint : Qt.Window

    Connections {
        target: root.link
        function onPlaceRequested(rect, parent) {
            Dock.place(root, rect, parent)
            root.show()
        }
        function onHideRequested() {
            if (Dock.fullScreen)
                Dock.toggleFullScreen(root)
            root.hide()
        }
        function onThemeRequested(colors) { Theme.apply(colors) }
        function onPlayerChanged(player) {
            root.player = player
            root.server.setLauncherSignedIn(player.signed_in === true)
        }
        function onPrefsChanged(prefs) { Theme.reducedMotion = prefs.reduced_motion === true }
    }

    // Asked of ARQADE's server only while the window shows.
    Binding {
        target: root.server
        property: "active"
        value: root.visible
    }

    Home {
        anchors.fill: parent
        link: root.link
        server: root.server
        player: root.player
        launched: root.launched
    }

    Shortcut {
        sequence: "F11"
        onActivated: Dock.toggleFullScreen(root)
    }
    Shortcut {
        sequence: "Esc"
        enabled: Dock.fullScreen
        onActivated: Dock.toggleFullScreen(root)
    }
}
