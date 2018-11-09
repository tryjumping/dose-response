replay:
	cargo run -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

replay-release:
	cargo run --release -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

replay-full-speed:
	cargo run --release -- --replay-full-speed `find replays -type f -name 'replay-*' | sort | tail -n 1`

release:
	scripts/prep-release.sh

wasm:
	cargo build --release --target wasm32-unknown-unknown --no-default-features --features web

wasm-release: wasm
	scripts/wasm-release.sh

.PHONY: replay release wasm wasm-release
