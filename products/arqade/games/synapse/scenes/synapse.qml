import QQ

Scene {
    name: "synapse"
    skyTop: "#05060c"
    skyHorizon: "#14202a"
    groundHorizon: "#0a0c10"
    groundBottom: "#020203"
    sky: "../assets/mind.hdr"
    skyTurn: 0
    skyLight: 0.6
    exposure: 1
    bloom: true
    gravity: 9.81
    screens: "title"
    view: "3d"
    pixelsPerUnit: 16
    viewHeight: 12
    pixelArt: false
    seed: 0

    Sun {
        name: "Sun"
        elevation: 50
        azimuth: -30
        color: "#dfe6ff"
        brightness: 0.6
    }

    Group {
        name: "Mind"
        position: Qt.vector3d(0, 1, 0)
        eulerRotation: Qt.vector3d(0, 25, 0)
        scale: Qt.vector3d(1, 1, 1)

        Emitter {
            name: "Drift"
            position: Qt.vector3d(0, -1.6, -1)
            colour: "#7fd3cc"
            rate: 10
            life: 5
            size: 0.05
            speed: 0.25
            spread: 2.2
        }
    }

    Behaviour {
        name: "Synapse"
        source: "../logic/synapse.qml"
        target: ""
    }

    Music {
        name: "Music"
        source: "../assets/run.wav"
        volume: 0.35
        fade: 2
        loops: true
        group: "music"
    }

    ScreenPanel {
        name: "Title veil"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.5
    }

    ScreenText {
        name: "Title"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -200
        text: "SYNAPSE"
        size: 96
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Tagline"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -120
        text: "Trust the signal. Question your mind."
        size: 26
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "How"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 200
        text: "Watch the nodes light, then repeat the signal: 1 to 4, the pad's X, Y, B and A, or the buttons below. Each signal repeated grows by one; eight masters it."
        size: 18
        colour: "#9ba2ad"
        typeface: "ui"
    }

    ScreenButton {
        name: "Play"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 266
        width: 0
        text: "Play"
        colour: "#c9a45c"
        action: ""
    }

    ScreenText {
        name: "Length"
        screen: "hud"
        anchor: "top-left"
        insetX: 40
        insetY: 32
        text: "SIGNAL 1 / 8"
        size: 28
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenText {
        name: "Stage"
        screen: "hud"
        anchor: "top"
        insetX: 0
        insetY: 36
        text: "OBSERVE THE SIGNAL"
        size: 26
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenButton {
        name: "Pause"
        screen: "hud"
        anchor: "top-right"
        insetX: 32
        insetY: 28
        width: 0
        text: "Pause"
        colour: "#9ba2ad"
        action: "pause"
    }

    ScreenButton {
        name: "Origin"
        screen: "hud"
        anchor: "bottom"
        insetX: -393
        insetY: 40
        width: 240
        text: "1  Origin"
        colour: "#e8b860"
        action: "node1"
    }

    ScreenButton {
        name: "Echo"
        screen: "hud"
        anchor: "bottom"
        insetX: -131
        insetY: 40
        width: 240
        text: "2  Echo"
        colour: "#ff7a3a"
        action: "node2"
    }

    ScreenButton {
        name: "Pulse"
        screen: "hud"
        anchor: "bottom"
        insetX: 131
        insetY: 40
        width: 240
        text: "3  Pulse"
        colour: "#4fc8c0"
        action: "node3"
    }

    ScreenButton {
        name: "Aether"
        screen: "hud"
        anchor: "bottom"
        insetX: 393
        insetY: 40
        width: 240
        text: "4  Aether"
        colour: "#9a86ff"
        action: "node4"
    }

    ScreenPanel {
        name: "Pause veil"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.62
    }

    ScreenText {
        name: "Paused"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: -60
        text: "PAUSED"
        size: 64
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenButton {
        name: "Resume"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: 40
        width: 0
        text: "Resume"
        colour: "#c9a45c"
        action: ""
    }

    ScreenPanel {
        name: "Result veil"
        screen: "result"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.55
    }

    ScreenText {
        name: "Result"
        screen: "result"
        anchor: "centre"
        insetX: 0
        insetY: -110
        text: "SIGNAL MASTERED"
        size: 72
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Final"
        screen: "result"
        anchor: "centre"
        insetX: 0
        insetY: -30
        text: "SCORE 0"
        size: 30
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenButton {
        name: "Play again"
        screen: "result"
        anchor: "centre"
        insetX: 0
        insetY: 50
        width: 0
        text: "Play again"
        colour: "#c9a45c"
        action: ""
    }

    InputAction {
        name: "node1"
        keys: "1"
        pad: "X, Left"
    }

    InputAction {
        name: "node2"
        keys: "2"
        pad: "Y, Up"
    }

    InputAction {
        name: "node3"
        keys: "3"
        pad: "B, Right"
    }

    InputAction {
        name: "node4"
        keys: "4"
        pad: "A, Down"
    }

    InputAction {
        name: "pause"
        keys: "P"
        pad: "Start"
    }
}
