pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import RimStack.Ui

Item {
    id: view
    readonly property var related: Hub.workshop.related
    readonly property bool opened: !!Hub.workshop.details
    readonly property bool found: Hub.workshop.ready && Hub.workshop.problem === ""

    component Panel: Rectangle {
        radius: Theme.cornerLarge
        color: Theme.panel
        border.width: 1
        border.color: Theme.line
    }

    onOpenedChanged: details.enlarged = ""

    Component.onCompleted: {
        field.text = Mods.browseText
        Mods.browseText = ""
        Hub.workshop.select("")
        Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, Hub.workshop.sort)
    }

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: Theme.gapWide
        spacing: Theme.gapLarge

        RowLayout {
            Layout.fillWidth: true
            spacing: Theme.gap
            Btn { icon: "undo-2"; text: Theme.say("Back"); tip: Theme.say("Back to Downloads"); onClicked: Mods.browsing = false }
            FilterField {
                id: field
                Layout.fillWidth: true
                Layout.preferredWidth: 0
                implicitHeight: Theme.sized(30)
                icon: "search"
                placeholder: Theme.say("Search, or w: id  t: tag  d: expansion  m: needs a mod  a: author  or a link")
                onAccepted: Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, Hub.workshop.sort)
                Tip { text: Theme.say("Words match titles, authors and package ids. w:<id> one item, t:<tag>, d:<expansion app id>, m:<id> mods that need that mod, a:<author>. A pasted Workshop or collection link works too.") }
            }
            Btn { primary: true; text: Theme.say("Search"); onClicked: Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, Hub.workshop.sort) }
        }

        Flow {
            Layout.fillWidth: true
            spacing: Theme.gapSmall
            Text { height: Theme.control; verticalAlignment: Text.AlignVCenter; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Type") }
            Repeater {
                model: ["Mod", "Translation", "Scenario", "Any"]
                Chip {
                    required property string modelData
                    pressable: true
                    onAccent: Hub.workshop.kind === modelData.toLowerCase()
                    text: Theme.say(modelData)
                    onClicked: Hub.workshop.search(field.text, Hub.workshop.version, modelData.toLowerCase(), Hub.workshop.sort)
                }
            }
            Item { width: Theme.gapLarge; height: Theme.control }
            Text { height: Theme.control; verticalAlignment: Text.AlignVCenter; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Game version") }
            Chip {
                pressable: true
                onAccent: Hub.workshop.version === ""
                text: Theme.say("Any")
                onClicked: Hub.workshop.search(field.text, "", Hub.workshop.kind, Hub.workshop.sort)
            }
            Repeater {
                model: Hub.workshop.versions || []
                Chip {
                    required property string modelData
                    pressable: true
                    onAccent: Hub.workshop.version === modelData
                    text: modelData
                    onClicked: Hub.workshop.search(field.text, modelData, Hub.workshop.kind, Hub.workshop.sort)
                }
            }
            Item { width: Theme.gapLarge; height: Theme.control }
            Text { height: Theme.control; verticalAlignment: Text.AlignVCenter; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Expansions") }
            Chip {
                pressable: true
                onAccent: !Hub.workshop.owned
                text: Theme.say("Any")
                onClicked: { Hub.workshop.owned = false; Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, Hub.workshop.sort) }
            }
            Chip {
                pressable: true
                onAccent: Hub.workshop.owned
                text: Theme.say("Only those I own")
                tip: Theme.say("Leaves out mods that need an expansion this game does not have.")
                onClicked: { Hub.workshop.owned = true; Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, Hub.workshop.sort) }
            }
        }

        Flow {
            visible: Hub.workshop.keyMode
            Layout.fillWidth: true
            spacing: Theme.gapSmall
            Text { height: Theme.control; verticalAlignment: Text.AlignVCenter; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Sort") }
            Repeater {
                model: Hub.workshop.sorts || []
                Chip {
                    required property var modelData
                    pressable: true
                    onAccent: modelData.chosen
                    text: modelData.label
                    onClicked: Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, modelData.id)
                }
            }
            Item { width: Theme.gapLarge; height: Theme.control }
            Text { height: Theme.control; verticalAlignment: Text.AlignVCenter; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Show") }
            Chip {
                pressable: true
                onAccent: !Hub.workshop.collections
                text: Theme.say("Items")
                onClicked: Hub.workshop.showCollections(false)
            }
            Chip {
                pressable: true
                onAccent: Hub.workshop.collections
                text: Theme.say("Collections")
                onClicked: Hub.workshop.showCollections(true)
            }
        }

        Rectangle {
            visible: view.related.active
            Layout.fillWidth: true
            implicitHeight: bannerRow.implicitHeight + 2 * Theme.gapSmall
            radius: Theme.corner
            color: Theme.banner
            RowLayout {
                id: bannerRow
                anchors.fill: parent
                anchors.margins: Theme.gapSmall
                spacing: Theme.gap
                Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; text: view.related.said || "" }
                Btn { text: Theme.say("Back to results"); onClicked: Hub.workshop.back() }
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: Theme.gapLarge

            ColumnLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredWidth: 0
                spacing: Theme.gap
                Text { visible: Hub.workshop.notice !== ""; Layout.fillWidth: true; Layout.preferredWidth: 0; wrapMode: Text.WordWrap; color: Theme.warn; font.pixelSize: Theme.small; text: Hub.workshop.notice }
                Row {
                    spacing: Theme.gap
                    Text { visible: view.found; color: Theme.ink2; font.pixelSize: Theme.small; text: Theme.plural(Hub.workshop.total, "{} result", "{} results") }
                    Text { visible: Hub.workshop.searching; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Searching…") }
                }
                Item {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    Scrolled {
                        anchors.fill: parent
                        visible: view.found && Hub.workshop.total > 0
                        Flow {
                            id: cards
                            width: parent.width
                            spacing: Theme.gapLarge
                            readonly property int across: Math.max(1, Math.floor((width + spacing) / (Theme.sized(200) + spacing)))
                            readonly property real each: (width - (across - 1) * spacing) / across
                            Repeater {
                                model: Hub.workshop.rows
                                ResultCard { width: cards.each }
                            }
                        }
                    }
                    Column {
                        visible: !Hub.workshop.ready
                        anchors.centerIn: parent
                        width: Math.min(parent.width, Theme.sized(420))
                        spacing: Theme.gapLarge
                        EmptyState {
                            width: parent.width
                            icon: "download"
                            title: Theme.say("The Workshop database is not downloaded")
                            hint: Theme.say("Browsing searches a list of the Workshop's mods that RimStack keeps on this computer. It comes from the community database, downloaded once and refreshed from Settings.")
                        }
                        Btn {
                            anchors.horizontalCenter: parent.horizontalCenter
                            primary: true
                            enabled: !Hub.settings.communityBusy
                            icon: "download"
                            text: Theme.say("Download the Workshop database")
                            onClicked: Hub.settings.downloadCommunity()
                        }
                        Text { visible: Hub.settings.communityBusy; width: parent.width; horizontalAlignment: Text.AlignHCenter; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Downloading…") }
                    }
                    Column {
                        visible: Hub.workshop.ready && Hub.workshop.problem !== ""
                        anchors.centerIn: parent
                        width: Math.min(parent.width, Theme.sized(420))
                        spacing: Theme.gapLarge
                        EmptyState { width: parent.width; icon: "circle-alert"; title: Theme.say("The search did not work"); hint: Hub.workshop.problem }
                        Btn { anchors.horizontalCenter: parent.horizontalCenter; text: Theme.say("Try again"); onClicked: Hub.workshop.search(field.text, Hub.workshop.version, Hub.workshop.kind, Hub.workshop.sort) }
                    }
                    EmptyState {
                        visible: view.found && !Hub.workshop.searching && Hub.workshop.total === 0
                        anchors.centerIn: parent
                        width: Math.min(parent.width, Theme.sized(420))
                        icon: "search"
                        title: Theme.say("No mods match")
                        hint: Theme.say("Try fewer words, another type or any game version.")
                    }
                }
                Btn {
                    visible: Hub.workshop.more
                    Layout.alignment: Qt.AlignHCenter
                    enabled: !Hub.workshop.searching
                    text: Theme.say("Show more")
                    onClicked: Hub.workshop.showMore()
                }
            }

            Panel {
                Layout.fillHeight: true
                Layout.preferredWidth: Math.min(Theme.sized(360), view.width * 0.4)
                BrowseDetails {
                    id: details
                    visible: view.opened
                    anchors.fill: parent
                    anchors.margins: Theme.gapCard
                }
                ColumnLayout {
                    visible: !view.opened
                    anchors.fill: parent
                    anchors.margins: Theme.gapCard
                    spacing: Theme.gap
                    RowLayout {
                        Layout.fillWidth: true
                        spacing: Theme.gap
                        Text { color: Theme.ink2; font.pixelSize: Theme.small; font.weight: Font.DemiBold; text: Theme.say("Download list") }
                        Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink3; font.pixelSize: Theme.small; text: Hub.workshop.list.sizeSaid || "" }
                    }
                    ListPane { Layout.fillWidth: true; Layout.fillHeight: true }
                }
            }
        }
    }

    Rectangle {
        visible: details.enlarged !== ""
        anchors.fill: parent
        z: 10
        color: Theme.scrim
        Image {
            anchors.fill: parent
            anchors.margins: Theme.gapWide
            asynchronous: true
            fillMode: Image.PreserveAspectFit
            sourceSize.width: Theme.sized(1600)
            source: Theme.fileUrl(details.enlarged)
        }
        TapHandler { onTapped: details.enlarged = "" }
    }
}
