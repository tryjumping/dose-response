run:
	./dose-response

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
	rm -rf dist *.pyc
