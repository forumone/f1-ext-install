//! Type and helpers for PHP builtin extensions.

use lazy_static::lazy_static;
use maplit::btreemap;
use nom::{combinator::map, error::ParseError, IResult};
use std::{collections::BTreeMap, str::FromStr};

use super::{parse, StringList};

/// Represents the information needed for a PHP builtin extension.
#[derive(Clone, Debug)]
pub struct Builtin {
    /// The name of this extension, as used by the `docker-php-ext-install` utility.
    name: String,
    /// The list of external packages (if any) this extension needs.
    packages: Option<StringList<'static>>,
    /// Represents the arguments to pass to `docker-php-ext-configure`, if that utility
    /// needs to be called.
    configure_cmd: Option<StringList<'static>>,
}

impl Builtin {
    /// Creates a new builtin by assuming that no external packages or configuration is needed.
    pub fn new(name: String) -> Self {
        Self {
            name,
            packages: None,
            configure_cmd: None,
        }
    }

    /// Adds external package dependencies to this builtin.
    pub fn with_packages(mut self, packages: StringList<'static>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Adds `docker-php-ext-configure` args to this builtin.
    pub fn with_configure_cmd(mut self, configure_cmd: StringList<'static>) -> Self {
        self.configure_cmd = Some(configure_cmd);
        self
    }

    /// Returns the builtin name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the list of external packages (if any) needed by this builtin.
    pub fn packages(&self) -> &Option<StringList<'_>> {
        &self.packages
    }

    /// Returns the configure command (if any) needed by this builtin.
    pub fn configure_cmd(&self) -> &Option<StringList<'_>> {
        &self.configure_cmd
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
        let parser = map(parse::name, |name: &str| {
            REGISTRY
                .get(name)
                .cloned()
                .unwrap_or_else(|| Builtin::new(name.to_owned()))
        });

        parser(input)
    }
}

lazy_static! {
    static ref REGISTRY: BTreeMap<&'static str, Builtin> = btreemap! {
        "gd" => Builtin::new("gd".to_owned())
            .with_packages(&["coreutils", "freetype-dev", "libjpeg-turbo-dev"])
            .with_configure_cmd(&[
                "--with-freetype-dir=/usr/include/",
                "--with-jpeg-dir=/usr/include/",
                "--with-png-dir=/usr/include/",
            ]),
        "soap" => Builtin::new("soap".to_owned()).with_packages(&["libxml2-dev"]),
        "zip" => Builtin::new("zip".to_owned()).with_packages(&["libzip-dev"]),
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
