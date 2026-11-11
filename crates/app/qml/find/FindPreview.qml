pragma ComponentBehavior: Bound
import QtQuick
import QtQuick.Controls.Basic
import QtQuick.Layouts
import RimStack.Ui

Rectangle {
    id: panel
    required property var shown
    required property int row
    readonly property var lines: shown.lines || []
    readonly property int first: (lines[0] || ({ number: 0 })).number
    color: Theme.panel

    TextMetrics { id: digits; font.family: "monospace"; font.pixelSize: Theme.small; text: "00000" }

    ColumnLayout {
        anchors.fill: parent
        spacing: Theme.gapSmall
        Text {
            Layout.fillWidth: true
            Layout.topMargin: Theme.gapLarge
            Layout.leftMargin: Theme.gapLarge
            Layout.rightMargin: Theme.gapLarge
            elide: Text.ElideLeft
            textFormat: Text.PlainText
            color: Theme.ink
            font.family: "monospace"
            font.pixelSize: Theme.small
            text: panel.shown.path || ""
        }
        Flow {
            Layout.fillWidth: true
            Layout.leftMargin: Theme.gapLarge
            Layout.rightMargin: Theme.gapLarge
            spacing: Theme.gapSmall
            Btn { icon: "external-link"; text: Theme.say("Open file"); onClicked: Qt.openUrlExternally(Theme.fileUrl(Hub.find.openFile(panel.row))) }
            Btn { icon: "folder-open"; text: Theme.say("Open folder"); onClicked: Qt.openUrlExternally(Theme.fileUrl(Hub.find.openFolder(panel.row))) }
            Btn { icon: "copy"; text: Theme.say("Copy path"); onClicked: Theme.copy(Hub.find.copyPath(panel.row)) }
            Btn { icon: "copy"; text: Theme.say("Copy line"); onClicked: Theme.copy(Hub.find.copyLine(panel.row)) }
        }
        Text {
            visible: !!panel.shown.problem
            Layout.fillWidth: true
            Layout.leftMargin: Theme.gapLarge
            Layout.rightMargin: Theme.gapLarge
            wrapMode: Text.WordWrap
            color: Theme.err
            font.pixelSize: Theme.small
            text: panel.shown.problem || ""
        }
        ListView {
            id: view
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            boundsBehavior: Flickable.StopAtBounds
            model: panel.lines
            currentIndex: Math.max(0, panel.shown.line - panel.first)
            highlightRangeMode: ListView.ApplyRange
            highlightMoveDuration: 0
            preferredHighlightBegin: height / 2 - Theme.row
            preferredHighlightEnd: height / 2 + Theme.row
            Wheel {}
            ScrollBar.vertical: Scroller {}
            delegate: Item {
                id: line
                required property var modelData
                width: view.width
                height: code.implicitHeight + Theme.gapHair
                Rectangle { visible: line.modelData.marked; anchors.fill: parent; color: Theme.accent; opacity: 0.25 }
                Text {
                    id: number
                    x: Theme.gap
                    width: digits.advanceWidth
                    horizontalAlignment: Text.AlignRight
                    color: Theme.ink3
                    font.family: "monospace"
                    font.pixelSize: Theme.small
                    text: line.modelData.number
                }
                Text {
                    id: code
                    x: number.x + number.width + Theme.gap
                    width: parent.width - x - Theme.gapLarge
                    elide: Text.ElideRight
                    textFormat: Text.PlainText
                    color: Theme.ink
                    font.family: "monospace"
                    font.pixelSize: Theme.small
                    font.weight: Font.Normal
                    text: line.modelData.text
                }
            }
        }
    }
    Rectangle { anchors.left: parent.left; width: 1; height: parent.height; color: Theme.edge }
}
