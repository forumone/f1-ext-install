#!/bin/bash

set -euo pipefail

test_directory="$(dirname "${BASH_SOURCE[0]}")"

# Integration test script to verify that all of the supported builtins in the
# registry (src/dependency/builtin.rs) will compile with the added flags.

builtins=(
  gd
  soap
  zip

  # We should test that this installs successfully, but it's not enabled by default for
  # the PHP CLI.
  # opcache
)

bash "$test_directory/harness.sh" builtin "${builtins[@]}"
