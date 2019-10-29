#!/bin/bash

set -euo pipefail

buildkite_directory="$(dirname "${BASH_SOURCE[0]}")"
project_directory="$(realpath "$buildkite_directory/..")"

repository_name=f1-ext-install

declare -a versions

chmod +x target/x86_64-unknown-linux-musl/debug/versions
coproc VERSIONS { target/x86_64-unknown-linux-musl/debug/versions; }

# Immediately save the PID of the VERSIONS coproc
id="$VERSIONS_PID"

# Clone the VERSIONS coproc's stdout
exec 4<&"${VERSIONS[0]}"

# Propagate non-zero exits of the 'versions' command
wait "$id"

mapfile -t -u 4 versions

echo '--- :docker: Build'
docker build "$project_directory" --tag "$repository_name:latest"

for version in "${versions[@]}"; do
  docker tag "$repository_name:latest" "$repository_name:$version"
done

echo '--- :docker: Push'
echo '^^^ +++ TODO'
