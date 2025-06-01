//! The various errors that can be raised by this crate's functions.
//!
//! The primary return type of this crate is [`Result<T>`], which may return an [`Error`].

use std::ffi::{NulError, c_int};
use std::str::Utf8Error;

/// [`std::result::Result`] wrapper around [`Error`] for mpv functions.
///
/// Many mpv API functions returning error codes can also return positive values, which also indicate success.
/// Where relevant, these are exposed as [`Result<i32>`].
pub type Result<T> = std::result::Result<T, Error>;

/// Interpret an error code from an mpv API function into a `Result`, discarding the success code.
pub(crate) fn error_to_result_code(value: c_int) -> Result<i32> {
    if value >= 0 {
        Ok(value)
    } else {
        Err(Error::from(value))
    }
}

/// Interpret an error code from an mpv API function into a `Result`, retaining the success code.
pub(crate) fn error_to_result(value: c_int) -> Result<()> {
    if value >= 0 {
        Ok(())
    } else {
        Err(Error::from(value))
    }
}

/// A structure representing a mismatch between an expected and actual version number.
#[derive(Debug, Clone)]
pub struct VersionError {
    /// The version number that was expected.
    pub expected: u64,
    /// The actual version number that was found.
    pub found: u64,
}

/// A debug struct for marking specific locations in code.
#[derive(Debug)]
pub struct DebugLoc {
    /// The referenced file.
    pub file: &'static str,
    /// The referenced line.
    pub line: u32,
    /// Name of the referenced function.
    pub function: &'static str,
    /// Name of the referenced variable.
    pub variable: Option<&'static str>,
}

/// Enum containing the possible errors in the Rust wrapper over mpv, which may be caused by data passed from mpv or by the user.
#[derive(Debug)]
pub enum RustError {
    /// Invalid UTF-8 data was encountered while parsing a C string into a Rust string.
    InvalidUtf8(Utf8Error),
    /// An unexpected NULL byte was found while parsing a Rust string into a C string.
    InteriorNull(NulError),
    /// The version of the mpv header or client does not match the version this crate was built for.
    VersionMismatch(VersionError),
    /// mpv provided us with a null or otherwise malformed pointer.
    ///
    /// This can happen occasionally, especially during init, and is not fatal.
    Pointer(Option<DebugLoc>),
}

/// List of error codes than can be returned by API functions.
#[derive(Debug)]
pub enum Error {
    /// No error happened (used to signal successful operation).
    ///
    /// Keep in mind that many API functions returning error codes can also return positive values, which also indicate success.
    Success(i32),
    /// The event ringbuffer is full. This means the client is choked, and can't receive any events. This can happen when too many asynchronous requests have been made, but not answered. Probably never happens in practice, unless the mpv core is frozen for some reason, and the client keeps making asynchronous requests. (Bugs in the client API implementation could also trigger this, e.g. if events become "lost".)
    EventQueueFull,
    /// Memory allocation failed.
    NoMemory,
    /// The mpv core wasn't configured and initialized yet.
    Uninitialized,
    /// Generic catch-all error if a parameter is set to an invalid or unsupported value. This is used if there is no better error code.
    InvalidParameter,
    /// Trying to set an option that doesn't exist.
    OptionNotFound,
    /// Trying to set an option using an unsupported MPV_FORMAT.
    OptionFormat,
    /// Setting the option failed. Typically this happens if the provided option value could not be parsed.
    OptionError,
    /// The accessed property doesn't exist.
    PropertyNotFound,
    /// Trying to set or get a property using an unsupported MPV_FORMAT.
    PropertyFormat,
    /// The property exists, but is not available. This usually happens when the associated subsystem is not active, e.g. querying audio parameters while audio is disabled.
    PropertyUnavailable,
    /// Error setting or getting a property.
    PropertyError,
    /// General error when running a command with mpv_command and similar.
    Command,
    /// Generic error on loading (usually used with mpv_event_end_file.error).
    LoadingFailed,
    /// Initializing the audio output failed.
    AoInitFailed,
    /// Initializing the video output failed.
    VoInitFailed,
    /// There was no audio or video data to play. This also happens if the file was recognized, but did not contain any audio or video streams, or no streams were selected.
    NothingToPlay,
    /// When trying to load the file, the file format could not be determined, or the file was too broken to open it.
    UnknownFormat,
    /// Generic error for signaling that certain system requirements are not fulfilled.
    Unsupported,
    /// The API function which was called is a stub only.
    NotImplemented,
    /// Unspecified error.
    Generic,
    /// Rust implementation specific error.
    Rust(RustError),
}

impl From<Utf8Error> for Error {
    fn from(value: Utf8Error) -> Self {
        Self::Rust(RustError::InvalidUtf8(value))
    }
}

impl From<NulError> for Error {
    fn from(value: NulError) -> Self {
        Self::Rust(RustError::InteriorNull(value))
    }
}

impl From<c_int> for Error {
    fn from(value: c_int) -> Self {
        match value {
            value @ 0.. => Error::Success(value),
            libmpv_client_sys::mpv_error_MPV_ERROR_EVENT_QUEUE_FULL => Error::EventQueueFull,
            libmpv_client_sys::mpv_error_MPV_ERROR_NOMEM => Error::NoMemory,
            libmpv_client_sys::mpv_error_MPV_ERROR_UNINITIALIZED => Error::Uninitialized,
            libmpv_client_sys::mpv_error_MPV_ERROR_INVALID_PARAMETER => Error::InvalidParameter,
            libmpv_client_sys::mpv_error_MPV_ERROR_OPTION_NOT_FOUND => Error::OptionNotFound,
            libmpv_client_sys::mpv_error_MPV_ERROR_OPTION_FORMAT => Error::OptionFormat,
            libmpv_client_sys::mpv_error_MPV_ERROR_OPTION_ERROR => Error::OptionError,
            libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_NOT_FOUND => Error::PropertyNotFound,
            libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_FORMAT => Error::PropertyFormat,
            libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_UNAVAILABLE => Error::PropertyUnavailable,
            libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_ERROR => Error::PropertyError,
            libmpv_client_sys::mpv_error_MPV_ERROR_COMMAND => Error::Command,
            libmpv_client_sys::mpv_error_MPV_ERROR_LOADING_FAILED => Error::LoadingFailed,
            libmpv_client_sys::mpv_error_MPV_ERROR_AO_INIT_FAILED => Error::AoInitFailed,
            libmpv_client_sys::mpv_error_MPV_ERROR_VO_INIT_FAILED => Error::VoInitFailed,
            libmpv_client_sys::mpv_error_MPV_ERROR_NOTHING_TO_PLAY => Error::NothingToPlay,
            libmpv_client_sys::mpv_error_MPV_ERROR_UNKNOWN_FORMAT => Error::UnknownFormat,
            libmpv_client_sys::mpv_error_MPV_ERROR_UNSUPPORTED => Error::Unsupported,
            libmpv_client_sys::mpv_error_MPV_ERROR_NOT_IMPLEMENTED => Error::NotImplemented,
            libmpv_client_sys::mpv_error_MPV_ERROR_GENERIC => Error::Generic,
            _ => unimplemented!(),
        }
    }
}

impl From<&Error> for c_int {
    fn from(value: &Error) -> Self {
        match value {
            Error::EventQueueFull => libmpv_client_sys::mpv_error_MPV_ERROR_EVENT_QUEUE_FULL,
            Error::NoMemory => libmpv_client_sys::mpv_error_MPV_ERROR_NOMEM,
            Error::Uninitialized => libmpv_client_sys::mpv_error_MPV_ERROR_UNINITIALIZED,
            Error::InvalidParameter => libmpv_client_sys::mpv_error_MPV_ERROR_INVALID_PARAMETER,
            Error::OptionNotFound => libmpv_client_sys::mpv_error_MPV_ERROR_OPTION_NOT_FOUND,
            Error::OptionFormat => libmpv_client_sys::mpv_error_MPV_ERROR_OPTION_FORMAT,
            Error::OptionError => libmpv_client_sys::mpv_error_MPV_ERROR_OPTION_ERROR,
            Error::PropertyNotFound => libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_NOT_FOUND,
            Error::PropertyFormat => libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_FORMAT,
            Error::PropertyUnavailable => libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_UNAVAILABLE,
            Error::PropertyError => libmpv_client_sys::mpv_error_MPV_ERROR_PROPERTY_ERROR,
            Error::Command => libmpv_client_sys::mpv_error_MPV_ERROR_COMMAND,
            Error::LoadingFailed => libmpv_client_sys::mpv_error_MPV_ERROR_LOADING_FAILED,
            Error::AoInitFailed => libmpv_client_sys::mpv_error_MPV_ERROR_AO_INIT_FAILED,
            Error::VoInitFailed => libmpv_client_sys::mpv_error_MPV_ERROR_VO_INIT_FAILED,
            Error::NothingToPlay => libmpv_client_sys::mpv_error_MPV_ERROR_NOTHING_TO_PLAY,
            Error::UnknownFormat => libmpv_client_sys::mpv_error_MPV_ERROR_UNKNOWN_FORMAT,
            Error::Unsupported => libmpv_client_sys::mpv_error_MPV_ERROR_UNSUPPORTED,
            Error::NotImplemented => libmpv_client_sys::mpv_error_MPV_ERROR_NOT_IMPLEMENTED,
            Error::Generic => libmpv_client_sys::mpv_error_MPV_ERROR_GENERIC,
            Error::Rust(_) => libmpv_client_sys::mpv_error_MPV_ERROR_GENERIC,
            Error::Success(x) => *x as c_int,
        }
    }
}