use std::os::raw::c_void;
use libmpv_client_sys::mpv_format;
use crate::*;
use crate::traits::MpvSend;

#[derive(Debug)]
/// An enum of the possible values returned in a [`GetPropertyReply`](event::GetPropertyReply) or a [`PropertyChange`](event::PropertyChange).
pub enum PropertyValue {
    /// Sometimes used for empty values or errors. See [`Format::NONE`].
    None,
    /// A raw property string. See [`Format::STRING`].
    String(String),
    /// An OSD property string. See [`Format::OSD_STRING`].
    OsdString(OsdString),
    /// A flag property. See [`Format::FLAG`].
    Flag(bool),
    /// An int64 property. See [`Format::INT64`].
    Int64(i64),
    /// A double property. See [`Format::DOUBLE`].
    Double(f64),
    /// A [`Node`] property. See [`Format::NODE`].
    Node(Node),
    /// A [`NodeArray`] property. See [`Format::NODE_ARRAY`].
    NodeArray(NodeArray),
    /// A [`NodeMap`] property. See [`Format::NODE_MAP`].
    NodeMap(NodeMap),
    /// A [`ByteArray`] property. See [`Format::BYTE_ARRAY`].
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