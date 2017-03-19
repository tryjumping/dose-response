#!/bin/bash

die() { echo "$@" 1>&2 ; exit 1; }

verify_repo_is_clean() {
    git diff-files --quiet || die "The repo is dirty. Commit everything before pushing a release.";
}

set -eux

verify_repo_is_clean

VERSION=$(grep '^version\s\+' Cargo.toml | awk '{gsub(/"/, "", $3); print $3}')

cargo build --release
verify_repo_is_clean
git tag -a -m "Release version v${VERSION}" "v${VERSION}"
git push --follow-tags origin master
