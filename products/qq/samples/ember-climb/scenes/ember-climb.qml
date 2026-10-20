import QQ

Scene {
    name: "ember-climb"
    skyTop: "#120d1f"
    skyHorizon: "#3a1630"
    groundHorizon: "#0a0710"
    groundBottom: "#07080c"
    skyLight: 0.75
    exposure: 1
    bloom: false
    gravity: 22
    screens: "title"
    view: "2d"
    pixelsPerUnit: 16
    viewHeight: 12
    pixelArt: true

    TileMap {
        name: "Wall"
        position: Qt.vector3d(0, 16, -0.5)
        source: "../assets/level.tmj"
        layer: "back"
        solid: false
        bounds: false
        lit: false
        colour: "#ffffff"
    }

    TileMap {
        name: "Level"
        position: Qt.vector3d(0, 16, 0)
        source: "../assets/level.tmj"
        layer: "ground"
        solid: true
        bounds: true
        lit: false
        colour: "#ffffff"
    }

    Sprite {
        name: "Torch 1"
        position: Qt.vector3d(3.5, 3.6, -0.25)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/torch.png"
        columns: 4
        rows: 1
        frame: 0
        first: 0
        count: 4
        fps: 8
        colour: "#ffffff"
        lit: false
        crisp: true
        flip: false
        body: "none"
        turns: true
        mass: 1
        bounce: 0.1
        friction: 0.6
    }

    Sprite {
        name: "Torch 2"
        position: Qt.vector3d(29.5, 3.6, -0.25)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/torch.png"
        columns: 4
        rows: 1
        frame: 1
        first: 1
        count: 4
        fps: 8
        colour: "#ffffff"
        lit: false
        crisp: true
        flip: false
        body: "none"
        turns: true
        mass: 1
        bounce: 0.1
        friction: 0.6
    }

    Sprite {
        name: "Torch 3"
        position: Qt.vector3d(42.5, 3.6, -0.25)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/torch.png"
        columns: 4
        rows: 1
        frame: 2
        first: 2
        count: 4
        fps: 8
        colour: "#ffffff"
        lit: false
        crisp: true
        flip: false
        body: "none"
        turns: true
        mass: 1
        bounce: 0.1
        friction: 0.6
    }

    Sprite {
        name: "Hero look"
        position: Qt.vector3d(2.5, 2.5, 0.1)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/hero.png"
        columns: 4
        rows: 2
        frame: 0
        first: 0
        count: 2
        fps: 3
        colour: "#ffffff"
        lit: false
        crisp: true
        flip: false
        body: "none"
        turns: true
        mass: 1
        bounce: 0.1
        friction: 0.6
    }

    Player2D {
        name: "Hero"
        position: Qt.vector3d(2.5, 2, 0)
        style: "platform"
        width: 0.75
        height: 1
        speed: 6
        jumpHeight: 3.5
        colour: "#ff6a00"
        look: "Hero look"
    }

    Behaviour {
        name: "Rules"
        source: "../logic/rules.qml"
        target: ""
    }

    ScreenText {
        name: "Embers"
        screen: "hud"
        anchor: "top-left"
        insetX: 40
        insetY: 32
        text: "EMBERS 0 / 8"
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
        text: "A and D to run, Space to jump, P to pause"
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
        text: "EMBER CLIMB"
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
        text: "Gather the embers. Light the forge."
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

    ScreenPanel {
        name: "Won veil"
        screen: "won"
        anchor: "centre"
        insetX: 0
        insetY: 0
        width: 0
        height: 0
        colour: "#06070a"
        shade: 0.62
    }

    ScreenText {
        name: "Lit"
        screen: "won"
        anchor: "centre"
        insetX: 0
        insetY: -100
        text: "THE FORGE IS LIT"
        size: 80
        colour: "#ead39a"
        typeface: "display"
    }

    ScreenText {
        name: "Took"
        screen: "won"
        anchor: "centre"
        insetX: 0
        insetY: -20
        text: "Climbed in 0:00"
        size: 24
        colour: "#d6d1c6"
        typeface: "ui"
    }

    ScreenButton {
        name: "Again"
        screen: "won"
        anchor: "centre"
        insetX: 0
        insetY: 60
        width: 0
        text: "Again"
        colour: "#c9a45c"
        action: ""
    }

    InputAction {
        name: "pause"
        keys: "P"
        pad: "Start"
    }
}
