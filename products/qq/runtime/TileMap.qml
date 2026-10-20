// A tile map (DIRECTION P3.10, ADR-105): a level drawn in Tiled, the editor 2D games are built with, saved as a JSON map
// (`.tmj`) in the project with its tileset, and placed with its top-left corner at `position`. One tile layer is drawn
// (`layer`, or the first), as one mesh, each tile its own size in units at the scene's `pixelsPerUnit`, crisp.
//
// In play, a `solid` map's tiles are what things stand on and run into: colliders over the fewest rectangles that cover
// them (tilemap.h). A map with several layers is drawn with a TileMap for each, the solid one among them. A map marked
// `bounds` is the level's frame: a 2D view following the player stops at its edges.

import QtQuick
import QtQuick3D
import QtQuick3D.Physics
import QQ

Node {
    id: tiles

    readonly property string kind: "TileMap"
    readonly property var fields: ["name", "position", "source", "layer", "solid", "bounds", "lit", "colour"]
    readonly property var fileTypes: ({ source: ["Tiled maps (*.tmj *.json)"] })

    property string name: "Tiles"
    /// The Tiled map, kept relative to the scene file.
    property url source
    /// The tile layer drawn, by its name in Tiled; empty for the first.
    property string layer: ""
    /// Whether its tiles are walls and floors in play.
    property bool solid: true
    /// Whether a 2D view following the player stops at its edges (Scene.keptInView): the level's frame.
    property bool bounds: false
    /// Whether the scene's lights fall on it; unlit, it is drawn exactly as painted.
    property bool lit: false
    /// A tint, multiplied with the tiles: white shows them as painted.
    property color colour: "#ffffff"

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// The body play made, or null.
    property Node physics: null

    /// The map as read: { error, width, height, tileWidth, tileHeight, gids, solids, … } (SceneIO.readTiledMap).
    readonly property var map: SceneIO.readTiledMap(SceneIO.resolvedUrl(tiles, tiles.source), tiles.layer)
    /// Why it could not be read, or empty.
    readonly property string error: source.toString() === "" ? "" : (map.error ?? "")
    readonly property real pixelsPerUnit: parent !== null && parent.pixelsPerUnit !== undefined ? parent.pixelsPerUnit : 16
    readonly property real tileWidth: (map.tileWidth ?? 16) / pixelsPerUnit
    readonly property real tileHeight: (map.tileHeight ?? 16) / pixelsPerUnit
    /// How many tiles are drawn, and the map's size in units, from its top-left corner at `position`.
    readonly property int count: geometry.tiles
    readonly property size extent: Qt.size((map.width ?? 0) * tileWidth, (map.height ?? 0) * tileHeight)

    onErrorChanged: if (error !== "") console.warn("QQ: " + name + ": " + error)

    Model {
        id: look
        pickable: true
        castsShadows: false
        receivesShadows: tiles.lit
        geometry: TileGeometry {
            id: geometry
            map: tiles.map
            pixelsPerUnit: tiles.pixelsPerUnit
        }
        materials: PrincipledMaterial {
            baseColor: tiles.colour
            baseColorMap: Texture {
                source: tiles.map.image ?? ""
                minFilter: Texture.Nearest
                magFilter: Texture.Nearest
                mipFilter: Texture.None
                generateMipmaps: false
                tilingModeHorizontal: Texture.ClampToEdge
                tilingModeVertical: Texture.ClampToEdge
            }
            lighting: tiles.lit ? PrincipledMaterial.FragmentLighting : PrincipledMaterial.NoLighting
            alphaMode: PrincipledMaterial.Mask
            alphaCutoff: 0.5
            cullMode: Material.NoCulling
            metalness: 0
            roughness: 1
            specularAmount: 0
        }
    }

    function makeBody() {
        if (!live || physics || !solid || !map.solids || map.solids.length === 0)
            return
        const made = wall.createObject(parent, { position: tiles.position, rotation: tiles.rotation })
        const shapes = []
        for (const r of map.solids) {
            // [x, y, w, h] in tiles from the top-left; a box a unit deep, centred on its run.
            shapes.push(box.createObject(made, {
                position: Qt.vector3d((r[0] + r[2] / 2) * tileWidth, -(r[1] + r[3] / 2) * tileHeight, 0),
                extents: Qt.vector3d(r[2] * tileWidth, r[3] * tileHeight, 1)
            }))
        }
        made.collisionShapes = shapes
        physics = made
    }
    onLiveChanged: makeBody()
    Component.onCompleted: makeBody()

    Component {
        id: box
        BoxShape {}
    }
    Component {
        id: wall
        StaticRigidBody {
            readonly property QtObject entity: tiles
            sendContactReports: true
            physicsMaterial: PhysicsMaterial { staticFriction: 0.8; dynamicFriction: 0.7; restitution: 0.05 }
        }
    }
}
