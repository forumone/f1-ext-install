//! Type and helpers for PECL extensions.

use lazy_static::lazy_static;
use maplit::btreemap;
use nom::{
    character::complete::char,
    combinator::{map, opt},
    error::ParseError,
    sequence::{preceded, tuple},
    IResult,
};
use std::{collections::BTreeMap, fmt, str::FromStr};

use super::{parse, StringList, Version};

/// Represents the information needed to install and configure a PECL extension.
#[derive(Clone, Debug)]
pub struct Pecl {
    /// The name of this PECL extension.
    name: String,

    /// The external package (if any) needed by this extension.
    packages: Option<StringList<'static>>,

    /// Should this extension be enabled by default in the Docker image being built?
    ///
    /// This field exists primarily to support XDebug, which is not enabled by default
    /// due to the performance penalty it imposes.
    default_enabled: bool,

    /// The version requested for this installation.
    version: Version,
}

impl Pecl {
    /// Creates a new PECL extension with no external dependencies, enabled by default,
    /// and using the stable channel.
    pub fn new(name: String) -> Self {
        Self {
            name,
            packages: None,
            default_enabled: true,
            version: Version::default(),
        }
    }

    /// Adds external packages to this PECL extension.
    pub fn with_packages(mut self, packages: StringList<'static>) -> Self {
        self.packages = Some(packages);
        self
    }

    /// Marks this PECL package as disabled by default.
    pub fn disabled(mut self) -> Self {
        self.default_enabled = false;
        self
    }


    /// Requests the specified version for installation.
    pub fn with_version(mut self, version: Version) -> Self {
        self.version = version;
        self
    }

    /// Returns the name of this extension.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the list of external packages (if any) needed by this extension.
    pub fn packages(&self) -> &Option<StringList<'_>> {
        &self.packages
    }

    /// Determines if this extension should be enabled by default.
    pub fn default_enabled(&self) -> bool {
        self.default_enabled
    }

    /// Returns the version requested by this extension.
    pub fn version(&self) -> &Version {
        // This method is only used in testing right now, so let it pass
        #![allow(dead_code)]

        &self.version
    }

    /// Returns the PECL package specifier for this PECL extension, in the format NAME-VERSION.
    pub fn specifier(&self) -> String {
        format!("{}", self)
    }
}

impl<'a> Pecl {
    /// Attempts to parse a PECL extension name and version from the input.
    ///
    /// The syntax of an extension takes two forms:
    /// * `<name>` - this is an installation request for the latest stable `<name>`
    /// * `<name>@<version>` - this is an installation request for a specific version,
    ///   which is either the string "stable" or a semver version (MAJOR.MINOR.PATCH).
    pub fn parse<E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let name = parse::name;
        let version = opt(preceded(char('@'), Version::parse));
        let parser = tuple((name, version));
        let parser = map(parser, |(name, version): (&str, _)| {
            let pecl = REGISTRY
                .get(name)
                .cloned()
                .unwrap_or_else(|| Pecl::new(name.to_owned()));

            pecl.with_version(version.unwrap_or_default())
        });

        parser(input)
    }
}

impl fmt::Display for Pecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.name, self.version)
    }
}

lazy_static! {
    static ref REGISTRY: BTreeMap<&'static str, Pecl> = btreemap! {
        "imagick" => Pecl::new("imagick".to_owned())
            .with_packages(&["imagemagick-dev"]),

        "memcached" => Pecl::new("memcached".to_owned())
            .with_packages(&["libmemcached-dev", "zlib-dev", "libevent-dev"]),

        "xdebug" => Pecl::new("xdebug".to_owned()).disabled(),
    };
}

impl FromStr for Pecl {
    type Err = super::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse::parse_all(input, Self::parse)
    }
}

#[cfg(test)]
mod tests {
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
        assert_eq!(xdebug.name(), "xdebug");
        match xdebug.version {
            Version::Stable => {}
            _ => assert!(false),
        }
    }

    #[test]
    fn test_version() {
        let xdebug: Pecl = "xdebug@2.5.5".parse().unwrap();
        assert_eq!(xdebug.name(), "xdebug");
        match xdebug.version {
            Version::Custom(version) => {
                assert_eq!(version, "2.5.5");
            }
            _ => assert!(false),
        }
    }
}
