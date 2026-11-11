pragma ComponentBehavior: Bound
import QtQuick
import RimStack.Ui

Rectangle {
    id: option
    property string text
    property bool chosen: false
    property real maxWidth: Infinity
    signal clicked()

    implicitHeight: Theme.control
    implicitWidth: Math.min(maxWidth, content.implicitWidth + 2 * Theme.gap)
    radius: Theme.corner
    color: Theme.ctl
    border.width: Theme.outline
    border.color: Theme.fieldLine
    opacity: 0.4 + 0.6 * Number(enabled)
    activeFocusOnTab: true
    Accessible.role: Accessible.CheckBox
    Accessible.name: text
    Accessible.checkable: true
    Accessible.checked: chosen
    Accessible.onToggleAction: option.clicked()
    Keys.onSpacePressed: option.clicked()
    Keys.onReturnPressed: option.clicked()
    FocusRing {}
    Rectangle {
        anchors.fill: parent
        radius: option.radius
        color: Theme.accent
        opacity: 0.3 * Number(option.chosen)
        Behavior on opacity { NumberAnimation { duration: Theme.quick } }
    }
    Rectangle {
        anchors.fill: parent
        radius: option.radius
        color: Theme.hover
        opacity: 0.06 * Number(pointed.hovered && option.enabled)
    }
    Row {
        id: content
        anchors.centerIn: parent
        spacing: Theme.gapTiny
        Icon { visible: option.chosen; anchors.verticalCenter: parent.verticalCenter; name: "check"; size: Theme.sized(11); color: label.color }
        Text {
            id: label
            width: Math.min(implicitWidth, option.maxWidth - 2 * Theme.gap - Number(option.chosen) * (Theme.sized(11) + content.spacing))
            elide: Text.ElideRight
            color: Theme.ink
            font.pixelSize: Theme.small
            text: option.text
        }
    }
    HoverHandler { id: pointed; cursorShape: Qt.PointingHandCursor }
    TapHandler { enabled: option.enabled; gesturePolicy: TapHandler.ReleaseWithinBounds; onTapped: option.clicked() }
}
