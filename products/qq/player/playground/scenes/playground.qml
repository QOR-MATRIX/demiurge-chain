import QQ

Scene {
    name: "playground"
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
        elevation: 32
        azimuth: -40
        color: "#fff1dc"
        brightness: 1.15
    }

    Ground {
        name: "Ground"
        extent: 60
        colour: "#151821"
    }

    Shape {
        name: "Plinth"
        position: Qt.vector3d(0, 0.25, -4)
        eulerRotation: Qt.vector3d(0, 15, 0)
        scale: Qt.vector3d(2.4, 0.5, 2.4)
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

    Prop {
        name: "Orb"
        position: Qt.vector3d(0, 1.9, -4)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1, 1, 1)
        source: "../assets/orb.gltf"
    }

    Shape {
        name: "Step 1"
        position: Qt.vector3d(4, 0.2, -1)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1.6, 0.4, 1.6)
        form: "cube"
        colour: "#3a2f45"
        metalness: 0
        roughness: 0.5
        emissive: "#000000"
        emissivePower: 0
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "Step 2"
        position: Qt.vector3d(5.6, 0.5, -2.4)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1.6, 1, 1.6)
        form: "cube"
        colour: "#3a2f45"
        metalness: 0
        roughness: 0.5
        emissive: "#000000"
        emissivePower: 0
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "Step 3"
        position: Qt.vector3d(7.2, 0.8, -3.8)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1.6, 1.6, 1.6)
        form: "cube"
        colour: "#3a2f45"
        metalness: 0
        roughness: 0.5
        emissive: "#000000"
        emissivePower: 0
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Shape {
        name: "Beacon"
        position: Qt.vector3d(7.2, 2.6, -3.8)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.35, 2, 0.35)
        form: "cylinder"
        colour: "#0d1a24"
        metalness: 0.4
        roughness: 0.3
        emissive: "#5ad1ff"
        emissivePower: 3
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }

    Emitter {
        name: "Beacon sparks"
        position: Qt.vector3d(7.2, 3.6, -3.8)
        colour: "#5ad1ff"
        rate: 50
        life: 2.4
        size: 0.08
        speed: 1.1
        spread: 0.5
    }

    Shape {
        name: "Crate 1"
        position: Qt.vector3d(-4, 0.5, -2)
        eulerRotation: Qt.vector3d(0, 10, 0)
        scale: Qt.vector3d(1, 1, 1)
        form: "cube"
        colour: "#ff6a00"
        metalness: 0
        roughness: 0.45
        emissive: "#000000"
        emissivePower: 0
        body: "dynamic"
        mass: 8
        bounce: 0.1
        friction: 0.7
    }

    Shape {
        name: "Crate 2"
        position: Qt.vector3d(-2.8, 0.5, -2.2)
        eulerRotation: Qt.vector3d(0, -8, 0)
        scale: Qt.vector3d(1, 1, 1)
        form: "cube"
        colour: "#ff8a2a"
        metalness: 0
        roughness: 0.45
        emissive: "#000000"
        emissivePower: 0
        body: "dynamic"
        mass: 8
        bounce: 0.1
        friction: 0.7
    }

    Shape {
        name: "Crate 3"
        position: Qt.vector3d(-3.4, 2.2, -2.1)
        eulerRotation: Qt.vector3d(12, 30, 0)
        scale: Qt.vector3d(1, 1, 1)
        form: "cube"
        colour: "#ffb15c"
        metalness: 0
        roughness: 0.45
        emissive: "#000000"
        emissivePower: 0
        body: "dynamic"
        mass: 8
        bounce: 0.1
        friction: 0.7
    }

    Shape {
        name: "Ball"
        position: Qt.vector3d(-3.4, 5, -2)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.7, 0.7, 0.7)
        form: "sphere"
        colour: "#d8dee9"
        metalness: 0.9
        roughness: 0.15
        emissive: "#000000"
        emissivePower: 0
        body: "dynamic"
        mass: 3
        bounce: 0.6
        friction: 0.4
    }

    Lamp {
        name: "Ember lamp"
        position: Qt.vector3d(-1, 3.2, 1.5)
        color: "#ffb15c"
        brightness: 2.2
        reach: 7
    }

    Lamp {
        name: "Beacon lamp"
        position: Qt.vector3d(7.2, 3.2, -2.8)
        color: "#5ad1ff"
        brightness: 3
        reach: 6
    }

    Player {
        name: "Player"
        position: Qt.vector3d(1.6, 0, 4.5)
        eulerRotation: Qt.vector3d(0, 0, 0)
        colour: "#ff6a00"
        speed: 4.5
        jumpHeight: 1.2
    }

    Sound {
        name: "Chime"
        position: Qt.vector3d(0, 1.9, -4)
        source: "../assets/chime.wav"
        volume: 0.6
        reach: 25
        loops: false
    }

    Behaviour {
        name: "Orb hover"
        source: "../logic/hover.qml"
        target: "Orb"
    }
}
