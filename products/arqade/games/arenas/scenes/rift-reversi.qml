import QQ

Scene {
    name: "rift-reversi"
    skyTop: "#030709"
    skyHorizon: "#0e2226"
    groundHorizon: "#090d0f"
    groundBottom: "#020303"
    sky: "../assets/arena.hdr"
    skyTurn: 0
    skyLight: 0.6
    exposure: 1
    bloom: true
    gravity: 9.81
    screens: "lobby"
    view: "3d"
    pixelsPerUnit: 16
    viewHeight: 12
    pixelArt: false
    seed: 0

    Sun {
        name: "Sun"
        elevation: 50
        azimuth: -30
        color: "#dff6ff"
        brightness: 0.7
    }

    // A lamp beside the eye, so the pieces shine and cast their shadows on the board.
    Lamp {
        name: "Lamp"
        position: Qt.vector3d(3, 6, 7)
        color: "#fff1dc"
        brightness: 3
        reach: 14
    }

    // The board, turned nearly toward where the eye looks from, so its depth shows: the arena's logic builds it.
    Group {
        name: "Board"
        position: Qt.vector3d(0, 1, 0)
        eulerRotation: Qt.vector3d(-8, 13, 0)
        scale: Qt.vector3d(1, 1, 1)

        Emitter {
            name: "Drift"
            position: Qt.vector3d(0, 0, -1.2)
            colour: "#85c3cf"
            rate: 8
            life: 6
            size: 0.04
            speed: 0.2
            spread: 3
        }
    }

    Behaviour {
        name: "Arena"
        source: "../logic/arena.qml"
        target: ""
    }

    Music {
        name: "Music"
        source: "../assets/arena.wav"
        volume: 0.3
        fade: 2
        loops: true
        group: "music"
    }

    // The lobby: the arena, and what the player can do in it.
    ScreenPanel {
        name: "Lobby veil"
        screen: "lobby"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.55
    }

    ScreenText {
        name: "Title"
        screen: "lobby"
        anchor: "centre"
        insetX: 0
        insetY: -210
        text: "RIFT REVERSI"
        size: 88
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Tagline"
        screen: "lobby"
        anchor: "centre"
        insetX: 0
        insetY: -138
        text: "Turn their territory into your advantage."
        size: 26
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "Detail"
        screen: "lobby"
        anchor: "centre"
        insetX: 0
        insetY: -92
        text: "Bracket their discs in any direction to turn them. Hold the corners and the edges, and finish with more."
        size: 18
        colour: "#9ba2ad"
        typeface: "ui"
    }

    ScreenText {
        name: "Status"
        screen: "lobby"
        anchor: "centre"
        insetX: 0
        insetY: -30
        text: ""
        size: 22
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenButton {
        name: "Create"
        screen: "lobby-create"
        anchor: "centre"
        insetX: 0
        insetY: 50
        width: 340
        text: "Open a match"
        colour: "#c9a45c"
        action: ""
    }

    ScreenButton {
        name: "Join"
        screen: "lobby-join"
        anchor: "centre"
        insetX: 0
        insetY: 118
        width: 340
        text: "Join a match"
        colour: "#85c3cf"
        action: ""
    }

    ScreenButton {
        name: "Return"
        screen: "lobby-return"
        anchor: "centre"
        insetX: 0
        insetY: 186
        width: 340
        text: "Return to your match"
        colour: "#f0a35c"
        action: ""
    }

    ScreenText {
        name: "Ranked"
        screen: "lobby"
        anchor: "centre"
        insetX: 0
        insetY: 268
        text: "Played against another person on ARQADE's server, and ranked there, as on the website."
        size: 15
        colour: "#9ba2ad"
        typeface: "ui"
    }

    // The player's own lobby, open for a challenger.
    ScreenPanel {
        name: "Waiting veil"
        screen: "waiting"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.45
    }

    ScreenText {
        name: "Waiting title"
        screen: "waiting"
        anchor: "centre"
        insetX: 0
        insetY: -100
        text: "One universe. Two minds."
        size: 48
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Waiting line"
        screen: "waiting"
        anchor: "centre"
        insetX: 0
        insetY: -36
        text: ""
        size: 20
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenButton {
        name: "Close lobby"
        screen: "waiting"
        anchor: "centre"
        insetX: 0
        insetY: 50
        width: 320
        text: "Close this lobby"
        colour: "#c9a45c"
        action: ""
    }

    ScreenButton {
        name: "Leave waiting"
        screen: "waiting"
        anchor: "centre"
        insetX: 0
        insetY: 118
        width: 320
        text: "Back to the lobby"
        colour: "#9ba2ad"
        action: ""
    }

    // The match: who plays, whose move it is, and the seconds left for it.
    ScreenText {
        name: "Host"
        screen: "match"
        anchor: "top-left"
        insetX: 40
        insetY: 32
        text: ""
        size: 24
        colour: "#f0a35c"
        typeface: "ui"
    }

    ScreenText {
        name: "Guest"
        screen: "match"
        anchor: "top-right"
        insetX: 40
        insetY: 32
        text: ""
        size: 24
        colour: "#85c3cf"
        typeface: "ui"
    }

    ScreenText {
        name: "Turn"
        screen: "match"
        anchor: "top"
        insetX: 0
        insetY: 28
        text: ""
        size: 28
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenText {
        name: "Clock"
        screen: "match"
        anchor: "top"
        insetX: 0
        insetY: 70
        text: ""
        size: 20
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "Keys"
        screen: "match"
        anchor: "bottom"
        insetX: 0
        insetY: 104
        text: "Arrows choose a cell   ·   Q and E jump between moves   ·   Space or the pad's A places   ·   or click a cell"
        size: 15
        colour: "#9ba2ad"
        typeface: "ui"
    }

    ScreenButton {
        name: "Concede"
        screen: "match"
        anchor: "bottom-left"
        insetX: 32
        insetY: 32
        width: 0
        text: "Concede"
        colour: "#9ba2ad"
        action: ""
    }

    ScreenButton {
        name: "Leave match"
        screen: "match"
        anchor: "bottom-right"
        insetX: 32
        insetY: 32
        width: 0
        text: "Lobby"
        colour: "#9ba2ad"
        action: ""
    }

    ScreenPanel {
        name: "Concede veil"
        screen: "concede"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.5
    }

    ScreenText {
        name: "Concede ask"
        screen: "concede"
        anchor: "centre"
        insetX: 0
        insetY: -50
        text: "Concede this match to your opponent?"
        size: 30
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenButton {
        name: "Concede match"
        screen: "concede"
        anchor: "centre"
        insetX: -136
        insetY: 30
        width: 250
        text: "Concede match"
        colour: "#ff7a3a"
        action: ""
    }

    ScreenButton {
        name: "Keep playing"
        screen: "concede"
        anchor: "centre"
        insetX: 136
        insetY: 30
        width: 250
        text: "Keep playing"
        colour: "#c9a45c"
        action: ""
    }

    // The match over, as the server decided it, over the board where it was decided.
    ScreenPanel {
        name: "Result veil"
        screen: "result"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.3
    }

    ScreenText {
        name: "Result"
        screen: "result"
        anchor: "top"
        insetX: 0
        insetY: 22
        text: ""
        size: 64
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Result line"
        screen: "result"
        anchor: "top"
        insetX: 0
        insetY: 104
        text: ""
        size: 22
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenButton {
        name: "Rematch"
        screen: "result"
        anchor: "bottom"
        insetX: -170
        insetY: 40
        width: 300
        text: "Play again"
        colour: "#c9a45c"
        action: ""
    }

    ScreenButton {
        name: "Leave result"
        screen: "result"
        anchor: "bottom"
        insetX: 170
        insetY: 40
        width: 300
        text: "Back to the lobby"
        colour: "#9ba2ad"
        action: ""
    }

    ScreenPanel {
        name: "Closed veil"
        screen: "closed"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.5
    }

    ScreenText {
        name: "Closed"
        screen: "closed"
        anchor: "centre"
        insetX: 0
        insetY: -60
        text: "This lobby is closed."
        size: 40
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenButton {
        name: "Find another"
        screen: "closed"
        anchor: "centre"
        insetX: 0
        insetY: 30
        width: 320
        text: "Find another match"
        colour: "#c9a45c"
        action: ""
    }

    // Why something asked for was not done, over whatever shows.
    ScreenText {
        name: "Notice"
        screen: "notice"
        anchor: "top"
        insetX: 0
        insetY: 116
        text: ""
        size: 18
        colour: "#ffb27a"
        typeface: "ui"
    }

    InputAction {
        name: "left"
        keys: "A, Left"
        pad: "Left"
    }

    InputAction {
        name: "right"
        keys: "D, Right"
        pad: "Right"
    }

    InputAction {
        name: "up"
        keys: "W, Up"
        pad: "Up"
    }

    InputAction {
        name: "down"
        keys: "S, Down"
        pad: "Down"
    }

    InputAction {
        name: "choose"
        keys: "Space, Enter"
        pad: "A"
    }

    InputAction {
        name: "pick"
        keys: "Mouse Left"
        pad: ""
    }

    InputAction {
        name: "next"
        keys: "E"
        pad: "RB"
    }

    InputAction {
        name: "previous"
        keys: "Q"
        pad: "LB"
    }
}
