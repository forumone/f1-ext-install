//! Type and helpers for PECL version specifiers.

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, recognize},
    error::ParseError,
    sequence::{terminated, tuple},
    IResult,
};
use std::{fmt, str::FromStr};

use super::parse;

/// Represents a PECL version.
#[derive(Clone, Debug)]
pub enum Version {
    /// The `stable` version/channel.
    Stable,
    /// A specific version (in MAJOR.MINOR.PATCH format).
    Custom(String),
}

impl Version {
    /// Attempts to parse a version from the input.
    ///
    /// The syntax of the input is a subset of what is allowed by the `pecl install` command.
    /// The input is required to be either the string "stable", or a full semver version
    /// (MAJOR.MINOR.PATCH).
    ///
    /// This method is exposed publicly for easier composition with other parsers. For example,
    /// the `parse` method of [`Pecl`] uses this method.
    ///
    /// [`Pecl`]: crate::dependency::Pecl
    pub fn parse<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, Self, E> {
        let stable = tag("stable");
        let stable = map(stable, |_| Version::Stable);

        let version_part = || terminated(digit1, char('.'));
        let version = recognize(tuple((version_part(), version_part(), digit1)));
        let version = map(version, |version: &str| Version::Custom(version.to_owned()));

        alt((stable, version))(input)
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::Stable
    }
}

impl FromStr for Version {
    type Err = super::ParseError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        parse::parse_all(input, Self::parse)
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
