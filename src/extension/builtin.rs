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
    // NB. A few extensions are indicated in comments but not explicitly listed:
    // - A "no need" comment just means that there are no external dependencies
    //   for the extension, so we let BuiltinData::default() handle it.
    // - An "already loaded" comment means that for php:7.4-cli-alpine, the test
    //   extension_loaded("<name>") returns true, and we assume we don't need to add it.
    // - A "TODO" comment indicates that we can add the extension, but there may not be a
    //   need, so we've avoided adding it to the built-in registry for now.
    static ref REGISTRY: BTreeMap<&'static str, BuiltinData> = btreemap! {
        // bcmath: no need

        "bz2" => BuiltinData {
            packages: Some(vec![
                String::from("bzip2-dev"),
            ]),
            configure_cmd: Some(vec![
                String::from("--with-bz2")
            ]),
        },

        // calendar: no need

        // ctype: already loaded
        // curl: already loaded
        // dom: already loaded

        "enchant" => BuiltinData {
            packages: Some(vec![
                String::from("enchant-dev")
            ]),
            configure_cmd: Some(vec![
                String::from("--with-enchant"),
            ]),
        },

        // exif: no need

        // fileinfo: already loaded
        // filter: already loaded
        // ftp: already loaded

        "gd" => BuiltinData {
            packages: Some(vec![
                String::from("coreutils"),
                String::from("freetype-dev"),
                String::from("libjpeg-turbo-dev"),
            ]),
            configure_cmd: Some(vec![
                // Skip option checking while we still support pre-7.4 PHPs - this is a
                // pretty bad idea in general, but since we're applying it only to the GD
                // extension in particular, we should be relatively safe.
                String::from("--disable-option-checking"),

                String::from("--with-freetype-dir=/usr/include/"),
                String::from("--with-jpeg-dir=/usr/include/"),
                String::from("--with-png-dir=/usr/include/"),
            ]),
        },

        "gettext" => BuiltinData {
            packages: Some(vec![
                String::from("gettext"),
                String::from("gettext-dev"),
            ]),
            configure_cmd: Some(vec![
                String::from("--with-gettext")
            ]),
        },

        "gmp" => BuiltinData {
            packages: Some(vec![
                String::from("gmp-dev"),
            ]),
            configure_cmd: Some(vec![
                String::from("--with-gmp")
            ]),
        },

        // iconv: already loaded

        "imap" => BuiltinData {
            packages: Some(vec![
                String::from("imap-dev"),
                String::from("openssl-dev"),
            ]),
            configure_cmd: Some(vec![
                String::from("--with-imap"),
                String::from("--with-imap-ssl"),
            ]),
        },

        "intl" => BuiltinData {
            packages: Some(vec![
                String::from("icu-dev"),
            ]),
            ..BuiltinData::default()
        },

        // json: already loaded

        "ldap" => BuiltinData {
            packages: Some(vec![
                String::from("openldap-dev")
            ]),
            configure_cmd: Some(vec![
                String::from("--with-ldap"),
                String::from("--with-ldap-sasl"),
            ]),
        },

        // mbstring: already loaded
        // mysqli: no need
        // mysqlnd: no need
        // opcache: no need
        // pcntl: no need
        // phar: no need
        // pdo: already loaded
        // pdo_mysql: no need
        // pdo_pgsql: TODO
        // posix: already loaded
        // pspell: TODO
        // session: already loaded
        // simplexml: already loaded

        "soap" => BuiltinData {
            packages: Some(vec![String::from("libxml2-dev")]),
            ..BuiltinData::default()
        },

        // sodium: already loaded
        // sqlite3: already loaded
        // tokenizer: already loaded
        // xml: already loaded
        // xmlreader: already loaded
        // xmlrpc: TODO
        // xmlwriter: already loaded
        // xsl: TODO

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
