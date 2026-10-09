// QQ Studio's window: the world filling it, and a panel for what is in it and how it is lit.
//
// The viewport orbits around its focus with the left mouse button and zooms with the wheel. Everything the panel
// changes is a property of the world, which is what a scene file will hold (P3.1) and what the agent will set (P3.3).

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Dialogs
import QtQuick.Layouts
import QQ

ApplicationWindow {
    id: window

    required property url model

    width: 1440
    height: 900
    visible: true
    title: qsTr("QQ Studio")
    color: "#07080c"

    Material.theme: Material.Dark
    Material.accent: "#ff6a00"
    Material.primary: "#11131a"

    // The eye on a sphere around the focus: yaw and pitch in degrees, and distance.
    property real yaw: 20
    property real pitch: 14
    property real distance: 6.5

    RowLayout {
        anchors.fill: parent
        spacing: 0

        World {
            id: world
            Layout.fillWidth: true
            Layout.fillHeight: true

            target: Qt.vector3d(0, 0.9, 0)
            eye: Qt.vector3d(
                target.x + window.distance * Math.cos(window.pitch * Math.PI / 180) * Math.sin(window.yaw * Math.PI / 180),
                target.y + window.distance * Math.sin(window.pitch * Math.PI / 180),
                target.z + window.distance * Math.cos(window.pitch * Math.PI / 180) * Math.cos(window.yaw * Math.PI / 180))
            exposure: exposureSlider.value
            bloom: bloomSwitch.checked

            Sun {
                elevation: elevationSlider.value
                azimuth: azimuthSlider.value
            }
            Ground {}
            Prop {
                id: prop
                objectName: "prop"
                y: 1.25
                source: window.model
            }

            DragHandler {
                id: orbit
                target: null
                property real startYaw
                property real startPitch
                onActiveChanged: if (active) { startYaw = window.yaw; startPitch = window.pitch }
                onTranslationChanged: {
                    window.yaw = startYaw - translation.x * 0.3
                    window.pitch = Math.max(-5, Math.min(85, startPitch + translation.y * 0.3))
                }
            }
            WheelHandler {
                onWheel: (event) => window.distance = Math.max(1.5, Math.min(60, window.distance * Math.pow(0.9, event.angleDelta.y / 120)))
            }
        }

        Pane {
            Layout.preferredWidth: 320
            Layout.fillHeight: true
            Material.elevation: 6
            padding: 20

            ColumnLayout {
                anchors.fill: parent
                spacing: 14

                Label {
                    text: "QQ Studio"
                    font.pixelSize: 22
                    font.weight: Font.DemiBold
                }
                Label {
                    text: qsTr("The QOR Engine")
                    opacity: 0.6
                }

                MenuSeparator { Layout.fillWidth: true }

                Label { text: qsTr("Model"); font.capitalization: Font.AllUppercase; font.pixelSize: 11; opacity: 0.6 }
                Label {
                    Layout.fillWidth: true
                    elide: Text.ElideMiddle
                    text: prop.status === Prop.Ready ? decodeURIComponent(String(window.model).split("/").pop())
                        : prop.status === Prop.Failed ? qsTr("Could not load: %1").arg(prop.errorString)
                        : qsTr("Loading…")
                    color: prop.status === Prop.Failed ? "#d57889" : Material.foreground
                    wrapMode: prop.status === Prop.Failed ? Text.Wrap : Text.NoWrap
                }
                Button {
                    text: qsTr("Open a glTF model…")
                    highlighted: true
                    onClicked: chooser.open()
                }

                MenuSeparator { Layout.fillWidth: true }

                Label { text: qsTr("Light"); font.capitalization: Font.AllUppercase; font.pixelSize: 11; opacity: 0.6 }
                Label { text: qsTr("Sun height %1°").arg(Math.round(elevationSlider.value)) }
                Slider { id: elevationSlider; Layout.fillWidth: true; from: 5; to: 85; value: 38 }
                Label { text: qsTr("Sun direction %1°").arg(Math.round(azimuthSlider.value)) }
                Slider { id: azimuthSlider; Layout.fillWidth: true; from: -180; to: 180; value: -35 }
                Label { text: qsTr("Exposure %1").arg(exposureSlider.value.toFixed(2)) }
                Slider { id: exposureSlider; Layout.fillWidth: true; from: 0.3; to: 2.5; value: 1.0 }
                Switch { id: bloomSwitch; text: qsTr("Bloom"); checked: true }

                Item { Layout.fillHeight: true }

                Label {
                    Layout.fillWidth: true
                    wrapMode: Text.Wrap
                    opacity: 0.5
                    font.pixelSize: 12
                    text: qsTr("Drag to orbit, scroll to zoom.")
                }
            }
        }
    }

    FileDialog {
        id: chooser
        title: qsTr("Open a glTF model")
        nameFilters: [qsTr("glTF 2.0 (*.gltf *.glb)")]
        onAccepted: window.model = selectedFile
    }
}
