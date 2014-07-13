APP=dose-response
LIB_DIR=lib
SOURCES=$(wildcard src/**/*.rs src/*.rs)
LIBS=$(wildcard lib/*.rlib)
CFLAGS=-C link-args='-Wl,--rpath=$$ORIGIN/lib'
CARGO_RUSTFLAGS?=

build: $(SOURCES)
	cargo build
	patchelf --set-rpath '$$ORIGIN/deps/' target/$(APP)

test: $(SOURCES) $(LIBS)
	rustc --test -W ctypes src/tests.rs -o test-$(APP)
	./test-$(APP)

run: build
	./target/$(APP)

replay: $(APP)
	./$(APP) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf dist *.pyc $(APP) test-$(APP) lib/librtcod-*.so
