// A solid shape with a physically based surface: a cube, sphere, cylinder, cone or plane, one unit across before its
// scale. The quickest thing to build a world from, and what a 2D QQ scene's shapes become when it is imported.

import QtQuick
import QtQuick3D

Node {
    id: shape

    readonly property string kind: "Shape"
    readonly property var fields: ["name", "position", "eulerRotation", "scale", "form", "colour", "metalness",
                                   "roughness", "emissive", "emissivePower"]
    readonly property var ranges: ({ metalness: [0, 1], roughness: [0, 1], emissivePower: [0, 10] })
    readonly property var choices: ({ form: ["cube", "sphere", "cylinder", "cone", "plane"] })

    property string name: "Shape"
    /// cube, sphere, cylinder, cone or plane.
    property string form: "cube"
    property color colour: "#5ad1ff"
    property real metalness: 0.0
    property real roughness: 0.4
    /// Light the surface gives off itself; with emissivePower above 1 it blooms.
    property color emissive: "#000000"
    property real emissivePower: 0.0

    Model {
        // Qt's built-in meshes are a hundred units across; a QQ unit is one.
        scale: Qt.vector3d(0.01, 0.01, 0.01)
        source: ({ cube: "#Cube", sphere: "#Sphere", cylinder: "#Cylinder", cone: "#Cone", plane: "#Rectangle" })[shape.form]
                ?? "#Cube"
        pickable: true
        castsShadows: true
        receivesShadows: true
        materials: PrincipledMaterial {
            baseColor: shape.colour
            metalness: shape.metalness
            roughness: shape.roughness
            emissiveFactor: Qt.vector3d(shape.emissive.r * shape.emissivePower,
                                        shape.emissive.g * shape.emissivePower,
                                        shape.emissive.b * shape.emissivePower)
        }
    }
}
