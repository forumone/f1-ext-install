//! Type and helpers for PECL extensions.

use lazy_static::lazy_static;
use maplit::btreemap;
use regex::Regex;
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr};

use super::{ParseError, Version};

/// Represents the data for a PECL extension.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct PeclData {
    /// The external package (if any) needed by this extension.
    #[serde(default)]
    packages: Option<Vec<String>>,

    /// Should this extension be disabled by default in the Docker image being built?
    ///
    /// This field exists primarily to support XDebug, which is not enabled by default
    /// due to the performance penalty it imposes.
    #[serde(default)]
    disabled: bool,
}

/// Represents the information needed to install and configure a PECL extension.
#[derive(Clone, Debug)]
pub struct Pecl {
    /// The name of this PECL extension.
    name: String,

    /// The version requested for this installation.
    version: Version,

    /// The data for this extension.
    data: PeclData,
}

impl Pecl {
    /// Returns the name of this extension.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the list of external packages (if any) needed by this extension.
    pub fn packages(&self) -> Option<&Vec<String>> {
        self.data.packages.as_ref()
    }

    /// Determines if this extension should be enabled by default.
    pub fn is_enabled(&self) -> bool {
        !self.data.disabled
    }

    /// Returns the PECL extension specifier for this PECL extension, in the format NAME-VERSION.
    pub fn specifier(&self) -> String {
        format!("{}-{}", self.name, self.version)
    }

    // Allow access to the extension's version for unit testing
    #[cfg(test)]
    pub fn version(&self) -> &Version {
        &self.version
    }
}

lazy_static! {
    static ref REGISTRY: BTreeMap<&'static str, PeclData> = btreemap! {
        "imagick" => PeclData {
            packages: Some(vec![String::from("imagemagick-dev")]),
            ..PeclData::default()
        },

        "memcached" => PeclData {
            packages: Some(vec![
                String::from("libmemcached-dev"),
                String::from("zlib-dev"),
                String::from("libevent-dev"),
            ]),
            ..PeclData::default()
        },

        "xdebug" => PeclData {
            disabled: true,
            ..PeclData::default()
        },
    };
}

/// Finds a PECL extension's data from either the internal registry or the environment.
/// If neither attempt succeeds, returns empty PECL data.
fn find_pecl_data(name: &str) -> PeclData {
    if let Some(found) = REGISTRY.get(name) {
        return found.clone();
    }

    let prefix = format!("F1_PECL_{}", name.to_ascii_uppercase());
    if let Ok(data) = envy::prefixed(prefix).from_env() {
        return data;
    }

    PeclData::default()
}

impl FromStr for Pecl {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref PECL: Regex = Regex::new(
                r#"(?x)
                ^
                (?P<name>[_a-zA-Z0-9]+)
                (?:@(?P<version>stable|\d+\.\d+\.\d+))?
                $
                "#
            )
            .unwrap();
        }

        let caps = match PECL.captures(input) {
            Some(caps) => caps,
            None => return Err(ParseError::InvalidSyntax {}),
        };

        let name = &caps["name"];
        let version = match caps.name("version") {
            Some(cap) => {
                let cap = cap.as_str();
                if cap == "stable" {
                    Version::Stable
                } else {
                    Version::Custom(String::from(cap))
                }
            }
            None => Version::default(),
        };

        Ok(Pecl {
            name: String::from(name),
            version,
            data: find_pecl_data(name),
        })
    }
}

#[cfg(test)]
mod tests {
    use cool_asserts::assert_matches;

    use super::*;

    #[test]
    fn test_basic_parse() {
        let xdebug: Pecl = "xdebug".parse().unwrap();
        assert_eq!(xdebug.name(), "xdebug");
    }

    #[test]
    fn test_name_underscores() {
        let example_foo: Pecl = "example_foo".parse().unwrap();
        assert_eq!(example_foo.name(), "example_foo");
    }

    #[test]
    fn test_stable() {
        let xdebug: Pecl = "xdebug@stable".parse().unwrap();
        assert_eq!(xdebug.name(), "xdebug", "xdebug should have name xdebug");
        assert_matches!(
            xdebug.version(),
            Version::Stable,
            "xdebug@stable should have an explicit stable version",
        );
    }

    #[test]
    fn test_version() {
        let xdebug: Pecl = "xdebug@2.5.5".parse().unwrap();
        assert_eq!(xdebug.name(), "xdebug", "xdebug should have name xdebug");
        assert_matches!(
            xdebug.version(),
            Version::Custom(version) => {
                assert_eq!(version, "2.5.5");
            },
            "xdebug@2.5.5 should have custom version 2.5.5",
        );
    }
}
