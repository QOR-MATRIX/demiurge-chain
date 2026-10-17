pragma Singleton
import QtQuick

// The launcher's colours (ADR-094): its theme's tokens, sent with every theme change. Architect's until the launcher says.
QtObject {
    id: theme
    property color void_: "#06070a"
    property color base: "#0b0c10"
    property color surface: "#12151c"
    property color raised: "#1a1f29"
    property color well: "#0e1117"
    property color edge: "#333a47"
    property color accent: "#ff6a00"
    property color accentBright: "#ff8a33"
    property color accentDim: "#7a3a10"
    property color ink: "#ffffff"
    property color inkBody: "#d6dbe4"
    property color inkMuted: "#9aa3b2"
    property color inkFaint: "#6b7385"
    property var tokens: ({})

    readonly property string display: "Segoe UI Variable Display"
    readonly property string text: "Segoe UI Variable Text"
    readonly property string mono: "Cascadia Mono"

    function apply(colors) {
        const set = (name, key) => { if (colors[key]) theme[name] = colors[key] }
        set("void_", "void"); set("base", "base"); set("surface", "surface"); set("raised", "raised")
        set("well", "well"); set("edge", "edge"); set("accent", "accent"); set("accentBright", "accent-bright")
        set("accentDim", "accent-dim"); set("ink", "ink"); set("inkBody", "ink-body"); set("inkMuted", "ink-muted")
        set("inkFaint", "ink-faint")
        tokens = colors
    }

    function clock(seconds) {
        if (!isFinite(seconds) || seconds < 0) seconds = 0
        const s = Math.floor(seconds % 60), m = Math.floor(seconds / 60) % 60, h = Math.floor(seconds / 3600)
        const two = (n) => (n < 10 ? "0" : "") + n
        return h > 0 ? h + ":" + two(m) + ":" + two(s) : m + ":" + two(s)
    }
}
