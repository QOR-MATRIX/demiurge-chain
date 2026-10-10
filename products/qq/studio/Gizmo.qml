// The handles drawn on the selected entity: an arrow for each axis (red X, green Y, blue Z) to move it, a knob that
// turns it about the vertical, and a cube at the centre that scales it evenly. Drawn in the Studio's overlay, so they
// are always on top and never part of the scene; they keep one size on screen however far the eye is.
//
// Each handle's model carries `axis`: 0, 1, 2 to move along X, Y, Z, 3 to turn about Y, and 4 to scale. The Studio
// reads it from what a press lands on.

import QtQuick
import QtQuick3D

Node {
    id: gizmo

    property QtObject target: null
    property vector3d eye: Qt.vector3d(0, 0, 10)

    // Handles for what a file keeps: an entity with no place (a sun, the ground, a behaviour) has none, and one that
    // keeps no turn or scale has no knob or cube.
    function keeps(field) { return target !== null && target.fields !== undefined && target.fields.indexOf(field) >= 0 }
    visible: target !== null && target.kind !== undefined && target.kind !== "Scene" && keeps("position")
    position: target && visible ? target.scenePosition : Qt.vector3d(0, 0, 0)
    readonly property real size: Math.max(0.15, eye.minus(position).length() * 0.12)
    scale: Qt.vector3d(size, size, size)

    component Arrow: Node {
        id: arrow
        property int axis
        property color colour

        Model {
            readonly property int axis: arrow.axis
            source: "#Cylinder"
            pickable: true
            position: Qt.vector3d(0, 0.5, 0)
            scale: Qt.vector3d(0.0007, 0.01, 0.0007)
            materials: DefaultMaterial { lighting: DefaultMaterial.NoLighting; diffuseColor: arrow.colour }
        }
        Model {
            readonly property int axis: arrow.axis
            source: "#Cone"
            pickable: true
            position: Qt.vector3d(0, 1.0, 0)
            scale: Qt.vector3d(0.0018, 0.0025, 0.0018)
            materials: DefaultMaterial { lighting: DefaultMaterial.NoLighting; diffuseColor: arrow.colour }
        }
    }

    Arrow { axis: 0; colour: "#ff4d5e"; eulerRotation: Qt.vector3d(0, 0, -90) }
    Arrow { axis: 1; colour: "#6cf07f" }
    Arrow { axis: 2; colour: "#4da3ff"; eulerRotation: Qt.vector3d(90, 0, 0) }

    Model {
        readonly property int axis: 3
        visible: gizmo.keeps("eulerRotation")
        source: "#Sphere"
        pickable: visible
        position: Qt.vector3d(0.55, 0.55, 0)
        scale: Qt.vector3d(0.0016, 0.0016, 0.0016)
        materials: DefaultMaterial { lighting: DefaultMaterial.NoLighting; diffuseColor: "#ffd27a" }
    }

    Model {
        readonly property int axis: 4
        visible: gizmo.keeps("scale")
        source: "#Cube"
        pickable: visible
        scale: Qt.vector3d(0.0016, 0.0016, 0.0016)
        materials: DefaultMaterial { lighting: DefaultMaterial.NoLighting; diffuseColor: "#f2f2f2" }
    }
}
