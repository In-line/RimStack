pragma ComponentBehavior: Bound
import QtQuick
import RimStack.Ui

Rectangle {
    id: card
    required property var model
    readonly property bool current: (Hub.workshop.details || ({})).id === model.id
    implicitHeight: body.implicitHeight + 2 * Theme.gapMedium
    radius: Theme.cornerLarge
    color: Theme.panel
    border.width: 1
    border.color: Theme.line
    activeFocusOnTab: true
    Keys.onReturnPressed: Hub.workshop.select(card.model.id)
    Keys.onSpacePressed: Hub.workshop.select(card.model.id)
    FocusRing { corner: card.radius }
    Accessible.role: Accessible.Button
    Accessible.name: model.title
    Accessible.onPressAction: Hub.workshop.select(card.model.id)

    FontMetrics { id: titleFont; font.pixelSize: Theme.body; font.weight: Font.DemiBold }

    Rectangle { anchors.fill: parent; radius: card.radius; color: Theme.hover; opacity: Number(pointed.hovered) * 0.05 }
    Rectangle { visible: card.current; anchors.fill: parent; radius: card.radius; color: "transparent"; border.width: 2; border.color: Theme.focus }
    HoverHandler { id: pointed; cursorShape: Qt.PointingHandCursor }
    TapHandler { onTapped: Hub.workshop.select(card.model.id) }

    Column {
        id: body
        x: Theme.gapMedium
        y: Theme.gapMedium
        width: parent.width - 2 * Theme.gapMedium
        spacing: Theme.gapSmall
        Rectangle {
            width: parent.width
            height: Theme.sized(60)
            radius: Theme.corner
            color: Theme.ctl
            clip: true
            Text { anchors.centerIn: parent; color: Theme.ink3; font.pixelSize: Theme.display; font.weight: Font.DemiBold; text: card.model.title.charAt(0).toUpperCase() }
            Loader {
                anchors.fill: parent
                active: card.model.preview !== ""
                sourceComponent: Image {
                    asynchronous: true
                    fillMode: Image.PreserveAspectCrop
                    sourceSize.width: Theme.sized(400)
                    source: Theme.fileUrl(card.model.preview)
                }
            }
        }
        Text {
            width: parent.width
            height: 2 * titleFont.lineSpacing
            wrapMode: Text.Wrap
            maximumLineCount: 2
            elide: Text.ElideRight
            color: Theme.ink
            font: titleFont.font
            text: card.model.title
        }
        Text { width: parent.width; height: titleFont.lineSpacing; elide: Text.ElideRight; color: Theme.ink3; font.pixelSize: Theme.small; text: card.model.author }
        Row {
            height: Theme.tag
            spacing: Theme.gapTiny
            Repeater {
                model: card.model.versions.slice(0, 3)
                Chip { required property string modelData; text: modelData }
            }
        }
        Row {
            height: Theme.control
            spacing: Theme.gapSmall
            Btn { visible: card.model.canAdd; icon: "plus"; text: Theme.say("Add"); onClicked: Hub.workshop.add(card.model.id, false) }
            Chip { visible: card.model.installed; anchors.verticalCenter: parent.verticalCenter; tone: Theme.ok; icon: "check"; text: card.model.badge }
            Chip { visible: card.model.onList && !card.model.installed; anchors.verticalCenter: parent.verticalCenter; tone: Theme.moved; icon: "list"; text: card.model.badge }
        }
    }
}
