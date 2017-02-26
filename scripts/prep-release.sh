#!/bin/bash

die() { echo "$@" 1>&2 ; exit 1; }

git diff-files --quiet || die "The repo is dirty. Commint everything before pushing a release."

set -eux

VERSION=$(grep '^version\s\+' Cargo.toml | awk '{gsub(/"/, "", $3); print $3}')

cargo build --release
git tag -a -m "Release version v${VERSION}" "v${VERSION}"
git push --follow-tags github master
