import QQ

Scene {
    name: "rift-survivor"
    skyTop: "#040a0e"
    skyHorizon: "#0c1a20"
    groundHorizon: "#081014"
    groundBottom: "#020304"
    sky: "../assets/rift.hdr"
    skyTurn: 0
    skyLight: 0.55
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
        elevation: 62
        azimuth: -20
        color: "#e6f4ff"
        brightness: 0.7
    }

    Group {
        name: "Arena"
        position: Qt.vector3d(0, 1, 0)
        eulerRotation: Qt.vector3d(0, 25, 0)
        scale: Qt.vector3d(1, 1, 1)

        Group {
            name: "Board"
            position: Qt.vector3d(0, -0.4, 0)
            eulerRotation: Qt.vector3d(40, 0, 0)
            scale: Qt.vector3d(0.82, 0.82, 0.82)

            Shape {
                name: "Floor"
                position: Qt.vector3d(0, -0.02, 0)
                eulerRotation: Qt.vector3d(-90, 0, 0)
                scale: Qt.vector3d(16.6, 10.5, 1)
                form: "plane"
                colour: "#0e1d27"
                metalness: 0.3
                roughness: 0.6
                emissive: "#000000"
                emissivePower: 0
                image: ""
                imageSize: 1
                normalMap: ""
                crisp: false
                body: "none"
                mass: 1
                bounce: 0.2
                friction: 0.6
            }

            Shape {
                name: "Ship"
                position: Qt.vector3d(0, 0.15, 0)
                eulerRotation: Qt.vector3d(-90, 0, 0)
                scale: Qt.vector3d(0.55, 0.8, 0.18)
                form: "cone"
                colour: "#ffe7b1"
                metalness: 0.5
                roughness: 0.3
                emissive: "#ffd08a"
                emissivePower: 0.5
                image: ""
                imageSize: 1
                normalMap: ""
                crisp: false
                body: "none"
                mass: 1
                bounce: 0.2
                friction: 0.6
            }

            Emitter {
                name: "Wake"
                position: Qt.vector3d(0, 0.15, 0)
                colour: "#f7c887"
                rate: 24
                life: 0.6
                size: 0.05
                speed: 0.25
                spread: 0.4
            }
        }
    }

    Behaviour {
        name: "Rift Survivor"
        source: "../logic/rift-survivor.qml"
        target: ""
    }

    Music {
        name: "Music"
        source: "../assets/run.wav"
        volume: 0.45
        fade: 1.5
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
        shade: 0.55
    }

    ScreenText {
        name: "Title"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -190
        text: "RIFT SURVIVOR"
        size: 92
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Tagline"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -110
        text: "One signal. Ninety seconds. Evolve."
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
        text: "Move with W, A, S and D, the arrows, the pad's d-pad, or the buttons below; the ship fires by itself. Keep out of the amber rifts. Every so many kills, choose a perk."
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
        name: "Score"
        screen: "hud"
        anchor: "top-left"
        insetX: 40
        insetY: 32
        text: "SCORE 0"
        size: 28
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenText {
        name: "Chain"
        screen: "hud"
        anchor: "top-left"
        insetX: 42
        insetY: 74
        text: "CHAIN ×1.0"
        size: 18
        colour: "#74cdd3"
        typeface: "ui"
    }

    ScreenText {
        name: "Hull"
        screen: "hud"
        anchor: "top-left"
        insetX: 42
        insetY: 104
        text: "HULL 4 / 4"
        size: 18
        colour: "#c9a45c"
        typeface: "ui"
    }

    ScreenText {
        name: "Clock"
        screen: "hud"
        anchor: "top"
        insetX: 0
        insetY: 30
        text: "90"
        size: 40
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Level"
        screen: "hud"
        anchor: "top"
        insetX: 0
        insetY: 88
        text: "LEVEL 1  ·  0 / 10"
        size: 16
        colour: "#9ba2ad"
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
        name: "Up"
        screen: "hud"
        anchor: "bottom-left"
        insetX: 130
        insetY: 130
        width: 96
        text: "Up"
        colour: "#74cdd3"
        action: "up"
    }

    ScreenButton {
        name: "Left"
        screen: "hud"
        anchor: "bottom-left"
        insetX: 28
        insetY: 72
        width: 96
        text: "Left"
        colour: "#74cdd3"
        action: "left"
    }

    ScreenButton {
        name: "Right"
        screen: "hud"
        anchor: "bottom-left"
        insetX: 232
        insetY: 72
        width: 96
        text: "Right"
        colour: "#74cdd3"
        action: "right"
    }

    ScreenButton {
        name: "Down"
        screen: "hud"
        anchor: "bottom-left"
        insetX: 130
        insetY: 14
        width: 96
        text: "Down"
        colour: "#74cdd3"
        action: "down"
    }

    ScreenPanel {
        name: "Evolve veil"
        screen: "evolve"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#05060a"
        shade: 0.6
    }

    ScreenText {
        name: "Evolved"
        screen: "evolve"
        anchor: "centre"
        insetX: 0
        insetY: -170
        text: "SIGNAL EVOLVED  ·  LEVEL 2"
        size: 20
        colour: "#c9a45c"
        typeface: "ui"
    }

    ScreenText {
        name: "Rewrite"
        screen: "evolve"
        anchor: "centre"
        insetX: 0
        insetY: -120
        text: "Rewrite your survival."
        size: 44
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Held"
        screen: "evolve"
        anchor: "centre"
        insetX: 0
        insetY: -66
        text: "Time holds. Choose your next perk: 1, 2 or 3, the pad's X, Y or B, or a button."
        size: 18
        colour: "#9ba2ad"
        typeface: "ui"
    }

    ScreenButton {
        name: "Overclock"
        screen: "evolve"
        anchor: "centre"
        insetX: -300
        insetY: 20
        width: 260
        text: "1  Overclock"
        colour: "#ffb466"
        action: ""
    }

    ScreenButton {
        name: "Fracture"
        screen: "evolve"
        anchor: "centre"
        insetX: 0
        insetY: 20
        width: 260
        text: "2  Fracture"
        colour: "#74cdd3"
        action: ""
    }

    ScreenButton {
        name: "Second wind"
        screen: "evolve"
        anchor: "centre"
        insetX: 300
        insetY: 20
        width: 260
        text: "3  Second wind"
        colour: "#c9a45c"
        action: ""
    }

    ScreenText {
        name: "Overclock does"
        screen: "evolve"
        anchor: "centre"
        insetX: -300
        insetY: 74
        text: "Fire a quarter faster."
        size: 16
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "Fracture does"
        screen: "evolve"
        anchor: "centre"
        insetX: 0
        insetY: 74
        text: "Split each shot into a wider burst."
        size: 16
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "Second wind does"
        screen: "evolve"
        anchor: "centre"
        insetX: 300
        insetY: 74
        text: "Repair two hull; draw fragments from further."
        size: 16
        colour: "#d6d1c6"
        typeface: "ui"
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
        text: "RIFT SURVIVED"
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
        name: "perk1"
        keys: "1"
        pad: "X"
    }

    InputAction {
        name: "perk2"
        keys: "2"
        pad: "Y"
    }

    InputAction {
        name: "perk3"
        keys: "3"
        pad: "B"
    }

    InputAction {
        name: "pause"
        keys: "P"
        pad: "Start"
    }
}
