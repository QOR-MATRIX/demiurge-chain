import QtQuick
import ARQADE

// A panel over the portal: dark glass with a gilt hairline, so the scene shows faintly through.
Rectangle {
    radius: 16
    gradient: Gradient {
        GradientStop { position: 0; color: Qt.rgba(Theme.surface.r, Theme.surface.g, Theme.surface.b, 0.78) }
        GradientStop { position: 1; color: Qt.rgba(Theme.base.r, Theme.base.g, Theme.base.b, 0.9) }
    }
    border.width: 1
    border.color: Theme.line

    Rectangle {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        width: parent.width * 0.6
        height: 1
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0; color: "transparent" }
            GradientStop { position: 0.5; color: Theme.gilt }
            GradientStop { position: 1; color: "transparent" }
        }
    }
}
