// A model brought into a world from a glTF 2.0 file at runtime: no import step, no conversion, so a creator's own asset
// appears as soon as it is chosen. A file that cannot be read is an error the prop reports, never a crash.

import QtQuick
import QtQuick3D
import QtQuick3D.AssetUtils

Node {
    id: prop

    /// The glTF 2.0 file (.gltf or .glb), as a URL.
    property alias source: loader.source

    /// Empty, Ready or Failed, as the loader reports it.
    readonly property int status: loader.status === RuntimeLoader.Success ? Prop.Ready
                                : loader.status === RuntimeLoader.Error ? Prop.Failed
                                : Prop.Empty
    readonly property string errorString: loader.errorString
    readonly property var bounds: loader.bounds

    enum Status { Empty, Ready, Failed }

    RuntimeLoader {
        id: loader
    }
}
