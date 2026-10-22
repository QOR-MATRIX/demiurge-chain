pragma Singleton
import QtQuick

// QOR Media's look: Demiurge's one look in the tool register (ADR-108). Every value comes from Tokens, which
// docs/design/tokens.mjs generates from docs/design/tokens.json; nothing here is written by hand. Docked in the launcher
// it wears the launcher's theme (ADR-094): apply() lays the theme's grounds, accent and inks over these, sent with every
// theme change.
QtObject {
    id: theme

    // Grounds, edges, action and inks: the launcher's theme lays its own over these (apply()).
    property color void_: Tokens.colorVoid
    property color base: Tokens.colorBase
    property color surface: Tokens.colorSurface
    property color raised: Tokens.colorRaised
    property color well: Tokens.colorWell
    property color edge: Tokens.colorEdge
    property color accent: Tokens.colorAccent
    property color accentBright: Tokens.colorAccentBright
    property color accentDim: Tokens.colorAccentDim
    property color ink: Tokens.colorInk
    property color inkBody: Tokens.colorInkBody
    property color inkMuted: Tokens.colorInkMuted
    property color inkFaint: Tokens.colorInkFaint
    // The ink on an ember fill: dark, in every theme (white would read 2.9:1).
    readonly property color inkOnAccent: Tokens.colorOnAccent
    property var tokens: ({})

    // The faces, carried in the executable (look.h): Barlow Semi Condensed for titles and controls, Barlow for reading,
    // IBM Plex Mono for code. Each is asked for by its own family name, by which Qt finds its file on every system.
    readonly property string ui: Tokens.fontInterface
    readonly property string text: Tokens.fontBody
    readonly property string mono: Tokens.fontMono

    // The type scale: every text is one of these steps.
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

    // Corners: a control's and a panel's.
    readonly property int radiusControl: Tokens.radiusControl
    readonly property int radiusPanel: Tokens.radiusPanel

    // The person's choice, passed on by the launcher (QOR_REDUCED_MOTION) or read from the system when started by hand
    // (look.h). On, nothing that is only decoration moves, and every transition is immediate.
    property bool reducedMotion: false

    // Motion: the durations, 0 with reduced motion, and the one easing (easing.type: Easing.BezierSpline).
    readonly property int quick: reducedMotion ? 0 : Tokens.motionQuick
    readonly property int standard: reducedMotion ? 0 : Tokens.motionStandard
    readonly property int enter: reducedMotion ? 0 : Tokens.motionEnter
    readonly property var ease: Tokens.easeCurve

    // The focus ring every surface draws, for the keyboard only.
    readonly property int focusWidth: Tokens.focusWidth
    readonly property int focusHalo: Tokens.focusHalo
    readonly property color focus: accentBright

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
