//! Helper types to handle PHP dependencies.
//!
//! A dependency is broken down into two categories: builtins and PECL. The structs in
//! this module exist to capture the information needed to configure and install them.

use nom::{
    branch::alt, bytes::complete::tag, combinator::map, error::ParseError as NomParseError,
    sequence::preceded, IResult,
};
use std::str::FromStr;

mod builtin;
mod parse;
mod pecl;
mod version;

pub use builtin::Builtin;
pub use parse::ParseError;
pub use pecl::Pecl;
pub use version::Version;

/// Encapsulates a dependency needed by the Docker image currently being built.
#[derive(Clone, Debug)]
pub enum Dependency {
    /// This dependency is a PHP builtin (e.g., `gd`, `opcache).
    Builtin(Builtin),

    /// This dependency is a PECL extension (e.g., `memcached`, XDebug).
    Pecl(Pecl),
}

impl Dependency {
    /// Retrieves the list of packages (if any) needed by this dependency. A package is
    /// represented by its name as intepreted by the `apk` package manager.
    pub fn packages(&self) -> Option<&[String]> {
        match self {
            Self::Builtin(builtin) => builtin.packages(),
            Self::Pecl(pecl) => pecl.packages(),
        }
    }

    /// Determines if this dependency needs any external packages.
    pub fn has_packages(&self) -> bool {
        match self.packages() {
            None => false,
            Some(packages) => !packages.is_empty(),
        }
    }

    /// Attempts to read a dependency from the given input.
    ///
    /// The syntax for a dependency is `<type>:<specifier>`, where `<type>` is one of
    /// `builtin` or `pecl` and `<specifier>` varies. See the `parse` methods of [`Builtin`]
    /// and [`Pecl`] for more details.
    ///
    /// This method is exposed publicly for ease of composition with other parsers.
    ///
    /// [`Builtin`]: crate::dependency::Builtin
    /// [`Pecl`]: crate::dependency::Pecl
    pub fn parse<'a, E: NomParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let builtin = preceded(tag("builtin:"), Builtin::parse);
        let builtin = map(builtin, Self::Builtin);

        let pecl = preceded(tag("pecl:"), Pecl::parse);
        let pecl = map(pecl, Self::Pecl);

        let parser = alt((builtin, pecl));

        parser(input)
    }
}

impl FromStr for Dependency {
    type Err = ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse::parse_all(input, Self::parse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_builtin() {
        let gd: Dependency = "builtin:gd".parse().unwrap();
        match gd {
            Dependency::Builtin(gd) => {
                assert_eq!(gd.name(), "gd");
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_pecl() {
        let xdebug: Dependency = "pecl:xdebug".parse().unwrap();
        match xdebug {
            Dependency::Pecl(xdebug) => {
                assert_eq!(xdebug.name(), "xdebug");
                match xdebug.version() {
                    Version::Stable => assert!(true),
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_pecl_explicit_stable() {
        let xdebug: Dependency = "pecl:xdebug@stable".parse().unwrap();
        match xdebug {
            Dependency::Pecl(xdebug) => {
                assert_eq!(xdebug.name(), "xdebug");
                match xdebug.version() {
                    Version::Stable => assert!(true),
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    fn test_parse_pecl_explicit_version() {
        let xdebug: Dependency = "pecl:xdebug@2.5.5".parse().unwrap();
        match xdebug {
            Dependency::Pecl(xdebug) => {
                assert_eq!(xdebug.name(), "xdebug");
                match xdebug.version() {
                    Version::Custom(version) => assert_eq!(*version, "2.5.5"),
                    _ => assert!(false),
                }
            }
            _ => assert!(false),
        }
    }

    #[test]
    #[should_panic]
    fn test_parse_pecl_garbage_version() {
        let _: Dependency = "pecl:xdebug@askjdfh".parse().unwrap();
    }
}
