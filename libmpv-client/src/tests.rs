#![cfg(test)]

use std::cell::Cell;
use std::ffi::c_void;
use libmpv_client_sys::mpv_node;

thread_local! {
    pub(crate) static MPV_FREE_CALLS: Cell<usize> = Cell::new(0);
    pub(crate) static MPV_FREE_NODE_CONTENTS_CALLS: Cell<usize> = Cell::new(0);
}

pub(crate) fn mpv_free_stub(_data: *mut c_void) {
    MPV_FREE_CALLS.set(MPV_FREE_CALLS.get() + 1);
}

pub(crate) fn mpv_free_node_contents_stub(_node: *mut mpv_node) {
    MPV_FREE_NODE_CONTENTS_CALLS.set(MPV_FREE_NODE_CONTENTS_CALLS.get() + 1);
}
