#[cfg(debug_assertions)]
macro_rules! build_debug_loc {
    ($var:expr) => {
        Some(crate::error::DebugLoc {
            file: file!(),
            line: line!(),
            function: {
                fn f() {}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                type_name_of(f)
            },
            variable: Some(stringify!($var)),
        })
    };
}

#[cfg(not(debug_assertions))]
macro_rules! build_debug_loc {
    ($var:expr) => {
        None
    };
}

macro_rules! check_null {
    ($ptr:expr) => {
        if $ptr.is_null() {
            return Err(crate::Error::Rust(crate::error::RustError::Pointer(build_debug_loc!($ptr))));
        }
    };
}