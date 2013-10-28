BIN=./bin
APP_NAME=dose-response
APP=$(BIN)/$(APP_NAME)
LIB=./lib
LAUNCHER=./$(APP_NAME)
SOURCES=$(wildcard src/**/*.rs src/*.rs) src/components.rs

all: build

build: $(APP) $(LAUNCHER)

test: $(SOURCES)
	mkdir -p $(BIN)
	rust build --test -W ctypes -L./lib -O src/main.rs -o $(BIN)/test-$(APP_NAME)
	LD_LIBRARY_PATH="$(LIB)" $(BIN)/test-$(APP_NAME)


src/components.rs: build_ecm.py components.rs components_prelude.rs
	cp components_prelude.rs src/components.rs
	python build_ecm.py components.rs | rustc --pretty normal - >> src/components.rs

test_component_codegen:
	python build_ecm.py | rustc --pretty normal - > test_component_codegen.rs
	rustc --test -W ctypes test_component_codegen.rs -o test_component_codegen
	./test_component_codegen

$(APP): $(SOURCES)
	mkdir -p $(BIN)
	rust build -W ctypes -L./lib -O src/main.rs -o $(APP)

$(LAUNCHER):
	echo '#!/bin/bash\nLD_LIBRARY_PATH="$(LIB)" $(APP) $$@' > $(LAUNCHER)
	chmod a+x $(LAUNCHER)

run: build
	$(LAUNCHER)

replay: build
	$(LAUNCHER) `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf dist *.pyc $(BIN) $(LAUNCHER) lib/librtcod-*.so

test-py:
	python test_entity_component_manager.py

bench-py:
	python ./benchmark.py all artemis
