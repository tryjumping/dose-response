APP=dose-response
LIB_DIR=lib
SOURCES=$(wildcard src/**/*.rs src/*.rs)
CFLAGS=-C link-args='-Wl,--rpath=$$ORIGIN/lib'
CARGO_RUSTFLAGS?=

all: $(APP)

deps:
	cargo-lite build

# Don't call directly -- will be invoked by cargo-lite from deps
bin:
	rustc -W ctypes -Llib $(CFLAGS) $(CARGO_RUSTFLAGS) src/main.rs -o $(APP)

test: $(SOURCES)
	rustc --test -W ctypes src/main.rs -o test-$(APP)
	./test-$(APP)

$(APP): $(SOURCES) deps

run: $(APP)
	./$(APP)

replay: build
	./$(APP) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf dist *.pyc $(APP) test-$(APP) lib/librtcod-*.so
