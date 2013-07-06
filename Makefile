run:
	python hedonic-hypothesis.py

test:
	python -m unittest discover

bench:
	python ./benchmark.py

gamebench:
	python -m cProfile -s cumulative ./hedonic-hypothesis.py
