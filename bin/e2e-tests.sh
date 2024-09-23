#!/bin/bash

set -euxo pipefail

for f in replays/e2e-tests/*; do
    cargo run --release -- --exit-after --replay-full-speed "$f"
done
