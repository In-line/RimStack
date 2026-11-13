pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import RimStack.Ui

Item {
    id: screen
    Appear { of: screen }

    readonly property DownloadsData downloads: Hub.downloads
    signal showMod(int modId)

    readonly property var report: downloads.updates || ({})
    readonly property var info: Hub.settings.report
    readonly property var needed: Mods.overview.toGet || []
    readonly property var replaced: downloads.replacements || []
    readonly property var failed: report.checkoutsFailed || []
    readonly property var due: report.checkoutsDue || []
    readonly property var activeUpdates: report.active || []
    readonly property var inactiveUpdates: report.inactive || []
    property bool inactiveOpen: false
    readonly property int ignoredReplacements: (info.replacementsIgnored || []).length
    readonly property int needs: needed.length + failed.length

    TextMetrics { id: listButtons; font.pixelSize: Theme.small; text: Theme.say("Download with SteamCMD…") + Theme.say("Subscribe in Steam") }
    readonly property real paneWidth: listButtons.advanceWidth + 2 * Theme.icon + 4 * Theme.gapPage
    readonly property bool side: width - paneWidth >= 2 * paneWidth

    component UpdateRow: RowLayout {
        id: row
        required property var modelData
        width: page.width
        spacing: Theme.gapLarge
        Icon { name: "cloud"; color: Theme.ink3 }
        Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; text: row.modelData.name }
        Chip { visible: row.modelData.asked; tone: Theme.ok; icon: "check"; text: Theme.say("Sent to Steam") }
        Chip { tone: Theme.moved; text: Theme.day(row.modelData.have) + " → " + Theme.day(row.modelData.now); tip: Theme.fill("Published {}", Theme.ago(row.modelData.now)) }
        Btn { text: Theme.say("Show"); spoken: Theme.fill("Show {}", row.modelData.name); onClicked: screen.showMod(row.modelData.modId) }
        Btn { icon: "external-link"; spoken: Theme.fill("What changed in {}", row.modelData.name); tip: Theme.say("The mod's change notes on the Workshop."); onClicked: Theme.openChangeNotes(row.modelData.id) }
    }

    component CheckoutRow: RowLayout {
        id: checkout
        required property var modelData
        readonly property bool updating: screen.downloads.updating === modelData.modId
        width: page.width
        spacing: Theme.gapLarge
        Icon { name: "git-branch"; color: Theme.ink3 }
        Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; text: checkout.modelData.name }
        Chip { maxWidth: page.width / 2; tone: [Theme.ink2, Theme.ok, Theme.moved, Theme.err][checkout.modelData.standing]; text: checkout.modelData.said }
        Btn {
            visible: checkout.modelData.updatable
            enabled: screen.downloads.updating < 0
            primary: checkout.modelData.due && !checkout.updating
            text: checkout.updating ? Theme.say("Updating…") : Theme.say("Update")
            spoken: Theme.fill("Update {}", checkout.modelData.name)
            tip: Theme.say("Updates to the newest commit; stops if the folder has local changes.")
            onClicked: screen.downloads.updateFromGit(checkout.modelData.modId)
        }
        Btn { text: Theme.say("Show"); spoken: Theme.fill("Show {}", checkout.modelData.name); onClicked: screen.showMod(checkout.modelData.modId) }
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.margins: Theme.gapWide
            Layout.bottomMargin: Theme.gapMedium
            spacing: Theme.gap
            FilterField {
                id: pasted
                Layout.fillWidth: true
                Layout.preferredWidth: 0
                implicitHeight: Theme.field
                icon: "link"
                placeholder: Theme.say("Paste Workshop links, collection links, mod ids or Git links")
                onAccepted: add.clicked()
            }
            Btn {
                id: add
                enabled: !screen.downloads.queueBusy && pasted.text.trim() !== ""
                primary: pasted.text.trim() !== ""
                text: screen.downloads.queueBusy ? Theme.say("Asking Steam…") : Theme.say("Look up")
                tip: Theme.say("Looks up what is pasted beside it: paste a link or an id first.")
                onClicked: screen.downloads.lookUp(pasted.text)
            }
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            Flickable {
                id: scroller
                Layout.fillWidth: true
                Layout.fillHeight: true
                contentHeight: page.implicitHeight + Theme.gapWide
                clip: true
                boundsBehavior: Flickable.StopAtBounds
                Wheel {}
                ScrollBar.vertical: Scroller {}

                Column {
                    id: page
                    x: Theme.gapWide
                    y: 0
                    width: scroller.width - 2 * Theme.gapWide
                    spacing: Theme.gapPage

                    Rectangle {
                        visible: !screen.info.autoUpdatesAsked
                        width: parent.width
                        height: offer.implicitHeight + 2 * Theme.gapMedium
                        radius: Theme.corner
                        color: Theme.banner
                        RowLayout {
                            id: offer
                            x: Theme.gapLarge
                            y: Theme.gapMedium
                            width: parent.width - 2 * Theme.gapLarge
                            spacing: Theme.gapMedium
                            Icon { Layout.alignment: Qt.AlignTop; name: "refresh-cw"; color: Theme.focus }
                            Column {
                                Layout.fillWidth: true
                                Layout.preferredWidth: 0
                                spacing: Theme.gapHair
                                Text { width: parent.width; wrapMode: Text.WordWrap; color: Theme.ink; font.pixelSize: Theme.body; font.weight: Font.DemiBold; text: Theme.say("Keep mods up to date by themselves?") }
                                Text { width: parent.width; wrapMode: Text.WordWrap; color: Theme.ink2; font.pixelSize: Theme.small; text: Theme.say("Steam is asked about the installed Workshop mods (only their Workshop ids are sent) and Git about each checkout, as often as chosen. Steam then downloads the newer versions, and each checkout with no local changes is updated, except while the game runs.") }
                            }
                            Btn { primary: true; text: Theme.say("Turn on"); onClicked: Hub.settings.keep("autoUpdates", "on") }
                            Btn { text: Theme.say("Not now"); onClicked: Hub.settings.keep("autoUpdates", "off") }
                        }
                    }

                    Column {
                        width: parent.width
                        spacing: Theme.gapSmall
                        RowLayout {
                            width: parent.width
                            spacing: Theme.gap
                            Chip { visible: !!screen.info.autoUpdates; tone: Theme.ok; icon: "refresh-cw"; text: screen.info.updateEverySaid || ""; tip: Theme.say("Settings, Updates: Update by themselves") }
                            Chip { visible: screen.downloads.steamWaits; maxWidth: page.width / 3; tone: Theme.warn; icon: "clock"; text: Theme.say("Steam is not running: the updates wait for it") }
                            Btn { visible: screen.downloads.steamWaits; text: Theme.say("Start Steam"); onClicked: Qt.openUrlExternally("steam://open/main") }
                            Chip { visible: screen.downloads.updatesBusy; text: Theme.say("Asking Steam…") }
                            Chip { visible: !screen.downloads.updatesBusy && !!screen.report.problem; tone: Theme.err; icon: "circle-alert"; text: Theme.say("Steam not reached") }
                            Chip { visible: !screen.downloads.updatesBusy && !screen.report.problem && !!screen.report.checked && screen.report.count === 0; tone: Theme.ok; icon: "check"; text: Theme.say("All Workshop mods are current") }
                            Chip { visible: !screen.downloads.updatesBusy && !screen.report.problem && screen.report.count > 0; tone: Theme.moved; icon: "download"; text: Theme.plural(screen.report.count, "{} update available", "{} updates available") }
                            Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink3; font.pixelSize: Theme.small; text: screen.report.checkedSaid || Theme.say("Never checked") }
                            Btn {
                                primary: !screen.report.checked
                                enabled: !screen.downloads.updatesBusy
                                icon: "refresh-cw"
                                text: Theme.say("Check now")
                                tip: screen.info.gitForWorkshop ? Theme.say("Asks Steam for the current version of each installed Workshop mod (only their Workshop ids are sent), and Git for new commits on each checkout in the game's Mods folder and in the Workshop folder.")
                                    : Theme.say("Asks Steam for the current version of each installed Workshop mod (only their Workshop ids are sent), and Git for new commits on each checkout in the game's Mods folder.")
                                onClicked: screen.downloads.checkUpdates()
                            }
                        }
                        Rectangle {
                            visible: screen.downloads.updatesBusy
                            width: parent.width
                            height: Theme.gapThin
                            radius: Math.min(width, height) / 2
                            color: Theme.chip
                            clip: true
                            Rectangle {
                                width: parent.width / 4
                                height: parent.height
                                radius: Math.min(width, height) / 2
                                color: Theme.accent
                                NumberAnimation on x { running: screen.downloads.updatesBusy && !Theme.stored.reduceMotion; from: -page.width / 4; to: page.width; duration: 1200; loops: Animation.Infinite }
                            }
                        }
                        Text { visible: !!screen.report.problem; width: parent.width; wrapMode: Text.WordWrap; color: Theme.err; font.pixelSize: Theme.body; text: Theme.fill("Steam not reached: {}", (screen.report.problem || "")) }
                    }

                    Card {
                        visible: screen.needs > 0
                        title: Theme.say("Needs you")
                        tone: Theme.warn
                        note: screen.needs
                        Repeater {
                            model: screen.needed
                            ProblemLine {
                                required property var modelData
                                width: page.width
                                name: modelData.name
                                words: modelData.words
                                fix: modelData.fix
                                ignore: modelData.ignore
                                packageId: modelData.packageId
                                modId: modelData.modId
                            }
                        }
                        Repeater {
                            model: screen.failed
                            CheckoutRow {}
                        }
                    }

                    Loader {
                        active: !screen.side
                        visible: active
                        width: parent.width
                        sourceComponent: AddMods {}
                    }

                    Card {
                        visible: screen.downloads.updatesBusy || (screen.report.count || 0) + screen.due.length > 0
                        title: Theme.say("Newer versions")
                        note: screen.downloads.updatesBusy ? Theme.say("Asking Steam…") : Theme.fill("{} for active mods", screen.activeUpdates.length)
                        actions: [
                            Btn {
                                visible: (screen.report.count || 0) > 0
                                primary: screen.activeUpdates.length > 0
                                enabled: !screen.downloads.taskRunning
                                icon: "download"
                                text: Theme.say("Update in Steam")
                                tip: Theme.say("Steam must be running. It downloads the newer version of each now; otherwise it does so the next time it starts.")
                                onClicked: screen.downloads.updateInSteam()
                            }
                        ]
                        Repeater {
                            model: screen.activeUpdates
                            UpdateRow {}
                        }
                        Repeater {
                            model: screen.due
                            CheckoutRow {}
                        }
                        Row {
                            visible: screen.inactiveUpdates.length > 0
                            topPadding: screen.activeUpdates.length > 0 ? Theme.gap : 0
                            spacing: Theme.gap
                            Text { color: Theme.ink2; font.pixelSize: Theme.small; font.weight: Font.DemiBold; text: Theme.say("Inactive mods") }
                            Text { color: Theme.ink3; font.pixelSize: Theme.small; text: screen.inactiveUpdates.length }
                        }
                        Repeater {
                            model: screen.inactiveOpen ? screen.inactiveUpdates : screen.inactiveUpdates.slice(0, 3)
                            UpdateRow {}
                        }
                        Btn {
                            visible: screen.inactiveUpdates.length > 3
                            text: screen.inactiveOpen ? Theme.fill("Show {}", 3) : Theme.fill("{} more", (screen.inactiveUpdates.length - 3))
                            onClicked: screen.inactiveOpen = !screen.inactiveOpen
                        }
                    }

                    ChangedCard {
                        flicked: scroller
                        Component.onCompleted: showMod.connect(screen.showMod)
                    }

                    Card {
                        visible: (screen.report.checkouts || []).length > 0
                        title: Theme.say("From Git")
                        note: screen.report.checkoutsSaid || ""
                        key: "downloads-git"
                        startsFolded: true
                        Repeater {
                            model: screen.report.checkouts || []
                            CheckoutRow {}
                        }
                    }

                    Card {
                        visible: screen.replaced.length > 0 || screen.ignoredReplacements > 0
                        title: Theme.plural(screen.replaced.length, "{} mod has a replacement", "{} mods have a replacement")
                        actions: [
                            Chip {
                                visible: screen.ignoredReplacements > 0
                                pressable: true
                                icon: "eye-off"
                                text: Theme.fill("{} ignored", screen.ignoredReplacements)
                                tip: Theme.say("Suggest them again")
                                onClicked: screen.downloads.suggestReplacementsAgain()
                            }
                        ]
                        Repeater {
                            model: screen.replaced
                            RowLayout {
                                id: old
                                required property var modelData
                                width: page.width
                                spacing: Theme.gapLarge
                                Column {
                                    Layout.fillWidth: true
                                    Layout.minimumWidth: 0
                                    spacing: Theme.gapHair
                                    RowLayout {
                                        width: parent.width
                                        spacing: Theme.gap
                                        Text { Layout.maximumWidth: parent.width - (wasOff.visible ? wasOff.width + Theme.gap : 0); elide: Text.ElideRight; color: Theme.ink2; font.pixelSize: Theme.body; text: old.modelData.name }
                                        Chip { id: wasOff; visible: !old.modelData.active; text: Theme.say("inactive") }
                                        Item { Layout.fillWidth: true }
                                    }
                                    RowLayout {
                                        width: parent.width
                                        spacing: Theme.gap
                                        Icon { name: "corner-down-right"; color: Theme.ink3 }
                                        Text { Layout.maximumWidth: parent.width - Theme.icon - Theme.gap - (hasNew.visible ? hasNew.width + Theme.gap : 0); elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; font.weight: Font.Medium; text: old.modelData.newName }
                                        Chip { id: hasNew; visible: old.modelData.haveNew; tone: Theme.ok; icon: "check"; text: Theme.say("installed") }
                                        Item { Layout.fillWidth: true }
                                    }
                                }
                                Btn { visible: !old.modelData.haveNew; icon: "external-link"; text: Theme.say("Replacement"); tip: Theme.say("Workshop page of the replacement"); onClicked: Theme.openWorkshop(old.modelData.newId) }
                                Btn { text: Theme.say("Show old mod"); spoken: Theme.fill("Show {}", old.modelData.name); tip: Theme.say("The installed mod the list would replace"); onClicked: screen.showMod(old.modelData.modId) }
                                Btn { icon: "eye-off"; text: Theme.say("Ignore"); spoken: Theme.fill("Ignore the replacement of {}", old.modelData.name); tip: Theme.say("No replacement is suggested for this mod again"); onClicked: screen.downloads.ignoreReplacement(old.modelData.modId) }
                                Btn {
                                    danger: true
                                    text: Theme.say("Unsubscribe…")
                                    spoken: Theme.fill("Unsubscribe from {}", old.modelData.name)
                                    tip: Theme.say("Removes the replaced mod: Steam unsubscribes from it and deletes its folder")
                                    onClicked: Mods.unsubscribe(old.modelData.modId)
                                }
                            }
                        }
                    }
                }
            }

            Rectangle {
                visible: screen.side
                Layout.preferredWidth: screen.paneWidth
                Layout.fillHeight: true
                color: Theme.panel
                Rectangle { width: Theme.gapLine; height: parent.height; color: Theme.line }
                Scrolled {
                    anchors.fill: parent
                    padding: Theme.gapWide
                    Loader {
                        active: screen.side
                        width: parent.width
                        sourceComponent: AddMods {}
                    }
                }
            }
        }
    }
}
