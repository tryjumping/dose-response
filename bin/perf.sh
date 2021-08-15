#!/bin/bash

# Don't pollute the working directory with perf output.
# That makes it dirty and causes recompilation.
cd target

#cargo flamegraph --output flamegraph.svg --bin dose-response
cargo flamegraph --output flamegraph.svg --bin dose-response -- --replay-full-speed --exit-after ../bin/replay.log
