use std::ffi::{CStr, CString, c_char, c_int, c_void};
use std::ptr::null_mut;
use libmpv_client_sys::free;
use crate::*;
use crate::error::RustError;
use crate::traits::MpvSend;

#[derive(Debug)]
pub struct OsdString(pub String);

impl MpvSend for String {
    const MPV_FORMAT: Format = Format::STRING;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        check_null!(ptr);
        let cstr = unsafe { *(ptr as *const *const c_char) };

        check_null!(cstr);
        Ok(unsafe { CStr::from_ptr(cstr) }.to_str()?.to_string())
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut cstr: *mut c_char = null_mut();

        fun(&raw mut cstr as *mut c_void).and_then(|_| {
            let ret = unsafe { Self::from_ptr(&raw mut cstr as *const c_void) };
            unsafe { free(cstr as *mut c_void) }
            Ok(ret)
        })?
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        let cstring = CString::new(self.as_bytes())?;
        let cstr = cstring.as_ptr();

        fun(&raw const cstr as *mut c_void)
    }
}

impl MpvSend for OsdString {
    const MPV_FORMAT: Format = Format::OSD_STRING;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        Ok(OsdString(unsafe { String::from_ptr(ptr)? }))
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        unsafe { String::from_mpv(fun) }.map(|s| Self(s))
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        self.0.to_mpv(fun)
    }
}

impl MpvSend for bool {
    const MPV_FORMAT: Format = Format::FLAG;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        check_null!(ptr);
        Ok(unsafe { *(ptr as *const c_int) != 0 })
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
    const MPV_FORMAT: Format = Format::INT64;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        check_null!(ptr);
        Ok(unsafe { *(ptr as *const Self) })
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
    const MPV_FORMAT: Format = Format::DOUBLE;

    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self> {
        check_null!(ptr);
        Ok(unsafe { *(ptr as *const Self) })
    }

    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self> {
        let mut val: Self = 0.0;
        fun(&raw mut val as *mut c_void).map(|_| val)
    }

    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32> {
        fun(self as *const Self as *mut c_void)
    }
}