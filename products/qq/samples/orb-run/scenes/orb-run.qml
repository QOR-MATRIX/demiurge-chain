import QQ

Scene {
    name: "orb-run"
    skyTop: "#0b1430"
    skyHorizon: "#c7623a"
    groundHorizon: "#2a1a1a"
    groundBottom: "#07080c"
    skyLight: 0.75
    exposure: 1
    bloom: true
    gravity: 9.81

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
        colour: "#151821"
    }

    Shape {
        name: "North wall"
        position: Qt.vector3d(0, 0.75, -11)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(22, 1.5, 0.5)
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

    Shape {
        name: "South wall"
        position: Qt.vector3d(0, 0.75, 11)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(22, 1.5, 0.5)
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

    Shape {
        name: "East wall"
        position: Qt.vector3d(11, 0.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.5, 1.5, 22)
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

    Shape {
        name: "West wall"
        position: Qt.vector3d(-11, 0.75, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.5, 1.5, 22)
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
        body: "dynamic"
        mass: 8
        bounce: 0.1
        friction: 0.8
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
}
