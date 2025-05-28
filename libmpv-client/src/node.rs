#![allow(non_upper_case_globals)]

//! # Safety Notes
//!
//! This module contains several casts from `*const T` to `*mut T` when interfacing
//! with the MPV C API.
//!
//! **Invariant**: All `*const` to `*mut` casts in this module rely on mpv's documented
//! promise to treat the data as read-only.

use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString};
use std::fmt::Debug;
use std::os::raw::{c_char, c_int};
use std::ptr::null;
use libmpv_client_sys::{mpv_byte_array, mpv_format_MPV_FORMAT_BYTE_ARRAY, mpv_format_MPV_FORMAT_DOUBLE, mpv_format_MPV_FORMAT_FLAG, mpv_format_MPV_FORMAT_INT64, mpv_format_MPV_FORMAT_NODE_ARRAY, mpv_format_MPV_FORMAT_NODE_MAP, mpv_format_MPV_FORMAT_NONE, mpv_format_MPV_FORMAT_STRING, mpv_node, mpv_node__bindgen_ty_1, mpv_node_list};

/// Generic data storage.
#[derive(Debug)]
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

/// An MPV-compatible representation of a `Node`.
///
/// # Lifetime
/// This struct must be created from a `Node` and must not outlive it.
/// It is intended to be consumed immediately after creation.
///
/// # Safety
/// This struct contains offsets to the underlying `Node` and its children.
/// This underlying `Node` (and its children) must NOT be modified before consuming this struct.
#[derive(Debug)]
pub struct MpvNode<'a> {
    _original: &'a Node,

    _owned_cstring: Option<CString>,
    _guarded_array: Option<MpvNodeArray<'a>>,
    _guarded_map: Option<MpvNodeMap<'a>>,
    _guarded_bytes: Option<MpvByteArray<'a>>,

    node: mpv_node,
}

impl<'a> MpvNode<'a> {
    /// Obtain a pointer to the underlying MPV structure suitable for passing to MPV functions.
    ///
    /// # Lifetime
    /// This pointer should not be stored and should be consumed by an MPV function immediately after creation.
    pub fn ptr(&self) -> *const mpv_node {
        &self.node
    }

    pub fn mut_ptr(&self) -> *mut mpv_node {
        self.ptr() as *mut mpv_node
    }
}

impl Node {
    /// Creates a `Node` from an MPV-provided `mpv_node`.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a properly initialized `mpv_node`.
    /// - The `mpv_node` and any data it references must remain valid for the duration of use.
    /// - Any referenced string data must be valid UTF-8/properly formatted.
    pub(crate) unsafe fn from_ptr(ptr: *const mpv_node) -> Self {
        assert!(!ptr.is_null());

        match unsafe { (*ptr).format } {
            mpv_format_MPV_FORMAT_NONE => Node::None,
            mpv_format_MPV_FORMAT_STRING => Node::String(unsafe { CStr::from_ptr((*ptr).u.string) }.to_string_lossy().to_string()),
            mpv_format_MPV_FORMAT_FLAG => Node::Flag(unsafe { (*ptr).u.flag } != 0),
            mpv_format_MPV_FORMAT_INT64 => Node::Int64(unsafe { (*ptr).u.int64 }),
            mpv_format_MPV_FORMAT_DOUBLE => Node::Double(unsafe { (*ptr).u.double_ }),
            mpv_format_MPV_FORMAT_NODE_ARRAY => Node::Array(unsafe { NodeArray::from_ptr((*ptr).u.list) }),
            mpv_format_MPV_FORMAT_NODE_MAP => Node::Map(unsafe { NodeMap::from_ptr((*ptr).u.list) }),
            mpv_format_MPV_FORMAT_BYTE_ARRAY => Node::ByteArray(unsafe { ByteArray::from_ptr((*ptr).u.ba) }),
            _ => unimplemented!()
        }
    }

    /// Creates an MPV-compatible representation of this `Node`.
    ///
    /// # Safety Guarantees
    /// The returned `MpvNode` contains pointers that are cast from `*const` to `*mut`
    /// to satisfy MPV's C API signatures. MPV guarantees it will not mutate data passed
    /// through `mpv_node` structures, and this data is managed entirely by us.
    ///
    /// # Lifetime
    /// The returned `MpvNode` borrows from `self` and must not outlive it.
    /// The internal C-compatible pointers remain valid until the `MpvNode` is dropped.
    pub fn to_mpv(&self) -> MpvNode {
        let mut owned_cstring = None;
        let mut guarded_array = None;
        let mut guarded_map = None;
        let mut guarded_bytes = None;

        let node = match self {
            Node::None => {
                mpv_node {
                    u: mpv_node__bindgen_ty_1 { flag: 0 },
                    format: mpv_format_MPV_FORMAT_NONE,
                }
            },
            Node::String(x) => {
                owned_cstring = Some(CString::new(x.as_bytes()).unwrap_or_default());
                let cstring_ptr = owned_cstring.as_ref().unwrap().as_ptr();

                mpv_node {
                    u: mpv_node__bindgen_ty_1 { string: cstring_ptr as *mut c_char },
                    format: mpv_format_MPV_FORMAT_STRING,
                }
            }
            Node::Flag(x) => {
                mpv_node {
                    u: mpv_node__bindgen_ty_1 { flag: if *x { 1 } else { 0 } },
                    format: mpv_format_MPV_FORMAT_FLAG,
                }
            }
            Node::Int64(x) => {
                mpv_node {
                    u: mpv_node__bindgen_ty_1 { int64: *x },
                    format: mpv_format_MPV_FORMAT_INT64,
                }
            }
            Node::Double(x) => {
                mpv_node {
                    u: mpv_node__bindgen_ty_1 { double_: *x },
                    format: mpv_format_MPV_FORMAT_DOUBLE,
                }
            }
            Node::Array(x) => {
                guarded_array = Some(x.to_mpv());
                let mpv_ptr = guarded_array.as_ref().unwrap().ptr();

                mpv_node {
                    u: mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list },
                    format: mpv_format_MPV_FORMAT_NODE_ARRAY,
                }
            }
            Node::Map(x) => {
                guarded_map = Some(x.to_mpv());
                let mpv_ptr = guarded_map.as_ref().unwrap().ptr();

                mpv_node {
                    u: mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list },
                    format: mpv_format_MPV_FORMAT_NODE_MAP,
                }
            }
            Node::ByteArray(x) => {
                guarded_bytes = Some(x.to_mpv());
                let mpv_ptr = guarded_bytes.as_ref().unwrap().ptr();

                mpv_node {
                    u: mpv_node__bindgen_ty_1 { ba: mpv_ptr as *mut mpv_byte_array },
                    format: mpv_format_MPV_FORMAT_BYTE_ARRAY,
                }
            }
        };

        MpvNode {
            _original: self,
            _owned_cstring: owned_cstring,
            _guarded_array: guarded_array,
            _guarded_map: guarded_map,
            _guarded_bytes: guarded_bytes,
            node,
        }
    }
}

/// Used with mpv_node only. Can usually not be used directly.
#[derive(Debug)]
pub struct NodeArray(pub Vec<Node>);

/// An MPV-compatible representation of a `NodeArray`.
///
/// # Lifetime
/// This struct must be created from a `NodeArray` and must not outlive it.
/// It is intended to be consumed immediately after creation.
///
/// # Safety
/// This struct contains offsets to the underlying `NodeArray`.
/// This underlying `NodeArray` must NOT be modified before consuming this struct.
#[derive(Debug)]
pub(crate) struct MpvNodeArray<'a> {
    _original: &'a NodeArray,

    _guarded_nodes: Vec<MpvNode<'a>>,
    _flat_nodes: Box<[mpv_node]>,

    node_list: mpv_node_list,
}

impl<'a> MpvNodeArray<'a> {
    pub(crate) fn ptr(&self) -> *const mpv_node_list {
        &self.node_list
    }
}

impl NodeArray {
    /// Creates a `NodeArray` from an MPV-provided `mpv_node_list`.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a properly initialized `mpv_node_list`.
    /// - The `mpv_node_list` and any data it references must remain valid for the duration of use.
    /// - Any referenced string data must be valid UTF-8/properly formatted.
    pub(crate) unsafe fn from_ptr(ptr: *const mpv_node_list) -> Self {
        assert!(!ptr.is_null());

        let data = unsafe { std::slice::from_raw_parts((*ptr).values, (*ptr).num as usize) }
            .iter().map(|x| unsafe { Node::from_ptr(x) }).collect();

        Self(data)
    }

    /// Creates an MPV-compatible representation of this `NodeArray`.
    ///
    /// # Safety Guarantees
    /// The returned `MpvNodeArray` contains pointers that are cast from `*const` to `*mut`
    /// to satisfy MPV's C API signatures. MPV guarantees it will not mutate data passed
    /// through `mpv_node_list` structures, and this data is managed entirely by us.
    ///
    /// # Lifetime
    /// The returned `MpvNodeArray` borrows from `self` and must not outlive it.
    /// The internal C-compatible pointers remain valid until the `MpvNodeArray` is dropped.
    pub(crate) fn to_mpv(&self) -> MpvNodeArray {
        let guarded_nodes: Vec<_> = self.0.iter().map(|x| x.to_mpv()).collect();
        let flat_nodes = guarded_nodes.iter().map(|x| x.node).collect::<Vec<_>>().into_boxed_slice();

        let values_ptr = if flat_nodes.is_empty() {
            null()
        } else {
            flat_nodes.as_ptr()
        };

        let node_list = mpv_node_list {
            num: self.0.len() as c_int,
            values: values_ptr as *mut mpv_node,
            keys: null::<c_char>() as *mut *mut c_char,
        };

        MpvNodeArray {
            _original: self,
            _guarded_nodes: guarded_nodes,
            _flat_nodes: flat_nodes,
            node_list,
        }
    }
}

/// Used with mpv_node only. Can usually not be used directly.
#[derive(Debug)]
pub struct NodeMap(pub HashMap<String, Node>);

/// An MPV-compatible representation of a `NodeMap`.
///
/// # Lifetime
/// This struct must be created from a `NodeMap` and must not outlive it.
/// It is intended to be consumed immediately after creation.
///
/// # Safety
/// This struct contains offsets to the underlying `NodeMap`.
/// This underlying `NodeMap` must NOT be modified before consuming this struct.
#[derive(Debug)]
pub(crate) struct MpvNodeMap<'a> {
    _original: &'a NodeMap,

    _guarded_nodes: Vec<MpvNode<'a>>,
    _flat_nodes: Box<[mpv_node]>,

    _owned_keys: Vec<CString>,
    _flat_keys: Box<[*const c_char]>,

    node_list: mpv_node_list,
}

impl MpvNodeMap<'_> {
    pub(crate) fn ptr(&self) -> *const mpv_node_list {
        &self.node_list
    }
}

impl NodeMap {
    /// Creates a `NodeMap` from an MPV-provided `mpv_node_list`.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a properly initialized `mpv_node_list`.
    /// - The `mpv_node_list` MUST be of type `MPV_FORMAT_NODE_MAP`, with N keys.
    /// - The `mpv_node_list` and any data it references must remain valid for the duration of use.
    /// - Any referenced string data must be valid UTF-8/properly formatted.
    pub(crate) unsafe fn from_ptr(ptr: *const mpv_node_list) -> Self {
        assert!(!ptr.is_null());

        let values = unsafe { std::slice::from_raw_parts((*ptr).values, (*ptr).num as usize) }
            .iter().map(|x| unsafe { Node::from_ptr(x) });

        let keys = unsafe { std::slice::from_raw_parts((*ptr).keys, (*ptr).num as usize) }
            .iter().map(|x| unsafe { CStr::from_ptr(*x).to_string_lossy().to_string() });

        let map = keys.zip(values).collect();
        Self(map)
    }

    /// Creates an MPV-compatible representation of this `NodeMap`.
    ///
    /// # Safety Guarantees
    /// The returned `MpvNodeMap` contains pointers that are cast from `*const` to `*mut`
    /// to satisfy MPV's C API signatures. MPV guarantees it will not mutate data passed
    /// through `mpv_node_list` structures, and this data is managed entirely by us.
    ///
    /// # Lifetime
    /// The returned `MpvNodeMap` borrows from `self` and must not outlive it.
    /// The internal C-compatible pointers remain valid until the `MpvNodeMap` is dropped.
    pub(crate) fn to_mpv(&self) -> MpvNodeMap {
        let mut guarded_nodes = Vec::with_capacity(self.0.len());
        let mut owned_keys = Vec::with_capacity(self.0.len());

        for (key, value) in &self.0 {
            owned_keys.push(CString::new(key.as_bytes()).unwrap_or_default());
            guarded_nodes.push(value.to_mpv());
        }

        let flat_nodes = guarded_nodes.iter().map(|x| x.node).collect::<Vec<_>>().into_boxed_slice();
        let flat_keys = owned_keys.iter().map(|x| x.as_ptr()).collect::<Vec<_>>().into_boxed_slice();

        let (values_ptr, keys_ptr) = if self.0.is_empty() {
            (null(), null())
        } else {
            (flat_nodes.as_ptr(), flat_keys.as_ptr())
        };

        let node_list = mpv_node_list {
            num: self.0.len() as c_int,
            values: values_ptr as *mut mpv_node,
            keys: keys_ptr as *mut *mut c_char,
        };

        MpvNodeMap {
            _original: self,
            _guarded_nodes: guarded_nodes,
            _flat_nodes: flat_nodes,
            _owned_keys: owned_keys,
            _flat_keys: flat_keys,
            node_list,
        }
    }
}

/// A raw, untyped byte array. Only used only with `Node`, and only in some very specific situations. (Some commands use it.)
#[derive(Debug, Clone)]
pub struct ByteArray(pub Vec<u8>);

/// An MPV-compatible representation of a `ByteArray`.
///
/// # Lifetime
/// This struct must be created from a `ByteArray` and must not outlive it.
/// It is intended to be consumed immediately after creation.
///
/// # Safety
/// This struct contains offsets to the underlying `ByteArray`.
/// This underlying `ByteArray` must NOT be modified before consuming this struct.
#[derive(Debug)]
pub(crate) struct MpvByteArray<'a> {
    _original: &'a ByteArray,

    byte_array: mpv_byte_array
}

impl MpvByteArray<'_> {
    pub(crate) fn ptr(&self) -> *const mpv_byte_array {
        &self.byte_array
    }
}

impl ByteArray {
    /// Creates a `ByteArray` from an MPV-provided `mpv_byte_array`.
    ///
    /// # Safety
    /// - `ptr` must be a valid pointer to a properly initialized `mpv_byte_array`.
    /// - The `mpv_byte_array` and any data it references must remain valid for the duration of use.
    /// - Any referenced string data must be valid UTF-8/properly formatted.
    pub(crate) unsafe fn from_ptr(ptr: *const mpv_byte_array) -> Self {
        assert!(!ptr.is_null());
        let data = unsafe { std::slice::from_raw_parts((*ptr).data as *const u8, (*ptr).size) }.to_vec();
        Self(data)
    }

    /// Creates an MPV-compatible representation of this `ByteArray`.
    ///
    /// # Safety Guarantees
    /// The returned `MpvByteArray` contains pointers that are cast from `*const` to `*mut`
    /// to satisfy MPV's C API signatures. MPV guarantees it will not mutate data passed
    /// through `mpv_byte_array` structures, and this data is managed entirely by us.
    ///
    /// # Lifetime
    /// The returned `MpvByteArray` borrows from `self` and must not outlive it.
    /// The internal C-compatible pointers remain valid until the `MpvByteArray` is dropped.
    pub(crate) fn to_mpv(&self) -> MpvByteArray {
        MpvByteArray {
            _original: self,
            byte_array: mpv_byte_array {
                data: self.0.as_ptr() as *mut c_void,
                size: self.0.len(),
            },
        }
    }
}