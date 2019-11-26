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

       RUN f1-ext-install "pecl:${PACKAGE_NAME}" >/dev/null
       RUN php -r "if (!extension_loaded(getenv('PACKAGE_NAME'))) exit(1);""#
);

#[test]
fn test_pecl_from_registry() {
    let client = connect();

    let packages = &["imagick", "memcached"];

    for &package in packages {
        for &version in PHP_VERSIONS {
            let tag = tag_for_test("pecl", package, version);

            build_image(
                &client,
                REGISTRY_DOCKERFILE,
                &[("PACKAGE_NAME", package), ("PHP_VERSION", version)],
                &tag,
            );
        }
    }
}

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

       RUN f1-ext-install "pecl:${PACKAGE_NAME}" >/dev/null
       RUN php -r "if (!extension_loaded(getenv('PACKAGE_NAME'))) exit(1);""#
);

#[test]
fn test_external_pecl_config() {
    let client = connect();

    let packages = &[("mcrypt", ("F1_PECL_MCRYPT_PACKAGES", "libmcrypt-dev"))];

    for (package, (packages_key, packages_value)) in packages {
        for &version in PHP_VERSIONS {
            let tag = tag_for_test("pecl", package, version);

            build_image(
                &client,
                EXTERNAL_DOCKERFILE,
                &[
                    ("PACKAGE_NAME", "ldap"),
                    ("PHP_VERSION", version),
                    ("PACKAGES_KEY", packages_key),
                    ("PACKAGES_VALUE", packages_value),
                ],
                &tag,
            );
        }
    }
}
