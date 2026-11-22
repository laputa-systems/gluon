.PHONY: lint clean test test-ci coverage bench bench-check bench-list

lint:
	cargo clippy --workspace --all-targets -- -D warnings

test: test-ci

test-ci:
	PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s tests/benchmarks -p 'test_*.py'
	bash tests/differential/test-references.sh
	bash tests/differential/test-symbols.sh
	bash tests/differential/test-ninja-observables.sh
	cargo test --profile dist --locked

coverage:
	cargo llvm-cov --workspace --locked --branch --summary-only

clean:
	@for artifact in ./target; do \
	  if test -L "$$artifact" || { test -e "$$artifact" && ! test -d "$$artifact"; }; then \
	    printf '%s\n' "make clean: $$artifact must be a local build directory" >&2; exit 1; \
	  fi; \
	done
	rm -rf -- ./target

bench:
	python3 tests/benchmarks/driver.py run $(BENCH_ARGS)

bench-check:
	python3 tests/benchmarks/driver.py check $(BENCH_ARGS)

bench-list:
	python3 tests/benchmarks/driver.py list
