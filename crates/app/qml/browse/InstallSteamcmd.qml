import QtQuick
import RimStack.Ui

Btn {
    readonly property var action: Hub.workshop.list.action
    visible: action.installSteamcmd
    primary: true
    enabled: action.enabled
    icon: "download"
    text: action.label
    onClicked: Notices.ask({
        title: Theme.say("Install SteamCMD"),
        lines: [Theme.say("Valve's own downloader is fetched from Valve into"), { path: Hub.settings.report.steamcmdFolder || "" }, Theme.say("It runs with a home folder of its own there, apart from Steam's settings.")],
        label: Theme.say("Install"), run: Hub.downloads.installSteamcmd })
}
