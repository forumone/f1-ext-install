#!/bin/bash

set -exuo pipefail

test_directory="$(dirname "${BASH_SOURCE[0]}")"

# Integration test script to verify that all of the supported PECL in the
# registry (src/dependency/pecl.rs) will compile with the added flags.

packages=(
  imagick
  memcached

  # Need to figure out a way to test an extension that isn't enabled by default.
  # xdebug
)

bash "$test_directory/harness.sh" pecl "${packages[@]}"
