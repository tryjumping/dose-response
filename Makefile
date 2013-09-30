BIN=./bin
APP_NAME=dose-response
APP=$(BIN)/$(APP_NAME)
LIB=./lib
LAUNCHER=./$(APP_NAME)

all: build

build: $(APP) $(LAUNCHER)

$(APP): $(wildcard src/**/*.rs src/*.rs)
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
