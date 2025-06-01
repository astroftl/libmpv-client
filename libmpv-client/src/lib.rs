//! Rust wrapper over mpv/client.h
//!
//! A Rust implementation of the mpv client API, suitable for cplugins.

#![warn(missing_docs)]

#[macro_use]
mod macros;

use libmpv_client_sys as mpv;
pub use mpv::mpv_handle;

mod traits;

mod handle;
#[doc(inline)]
pub use handle::Handle;

mod property;
#[doc(inline)]
pub use property::PropertyValue;

pub mod event;
#[doc(inline)]
pub use event::{Event, EventId};

mod types;
pub use types::*;

pub mod error;
#[doc(inline)]
pub use error::{Error, Result};

pub mod version;
mod tests;