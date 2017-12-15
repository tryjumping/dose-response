replay:
	cargo run -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

release:
	scripts/prep-release.sh

wasm:
	cargo +nightly build --release --target wasm32-unknown-unknown --no-default-features --features web

.PHONY: replay release wasm
