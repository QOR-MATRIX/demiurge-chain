// A sprite (DIRECTION P3.10, ADR-105): an image, or one frame of a sheet of them, standing on the plane z = 0 of a 2D
// scene (or anywhere in a 3D one). Its size is its pixels over the scene's `pixelsPerUnit`, times its scale, so a sprite
// is drawn exactly as painted. A sheet is `columns` by `rows` frames, read left to right and top to bottom; it plays
// `count` frames from `first` at `fps`, or holds `frame` when `fps` is 0. `crisp` keeps every art pixel square (pixel
// art); `lit` lets the scene's lights fall on it; `flip` faces it the other way.
//
// In play it can be a body, as a Shape is: static, dynamic, kinematic or none. In a 2D scene a dynamic one moves on the
// plane and turns only about the depth axis, or not at all when `turns` is off (a character, a crate that slides).
// Logic hears its contacts and pushes it as it does a Shape's.

import QtQuick
import QtQuick3D
import QtQuick3D.Physics
import QQ

Node {
    id: sprite

    readonly property string kind: "Sprite"
    readonly property var fields: ["name", "position", "eulerRotation", "scale", "source", "columns", "rows", "frame",
                                   "first", "count", "fps", "colour", "lit", "crisp", "flip", "body", "turns", "mass",
                                   "bounce", "friction"]
    readonly property var ranges: ({ columns: [1, 256], rows: [1, 256], frame: [0, 65535], first: [0, 65535],
                                     count: [1, 65536], fps: [0, 120], mass: [0.1, 100], bounce: [0, 1],
                                     friction: [0, 1.5] })
    readonly property var choices: ({ body: ["none", "static", "dynamic", "kinematic"] })
    readonly property var fileTypes: ({ source: ["Images (*.png *.jpg *.jpeg *.webp)"] })

    property string name: "Sprite"
    /// The image or sheet, kept relative to the scene file.
    property url source
    /// The sheet's frames across and down: 1 by 1 for a single image.
    property int columns: 1
    property int rows: 1
    /// The frame shown, counted left to right, top to bottom from 0.
    property int frame: 0
    /// The frames played when `fps` is above 0: `count` of them from `first`, looping.
    property int first: 0
    property int count: 1
    property real fps: 0
    /// A tint, multiplied with the image: white shows it as painted.
    property color colour: "#ffffff"
    /// Whether the scene's lights fall on it; unlit, it is drawn exactly as painted.
    property bool lit: false
    /// Pixel art: nearest pixels, never blurred, edges cut clean.
    property bool crisp: true
    /// Faced the other way, left for right.
    property bool flip: false

    /// none, static, dynamic or kinematic: what it is in play.
    property string body: "none"
    /// A dynamic sprite in a 2D scene turns about the depth axis; off, it never turns.
    property bool turns: true
    property real mass: 1.0
    property real bounce: 0.1
    property real friction: 0.6

    /// True while the scene this belongs to plays, and whether that scene is 2D.
    readonly property bool live: parent !== null && parent.playing === true
    readonly property bool flat: parent !== null && parent.view === "2d"
    /// The body play made, or null.
    property Node physics: null

    /// The image's pixels, one frame's pixels, and one frame's size in units before the sprite's own scale.
    readonly property size pixels: SceneIO.imageSize(SceneIO.resolvedUrl(sprite, sprite.source))
    readonly property real frameWidth: pixels.width / Math.max(1, columns)
    readonly property real frameHeight: pixels.height / Math.max(1, rows)
    readonly property real pixelsPerUnit: parent !== null && parent.pixelsPerUnit !== undefined ? parent.pixelsPerUnit : 16
    readonly property vector3d size: Qt.vector3d(frameWidth / pixelsPerUnit, frameHeight / pixelsPerUnit, 1)

    /// The body met another entity's (null for something that is not an entity), at `point`, pushed along `normal`.
    signal contact(QtObject other, vector3d point, vector3d normal)

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

    function makeBody() {
        if (!live || physics || (body !== "static" && body !== "dynamic" && body !== "kinematic"))
            return
        const made = (body === "dynamic" ? movingBody : body === "kinematic" ? ledBody : standingBody).createObject(parent, {
            position: sprite.position,
            rotation: sprite.rotation
        })
        made.collisionShapes = [box.createObject(made)]
        physics = made
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    // The frames play while the world does.
    property real elapsedFrames: 0
    onFirstChanged: { frame = first; elapsedFrames = 0 }
    FrameAnimation {
        running: sprite.live && sprite.fps > 0 && sprite.count > 1 && !(sprite.parent && sprite.parent.paused)
        // Played from the first of its frames, unless it is already among them.
        onRunningChanged: if (running && (sprite.frame < sprite.first || sprite.frame >= sprite.first + sprite.count))
                              sprite.frame = sprite.first
        onTriggered: {
            sprite.elapsedFrames += Math.min(frameTime, 0.05) * sprite.fps
            while (sprite.elapsedFrames >= 1) {
                sprite.elapsedFrames -= 1
                const at = sprite.frame - sprite.first
                sprite.frame = sprite.first + (at >= 0 && at < sprite.count ? (at + 1) % sprite.count : 0)
            }
        }
    }

    readonly property int column: frame % Math.max(1, columns)
    readonly property int row: Math.floor(frame / Math.max(1, columns)) % Math.max(1, rows)

    Texture {
        id: sheet
        source: SceneIO.resolvedUrl(sprite, sprite.source)
        minFilter: sprite.crisp ? Texture.Nearest : Texture.Linear
        magFilter: sprite.crisp ? Texture.Nearest : Texture.Linear
        mipFilter: sprite.crisp ? Texture.None : Texture.Linear
        generateMipmaps: !sprite.crisp
        tilingModeHorizontal: Texture.ClampToEdge
        tilingModeVertical: Texture.ClampToEdge
        scaleU: 1 / Math.max(1, sprite.columns)
        scaleV: 1 / Math.max(1, sprite.rows)
        positionU: sprite.column / Math.max(1, sprite.columns)
        // Texture coordinates count up from the image's foot: the top row is the last band.
        positionV: (Math.max(1, sprite.rows) - 1 - sprite.row) / Math.max(1, sprite.rows)
    }

    // A dynamic sprite's look rides on its body; any other's stays here. Qt's rectangle is a hundred units across.
    Model {
        id: look
        parent: sprite.physics && sprite.body === "dynamic" ? sprite.physics : sprite
        source: "#Rectangle"
        pickable: true
        castsShadows: sprite.lit
        receivesShadows: sprite.lit
        scale: {
            const s = parent === sprite ? Qt.vector3d(1, 1, 1) : sprite.scale
            return Qt.vector3d((sprite.flip ? -1 : 1) * s.x * sprite.size.x / 100, s.y * sprite.size.y / 100, 1)
        }
        materials: PrincipledMaterial {
            baseColorMap: sheet
            baseColor: sprite.colour
            lighting: sprite.lit ? PrincipledMaterial.FragmentLighting : PrincipledMaterial.NoLighting
            alphaMode: sprite.crisp ? PrincipledMaterial.Mask : PrincipledMaterial.Blend
            alphaCutoff: 0.5
            cullMode: Material.NoCulling
            metalness: 0
            roughness: 1
            specularAmount: 0
        }
    }

    PhysicsMaterial {
        id: surface
        restitution: sprite.bounce
        staticFriction: sprite.friction
        dynamicFriction: sprite.friction * 0.8
    }

    // The collider is the frame's box, a unit deep, so it meets whatever else stands on the plane.
    Component {
        id: box
        BoxShape {
            extents: Qt.vector3d(sprite.size.x * sprite.scale.x, sprite.size.y * sprite.scale.y, 1)
        }
    }
    Component {
        id: standingBody
        StaticRigidBody {
            readonly property QtObject entity: sprite
            physicsMaterial: surface
            sendContactReports: true
            receiveContactReports: true
            sendTriggerReports: true
            onBodyContact: (other, positions, impulses, normals) => sprite.reportContact(other, positions, normals)
        }
    }
    Component {
        id: movingBody
        DynamicRigidBody {
            readonly property QtObject entity: sprite
            physicsMaterial: surface
            massMode: DynamicRigidBody.Mass
            mass: sprite.mass
            // On the plane: no drifting in depth, and turning only about the depth axis, if at all.
            linearAxisLock: sprite.flat ? DynamicRigidBody.LockZ : DynamicRigidBody.LockNone
            angularAxisLock: sprite.flat ? (sprite.turns ? DynamicRigidBody.LockX | DynamicRigidBody.LockY
                                                         : DynamicRigidBody.LockX | DynamicRigidBody.LockY
                                                           | DynamicRigidBody.LockZ)
                                         : DynamicRigidBody.LockNone
            sendContactReports: true
            receiveContactReports: true
            sendTriggerReports: true
            onBodyContact: (other, positions, impulses, normals) => sprite.reportContact(other, positions, normals)
        }
    }
    Component {
        id: ledBody
        DynamicRigidBody {
            readonly property QtObject entity: sprite
            physicsMaterial: surface
            isKinematic: true
            kinematicPosition: sprite.position
            kinematicRotation: sprite.rotation
            sendContactReports: true
            receiveContactReports: true
            sendTriggerReports: true
            onBodyContact: (other, positions, impulses, normals) => sprite.reportContact(other, positions, normals)
        }
    }

    function reportContact(other, positions, normals) {
        sprite.contact(other && other.entity !== undefined ? other.entity : null,
                       positions.length > 0 ? positions[0] : sprite.scenePosition,
                       normals.length > 0 ? normals[0] : Qt.vector3d(0, 1, 0))
    }
}
