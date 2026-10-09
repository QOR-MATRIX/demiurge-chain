// A model brought into a world from a glTF 2.0 file at runtime: no import step, no conversion, so a creator's own asset
// appears as soon as it is chosen. A file that cannot be read is an error the prop reports, never a crash.

import QtQuick
import QtQuick3D
import QtQuick3D.AssetUtils

Node {
    id: prop

    readonly property string kind: "Prop"
    readonly property var fields: ["name", "position", "eulerRotation", "scale", "source"]

    property string name: "Model"

    /// The glTF 2.0 file (.gltf or .glb), as a URL. A scene file keeps it relative to itself; declared here rather than
    /// aliased to the loader so QML resolves it against the scene file, and it is always a whole URL once read.
    property url source

    /// Empty, Ready or Failed, as the loader reports it.
    readonly property int status: loader.status === RuntimeLoader.Success ? Prop.Ready
                                : loader.status === RuntimeLoader.Error ? Prop.Failed
                                : Prop.Empty
    readonly property string errorString: loader.errorString
    readonly property var bounds: loader.bounds

    enum Status { Empty, Ready, Failed }

    RuntimeLoader {
        id: loader
        source: prop.source
    }

    // What a click lands on: an invisible box over the model's bounds. The meshes the loader makes cannot be marked
    // pickable from here, and a box is what selecting a whole model needs anyway.
    Model {
        visible: prop.status === Prop.Ready
        source: "#Cube"
        pickable: true
        castsShadows: false
        receivesShadows: false
        position: loader.bounds.minimum.plus(loader.bounds.maximum).times(0.5)
        scale: loader.bounds.maximum.minus(loader.bounds.minimum).times(0.01)
        materials: PrincipledMaterial {
            alphaMode: PrincipledMaterial.Blend
            opacity: 0.0
            depthDrawMode: Material.NeverDepthDraw
        }
    }
}
