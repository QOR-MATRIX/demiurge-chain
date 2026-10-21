// Generated from docs/design/tokens.json by docs/design/tokens.mjs (ADR-108). Do not edit: edit the source,
// run `node docs/design/tokens.mjs`, and commit what it writes.

pragma Singleton
import QtQuick

QtObject {
    // Colours.
    readonly property color colorVoid: "#06070a"
    readonly property color colorBase: "#0b0e14"
    readonly property color colorWell: "#0e121a"
    readonly property color colorSurface: "#131821"
    readonly property color colorRaised: "#1b212d"
    readonly property color colorEdge: "#2a3140"
    readonly property color colorEdgeSoft: "#1f2530"
    readonly property color colorInk: "#f3ede1"
    readonly property color colorInkBody: "#d6d1c6"
    readonly property color colorInkMuted: "#9ba2ad"
    readonly property color colorInkFaint: "#868f9d"
    readonly property color colorAccent: "#ff6a00"
    readonly property color colorAccentBright: "#ff9142"
    readonly property color colorAccentDim: "#b44a00"
    readonly property color colorOnAccent: "#06070a"
    readonly property color colorGilt: "#c9a45c"
    readonly property color colorGiltBright: "#ead39a"
    readonly property color colorGiltDim: "#7a6236"
    readonly property color colorOk: "#22c55e"
    readonly property color colorWarn: "#e6a817"
    readonly property color colorBad: "#d57889"

    // Faces: family names, as each is registered once its file is loaded.
    readonly property string fontDisplay: "Cinzel"
    readonly property string fontInterface: "Barlow Semi Condensed"
    readonly property string fontBody: "Barlow"
    readonly property string fontMono: "IBM Plex Mono"

    // The type scale, in pixels; tracking, in ems (multiply by the size for letterSpacing).
    readonly property int sizeMicro: 10
    readonly property int sizeCaption: 11
    readonly property int sizeUi: 12
    readonly property int sizeBody: 13
    readonly property int sizeTitle: 15
    readonly property int sizeHeading: 18
    readonly property int sizeDisplay: 24
    readonly property int sizeFigure: 38
    readonly property real trackingHeading: 0.06
    readonly property real trackingLabel: 0.1
    readonly property real trackingEyebrow: 0.18

    // Corners, in pixels.
    readonly property int radiusControl: 8
    readonly property int radiusPanel: 12

    // Motion: durations in milliseconds, and the one easing as an Easing.BezierSpline curve.
    readonly property int motionQuick: 150
    readonly property int motionStandard: 260
    readonly property int motionEnter: 420
    readonly property int motionSheen: 760
    readonly property var easeCurve: [0.16, 1, 0.3, 1, 1, 1]

    // The focus ring.
    readonly property int focusWidth: 2
    readonly property int focusHalo: 5
    readonly property color focusColor: colorAccentBright
}
