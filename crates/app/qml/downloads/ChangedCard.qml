pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import RimStack.Ui

Card {
    id: card
    required property Flickable flicked
    signal showMod(int modId)
    readonly property var since: Hub.downloads.since || ({ time: 0, count: 0, active: 0, rows: [], workshop: 0 })
    readonly property var changedShown: since.rows
    property bool sinceOpen: false

    visible: since.count > 0
    title: Theme.say("Changed since the game last started")
    note: Theme.fill("{}, since {}", since.count, Theme.moment(since.time))
    key: "downloads-changed"
    actions: [
        Choice {
            readonly property int active: card.since.active
            visible: active > 0 && active < card.since.count
            title: Theme.say("Changed mods shown")
            options: [Theme.say("All"), Theme.fill("Active {}", active), Theme.fill("Inactive {}", card.since.count - active)]
            current: ["", "active", "inactive"].indexOf(Hub.downloads.sinceOnly)
            onPicked: index => Hub.downloads.sinceOnly = ["", "active", "inactive"][index]
        },
        AskSteam {
            visible: card.since.workshop > 0
            idle: Theme.say("Fetch change notes")
            tip: Theme.say("Reads each mod's change notes from its Workshop page. Only the Workshop ids are sent.")
            onClicked: Hub.downloads.fetchSinceNotes()
        }
    ]
    spacing: Theme.gapLarge
    Column {
        id: changedRows
        property int at: 0
        readonly property var shown: card.sinceOpen ? card.changedShown : card.changedShown.slice(0, 5)
        function go(to) {
            at = Math.max(0, Math.min(shown.length - 1, to))
            const row = rows.itemAt(at)
            if (!row)
                return
            const top = row.mapToItem(card.flicked.contentItem, 0, 0).y
            if (top < card.flicked.contentY)
                card.flicked.contentY = Math.max(0, top - Theme.gapLarge)
            else if (top + row.height > card.flicked.contentY + card.flicked.height)
                card.flicked.contentY = top + row.height + Theme.gapLarge - card.flicked.height
        }
        width: card.width
        spacing: Theme.gapLarge
        activeFocusOnTab: shown.length > 0
        Accessible.role: Accessible.List
        Accessible.name: Theme.say("Changed since the game last started")
        Keys.onUpPressed: go(at - 1)
        Keys.onDownPressed: go(at + 1)
        Keys.onReturnPressed: if (shown[at]) card.showMod(shown[at].modId)
        Keys.onEnterPressed: if (shown[at]) card.showMod(shown[at].modId)
        Shortcut {
            sequence: "N"
            enabled: changedRows.activeFocus && !!changedRows.shown[changedRows.at] && !changedRows.shown[changedRows.at].git
            onActivated: Theme.openChangeNotes(changedRows.shown[changedRows.at].id)
        }
        Repeater {
            id: rows
            model: changedRows.shown
            Column {
                id: changed
                readonly property bool reached: changedRows.activeFocus && changedRows.at === index
                required property var modelData
                required property int index
                readonly property string day: Theme.day(modelData.now)
                readonly property bool opensDay: modelData.opensDay
                readonly property var said: modelData.said
                width: changedRows.width
                spacing: Theme.gapTiny
                Text {
                    visible: changed.opensDay
                    topPadding: changed.index === 0 ? 0 : Theme.gap
                    bottomPadding: Theme.gapThin
                    color: Theme.ink2
                    font.pixelSize: Theme.small
                    font.weight: Font.DemiBold
                    text: changed.day + "  " + Theme.plural(changed.modelData.dayCount, "{} mod", "{} mods")
                }
                RowLayout {
                    width: parent.width
                    spacing: Theme.gap
                    Icon { name: changed.modelData.git ? "git-branch" : "cloud"; color: changed.reached ? Theme.focus : Theme.ink3 }
                    Text { Layout.fillWidth: true; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; font.weight: Font.Medium; font.underline: changed.reached; text: changed.modelData.name; Accessible.role: Accessible.ListItem; Accessible.name: text }
                    Chip { visible: !changed.modelData.active; text: Theme.say("inactive") }
                    Chip { visible: changed.modelData.git; tone: Theme.moved; text: Theme.plural(changed.modelData.notes.length, "{} commit", "{} commits") }
                    Text {
                        visible: !changed.modelData.git
                        color: Theme.ink3
                        font.pixelSize: Theme.small
                        text: changed.modelData.sameDay ? Theme.clock(changed.modelData.was) + " → " + Theme.clock(changed.modelData.now)
                            : Theme.day(changed.modelData.was) + " → " + (changed.day === Theme.day(Date.now() / 1000) ? Theme.say("today") : changed.day)
                    }
                    Btn { activeFocusOnTab: false; text: Theme.say("Show"); spoken: Theme.fill("Show {}", changed.modelData.name); tip: Theme.say("Enter"); onClicked: card.showMod(changed.modelData.modId) }
                    Btn { activeFocusOnTab: false; visible: !changed.modelData.git; icon: "external-link"; spoken: Theme.fill("Change notes of {}", changed.modelData.name); tip: Theme.say("The mod's change notes on the Workshop."); onClicked: Theme.openChangeNotes(changed.modelData.id) }
                }
                Repeater {
                    model: changed.said
                    RowLayout {
                        id: entry
                        required property var modelData
                        width: changed.width
                        spacing: Theme.gap
                        Text { Layout.preferredWidth: Theme.sized(22); color: Theme.ink3; font.pixelSize: Theme.small; text: "" }
                        Text { Layout.preferredWidth: Theme.sized(74); Layout.alignment: Qt.AlignTop; color: Theme.ink3; font.pixelSize: Theme.small; text: entry.modelData.time ? Theme.day(entry.modelData.time) : "" }
                        Text {
                            Layout.fillWidth: true
                            wrapMode: Text.WordWrap
                            maximumLineCount: 12
                            elide: Text.ElideRight
                            textFormat: Text.PlainText
                            color: entry.modelData.time ? Theme.ink2 : Theme.err
                            font.pixelSize: Theme.small
                            lineHeight: 1.25
                            text: entry.modelData.text !== "" ? entry.modelData.text : Theme.say("No note written")
                        }
                    }
                }
            }
        }
    }
    Btn {
        visible: card.changedShown.length > 5
        text: card.sinceOpen ? Theme.fill("Show {}", 5) : Theme.fill("{} more", (card.changedShown.length - 5))
        onClicked: card.sinceOpen = !card.sinceOpen
    }
}
