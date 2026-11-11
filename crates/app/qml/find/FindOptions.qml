pragma ComponentBehavior: Bound
import QtQuick
import RimStack.Ui

Column {
    id: options
    readonly property var all: Hub.find.options || ({})
    spacing: Theme.gapSmall

    component Caption: Item {
        id: caption
        property alias text: captionText.text
        implicitWidth: captionText.implicitWidth + Theme.gapSmall
        implicitHeight: Theme.control
        Text { id: captionText; anchors.verticalCenter: parent.verticalCenter; color: Theme.ink3; font.pixelSize: Theme.small }
    }

    Flow {
        width: options.width
        spacing: Theme.gapSmall
        Repeater {
            model: options.all.switches
            FindOption {
                required property var modelData
                text: modelData.label
                chosen: modelData.chosen
                maxWidth: options.width
                onClicked: Hub.find.flip(modelData.id)
            }
        }
        Caption { text: Theme.say("Files") }
        Repeater {
            model: options.all.files
            FindOption {
                required property var modelData
                text: modelData.label
                chosen: modelData.chosen
                maxWidth: options.width
                onClicked: Hub.find.choose("files", modelData.id)
            }
        }
    }
    Flow {
        width: options.width
        spacing: Theme.gapSmall
        Caption { text: Theme.say("Where") }
        Repeater {
            model: options.all.where
            FindOption {
                required property var modelData
                text: modelData.label
                chosen: modelData.chosen
                enabled: modelData.available
                maxWidth: options.width
                onClicked: Hub.find.choose("where", modelData.id)
            }
        }
    }
    Flow {
        width: options.width
        spacing: Theme.gapSmall
        Caption { text: Theme.say("Leave out folders named") }
        Repeater {
            model: options.all.leave
            FindOption {
                required property var modelData
                text: modelData.label
                chosen: modelData.chosen
                maxWidth: options.width
                onClicked: Hub.find.flip(modelData.id)
            }
        }
    }
}
