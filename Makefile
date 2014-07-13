APP=dose-response
LIB_DIR=lib
SOURCES=$(wildcard src/**/*.rs src/*.rs)
LIBS=$(wildcard lib/*.rlib)
CFLAGS=-C link-args='-Wl,--rpath=$$ORIGIN/lib'
CARGO_RUSTFLAGS?=

build: $(SOURCES) target/deps/libtcod.so
	cargo build
	patchelf --set-rpath '$$ORIGIN/deps/' target/$(APP)

target/deps/libtcod.so: lib/libtcod.so
	@mkdir -p target/deps/ target/deps/libtcod.so
	ln -s -r lib/libtcod.so target/deps/

target/deps/release/libtcod.so: lib/libtcod.so
	@mkdir -p target/deps/release/ target/deps/release/libtcod.so
	ln -s -r lib/libtcod.so target/deps/

release: $(SOURCES) target/deps/release/libtcod.so
	cargo build --release
	patchelf --set-rpath '$$ORIGIN/deps/' target/release/$(APP)

test: $(SOURCES) $(LIBS)
	rustc --test -W ctypes src/tests.rs -o test-$(APP)
	./test-$(APP)

run: release
	./target/release/$(APP)

replay: $(APP)
	./$(APP) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf dist *.pyc $(APP) test-$(APP) lib/librtcod-*.so
