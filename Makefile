.PHONY: lint clean test test-ci coverage

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test: test-ci

test-ci:
	bash tests/differential/test-references.sh
	bash tests/differential/test-symbols.sh
	bash tests/differential/test-ninja-observables.sh
	cargo test --profile dist --locked

coverage:
	cargo llvm-cov --workspace --locked --branch --summary-only

clean:
	cargo clean
