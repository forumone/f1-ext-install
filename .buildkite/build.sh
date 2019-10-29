#!/bin/bash

set -euo pipefail

buildkite_directory="$(dirname "${BASH_SOURCE[0]}")"
project_directory="$(realpath "$buildkite_directory/..")"

repository_name=f1-ext-install

declare -a versions

coproc VERSIONS { bin/versions; }

mapfile -t -u "${VERSIONS[0]}" versions

echo '--- :docker: Build'
docker build "$project_directory" --tag "$repository_name:latest"

for version in "${versions[@]}"; do
  docker tag "$repository_name:latest" "$repository_name:$version"
done

echo '--- :docker: Push'
echo '^^^ +++ TODO'
