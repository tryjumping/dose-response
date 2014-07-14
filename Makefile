APP=dose-response
SOURCES=$(wildcard src/**/*.rs src/*.rs)

all: build

# libtcod.so must exist in target/*/deps before the build starts.
# This is called by cargo-build's `build` directive.
lib-symlinks:
	@mkdir -p target/deps/
	test ! -e target/deps/libtcod.so && \
		ln -s -r lib/libtcod.so target/deps/ || true
	@mkdir -p target/release/deps/
	test ! -e target/release/deps/libtcod.so && \
		ln -s -r lib/libtcod.so target/release/deps/ || true

build: $(SOURCES)
	cargo build

release: $(SOURCES)
	cargo build --release

run: release
	./target/release/$(APP)

replay: target/release/$(APP)
	./target/release/$(APP) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf target
