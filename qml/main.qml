import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Window 2.12
import QtQuick.Layouts

import demo 1.0

ApplicationWindow {
    visible: true
    title: qsTr("Hello World")
    width: 640
    height: 480

    Hello {
        id : myHello
    }
    
    Reader {
        id : sreader
    }

    // Menu
    menuBar: MenuBar {
        Menu {
            title: qsTr("File")
            MenuItem {
                text: qsTr("&Open")
                onTriggered: console.log("Open action triggered");
            }
            MenuItem {
                text: qsTr("Exit")
                onTriggered: Qt.quit();
            }
        }
    }

    Column {
        anchors.fill: parent
        anchors.margins: 10
        spacing:10
        Button {
            text: qsTr("Say Hello")
            onClicked: myHello.sayHello()
        }

        Button {
            text: qsTr("Test Sql")
            onClicked: sreader.testConnection();
        }
    }
}