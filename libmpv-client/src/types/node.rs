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
use libmpv_client_sys::{mpv_byte_array, mpv_format, mpv_format_MPV_FORMAT_BYTE_ARRAY, mpv_format_MPV_FORMAT_DOUBLE, mpv_format_MPV_FORMAT_FLAG, mpv_format_MPV_FORMAT_INT64, mpv_format_MPV_FORMAT_NODE, mpv_format_MPV_FORMAT_NODE_ARRAY, mpv_format_MPV_FORMAT_NODE_MAP, mpv_format_MPV_FORMAT_NONE, mpv_format_MPV_FORMAT_STRING, mpv_node, mpv_node__bindgen_ty_1, mpv_node_list};
use crate::*;
use crate::byte_array::MpvByteArray;
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
        let ptr = &raw const self.node;

        // println!("Returning pointer {ptr:p} to node_list: {:#?}", self.node);

        ptr
    }
}

impl Drop for MpvNode<'_> {
    fn drop(&mut self) {
        // println!("Dropping MpvNode");
    }
}

impl MpvSend for Node {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_NODE;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        assert!(!ptr.is_null());

        unsafe { Self::from_node_ptr(ptr as *const mpv_node) }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut node: mpv_node = unsafe { std::mem::zeroed() };

        fun(&raw mut node as *mut c_void).map(|_| {
            let ret = unsafe { Self::from_node_ptr(&node) };
            unsafe { mpv::free_node_contents(&mut node) }
            ret
        })
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
                repr._owned_cstring = Some(CString::new(x.as_bytes()).unwrap_or_default());
                let cstring_ptr = repr._owned_cstring.as_ref().unwrap().as_ptr();

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
                let mpv_ptr = repr._array_repr.as_ref().unwrap().ptr();
                // println!("got pointer {mpv_ptr:p} for _array_repr");

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list },
                    format: mpv_format_MPV_FORMAT_NODE_ARRAY,
                }
            }
            Node::Map(x) => {
                repr._map_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._map_repr.as_ref().unwrap().ptr();
                // println!("got pointer {mpv_ptr:p} for _map_repr");

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list },
                    format: mpv_format_MPV_FORMAT_NODE_MAP,
                }
            }
            Node::ByteArray(x) => {
                repr._bytes_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._bytes_repr.as_ref().unwrap().ptr();

                repr.node = mpv_node {
                    u: mpv_node__bindgen_ty_1 { ba: mpv_ptr as *mut mpv_byte_array },
                    format: mpv_format_MPV_FORMAT_BYTE_ARRAY,
                }
            }
        };

        // println!("created Node repr: {repr:#?}");

        repr
    }
}

impl Node {
    pub(crate) unsafe fn from_node_ptr(ptr: *const mpv_node) -> Self {
        assert!(!ptr.is_null());

        match unsafe { (*ptr).format } {
            mpv_format_MPV_FORMAT_NONE => Node::None,
            mpv_format_MPV_FORMAT_STRING => Node::String(unsafe { CStr::from_ptr((*ptr).u.string) }.to_string_lossy().to_string()),
            mpv_format_MPV_FORMAT_FLAG => Node::Flag(unsafe { (*ptr).u.flag } != 0),
            mpv_format_MPV_FORMAT_INT64 => Node::Int64(unsafe { (*ptr).u.int64 }),
            mpv_format_MPV_FORMAT_DOUBLE => Node::Double(unsafe { (*ptr).u.double_ }),
            mpv_format_MPV_FORMAT_NODE_ARRAY => Node::Array(unsafe { NodeArray::from_node_list_ptr((*ptr).u.list) }),
            mpv_format_MPV_FORMAT_NODE_MAP => Node::Map(unsafe { NodeMap::from_node_list_ptr((*ptr).u.list) }),
            mpv_format_MPV_FORMAT_BYTE_ARRAY => Node::ByteArray(unsafe { ByteArray::from_ptr((*ptr).u.ba as *const c_void ) }),
            _ => unimplemented!()
        }
    }
}