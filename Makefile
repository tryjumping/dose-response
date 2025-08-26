# NOTE: we're using a quick check as the default target.
#
# Unlike the `package-release`, it takes a short time to finish so you
# don't waste a lot of time by running it accidentally.
check:
	cargo check
.PHONY: check

package-release: cargo-all-tests
	cargo install cargo-about --version "0.6.1"
	cargo about generate --no-default-features --features "prod ${EXTRA_FEATURES}" about.hbs --output-file third-party-licenses.html
	cargo build --release --no-default-features --features "prod ${EXTRA_FEATURES}"
	rm -rf target/out target/package.zip
	cargo +nightly -Zscript package-release.rs
.PHONY: package-release

steam-deck:
	bin/container-build.sh
.PHONY: steam-deck

cargo-all-tests:
	cargo check
	cargo check --no-default-features --features "prod ${EXTRA_FEATURES}"
	cargo clippy --features "all-backends"
	cargo build
	cargo test --release --all-targets  # NOTE: needs to be in release. Replays take too long otherwise
.PHONY: cargo-all-tests

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
