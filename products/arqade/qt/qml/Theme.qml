pragma Singleton
import QtQuick

// ARQADE's look: Demiurge's one look in the showcase register (ADR-108). Every value comes from Tokens, which
// docs/design/tokens.mjs generates from docs/design/tokens.json; nothing here is written by hand. When the launcher sends
// its theme, its grounds, accent and inks are laid over them, so ARQADE sits in the launcher as one of its screens.
QtObject {
    id: theme

    // Grounds, edges, action and inks: the launcher's theme may lay its own over these (apply()).
    property color void_: Tokens.colorVoid
    property color base: Tokens.colorBase
    property color well: Tokens.colorWell
    property color surface: Tokens.colorSurface
    property color raised: Tokens.colorRaised
    property color edge: Tokens.colorEdge
    property color edgeSoft: Tokens.colorEdgeSoft
    property color accent: Tokens.colorAccent
    property color accentBright: Tokens.colorAccentBright
    property color accentDim: Tokens.colorAccentDim
    property color inkOnAccent: Tokens.colorOnAccent
    property color ink: Tokens.colorInk
    property color inkBody: Tokens.colorInkBody
    property color inkMuted: Tokens.colorInkMuted
    property color inkFaint: Tokens.colorInkFaint

    // The gilt of ARQADE's rings, rules and numerals, and the status colours: the same in every launcher theme.
    readonly property color gilt: Tokens.colorGilt
    readonly property color giltBright: Tokens.colorGiltBright
    readonly property color giltDim: Tokens.colorGiltDim
    readonly property color ok: Tokens.colorOk
    readonly property color warn: Tokens.colorWarn
    readonly property color bad: Tokens.colorBad
    readonly property color line: Qt.rgba(gilt.r, gilt.g, gilt.b, 0.18)
    readonly property color lineStrong: Qt.rgba(gilt.r, gilt.g, gilt.b, 0.42)

    // The light every 3D view's metal reflects, one image all of them share, and an ember's halo (tools/make_light.py).
    readonly property url sky: "qrc:/qt/qml/ARQADE/assets/sky.hdr"
    readonly property url halo: "qrc:/qt/qml/ARQADE/assets/halo.png"

    // The faces, carried in the executable (fonts.h): Cinzel for names and numerals, Barlow Semi Condensed for labels and
    // controls, Barlow for reading. Each is asked for by its own family name, by which Qt finds its file on every system.
    readonly property string display: Tokens.fontDisplay
    readonly property string ui: Tokens.fontInterface
    readonly property string body: Tokens.fontBody

    // The type scale. The showcase's Cinzel titles may be larger; every other text is one of these steps.
    readonly property int sizeMicro: Tokens.sizeMicro
    readonly property int sizeCaption: Tokens.sizeCaption
    readonly property int sizeUi: Tokens.sizeUi
    readonly property int sizeBody: Tokens.sizeBody
    readonly property int sizeTitle: Tokens.sizeTitle
    readonly property int sizeHeading: Tokens.sizeHeading
    readonly property int sizeDisplay: Tokens.sizeDisplay
    readonly property int sizeFigure: Tokens.sizeFigure

    // Tracking, in ems: letterSpacing is the size times one of these.
    readonly property real trackingHeading: Tokens.trackingHeading
    readonly property real trackingLabel: Tokens.trackingLabel
    readonly property real trackingEyebrow: Tokens.trackingEyebrow

    // Corners: a control's and a panel's.
    readonly property int radiusControl: Tokens.radiusControl
    readonly property int radiusPanel: Tokens.radiusPanel

    // The person's choice, from the launcher. Off, everything moves; on, nothing that is only decoration does, and every
    // transition is immediate.
    property bool reducedMotion: false

    // Motion: the durations, 0 with reduced motion, and the one easing (easing.type: Easing.BezierSpline).
    readonly property int quick: reducedMotion ? 0 : Tokens.motionQuick
    readonly property int standard: reducedMotion ? 0 : Tokens.motionStandard
    readonly property int enter: reducedMotion ? 0 : Tokens.motionEnter
    readonly property int sheen: reducedMotion ? 0 : Tokens.motionSheen
    readonly property var ease: Tokens.easeCurve

    // The focus ring every surface draws, for the keyboard only.
    readonly property int focusWidth: Tokens.focusWidth
    readonly property int focusHalo: Tokens.focusHalo
    readonly property color focus: accentBright

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
