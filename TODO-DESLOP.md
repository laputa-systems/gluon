Prefix: DSLP
Next: 6

Deslop campaign backlog ([plan](https://github.com/laputa-systems/stuff/blob/main/docs/campaigns/deslop.md), tier B). One agent works this file with the `deslop` skill: take the lowest-numbered open ticket whose area does not start with `Needs owner ·`. After DSLP-5, work the open P1 and P2 findings, then fill in `## Report` below.

## Report

Ground at `2a1bca45da57bc36da9a60e1666bd392c9fe011f` used
macOS ARM64 and pinned nightly 2026-11-09. The default Make goal
remains lint. The new `make test` alias runs the locked dist suite
used by the existing PR job. The normal workflow also runs on main.

With `TMPDIR="$PWD/.work/tmp"`, `CARGO_BUILD_JOBS=2`, and an
isolated `CARGO_TARGET_DIR`, both `make test` runs passed 85
top-level cases and one filtered child case. The empty-target run
took 278.19 s and the repeat took 4.81 s. Clippy passed in 291.12 s.
These observations include shared host load and compiler caching.
Cargo reported existing warnings about the workspace edition and
the `glob` dependency.

The workspace has eight packages. Tracked Rust and Python files
contain 32,758 and 397 physical lines respectively, including
tests and comments. [Verification](AGENTS.md#verification)
records the owning boundaries and both fleet consumers. The
existing benchmark suite remains the workload for build hot paths.
Consumer, differential, install and release performance gates
remain separate work.

The differential harness review reproduced the self-comparison problem
from GLU-28. The installed gluon binaries declared the same versions as
upstream. With those binaries first on `PATH`, the prior harness accepted
them as its own references. The preflight now checks the pinned versions,
rejects matching binaries, and keeps the resolved reference paths for the
fixture run. `make test-ci` exercises the preflight fixtures.

The updated harness passed with Homebrew Meson 1.12.1 and Ninja 1.13.2
and the installed `release-fa8b6aa` candidate. Six cases still shared a
build or setup failure. Among them, `subninja` used a child-scoped rule
from the parent, both Wayland fixtures lacked dependencies, and
`llvm-config-tool` failed at link time. The full hosted parity gate and
remaining Diagnose review stay open.

The corrected `subninja` and `llvm-config-tool` fixtures both built with
the installed candidate and pinned upstream tools. The fixture harness
rejected matching failures when the parent used an undefined rule or
the macOS linker received the old `--defsym` flags. Both LLVM products
exported the required module, static link, and system-library markers.
In the full repeat, four fixtures still shared failures. Two required
unavailable Wayland dependencies, and the other two were the intentional
missing install script and missing Ninja dependency cases.

The harness now rejects changed Ninja output bytes, commands during the
idle repeat, and failed idle exits. The shell regression runs each fault
through the fixture runner. Archive symbol comparison ignores member
headings and rejects an `nm` failure even after partial output. The
symbol regression passed with both Apple and GNU `nm`.

The 55-fixture run used the `release-fa8b6aa` binaries and upstream Meson
1.12.1 and Ninja 1.13.2 on macOS ARM64 and Alpine 3.24.2 ARM64. The Linux
Ninja was built from upstream `3441b633`; the PyPI Ninja wheel identifies
itself as a modified build and was rejected. Both Wayland fixtures linked
on Alpine with wayland-dev 1.25.0-r0 and wayland-protocols 1.48-r0.
On macOS those two cases shared missing-dependency exits. On Alpine
`generator-depends` shared a duplicate-symbol failure and
`pkgconfig-generate` shared a missing zlib failure. Both runs also
reported the intentional failing install-script and missing-dependency
fixtures. GLU-28 and the full hosted parity gate stay open.

The generator fixture now writes its declared `dep.txt` and defines its
generated C symbol in one translation unit. On macOS ARM64 and Alpine
3.24.2 ARM64 both binaries produced `dep.txt`, linked an executable
that returned zero, and ran no edges on the repeat build. The idle check
now applies to successful Meson fixtures too. With zlib-dev 1.4-r0 added,
the pkg-config fixture built on Alpine. The full Linux rerun had 53
successful builds and the two intentional failures. The macOS rerun had
51 successful builds, the two intentional failures, and two shared
Wayland setup failures caused by missing dependencies.

## Tickets


### DSLP-2 · P2 · Diagnose: ultrareview into this backlog

- **Area:** deslop · diagnose
- **Where:** whole repository
- **Problem:** No review has ranked this repository's risks for the deslop pass.
- **Recommendation:** Run the deslop Diagnose step. Accept: the diff touches only docs, comments, and this file, and every finding names its Fix with.
- **Fix with:** deslop
- **Effort:** S

### DSLP-3 · P2 · Types: strict profile and typing findings

- **Area:** deslop · types
- **Where:** type checker and lint configuration; Rust/Python sources
- **Problem:** The strict-profile candidates in the deslop skill's tools reference have not been measured here.
- **Recommendation:** Run the deslop Types step. Accept: clean at the adopted profile, escape hatches not increased, witnesses recorded.
- **Fix with:** deslop
- **Effort:** M

### DSLP-4 · P2 · Tests: coverage, boundary gaps, mutation

- **Area:** deslop · tests
- **Where:** test suites and CI
- **Problem:** Coverage and mutation results are unmeasured; no deslop mutation lane exists.
- **Recommendation:** Run the deslop Tests step. Accept: no undispositioned survivor in critical code, the test cost budget holds, and the lane is green where the tools are mirrored.
- **Fix with:** deslop
- **Effort:** M

### DSLP-5 · P2 · Dead code: tools and debloat

- **Area:** deslop · dead code
- **Where:** whole repository
- **Problem:** No dead-code scan has run.
- **Recommendation:** Run the deslop Dead code step. Accept: tools clean or allowlisted with reasons; walk-backs filed as Needs owner tickets.
- **Fix with:** deslop
- **Effort:** M

