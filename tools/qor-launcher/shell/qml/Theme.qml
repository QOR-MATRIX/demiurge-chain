pragma Singleton
import QtQuick

// The launcher's own palette (tools/qor-launcher/src/styles/qor.css, Architect), so the phone looks like the launcher.
QtObject {
    readonly property color void_: "#06070a"
    readonly property color base: "#0b0c10"
    readonly property color surface: "#12151c"
    readonly property color raised: "#1a1f29"
    readonly property color edge: "#333a47"
    readonly property color accent: "#ff6a00"
    readonly property color accentBright: "#ff9142"
    readonly property color ink: "#ffffff"
    readonly property color inkBody: "#d7d8da"
    readonly property color inkMuted: "#93a0ae"
    readonly property color bad: "#ff6b6b"

    // The phone's own faces: Roboto on Android, Segoe UI on Windows.
    readonly property string text: Qt.platform.os === "android" ? "Roboto" : "Segoe UI"
    readonly property string mono: Qt.platform.os === "android" ? "monospace" : "Cascadia Mono"
}
