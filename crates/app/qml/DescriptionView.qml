pragma ComponentBehavior: Bound
import QtQuick
import RimStack.Ui

// A description as Rust cut it: each part is `{ html, image, note }`, Qt's rich text, a picture's
// file, or a note standing in for a picture still on its way.
Column {
    id: view
    property var parts: []
    // Links to Workshop items select that item in Browse instead of opening the page.
    property bool inWorkshop: false
    readonly property string linkColour: "<style>a { color: " + Theme.focus + "; }</style>"
    spacing: Theme.gap

    Repeater {
        model: view.parts
        Column {
            id: part
            required property var modelData
            width: view.width
            spacing: Theme.gapSmall
            CopyText {
                visible: part.modelData.html !== ""
                width: parent.width
                textFormat: TextEdit.RichText
                text: view.linkColour + part.modelData.html
                onLinkActivated: link => Qt.openUrlExternally(Hub.workshop.followed(link, view.inWorkshop))
                HoverHandler { enabled: parent.hoveredLink !== ""; cursorShape: Qt.PointingHandCursor }
            }
            Text { visible: !!part.modelData.note; width: parent.width; wrapMode: Text.WordWrap; color: Theme.ink3; font.pixelSize: Theme.small; font.italic: true; text: part.modelData.note || "" }
            Image {
                visible: part.modelData.image !== ""
                width: parent.width
                height: width * implicitHeight / Math.max(implicitWidth, 1)
                asynchronous: true
                fillMode: Image.PreserveAspectFit
                sourceSize.width: Theme.sized(1200)
                source: Theme.fileUrl(part.modelData.image)
            }
        }
    }
}
