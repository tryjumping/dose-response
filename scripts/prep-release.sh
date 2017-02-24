#!/bin/bash

set -eux

VERSION=$(grep version Cargo.toml | awk '{gsub(/"/, "", $3); print $3}')

git commit -m "Release version v${VERSION}"
git tag -a -m "Release version v${VERSION}" "v${VERSION}"
git push --follow-tags github github-release:master
