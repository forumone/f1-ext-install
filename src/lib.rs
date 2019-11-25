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

#![forbid(unsafe_code)]
#![deny(future_incompatible)]
#![deny(nonstandard_style)]
#![deny(rust_2018_compatibility)]
#![deny(rust_2018_idioms)]
#![deny(unused)]
#![deny(rustdoc)]
#![warn(clippy::missing_docs_in_private_items)]

pub mod dependency;
pub mod system;
