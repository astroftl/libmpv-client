use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr::null_mut;
use libmpv_client_sys::{free, mpv_format, mpv_format_MPV_FORMAT_DOUBLE, mpv_format_MPV_FORMAT_FLAG, mpv_format_MPV_FORMAT_INT64, mpv_format_MPV_FORMAT_OSD_STRING, mpv_format_MPV_FORMAT_STRING};
use crate::*;
use crate::traits::MpvSend;

#[derive(Debug)]
pub struct OsdString(pub String);

impl MpvSend for String {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_STRING;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        let ptr = ptr as *const *const c_char;
        unsafe { CStr::from_ptr(*ptr) }.to_string_lossy().to_string()
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut cstr: *mut c_char = null_mut();
        fun(&raw mut cstr as *mut c_void).and_then(|_| {
            let ret = unsafe { Self::from_ptr(&raw mut cstr as *const c_void) };
            unsafe { free(cstr as *mut c_void) }
            Ok(ret)
        })
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let cstring = CString::new(self.as_bytes())?;
        let ptr = cstring.as_ptr();
        fun(&raw const ptr as *mut c_void)
    }
}

impl MpvSend for OsdString {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_OSD_STRING;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        OsdString(unsafe { String::from_ptr(ptr) })
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        unsafe { String::from_mpv(fun) }.map(|s| Self(s))
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        self.0.to_mpv(fun)
    }
}

impl MpvSend for bool {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_FLAG;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        unsafe { *(ptr as *const c_int) != 0 }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut flag: c_int = 0;
        fun(&raw mut flag as *mut c_void).map(|_| flag != 0)
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let flag = if *self { 1 } else { 0 };
        fun(&raw const flag as *mut c_void)
    }
}

impl MpvSend for i64 {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_INT64;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        unsafe { *(ptr as *const Self) }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut val: Self = 0;
        fun(&raw mut val as *mut c_void).map(|_| val)
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        fun(self as *const Self as *mut c_void)
    }
}

impl MpvSend for f64 {
    const MPV_FORMAT: mpv_format = mpv_format_MPV_FORMAT_DOUBLE;

    unsafe fn from_ptr(ptr: *const c_void) -> Self {
        unsafe { *(ptr as *const Self) }
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut val: Self = 0.0;
        fun(&raw mut val as *mut c_void).map(|_| val)
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        fun(self as *const Self as *mut c_void)
    }
}