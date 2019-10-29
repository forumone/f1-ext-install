# Dockerfile for quick integration tests against the f1-ext-install command.

# Which PHP version we're building against
ARG PHP_VERSION

FROM php:${PHP_VERSION}-cli-alpine

COPY f1-ext-install /usr/bin/
RUN chmod +x /usr/bin/f1-ext-install

# PACKAGE_TYPE is either "builtin" or "pecl"
# PACKAGE_NAME is an extension (gd, memcached, etc.)
ARG PACKAGE_TYPE
ARG PACKAGE_NAME

ENV PACKAGE_TYPE=${PACKAGE_TYPE} PACKAGE_NAME=${PACKAGE_NAME}

RUN f1-ext-install "${PACKAGE_TYPE}:${PACKAGE_NAME}" >/dev/null

# Verify the extension is loaded - success here means that PHP has all it needs for the
# package to run.
RUN php -r "if (!extension_loaded(getenv('PACKAGE_NAME'))) exit(1);"
