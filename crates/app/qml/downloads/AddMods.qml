pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Layouts
import RimStack.Ui

Column {
    id: adding
    readonly property DownloadsData downloads: Hub.downloads
    readonly property var found: downloads.queue || ({})
    readonly property var queue: found.items || []
    readonly property var installed: found.installed || []
    readonly property var repositories: found.repositories || []
    property bool installedOpen: false
    readonly property var info: Hub.settings.report
    spacing: Theme.gapPage

    function day(seconds) {
        return seconds ? Theme.day(seconds) : ""
    }

    function megabytes(n) {
        return !n ? "" : n < 10 ? Theme.fill("{} MB", n.toFixed(1)) : Theme.fill("{} MB", Theme.count(Math.round(n)))
    }

    component AskedRow: RowLayout {
        id: item
        required property var modelData
        width: adding.width
        spacing: Theme.gap
        Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; text: item.modelData.title || item.modelData.id; Tip { text: adding.day(item.modelData.updated) } }
        Text { color: Theme.ink3; font.pixelSize: Theme.small; text: adding.megabytes(item.modelData.megabytes) }
        Chip { visible: !item.modelData.title; tone: Theme.err; icon: "circle-alert"; text: Theme.say("Not on the Workshop") }
        Chip { visible: !!item.modelData.title && item.modelData.installed; tone: Theme.ok; icon: "check"; text: Theme.say("Installed") }
        Btn { visible: !!item.modelData.title && !item.modelData.installed; icon: "external-link"; tip: Theme.say("Workshop page"); spoken: Theme.fill("Show {}", item.modelData.title || ""); onClicked: Theme.openWorkshop(item.modelData.id) }
    }

    Card {
        visible: adding.queue.length + adding.installed.length + adding.repositories.length > 0 || !!(adding.found.problem || adding.found.note)
        title: Theme.say("Look up")
        note: adding.queue.length + adding.repositories.length > 0 ? adding.queue.length + adding.repositories.length : ""
        Text { visible: adding.queue.length === 0 && !!(adding.found.problem || adding.found.note); width: parent.width; wrapMode: Text.WordWrap; color: adding.found.problem ? Theme.err : Theme.ink2; font.pixelSize: Theme.body; text: adding.found.problem || adding.found.note || "" }
        Repeater {
            model: adding.found.collections || []
            RowLayout {
                id: collection
                required property string modelData
                width: adding.width
                spacing: Theme.gap
                Icon { name: "package"; color: Theme.ink3 }
                Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; font.weight: Font.DemiBold; text: collection.modelData }
            }
        }
        Repeater {
            model: adding.repositories
            RowLayout {
                id: repository
                required property var modelData
                required property int index
                readonly property bool cloning: adding.downloads.cloning === index
                width: adding.width
                spacing: Theme.gap
                Icon { name: "git-branch"; color: Theme.ink3 }
                Text { Layout.fillWidth: true; Layout.preferredWidth: 0; elide: Text.ElideRight; color: Theme.ink; font.pixelSize: Theme.body; text: repository.modelData.name }
                Chip { visible: repository.modelData.installed; tone: Theme.ok; icon: "check"; text: Theme.say("Installed") }
                Btn { icon: "external-link"; tip: Theme.say("Repository page"); onClicked: Qt.openUrlExternally(repository.modelData.link) }
                Btn {
                    visible: !repository.modelData.installed
                    enabled: !!adding.info.git && !!repository.modelData.into && adding.downloads.cloning < 0
                    primary: !repository.cloning
                    icon: "download"
                    text: repository.cloning ? Theme.say("Cloning…") : Theme.say("Clone…")
                    tip: adding.info.git ? Theme.say("git clone into the game's own Mods folder; Check for updates then asks its repository") : Theme.say("Git is not installed")
                    onClicked: Notices.ask({
                        title: Theme.fill("Clone {}", repository.modelData.name),
                        lines: [Theme.say("git clone, from"), { path: repository.modelData.link }, Theme.say("Into"), { path: repository.modelData.into }],
                        label: Theme.say("Clone"), run: () => adding.downloads.cloneRepository(repository.index) })
                }
            }
        }
        Repeater {
            model: adding.queue
            AskedRow {}
        }
        Btn {
            visible: adding.installed.length > 0
            icon: "check"
            text: Theme.plural(adding.installed.length, "{} mod already installed", "{} mods already installed")
            onClicked: adding.installedOpen = !adding.installedOpen
        }
        Repeater {
            model: adding.installedOpen ? adding.installed : []
            AskedRow {}
        }
    }

    Card {
        title: Theme.say("Download list")
        note: Hub.workshop.list.sizeSaid || ""
        ListPane { width: parent.width }
    }

    Card {
        title: Theme.say("SteamCMD")
        actions: Chip { visible: adding.downloads.steamcmd; tone: Theme.ok; icon: "check"; text: Theme.say("Installed") }
        Text { width: parent.width; wrapMode: Text.WordWrap; color: Theme.ink3; font.pixelSize: Theme.small; text: Theme.say("Fetches a mod without subscribing."); Tip { text: Theme.say("Valve's own downloader, fetched from Valve into RimStack's data folder. It downloads Workshop mods without a Steam account, into the game's Mods folder. It runs with a home folder of its own, so it cannot touch Steam's settings.") } }
        Text { visible: !!adding.info.steamcmdRefused; width: parent.width; wrapMode: Text.WordWrap; color: Theme.ink3; font.pixelSize: Theme.small; text: adding.info.steamcmdRefused || "" }
    }
}
