all: build

run: build
	LD_LIBRARY_PATH="./lib" ./dose-response

build: dose-response

dose-response:
	rust build -W ctypes -L./lib -O main.rs -o dose-response

replay:
	LD_LIBRARY_PATH="./lib" ./dose-response `find replays -type f -name 'replay-*' | sort | tail -n 1`

clean:
	rm -rf dist *.pyc dose-response-rust lib/librtcod-*.so

test-py:
	python test_entity_component_manager.py

bench-py:
	python ./benchmark.py all artemis
