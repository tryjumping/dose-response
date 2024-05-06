# NOTE: we're using a quick check as the default target.
#
# Unlike the `package-release`, it takes a short time to finish so you
# don't waste a lot of time by running it accidentally.
check:
	cargo check
.PHONY: check

package-release:
	cargo check
	cargo check --no-default-features --features "prod ${EXTRA_FEATURES}"
	cargo clippy
	cargo build
	cargo test --all-targets
	cargo install cargo-about --version "0.6.1"
	cargo about generate --no-default-features --features "prod ${EXTRA_FEATURES}" about.hbs --output-file third-party-licenses.html
	cargo build --release --no-default-features --features "prod ${EXTRA_FEATURES}"
	cargo run --manifest-path bin/Cargo.toml --bin package-release
.PHONY: package-release

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

cargo-release:
	bin/prep-release.sh
.PHONY: cargo-release

# NOTE: the `convert` binary comes with ImageMagick, so install that!
windows-icon: assets/icon_16x16.png assets/icon_32x32.png assets/icon_48x48.png assets/icon_64x64.png assets/icon_256x256.png
	convert assets/icon_16x16.png assets/icon_32x32.png assets/icon_48x48.png assets/icon_64x64.png assets/icon_256x256.png -colors 256 assets/icon.ico
.PHONY: windows-icon
