replay:
	cargo run -- `find replays -type f -name 'replay-*' | sort | tail -n 1`

release:
	scripts/prep-release.sh

.PHONY: replay release
