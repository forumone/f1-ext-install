//! The `f1-ext-install` utility is a small binary meant to simplify the authoring process
//! for PHP-based Dockerfiles. It encapsulates the process of installing PHP extensions —
//! both builtin and from PECL — so that users won't have to remember long strings of `apk`
//! commands. This is not only more ergonomic, but it is able to handle Dockerfile best
//! practices (for example, minimizing the layer size) automatically.
//!
//! For example, the process of installing the `memcached` extension has been changed from
//! this:
//!
//! ```dockerfile
//! RUN set -ex \
//!   # Install necessary build-time dependencies
//!   && apk add --no-cache --virtual .build-deps $PHPIZE_DEPS libmemcached-dev zlib-dev libevent-dev \
//!   # Install the memcached extension from PECL
//!   && pecl install memcached \
//!   # Enable the extension
//!   && docker-php-ext-enable memcached \
//!   # Scan /usr/local to capture run-time dependencies
//!   && runDeps="$(\
//!   scanelf --needed --nobanner --format '%n#p' --recursive /usr/local \
//!     | tr ',' '\n' \
//!     | sort -u \
//!     | awk 'system("[ -e /usr/local/lib/" $1 " ]") == 0 { next } { print "so:" $1 }' \
//!   )" \
//!   # Force apk to save the runtime dependencies
//!   && apk add --virtual .docker-phpexts-rundeps $runDeps \
//!   # Delete build-time dependencies to save image space
//!   && apk del .build-deps
//! ```
//!
//! To this single line:
//!
//! ```dockerfile
//! RUN f1-ext-install pecl:memcached
//! ```
//!
//! It is a hard-coded assumption that `f1-ext-install` is run inside a container during
//! build, and is not recommended to be used anywhere else.

#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(unused)]
#![deny(rustdoc)]
#![warn(clippy::missing_docs_in_private_items)]

mod dependency;
mod system;

use quick_error::quick_error;
use std::str;
use structopt::StructOpt;

use dependency::Dependency;
use system::Apk;

quick_error! {
    /// Overall enum for errors.
    #[derive(Debug)]
    pub enum MainError {
        /// Represents an error that arose during execution of an external command.
        Command(err: system::command::CommandError) {
            cause(err)
            from()
        }
    }
}

/// Command-line options provided to `f1-ext-install`.
#[derive(StructOpt, Debug)]
#[structopt(about)]
struct Opts {
    /// The dependencies to install during this execution.
    ///
    /// Dependencies are provided with a simple syntax:
    ///
    /// * `builtin:<name>` - install the named PHP builtin
    ///
    /// * `pecl:<name>` - install the latest stable version of the named PECL extension
    ///
    /// * `pecl:<name>@stable` - explicitly use the stable channel
    ///
    /// * `pecl:<name>@<version>` - install a specific version (in MAJOR.MINOR.PATCH) format
    #[structopt(min_values(1))]
    dependencies: Vec<Dependency>,
}

fn main() -> Result<(), MainError> {
    env_logger::init();

    let opts = Opts::from_args();
    let manager = Apk;

    manager.install_packages(&opts.dependencies)?;

    let builtins: Vec<_> = opts
        .dependencies
        .iter()
        .filter_map(|dependency| match dependency {
            Dependency::Builtin(builtin) => Some(builtin),
            _ => None,
        })
        .collect();

    for builtin in &builtins {
        if let Some(configure_cmd) = builtin.configure_cmd() {
            system::configure_builtin(builtin.name(), configure_cmd)?;
        }
    }

    system::install_builtins(builtins.iter().map(|builtin| builtin.name()))?;

    for dependency in &opts.dependencies {
        let pecl = match dependency {
            Dependency::Pecl(pecl) => pecl,
            _ => continue,
        };

        system::install_pecl_extension(pecl)?;
    }

    let save_rundeps = opts.dependencies.iter().any(Dependency::has_packages);
    if save_rundeps {
        manager.save_runtime_deps()?;
    }

    manager.remove_build_deps()?;

    Ok(())
}
