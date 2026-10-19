import QQ

Scene {
    name: "sample"
    skyTop: "#0b1430"
    skyHorizon: "#c7623a"
    groundHorizon: "#2a1a1a"
    groundBottom: "#07080c"
    skyLight: 0.75
    exposure: 1
    bloom: true
    gravity: 9.81
    screens: "hud"

    Sun {
        name: "Sun"
        elevation: 38
        azimuth: -35
        color: "#fff1dc"
        brightness: 1.15
    }

    Ground {
        name: "Ground"
        extent: 40
        colour: "#151821"
    }

    Shape {
        name: "Plinth"
        position: Qt.vector3d(0, 0.25, 0)
        eulerRotation: Qt.vector3d(0, 15, 0)
        scale: Qt.vector3d(2, 0.5, 2)
        form: "cube"
        colour: "#2b3140"
        metalness: 0.2
        roughness: 0.6
        emissive: "#000000"
        emissivePower: 0
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Lamp {
        name: "Ember lamp"
        position: Qt.vector3d(1.5, 2.2, 1)
        color: "#ffb15c"
        brightness: 4
        reach: 6
    }

    Prop {
        name: "Orb"
        position: Qt.vector3d(0, 1.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../orb.gltf"
    }

    Player {
        name: "Player"
        position: Qt.vector3d(0, 0, 4)
        eulerRotation: Qt.vector3d(0, 0, 0)
        colour: "#ff6a00"
        speed: 4.5
        jumpHeight: 1.2
    }

    Emitter {
        name: "Embers"
        position: Qt.vector3d(-1.5, 0.1, 0)
        colour: "#ffb15c"
        rate: 60
        life: 2
        size: 0.12
        speed: 1.5
        spread: 0.6
    }

    Sound {
        name: "Chime"
        position: Qt.vector3d(1.5, 1, 0)
        source: "../chime.wav"
        volume: 1
        reach: 25
        loops: true
    }

    Behaviour {
        name: "Spin"
        source: "../logic/spin.qml"
        target: "Orb"
    }

    Trigger {
        name: "Gate"
        position: Qt.vector3d(0, 1, -4)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(3, 2, 0.5)
        form: "box"
    }

    ScreenText {
        name: "Score"
        screen: "hud"
        anchor: "top-left"
        insetX: 32
        insetY: 32
        text: "Orbs 0"
        size: 28
        colour: "#f3ede1"
        typeface: "ui"
    }

    ScreenBar {
        name: "Health"
        screen: "hud"
        anchor: "top-right"
        insetX: 32
        insetY: 40
        width: 240
        height: 14
        value: 0.75
        maximum: 1
        colour: "#ff6a00"
    }

    ScreenImage {
        name: "Mark"
        screen: "hud"
        anchor: "bottom-right"
        insetX: 24
        insetY: 24
        width: 64
        height: 64
        source: "../mark.png"
        crisp: true
    }

    ScreenPanel {
        name: "Veil"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#06070a"
        shade: 0.6
    }

    ScreenButton {
        name: "Resume"
        screen: "pause"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        text: "Resume"
        colour: "#c9a45c"
        action: ""
    }

    InputAction {
        name: "fire"
        keys: "F, Mouse Left"
        pad: "X, RT"
    }
}
