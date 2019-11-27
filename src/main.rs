use anyhow::Result;
use structopt::StructOpt;

use f1_ext_install::{
    dependency::Dependency,
    system::{self, Apk},
};

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

fn main() -> Result<()> {
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
