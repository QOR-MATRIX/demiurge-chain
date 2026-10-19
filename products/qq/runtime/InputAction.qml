// An action a player can take (DIRECTION P3.9, ADR-105): "fire", "interact", "pause". Named in the scene with its
// bindings, so the Studio and the agent edit it like any entity, and it exists while the scene plays.
//
// `keys` are keyboard keys and mouse buttons ("E", "Space", "Enter", "Escape", "Shift", "Ctrl", "Alt", "Tab",
// "Up", "F1", "Mouse Left", "Mouse Right", "Mouse Middle"); `pad` are gamepad buttons ("A", "B", "X", "Y", "LB", "RB",
// "LT", "RT", "Back", "Start", "LS", "RS", "Up", "Down", "Left", "Right"); each list separated by commas. A
// ScreenButton whose `action` names it holds it while a finger is on it. Naming "jump" or "run" rebinds those.
//
// Logic hears it with onPressed and onReleased, asks held(name), and lets a player rebind it with rebind(name).

import QtQuick
import QtQuick3D
import QQ

Node {
    id: action

    readonly property string kind: "InputAction"
    readonly property var fields: ["name", "keys", "pad"]

    property string name: "action"
    /// Keyboard keys and mouse buttons, separated by commas.
    property string keys: ""
    /// Gamepad buttons, separated by commas.
    property string pad: ""

    /// True while the scene this belongs to plays.
    readonly property bool live: parent !== null && parent.playing === true

    function split(text) {
        return text.split(",").map(s => s.trim()).filter(s => s !== "")
    }

    property string declared: ""
    function declare() {
        if (!live || declared !== "")
            return
        Input.define(name, split(keys), split(pad))
        declared = name
    }
    onLiveChanged: declare()
    Component.onCompleted: declare()
    Component.onDestruction: if (declared !== "") Input.forget(declared)
}
