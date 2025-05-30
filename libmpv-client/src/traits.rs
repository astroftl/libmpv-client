use std::ffi::c_void;
use crate::{Format, Result};

pub trait MpvSend: Sized {
    const MPV_FORMAT: Format;

    // TODO: Reevaluate whether functions which are now properly guarded need to be marked themselves unsafe.
    // I think they probably do, since they still rely on the pointer being to a valid data structure, which cannot be checked at runtime.
    // During my documentation overhaul I will clearly document this contract.
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self>;
    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self>;
    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32>;
}

pub(crate) trait ToMpvRepr: MpvSend {
    type ReprWrap<'a>: MpvRepr where Self: 'a;

    // TODO: Make this return a Result<> for better error handling.
    fn to_mpv_repr(&self) -> Box<Self::ReprWrap<'_>>;
}

pub(crate) trait MpvRepr: Sized {
    type Repr;

    fn ptr(&self) -> *const Self::Repr;
}