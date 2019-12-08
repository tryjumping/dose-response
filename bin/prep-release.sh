#!/bin/bash

die() { echo "$@" 1>&2 ; exit 1; }

verify_repo_is_clean() {
    git diff-files --quiet || die "The repo is dirty. Commit everything before pushing a release.";
    test -z "$(git status --porcelain)" || die "The repo has staged but uncommited files."
}

set -eux

verify_repo_is_clean

VERSION=$(grep '^version\s\+' Cargo.toml | head -n 1 | awk '{gsub(/"/, "", $3); print $3}')
echo Version: $VERSION

cargo build --release --no-default-features --features prod
verify_repo_is_clean
git tag -a -m "Release version v${VERSION}" "v${VERSION}"
git push --tags --follow-tags origin master
