#![allow(non_upper_case_globals)]

use std::collections::HashMap;
use std::ffi::{c_void, CStr, CString, c_char};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem::MaybeUninit;
use libmpv_client_sys::{mpv_byte_array, mpv_format_MPV_FORMAT_BYTE_ARRAY, mpv_format_MPV_FORMAT_DOUBLE, mpv_format_MPV_FORMAT_FLAG, mpv_format_MPV_FORMAT_INT64, mpv_format_MPV_FORMAT_NODE_ARRAY, mpv_format_MPV_FORMAT_NODE_MAP, mpv_format_MPV_FORMAT_NONE, mpv_format_MPV_FORMAT_STRING, mpv_node, mpv_node__bindgen_ty_1, mpv_node_list};
use crate::*;
use crate::byte_array::MpvByteArray;
use crate::node_array::MpvNodeArray;
use crate::node_map::MpvNodeMap;
use crate::types::traits::{MpvFormat, MpvRecv, MpvRecvInternal, MpvRepr, MpvSend, MpvSendInternal, ToMpvRepr};

/// Generic data storage for various mpv argument types and responses.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    /// The [`Node`] is empty. See [`Format::NONE`].
    None,
    /// The [`Node`] contains a string. See [`Format::STRING`].
    String(String),
    /// The [`Node`] contains a boolean flag. See [`Format::NONE`].
    Flag(bool),
    /// The [`Node`] contains an integer. See [`Format::INT64`].
    Int64(i64),
    /// The [`Node`] contains a double. See [`Format::DOUBLE`].
    Double(f64),
    /// The [`Node`] contains an array of [`Node`]s. See [`Format::NODE_ARRAY`].
    Array(NodeArray),
    /// The [`Node`] contains a map of [`String`] keys and [`Node`] values. See [`Format::NODE_MAP`].
    Map(NodeMap),
    /// The [`Node`] contains a raw, untyped byte array. See [`Format::BYTE_ARRAY`].
    ByteArray(ByteArray),
}

#[derive(Debug)]
pub(crate) struct MpvNode<'a> {
    _original: PhantomData<&'a Node>,

    _owned_cstring: Option<CString>,
    _array_repr: Option<MpvNodeArray<'a>>,
    _map_repr: Option<MpvNodeMap<'a>>,
    _bytes_repr: Option<MpvByteArray<'a>>,

    pub(crate) node: Box<mpv_node>,
}

impl MpvRepr for MpvNode<'_> {
    type Repr = mpv_node;

    fn ptr(&self) -> *const Self::Repr {
        &raw const *self.node
    }
}

impl MpvFormat for Node {
    const MPV_FORMAT: Format = Format::NODE;
}

impl MpvRecv for Node {}
impl MpvRecvInternal for Node {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        unsafe { Self::from_node_ptr(ptr as *const mpv_node) }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut node: MaybeUninit<mpv_node> = MaybeUninit::uninit();

        fun(node.as_mut_ptr() as *mut c_void).map(|_| {
            let ret = unsafe { Self::from_node_ptr(node.as_ptr()) };
            unsafe { mpv_free_node_contents(node.as_mut_ptr()) }
            ret
        })?
    }
}

impl MpvSend for Node {}
impl MpvSendInternal for Node {
    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let repr = self.to_mpv_repr();

        fun(repr.ptr() as *mut c_void)
    }
}

impl ToMpvRepr for Node {
    type ReprWrap<'a> = MpvNode<'a>;

    fn to_mpv_repr(&self) -> Self::ReprWrap<'_> {
        let mut repr = MpvNode {
            _original: PhantomData,
            _owned_cstring: None,
            _array_repr: None,
            _map_repr: None,
            _bytes_repr: None,
            node: Box::new(mpv_node { u: mpv_node__bindgen_ty_1 { int64: 0 }, format: 0 }),
        };

        match self {
            Node::None => {
                repr.node.u = mpv_node__bindgen_ty_1 { flag: 0 };
                repr.node.format = mpv_format_MPV_FORMAT_NONE;
            },
            Node::String(x) => {
                // TODO: Remove this unwrap() by converting to_mpv_repr to return Result<>. See traits.rs.
                repr._owned_cstring = Some(CString::new(x.as_bytes()).unwrap_or_default());
                let cstring_ptr = repr._owned_cstring.as_ref().unwrap().as_ptr(); // SAFETY: We just assigned Some.

                repr.node.u = mpv_node__bindgen_ty_1 { string: cstring_ptr as *mut c_char };
                repr.node.format = mpv_format_MPV_FORMAT_STRING;
            }
            Node::Flag(x) => {
                repr.node.u = mpv_node__bindgen_ty_1 { flag: if *x { 1 } else { 0 } };
                repr.node.format = mpv_format_MPV_FORMAT_FLAG;
            }
            Node::Int64(x) => {
                repr.node.u = mpv_node__bindgen_ty_1 { int64: *x };
                repr.node.format = mpv_format_MPV_FORMAT_INT64;
            }
            Node::Double(x) => {
                repr.node.u = mpv_node__bindgen_ty_1 { double_: *x };
                repr.node.format = mpv_format_MPV_FORMAT_DOUBLE;
            }
            Node::Array(x) => {
                repr._array_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._array_repr.as_ref().unwrap().ptr(); // SAFETY: We just assigned Some.

                repr.node.u = mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list };
                repr.node.format = mpv_format_MPV_FORMAT_NODE_ARRAY;
            }
            Node::Map(x) => {
                repr._map_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._map_repr.as_ref().unwrap().ptr(); // SAFETY: We just assigned Some.

                repr.node.u = mpv_node__bindgen_ty_1 { list: mpv_ptr as *mut mpv_node_list };
                repr.node.format = mpv_format_MPV_FORMAT_NODE_MAP;
            }
            Node::ByteArray(x) => {
                repr._bytes_repr = Some(x.to_mpv_repr());
                let mpv_ptr = repr._bytes_repr.as_ref().unwrap().ptr(); // SAFETY: We just assigned Some.

                repr.node.u = mpv_node__bindgen_ty_1 { ba: mpv_ptr as *mut mpv_byte_array };
                repr.node.format = mpv_format_MPV_FORMAT_BYTE_ARRAY;
            }
        };

        repr
    }
}

impl Node {
    pub(crate) unsafe fn from_node_ptr(ptr: *const mpv_node) -> Result<Self> {
        check_null!(ptr);
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

impl From<&[(&str, Node)]> for Node {
    /// Convenience function to create a [`Node::Map`] from a [`&[(&str, Node)]`] slice.
    ///
    /// This creates the underlying [`HashMap`] and clones the references [`Node`]s.
    fn from(slice: &[(&str, Node)]) -> Self {
        let map: HashMap<String, Node> = slice.iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect();
        Node::Map(map)
    }
}

impl From<&[Node]> for Node {
    /// Convenience function to create a [`Node::Map`] from a [`&[Node]`] slice.
    ///
    /// This creates the underlying [`Vec`].
    fn from(slice: &[Node]) -> Self {
        Node::Array(slice.to_vec())
    }
}