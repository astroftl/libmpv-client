use std::ffi::{c_char, c_int, c_void};
use std::ptr::null;
use libmpv_client_sys::{mpv_format, mpv_format_MPV_FORMAT_NODE_ARRAY, mpv_node, mpv_node_list};
use crate::*;
use crate::node::MpvNode;
use crate::traits::{MpvRepr, MpvSend, ToMpvRepr};

/// Used with mpv_node only. Can usually not be used directly.
#[derive(Debug)]
pub struct NodeArray(pub Vec<Node>);

#[derive(Debug)]
pub(crate) struct MpvNodeArray<'a> {
    _original: &'a NodeArray,

    _node_reprs: Vec<MpvNode<'a>>,
    _flat_nodes: Box<[mpv_node]>,

    node_list: mpv_node_list,
}

impl MpvRepr for MpvNodeArray<'_> {
    type Repr = mpv_node_list;

    fn ptr(&self) -> *const Self::Repr {
        &self.node_list
    }
}

impl MpvSend for NodeArray {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_NODE_ARRAY;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        unsafe { Self::from_node_list_ptr(ptr as *const mpv_node_list) }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut node_list: mpv_node_list = unsafe { std::mem::zeroed() };

        fun(&raw mut node_list as *mut c_void).map(|_| {
            unsafe { Self::from_node_list_ptr(&node_list) }
        })
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let repr = self.to_mpv_repr();

        fun(repr.ptr() as *mut c_void)
    }
}

impl ToMpvRepr for NodeArray {
    type ReprWrap<'a> = MpvNodeArray<'a>;

    fn to_mpv_repr(&self) -> Self::ReprWrap<'_> {
        let guarded_nodes: Vec<_> = self.0.iter().map(|x| x.to_mpv_repr()).collect();
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
            _node_reprs: guarded_nodes,
            _flat_nodes: flat_nodes,
            node_list,
        }
    }
}

impl NodeArray {
    pub(crate) unsafe fn from_node_list_ptr(ptr: *const mpv_node_list) -> Self {
        assert!(!ptr.is_null());

        let data = unsafe { std::slice::from_raw_parts((*ptr).values, (*ptr).num as usize) }
            .iter().map(|x| unsafe { Node::from_node_ptr(x) }).collect();

        Self(data)
    }
}