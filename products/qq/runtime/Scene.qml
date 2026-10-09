// A QQ scene: the root of everything a creator places, and the look of the world it is shown in. It is what a scene
// file holds (`scenes/<name>.qml`, ADR-083 decision 3): the Studio writes it with SceneIO in one canonical layout, one
// property per line, so a change is a small true diff in Projects.
//
// Every entity in a scene declares `kind` (its type's name, as written in the file) and `fields` (the properties a
// file keeps, in the order it keeps them). The writer needs nothing else to know about a type.

import QtQuick
import QtQuick3D

Node {
    id: scene

    readonly property string kind: "Scene"
    readonly property var fields: ["name", "skyTop", "skyHorizon", "groundHorizon", "groundBottom", "skyLight",
                                   "exposure", "bloom"]
    readonly property var ranges: ({ skyLight: [0, 3], exposure: [0.2, 3] })

    property string name: "untitled"
    property color skyTop: "#0b1430"
    property color skyHorizon: "#c7623a"
    property color groundHorizon: "#2a1a1a"
    property color groundBottom: "#07080c"
    property real skyLight: 0.75
    property real exposure: 1.0
    property bool bloom: true
}
