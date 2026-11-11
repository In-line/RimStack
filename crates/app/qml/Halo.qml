pragma ComponentBehavior: Bound
import QtQuick
import RimStack.Ui

// Outlines rather than a shader: an effect does not draw on the offscreen platform the checks run on.
Item {
    id: halo
    property color tone: Theme.accent
    readonly property real corner: (parent as Rectangle)?.radius ?? 0
    anchors.fill: parent
    z: -1
    Repeater {
        model: Theme.haloRings * Number(halo.visible)
        Rectangle {
            required property int index
            anchors.fill: parent
            anchors.margins: -(index + 1) * Theme.haloStep
            radius: halo.corner + (index + 1) * Theme.haloStep
            color: "transparent"
            border.width: Theme.haloStep
            border.color: Qt.rgba(halo.tone.r, halo.tone.g, halo.tone.b, Theme.haloTop * Math.pow(Theme.haloFall, index))
        }
    }
}
