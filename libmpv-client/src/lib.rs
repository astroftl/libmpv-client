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