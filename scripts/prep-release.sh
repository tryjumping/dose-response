#!/bin/bash

set -eux

VERSION=grep version Cargo.toml | awk '{gsub(/"/, "", $3); print $3}'

rm -rf .copy
mkdir -p .copy
cp -r Cargo.lock Cargo.toml fonts src .copy

git checkout github-release
mv .copy/* .
rm -d .copy
git add -A
git commit -m "Release version v${VERSION}"
git tag "v${VERSION}"
git push --follow-tags github github-release:master
git checkout -
