use std::ffi::c_void;
use libmpv_client_sys::{mpv_byte_array, mpv_format, mpv_format_MPV_FORMAT_BYTE_ARRAY};
use crate::*;
use crate::traits::{MpvRepr, MpvSend, ToMpvRepr};

pub type ByteArray = Vec<u8>;

#[derive(Debug)]
pub(crate) struct MpvByteArray<'a> {
    _original: &'a ByteArray,

    byte_array: mpv_byte_array
}

impl MpvRepr for MpvByteArray<'_> {
    type Repr = mpv_byte_array;

    fn ptr(&self) -> *const mpv_byte_array {
        &raw const self.byte_array
    }
}

impl MpvSend for ByteArray {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_BYTE_ARRAY;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        assert!(!ptr.is_null());

        let ptr = ptr as *const mpv_byte_array;

        unsafe { std::slice::from_raw_parts((*ptr).data as *const u8, (*ptr).size) }.to_vec()
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut ba: mpv_byte_array = unsafe { std::mem::zeroed() };

        fun(&raw mut ba as *mut c_void).map(|_| {
            unsafe { Self::from_ptr(&raw const ba as *const c_void) }
        })
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let repr = self.to_mpv_repr();

        fun(repr.ptr() as *mut c_void)
    }
}

impl ToMpvRepr for ByteArray {
    type ReprWrap<'a> = MpvByteArray<'a>;

    fn to_mpv_repr(&self) -> Box<Self::ReprWrap<'_>> {
        Box::new(MpvByteArray {
            _original: self,
            byte_array: mpv_byte_array {
                data: self.as_ptr() as *mut c_void,
                size: self.len(),
            },
        })
    }
}