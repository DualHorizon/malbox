//! Plugin API version management.
//!
//! This module provides versioned access to the plugin API,
//! ensuring backward compatibility and smooth transitions between versions.

pub mod compatibility;
pub mod v1;

pub use v1::*;

/// API version information and metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl ApiVersion {
    /// Current stable API version.
    pub const V1_0_0: Self = Self {
        major: 1,
        minor: 0,
        patch: 0,
    };

    /// Get the current API version.
    pub fn current() -> Self {
        Self::V1_0_0
    }

    /// Check if this version is compatible with another version.
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Same major version, plugin minor <= core minor
        self.major == other.major && other.minor <= self.minor
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl std::str::FromStr for ApiVersion {
    type Err = semver::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let version = semver::Version::parse(s)?;
        Ok(Self {
            major: version.major as u32,
            minor: version.minor as u32,
            patch: version.patch as u32,
        })
    }
}
