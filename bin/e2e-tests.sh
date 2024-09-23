#!/bin/bash

set -euo pipefail

for f in replays/e2e-tests/*; do
    printf "\nRunning test: $f\n"
    /usr/bin/time --format "Elapsed: %e seconds" cargo run --release -- --quiet --headless --exit-after --replay-full-speed "$f"
done
