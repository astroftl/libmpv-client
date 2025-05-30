use std::os::raw::c_void;
use libmpv_client_sys::mpv_format;
use crate::*;
use crate::traits::MpvSend;

#[derive(Debug)]
pub enum PropertyValue {
    /// Invalid. Sometimes used for empty values.
    None,
    /// The raw property string, like using ${=property} in input.conf (see input.rst).
    String(String),
    /// The OSD property string, like using ${property} in input.conf (see input.rst).
    /// 
    /// In many cases, this is the same as the raw string, but in other cases it's formatted for display on OSD.
    /// 
    /// It's intended to be human readable. Do not attempt to parse these strings.
    OsdString(OsdString),
    Flag(bool),
    Int64(i64),
    Double(f64),
    Node(Node),
    /// Used with `Node` only. Can usually not be used directly.
    NodeArray(NodeArray),
    /// Used with `Node` only. Can usually not be used directly.
    NodeMap(NodeMap),
    /// Only used only with `Node`, and only in some very specific situations. (Some commands use it.)
    ByteArray(ByteArray),
}

impl PropertyValue {
    pub(crate) unsafe fn from_mpv(format: mpv_format, data: *mut c_void) -> Result<Self> {
        match format {
            libmpv_client_sys::mpv_format_MPV_FORMAT_NONE => Ok(Self::None),
            libmpv_client_sys::mpv_format_MPV_FORMAT_STRING => Ok(Self::String(unsafe { String::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_OSD_STRING => Ok(Self::OsdString(unsafe { OsdString::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_FLAG => Ok(Self::Flag(unsafe { bool::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_INT64 => Ok(Self::Int64(unsafe { i64::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_DOUBLE => Ok(Self::Double(unsafe { f64::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_NODE => Ok(Self::Node(unsafe { Node::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_ARRAY => Ok(Self::NodeArray(unsafe { NodeArray::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_NODE_MAP => Ok(Self::NodeMap(unsafe { NodeMap::from_ptr(data)? })),
            libmpv_client_sys::mpv_format_MPV_FORMAT_BYTE_ARRAY => Ok(Self::ByteArray(unsafe { ByteArray::from_ptr(data)? })),
            _ => unimplemented!()
        }
    }
}