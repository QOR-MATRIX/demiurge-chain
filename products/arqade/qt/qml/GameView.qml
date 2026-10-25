pragma ComponentBehavior: Bound
import QtQuick
import QQ
import ARQADE

// A game rebuilt in QQ (DIRECTION P7.20; ADR-102 decision 2), played in ARQADE's window by QQ's runtime: a bar with the
// way back and the game's name, and the game beneath it. Esc, or Back, puts the game away and returns to the home.
//
// The games here are ARQADE's own, written and reviewed in this repository and carried in the executable
// (qrc:/games/<kind>/), so they play in ARQADE's engine as the QQ Player plays a person's own project. QQ's lockdown
// (ADR-087 decision 2) is the second wall around a stranger's game, and a stranger's game never plays here: it plays in
// the browser (ADR-087 decision 1).
//
// Each play is kept as a run (QQ's RunRecord): the seed it started from, what every tick took, and what it came to. A
// game's rules replay that run to the same outcome, in QQ and in Node, which is what ARQADE's service will do before it
// confirms a run (P7.22).
//
// The arenas (P7.24) are played against another person on ARQADE's server, which keeps every match and decides every
// move, as it does for the website's arcade. A QQ game reaches no server of its own: an arena asks this view, its host,
// and this view answers from the server (ADR-117; the conversation is in games/arenas/logic/arena.mjs). It hands on the
// arena's open matches from the live arcade, read every 3 seconds while the lobby shows, and the match the arena
// follows, asked after every 1.2 seconds as the website asks; it tells the game only what changed, so the run keeps
// each answer once. What it asks for, the server
// checks: this view decides nothing of a match.
Item {
    id: view

    /// Which game: its card's kind.
    required property string kind
    /// ARQADE's server (ArqadeServer): where the arenas' matches are kept and played.
    required property var server

    /// The games played here, by kind: their names and scene files.
    readonly property var games: ({
        orbital: { name: "Orbital", file: "qrc:/games/orbital/scenes/orbital.qml" },
        "void": { name: "Void Runner", file: "qrc:/games/void-runner/scenes/void-runner.qml" },
        synapse: { name: "Synapse", file: "qrc:/games/synapse/scenes/synapse.qml" },
        rift: { name: "Rift Survivor", file: "qrc:/games/rift-survivor/scenes/rift-survivor.qml" },
        flux: { name: "Flux Four", file: "qrc:/games/arenas/scenes/flux-four.qml", arena: "flux" },
        reversi: { name: "Rift Reversi", file: "qrc:/games/arenas/scenes/rift-reversi.qml", arena: "reversi" },
    })
    readonly property var game: games[kind] ?? null
    /// For an arena, its name on ARQADE's server ("flux", "reversi"); "" for a game played alone.
    readonly property string arena: game && game.arena ? game.arena : ""
    /// The server's state as a number, as Rankings reads it, to compare with ArqadeServer.SignedIn.
    readonly property int serverState: server ? server.state : 0

    /// The run so far.
    readonly property alias record: run
    /// The scene playing, or null.
    readonly property alias playing: play.scene

    /// The game has been put away, and the home should show.
    signal closed()

    // Another game begins with its host knowing nothing of the last one's.
    onKindChanged: host.reset()

    /// Put the game away in a safe order, its bodies before their world.
    function release() {
        play.release()
    }

    function leave() {
        release()
        closed()
    }

    RunRecord {
        id: run
    }

    Rectangle {
        id: bar
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: 56
        color: Theme.void_

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: Theme.line
        }

        Row {
            anchors.left: parent.left
            anchors.leftMargin: 16
            anchors.verticalCenter: parent.verticalCenter
            spacing: 18

            ArqButton {
                objectName: "back"
                text: qsTr("Back")
                implicitHeight: 34
                // Never the keyboard's: Space and Enter are the game's, and Esc is the way back.
                focusPolicy: Qt.NoFocus
                onClicked: view.leave()
            }
            Text {
                anchors.verticalCenter: parent.verticalCenter
                text: view.game ? view.game.name : ""
                font.family: Theme.display
                font.pixelSize: Theme.sizeHeading
                font.weight: Font.DemiBold
                font.letterSpacing: Theme.sizeHeading * Theme.trackingHeading
                color: Theme.ink
            }
        }

        Text {
            anchors.right: parent.right
            anchors.rightMargin: 20
            anchors.verticalCenter: parent.verticalCenter
            text: qsTr("Esc returns to ARQADE  ·  F11 fills the screen")
            font.family: Theme.ui
            font.pixelSize: Theme.sizeUi
            font.letterSpacing: Theme.sizeUi * Theme.trackingLabel
            color: Theme.inkMuted
        }
    }

    Game {
        id: play
        objectName: "game"
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: bar.bottom
        anchors.bottom: parent.bottom
        recorder: run
        file: view.game ? view.game.file : ""
    }

    // An arena's host: what the game asks, answered from ARQADE's server.
    QtObject {
        id: host

        /// The match the arena follows, by its id; whether a match it asked to create is awaited; and whether the match
        /// it asked to join or open is, which is not asked after until it comes.
        property string following: ""
        property bool creating: false
        property bool finding: false
        /// Whether the arena listens for its lobby; what it was last told of the lobby, the match and a refusal.
        property bool listening: false
        property string lobbyTold: ""
        property string matchTold: ""
        property string refusalTold: ""
        /// A poll of the match is on its way: one at a time, as the website asks.
        property bool polling: false

        /// Nothing heard or followed yet: as a game begins.
        function reset() {
            following = ""
            creating = false
            finding = false
            listening = false
            lobbyTold = ""
            matchTold = ""
            refusalTold = ""
            polling = false
        }

        /// The arena's open matches and whether the player is signed in, if they changed and the arena follows none.
        function lobby() {
            if (!listening || following !== "" || creating)
                return
            const signedIn = view.serverState === ArqadeServer.SignedIn
            const rows = (signedIn ? view.server.matches : []).filter(m => m.game === view.arena)
                .map(m => ({ id: m.id, game: m.game, status: m.status, alias: m.alias, seat: m.seat }))
            const message = { lobby: rows, signedIn: signedIn, why: signedIn ? "" : view.server.message }
            const said = JSON.stringify(message)
            if (said === lobbyTold)
                return
            lobbyTold = said
            play.tell(message)
        }

        /// The match as the server keeps it: told if it is the one followed and it changed.
        function answered(match) {
            if (creating && match.game === view.arena) {
                creating = false
                following = match.id
            }
            if (match.id !== following)
                return
            finding = false
            refusalTold = ""
            const said = JSON.stringify([match.id, match.revision, match.status])
            if (said === matchTold)
                return
            matchTold = said
            play.tell({ match: match })
        }

        /// Why something was not done; the same reason told once, however many polls meet it. A match that could not
        /// be created, joined or opened is not followed, and the lobby is told again.
        function refused(why) {
            if (following === "" && !creating && !listening)
                return
            const lost = creating || finding
            if (finding)
                follow("")
            creating = false
            if (why !== refusalTold) {
                refusalTold = why
                play.tell({ refused: why })
            }
            if (lost) {
                lobbyTold = ""
                lobby()
            }
        }

        function follow(id) {
            following = typeof id === "string" ? id : ""
            matchTold = ""
            creating = false
            finding = following !== ""
        }

        function asked(message) {
            if (!message || typeof message !== "object")
                return
            refusalTold = ""
            switch (message.ask) {
            case "lobby":
                listening = true
                lobbyTold = ""
                lobby()
                break
            case "create":
                follow("")
                creating = true
                view.server.createMatch(view.arena)
                break
            case "join":
                follow(message.id)
                view.server.actOnMatch(following, "join", 0, 0)
                break
            case "open":
                follow(message.id)
                view.server.fetchMatch(following)
                break
            case "move":
            case "resign":
            case "cancel":
                if (message.id === following && following !== "")
                    view.server.actOnMatch(following, message.ask, message.ask === "move" ? message.cell : 0,
                                           message.revision)
                break
            case "leave":
                follow("")
                lobbyTold = ""
                lobby()
                break
            }
        }
    }

    Connections {
        target: view.arena !== "" ? play : null
        function onAsked(message) { host.asked(message) }
    }

    Connections {
        target: view.arena !== "" ? view.server : null
        function onLiveChanged() { host.lobby() }
        function onChanged() { host.lobby() }
        function onMatchAnswered(match) {
            host.polling = false
            host.answered(match)
        }
        function onMatchRefused(message) {
            host.polling = false
            host.refused(message)
        }
    }

    Timer {
        interval: 1200
        repeat: true
        running: view.arena !== "" && host.following !== "" && !host.finding && play.scene !== null
        onTriggered: {
            if (host.polling)
                return
            host.polling = true
            view.server.fetchMatch(host.following)
        }
    }

    // While an arena shows its lobby, the live arcade is read every 3 seconds rather than the home's 15, so a lobby
    // opened or taken shows soon (the website reads it every 2).
    Timer {
        interval: 3000
        repeat: true
        running: view.arena !== "" && host.listening && host.following === "" && !host.creating && play.scene !== null
        onTriggered: view.server.refresh()
    }

    Text {
        visible: play.problem !== ""
        anchors.centerIn: play
        width: Math.min(implicitWidth, play.width - 48)
        wrapMode: Text.WordWrap
        text: qsTr("This game could not start: %1").arg(play.problem)
        font.family: Theme.body
        font.pixelSize: Theme.sizeBody
        color: Theme.bad
    }

    Shortcut {
        sequence: "Esc"
        // Full screen, Esc first puts the window back (Main).
        enabled: !Dock.fullScreen
        onActivated: view.leave()
    }
}
