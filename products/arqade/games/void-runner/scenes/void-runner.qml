import QQ

Scene {
    name: "void-runner"
    skyTop: "#05060c"
    skyHorizon: "#4a1c0c"
    groundHorizon: "#120a07"
    groundBottom: "#020203"
    sky: "../assets/collapse.hdr"
    skyTurn: 0
    skyLight: 0.35
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
        elevation: 24
        azimuth: 180
        color: "#ffb27a"
        brightness: 0.3
    }

    Group {
        name: "Track"
        position: Qt.vector3d(0, 0, 0)
        eulerRotation: Qt.vector3d(0, 25, 0)
        scale: Qt.vector3d(1, 1, 1)

        Shape {
            name: "Floor"
            position: Qt.vector3d(0, 0, -40)
            eulerRotation: Qt.vector3d(-90, 0, 0)
            scale: Qt.vector3d(70, 110, 1)
            form: "plane"
            colour: "#0a0b10"
            metalness: 0.25
            roughness: 0.7
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
            name: "Collapse"
            position: Qt.vector3d(0, -4, -80)
            eulerRotation: Qt.vector3d(0, 0, 0)
            scale: Qt.vector3d(28, 28, 28)
            form: "sphere"
            colour: "#3a1206"
            metalness: 0
            roughness: 1
            emissive: "#ff3a0a"
            emissivePower: 0.75
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
            position: Qt.vector3d(0, 0.35, 4.5)
            eulerRotation: Qt.vector3d(-90, 0, 0)
            scale: Qt.vector3d(0.6, 0.95, 0.2)
            form: "cone"
            colour: "#f3e0c0"
            metalness: 0.6
            roughness: 0.3
            emissive: "#ffb070"
            emissivePower: 0.45
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
            name: "Engine"
            position: Qt.vector3d(0, 0.35, 5)
            eulerRotation: Qt.vector3d(0, 0, 0)
            scale: Qt.vector3d(0.24, 0.16, 0.24)
            form: "sphere"
            colour: "#ff6a1a"
            metalness: 0
            roughness: 0.5
            emissive: "#ff5a14"
            emissivePower: 0.9
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
            position: Qt.vector3d(0, 0.35, 5.2)
            colour: "#ff9a4a"
            rate: 40
            life: 0.7
            size: 0.06
            speed: 0.3
            spread: 0.5
        }
    }

    Behaviour {
        name: "Void Runner"
        source: "../logic/void-runner.qml"
        target: ""
    }

    Music {
        name: "Music"
        source: "../assets/run.wav"
        volume: 0.5
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
        shade: 0.45
    }

    ScreenText {
        name: "Title"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -170
        text: "VOID RUNNER"
        size: 92
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Tagline"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -90
        text: "Outrun the collapse."
        size: 26
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "How"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 240
        text: "Change lane with A and D, the arrows, the pad, or Left and Right. Catch the gold; dodge the red. Survive forty-five seconds: three hits end the run."
        size: 18
        colour: "#9ba2ad"
        typeface: "ui"
    }

    ScreenButton {
        name: "Play"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 306
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
        name: "Hull"
        screen: "hud"
        anchor: "top-left"
        insetX: 42
        insetY: 74
        text: "HULL 3 / 3"
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
        text: "45"
        size: 40
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Message"
        screen: "hud"
        anchor: "top"
        insetX: 0
        insetY: 90
        text: ""
        size: 22
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
        name: "Left"
        screen: "hud"
        anchor: "bottom-left"
        insetX: 40
        insetY: 40
        width: 180
        text: "Left"
        colour: "#ffb466"
        action: "left"
    }

    ScreenButton {
        name: "Right"
        screen: "hud"
        anchor: "bottom-right"
        insetX: 40
        insetY: 40
        width: 180
        text: "Right"
        colour: "#ffb466"
        action: "right"
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
        text: "RUN SURVIVED"
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
        name: "left"
        keys: "A, Left"
        pad: "Left, LB"
    }

    InputAction {
        name: "right"
        keys: "D, Right"
        pad: "Right, RB"
    }

    InputAction {
        name: "pause"
        keys: "P"
        pad: "Start"
    }
}
