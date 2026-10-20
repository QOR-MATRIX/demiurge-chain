import QQ

Scene {
    name: "orb-run"
    skyTop: "#0b1430"
    skyHorizon: "#c7623a"
    groundHorizon: "#2a1a1a"
    groundBottom: "#07080c"
    sky: "../assets/dusk.hdr"
    skyTurn: 0
    skyLight: 0.75
    exposure: 1
    bloom: true
    gravity: 9.81
    screens: "title"
    view: "3d"
    pixelsPerUnit: 16
    viewHeight: 12
    pixelArt: true
    seed: 0

    Sun {
        name: "Sun"
        elevation: 38
        azimuth: -35
        color: "#fff1dc"
        brightness: 1.15
    }

    Ground {
        name: "Ground"
        extent: 30
        colour: "#ffffff"
        image: "../assets/floor.png"
        imageSize: 2
    }

    Shape {
        name: "North wall"
        position: Qt.vector3d(0, 0.75, -11)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(22, 1.5, 0.5)
        form: "cube"
        colour: "#ece4d8"
        metalness: 0
        roughness: 0.85
        emissive: "#000000"
        emissivePower: 0
        image: "../assets/stone.png"
        imageSize: 1.5
        normalMap: "../assets/stone-normal.png"
        crisp: false
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "South wall"
        position: Qt.vector3d(0, 0.75, 11)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(22, 1.5, 0.5)
        form: "cube"
        colour: "#ece4d8"
        metalness: 0
        roughness: 0.85
        emissive: "#000000"
        emissivePower: 0
        image: "../assets/stone.png"
        imageSize: 1.5
        normalMap: "../assets/stone-normal.png"
        crisp: false
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "East wall"
        position: Qt.vector3d(11, 0.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.5, 1.5, 22)
        form: "cube"
        colour: "#ece4d8"
        metalness: 0
        roughness: 0.85
        emissive: "#000000"
        emissivePower: 0
        image: "../assets/stone.png"
        imageSize: 1.5
        normalMap: "../assets/stone-normal.png"
        crisp: false
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "West wall"
        position: Qt.vector3d(-11, 0.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.5, 1.5, 22)
        form: "cube"
        colour: "#ece4d8"
        metalness: 0
        roughness: 0.85
        emissive: "#000000"
        emissivePower: 0
        image: "../assets/stone.png"
        imageSize: 1.5
        normalMap: "../assets/stone-normal.png"
        crisp: false
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "Boulder"
        position: Qt.vector3d(-8, 0.6, -8)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1.2, 1.2, 1.2)
        form: "sphere"
        colour: "#5a4636"
        metalness: 0
        roughness: 0.9
        emissive: "#000000"
        emissivePower: 0
        image: ""
        imageSize: 1
        normalMap: ""
        crisp: false
        body: "dynamic"
        mass: 8
        bounce: 0.1
        friction: 0.8
    }

    Prop {
        name: "Lantern west"
        position: Qt.vector3d(-9, 0, -9)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/lantern.gltf"
        clip: "idle"
        clipSpeed: 1
        clipLoops: true
    }

    Prop {
        name: "Lantern east"
        position: Qt.vector3d(9, 0, -9)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/lantern.gltf"
        clip: "idle"
        clipSpeed: 0.9
        clipLoops: true
    }

    Spotlight {
        name: "Arena light"
        position: Qt.vector3d(0, 9, 0)
        eulerRotation: Qt.vector3d(-90, 0, 0)
        color: "#ffd9a8"
        brightness: 2.5
        reach: 20
        cone: 80
        softness: 0.6
    }

    Spotlight {
        name: "Wall wash"
        position: Qt.vector3d(0, 5, 8)
        eulerRotation: Qt.vector3d(-19.65, 0, 0)
        color: "#bcd4ff"
        brightness: 1.5
        reach: 30
        cone: 40
        softness: 0.6
    }

    Player {
        name: "Player"
        position: Qt.vector3d(0, 0, 6)
        eulerRotation: Qt.vector3d(0, 0, 0)
        colour: "#ff6a00"
        speed: 4.5
        jumpHeight: 1.2
    }

    Behaviour {
        name: "Rules"
        source: "../logic/rules.qml"
        target: ""
    }

    Music {
        name: "Music"
        source: "../assets/title.wav"
        volume: 0.6
        fade: 1.2
        loops: true
        group: "music"
    }

    ScreenText {
        name: "Score"
        screen: "hud"
        anchor: "top-left"
        insetX: 40
        insetY: 32
        text: "ORBS 0"
        size: 34
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenText {
        name: "Hint"
        screen: "hud"
        anchor: "bottom"
        insetX: 0
        insetY: 32
        text: "W A S D to move, drag to look, Space to jump"
        size: 18
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
        colour: "#c9a45c"
        action: "pause"
    }

    ScreenPanel {
        name: "Title veil"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#06070a"
        shade: 0.62
    }

    ScreenText {
        name: "Title"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -120
        text: "ORB RUN"
        size: 96
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Tagline"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: -40
        text: "Gather the orbs. Mind the boulder."
        size: 24
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenButton {
        name: "Play"
        screen: "title"
        anchor: "centre"
        insetX: 0
        insetY: 60
        width: 0
        text: "Play"
        colour: "#c9a45c"
        action: ""
    }

    ScreenPanel {
        name: "Pause veil"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#06070a"
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

    ScreenButton {
        name: "Music switch"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: 110
        width: 0
        text: "Music: on"
        colour: "#9ba2ad"
        action: ""
    }

    InputAction {
        name: "pause"
        keys: "P"
        pad: "Start"
    }
}
