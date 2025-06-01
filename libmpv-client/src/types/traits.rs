//! Traits that define which types can be sent to and received from mpv.
#![allow(private_bounds)]

use std::ffi::c_void;
use crate::{Format, Result};

/// Defines a type understood by mpv.
pub trait MpvFormat: Sized {
    /// Defines the [`mpv_format`](libmpv_client_sys::mpv_format) used with mpv when transferring and requesting data.
    const MPV_FORMAT: Format;
}

/// Defines a type which may be sent to mpv.
///
/// All types sent to mpv must have a stable C representation. However, since trait implementations may be able to
/// construct these intermediate representations before sending the data to mpv, this trait is a superset of [`MpvRecv`]
/// (i.e., it includes [`&str`] in addition to [`String`]).
pub trait MpvSend: MpvSendInternal {}
pub(crate) trait MpvSendInternal: MpvFormat {
    /// Prepare and send data to mpv.
    ///
    /// Functionally, it prepares the data for sending to mpv (if necessary, by allocating an mpv-friendly data structure
    /// and cloning the Rust structure's data into it) and then calls the provided function pointer `fun`. This function
    /// is provided a `*mut c_void` and is expected to pass it to a libmpv function.
    ///
    /// See [`Handle::set_property()`](crate::Handle::set_property) for example usage.
    fn to_mpv<F: Fn(*mut c_void) -> Result<i32>>(&self, fun: F) -> Result<i32>;
}

/// Defines a type which may be received from mpv.
///
/// Any data received from mpv is read-only. Thus, all types implementing this trait must own their own storage (i.e., cannot be references).
/// Trait implementations are expected to copy any data mpv provides into their own structures as necessary.
/// For complex types (i.e., [`Node`](crate::Node), or which may contain [`NodeArray`](crate::NodeArray)/[`NodeMap`](crate::NodeMap),
/// which themselves contain more nodes), this can be comparatively expensive.
pub trait MpvRecv: MpvRecvInternal {}
pub(crate) trait MpvRecvInternal: MpvFormat {
    // TODO: Reevaluate whether functions which are now properly guarded need to be marked themselves unsafe.
    // I think they probably do, since they still rely on the pointer being to a valid data structure, which cannot be checked at runtime.
    // During my documentation overhaul I will clearly document this contract.
    /// Get a type T from a pointer to its mpv representation.
    ///
    /// Performs any necessary data copying or adjustment to go from a pointer to the mpv-friendly
    /// C representation block into a Rust type.
    ///
    /// In simple cases this may just be a pointer cast, dereference, and copy,
    /// while more complex types require several allocations.
    ///
    /// # Safety
    /// This function assumes that the block pointer to by `ptr` is of the correct format for the type.
    ///
    /// Since any type information is lost to the [`c_void`] pointer, it is critical that only pointers to
    /// properly structured data be passed in. In nearly all cases, these should be provided directly by mpv.
    unsafe fn from_ptr(ptr: *const c_void) -> Result<Self>;

    /// Receive data from mpv and process it into a Rust type.
    ///
    /// Functionally, it allocates a zero'd mpv-friendly data structure for the type `T`.
    /// A `*mut c_void` pointer to this block is provided to the function pointer `fun`,
    /// which is expected to pass it to a libmpv function.
    /// Then the data is processed to turn it from the mpv-friendly C representation into a Rust type.
    ///
    /// See [`Handle::get_property()`](crate::Handle::get_property) for example usage.
    ///
    /// # Safety
    /// This function assumes that the function `fun` will fill the pass pointer only with data that is of the correct format for the type.
    ///
    /// In many implementations, this function calls [`MpvRecv::from_ptr()`] (usually except in cases of primitives) with no additional checks.
    /// In nearly all cases, `fun` should simply provide the pointer to mpv and let it handle writing the data.
    unsafe fn from_mpv<F: Fn(*mut c_void) -> Result<i32>>(fun: F) -> Result<Self>;
}

pub(crate) trait ToMpvRepr: MpvSend {
    type ReprWrap<'a>: MpvRepr where Self: 'a;

    // TODO: Make this return a Result<> for better error handling.
    fn to_mpv_repr(&self) -> Self::ReprWrap<'_>;
}

pub(crate) trait MpvRepr: Sized {
    type Repr;

    fn ptr(&self) -> *const Self::Repr;
}