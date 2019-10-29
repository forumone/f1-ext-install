#!/bin/bash

set -euo pipefail

test_directory="$(dirname "${BASH_SOURCE[0]}")"
project_directory="$(realpath "$test_directory/..")"

php_versions=(7.3)

# Integration test harness. Performs a test build of a given package (builtin or PECL)
# via Docker.
#
# USAGE: harness.sh <package_type> [package...]
#
# <package_type>: either "builtin" or "pecl", as used by f1-ext-install
# [package...]: a list of package names (gd, memcached, etc.)

package_type="$1"
shift

if test $# -eq 0; then
  echo "USAGE: $0 <package_type> [packages...]" >/dev/stderr
  exit 1
fi

build() {
  local php_version="$1"
  local package="$2"

  echo "--- Build $package_type:$package (PHP $php_version)"

  docker build "$test_directory" \
    --build-arg "PHP_VERSION=$php_version" \
    --build-arg "PACKAGE_TYPE=$package_type" \
    --build-arg "PACKAGE_NAME=$package" \
    --tag "$package_type-test:$package"
}

declare -a failures=()

cp "$project_directory/target/x86_64-unknown-linux-musl/debug/f1-ext-install" "$test_directory/f1-ext-install"

for version in "${php_versions[@]}"; do
  for package in "$@"; do
    if ! build "$version" "$package"; then
      failures+=("$package_type:$package (PHP ${version})")
    fi
  done
done

echo '+++ Results'

if test ${#failures[@]} -eq 0; then
  echo "Success"
else
  echo "The following packages failed to build:" >/dev/stderr

  for failure in "${failures[@]}"; do
    echo "  * $failure" >/dev/stderr
  done

  exit 1
fi
