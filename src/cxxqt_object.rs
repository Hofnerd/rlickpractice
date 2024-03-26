#[cxx_qt::bridge(cxx_file_stem = "reader")]
pub mod qobject {


    unsafe extern "C++" {
        include!(<QtCore/QAbstractListModel>);
    }

    unsafe extern "C++" {
        include!("cxx-qt-lib/qhash.h");
        /// QHash<i32, QByteArray> from cxx_qt_lib
        type QHash_i32_QByteArray = cxx_qt_lib::QHash<cxx_qt_lib::QHashPair_i32_QByteArray>;

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

    #[qenum(Reader)] 
    enum State {
        Running,
        Idle,
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        #[base = "QAbstractListModel"]
        #[qproperty(State, state)]
        type Reader = super::SqlReaderRust;
    }

    impl cxx_qt::Threading for Reader {}

    unsafe extern "RustQt" {
        #[inherit]
        #[qsignal]
        fn data_changed(
            self: Pin<&mut Reader>,
            top_left: &QModelIndex,
            bottom_right: &QModelIndex,
            roles: &QVector_i32,
        );
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn add(self: Pin<&mut Reader>);

        #[qinvokable]
        fn add_on_thread_delayed(
            self: Pin<&mut Reader>, 
            counter: i32, 
            delay_ms: u64
        );

        #[qinvokable]
        fn add_on_thread(self: Pin<&mut Reader>, counter: i32);
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        pub fn clear(self: Pin<&mut Reader>);
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        pub fn remove(self: Pin<&mut Reader>, index: i32);
    }

    extern "RustQt" {
        #[inherit]
        unsafe fn begin_insert_rows(
            self: Pin<&mut Reader>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_insert_rows(self: Pin<&mut Reader>);

        #[inherit]
        unsafe fn begin_remove_rows(
            self: Pin<&mut Reader>,
            parent: &QModelIndex,
            first: i32,
            last: i32,
        );

        #[inherit]
        unsafe fn end_remove_rows(self: Pin<&mut Reader>);

        #[inherit]
        unsafe fn begin_reset_model(self: Pin<&mut Reader>);

        #[inherit]
        unsafe fn end_reset_model(self: Pin<&mut Reader>);
    }

    unsafe extern "RustQt" {
        #[cxx_name = "canFetchMore"]
        #[inherit]
        fn base_can_fetch_more(self: &Reader, parent: &QModelIndex) -> bool;

        #[inherit]
        fn index(
            self: &Reader,
            row: i32,
            column: i32,
            parent: &QModelIndex,
        ) -> QModelIndex;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn data(self: &Reader, index: &QModelIndex, role:i32) -> QVariant;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn can_fetch_more(self: &Reader, parent: &QModelIndex) -> bool;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        #[cxx_override]
        fn role_names(self: &Reader) -> QHash_i32_QByteArray;

        #[qinvokable]
        #[cxx_override]
        fn row_count(self: &Reader, _parent: &QModelIndex) -> i32;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn test_connection(self: &Reader);

        #[qinvokable]
        fn get_licks(self: &Reader);
    }
}

use core::pin::Pin;
use crate::sqlreader::SqlReader;
use cxx_qt_lib::QByteArray;
use cxx_qt_lib::QHash;
use cxx_qt_lib::QHashPair_i32_QByteArray;
use cxx_qt_lib::QModelIndex;
use cxx_qt_lib::QVariant;
use cxx_qt::{CxxQtType,Threading};
use crate::sqlreader::Lick;


impl Default for qobject::State {
    fn default() -> Self {
        Self::Idle
    }
}

#[derive(Default)]
pub struct SqlReaderRust {
    state: qobject::State,
    pending_adds: i32,
    pub(crate) id: u32,
    pub(crate) vector: Vec<Lick>,
}

impl qobject::Reader {
    pub fn test_connection(&self) {
        let reader = SqlReader::new();
        reader.test_connection();
    }

    pub fn get_licks(&self) {
        let reader = SqlReader::new();
        let tmp = reader.get_licks();
        match tmp {
            None => {
                return;
            } 
            Some(vals) => {
                for i in vals {
                    println!("Lick: {}", i.get_id());
                }
            }
        }
    }

    /// Add a new row to the QAbstractListModel on the current thread
    pub fn add(self: Pin<&mut Self>) {
        println!("test");
        self.add_cpp_context();
    }

    pub fn add_on_thread_delayed(mut self: Pin<&mut Self>, mut counter: i32, delay_ms: u64) {
        let qt_thread = self.qt_thread();

        self.as_mut().rust_mut().pending_adds += counter;
        self.as_mut().set_state(qobject::State::Running);

        std::thread::spawn(move || {
            while counter > 0 {
                counter -= 1;
                if delay_ms > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                }

                qt_thread
                    .queue(|mut this| {
                        this.as_mut().add_cpp_context();
                        this.as_mut().rust_mut().pending_adds -= 1;
                        if this.pending_adds == 0 {
                            this.set_state(qobject::State::Idle);
                        }
                    })
                    .unwrap();
            }
        });
    }

    pub fn add_on_thread(self:Pin<&mut Self>, counter:i32) {
        self.add_on_thread_delayed(counter, 250);
    }

    fn add_cpp_context(mut self: Pin<&mut Self>) {
        let count = self.vector.len();
        unsafe {
            self.as_mut()
                .begin_insert_rows(&QModelIndex::default(), count as i32, count as i32);
            let id = self.id;
            self.as_mut().rust_mut().id = id + 1;
            self.as_mut()
                .rust_mut()
                .vector
                .push(Lick::new());
            self.as_mut().end_insert_rows();
        }
    }

    pub fn remove(mut self: Pin<&mut Self>, index: i32) {
        if index < 0 || (index as usize) >= self.vector.len() {
            return;
        }

        unsafe {
            self.as_mut()
                .begin_remove_rows(&QModelIndex::default(), index, index);
            self.as_mut().rust_mut().vector.remove(index as usize);
            self.as_mut().end_remove_rows();
        }
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

    pub fn can_fetch_more(&self, parent: &QModelIndex) -> bool {
        self.base_can_fetch_more(parent)
    }

    pub fn role_names(&self) ->QHash<QHashPair_i32_QByteArray> {
        let mut roles = QHash::<QHashPair_i32_QByteArray>::default();
        roles.insert(qobject::Roles::Id.repr, QByteArray::from("id"));
        roles.insert(qobject::Roles::Learned.repr, QByteArray::from("Learned"));
        return roles;
    }

    pub fn row_count(&self, _parent: &QModelIndex) -> i32 {
        return self.vector.len() as i32;
    }

    pub fn clear(mut self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().begin_reset_model();
            self.as_mut().rust_mut().id = 0;
            self.as_mut().rust_mut().vector.clear();
            self.as_mut().end_reset_model();
        }
    }
}
