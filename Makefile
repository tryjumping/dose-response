APP=dose-response
LIB_DIR=lib
SOURCES=$(wildcard src/**/*.rs src/*.rs) src/components.rs
CFLAGS=-L$(LIB_DIR) --link-args '-Wl,--rpath=$$ORIGIN/$(LIB_DIR)'

all: build

build: $(APP)

test: $(SOURCES)
	rustc --test -W ctypes $(CFLAGS) src/main.rs -o test-$(APP)
	./test-$(APP)


src/components.rs: build_ecm.py component_template.rs
	./.venv/bin/python build_ecm.py component_template.rs > src/components.rs

test_component_codegen:
	python build_ecm.py | rustc --pretty normal - > test_component_codegen.rs
	rustc --test -W ctypes test_component_codegen.rs -o test_component_codegen
	./test_component_codegen

$(APP): $(SOURCES)
	rustc -W ctypes -O $(CFLAGS) src/main.rs -o $(APP)

run: build
	./$(APP)

replay: build
	./$(APP) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf dist *.pyc $(APP) test-$(APP) lib/librtcod-*.so

test-py:
	python test_entity_component_manager.py

bench-py:
	python ./benchmark.py all artemis
