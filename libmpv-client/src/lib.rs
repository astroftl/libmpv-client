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