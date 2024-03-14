mod cxxqt_object;
use cxx_qt_lib::{QGuiApplication, QQmlApplicationEngine, QUrl};

// mod sqlreader;

fn main() {
    let mut app = QGuiApplication::new();
    let mut engine = QQmlApplicationEngine::new();
    
    if let Some(engine) = engine.as_mut() {
        engine.load(&QUrl::from("qml/qml.qrc"));
    }
    
    if let Some(app) = app.as_mut() {
        app.exec();
    }
}
