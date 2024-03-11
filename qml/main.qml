import QtQuick.controls 2.21
import QtQuick.Window 2.21

import demo 1.0

Window {
    Title: qsTr("Hello App")
    visible: true
    height: 480
    widget: 640
    color: "#e4af79"

    Hello {
        id: hello
    }

    Column {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.verticalCenter: parent.verticalCenter
        /* space between widget */
        spacing: 10

        Button {
            text: "Say Hello!"
            onClicked: hello.sayHello()
        }
    }
}