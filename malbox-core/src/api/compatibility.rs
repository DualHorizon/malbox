//! API compatibility utilities.

use super::ApiVersion;
use semver::{Version, VersionReq};

/// Check if a plugin API version is compatible with the current core version
pub fn is_plugin_compatible(plugin_version: &str) -> bool {
    let Ok(plugin_ver) = Version::parse(plugin_version) else {
        return false;
    };

    let current = ApiVersion::current();
    let current_semver = Version::new(
        current.major as u64,
        current.minor as u64,
        current.patch as u64,
    );

    // Compatible if same major version and plugin version <= core version
    plugin_ver.major == current_semver.major && plugin_ver <= current_semver
}

/// Get the best compatible API version for a plugin request
pub fn negotiate_api_version(requested: &str) -> Option<String> {
    let Ok(requested_ver) = Version::parse(requested) else {
        return None;
    };

    let current = ApiVersion::current();
    let current_semver = Version::new(
        current.major as u64,
        current.minor as u64,
        current.patch as u64,
    );

    // Return current version if compatible
    if requested_ver.major == current_semver.major && requested_ver <= current_semver {
        Some(current.to_string())
    } else {
        None
    }
}

/// Get all supported API versions
pub fn supported_versions() -> Vec<String> {
    vec![ApiVersion::current().to_string()]
}

/// Create a version requirement for the current API
pub fn current_version_req() -> VersionReq {
    let current = ApiVersion::current();
    VersionReq::parse(&format!(
        "^{}.{}.{}",
        current.major, current.minor, current.patch
    ))
    .expect("Current version should be valid")
}
