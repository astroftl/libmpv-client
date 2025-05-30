use std::collections::HashMap;
use std::ffi::{CStr, CString,c_char, c_int, c_void};
use std::ptr::null_mut;
use libmpv_client_sys::{mpv_node, mpv_node_list};
use crate::*;
use crate::node::MpvNode;
use crate::traits::{MpvRepr, MpvSend, ToMpvRepr};

/// Used with mpv_node only. Can usually not be used directly.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeMap(pub HashMap<String, Node>);

#[derive(Debug)]
pub(crate) struct MpvNodeMap<'a> {
    _original: &'a NodeMap,

    _owned_reprs: Vec<Box<MpvNode<'a>>>,
    _flat_reprs: Vec<mpv_node>,

    _owned_keys: Vec<CString>,
    _flat_keys: Vec<*const c_char>,

    node_list: mpv_node_list,
}

impl MpvRepr for MpvNodeMap<'_> {
    type Repr = mpv_node_list;

    fn ptr(&self) -> *const Self::Repr {
        let ptr = &raw const self.node_list;

        // println!("Returning pointer {ptr:p} to node_list: {:#?}", self.node_list);

        ptr
    }
}

impl MpvSend for NodeMap {
    const MPV_FORMAT: Format = Format::NODE_MAP;

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

    fn to_mpv_repr(&self) -> Box<Self::ReprWrap<'_>> {
        let mut repr = Box::new(MpvNodeMap {
            _original: self,
            _owned_reprs: Vec::with_capacity(self.0.len()),
            _flat_reprs: Vec::with_capacity(self.0.len()),
            _owned_keys: Vec::with_capacity(self.0.len()),
            _flat_keys: Vec::with_capacity(self.0.len()),
            node_list: mpv_node_list {
                num: self.0.len() as c_int,
                values: null_mut(),
                keys: null_mut(),
            },
        });

        for (key, value) in &self.0 {
            repr._owned_keys.push(CString::new(key.as_bytes()).unwrap_or_default());
            repr._flat_keys.push(repr._owned_keys.last().unwrap().as_ptr());

            repr._owned_reprs.push(value.to_mpv_repr());
            repr._flat_reprs.push(repr._owned_reprs.last().unwrap().node);
        }

        repr.node_list.values = repr._flat_reprs.as_ptr() as *mut _;
        repr.node_list.keys = repr._flat_keys.as_ptr() as *mut _;

        // println!("created NodeMap repr: {repr:#?}");

        repr
    }
}

impl NodeMap {
    pub(crate) unsafe fn from_node_list_ptr(ptr: *const mpv_node_list) -> Self {
        assert!(!ptr.is_null());

        if ptr.is_null() || unsafe { (*ptr).values.is_null() } || unsafe { (*ptr).keys.is_null() } {
            return Self(HashMap::new())
        }

        let values = unsafe { std::slice::from_raw_parts((*ptr).values, (*ptr).num as usize) }
            .iter().map(|x| unsafe { Node::from_node_ptr(x) });

        let keys = unsafe { std::slice::from_raw_parts((*ptr).keys, (*ptr).num as usize) }
            .iter().map(|x| unsafe { CStr::from_ptr(*x).to_string_lossy().to_string() });

        let map = keys.zip(values).collect();
        Self(map)
    }
}