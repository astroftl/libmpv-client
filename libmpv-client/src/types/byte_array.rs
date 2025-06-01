use std::ffi::c_void;
use libmpv_client_sys::mpv_byte_array;
use crate::*;
use crate::types::traits::{MpvFormat, MpvRecv, MpvRecvInternal, MpvRepr, MpvSend, MpvSendInternal, ToMpvRepr};

/// A [`Vec<u8>`] representing a raw, untyped byte array. Only used with [`Node`], and only in some very specific situations. (Some commands use it.)
pub type ByteArray = Vec<u8>;

#[derive(Debug)]
pub(crate) struct MpvByteArray<'a> {
    _original: &'a ByteArray,

    byte_array: Box<mpv_byte_array>
}

impl MpvRepr for MpvByteArray<'_> {
    type Repr = mpv_byte_array;

    fn ptr(&self) -> *const Self::Repr {
        &raw const *self.byte_array
    }
}

impl MpvFormat for ByteArray {
    const MPV_FORMAT: Format = Format::BYTE_ARRAY;
}

impl From<ByteArray> for Node {
    fn from(value: ByteArray) -> Self {
        Node::ByteArray(value)
    }
}

impl From<&ByteArray> for Node {
    fn from(value: &ByteArray) -> Self {
        Node::ByteArray(value.clone())
    }
}

impl MpvRecv for ByteArray {}
impl MpvRecvInternal for ByteArray {
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        check_null!(ptr);
        let byte_array = unsafe { *(ptr as *const mpv_byte_array) };

        check_null!(byte_array.data);
        Ok(unsafe { std::slice::from_raw_parts(byte_array.data as *const u8, byte_array.size) }.to_vec())
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut ba: mpv_byte_array = unsafe { std::mem::zeroed() };

        fun(&raw mut ba as *mut c_void).map(|_| {
            unsafe { Self::from_ptr(&raw const ba as *const c_void) }
        })?
    }
}

impl MpvSend for ByteArray {}
impl MpvSendInternal for ByteArray {
    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let repr = self.to_mpv_repr();

        fun(repr.ptr() as *mut c_void)
    }
}

impl ToMpvRepr for ByteArray {
    type ReprWrap<'a> = MpvByteArray<'a>;

    fn to_mpv_repr(&self) -> Self::ReprWrap<'_> {
        MpvByteArray {
            _original: self,
            byte_array: Box::new(mpv_byte_array {
                data: self.as_ptr() as *mut c_void,
                size: self.len(),
            }),
        }
    }
}