# AGENTS.md

gluon is a Rust workspace implementing Meson- and Ninja-compatible build
tools (`meson`, `ninja`, `gluon-build` in `bins/`). Library crates live in
`crates/`; differential tests in `tests/`. Details:
[docs/architecture.md](docs/architecture.md),
[docs/decisions.md](docs/decisions.md), [docs/testing.md](docs/testing.md),
[docs/status.md](docs/status.md).
Open defects: [TODO.md](TODO.md).

Build: `cargo build --profile dist --locked`
Test and lint: `make test`, `make lint`
Parity gate: `tests/differential/run.sh` (empty ledger)
Benchmarks: `tests/benchmarks/run.sh` (needs `perf-gate-ready`)

## Verification

- `make test` and hosted `make test-ci` run the differential checks and the locked dist tests.
  The [backlog report](TODO-DESLOP.md#report) records measured suite costs.
- Meson and Ninja CLI unit tests exercise selected arguments, exit paths, and help.
  The installed help and differential fixtures separately check user-facing behavior.
- Crate unit tests and the engine equivalence test cover public Rust behavior and
  generated Ninja manifests. Install and introspection JSON have selected CLI
  tests and separate install fixtures.
- Ninja manifest, depfile, log, dependency and snapshot tests live in their owning
  crates. The differential fixtures cover build and rebuild behavior.
- FFmpeg-build, Laputa and Play exercise the command-line tools. See the
  [consumer checks](docs/testing.md#consumer-checks).
- The Meson and Ninja notices and `tests/upstream/PINS` record imported revisions.

Invariants:

- First-party Rust crates and tests inherit the workspace's unsafe-code prohibition.
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
