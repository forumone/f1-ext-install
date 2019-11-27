//! Helper for Alpine `apk` package management.

use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashSet, fs::File, path::Path};

use super::{
    collect_packages,
    command::{self, Command},
};
use crate::dependency::Dependency;

/// Helper function to split the output of `scanelf`.
fn split_scanelf_output(input: &str) -> HashSet<&str> {
    lazy_static!{
        static ref DELIM: Regex = Regex::new(r"[,\s]+").unwrap();
    };

    DELIM.split(input).filter(|s| !s.is_empty()).collect()
}

/// Struct representing an Alpine package manager.
pub struct Apk;

impl Apk {
    /// Uses the system package manager to install the packages required by the given
    /// list of dependencies.
    ///
    /// This method also uses the dependencies stored in `$PHPIZE_DEPS`, granting access
    /// to the C compiler and other tools.
    pub fn install_packages(&self, dependencies: &[Dependency]) -> command::Result<()> {
        let packages = collect_packages(dependencies);

        let mut command = Command::new("apk");
        command.args(&["add", "--no-cache", "--virtual", ".build-deps"]);
        command.args(&packages);

        let _ = command.status()?;

        Ok(())
    }

    /// Marks all runtime dependencies of binaries in `/usr/local` as required in the
    /// system package manager.
    ///
    /// This method ensures that, when cleaning build-time dependencies, packages that
    /// provide needed `.so` files aren't cleared away.
    pub fn save_runtime_deps(&self) -> command::Result<()> {
        let mut command = Command::new("scanelf");
        command.args(&[
            "--needed",
            "--nobanner",
            "--format",
            "%n#p",
            "--recursive",
            "/usr/local",
        ]);

        let output = command.stdout()?;
        let deps_found = split_scanelf_output(&output);
        let rundeps: Vec<_> = deps_found
            .iter()
            .filter_map(|dep_name| {
                let path = Path::new("/usr/local/lib").join(dep_name);
                if File::open(path).is_ok() {
                    return None;
                }

                Some(format!("so:{}", dep_name))
            })
            .collect();

        if !rundeps.is_empty() {
            let mut command = Command::new("apk");
            command.args(&["add", "--virtual", ".docker-phpexts-rundeps"]);
            command.args(rundeps);
            command.wait()?;
        }

        Ok(())
    }

    /// Clear out all build-time dependencies (both `$PHPIZE_DEPS` and user-requested).
    pub fn remove_build_deps(&self) -> command::Result<()> {
        let mut command = Command::new("apk");
        command.args(&["del", ".build-deps"]);
        command.wait()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_elements() {
        let expected: HashSet<_> = [
            "libc.musl-x86_64.so.1",
            "libcrypto.so.43",
            "libcurl.so.4",
            "libedit.so.0",
            "libfreetype.so.6",
            "libjpeg.so.8",
            "libpng16.so.16",
            "libssl.so.45",
            "libxml2.so.2",
            "libz.so.1",
        ]
        .iter()
        .cloned()
        .collect();

        let input = r#"
libedit.so.0,libcurl.so.4,libz.so.1,libxml2.so.2,libssl.so.45,libcrypto.so.43,libc.musl-x86_64.so.1
libc.musl-x86_64.so.1
libpng16.so.16,libz.so.1,libjpeg.so.8,libfreetype.so.6,libc.musl-x86_64.so.1
libz.so.1,libc.musl-x86_64.so.1
libedit.so.0,libcurl.so.4,libz.so.1,libxml2.so.2,libssl.so.45,libcrypto.so.43,libc.musl-x86_64.so.1
"#;

        let output = split_scanelf_output(input);

        assert_eq!(expected, output);
    }
}
