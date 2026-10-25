import QtQuick
import QtQuick.Window
import QQ
import ARQADE

// ARQADE's one window (ADR-102). Docked inside the launcher it is borderless, sits over the launcher's ARQADE screen and
// shows when the launcher first places it; started by hand it is an ordinary window. It shows the home, or a game rebuilt
// in QQ (P7.20) or an arena (P7.24) in its place while one is played.
Window {
    id: root
    required property var link
    required property var server
    required property bool launched

    // Who is playing, from the launcher: { signed_in, name, level, sparks, cgt }.
    property var player: ({ signed_in: false })

    /// The game being played, by its card's kind ("orbital"), or "" on the home.
    property string playing: ""
    /// Every game a card plays, by its kind: all the launcher may ask to open (ADR-118).
    readonly property var games: ["orbital", "void", "synapse", "rift", "flux", "reversi"]

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
        // A `qor://arqade` link named a game (ADR-118): it opens as its card would, in place of any being played.
        function onPlayRequested(game) {
            if (root.games.indexOf(game) >= 0)
                root.playing = game
        }
        function onPrefsChanged(prefs) {
            Theme.reducedMotion = prefs.reduced_motion === true
            // And the games', which otherwise follow Windows' own setting.
            Look.reducedMotion = Theme.reducedMotion
        }
    }

    // Asked of ARQADE's server only while the window shows.
    Binding {
        target: root.server
        property: "active"
        value: root.visible
    }

    Home {
        objectName: "home"
        anchors.fill: parent
        visible: root.playing === ""
        link: root.link
        server: root.server
        player: root.player
        launched: root.launched
        onPlayRequested: (kind) => root.playing = kind
    }

    Loader {
        id: game
        anchors.fill: parent
        active: root.playing !== ""
        sourceComponent: GameView {
            kind: root.playing
            server: root.server
            onClosed: root.playing = ""
        }
    }

    /// Put the game away in a safe order, its bodies before their world: what ARQADE does as it quits.
    function release() {
        if (game.item)
            game.item.release()
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
