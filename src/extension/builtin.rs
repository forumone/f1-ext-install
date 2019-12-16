//! Type and helpers for PHP builtin extensions.

use lazy_static::lazy_static;
use maplit::btreemap;
use regex::Regex;
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr};

use super::ParseError;

/// Represents the data for a PHP builtin extension.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct BuiltinData {
    /// The list of external packages (if any) this extension needs.
    #[serde(default)]
    packages: Option<Vec<String>>,
    /// Represents the arguments to pass to `docker-php-ext-configure`, if that utility
    /// needs to be called.
    #[serde(default)]
    configure_cmd: Option<Vec<String>>,
}

/// Represents the information needed for a PHP builtin extension.
#[derive(Clone, Debug)]
pub struct Builtin {
    /// The name of this extension, as used by the `docker-php-ext-install` utility.
    name: String,

    /// The data for this builtin.
    data: BuiltinData,
}

impl Builtin {
    /// Returns the builtin name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the list of external packages (if any) needed by this builtin.
    pub fn packages(&self) -> Option<&Vec<String>> {
        self.data.packages.as_ref()
    }

    /// Returns the configure command (if any) needed by this builtin.
    pub fn configure_cmd(&self) -> Option<&Vec<String>> {
        self.data.configure_cmd.as_ref()
    }
}

lazy_static! {
    static ref REGISTRY: BTreeMap<&'static str, BuiltinData> = btreemap! {
        "gd" => BuiltinData {
            packages: Some(vec![
                String::from("coreutils"),
                String::from("freetype-dev"),
                String::from("libjpeg-turbo-dev"),
            ]),
            configure_cmd: Some(vec![
                String::from("--with-freetype-dir=/usr/include/"),
                String::from("--with-jpeg-dir=/usr/include/"),
                String::from("--with-png-dir=/usr/include/"),
            ]),
        },

        "soap" => BuiltinData {
            packages: Some(vec![String::from("libxml2-dev")]),
            ..BuiltinData::default()
        },

        "zip" => BuiltinData {
            packages: Some(vec![String::from("libzip-dev")]),
            ..BuiltinData::default()
        },
    };
}

/// Finds a builtin extensoin's data from either the internal registry or the environment.
/// If neither attempt succeeds, returns empty builtin data.
fn find_builtin_data(name: &str) -> BuiltinData {
    if let Some(found) = REGISTRY.get(name) {
        return found.clone();
    }

    let prefix = format!("F1_BUILTIN_{}_", name.to_ascii_uppercase());

    if let Ok(data) = envy::prefixed(prefix).from_env() {
        return data;
    };

    BuiltinData::default()
}

impl FromStr for Builtin {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        lazy_static! {
            static ref BUILTIN: Regex = Regex::new(r"^[_a-zA-Z0-9]+$").unwrap();
        }

        if !BUILTIN.is_match(input) {
            return Err(ParseError::InvalidSyntax);
        }

        Ok(Builtin {
            name: String::from(input),
            data: find_builtin_data(input),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ok() {
        let parsed: Builtin = "foo".parse().unwrap();
        assert_eq!(parsed.name, "foo");
    }

    #[test]
    #[should_panic]
    fn test_parse_fail() {
        let _: Builtin = "  whoops  ".parse().unwrap();
    }

    #[test]
    fn test_name_underscores() {
        let pdo_mysql: Builtin = "pdo_mysql".parse().unwrap();
        assert_eq!(pdo_mysql.name, "pdo_mysql");
    }
}
