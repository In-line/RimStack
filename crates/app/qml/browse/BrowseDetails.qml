pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import RimStack.Ui

ColumnLayout {
    id: pane
    readonly property var d: Hub.workshop.details || ({})
    readonly property var needs: d.requires || []
    readonly property var expansions: d.dlc || []
    readonly property var shots: d.screenshots || []
    property string enlarged
    spacing: Theme.gap

    RowLayout {
        Layout.fillWidth: true
        spacing: Theme.gap
        Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink2; font.pixelSize: Theme.small; font.weight: Font.DemiBold; text: Theme.say("Details") }
        Btn {
            icon: "list"
            text: Theme.fill("Download list ({})", Hub.workshop.list.count)
            tip: Theme.say("Close the details and show the download list.")
            onClicked: Hub.workshop.select("")
        }
    }
    Scrolled {
        Layout.fillWidth: true
        Layout.fillHeight: true
        spacing: Theme.gapLarge
        Loader {
            width: parent.width
            active: !!pane.d.previewPath
            sourceComponent: Rectangle {
                height: Theme.sized(130)
                radius: Theme.corner
                color: Theme.ctl
                clip: true
                Image {
                    anchors.fill: parent
                    asynchronous: true
                    fillMode: Image.PreserveAspectFit
                    sourceSize.width: Theme.sized(640)
                    source: Theme.fileUrl(pane.d.previewPath)
                }
            }
        }
        Column {
            width: parent.width
            spacing: Theme.gapThin
            Text { width: parent.width; wrapMode: Text.WordWrap; color: Theme.ink; font.pixelSize: Theme.heading; font.weight: Font.DemiBold; text: pane.d.title || "" }
            Text { visible: !!pane.d.author; width: parent.width; elide: Text.ElideRight; color: Theme.ink2; font.pixelSize: Theme.body; text: pane.d.author || "" }
            Flow {
                width: parent.width
                topPadding: Theme.gapThin
                spacing: Theme.gapTiny
                Chip { visible: !!pane.d.installed; tone: Theme.ok; icon: "check"; text: pane.d.badge || "" }
                Chip { visible: !!pane.d.onList && !pane.d.installed; tone: Theme.moved; icon: "list"; text: pane.d.badge || "" }
                Chip { visible: !!pane.d.banned; tone: Theme.err; icon: "circle-alert"; text: Theme.say("Banned") }
                Repeater {
                    model: pane.d.versions || []
                    Chip { required property string modelData; text: modelData }
                }
            }
        }
        Flow {
            width: parent.width
            spacing: Theme.gap
            Btn { visible: !!pane.d.canAdd; primary: pane.needs.length === 0; icon: "plus"; text: Theme.say("Add"); onClicked: Hub.workshop.add(pane.d.id, false) }
            Btn { visible: !!pane.d.canAdd && pane.needs.length > 0; primary: true; icon: "plus"; text: Theme.say("Add with what it requires"); onClicked: Hub.workshop.add(pane.d.id, true) }
            Btn { icon: "external-link"; text: Theme.say("Workshop page"); onClicked: Theme.openWorkshop(pane.d.id) }
        }
        Flow {
            width: parent.width
            spacing: Theme.gap
            Btn { text: Theme.say("Requires"); tip: Theme.say("The mods this one needs."); onClicked: Hub.workshop.showRelated(pane.d.id, "requires") }
            Btn { text: Theme.say("Needed by"); tip: Theme.say("The mods that need this one."); onClicked: Hub.workshop.showRelated(pane.d.id, "neededBy") }
            Btn { visible: !!pane.d.author; text: Theme.say("Same author"); onClicked: Hub.workshop.showRelated(pane.d.id, "sameAuthor") }
        }
        Text { visible: !!pane.d.problem; width: parent.width; wrapMode: Text.WordWrap; color: Theme.err; font.pixelSize: Theme.body; text: pane.d.problem || "" }
        Flow {
            visible: !!pane.d.loaded
            width: parent.width
            spacing: Theme.gap
            Text { visible: !!pane.d.sizeSaid; color: Theme.ink2; font.pixelSize: Theme.small; text: pane.d.sizeSaid || "" }
            Text { visible: !!pane.d.updatedSaid; color: Theme.ink2; font.pixelSize: Theme.small; text: pane.d.updatedSaid || "" }
            Text { color: Theme.ink2; font.pixelSize: Theme.small; text: Theme.plural(pane.d.subscribers || 0, "{} subscriber", "{} subscribers") }
            Text { color: Theme.ink2; font.pixelSize: Theme.small; text: Theme.plural(pane.d.favorites || 0, "{} favourite", "{} favourites") }
            Text { visible: !!pane.d.votesSaid; color: Theme.ink2; font.pixelSize: Theme.small; text: pane.d.votesSaid || "" }
        }
        Flow {
            id: gallery
            visible: pane.shots.length > 0
            width: parent.width
            spacing: Theme.gapTiny
            readonly property real each: (width - 2 * spacing) / 3
            Repeater {
                model: pane.shots
                Rectangle {
                    id: shot
                    required property string modelData
                    width: gallery.each
                    height: width * 9 / 16
                    radius: Theme.corner
                    color: Theme.ctl
                    clip: true
                    Accessible.role: Accessible.Button
                    Accessible.name: Theme.say("Screenshot")
                    Image {
                        anchors.fill: parent
                        asynchronous: true
                        fillMode: Image.PreserveAspectCrop
                        sourceSize.width: Theme.sized(240)
                        source: Theme.fileUrl(shot.modelData)
                    }
                    TapHandler { onTapped: pane.enlarged = shot.modelData }
                }
            }
        }
        Text {
            visible: !!pane.d.description
            width: parent.width
            wrapMode: Text.WordWrap
            textFormat: Text.RichText
            color: Theme.ink2
            font.pixelSize: Theme.body
            lineHeight: 1.2
            text: pane.d.description || ""
            onLinkActivated: link => Qt.openUrlExternally(Hub.workshop.followed(link))
        }
        Column {
            visible: pane.needs.length > 0
            width: parent.width
            spacing: Theme.gapSmall
            Text { color: Theme.ink2; font.pixelSize: Theme.small; font.weight: Font.DemiBold; text: Theme.say("Requires") }
            Repeater {
                model: pane.needs
                RowLayout {
                    id: need
                    required property var modelData
                    width: parent.width
                    spacing: Theme.gap
                    Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; text: need.modelData.title }
                    Chip { visible: need.modelData.installed; tone: Theme.ok; icon: "check"; text: need.modelData.badge || "" }
                    Chip { visible: need.modelData.onList && !need.modelData.installed; tone: Theme.moved; icon: "list"; text: need.modelData.badge || "" }
                    Btn { visible: need.modelData.canAdd; icon: "plus"; text: Theme.say("Add"); onClicked: Hub.workshop.add(need.modelData.id, false) }
                }
            }
        }
        Text { visible: !!pane.d.unknownSaid; width: parent.width; wrapMode: Text.WordWrap; color: Theme.warn; font.pixelSize: Theme.small; text: pane.d.unknownSaid || "" }
        Column {
            visible: pane.expansions.length > 0
            width: parent.width
            spacing: Theme.gapSmall
            Text { color: Theme.ink2; font.pixelSize: Theme.small; font.weight: Font.DemiBold; text: Theme.say("Needs the expansion") }
            Flow {
                width: parent.width
                spacing: Theme.gapTiny
                Repeater {
                    model: pane.expansions
                    Chip { required property var modelData; maxWidth: parent.width; icon: "package"; text: modelData.name || modelData.id }
                }
            }
        }
    }
}
