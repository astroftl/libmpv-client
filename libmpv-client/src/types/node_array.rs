use std::ffi::{c_int, c_void};
use std::ptr::null_mut;
use libmpv_client_sys::{mpv_node, mpv_node_list};
use crate::*;
use crate::node::MpvNode;
use crate::traits::{MpvRepr, MpvSend, ToMpvRepr};

/// Used with mpv_node only. Can usually not be used directly.
#[derive(Debug, Clone, PartialEq)]
pub struct NodeArray(pub Vec<Node>);

#[derive(Debug)]
pub(crate) struct MpvNodeArray<'a> {
    _original: &'a NodeArray,

    _owned_reprs: Vec<Box<MpvNode<'a>>>,
    _flat_reprs: Vec<mpv_node>,

    node_list: mpv_node_list,
}

impl MpvRepr for MpvNodeArray<'_> {
    type Repr = mpv_node_list;

    fn ptr(&self) -> *const Self::Repr {
        &raw const self.node_list
    }
}

impl MpvSend for NodeArray {
    const MPV_FORMAT: Format = Format::NODE_ARRAY;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        check_null!(ptr);
        let node_list = unsafe { *(ptr as *const mpv_node_list) };

        check_null!(node_list.values);
        let mut values = Vec::with_capacity(node_list.num as usize);

        let node_values = unsafe { std::slice::from_raw_parts(node_list.values, node_list.num as usize) };
        for node_value in node_values {
            values.push(unsafe { Node::from_node_ptr(node_value)? });
        }

        Ok(Self(values))
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut node_list: mpv_node_list = unsafe { std::mem::zeroed() };

        fun(&raw mut node_list as *mut c_void).map(|_| {
            unsafe { Self::from_ptr(&raw const node_list as *const c_void) }
        })?
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let repr = self.to_mpv_repr();

        fun(repr.ptr() as *mut c_void)
    }
}

impl ToMpvRepr for NodeArray {
    type ReprWrap<'a> = MpvNodeArray<'a>;

    fn to_mpv_repr(&self) -> Box<Self::ReprWrap<'_>> {
        let mut repr = Box::new(MpvNodeArray {
            _original: self,
            _owned_reprs: Vec::with_capacity(self.0.len()),
            _flat_reprs: Vec::with_capacity(self.0.len()),
            node_list: mpv_node_list {
                num: self.0.len() as c_int,
                values: null_mut(),
                keys: null_mut(),
            },
        });

        for node in &self.0 {
            repr._owned_reprs.push(node.to_mpv_repr());
            repr._flat_reprs.push(repr._owned_reprs.last().unwrap().node); // SAFETY: We just inserted.
        }

        repr.node_list.values = repr._flat_reprs.as_ptr() as *mut _;

        repr
    }
}