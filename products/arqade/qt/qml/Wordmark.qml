import QtQuick
import QtQuick.Effects
import ARQADE

// ARQADE's name in Cinzel, filled with gilt that a slow light crosses, and a warm glow behind it.
Item {
    id: mark
    property string text: "ARQADE"
    property int size: 96

    implicitWidth: letters.implicitWidth
    implicitHeight: letters.implicitHeight

    Accessible.role: Accessible.Heading
    Accessible.name: text

    Text {
        id: letters
        text: mark.text
        font.family: Theme.display
        font.pixelSize: mark.size
        font.weight: Font.Bold
        font.letterSpacing: mark.size * 0.08
        color: "white"
        visible: false
        layer.enabled: true
    }

    // The fill: gilt, with a band of brighter light passing across it.
    Rectangle {
        id: fill
        anchors.fill: letters
        visible: false
        layer.enabled: true
        property real sweep: 0
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: Theme.gilt }
            GradientStop { position: Math.max(0, fill.sweep - 0.12); color: Theme.gilt }
            GradientStop { position: fill.sweep; color: "#fff4d6" }
            GradientStop { position: Math.min(1, fill.sweep + 0.12); color: Theme.giltBright }
            GradientStop { position: 1; color: Theme.giltDim }
        }
        SequentialAnimation on sweep {
            running: !Theme.reducedMotion && mark.visible
            loops: Animation.Infinite
            NumberAnimation { from: -0.2; to: 1.2; duration: 4200; easing.type: Easing.InOutSine }
            PauseAnimation { duration: 3800 }
        }
    }

    // The same letters for the glow, without a layer of their own: the effect pads its own copy, where a layer it was
    // handed would be stretched to fill the padding.
    Text {
        id: shape
        anchors.fill: letters
        text: letters.text
        font: letters.font
        color: "white"
        visible: false
    }

    // The glow: the letters, blurred and turned to ember, behind the gilt.
    MultiEffect {
        anchors.fill: shape
        source: shape
        autoPaddingEnabled: true
        blurEnabled: true
        blur: 0.7
        blurMax: 32
        colorization: 1.0
        colorizationColor: Theme.accent
        opacity: 0.6
    }

    MultiEffect {
        anchors.fill: letters
        source: fill
        maskEnabled: true
        maskSource: letters
    }
}
