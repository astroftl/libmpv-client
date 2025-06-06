//![`Handle`]: Handle
#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

#[macro_use]
mod macros;

use std::ffi::c_void;
use libmpv_client_sys as mpv;

/// An opaque handle provided by mpv. Only useful when wrapped by [`Handle`].
pub use mpv::mpv_handle;

pub mod handle;
pub use handle::Handle;
pub use handle::Client;

pub mod event;
pub use event::{Event, EventId};

pub mod types;
pub use types::*;

pub mod error;
pub use error::{Error, Result};
use libmpv_client_sys::mpv_node;

pub mod version;
mod tests;

pub(crate) unsafe fn mpv_free(data: *mut c_void) {
    #[cfg(not(test))]
    unsafe { mpv::free(data); }
    #[cfg(test)]
    tests::mpv_free_stub(data);
}

pub(crate) unsafe fn mpv_free_node_contents(node: *mut mpv_node) {
    #[cfg(not(test))]
    unsafe { mpv::free_node_contents(node) }
    #[cfg(test)]
    tests::mpv_free_node_contents_stub(node);
}