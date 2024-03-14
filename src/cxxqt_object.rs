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