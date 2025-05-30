pub(crate) mod basics;
pub(crate) mod node;
pub(crate) mod node_array;
pub(crate) mod node_map;
pub(crate) mod byte_array;
mod tests;

pub use basics::OsdString;
pub use node::Node;
pub use node_array::NodeArray;
pub use node_map::NodeMap;
pub use byte_array::ByteArray;
use libmpv_client_sys::mpv_format;

pub struct Format(pub(crate) mpv_format);

impl Format {
    pub const NONE: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NONE);
    pub const STRING: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_STRING);
    pub const OSD_STRING: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_OSD_STRING);
    pub const FLAG: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_FLAG);
    pub const INT64: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_INT64);
    pub const DOUBLE: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_DOUBLE);
    pub const NODE: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NODE);
    pub const NODE_ARRAY: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_ARRAY);
    pub const NODE_MAP: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_MAP);
    pub const BYTE_ARRAY: Format = Format(libmpv_client_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY);
}