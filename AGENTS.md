# AGENTS.md

gluon is a Rust workspace implementing Meson- and Ninja-compatible build
tools (`meson`, `ninja`, `gluon-build` in `bins/`). Library crates live in
`crates/`; differential tests in `tests/`. Details:
[docs/architecture.md](docs/architecture.md),
[docs/decisions.md](docs/decisions.md), [docs/testing.md](docs/testing.md),
[docs/status.md](docs/status.md).
Open defects: [TODO.md](TODO.md).

Build: `cargo build --profile dist --locked`
Test: `cargo test --profile dist --locked`
Parity gate: `tests/differential/run.sh` (empty ledger)
Benchmarks: `tests/benchmarks/run.sh` (needs `perf-gate-ready`)

## Verification

- `make test` and the hosted `make test-ci` run `cargo test --profile dist --locked`.
  The macOS ARM64 baseline passed 85 top-level tests and one filtered child.
  Tests took 278.19 s with an empty target and 4.81 s warm. Clippy passed in
  291.12 s. These suite costs include shared host load.
- Meson and Ninja CLI unit tests exercise selected arguments, exit paths, and help.
  The installed help and differential fixtures separately check user-facing behavior.
- Crate unit tests and the engine equivalence test cover public Rust behavior and
  generated Ninja manifests. Install and introspection JSON have selected CLI
  tests and separate install fixtures.
- Ninja manifest, depfile, log, dependency and snapshot tests live in their owning
  crates. The differential fixtures cover build and rebuild behavior.
- FFmpeg-build and Laputa consume the library. Their tests cover downstream API
  changes. The Ground check ran the owning suite.
- The performance suite covers setup, reconfigure, Ninja dry runs and incremental
  builds. Qualify it for affected runtime changes. [Testing](docs/testing.md)
- The Meson and Ninja notices and `tests/upstream/PINS` record imported revisions.

Invariants:

- User builds run in `crates/ninja`. `crates/engine` must not depend on
  Meson semantics. Meson setup writes `build.ninja` and does not execute
  the typed graph.
- Parity is observable behavior (artifacts, command sets, rebuild behavior,
  exit codes), not text; `tests/differential/expected-fail` lists divergences
  one per line with a one-line cause, and a listed fixture that passes fails
  the run.
- Upstream versions and import SHAs are pinned in `tests/upstream/PINS`;
  do not bump without differential tests.
- Licenses stay permissive-only: MIT/Apache-2.0 texts with NOTICEs per
  crate; Muon is a specification source, never copied code.
