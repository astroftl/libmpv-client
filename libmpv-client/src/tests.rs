#![cfg(test)]

pub mod stubs {
    use std::ffi::c_void;
    use libmpv_client_sys::mpv_node;

    #[unsafe(no_mangle)]
    pub extern "C" fn mpv_free(_data: *mut c_void) {
        println!("mpv free()'d")
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn mpv_free_node_contents(_node: *mut mpv_node) {
        println!("mpv free()'d node contents")
    }
}