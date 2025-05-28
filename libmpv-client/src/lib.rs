pub mod event;
pub mod error;
pub mod format;
pub mod node;
pub mod handle;

use libmpv_client_sys as mpv;
pub use mpv::mpv_handle;

#[derive(Debug, Clone)]
pub struct VersionError {
    pub expected: u64,
    pub found: u64,
}

/// Checks that the mpv/client.h passed to bindgen matches the version of mpv that this crate was written for.
pub fn generated_version_check() -> Result<(), VersionError> {
    if mpv::MPV_CLIENT_API_VERSION == mpv::EXPECTED_MPV_VERSION {
        Ok(())
    } else {
        Err(VersionError { expected: mpv::EXPECTED_MPV_VERSION as u64, found: mpv::MPV_CLIENT_API_VERSION as u64 })
    }
}

/// Return the MPV_CLIENT_API_VERSION the mpv source has been compiled with.
pub fn client_api_version() -> u64 {
    unsafe { mpv::client_api_version() as u64 }
}

/// Checks that the MPV_CLIENT_API_VERSION of the client matches the version of mpv this crate was built for.
pub fn version_check() -> Result<(), VersionError> {
    let version = client_api_version();
    if version == mpv::EXPECTED_MPV_VERSION as u64 {
        Ok(())
    } else {
        Err(VersionError {
            expected: mpv::EXPECTED_MPV_VERSION as u64,
            found: version,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_version_check_test() -> Result<(), VersionError> {
        generated_version_check()
    }
}