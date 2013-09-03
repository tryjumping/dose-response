run: build
	LD_LIBRARY_PATH="./lib" ./dose-response-rust

build:
	rust build -W ctypes -L./lib -O tcod.rc --lib --out-dir lib
	rust build -W ctypes -L./lib -O main.rs -o dose-response-rust

replay:
	./dose-response `find . -type f -name 'replay-*' | sort | tail -n 1`

test:
	python test_entity_component_manager.py

bench:
	python ./benchmark.py all artemis

gamebench:
	python -m cProfile -s cumulative ./hedonic-hypothesis.py

exe: hedonic-hypothesis.py libtcod.so libtcodgui.so libtcodpy.py
	cxfreeze -OO hedonic-hypothesis.py
	cp libtcod.so libtcodgui.so libtcodpy.py dist
	cp -r fonts dist

clean:
	rm -rf dist *.pyc dose-response-rust lib/librtcod-*.so

rust-bench:
	rust build -L./lib -O tcod_fps_bench.rs -o tcod-fps-bench-rust
	LD_LIBRARY_PATH="./lib" ./tcod-fps-bench-rust
