replay:
	cargo run -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

replay-debug-fast:
	cargo run -- --replay-full-speed `find replays -type f -name 'replay-*' | sort | tail -n 1`

replay-release:
	cargo run --release -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

replay-release-fast:
	cargo run --release -- --replay-full-speed `find replays -type f -name 'replay-*' | sort | tail -n 1`

release:
	scripts/prep-release.sh

wasm:
	cargo build --release --target wasm32-unknown-unknown --no-default-features --features web

wasm-release: wasm
	scripts/wasm-release.sh

# NOTE: the `convert` binary comes with ImageMagick, so install that!
windows-icon: assets/icon_16x16.png assets/icon_32x32.png assets/icon_48x48.png assets/icon_64x64.png assets/icon_256x256.png
	convert assets/icon_16x16.png assets/icon_32x32.png assets/icon_48x48.png assets/icon_64x64.png assets/icon_256x256.png -colors 256 assets/icon.ico

.PHONY: replay release wasm wasm-release
