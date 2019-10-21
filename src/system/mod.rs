//! System interaction helpers.

use lazy_static::lazy_static;
use num_cpus;
use std::{borrow::Cow, env};

mod alpine;
pub mod command;

use super::dependency::{Dependency, Pecl};
use command::Command;

pub use alpine::Apk;

lazy_static! {
    static ref NUM_CPUS: String = format!("{}", num_cpus::get());
}

/// Collect the system packages needed the provided lest of dependencies.
///
/// This function also collects the values in `$PHPIZE_DEPS`, which names the system
/// C compiler and other utilities needed to build extensions.
pub fn collect_packages<'a>(dependencies: &'a [Dependency]) -> Vec<Cow<'a, str>> {
    let mut all_packages: Vec<Cow<'_, _>> = Vec::new();

    let phpize_deps = env::var("PHPIZE_DEPS").unwrap_or_default();
    let phpize_deps = phpize_deps
        .split_ascii_whitespace()
        .map(str::to_owned)
        .map(Into::into);

    all_packages.extend(phpize_deps);

    for dependency in dependencies {
        if let Some(packages) = dependency.packages() {
            all_packages.extend(packages.iter().map(|s| (*s).into()));
        }
    }

    all_packages
}

/// Invokes `docker-php-ext-configure` for the given builtin name and configure arguments.
pub fn configure_builtin(name: &str, configure_args: &[&str]) -> command::Result<()> {
    let mut command = Command::new("docker-php-ext-configure");
    command.arg(name);
    command.args(configure_args);

    command.wait()
}

/// Invokes `docker-php-ext-install` for the given list of builtins.
///
/// If the list is empty, no installation is performed.
pub fn install_builtins<I, S>(builtins: I) -> command::Result<()>
where
    S: AsRef<str>,
    I: IntoIterator<Item = S>,
{
    let mut builtins = builtins.into_iter();
    let builtin = match builtins.next() {
        Some(builtin) => builtin,
        None => return Ok(()),
    };

    let mut command = Command::new("docker-php-ext-install");
    command.arg("-j");
    command.arg(&*NUM_CPUS);
    command.arg(builtin);
    command.args(builtins);

    command.wait()
}

/// Installs the given PECL extension, and enables it if specified.
pub fn install_pecl_extension(pecl: &Pecl) -> command::Result<()> {
    let name = pecl.name();
        let enabled = pecl.default_enabled();

        let mut command = Command::new("pecl");
        command.arg("install");
        command.arg(pecl.specifier());
        command.wait()?;

        if enabled {
            let mut command = Command::new("docker-php-ext-enable");
            command.arg(name);
            command.wait()?;
        }

        Ok(())
}
