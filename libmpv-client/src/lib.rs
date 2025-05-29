//! Rust wrapper over mpv/client.h
//!
//! A Rust implementation of the mpv client API, suitable for cplugins.

mod handle;
mod event;
mod error;
mod property;
mod types;
mod traits;
pub mod version;

use libmpv_client_sys as mpv;
pub use mpv::mpv_handle;
pub use handle::Handle;
pub use event::Event;
pub use property::PropertyValue;
pub use types::*;
pub use error::{Error, Result};

#[cfg(test)]
pub use mpv_stubs::setup_mpv_stubs;

#[cfg(test)]
mod mpv_stubs {
    use std::ffi::c_void;
    use libmpv_client_sys::{mpv_node, pfn_mpv_free, pfn_mpv_free_node_contents};

    #[unsafe(no_mangle)]
    pub extern "C" fn mpv_free(_data: *mut c_void) {
        println!("mpv free()'d")
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn mpv_free_node_contents(_node: *mut mpv_node) {
        println!("mpv free()'d node contents")
    }

    pub fn setup_mpv_stubs() {
        unsafe {
            pfn_mpv_free = Some(mpv_free);
            pfn_mpv_free_node_contents = Some(mpv_free_node_contents)
        }
    }
}