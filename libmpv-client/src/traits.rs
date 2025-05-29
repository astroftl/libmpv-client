use std::ffi::c_void;
use libmpv_client_sys::mpv_format;
use crate::Result;

pub trait MpvSend: Sized {
    const MPV_FORMAT: mpv_format;

    unsafe fn from_ptr(ptr: *const c_void) -> Self;
    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> crate::Result<Self>;
    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> crate::Result<i32>;
}

pub(crate) trait ToMpvRepr: MpvSend {
    type ReprWrap<'a>: MpvRepr where Self: 'a;

    fn to_mpv_repr(&self) -> Self::ReprWrap<'_>;
}

pub(crate) trait MpvRepr: Sized {
    type Repr;

    fn ptr(&self) -> *const Self::Repr;
}