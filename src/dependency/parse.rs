//! Parsing helpers.
//!
//! Since `f1-ext-install` depends entirely on the input passed to it via the command line,
//! this module stores some centralized helpers to ensure consistent behavior by the various
//! `FromStr` implementations.

use nom::{
    character::complete::{alpha1, alphanumeric0, char},
    combinator::{all_consuming, recognize},
    error::{convert_error, ParseError as NomParseError, VerboseError},
    multi::separated_nonempty_list,
    sequence::{pair},
    Err, IResult,
};
use std::{error::Error, fmt};

/// Wraps an error message as returned by the parser.
#[derive(Debug)]
pub struct ParseError(String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Parse error: {}", self.0)
    }
}

impl Error for ParseError {}

/// Parser for an identifier.
///
/// Under the assumption that neither `docker-php-ext-install` nor `pecl install` recognize
/// a name like `foo__bar`, the parser recognizes an underscore-separated list.
pub fn name<'a, E>(input: &'a str) -> IResult<&'a str, &'a str, E>
where
    E: NomParseError<&'a str>,
{
    let name_part = pair(alpha1, alphanumeric0);
    let parser = separated_nonempty_list(char('_'), name_part);

    recognize(parser)(input)
}

/// Parsing helper: forces a parser to consume its entire input and return a verbose error
/// message on failure.
///
/// This function exists as a bridge between the various `parse` methods implemented on
/// dependency types and their `FromStr` implementations. By using a verbose error message,
/// the user is given better feedback on what parse error was encountered when the code
/// transforms the command-line arguments into [`Dependency`] elements.
///
/// [`Dependency`]: crate::dependency::Dependency
pub fn parse_all<'a, O, F>(input: &'a str, parser: F) -> Result<O, ParseError>
where
    F: Fn(&'a str) -> IResult<&'a str, O, VerboseError<&'a str>>,
{
    all_consuming(parser)(input)
        .map(|(_, result)| result)
        .map_err(|e| match e {
            Err::Error(e) | Err::Failure(e) => ParseError(convert_error(input, e)),
            Err::Incomplete(_) => ParseError("incomplete input".to_owned()),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse<'a, F>(parser: F, input: &'a str) -> IResult<&'a str, &'a str>
    where
        F: Fn(&'a str) -> IResult<&'a str, &'a str>,
    {
        parser(input)
    }

    #[test]
    fn test_name_list_single() {
        let input = "abc";
        let (remaining, result) = parse(name, input).unwrap();
        assert_eq!(remaining, "", "no input left");
        assert_eq!(result, "abc", "all input consumed");
    }

    #[test]
    fn test_name_list_multi() {
        let input = "abc_def";
        let (remaining, result) = parse(name, input).unwrap();
        assert_eq!(remaining, "", "no input left");
        assert_eq!(result, "abc_def", "all input consumed");
    }

    #[test]
    fn test_name_list_empty() {
        let input = "";
        let result = parse(name, input);
        assert!(result.is_err());
    }
}
