// What every screen kind shares (DIRECTION P3.8, ADR-105): a part of a game's interface, drawn over the world on the
// World's overlay rather than in it. ScreenText, ScreenBar, ScreenButton, ScreenImage and ScreenPanel are built on it.
// It is not a kind of its own.
//
// A part belongs to a named `screen` ("hud", "title", "pause", …) and is drawn while the scene shows that screen
// (Scene.shown), fading in and out unless the person asked for less motion (Look.reducedMotion). It is placed by an
// `anchor` (a corner, an edge's middle, or the centre) and an inset from it, in points of a screen 720 tall, so an
// interface is laid out once and drawn the same at every size. Insets run inward: from a right or bottom anchor, a
// positive inset moves the part left or up.

import QtQuick
import QtQuick3D
import QQ

Node {
    id: part

    property string name: "Screen part"
    /// The screen it is part of: shown and hidden with it.
    property string screen: "hud"
    /// top-left, top, top-right, left, centre, right, bottom-left, bottom or bottom-right.
    property string anchor: "top-left"
    /// Points in from the anchor, across and down (inward from a right or bottom anchor).
    property real insetX: 32
    property real insetY: 32

    readonly property var places: ["top-left", "top", "top-right", "left", "centre", "right", "bottom-left", "bottom",
                                   "bottom-right"]

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true
    /// Whether its screen is shown now.
    readonly property bool shown: parent !== null && parent.shown !== undefined && parent.shown.indexOf(screen) >= 0

    /// The layer it is drawn on: the overlay of the World its scene is shown in, once there is one. A scene's parts are
    /// made before the scene is placed in its world, so this is asked again when the scene's own parent changes.
    readonly property Item layer: {
        const scene = parent
        const placed = scene ? scene.parent : null
        const view = placed ? SceneIO.viewOf(scene) : null
        return view && view.overlay ? view.overlay : null
    }
    /// One point, in pixels of the layer.
    readonly property real unit: layer && layer.height > 0 ? layer.height / Look.designHeight : 1

    /// What a kind draws: a Component whose root is an Item. Set by each kind.
    property Component drawing: null
    /// What is on the layer: a frame that places and fades the kind's drawing (`content`), named for the part.
    property Item drawn: null
    property Item content: null

    function horizontal() {
        return anchor.endsWith("left") ? 0 : anchor.endsWith("right") ? 1 : 0.5
    }
    function vertical() {
        return anchor.startsWith("top") ? 0 : anchor.startsWith("bottom") ? 1 : 0.5
    }

    Component {
        id: frame
        Item {
            // Set up first, then faded: a part of a hidden screen starts hidden rather than fading out of view.
            property bool placed: false
            Behavior on opacity {
                enabled: placed && Look.fade > 0
                NumberAnimation { duration: Look.fade; easing.type: Easing.OutCubic }
            }
        }
    }

    function draw() {
        if (drawn || !layer || !drawing)
            return
        const holder = frame.createObject(layer)
        const made = drawing.createObject(holder)
        if (!made) {
            console.warn("QQ: " + name + " could not be drawn: " + drawing.errorString())
            holder.destroy()
            return
        }
        holder.objectName = Qt.binding(() => part.name)
        holder.width = Qt.binding(() => made.width)
        holder.height = Qt.binding(() => made.height)
        // The layer goes before its parts when a scene is put away, so each place asks whether it is still there.
        holder.x = Qt.binding(() => {
            const h = part.horizontal()
            const room = part.layer ? part.layer.width - holder.width : 0
            return h * room + (h === 1 ? -1 : 1) * part.insetX * part.unit
        })
        holder.y = Qt.binding(() => {
            const v = part.vertical()
            const room = part.layer ? part.layer.height - holder.height : 0
            return v * room + (v === 1 ? -1 : 1) * part.insetY * part.unit
        })
        holder.opacity = Qt.binding(() => part.shown ? 1 : 0)
        holder.visible = Qt.binding(() => holder.opacity > 0)
        holder.placed = true
        content = made
        drawn = holder
    }
    onLayerChanged: draw()
    onDrawingChanged: draw()
    Component.onCompleted: draw()
    Component.onDestruction: if (drawn) drawn.destroy()
}
