// The Agent panel (DIRECTION P3.3): describe a game, and a model designs and builds it in this scene, plays it, looks at
// it and revises. Everything it does is shown here as it happens, frames included; nothing is saved: the creator
// reviews the scene, then saves and commits it, or undoes the whole run.
//
// `designer` is the Studio's DesignLoop (agent/designloop.h). The provider's key is typed once and kept in the
// operating system's keychain; it is never shown again or written anywhere else.

import QtQuick
import QtQuick.Controls
import QtQuick.Controls.Material
import QtQuick.Layouts

ColumnLayout {
    id: panel

    required property QtObject designer

    spacing: 12

    RowLayout {
        Layout.fillWidth: true
        Label {
            text: qsTr("Agent")
            font.pixelSize: 18
            font.weight: Font.DemiBold
            Layout.fillWidth: true
        }
        BusyIndicator {
            running: panel.designer.running
            visible: running
            implicitWidth: 28
            implicitHeight: 28
        }
    }

    Label {
        objectName: "agentStatus"
        Layout.fillWidth: true
        wrapMode: Text.Wrap
        text: panel.designer.status !== "" ? panel.designer.status
                                           : qsTr("Describe a game, and the agent designs and builds it here, plays it, "
                                                  + "looks at it and revises. Nothing is saved until you save.")
        opacity: 0.75
        font.pixelSize: 13
    }

    // The key, once: kept in the keychain, never shown again.
    ColumnLayout {
        visible: !panel.designer.hasKey
        Layout.fillWidth: true
        spacing: 6
        Label {
            text: qsTr("Anthropic API key")
            font.pixelSize: 12
            opacity: 0.7
        }
        TextField {
            id: keyField
            objectName: "agentKey"
            Layout.fillWidth: true
            echoMode: TextInput.Password
            placeholderText: "sk-ant-…"
            Accessible.name: qsTr("Anthropic API key")
        }
        Button {
            text: qsTr("Keep it in the keychain")
            enabled: keyField.text.trim() !== ""
            onClicked: {
                const why = panel.designer.setKey(keyField.text)
                keyField.text = ""
                keyProblem.text = why
            }
        }
        Label {
            id: keyProblem
            visible: text !== ""
            color: "#d57889"
            wrapMode: Text.Wrap
            Layout.fillWidth: true
        }
        Label {
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            opacity: 0.55
            font.pixelSize: 12
            text: qsTr("Kept in Windows Credential Manager for your account only, and never written to a file or a log.")
        }
    }

    TextArea {
        id: brief
        objectName: "agentBrief"
        Layout.fillWidth: true
        Layout.preferredHeight: 110
        wrapMode: TextEdit.Wrap
        enabled: !panel.designer.running
        placeholderText: qsTr("What should it be? What the player does, the mood, the look, how long a round lasts…")
        Accessible.name: qsTr("Describe the game")
    }

    RowLayout {
        Layout.fillWidth: true
        Button {
            objectName: "agentStart"
            text: qsTr("Design and build")
            highlighted: true
            enabled: panel.designer.hasKey && !panel.designer.running && brief.text.trim() !== ""
            onClicked: panel.designer.start(brief.text)
        }
        Button {
            text: qsTr("Stop")
            visible: panel.designer.running
            onClicked: panel.designer.stop()
        }
        Item { Layout.fillWidth: true }
        Button {
            text: qsTr("Undo the run")
            flat: true
            visible: panel.designer.canUndo
            onClicked: panel.designer.undo()
        }
    }

    ListView {
        id: steps
        Layout.fillWidth: true
        Layout.fillHeight: true
        Layout.minimumHeight: 200
        clip: true
        spacing: 8
        model: panel.designer.steps
        onCountChanged: positionViewAtEnd()

        delegate: ColumnLayout {
            required property var modelData
            width: steps.width
            spacing: 4
            Label {
                Layout.fillWidth: true
                wrapMode: Text.Wrap
                text: modelData.text
                font.pixelSize: modelData.kind === "said" ? 13 : 12
                font.family: modelData.kind === "tool" || modelData.kind === "frame" ? "Consolas" : undefined
                color: modelData.failed ? "#d57889" : modelData.kind === "done" ? "#3ddc84" : Material.foreground
                opacity: modelData.kind === "tool" ? 0.65 : 1
            }
            Image {
                visible: modelData.image !== ""
                source: modelData.image
                Layout.fillWidth: true
                Layout.preferredHeight: visible ? width * sourceSize.height / Math.max(1, sourceSize.width) : 0
                fillMode: Image.PreserveAspectFit
                asynchronous: true
            }
        }
    }

    Button {
        text: qsTr("Forget the key")
        flat: true
        visible: panel.designer.hasKey && !panel.designer.running
        font.pixelSize: 12
        onClicked: panel.designer.forgetKey()
    }
}
