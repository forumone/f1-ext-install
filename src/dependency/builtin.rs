//! Type and helpers for PHP builtin extensions.

use lazy_static::lazy_static;
use maplit::btreemap;
use nom::{combinator::map, error::ParseError, IResult};
use serde::Deserialize;
use std::{collections::BTreeMap, str::FromStr};

use super::parse;

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
    /// Creates a new builtin by assuming that no external packages or configuration is needed.
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            data: BuiltinData::default(),
        }
    }

    /// Replaces this extension's data with the given data.
    fn with_data(mut self, data: BuiltinData) -> Self {
        self.data = data;
        self
    }

    /// Adds external package dependencies to this builtin.
    pub fn with_packages(mut self, packages: Vec<String>) -> Self {
        self.data.packages = Some(packages);
        self
    }

    /// Adds `docker-php-ext-configure` args to this builtin.
    pub fn with_configure_cmd(mut self, configure_cmd: Vec<String>) -> Self {
        self.data.configure_cmd = Some(configure_cmd);
        self
    }

    /// Returns the builtin name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the list of external packages (if any) needed by this builtin.
    pub fn packages(&self) -> Option<&[String]> {
        self.data.packages.as_ref().map(AsRef::as_ref)
    }

    /// Returns the configure command (if any) needed by this builtin.
    pub fn configure_cmd(&self) -> Option<&[String]> {
        self.data.configure_cmd.as_ref().map(AsRef::as_ref)
    }

    /// Attempts to parse the name of a builtin from the input.
    ///
    /// The syntax of a builtin is a simple identifier (e.g., `gd`, `pdo_mysql`, and so on).
    ///
    /// This method is exposed publicly for easier composition with other parsers. For
    /// example, the `parse` method of [`Dependency`] uses this method.
    ///
    /// [`Dependency`]: crate::dependency::Dependency
    pub fn parse<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let parser = map(parse::name, find_builtin);

        parser(input)
    }
}

/// Finds (or creates) a builtin by name.
fn find_builtin(name: &str) -> Builtin {
    if let Some(found) = REGISTRY.get(name) {
        return found.clone();
    }

    let prefix = format!("F1_BUILTIN_{}_", name.to_ascii_uppercase());

    if let Ok(data) = envy::prefixed(prefix).from_env() {
        return Builtin::new(name).with_data(data);
    }

    Builtin::new(name)
}

lazy_static! {
    static ref REGISTRY: BTreeMap<&'static str, Builtin> = btreemap! {
        "gd" => Builtin::new("gd")
            .with_packages(vec![
                "coreutils".into(),
                "freetype-dev".into(),
                "libjpeg-turbo-dev".into()
            ])
            .with_configure_cmd(vec![
                "--with-freetype-dir=/usr/include/".into(),
                "--with-jpeg-dir=/usr/include/".into(),
                "--with-png-dir=/usr/include/".into(),
            ]),
        "soap" => Builtin::new("soap").with_packages(vec!["libxml2-dev".into()]),
        "zip" => Builtin::new("zip").with_packages(vec!["libzip-dev".into()]),
    };
}

impl FromStr for Builtin {
    type Err = super::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse::parse_all(input, Self::parse)
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
