use std::collections::HashMap;
use std::ffi::{CStr, CString,c_char, c_int, c_void};
use std::ptr::null;
use libmpv_client_sys::{mpv_format, mpv_format_MPV_FORMAT_NODE_MAP, mpv_node, mpv_node_list};
use crate::*;
use crate::node::MpvNode;
use crate::traits::{MpvRepr, MpvSend, ToMpvRepr};

/// Used with mpv_node only. Can usually not be used directly.
#[derive(Debug)]
pub struct NodeMap(pub HashMap<String, Node>);

#[derive(Debug)]
pub(crate) struct MpvNodeMap<'a> {
    _original: &'a NodeMap,

    _guarded_nodes: Vec<MpvNode<'a>>,
    _flat_nodes: Box<[mpv_node]>,

    _owned_keys: Vec<CString>,
    _flat_keys: Box<[*const c_char]>,

    node_list: mpv_node_list,
}

impl MpvRepr for MpvNodeMap<'_> {
    type Repr = mpv_node_list;

    fn ptr(&self) -> *const Self::Repr {
        &self.node_list
    }
}

impl MpvSend for NodeMap {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_NODE_MAP;

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

impl ToMpvRepr for NodeMap {
    type ReprWrap<'a> = MpvNodeMap<'a>;

    fn to_mpv_repr(&self) -> Self::ReprWrap<'_> {
        let mut guarded_nodes = Vec::with_capacity(self.0.len());
        let mut owned_keys = Vec::with_capacity(self.0.len());

        for (key, value) in &self.0 {
            owned_keys.push(CString::new(key.as_bytes()).unwrap_or_default());
            guarded_nodes.push(value.to_mpv_repr());
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

impl NodeMap {
    pub(crate) unsafe fn from_node_list_ptr(ptr: *const mpv_node_list) -> Self {
        assert!(!ptr.is_null());

        let values = unsafe { std::slice::from_raw_parts((*ptr).values, (*ptr).num as usize) }
            .iter().map(|x| unsafe { Node::from_node_ptr(x) });

        let keys = unsafe { std::slice::from_raw_parts((*ptr).keys, (*ptr).num as usize) }
            .iter().map(|x| unsafe { CStr::from_ptr(*x).to_string_lossy().to_string() });

        let map = keys.zip(values).collect();
        Self(map)
    }
}