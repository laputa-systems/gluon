.PHONY: lint clean test-ci

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test-ci:
	cargo test --profile dist --locked

clean:
	cargo clean
