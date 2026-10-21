// What the world costs, over the viewport (DIRECTION P3.15, F3): frames a second and a frame's time, the renderer's
// draw calls and the graphics API, the scene's entities, and while it plays its bodies, each drawn as its collider
// (Qt Quick 3D Physics' own debug drawing), so what collides can be seen against what is drawn.

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import QQ

Rectangle {
    id: overlay

    /// The World it measures, and the scene in it now (the play copy while playing).
    required property QtObject world
    property QtObject scene: null

    readonly property QtObject stats: world ? world.renderStats : null
    property int entities: 0
    property int bodies: 0

    implicitWidth: column.implicitWidth + 20
    implicitHeight: column.implicitHeight + 14
    radius: 6
    color: "#cc0b0d13"
    border.color: "#33ffffff"

    // Counted twice a second: an overlay that walks the scene every frame would cost what it measures.
    Timer {
        running: overlay.visible
        repeat: true
        interval: 500
        triggeredOnStart: true
        onTriggered: {
            const all = overlay.scene ? SceneIO.everyEntity(overlay.scene) : []
            overlay.entities = all.length
            overlay.bodies = all.filter(e => e.physics !== undefined && e.physics !== null).length
        }
    }

    ColumnLayout {
        id: column
        anchors.centerIn: parent
        spacing: 2
        Label {
            objectName: "statsFrames"
            text: overlay.stats ? qsTr("%1 fps · %2 ms").arg(Math.round(overlay.stats.fps))
                                                        .arg(overlay.stats.frameTime.toFixed(1)) : ""
            font.family: "Consolas"
            font.pixelSize: 12
        }
        Label {
            objectName: "statsDraws"
            text: overlay.stats ? qsTr("%1 draw calls · %2").arg(overlay.stats.drawCallCount).arg(overlay.stats.graphicsApiName)
                                : ""
            font.family: "Consolas"
            font.pixelSize: 12
            opacity: 0.8
        }
        Label {
            objectName: "statsScene"
            text: qsTr("%1 entities · %2 bodies").arg(overlay.entities).arg(overlay.bodies)
            font.family: "Consolas"
            font.pixelSize: 12
            opacity: 0.8
        }
    }
}
