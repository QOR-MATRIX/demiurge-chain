pragma Singleton
import QtQuick

// ARQADE's look: the homepage's obsidian, gilt and ember (site/styles.css), with the launcher's neutrals, accent and inks
// laid over them when the launcher sends its theme, so ARQADE sits in the launcher as one of its screens.
QtObject {
    id: theme

    // The launcher's tokens (the same names QOR Media wears).
    property color void_: "#06070a"
    property color base: "#0b0e14"
    property color surface: "#131821"
    property color raised: "#1a1f29"
    property color edge: "#333a47"
    property color accent: "#ff6a00"
    property color accentBright: "#ff9142"
    property color ink: "#f3ede1"
    property color inkBody: "#d6d1c6"
    property color inkMuted: "#9ba2ad"
    property color inkFaint: "#6b7385"

    // ARQADE's own: the gilt of its rings and rules, the same in every launcher theme.
    readonly property color gilt: "#c9a45c"
    readonly property color giltBright: "#ead39a"
    readonly property color giltDim: "#7a6236"
    readonly property color line: Qt.rgba(0.788, 0.643, 0.361, 0.18)
    readonly property color lineStrong: Qt.rgba(0.788, 0.643, 0.361, 0.42)

    // The light every 3D view's metal reflects, one image all of them share, and an ember's halo (tools/make_light.py).
    readonly property url sky: "qrc:/qt/qml/ARQADE/assets/sky.hdr"
    readonly property url halo: "qrc:/qt/qml/ARQADE/assets/halo.png"

    // Carried in the executable (fonts.h).
    readonly property string display: "Cinzel"
    readonly property string ui: "Barlow Semi Condensed"

    // The person's choice, from the launcher. Off, everything moves; on, nothing that is only decoration does.
    property bool reducedMotion: false

    function apply(colors) {
        const set = (name, key) => { if (colors[key]) theme[name] = colors[key] }
        set("void_", "void"); set("base", "base"); set("surface", "surface"); set("raised", "raised")
        set("edge", "edge"); set("accent", "accent"); set("accentBright", "accent-bright"); set("ink", "ink")
        set("inkBody", "ink-body"); set("inkMuted", "ink-muted"); set("inkFaint", "ink-faint")
    }

    // A colour as linear HDR light for a material's emissive factor: above 1 so the bloom catches it. Linear, so an
    // ember stays orange through the tonemapping instead of washing to yellow.
    function glow(c, strength) {
        const lin = (v) => v <= 0.04045 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4)
        return Qt.vector3d(lin(c.r) * strength, lin(c.g) * strength, lin(c.b) * strength)
    }
}
