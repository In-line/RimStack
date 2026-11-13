import QtQuick
import RimStack.Ui

Rectangle {
    id: notice
    property color tone: Theme.warn
    radius: Theme.gapHair
    color: Qt.rgba(tone.r, tone.g, tone.b, 0.06)
    Rectangle { width: 2; height: parent.height; color: notice.tone }
}
