import QQ

Group {
    name: "Orb"
    position: Qt.vector3d(0, 0, 0)
    eulerRotation: Qt.vector3d(0, 0, 0)
    scale: Qt.vector3d(1, 1, 1)

    Shape {
        name: "Glow"
        position: Qt.vector3d(0, 0, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.05, 0.05, 0.05)
        form: "sphere"
        colour: "#ffd27a"
        metalness: 0
        roughness: 0.2
        emissive: "#ffb340"
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

    Trigger {
        name: "Pickup"
        position: Qt.vector3d(0, 0, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(1.4, 1.4, 1.4)
        form: "sphere"
    }
}
