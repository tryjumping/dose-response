build-all:
	cargo build --release --no-default-features --features prod
	cargo check
	cargo build
	cargo build --release
.PHONY: build-all

replay:
	cargo run -- `find replays -type f -name 'replay-*' | sort | tail -n 1`
.PHONY: replay

replay-debug-fast:
	cargo run -- --replay-full-speed `find replays -type f -name 'replay-*' | sort | tail -n 1`
.PHONY: replay-debug-fast

replay-release:
	cargo run --release -- `find replays -type f -name 'replay-*' | sort | tail -n 1`
.PHONY: replay-release

replay-release-fast:
	cargo run --release -- --replay-full-speed `find replays -type f -name 'replay-*' | sort | tail -n 1`
.PHONY: replay-release-fast

release:
	bin/prep-release.sh
.PHONY: release

# NOTE: the `convert` binary comes with ImageMagick, so install that!
windows-icon: assets/icon_16x16.png assets/icon_32x32.png assets/icon_48x48.png assets/icon_64x64.png assets/icon_256x256.png
	convert assets/icon_16x16.png assets/icon_32x32.png assets/icon_48x48.png assets/icon_64x64.png assets/icon_256x256.png -colors 256 assets/icon.ico
.PHONY: windows-icon
