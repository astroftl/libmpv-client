use std::ffi::c_void;
use crate::{Format, Result};

pub trait MpvSend: Sized {
    const MPV_FORMAT: Format;

    unsafe fn from_ptr(ptr: *const c_void) -> Self;
    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self>;
    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32>;
}

pub(crate) trait ToMpvRepr: MpvSend {
    type ReprWrap<'a>: MpvRepr where Self: 'a;

    fn to_mpv_repr(&self) -> Box<Self::ReprWrap<'_>>;
}

pub(crate) trait MpvRepr: Sized {
    type Repr;

    fn ptr(&self) -> *const Self::Repr;
}