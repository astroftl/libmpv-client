//![`Handle`]: Handle
#![doc = include_str!("../../README.md")]
#![warn(missing_docs)]

#[macro_use]
mod macros;

use libmpv_client_sys as mpv;

/// An opaque handle provided by mpv. Only useful when wrapped by [`Handle`].
pub use mpv::mpv_handle;

pub mod handle;
pub use handle::Handle;

pub mod event;
pub use event::{Event, EventId};

pub mod types;
pub use types::*;

pub mod error;
pub use error::{Error, Result};

pub mod version;
mod tests;