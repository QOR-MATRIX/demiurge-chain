import QQ

Group {
    name: "Post"
    position: Qt.vector3d(0, 0, 0)
    eulerRotation: Qt.vector3d(0, 0, 0)
    scale: Qt.vector3d(1, 1, 1)

    Shape {
        name: "Pole"
        position: Qt.vector3d(0, 1, 0)
        eulerRotation: Qt.vector3d(0, 0, 0)
        scale: Qt.vector3d(0.2, 2, 0.2)
        form: "cylinder"
        colour: "#c9a45c"
        metalness: 0.4
        roughness: 0.5
        emissive: "#000000"
        emissivePower: 0
        image: ""
        imageSize: 1
        normalMap: ""
        crisp: false
        body: "static"
        mass: 1
        bounce: 0.2
        friction: 0.6
    }
}
