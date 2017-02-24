#!/bin/bash

set -eux

VERSION=$(grep '^version\s\+' Cargo.toml | awk '{gsub(/"/, "", $3); print $3}')

git tag -a -m "Release version v${VERSION}" "v${VERSION}"
git push --follow-tags github master
