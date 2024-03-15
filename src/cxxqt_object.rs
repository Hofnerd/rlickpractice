use crate::sqlreader::SqlReader;

#[cxx_qt::bridge]
pub mod qobject {
    unsafe extern "RustQt" {
        // QObject definition
        #[qobject]
        #[qml_element]
        #[qproperty(i32, number)]
        type Hello = super::HelloRust;
    }

    unsafe extern "RustQt" {
        // declare invokeable methods
        #[qinvokable]
        fn say_hello(self: &Hello);
    }

    unsafe extern "RustQt" {
        #[qobject]
        #[qml_element]
        type Reader = super::SqlReaderRust;
    }

    unsafe extern "RustQt" {
        #[qinvokable]
        fn test_connection(self: &Reader);
    }
}

#[derive(Default)]
pub struct HelloRust {
    number : i32,
}

impl qobject::Hello {
    pub fn say_hello(&self) {
        println!("Hello World!");
    }
}

#[derive(Default)]
pub struct SqlReaderRust {
}

impl qobject::Reader {
    pub fn test_connection(&self) {
        let reader = SqlReader::new();
        reader.acquire_connection();
    }
}
