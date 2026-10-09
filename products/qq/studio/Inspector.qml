// The selected entity's fields, each with the editor its type calls for. Nothing here knows an entity type: an entity
// declares `fields`, and optionally `ranges` (sliders) and `choices` (a list to pick from); SceneIO.fieldType says what
// each field holds. A change is written to the entity at once, and `edited` tells the Studio the scene changed.

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Dialogs
import QtQuick.Layouts
import QQ

ColumnLayout {
    id: inspector

    property QtObject target: null
    signal edited()

    spacing: 10

    readonly property var labels: ({
        name: qsTr("Name"), position: qsTr("Position"), eulerRotation: qsTr("Rotation"), scale: qsTr("Scale"),
        form: qsTr("Form"), colour: qsTr("Colour"), color: qsTr("Colour"), metalness: qsTr("Metal"),
        roughness: qsTr("Roughness"), emissive: qsTr("Glow colour"), emissivePower: qsTr("Glow"),
        brightness: qsTr("Brightness"), reach: qsTr("Reach"), elevation: qsTr("Height"), azimuth: qsTr("Direction"),
        extent: qsTr("Extent"), source: qsTr("Model file"), skyTop: qsTr("Sky, top"), skyHorizon: qsTr("Sky, horizon"),
        groundHorizon: qsTr("Ground, horizon"), groundBottom: qsTr("Ground, below"), skyLight: qsTr("Sky light"),
        exposure: qsTr("Exposure"), bloom: qsTr("Bloom")
    })

    function labelFor(field) { return labels[field] ?? field }
    function set(field, value) { target[field] = value; inspector.edited() }

    Repeater {
        model: inspector.target ? inspector.target.fields : []

        delegate: Loader {
            id: row
            required property string modelData
            readonly property string field: modelData
            readonly property string type: SceneIO.fieldType(inspector.target, field)
            Layout.fillWidth: true
            sourceComponent: inspector.target && inspector.target.choices && inspector.target.choices[field] ? choiceEditor
                           : type === "vector3d" ? vectorEditor
                           : type === "color" ? colourEditor
                           : type === "bool" ? boolEditor
                           : type === "url" ? urlEditor
                           : type === "real" || type === "int" ? numberEditor
                           : textEditor
        }
    }

    component FieldLabel: Label {
        font.pixelSize: 11
        font.capitalization: Font.AllUppercase
        opacity: 0.6
    }

    Component {
        id: textEditor
        ColumnLayout {
            readonly property string field: parent ? parent.field : ""
            spacing: 2
            FieldLabel { text: inspector.labelFor(parent.field) }
            TextField {
                Layout.fillWidth: true
                text: inspector.target ? inspector.target[parent.field] : ""
                onEditingFinished: if (text !== inspector.target[parent.field]) inspector.set(parent.field, text)
            }
        }
    }

    Component {
        id: numberEditor
        ColumnLayout {
            id: numberRow
            readonly property string field: parent ? parent.field : ""
            readonly property var range: inspector.target && inspector.target.ranges ? inspector.target.ranges[field] : undefined
            spacing: 2
            RowLayout {
                FieldLabel { text: inspector.labelFor(numberRow.field); Layout.fillWidth: true }
                Label {
                    text: inspector.target ? Number(inspector.target[numberRow.field]).toFixed(2) : ""
                    font.pixelSize: 12
                    opacity: 0.8
                }
            }
            Slider {
                Layout.fillWidth: true
                visible: numberRow.range !== undefined
                from: numberRow.range ? numberRow.range[0] : 0
                to: numberRow.range ? numberRow.range[1] : 1
                value: inspector.target ? inspector.target[numberRow.field] : 0
                onMoved: inspector.set(numberRow.field, value)
            }
            TextField {
                Layout.fillWidth: true
                visible: numberRow.range === undefined
                validator: DoubleValidator {}
                text: inspector.target ? String(inspector.target[numberRow.field]) : ""
                onEditingFinished: inspector.set(numberRow.field, Number(text))
            }
        }
    }

    Component {
        id: vectorEditor
        ColumnLayout {
            id: vectorRow
            readonly property string field: parent ? parent.field : ""
            readonly property vector3d value: inspector.target ? inspector.target[field] : Qt.vector3d(0, 0, 0)
            spacing: 2
            FieldLabel { text: inspector.labelFor(vectorRow.field) }
            RowLayout {
                spacing: 6
                Repeater {
                    model: ["x", "y", "z"]
                    delegate: TextField {
                        required property string modelData
                        required property int index
                        Layout.fillWidth: true
                        Layout.preferredWidth: 60
                        validator: DoubleValidator {}
                        placeholderText: modelData
                        text: Number(vectorRow.value[modelData]).toFixed(2).replace(/\.?0+$/, "")
                        leftPadding: 18
                        Label {
                            text: modelData
                            x: 4
                            anchors.verticalCenter: parent.verticalCenter
                            color: ["#ff4d5e", "#6cf07f", "#4da3ff"][index]
                            font.pixelSize: 11
                        }
                        onEditingFinished: {
                            const v = vectorRow.value
                            const n = Number(text)
                            inspector.set(vectorRow.field, Qt.vector3d(index === 0 ? n : v.x, index === 1 ? n : v.y,
                                                                       index === 2 ? n : v.z))
                        }
                    }
                }
            }
        }
    }

    Component {
        id: colourEditor
        ColumnLayout {
            id: colourRow
            readonly property string field: parent ? parent.field : ""
            spacing: 2
            FieldLabel { text: inspector.labelFor(colourRow.field) }
            RowLayout {
                Rectangle {
                    width: 30
                    height: 30
                    radius: 4
                    color: inspector.target ? inspector.target[colourRow.field] : "black"
                    border.color: Qt.rgba(1, 1, 1, 0.25)
                    MouseArea {
                        anchors.fill: parent
                        cursorShape: Qt.PointingHandCursor
                        onClicked: picker.open()
                    }
                }
                TextField {
                    Layout.fillWidth: true
                    text: inspector.target ? String(inspector.target[colourRow.field]) : ""
                    onEditingFinished: if (/^#[0-9a-fA-F]{6}$/.test(text)) inspector.set(colourRow.field, text)
                }
            }
            ColorDialog {
                id: picker
                selectedColor: inspector.target ? inspector.target[colourRow.field] : "black"
                onAccepted: inspector.set(colourRow.field, selectedColor)
            }
        }
    }

    Component {
        id: boolEditor
        Switch {
            readonly property string field: parent ? parent.field : ""
            text: inspector.labelFor(field)
            checked: inspector.target ? inspector.target[field] : false
            onToggled: inspector.set(field, checked)
        }
    }

    Component {
        id: choiceEditor
        ColumnLayout {
            id: choiceRow
            readonly property string field: parent ? parent.field : ""
            readonly property var options: inspector.target ? inspector.target.choices[field] : []
            spacing: 2
            FieldLabel { text: inspector.labelFor(choiceRow.field) }
            ComboBox {
                Layout.fillWidth: true
                model: choiceRow.options
                currentIndex: inspector.target ? choiceRow.options.indexOf(inspector.target[choiceRow.field]) : -1
                onActivated: (i) => inspector.set(choiceRow.field, choiceRow.options[i])
            }
        }
    }

    Component {
        id: urlEditor
        ColumnLayout {
            id: urlRow
            readonly property string field: parent ? parent.field : ""
            spacing: 2
            FieldLabel { text: inspector.labelFor(urlRow.field) }
            Label {
                Layout.fillWidth: true
                elide: Text.ElideMiddle
                text: inspector.target ? decodeURIComponent(String(inspector.target[urlRow.field]).split("/").pop()) : ""
            }
            Button {
                text: qsTr("Replace…")
                flat: true
                onClicked: modelChooser.open()
            }
            FileDialog {
                id: modelChooser
                title: qsTr("Choose a glTF model")
                nameFilters: [qsTr("glTF 2.0 (*.gltf *.glb)")]
                onAccepted: inspector.set(urlRow.field, selectedFile)
            }
        }
    }
}
