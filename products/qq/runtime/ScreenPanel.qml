// A panel on a game's screen (P3.8): the ground a menu or a message stands on. With no size, it is a veil over the whole
// screen, which a title or pause screen draws first so the world falls back behind it. Parts are drawn in the order of
// the scene's file, so a panel comes before the parts it carries.

import QtQuick
import QQ

ScreenPart {
    id: panel

    readonly property string kind: "ScreenPanel"
    readonly property var fields: ["name", "screen", "anchor", "insetX", "insetY", "width", "height", "colour", "shade"]
    readonly property var ranges: ({ insetX: [-2000, 2000], insetY: [-2000, 2000], width: [0, 1280], height: [0, 720],
                                     shade: [0, 1] })
    readonly property var choices: ({ anchor: places })

    name: "Panel"
    anchor: "centre"
    insetX: 0
    insetY: 0
    /// Points; 0 by 0 covers the whole screen.
    property real width: 0
    property real height: 0
    property color colour: "#06070a"
    /// How much of the world it hides, from 0 (clear) to 1 (none of it).
    property real shade: 0.6

    readonly property bool whole: width <= 0 || height <= 0

    drawing: Component {
        Rectangle {
            width: panel.whole && panel.layer ? panel.layer.width : panel.width * panel.unit
            height: panel.whole && panel.layer ? panel.layer.height : panel.height * panel.unit
            radius: panel.whole ? 0 : 10 * panel.unit
            color: Qt.rgba(panel.colour.r, panel.colour.g, panel.colour.b, panel.shade)
            border.width: panel.whole ? 0 : Math.max(1, panel.unit)
            border.color: Qt.rgba(Look.gilt.r, Look.gilt.g, Look.gilt.b, 0.32)
        }
    }
}
