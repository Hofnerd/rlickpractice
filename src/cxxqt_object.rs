use crate::sqlreader::SqlReader;
use cxx_qt_lib::QModelIndex;
use cxx_qt_lib::QVariant;
use crate::sqlreader::Lick;

#[cxx_qt::bridge(cxx_file_stem = "reader")]
pub mod qobject {

    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qvariant.h");
        /// QVariant from cxx_qt_lib
        type QVariant = cxx_qt_lib::QVariant;

        include!("cxx-qt-lib/qmodelindex.h");
        /// QModelIndex from cxx_qt_lib
        type QModelIndex = cxx_qt_lib::QModelIndex;


        include!("cxx-qt-lib/qvector.h");
        type QVector_i32 = cxx_qt_lib::QVector<i32>;
    }

    #[qenum(Reader)]
    enum Roles {
        Id,
        Learned,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = "QAbstractListModel"]
        type Reader = super::SqlReaderRust;
    }

    impl cxx_qt::Threading for Reader {}

    unsafe extern "RustQt" {
        #[qinvokable]
        fn test_connection(self: &Reader);

        #[qinvokable]
        fn get_licks(self: &Reader);

    }
    unsafe extern "RustQt" {

        #[qinvokable]
        #[cxx_override]
        fn data(self: &Reader, index: &QModelIndex, role:i32) -> QVariant;
    }

    // unsafe extern "RustQt" {
    //     #[inherit]
    //     #[qsignal]
    //     fn data_changed(
    //         self: Pin<&mut Reader>,
    //         top_left: &QModelIndex,
    //         bottom_right: QModelIndex,
    //         roles: QVector_i32,
    //     );
    // }
}

#[derive(Default)]
pub struct SqlReaderRust {
    pub(crate) vector: Vec<Lick>,
}

impl qobject::Reader {
    pub fn test_connection(&self) {
        let reader = SqlReader::new();
        reader.test_connection();
    }

    pub fn get_licks(&self) {
        let reader = SqlReader::new();
        reader.get_licks();
    }

    pub fn data(&self, index: &QModelIndex, role:i32) -> QVariant {
        let role = qobject::Roles {repr: role};

        if let Some(lick) = self.vector.get(index.row() as usize) {
            return match role {
                qobject::Roles::Id              => QVariant::from(&lick.get_id()),
                qobject::Roles::Learned         => QVariant::from(&lick.get_learned()),
                _                               => QVariant::default(),
            }
        }

        QVariant::default()
    }
}
