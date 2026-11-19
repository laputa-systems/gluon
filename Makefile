.PHONY: lint clean test test-ci

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test: test-ci

test-ci:
	cargo test --profile dist --locked

clean:
	cargo clean
