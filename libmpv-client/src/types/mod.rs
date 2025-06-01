//! Definitions and trait implementations for the various types that can be used to communicate with mpv.
pub(crate) mod basics;
pub(crate) mod node;
pub(crate) mod node_array;
pub(crate) mod node_map;
pub(crate) mod byte_array;
pub(crate) mod traits;
mod tests;

pub use node::Node;
pub use node_array::NodeArray;
pub use node_map::NodeMap;
pub use byte_array::ByteArray;
pub use basics::OsdString;

pub use traits::MpvFormat;
pub use traits::MpvSend;
pub use traits::MpvRecv;

use libmpv_client_sys::mpv_format;

/// A type representing the possible data types used in communication with mpv.
pub struct Format(pub(crate) mpv_format);

impl Format {
    /// A [`Format`] representing [`MPV_FORMAT_NONE`](libmpv_client_sys::mpv_format_MPV_FORMAT_NONE),
    /// sometimes returned from mpv to denote an error or other special circumstance.
    pub const NONE: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NONE);
    /// A [`Format`] representing Rust's [`String`]
    /// and mpv's [`MPV_FORMAT_STRING`](libmpv_client_sys::mpv_format_MPV_FORMAT_STRING).
    ///
    /// It represents a raw property string, like using `${=property}` in `input.conf`.
    /// See [the mpv docs on raw and formatted properties](https://mpv.io/manual/stable/#raw-and-formatted-properties).
    ///
    /// <div class="warning">
    ///
    /// # Warning
    /// Although the encoding is usually UTF-8, this is not always the case. File tags often store strings in some legacy codepage,
    /// and even filenames don't necessarily have to be in UTF-8 (at least on Linux).
    /// 
    /// If this crate receives invalid UTF-8, a [`RustError::InvalidUtf8`](crate::error::RustError::InvalidUtf8) may be returned.
    ///
    /// On Windows, filenames are always UTF-8, and libmpv converts between UTF-8 and UTF-16 when using Win32 API functions.
    ///
    /// </div>
    pub const STRING: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_STRING);
    /// A [`Format`] representing [`OsdString`] (a New Type around [`String`])
    /// and mpv's [`MPV_FORMAT_OSD_STRING`](libmpv_client_sys::mpv_format_MPV_FORMAT_OSD_STRING).
    ///
    /// It represents an OSD property string, like using `${property}` in `input.conf`.
    /// See [the mpv docs on raw and formatted properties](https://mpv.io/manual/stable/#raw-and-formatted-properties).
    ///
    /// In many cases, this is the same as the raw string, but in other cases it's formatted for display on OSD.
    ///
    /// It's intended to be human-readable. Do not attempt to parse these strings.
    pub const OSD_STRING: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_OSD_STRING);
    /// A [`Format`] representing Rust's [`bool`]
    /// and mpv's [`MPV_FORMAT_FLAG`](libmpv_client_sys::mpv_format_MPV_FORMAT_FLAG).
    pub const FLAG: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_FLAG);
    /// A [`Format`] representing Rust's [`i64`]
    /// and mpv's [`MPV_FORMAT_INT64`](libmpv_client_sys::mpv_format_MPV_FORMAT_INT64).
    pub const INT64: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_INT64);
    /// A [`Format`] representing Rust's [`f64`]
    /// and mpv's [`MPV_FORMAT_DOUBLE`](libmpv_client_sys::mpv_format_MPV_FORMAT_DOUBLE).
    pub const DOUBLE: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_DOUBLE);
    /// A [`Format`] representing the crate's [`Node`]
    /// and mpv's [`MPV_FORMAT_NODE`](libmpv_client_sys::mpv_format_MPV_FORMAT_NODE).
    pub const NODE: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NODE);
    /// A [`Format`] representing the crate's [`NodeArray`] (a type alias for [`Vec<Node>`])
    /// and mpv's [`MPV_FORMAT_NODE_ARRAY`](libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_ARRAY).
    pub const NODE_ARRAY: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_ARRAY);
    /// A [`Format`] representing the crate's [`NodeMap`] (a type alias for [`HashMap<String, Node>`](std::collections::HashMap))
    /// and mpv's [`MPV_FORMAT_NODE_MAP`](libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_MAP).
    pub const NODE_MAP: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_MAP);
    /// A [`Format`] representing the crate's [`ByteArray`] (a type alias for [`Vec<u8>`])
    /// and mpv's [`MPV_FORMAT_BYTE_ARRAY`](libmpv_client_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY).
    pub const BYTE_ARRAY: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY);
}