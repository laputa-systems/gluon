.PHONY: lint clean test test-ci

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test: test-ci

test-ci:
	bash tests/differential/test-references.sh
	bash tests/differential/test-symbols.sh
	bash tests/differential/test-ninja-observables.sh
	python3 tests/benchmarks/test_gate.py
	cargo test --profile dist --locked

clean:
	cargo clean
