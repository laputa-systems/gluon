.PHONY: lint clean test test-ci coverage bench bench-check bench-list

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test: test-ci

test-ci:
	python3 tests/benchmarks/test_driver.py
	bash tests/differential/test-references.sh
	bash tests/differential/test-symbols.sh
	bash tests/differential/test-ninja-observables.sh
	cargo test --profile dist --locked

coverage:
	cargo llvm-cov --workspace --locked --branch --summary-only

clean:
	cargo clean

bench:
	python3 tests/benchmarks/driver.py run $(BENCH_ARGS)

bench-check:
	python3 tests/benchmarks/driver.py check $(BENCH_ARGS)

bench-list:
	python3 tests/benchmarks/driver.py list
