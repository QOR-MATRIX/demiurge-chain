// A solid shape with a physically based surface: a cube, sphere, cylinder, cone or plane, one unit across before its
// scale. The quickest thing to build a world from, and what a 2D QQ scene's shapes become when it is imported.
//
// In play it is a body: `static` (the default) stands still and things collide with it, `dynamic` falls, tumbles and is
// pushed about, with its mass, bounce and friction; `kinematic` goes exactly where logic puts it (its `position` and
// turn) and pushes dynamic bodies aside, a moving platform or a door; `none` is scenery that nothing touches. The body
// is made beside the shape, in the scene, at the shape's place and turn and sized by its scale, so nothing it does is
// scaled twice.
//
// For logic (P3.7): `contact` as the body meets another, and push(), applyForce(), setVelocity() and setSpin() on a
// dynamic one.

import QtQuick
import QtQuick3D
import QtQuick3D.Helpers
import QtQuick3D.Physics

Node {
    id: shape

    readonly property string kind: "Shape"
    readonly property var fields: ["name", "position", "eulerRotation", "scale", "form", "colour", "metalness",
                                   "roughness", "emissive", "emissivePower", "body", "mass", "bounce", "friction"]
    readonly property var ranges: ({ metalness: [0, 1], roughness: [0, 1], emissivePower: [0, 10], mass: [0.1, 100],
                                     bounce: [0, 1], friction: [0, 1.5] })
    readonly property var choices: ({ form: ["cube", "sphere", "cylinder", "cone", "plane"],
                                      body: ["static", "dynamic", "kinematic", "none"] })

    property string name: "Shape"
    /// cube, sphere, cylinder, cone or plane.
    property string form: "cube"
    property color colour: "#5ad1ff"
    property real metalness: 0.0
    property real roughness: 0.4
    /// Light the surface gives off itself; with emissivePower above 1 it blooms.
    property color emissive: "#000000"
    property real emissivePower: 0.0

    /// static, dynamic, kinematic or none: what the shape is in play.
    property string body: "static"
    /// Kilograms, for a dynamic shape.
    property real mass: 1.0
    /// How much of its speed a collision gives back: 0 a thud, 1 a perfect bounce.
    property real bounce: 0.2
    property real friction: 0.6

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// The body play made, or null.
    property Node physics: null

    /// The body met another entity's (null for something that is not an entity), at `point`, pushed along `normal`.
    signal contact(QtObject other, vector3d point, vector3d normal)

    /// A dynamic body's: an impulse (kilogram metres a second) at once, a force (newtons) for this step, a velocity
    /// (metres a second) or a spin (degrees a second about each axis) set outright. False for any other body.
    function push(impulse) {
        if (body !== "dynamic" || !physics)
            return false
        physics.applyCentralImpulse(impulse)
        return true
    }
    function applyForce(force) {
        if (body !== "dynamic" || !physics)
            return false
        physics.applyCentralForce(force)
        return true
    }
    function setVelocity(velocity) {
        if (body !== "dynamic" || !physics)
            return false
        physics.setLinearVelocity(velocity)
        return true
    }
    function setSpin(spin) {
        if (body !== "dynamic" || !physics)
            return false
        physics.setAngularVelocity(spin)
        return true
    }

    readonly property string mesh: ({ cube: "#Cube", sphere: "#Sphere", cylinder: "#Cylinder", cone: "#Cone",
                                       plane: "#Rectangle" })[form] ?? "#Cube"

    function makeBody() {
        if (!live || physics || (body !== "static" && body !== "dynamic" && body !== "kinematic"))
            return
        const made = (body === "dynamic" ? movingBody : body === "kinematic" ? ledBody : standingBody).createObject(parent, {
            position: shape.position,
            rotation: shape.rotation
        })
        made.collisionShapes = [collider.createObject(made)]
        physics = made
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    PhysicsMaterial {
        id: surface
        restitution: shape.bounce
        staticFriction: shape.friction
        dynamicFriction: shape.friction * 0.8
    }

    // A dynamic shape's look rides on its body; any other's stays here. Qt's built-in meshes are a hundred units
    // across, and a QQ unit is one.
    Model {
        id: look
        parent: shape.physics && shape.body === "dynamic" ? shape.physics : shape
        scale: (parent === shape ? Qt.vector3d(1, 1, 1) : shape.scale).times(0.01)
        source: shape.mesh
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

    // The collision shape for each form, in the body's own unscaled space. A plane is a thin slab, so it has a side to
    // land on; a sphere scaled unevenly, a cylinder and a cone are the hulls of meshes made to match Qt's: a hundred
    // units across, the cylinder and sphere centred, the cone standing on its base (so a cone's position is the middle
    // of its base).
    Component {
        id: box
        BoxShape {
            extents: shape.form === "plane" ? Qt.vector3d(shape.scale.x, shape.scale.y, Math.max(0.02, shape.scale.z * 0.02))
                                            : shape.scale
        }
    }
    Component {
        id: ball
        SphereShape { diameter: shape.scale.x }
    }
    Component {
        id: hull
        ConvexMeshShape {
            scale: shape.scale.times(0.01)
            // Raised to stand on its base like Qt's cone. Qt Physics scales a shape's position by the shape's own scale
            // (qphysxactorbody.cpp), so the lift is in the mesh's units: half its hundred.
            position: Qt.vector3d(0, shape.form === "cone" ? 50 : 0, 0)
            geometry: shape.form === "cone" ? coneMesh : shape.form === "cylinder" ? cylinderMesh : sphereMesh
            ConeGeometry { id: coneMesh; topRadius: 0; bottomRadius: 50; length: 100; rings: 2; segments: 24; asynchronous: false }
            CylinderGeometry { id: cylinderMesh; radius: 50; length: 100; segments: 24; asynchronous: false }
            SphereGeometry { id: sphereMesh; radius: 50; rings: 8; segments: 16; asynchronous: false }
        }
    }
    readonly property bool evenlyScaled: Math.abs(scale.x - scale.y) < 1e-4 && Math.abs(scale.y - scale.z) < 1e-4
    readonly property Component collider: form === "cube" || form === "plane" ? box
                                        : form === "sphere" && evenlyScaled ? ball
                                        : hull

    // Every body names its entity, and reports what it meets and what it enters, so contacts, triggers and rays can say
    // which entity was involved (Trigger.qml, SceneIO::ray).
    Component {
        id: standingBody
        StaticRigidBody {
            readonly property QtObject entity: shape
            physicsMaterial: surface
            sendContactReports: true
            receiveContactReports: true
            sendTriggerReports: true
            onBodyContact: (other, positions, impulses, normals) => shape.reportContact(other, positions, normals)
        }
    }
    Component {
        id: movingBody
        DynamicRigidBody {
            readonly property QtObject entity: shape
            physicsMaterial: surface
            massMode: DynamicRigidBody.Mass
            mass: shape.mass
            sendContactReports: true
            receiveContactReports: true
            sendTriggerReports: true
            onBodyContact: (other, positions, impulses, normals) => shape.reportContact(other, positions, normals)
        }
    }
    // Led by the shape: its place and turn are the body's, so logic moves it by moving the shape.
    Component {
        id: ledBody
        DynamicRigidBody {
            readonly property QtObject entity: shape
            physicsMaterial: surface
            isKinematic: true
            kinematicPosition: shape.position
            kinematicRotation: shape.rotation
            sendContactReports: true
            receiveContactReports: true
            sendTriggerReports: true
            onBodyContact: (other, positions, impulses, normals) => shape.reportContact(other, positions, normals)
        }
    }

    function reportContact(other, positions, normals) {
        shape.contact(other && other.entity !== undefined ? other.entity : null,
                      positions.length > 0 ? positions[0] : shape.scenePosition,
                      normals.length > 0 ? normals[0] : Qt.vector3d(0, 1, 0))
    }
}
