pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import RimStack.Ui

Rectangle {
    id: side
    readonly property var all: Hub.find.options || ({})
    readonly property var listed: Hub.find.mods || ({})
    readonly property var types: all.types || ({})
    radius: Theme.cornerLarge
    color: Theme.panel
    border.width: 1
    border.color: Theme.line

    component Heading: Text {
        width: parent.width
        topPadding: Theme.gap
        color: Theme.ink3
        font.pixelSize: Theme.caption
        font.weight: Font.DemiBold
        font.capitalization: Font.AllUppercase
        font.letterSpacing: 1
        elide: Text.ElideRight
    }
    component Checks: Repeater {
        delegate: FindPick {
            required property var modelData
            width: parent.width
            text: modelData.label
            chosen: modelData.chosen
            onClicked: Hub.find.flip(modelData.id)
        }
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.gapCard
        spacing: Theme.gapSmall

        Scrolled {
            id: options
            Layout.fillWidth: true
            Layout.preferredHeight: Math.min(options.contentHeight, side.height * 0.6)
            Heading { text: Theme.say("Where") }
            Repeater {
                model: side.all.where
                FindPick {
                    required property var modelData
                    width: parent.width
                    radio: true
                    text: modelData.label
                    chosen: modelData.chosen
                    enabled: modelData.available
                    onClicked: Hub.find.choose(modelData.id)
                }
            }
            Heading { text: Theme.say("Match") }
            Checks { model: side.all.switches }
            Text { width: parent.width; topPadding: Theme.gapSmall; bottomPadding: Theme.gapTiny; color: Theme.ink2; font.pixelSize: Theme.small; text: Theme.say("File types") }
            FilterField {
                id: typesBox
                width: parent.width
                placeholder: Theme.say("xml, cs; json  or  .")
                onTextChanged: Hub.find.fileTypes = text
            }
            Binding { target: typesBox; property: "text"; value: Hub.find.fileTypes }
            Text {
                visible: !!side.types.problem
                width: parent.width
                topPadding: Theme.gapTiny
                wrapMode: Text.WordWrap
                color: Theme.err
                font.pixelSize: Theme.small
                text: side.types.problem || ""
            }
            Text {
                visible: !side.types.problem
                width: parent.width
                topPadding: Theme.gapTiny
                wrapMode: Text.WordWrap
                color: Theme.ink3
                font.pixelSize: Theme.caption
                text: Theme.say("Separate with commas, semicolons or spaces. A dot is every text file.")
            }
            Heading { text: Theme.say("Leave out folders named") }
            Checks { model: side.all.leave }
        }
        Text {
            Layout.fillWidth: true
            Layout.topMargin: Theme.gapSmall
            elide: Text.ElideRight
            color: Theme.ink3
            font.pixelSize: Theme.caption
            font.weight: Font.DemiBold
            font.capitalization: Font.AllUppercase
            font.letterSpacing: 1
            text: Theme.say("Mods with matches")
        }
        FindPick {
            Layout.fillWidth: true
            Layout.rightMargin: Theme.sized(12)
            radio: true
            text: side.listed.allLabel
            count: side.listed.allCounted
            chosen: side.listed.allChosen
            onClicked: Hub.find.pickMod(-1)
        }
        ListView {
            id: list
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: side.listed.rows
            Wheel {}
            ScrollBar.vertical: Scroller {}
            delegate: FindPick {
                required property var modelData
                width: list.width - Theme.sized(12)
                radio: true
                text: modelData.name
                count: modelData.counted
                chosen: modelData.chosen
                onClicked: Hub.find.pickMod(modelData.index)
            }
        }
    }
}
