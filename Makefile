SOURCES=$(wildcard src/**/*.rs src/*.rs)

all: build

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

prep-release:
	rm -rf .copy
	mkdir -p .copy
	cp -r Cargo.lock Cargo.toml fonts src .copy
