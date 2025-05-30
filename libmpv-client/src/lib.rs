//! Rust wrapper over mpv/client.h
//!
//! A Rust implementation of the mpv client API, suitable for cplugins.

#[macro_use]
mod macros;

use libmpv_client_sys as mpv;
pub use mpv::mpv_handle;

mod traits;

mod handle;
pub use handle::Handle;

mod property;
pub use property::PropertyValue;

pub mod event;
pub use event::{Event, EventId};

mod types;
pub use types::*;

pub mod error;
pub use error::{Error, Result};

pub mod version;

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