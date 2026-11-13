pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import RimStack.Ui

FilterField {
    id: filter
    z: 5
    icon: "funnel"
    keys: Theme.say("Ctrl + F")
    chips: true
    placeholder: Theme.say("Filter active and inactive mods")
    found: Hub.mods.filterFound
    readonly property alias suggesting: suggestions.visible
    onTypingChanged: suggestions.shut = false
    onEditingChanged: suggestions.shut = false
    onTapped: suggestions.shut = false
    onEscaped: suggestions.shut = true
    Popup {
        id: suggestions
        property bool shut: false
        visible: offers.count > 0 && filter.editing && filter.visible && !shut
        onClosed: shut = true
        closePolicy: Popup.CloseOnPressOutsideParent
        y: filter.height + 4
        width: Math.min(filter.width, Theme.sized(420))
        padding: 6
        background: Rectangle { radius: Theme.cornerLarge; color: Theme.bar; border.width: 1; border.color: Theme.line }
        readonly property var offered: filter.editing ? Hub.mods.filterOffers(filter.partial, filter.text) : ({ offers: [], widest: "" })
        TextMetrics { id: keyFont; font.family: "monospace"; font.pixelSize: Theme.small; text: suggestions.offered.widest }
        contentItem: Column {
            id: suggestList
            Repeater {
                id: offers
                model: suggestions.offered.offers
                Rectangle {
                    id: offer
                    required property var modelData
                    width: suggestList.width
                    height: Theme.sized(26)
                    radius: Theme.corner
                    color: offerHover.hovered ? Theme.chip : "transparent"
                    Accessible.role: Accessible.Button
                    Accessible.name: Theme.fill("Filter by {}", modelData.term)
                    Text { id: offerKey; width: Math.min(keyFont.advanceWidth, suggestList.width / 2); elide: Text.ElideRight; anchors.left: parent.left; anchors.leftMargin: Theme.gap; anchors.verticalCenter: parent.verticalCenter; color: Theme.ink; font: keyFont.font; text: offer.modelData.term }
                    Text { anchors.left: offerKey.right; anchors.leftMargin: Theme.sized(16); anchors.right: parent.right; anchors.rightMargin: Theme.gap; anchors.verticalCenter: parent.verticalCenter; elide: Text.ElideRight; color: Theme.ink2; font.pixelSize: Theme.small; text: offer.modelData.about }
                    HoverHandler { id: offerHover }
                    TapHandler {
                        onTapped: {
                            filter.take(offer.modelData.term)
                            filter.focusInput()
                        }
                    }
                }
            }
            Text {
                width: suggestList.width
                leftPadding: Theme.gap
                rightPadding: Theme.gap
                topPadding: Theme.gapSmall
                bottomPadding: Theme.gapTiny
                wrapMode: Text.Wrap
                color: Theme.ink3
                font.pixelSize: Theme.small
                text: Theme.say("Use ! to exclude, as in !tag:hud")
            }
        }
    }
}
