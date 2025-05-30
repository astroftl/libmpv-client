#![allow(non_upper_case_globals)]

//! # Safety Notes
//!
//! This module contains several casts from `*const T` to `*mut T` when interfacing
//! with the MPV C API.
//!
//! **Invariant**: All `*const` to `*mut` casts in this module rely on mpv's documented
//! promise to treat the data as read-only.

use std::ffi::{c_void, CStr, CString, c_char};
use std::fmt::Debug;
use libmpv_client_sys as mpv;
use libmpv_client_sys::{mpv_byte_array, mpv_format_MPV_FORMAT_BYTE_ARRAY, mpv_format_MPV_FORMAT_DOUBLE, mpv_format_MPV_FORMAT_FLAG, mpv_format_MPV_FORMAT_INT64, mpv_format_MPV_FORMAT_NODE_ARRAY, mpv_format_MPV_FORMAT_NODE_MAP, mpv_format_MPV_FORMAT_NONE, mpv_format_MPV_FORMAT_STRING, mpv_node, mpv_node__bindgen_ty_1, mpv_node_list};
use crate::*;
use crate::byte_array::MpvByteArray;
use crate::error::RustError;
use crate::node_array::MpvNodeArray;
use crate::node_map::MpvNodeMap;
use crate::traits::{MpvRepr, MpvSend, ToMpvRepr};

/// Generic data storage.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    None,
    String(String),
    Flag(bool),
    Int64(i64),
    Double(f64),
    Array(NodeArray),
    Map(NodeMap),
    ByteArray(ByteArray),
}

#[derive(Debug)]
pub(crate) struct MpvNode<'a> {
    _original: &'a Node,

    _owned_cstring: Option<CString>,
    _array_repr: Option<Box<MpvNodeArray<'a>>>,
    _map_repr: Option<Box<MpvNodeMap<'a>>>,
    _bytes_repr: Option<Box<MpvByteArray<'a>>>,

    pub(crate) node: mpv_node,
}

impl MpvRepr for MpvNode<'_> {
    type Repr = mpv_node;

    fn ptr(&self) -> *const Self::Repr {
        &raw const self.node
    }
}

impl MpvSend for Node {
    const MPV_FORMAT: Format = Format::NODE;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        unsafe { Self::from_node_ptr(ptr as *const mpv_node) }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut node: mpv_node = unsafe { std::mem::zeroed() };

        fun(&raw mut node as *mut c_void).map(|_| {
            let ret = unsafe { Self::from_node_ptr(&node) };
            unsafe { mpv::free_node_contents(&mut node) }
            ret
        })?
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let repr = self.to_mpv_repr();

        let ret = fun(repr.ptr() as *mut c_void);
        
        ret
    }
}

impl ToMpvRepr for Node {
    type ReprWrap<'a> = MpvNode<'a>;

    fn to_mpv_repr(&self) -> Box<Self::ReprWrap<'_>> {
        let mut repr = Box::new(MpvNode {
            _original: self,
            _owned_cstring: None,
            _array_repr: None,
            _map_repr: None,
            _bytes_repr: None,
            node: mpv_node { u: mpv_node__bindgen_ty_1 { int64: 0 }, format: 0 },
        });

        match self {
            Node::None => {
                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { flag: 0 },
                    format: mpv_format_MPV_FORMAT_NONE,
                }
            },
            Node::String(x) => {
                // TODO: Remove this unwrap() by converting to_mpv_repr to return Result<>. See traits.rs.
                repr._owned_cstring = Some(CString::new(x.as_bytes()).unwrap_or_default());
                let cstring_ptr = repr._owned_cstring.as_ref().unwrap().as_ptr(); // SAFETY: We just assigned Some.

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { string: cstring_ptr as *mut c_char },
                    format: mpv_format_MPV_FORMAT_STRING,
                }
            }
            Node::Flag(x) => {
                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { flag: if *x { 1 } else { 0 } },
                    format: mpv_format_MPV_FORMAT_FLAG,
                }
            }
            Node::Int64(x) => {
                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { int64: *x },
                    format: mpv_format_MPV_FORMAT_INT64,
                }
            }
            Node::Double(x) => {
                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { double_: *x },
                    format: mpv_format_MPV_FORMAT_DOUBLE,
                }
            }
            Node::Array(x) => {
                repr._array_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._array_repr.as_ref().unwrap().ptr(); // SAFETY: We just assigned Some.

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list },
                    format: mpv_format_MPV_FORMAT_NODE_ARRAY,
                }
            }
            Node::Map(x) => {
                repr._map_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._map_repr.as_ref().unwrap().ptr(); // SAFETY: We just assigned Some.

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list },
                    format: mpv_format_MPV_FORMAT_NODE_MAP,
                }
            }
            Node::ByteArray(x) => {
                repr._bytes_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._bytes_repr.as_ref().unwrap().ptr(); // SAFETY: We just assigned Some.

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { ba: mpv_ptr as *mut mpv_byte_array },
                    format: mpv_format_MPV_FORMAT_BYTE_ARRAY,
                }
            }
        };
        
        repr
    }
}

impl Node {
    pub(crate) unsafe fn from_node_ptr(ptr: *const mpv_node) -> Result<Self> {
        if ptr.is_null() {
            return Err(Error::Rust(RustError::Pointer))
        }

        let node = unsafe { *ptr };

        match node.format {
            mpv_format_MPV_FORMAT_NONE => Ok(Node::None),
            mpv_format_MPV_FORMAT_STRING => Ok(Node::String(unsafe { CStr::from_ptr(node.u.string) }.to_str()?.to_string())),
            mpv_format_MPV_FORMAT_FLAG => Ok(Node::Flag(unsafe { node.u.flag } != 0)),
            mpv_format_MPV_FORMAT_INT64 => Ok(Node::Int64(unsafe { node.u.int64 })),
            mpv_format_MPV_FORMAT_DOUBLE => Ok(Node::Double(unsafe { node.u.double_ })),
            mpv_format_MPV_FORMAT_NODE_ARRAY => Ok(Node::Array(unsafe { NodeArray::from_ptr(node.u.list as *const c_void)? })),
            mpv_format_MPV_FORMAT_NODE_MAP => Ok(Node::Map(unsafe { NodeMap::from_ptr(node.u.list as *const c_void)? })),
            mpv_format_MPV_FORMAT_BYTE_ARRAY => Ok(Node::ByteArray(unsafe { ByteArray::from_ptr(node.u.ba as *const c_void)? })),
            _ => unimplemented!()
        }
    }
}