//! # Safety Notes
//!
//! This module contains several casts from `*const T` to `*mut T` when interfacing
//! with the mpv C API.
//!
//! **Invariant**: All `*const` to `*mut` casts in this module rely on mpv's documented
//! promise to treat the data as read-only.

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::null;
use libmpv_client_sys::{mpv_byte_array, mpv_format, mpv_format_MPV_FORMAT_BYTE_ARRAY, mpv_format_MPV_FORMAT_DOUBLE, mpv_format_MPV_FORMAT_FLAG, mpv_format_MPV_FORMAT_INT64, mpv_format_MPV_FORMAT_NODE, mpv_format_MPV_FORMAT_NODE_ARRAY, mpv_format_MPV_FORMAT_NODE_MAP, mpv_format_MPV_FORMAT_NONE, mpv_format_MPV_FORMAT_OSD_STRING, mpv_format_MPV_FORMAT_STRING, mpv_node, mpv_node_list};
use crate::node::{ByteArray, MpvByteArray, MpvNode, MpvNodeArray, MpvNodeMap, Node, NodeArray, NodeMap};

pub type FormatType = mpv_format;

/// Data format for options and properties.
/// The API functions to get/set properties and options support multiple formats, and this enum describes them.
#[derive(Debug)]
pub enum Format {
    /// Invalid. Sometimes used for empty values.
    None,
    /// It returns the raw property string, like using ${=property} in input.conf (see input.rst).
    String(String),
    /// It returns the OSD property string, like using ${property} in input.conf (see input.rst).
    /// In many cases, this is the same as the raw string, but in other cases it's formatted for display on OSD.
    /// It's intended to be human readable. Do not attempt to parse these strings.
    OsdString(String),
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

impl Format {
    /// Invalid. Sometimes used for empty values.
    pub const NONE: FormatType = mpv_format_MPV_FORMAT_NONE;
    /// It returns the raw property string, like using ${=property} in input.conf (see input.rst).
    pub const STRING: FormatType = mpv_format_MPV_FORMAT_STRING;
    /// It returns the OSD property string, like using ${property} in input.conf (see input.rst).
    /// In many cases, this is the same as the raw string, but in other cases it's formatted for display on OSD.
    /// It's intended to be human readable. Do not attempt to parse these strings.
    pub const OSD_STRING: FormatType = mpv_format_MPV_FORMAT_OSD_STRING;
    pub const FLAG: FormatType = mpv_format_MPV_FORMAT_FLAG;
    pub const INT64: FormatType = mpv_format_MPV_FORMAT_INT64;
    pub const DOUBLE: FormatType = mpv_format_MPV_FORMAT_DOUBLE;
    pub const NODE: FormatType = mpv_format_MPV_FORMAT_NODE;
    /// Used with `Node` only. Can usually not be used directly.
    pub const NODE_ARRAY: FormatType = mpv_format_MPV_FORMAT_NODE_ARRAY;
    /// Used with `Node` only. Can usually not be used directly.
    pub const NODE_MAP: FormatType = mpv_format_MPV_FORMAT_NODE_MAP;
    /// Only used only with `Node`, and only in some very specific situations. (Some commands use it.)
    pub const BYTE_ARRAY: FormatType = mpv_format_MPV_FORMAT_BYTE_ARRAY;
}

/// An MPV-compatible representation of a `Format`.
/// 
/// # Lifetime
/// This struct must be created from a `Format` and must not outlive it.
/// It is intended to be consumed immediately after creation.
/// 
/// The numeric types are used directly from the `Format`. All others are copied to a different representation.
/// Thus, modifying the source `Format` after creating this but prior to consuming it may have unintended results.
#[derive(Debug)]
pub struct MpvFormat<'a> {
    original: &'a Format,

    _owned_cstring: Option<CString>,
    _owned_cstr: Option<*const c_char>,
    _owned_flag: Option<c_int>,
    _guarded_node: Option<MpvNode<'a>>,
    _guarded_array: Option<MpvNodeArray<'a>>,
    _guarded_map: Option<MpvNodeMap<'a>>,
    _guarded_bytes: Option<MpvByteArray<'a>>,
}

impl<'a> MpvFormat<'a> {
    pub fn ptr(&self) -> *const c_void {
        match self.original {
            Format::None => null(),
            Format::String(_) | Format::OsdString(_) => self._owned_cstr.as_ref().unwrap() as *const *const c_char as *const c_void,
            Format::Flag(_) => self._owned_flag.as_ref().unwrap() as *const c_int as *const c_void,
            Format::Int64(x) => x as *const i64 as *const c_void,
            Format::Double(x) => x as *const f64 as *const c_void,
            Format::Node(_) => self._guarded_node.as_ref().unwrap().ptr() as *const c_void,
            Format::NodeArray(_) => self._guarded_array.as_ref().unwrap().ptr() as *const c_void,
            Format::NodeMap(_) => self._guarded_map.as_ref().unwrap().ptr() as *const c_void,
            Format::ByteArray(_) => self._guarded_bytes.as_ref().unwrap().ptr() as *const c_void,
        }
    }
    
    pub fn mut_ptr(&self) -> *mut c_void {
        self.ptr() as *mut c_void
    }
}

impl Format {
    /// Creates a `Format` from an MPV-provided `mpv_format` and `void*`.
    ///
    /// # Safety
    /// - `data` must be a valid pointer to a valid format type.
    /// - `id` MUST match the data type provided in `data`.
    /// - Any `mpv_node` and any data it references must remain valid for the duration of use.
    /// - Any referenced string data must be valid UTF-8/properly formatted.
    pub(crate) fn from_mpv(id: mpv_format, data: *const c_void) -> Self {
        match id {
            Format::NONE => Format::None,
            Format::STRING => {
                let ptr = data as *const *const c_char;
                Format::String(unsafe { CStr::from_ptr(*ptr) }.to_string_lossy().to_string())
            },
            Format::OSD_STRING => {
                let ptr = data as *const *const c_char;
                Format::OsdString(unsafe { CStr::from_ptr(*ptr) }.to_string_lossy().to_string())
            },
            Format::FLAG => {
                Format::Flag(unsafe { *(data as *const c_int) != 0 })
            }
            Format::INT64 => {
                Format::Int64(unsafe { *(data as *const i64) })
            }
            Format::DOUBLE => {
                Format::Double(unsafe { *(data as *const f64) })
            }
            Format::NODE => {
                Format::Node(unsafe { Node::from_ptr(data as *const mpv_node) })
            }
            Format::NODE_ARRAY => {
                Format::NodeArray(unsafe { NodeArray::from_ptr(data as *const mpv_node_list) })
            }
            Format::NODE_MAP => {
                Format::NodeMap(unsafe { NodeMap::from_ptr(data as *const mpv_node_list) })
            }
            Format::BYTE_ARRAY => {
                Format::ByteArray(unsafe { ByteArray::from_ptr(data as *const mpv_byte_array) })
            }
            _ => unimplemented!()
        }
    }

    pub fn to_id(&self) -> mpv_format {
        match self {
            Format::None => Format::NONE,
            Format::String(_) => Format::STRING,
            Format::OsdString(_) => Format::OSD_STRING,
            Format::Flag(_) => Format::FLAG,
            Format::Int64(_) => Format::INT64,
            Format::Double(_) => Format::DOUBLE,
            Format::Node(_) => Format::NODE,
            Format::NodeArray(_) => Format::NODE_ARRAY,
            Format::NodeMap(_) => Format::NODE_MAP,
            Format::ByteArray(_) => Format::BYTE_ARRAY,
        }
    }

    /// Creates an MPV-compatible representation of this `Format`.
    ///
    /// # Safety Guarantees
    /// The returned `MpvFormat` contains pointers that are cast from `*const` to `*mut`
    /// to satisfy MPV's C API signatures. MPV guarantees it will not mutate data passed
    /// through `mpv_node` structures, and this data is managed entirely by us.
    ///
    /// # Lifetime
    /// The returned `MpvFormat` borrows from `self` and must not outlive it.
    /// The internal C-compatible pointers remain valid until the `MpvFormat` is dropped.
    pub fn to_mpv(&self) -> MpvFormat {
        let mut owned_cstring = None;
        let mut owned_cstr = None;
        let mut owned_flag = None;
        let mut guarded_node = None;
        let mut guarded_array = None;
        let mut guarded_map = None;
        let mut guarded_bytes = None;

        match self {
            Format::None => {},
            Format::String(x) | Format::OsdString(x) => {
                owned_cstring = Some(CString::new(x.as_bytes()).unwrap_or_default());
                owned_cstr = Some(owned_cstring.as_ref().unwrap().as_ptr());
            }
            Format::Flag(x) => owned_flag = Some(if *x { 1 } else { 0 }),
            Format::Int64(_) => {}
            Format::Double(_) => {}
            Format::Node(x) => guarded_node = Some(x.to_mpv()),
            Format::NodeArray(x) => guarded_array = Some(x.to_mpv()),
            Format::NodeMap(x) => guarded_map = Some(x.to_mpv()),
            Format::ByteArray(x) => guarded_bytes = Some(x.to_mpv()),
        }

        MpvFormat {
            original: self,
            _owned_cstring: owned_cstring,
            _owned_cstr: owned_cstr,
            _owned_flag: owned_flag,
            _guarded_node: guarded_node,
            _guarded_array: guarded_array,
            _guarded_map: guarded_map,
            _guarded_bytes: guarded_bytes,
        }
    }
}