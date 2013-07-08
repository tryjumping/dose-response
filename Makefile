run:
	python hedonic-hypothesis.py

test:
	python test_entity_component_manager.py

bench:
	python ./benchmark.py all artemis

gamebench:
	python -m cProfile -s cumulative ./hedonic-hypothesis.py
