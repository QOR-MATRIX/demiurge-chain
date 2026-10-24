import QQ

Scene {
    name: "orbital"
    skyTop: "#060a1c"
    skyHorizon: "#3a2410"
    groundHorizon: "#120c08"
    groundBottom: "#020203"
    sky: "../assets/space.hdr"
    skyTurn: 0
    skyLight: 0.45
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
        elevation: 46
        azimuth: -28
        color: "#ffe6c2"
        brightness: 0.9
    }

    Group {
        name: "Orbit"
        position: Qt.vector3d(0, 1, 0)
        eulerRotation: Qt.vector3d(0, 25, 0)
        scale: Qt.vector3d(1, 1, 1)

        Shape {
            name: "Core"
            position: Qt.vector3d(0, 0, 0)
            eulerRotation: Qt.vector3d(0, 0, 0)
            scale: Qt.vector3d(1.3, 1.3, 1.3)
            form: "sphere"
            colour: "#c9a45c"
            metalness: 0.35
            roughness: 0.55
            emissive: "#ff7a1a"
            emissivePower: 0.55
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
            name: "Embers"
            position: Qt.vector3d(0, -0.4, 0)
            colour: "#ff9a4a"
            rate: 18
            life: 2.4
            size: 0.07
            speed: 0.6
            spread: 0.9
        }

        Shape {
            name: "Satellite"
            position: Qt.vector3d(3.2, 0, 0)
            eulerRotation: Qt.vector3d(0, 0, 0)
            scale: Qt.vector3d(0.24, 0.24, 0.24)
            form: "sphere"
            colour: "#fff3d6"
            metalness: 0
            roughness: 0.3
            emissive: "#fff3d6"
            emissivePower: 4
            image: ""
            imageSize: 1
            normalMap: ""
            crisp: false
            body: "none"
            mass: 1
            bounce: 0.2
            friction: 0.6
        }
    }

    Behaviour {
        name: "Orbital"
        source: "../logic/orbital.qml"
        target: ""
    }

    Music {
        name: "Music"
        source: "../assets/orbit.wav"
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
        shade: 0.5
    }

    ScreenText {
        name: "Title"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -150
        text: "ORBITAL"
        size: 96
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Tagline"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -70
        text: "Find order in the chaos."
        size: 26
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenText {
        name: "How"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 250
        text: "Lock the orbit as the satellite crosses the amber arc: Space, Enter, the pad's A, or Lock. Ten locks complete it; three misses lose it."
        size: 18
        colour: "#9ba2ad"
        typeface: "ui"
    }

    ScreenButton {
        name: "Play"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 316
        width: 0
        text: "Play"
        colour: "#c9a45c"
        action: ""
    }

    ScreenText {
        name: "Locks"
        screen: "hud"
        anchor: "top-left"
        insetX: 40
        insetY: 32
        text: "LOCKS 0 / 10"
        size: 28
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenText {
        name: "Misses"
        screen: "hud"
        anchor: "top-left"
        insetX: 42
        insetY: 74
        text: "MISSES 0 / 3"
        size: 18
        colour: "#c9a45c"
        typeface: "ui"
    }

    ScreenText {
        name: "Message"
        screen: "hud"
        anchor: "top"
        insetX: 0
        insetY: 36
        text: "Catch the amber arc."
        size: 24
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
        name: "Lock"
        screen: "hud"
        anchor: "bottom"
        insetX: 0
        insetY: 40
        width: 220
        text: "Lock orbit"
        colour: "#ffb466"
        action: "lock"
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
        text: "ORBIT COMPLETE"
        size: 72
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Score"
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
        name: "lock"
        keys: "Space, Enter"
        pad: "A"
    }

    InputAction {
        name: "pause"
        keys: "P"
        pad: "Start"
    }
}
