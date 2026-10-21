// The project's files, as the Studio shows them (DIRECTION P3.15): its scenes, prefabs and logic, and the assets a
// scene can use (models, pictures, sounds, skies, maps), each with what using it does. A click on one uses it:
//
//   a scene         opens it
//   a prefab        places one
//   a logic file    opens it in the logic editor
//   a model         adds it as a Prop
//   a picture       puts it on the selection (its image) if it takes one; in a 2D scene, adds a Sprite of it
//   a sound         adds it as a Sound
//   a sky           makes it the scene's sky
//   a map           adds it as a TileMap
//
// A filter narrows the list by name. The list is read again whenever the project changes (saving, a prefab kept).
//
// Under the list, what the game keeps from one play to the next (P3.16), and Forget, to play it as someone new would.

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts
import QQ

ColumnLayout {
    id: panel

    /// The Studio's window: what using a file asks of it.
    required property QtObject studio
    /// The project as SceneIO.project() gives it.
    property var info: ({ scenes: [], logic: [], prefabs: [], assets: [] })
    /// The game's store (SceneIO.storeOf), or null for a scene in no folder.
    property QtObject store: null

    function sizeText(bytes) {
        return bytes < 1024 ? qsTr("%1 bytes").arg(bytes) : qsTr("%1 KB").arg((bytes / 1024).toFixed(1))
    }

    spacing: 4

    /// Every file, in sections: { section, name, kind, path or name, url }.
    readonly property var files: {
        const all = []
        const project = String(studio.project)
        for (const name of info.scenes ?? [])
            all.push({ kind: "scene", name: name, url: project + "/scenes/" + name + ".qml" })
        for (const name of info.prefabs ?? [])
            all.push({ kind: "prefab", name: name, url: project + "/prefabs/" + name + ".qml" })
        for (const name of info.logic ?? [])
            all.push({ kind: "logic", name: name, url: project + "/logic/" + name + ".qml" })
        for (const a of info.assets ?? [])
            all.push({ kind: a.kind, name: String(a.path).replace(/^assets\//, ""), url: String(a.url) })
        return all
    }
    readonly property var shown: files.filter(f => filter.text === ""
                                                   || f.name.toLowerCase().indexOf(filter.text.toLowerCase()) >= 0)

    readonly property var labels: ({ scene: qsTr("Scene"), prefab: qsTr("Prefab"), logic: qsTr("Logic"),
                                     model: qsTr("Model"), picture: qsTr("Picture"), sound: qsTr("Sound"),
                                     sky: qsTr("Sky"), map: qsTr("Map") })
    // A mark per kind: the scene, its prefabs and its logic in the look's own metal and ember; the assets in colours of
    // their own, so a kind is told apart at a glance.
    readonly property var marks: ({ scene: Look.gilt, prefab: Look.giltBright, logic: Look.accentBright, model: "#8ec5ff",
                                    picture: "#9fd49a", sound: "#c79bff", sky: "#f2a35e", map: "#5ad1ff" })

    TextField {
        id: filter
        objectName: "assetFilter"
        Layout.fillWidth: true
        placeholderText: qsTr("Find in the project")
        font.pixelSize: 12
    }

    ListView {
        id: list
        objectName: "assetList"
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.minimumHeight: 120
        clip: true
        model: panel.shown
        section.property: "kind"
        section.delegate: Label {
            required property string section
            text: panel.labels[section] ?? section
            font.family: Look.ui
            font.weight: Font.DemiBold
            font.pixelSize: Look.sizeMicro
            font.letterSpacing: Look.sizeMicro * Look.trackingEyebrow
            font.capitalization: Font.AllUppercase
            color: Look.inkFaint
            topPadding: 6
        }
        delegate: ItemDelegate {
            id: row
            required property var modelData
            width: list.width
            height: 30
            ToolTip.visible: hovered
            ToolTip.delay: 600
            ToolTip.text: panel.studio.useHint(modelData)
            onClicked: panel.studio.useFile(modelData)
            contentItem: RowLayout {
                spacing: 8
                // A picture is shown as itself; anything else by a mark of its kind's colour.
                Image {
                    visible: row.modelData.kind === "picture"
                    source: visible ? row.modelData.url : ""
                    sourceSize: Qt.size(40, 40)
                    Layout.preferredWidth: 20
                    Layout.preferredHeight: 20
                    fillMode: Image.PreserveAspectCrop
                    smooth: false
                }
                Rectangle {
                    visible: row.modelData.kind !== "picture"
                    width: 9
                    height: 9
                    radius: row.modelData.kind === "logic" ? 1 : 5
                    color: panel.marks[row.modelData.kind] ?? Look.inkMuted
                    Layout.leftMargin: 5
                    Layout.rightMargin: 6
                }
                Label {
                    text: row.modelData.name
                    font.pixelSize: 12
                    elide: Text.ElideMiddle
                    Layout.fillWidth: true
                }
            }
        }
    }

    RowLayout {
        objectName: "savedData"
        visible: panel.store !== null
        Layout.fillWidth: true
        spacing: 6
        Label {
            objectName: "savedSummary"
            Layout.fillWidth: true
            font.pixelSize: 11
            opacity: 0.7
            elide: Text.ElideRight
            text: !panel.store ? ""
                  : panel.store.keys.length === 0 ? qsTr("Nothing kept between plays")
                  : qsTr("Kept between plays: %1 (%2)").arg(panel.store.keys.join(", ")).arg(panel.sizeText(panel.store.size))
        }
        StudioButton {
            objectName: "forgetSaved"
            text: qsTr("Forget")
            flat: true
            enabled: panel.store !== null && panel.store.keys.length > 0
            ToolTip.visible: hovered
            ToolTip.text: qsTr("Forget what this game keeps between plays, to play it as someone new would")
            onClicked: {
                panel.store.clear()
                panel.studio.say(qsTr("Forgot what the game keeps between plays."))
            }
        }
    }
}
