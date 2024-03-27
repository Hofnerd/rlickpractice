import QtQuick 2.12
import QtQuick.Controls 2.12
import QtQuick.Window 2.12
import QtQuick.Layouts 1.12

import demo 1.0

ApplicationWindow {
    visible: true
    title: qsTr("Hello World")
    width: 640
    height: 480

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

    Reader {
        id: reader
    }

    RowLayout {
        anchors.fill: parent
        Button {
            text: qsTr("Add Row")
            onClicked: reader.add()
        }
        ScrollView {
            id: scroll
            // anchors.fill: parent
            clip: true
            ScrollBar.vertical.policy: ScrollBar.vertical.size === 1.0 ? ScrollBar.AlwaysOff : ScrollBar.AlwaysOn

            ListView {
                id: listView
                currentIndex: -1
                model: reader
                delegate: ItemDelegate {
                    highlighted: ListView.isCurrentItem
                    text: model.id
                    width: ListView.view.width

                    onClicked: ListView.view.currentIndex = index
                }
            }
        }
    }
    
}