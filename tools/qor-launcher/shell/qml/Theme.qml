pragma Singleton
import QtQuick

// The shell's look: Demiurge's one look in the tool register (ADR-108), as on the Windows launcher, whose default is
// Obsidian. Every value comes from Tokens, which docs/design/tokens.mjs generates from docs/design/tokens.json; nothing
// here is written by hand.
QtObject {
    // Grounds, edges, action and inks.
    readonly property color void_: Tokens.colorVoid
    readonly property color base: Tokens.colorBase
    readonly property color well: Tokens.colorWell
    readonly property color surface: Tokens.colorSurface
    readonly property color raised: Tokens.colorRaised
    readonly property color edge: Tokens.colorEdge
    readonly property color accent: Tokens.colorAccent
    readonly property color accentBright: Tokens.colorAccentBright
    readonly property color ink: Tokens.colorInk
    readonly property color inkBody: Tokens.colorInkBody
    readonly property color inkMuted: Tokens.colorInkMuted
    readonly property color inkFaint: Tokens.colorInkFaint

    // The gilt of the wordmark, and the status red.
    readonly property color giltBright: Tokens.colorGiltBright
    readonly property color bad: Tokens.colorBad

    // The faces, carried in the app (fonts.h), the same on Android and Windows: Cinzel for the wordmark alone, Barlow
    // Semi Condensed for headings and labels, Barlow for reading, IBM Plex Mono for numbers.
    readonly property string display: Tokens.fontDisplay
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
    readonly property real trackingEyebrow: Tokens.trackingEyebrow

    // Corners: a control's and a panel's.
    readonly property int radiusControl: Tokens.radiusControl
    readonly property int radiusPanel: Tokens.radiusPanel

    // The focus ring's width and colour.
    readonly property int focusWidth: Tokens.focusWidth
    readonly property color focus: accentBright
}
