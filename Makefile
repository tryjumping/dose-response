SOURCES=$(wildcard src/**/*.rs src/*.rs)

all: build

cargo-build:
	cp lib/* $(OUT_DIR)

build:
	cargo build

test:
	cargo test

release:
	cargo build --release

run:
	cargo run

replay:
	cargo run -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf target
