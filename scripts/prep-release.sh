#!/bin/bash

set -eux

VERSION=$(grep version Cargo.toml | awk '{gsub(/"/, "", $3); print $3}')

rm -rf .copy
mkdir -p .copy
cp -r Cargo.lock Cargo.toml fonts src .copy

git checkout github-release
cp -r .copy/* .
rm -rf .copy
git add -A
git commit -m "Release version v${VERSION}"
git tag -a -m "Release version v${VERSION}" "v${VERSION}"
git push --follow-tags github github-release:master
git checkout -
