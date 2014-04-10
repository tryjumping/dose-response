APP=dose-response
LIB_DIR=lib
SOURCES=$(wildcard src/**/*.rs src/*.rs) src/components.rs

all: build

build: $(APP)

test: $(SOURCES)
	rustc --test -W ctypes src/main.rs -o test-$(APP)
	./test-$(APP)


src/components.rs: build_ecm.py component_template.rs
	./.venv/bin/python $^ > $@

test_component_codegen:
	python build_ecm.py | rustc --pretty normal - > $@.rs
	rustc --test -W ctypes $@.rs -o $@
	./$@

$(APP): $(SOURCES)
	cargo-lite build
	rustc -W ctypes -O src/main.rs -o $@

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
