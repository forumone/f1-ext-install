//! Helper types to handle PHP dependencies.
//!
//! A dependency is broken down into two categories: builtins and PECL. The structs in
//! this module exist to capture the information needed to configure and install them.

use snafu::Snafu;
use std::str::FromStr;

mod builtin;
mod pecl;
mod version;

pub use builtin::Builtin;
pub use pecl::Pecl;
pub use version::Version;

/// Prefix indicating a builtin extension
const BUILTIN_TAG: &str = "builtin:";

/// Length of the "builtin:" prefix
const BUILTIN_LEN: usize = BUILTIN_TAG.len();

/// Prefix indicating a PECL extension
const PECL_TAG: &str = "pecl:";

/// Length of the "pecl:" prefix
const PECL_LEN: usize = PECL_TAG.len();

/// Errors returned during parsing
#[derive(Debug, Snafu)]
pub enum ParseError {
    /// A prefix mismatch was encountered.
    ///
    /// We expect either `"builtin:"` or `"pecl:"` in order to identify which installation method is to be used.
    #[snafu(display(
        r#"An extension name needs to begin with a prefix of either "{}" or "{}""#,
        BUILTIN_TAG,
        PECL_TAG
    ))]
    ExpectedPrefix,

    /// The name is invalid.
    ///
    /// Extension names should be valid identifiers (matching the expression `/^[_a-zA-Z][_a-zA-Z0-9]*/$`)
    #[snafu(display(
        "An extension name needs to be a valid name (e.g., memcached, pdo_mysql, gd)"
    ))]
    InvalidSyntax,
}

/// Encapsulates an extension needed by the Docker image currently being built.
#[derive(Clone, Debug)]
pub enum Extension {
    /// This extension is a PHP builtin (e.g., `gd`, `opcache).
    Builtin(Builtin),

    /// This extension is a PECL extension (e.g., `memcached`, XDebug).
    Pecl(Pecl),
}

impl Extension {
    /// Retrieves the list of packages (if any) needed by this extension. A package is
    /// represented by its name as intepreted by the `apk` package manager.
    pub fn packages(&self) -> Option<&Vec<String>> {
        match self {
            Self::Builtin(builtin) => builtin.packages(),
            Self::Pecl(pecl) => pecl.packages(),
        }
    }

    /// Determines if this extension needs any external packages.
    pub fn has_packages(&self) -> bool {
        match self.packages() {
            None => false,
            Some(packages) => !packages.is_empty(),
        }
    }
}

impl FromStr for Extension {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        if input.starts_with(BUILTIN_TAG) {
            let input = &input[BUILTIN_LEN..];
            let builtin = input.parse()?;
            Ok(Self::Builtin(builtin))
        } else if input.starts_with(PECL_TAG) {
            let input = &input[PECL_LEN..];
            let pecl = input.parse()?;
            Ok(Self::Pecl(pecl))
        } else {
            Err(ParseError::ExpectedPrefix)
        }
    }
}

#[cfg(test)]
mod tests {
    use cool_asserts::assert_matches;

    use super::*;

    #[test]
    fn test_parse_builtin() {
        let gd: Extension = "builtin:gd".parse().unwrap();
        assert_matches!(
            gd,
            Extension::Builtin(gd) => {
                assert_eq!(gd.name(), "gd", "builtin:gd should have name gd");
            },
            "builtin:gd should be a builtin extension",
        );
    }

    #[test]
    fn test_parse_pecl() {
        let xdebug: Extension = "pecl:xdebug".parse().unwrap();
        assert_matches!(
            xdebug,
            Extension::Pecl(xdebug) => {
                assert_eq!(xdebug.name(), "xdebug", "pecl:xdebug should have name xdebug");
                assert_matches!(
                    xdebug.version(),
                    Version::Stable,
                    "pecl:xdebug should have version stable",
                );
            },
            "pecl:xdebug should be a PECL extension"
        );
    }

    #[test]
    fn test_parse_pecl_explicit_stable() {
        let xdebug: Extension = "pecl:xdebug@stable".parse().unwrap();
        assert_matches!(
            xdebug,
            Extension::Pecl(xdebug) => {
                assert_eq!(xdebug.name(), "xdebug", "pecl:xdebug@stable should have name xdebug");
                assert_matches!(
                    xdebug.version(),
                    Version::Stable,
                    "pecl:xdebug@stable should have version stable",
                );
            },
            "pecl:xdebug@stable should be a PECL extension"
        );
    }

    #[test]
    fn test_parse_pecl_explicit_version() {
        let xdebug: Extension = "pecl:xdebug@2.5.5".parse().unwrap();
        assert_matches!(
            xdebug,
            Extension::Pecl(xdebug) => {
                assert_eq!(xdebug.name(), "xdebug", "pecl:xdebug@2.5.5 should have name xdebug");
                assert_matches!(
                    xdebug.version(),
                    Version::Custom(version) => {
                        assert_eq!(version, "2.5.5", "pecl:xdebug@2.5.5 should have version 2.5.5");
                    },
                    "pecl:xdebug@2.5.5 should have a custom version"
                );
            },
            "pecl@xdebug:2.5.5 should be a PECL extension"
        );
    }

    #[test]
    #[should_panic]
    fn test_parse_pecl_garbage_version() {
        let _: Extension = "pecl:xdebug@askjdfh".parse().unwrap();
    }
}
