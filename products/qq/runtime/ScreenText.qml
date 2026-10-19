// Text on a game's screen (P3.8): a score, a timer, a title, a message. Logic sets `text` as the game goes. Drawn in
// QQ's faces (Look): the interface face, or the display face for a title, with a soft shadow so it reads over any sky.

import QtQuick
import QtQuick.Effects
import QQ

ScreenPart {
    id: words

    readonly property string kind: "ScreenText"
    readonly property var fields: ["name", "screen", "anchor", "insetX", "insetY", "text", "size", "colour", "typeface"]
    readonly property var ranges: ({ insetX: [-2000, 2000], insetY: [-2000, 2000], size: [8, 240] })
    readonly property var choices: ({ anchor: places, typeface: ["ui", "display"] })

    name: "Text"
    property string text: "Text"
    /// Points tall.
    property real size: 28
    property color colour: "#f3ede1"
    /// ui (Barlow Semi Condensed) or display (Cinzel, for titles).
    property string typeface: "ui"

    drawing: Component {
        Text {
            text: words.text
            color: words.colour
            font.family: words.typeface === "display" ? Look.display : Look.ui
            font.pixelSize: Math.max(1, words.size * words.unit)
            font.weight: words.typeface === "display" ? Font.Normal : Font.DemiBold
            font.letterSpacing: (words.typeface === "display" ? 0.04 : 0.01) * Math.max(1, words.size * words.unit)
            renderType: Text.QtRendering
            layer.enabled: true
            layer.effect: MultiEffect {
                shadowEnabled: true
                shadowColor: "#000000"
                shadowOpacity: 0.7
                shadowBlur: 0.45
                shadowVerticalOffset: Math.max(1, words.unit * 2)
                shadowHorizontalOffset: 0
            }
        }
    }
}
