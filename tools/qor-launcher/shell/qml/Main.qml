pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import QOR

// The shell's first screen (DIRECTION L13.1): the core answering through the bridge, and an amount typed in Sparks or
// CGT read by the core exactly as the Windows launcher reads it. The wallet and the rest arrive in L13's next steps.
ApplicationWindow {
    id: root
    width: 412
    height: 892
    visible: true
    color: Theme.void_
    title: "QOR Launcher"

    readonly property var units: Core.call("token_info").ok ?? ({})
    property var reading: ({})
    // How far the on-screen keyboard covers the window. Since Android 15 an app draws behind the keyboard, which no
    // longer shrinks the window, so the page leaves room for it itself, whichever is told first: this, or the safe
    // area's bottom, which comes to include the keyboard. Qt reports the keyboard in the screen's physical
    // pixels on Android (seen on Android 16: 1080 wide over a 411-wide window), so it is scaled to the window's.
    readonly property real keyboard: {
        if (!Qt.inputMethod.visible)
            return 0
        const r = Qt.inputMethod.keyboardRectangle
        const scale = r.width > root.width + 1 ? Screen.devicePixelRatio : 1
        return Math.max(0, root.height - r.y / scale)
    }

    function read(text) {
        reading = text.trim().length === 0 ? ({}) : Core.call("parse_amount", { text: text })
    }

    Flickable {
        id: page
        anchors.fill: parent
        // Clear of the status bar, the camera cut-out and the gesture bar.
        anchors.topMargin: root.SafeArea.margins.top
        anchors.bottomMargin: Math.max(root.SafeArea.margins.bottom, root.keyboard)
        anchors.leftMargin: root.SafeArea.margins.left
        anchors.rightMargin: root.SafeArea.margins.right
        contentWidth: width
        contentHeight: column.implicitHeight + 48
        boundsBehavior: Flickable.StopAtBounds
        // Scrolled content ends at the safe area: nothing shows through under the status bar.
        clip: true
        // The safe area takes in the keyboard too once Android says so, a moment after the keyboard opens; and the
        // content's height follows the reading's only once the layout has run.
        onHeightChanged: Qt.callLater(root.keepInView)
        onContentHeightChanged: Qt.callLater(root.keepInView)

        ColumnLayout {
            id: column
            x: 20
            y: 28
            width: parent.width - 40
            spacing: 22

            Text {
                text: "QOR · DEMIURGE"
                font.family: Theme.text
                font.pixelSize: 12
                font.weight: Font.DemiBold
                font.letterSpacing: 2.4
                color: Theme.accentBright
            }
            Text {
                text: "QOR Launcher"
                font.family: Theme.text
                font.pixelSize: 34
                font.weight: Font.DemiBold
                color: Theme.ink
            }
            Text {
                Layout.fillWidth: true
                text: "The Android launcher's first build: the Qt shell over the Rust core that holds the keys. The wallet, QOR ID, Social, ARQADE, the Library and the Mesh arrive next."
                font.family: Theme.text
                font.pixelSize: 16
                lineHeight: 1.2
                wrapMode: Text.WordWrap
                color: Theme.inkMuted
            }

            Panel {
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 18
                    spacing: 8
                    Text {
                        text: "THE CORE ANSWERS"
                        font.family: Theme.text
                        font.pixelSize: 11
                        font.weight: Font.DemiBold
                        font.letterSpacing: 1.8
                        color: Theme.inkMuted
                    }
                    Text {
                        objectName: "coreVersion"
                        text: "Core " + Core.version
                        font.family: Theme.mono
                        font.pixelSize: 15
                        color: Theme.ink
                    }
                    Text {
                        Layout.fillWidth: true
                        objectName: "units"
                        text: root.units.symbol ? "1 " + root.units.symbol + " = 1,000 " + root.units.sub_unit + "s = 10^" + root.units.decimals + " " + root.units.atomic_unit + "s" : "The core did not answer."
                        font.family: Theme.text
                        font.pixelSize: 15
                        wrapMode: Text.WordWrap
                        color: Theme.inkBody
                    }
                }
            }

            Panel {
                id: amountPanel
                Layout.fillWidth: true
                ColumnLayout {
                    anchors.fill: parent
                    anchors.margins: 18
                    spacing: 12
                    Text {
                        text: "AN AMOUNT, READ BY THE CORE"
                        font.family: Theme.text
                        font.pixelSize: 11
                        font.weight: Font.DemiBold
                        font.letterSpacing: 1.8
                        color: Theme.inkMuted
                    }
                    TextField {
                        id: amount
                        objectName: "amount"
                        Layout.fillWidth: true
                        implicitHeight: 52
                        placeholderText: "250 Sparks or 1.25 CGT"
                        placeholderTextColor: Theme.inkMuted
                        inputMethodHints: Qt.ImhNoPredictiveText
                        font.family: Theme.text
                        font.pixelSize: 17
                        color: Theme.ink
                        leftPadding: 14
                        background: Rectangle {
                            radius: 6
                            color: Theme.base
                            border.width: amount.activeFocus ? 2 : 1
                            border.color: amount.activeFocus ? Theme.accent : Theme.edge
                        }
                        onTextChanged: root.read(text)
                        Accessible.name: "An amount in Sparks or CGT"
                    }
                    Text {
                        Layout.fillWidth: true
                        objectName: "reading"
                        visible: text.length > 0
                        text: root.reading.ok ? root.reading.ok.sparks + "\n" + root.reading.ok.cgt + " CGT\n" + root.reading.ok.embers + " embers"
                            : root.reading.error ? root.reading.error.message : ""
                        font.family: Theme.mono
                        font.pixelSize: 15
                        lineHeight: 1.3
                        wrapMode: Text.WrapAnywhere
                        color: root.reading.error ? Theme.bad : Theme.ink
                    }
                }
            }
        }
    }

    // With the keyboard up, the amount and what the core read stay in view above it, as the keyboard opens and as the
    // reading grows.
    function keepInView() {
        if (keyboard <= 0 || !amount.activeFocus)
            return
        const bottom = amountPanel.mapToItem(page.contentItem, 0, amountPanel.height).y + 16
        page.contentY = Math.max(0, Math.min(bottom - page.height, page.contentHeight - page.height))
    }
    onKeyboardChanged: Qt.callLater(keepInView)

    component Panel: Rectangle {
        implicitHeight: (children[0]?.implicitHeight ?? 0) + 36
        radius: 10
        color: Theme.surface
        border.width: 1
        border.color: Theme.edge
    }
}
