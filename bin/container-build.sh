#!/bin/bash

set -euo pipefail

if [ -z "$(podman images --quiet dose-response-builder)" ]; then
    podman build -t dose-response-builder .
fi

podman run -i -t --rm -v .:/app dose-response-builder bash -c 'cd app && cargo build --release --target-dir target/container-build'

printf "Build created in:\ntarget/container-build/release/dose-response\n"
