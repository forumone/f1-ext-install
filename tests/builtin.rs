use indoc::indoc;

mod common;
use common::{build_image, connect, tag_for_test, PHP_VERSIONS};

const REGISTRY_DOCKERFILE: &str = indoc!(
    r#"ARG PHP_VERSION
       FROM php:${PHP_VERSION}-cli-alpine

       COPY f1-ext-install /usr/bin/
       RUN chmod +x /usr/bin/f1-ext-install

       ARG PACKAGE_NAME
       ENV PACKAGE_NAME=${PACKAGE_NAME}

       RUN f1-ext-install "builtin:${PACKAGE_NAME}" >/dev/null
       RUN php -r "if (!extension_loaded(getenv('PACKAGE_NAME'))) exit(1);""#
);

/// Shorthand macro to define a test for a builtin from the registry. Takes an extension
/// name (as a Rust identifier) as its only argument.
///
/// This macro exists for two reasons:
/// 1. It abstracts away the boilerplate of setting up a test for a new builtin added to
/// the internal registry and
/// 2. It enables us to separate tests for each extension.
///
/// The cargo test infrastructure gets somewhat cranky when we run a test for too long,
/// so this macro helps us split the tests up with extremely minimal repetition.
///
/// For example, given this invocation:
///
/// ```
/// define_registry_test!(bz2);
/// ```
///
/// The macro expands to a test function named `bz2` that installs `builtin:bz2` in a
/// Docker build test.
macro_rules! define_registry_test {
    ($builtin:ident) => {
        #[test]
        fn $builtin() {
            let client = connect();

            let builtin = stringify!($builtin);

            for &version in PHP_VERSIONS {
                let tag = tag_for_test("builtin", builtin, version);

                build_image(
                    &client,
                    REGISTRY_DOCKERFILE,
                    &[("PACKAGE_NAME", builtin), ("PHP_VERSION", version)],
                    &tag,
                );
            }
        }
    };
}

define_registry_test!(bz2);
define_registry_test!(enchant);
define_registry_test!(gd);
define_registry_test!(gettext);
define_registry_test!(gmp);
define_registry_test!(imap);
define_registry_test!(intl);
define_registry_test!(ldap);
define_registry_test!(soap);
define_registry_test!(zip);

const EXTERNAL_DOCKERFILE: &str = indoc!(
    r#"ARG PHP_VERSION
       FROM php:${PHP_VERSION}-cli-alpine

       COPY f1-ext-install /usr/bin/
       RUN chmod +x /usr/bin/f1-ext-install

       ARG PACKAGE_NAME
       ENV PACKAGE_NAME=${PACKAGE_NAME}

       ARG PACKAGES_KEY
       ARG PACKAGES_VALUE
       ENV ${PACKAGES_KEY:-_p}=${PACKAGES_VALUE}

       ARG CONFIGURE_KEY
       ARG CONFIGURE_VALUE
       ENV ${CONFIGURE_KEY:-_c}=${CONFIGURE_VALUE}

       RUN f1-ext-install "builtin:${PACKAGE_NAME}" >/dev/null
       RUN php -r "if (!extension_loaded(getenv('PACKAGE_NAME'))) exit(1);""#
);

/// Shorthand to define a test for a PHP builtin that relies on external configuration.
/// Takes an extension name (as a Rust identifier) plus configuration as its parameters
/// (see below).
///
/// As with `define_builtin_test` above, this macro attemps to reduce boilerplate and
/// split each builtin test into its own #[test] function.
///
/// The syntax is as follows:
///
/// ```no_run
/// define_external_test!(
///     // The name of the builtin being tested, as a Rust identifier
///     ext,
///
///     // The environment variable needed to specify packages to install.
///     F1_BUILTIN_EXT_PACKAGES = "foo,bar",
///
///     // The environment variable specifying the configure flags that are needed.
///     F1_BUILTIN_EXT_CONFIGURE_ARGS = "--with-ext",
/// );
/// ```
///
/// Like `define_builtin_test`, this macro expands to a single test function named after
/// the extension under test.
///
/// Note that the `FOO = "foo"` syntax is required by the macro; it is designed to mimic
/// the shell/Dockerfile needed to install the package.
macro_rules! define_external_test {
    (
        // The extension name
        $name:ident,
        // The ENV_VAR = "value" for which packages (if any) to install
        $pkgs_key:ident = $pkgs_val:tt,
        // The ENV_VAR = "value" for additional configure flags (if any)
        $conf_key:ident = $conf_val:tt $(,)?
    ) => {
        #[test]
        fn $name() {
            let client = connect();

            let package = stringify!($name);
            let packages_key = stringify!($pkgs_key);
            let packages_value = stringify!($packages_value);
            let configure_key = stringify!($conf_key);
            let configure_value = stringify!($conf_val);

            for &version in PHP_VERSIONS {
                let tag = tag_for_test("builtin", package, version);

                build_image(
                    &client,
                    EXTERNAL_DOCKERFILE,
                    &[
                        ("PACKAGE_NAME", package),
                        ("PHP_VERSION", version),
                        ("PACKAGES_KEY", packages_key),
                        ("PACKAGES_VALUE", packages_value),
                        ("CONFIGURE_KEY", configure_key),
                        ("CONFIGURE_VALUE", configure_value),
                    ],
                    &tag,
                );
            }
        }
    };
}

define_external_test!(
    ffi,
    F1_BUILTIN_FFI_PACKAGES = "libffi-dev",
    F1_BUILTIN_FFI_CONFIGURE_ARGS = "--with-ffi",
);
