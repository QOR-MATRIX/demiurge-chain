// A bar on a game's screen (P3.8): health, fuel, a charge, a level's progress. Logic sets `value`, from 0 to `maximum`;
// the bar fills that much of its length, easing to it unless the person asked for less motion.

import QtQuick
import QQ

ScreenPart {
    id: bar

    readonly property string kind: "ScreenBar"
    readonly property var fields: ["name", "screen", "anchor", "insetX", "insetY", "width", "height", "value", "maximum",
                                   "colour"]
    readonly property var ranges: ({ insetX: [-2000, 2000], insetY: [-2000, 2000], width: [8, 1280], height: [2, 200],
                                     maximum: [0.001, 1000000] })
    readonly property var choices: ({ anchor: places })

    name: "Bar"
    /// Points.
    property real width: 240
    property real height: 14
    property real value: 1
    property real maximum: 1
    property color colour: "#ff6a00"

    /// How full it is, from 0 to 1.
    readonly property real fraction: maximum > 0 ? Math.max(0, Math.min(1, value / maximum)) : 0

    drawing: Component {
        Rectangle {
            id: track
            width: bar.width * bar.unit
            height: bar.height * bar.unit
            radius: height / 2
            color: Qt.rgba(Look.surface.r, Look.surface.g, Look.surface.b, 0.78)
            border.width: Math.max(1, bar.unit)
            border.color: Look.edge

            Rectangle {
                objectName: "fill"
                x: track.border.width
                y: track.border.width
                height: track.height - 2 * track.border.width
                width: Math.max(0, (track.width - 2 * track.border.width) * bar.fraction)
                radius: height / 2
                visible: width > 0.5
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop { position: 0; color: Qt.darker(bar.colour, 1.25) }
                    GradientStop { position: 1; color: Qt.lighter(bar.colour, 1.2) }
                }
                Behavior on width {
                    enabled: Look.fade > 0
                    NumberAnimation { duration: 140; easing.type: Easing.OutCubic }
                }
            }
        }
    }
}
