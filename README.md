# About this image

## Usage

```sh
# Install a builtin (GD, opache, etc.)
f1-ext-install builtin:gd

# Install a PECL package
f1-ext-install pecl:memcached

# Install a PECL package at a specific version
f1-ext-install pecl:xdebug@2.5.5 # last supported version for PHP 5.6

# Install multiple packages (recommended)
f1-ext-install builtin:gd builtin:opcache pecl:memcached

# View help
f1-ext-install --help
```

## Discussion

This image is intended for use when building PHP-based Docker images. Its sole function
is to provide a utility called `f1-ext-install`, a small binary that serves as an
abstraction over the vagaries of Linux package managers. At the time, there is only
support for `apk`, but support for `apt` is planned.

To give an example, here is the necessary set of "raw" commands needed to install the
[memcached](http://pecl.php.net/package/memcached) PECL extension into a Docker image:

```dockerfile
RUN set -ex \
  # Install necessary build-time dependencies
  && apk add --no-cache --virtual .build-deps $PHPIZE_DEPS libmemcached-dev zlib-dev libevent-dev \
  # Install the memcached extension from PECL
  && pecl install memcached \
  # Enable the extension
  && docker-php-ext-enable memcached \
  # Scan /usr/local to capture run-time dependencies
  && runDeps="$(\
  scanelf --needed --nobanner --format '%n#p' --recursive /usr/local \
    | tr ',' '\n' \
    | sort -u \
    | awk 'system("[ -e /usr/local/lib/" $1 " ]") == 0 { next } { print "so:" $1 }' \
  )" \
  # Force apk to save the runtime dependencies
  && apk add --virtual .docker-phpexts-rundeps $runDeps \
  # Delete build-time dependencies to save image space
  && apk del .build-deps
```

This utility includes built-in knowledge for `memcached`'s dependencies, so it suffices to
simply say `f1-ext-install pecl:memcached` to obtain the extension.

This utility is _not_ suitable for general-purpose use; it assumes that it is running
inside a Docker container during a build and can thus use mutate the systemwide state.

# About this repository

The `f1-ext-install` utility is written using the [Rust](https://www.rust-lang.org/)
programming language. Please see that site for any langauge-related needs.
