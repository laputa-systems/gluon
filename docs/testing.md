# Testing

The differential harness resolves Meson and Ninja from `PATH` once and
checks their versions against [`tests/upstream/PINS`](../tests/upstream/PINS).
It rejects executables that match the gluon candidates and stops if the
reference setup reports `oxidized-meson`. Put the pinned upstream tools
first on `PATH` before running the corpus.

For successful Ninja fixtures, the harness compares file contents. After
each successful Meson or Ninja build, the second Ninja invocation must
exit successfully without executing an edge. `make test-ci` exercises
the file and idle checks with injected failures.
For linked binaries and archives compared by symbols, the harness reads
defined global symbols in portable `nm` format. Archive member headings
are ignored, and an extraction error fails the fixture.

The expected-fail ledger (`tests/differential/expected-fail`) lists
fixtures known to diverge, one per line with a one-line cause. The harness
fails on any unlisted mismatch and on any listed fixture that now passes.
The ledger is empty. Setup and build must exit zero on both sides unless
the fixture declares the expected status in `setup-exit` or `build-exit`.
Intentional failure fixtures cover a missing install script and a missing
Ninja input.

Meson fixtures with `requires-pkg-config` are reported as skipped when
their listed dependencies are unavailable. For a full run, provide zlib,
wayland-client, wayland-protocols and the Wayland scanner. The Alpine
3.24.2 run used `zlib-dev`, `wayland-dev` and `wayland-protocols`.
Check `.work/tmp/diff/results.log` for skips before recording full
coverage.

Gates, narrowest first:

- `cargo test --workspace` — unit/integration suites for all crates.
- `tests/differential/run.sh` — the fixture corpus; exit 0 only with the
  ledger honored.
- `tests/install/run.sh` — focused install fixtures (versioned shared
  library, an executable linked against it, headers in a subdir, pkgconfig)
  installed with reference Meson and with gluon; installed trees and
  RPATH/RUNPATH must match, without rpath, with `install_rpath` and with a
  build-only `build_rpath`.
- `tests/upstream/musl.sh` — static-musl bins via Docker (pinned
  `rust:nightly-2026-09-15`, `--platform linux/amd64`, `x86_64-unknown-linux-musl`) that
  run `--version` in the container. Release CI builds both Linux targets in the
  shared musl toolchain image and checks the binaries with its static ELF
  verifier (see [decisions.md](decisions.md)).
- `tests/benchmarks/run.sh` — cold Meson setup, reconfigure, Ninja `-n` at
  1K/10K/100K edges, and a one-file incremental rebuild. The script prints
  each run. Rustybench harnesses in `crates/*/benches` time parse and plan
  and count allocations. Time-based gates run only when `perf-gate-ready`
  exits 0, and again with `-1` immediately after. A deferred gate is not a
  pass.

Qualification (`~/d/play`, read-only): drive libplacebo 7.387.0 with the
Metal patch and SPIRV-Cross vulkan-sdk 1.4.363.0 through gluon `meson` +
`ninja` per the Meson invocations in play's dependency build script, then link
play's player crates against the outputs with `CARGO_TARGET_DIR` under this
repo's `.work/tmp`.
