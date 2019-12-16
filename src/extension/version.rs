//! Type and helpers for PECL version specifiers.

use std::fmt;

/// Represents a PECL version.
#[derive(Clone, Debug)]
pub enum Version {
    /// The `stable` version/channel.
    Stable,
    /// A specific version (in MAJOR.MINOR.PATCH format).
    Custom(String),
}

impl Default for Version {
    fn default() -> Self {
        Self::Stable
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Custom(version) => write!(f, "{}", version),
        }
    }
}
