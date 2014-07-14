APP=dose-response
LIB_DIR=lib
SOURCES=$(wildcard src/**/*.rs src/*.rs)
CFLAGS=-C link-args='-Wl,--rpath=$$ORIGIN/lib'
CARGO_RUSTFLAGS?=

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

# Once cargo has the ability to set rpath, this can go away
build: $(SOURCES) lib-symlinks
	cargo build
	patchelf --set-rpath '$$ORIGIN/deps/' target/$(APP)

# Once cargo has the ability to set rpath, this can go away
release: $(SOURCES)
	cargo build --release
	patchelf --set-rpath '$$ORIGIN/deps/' target/release/$(APP)

run: release
	./target/release/$(APP)

replay: target/release/$(APP)
	./target/release/$(APP) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf target
