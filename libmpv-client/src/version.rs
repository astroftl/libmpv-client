//! Various functions for ensuring that the currently used mpv player and header match the version this crate was intended for.

use crate::*;
use crate::error::{RustError, VersionError};

/// Checks that the mpv/client.h passed to bindgen matches the version of mpv that this crate was written for.
pub fn generated_version_check() -> Result<()> {
    if libmpv_client_sys::MPV_CLIENT_API_VERSION == libmpv_client_sys::EXPECTED_MPV_VERSION {
        Ok(())
    } else {
        Err(Error::Rust(RustError::VersionMismatch(VersionError {
            expected: libmpv_client_sys::EXPECTED_MPV_VERSION as u64,
            found: libmpv_client_sys::MPV_CLIENT_API_VERSION as u64,
        })))
    }
}

/// Checks that the MPV_CLIENT_API_VERSION of the client matches the version of mpv this crate was built for.
pub fn version_check() -> Result<()> {
    let version = Handle::client_api_version();
    if version == libmpv_client_sys::EXPECTED_MPV_VERSION as u64 {
        Ok(())
    } else {
        Err(Error::Rust(RustError::VersionMismatch(VersionError {
            expected: libmpv_client_sys::EXPECTED_MPV_VERSION as u64,
            found: version,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_version_check_test() -> Result<()> {
        generated_version_check()
    }
}