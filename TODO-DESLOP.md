Prefix: DSLP
Next: 10

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

The runner now requires the expected stage exit status from each tool.
Only the missing install script and missing Ninja input declare a
nonzero exit. The shell regression rejects both an unexpected shared
failure and unexpected success in a failure fixture. Missing pkg-config
prerequisites are explicit skips. The Alpine rerun recorded 55 passes
including the two intentional failures, with zero skips. The macOS
rerun recorded 53 passes including the two intentional failures, and
skipped both Wayland fixtures because pkg-config could not resolve their
dependencies.

The macOS CI runner now builds the candidate and uses the checksum-pinned
Meson wheel and upstream Ninja archive. A local ARM64 run with freshly
built candidates and Python 3.14.8 passed 53 fixtures, including two
intentional failure cases, and skipped the two optional Wayland cases.
A changed wheel checksum stopped the runner before installation. The
hosted workflow pins Python 3.14.10 and rejects any other skipped fixture.

Declared-input validation now runs before dry runs or command dispatch.
Pinned Ninja and the candidate rejected four kinds of missing input in
normal and dry runs, and both still built an unrelated selected target.
Deleting a header listed in a stale depfile still triggered its rebuild.
The differential fixture for a dyndep-generated input verifies the
producer and consumer outputs. Meson now declares the force target its
install rule uses. Reconfiguring a build tree from `release-fa8b6aa`
restored its install with the original setup options.

The macOS corpus passed 54 fixtures and skipped the two Wayland cases.
Both rebuilt static ARM64 binaries passed ELF verification under the
pinned musl toolchain. Alpine 3.24.2 ran all 56 fixtures with zero skips
including the two intentional failures. The time-based performance
gate is deferred because `perf-gate-ready` reported a hot OrbStack
process tree. DSLP-7 through DSLP-9 retain the separate incremental
and generated-metadata failures reproduced on the prior release.

The clean dyndep fixture primes two metadata producers before building
the defaults. The old scheduler returned zero with four files missing.
The candidate produced matching primary, generated, output and separate
files with Ninja 1.13.2, then stayed idle. A phony producer that left
its declared metadata absent returned exit one and produced no consumer
output. The macOS corpus passed 56 fixtures with two Wayland skips.
Stable Alpine passed all 58 with zero skips. Both counts include three
intentional failures.

## Tickets



### DSLP-7 · P1 · Ninja can build from stale generated dyndep metadata

- **Area:** Ninja planning
- **Where:** crates/ninja/src/build/runner.rs, crates/ninja/src/build/dyndep.rs
- **Problem:** Build deps.dd from first.dd with cp. The metadata declares extra as an implicit output of a, and out consumes extra. Bind both edges to deps.dd and default a and out. Prime only deps.dd, then change the command to copy new.dd, which no longer declares extra. Pinned Ninja 1.13.2 exits 1 before dispatch. Gluon release-fa8b6aa and the declared-input fix regenerate deps.dd byte for byte from new.dd but still use the old producer and create out with exit zero. Changing a source file can also leave the old graph in effect.
- **Recommendation:** Check the producing edge before loading generated dyndep metadata and use ready output from the current generation. Verify both command and input changes in normal and dry builds, including a metadata input whose own producer needs work.
- **Fix with:** reference-parity
- **Effort:** L

### DSLP-8 · P1 · Ninja ignores newer declared inputs on incremental builds

- **Area:** Ninja planning
- **Where:** crates/ninja/src/build/runner.rs, tests/differential/fixtures/ninja
- **Problem:** Use a rule with command = cp $in $out and build output: copy input. Build with input containing first, then write second and set input to a newer mtime. Pinned Ninja 1.13.2 rebuilds output to second. Gluon release-fa8b6aa and the declared-input fix report no work and leave first with exit zero.
- **Recommendation:** Compare ordinary input freshness against outputs and recorded build state. Verify explicit and implicit inputs rebuild when newer, order-only inputs preserve ordering without forcing rebuild, and restat and phony edges preserve upstream behavior.
- **Fix with:** reference-parity
- **Effort:** M



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

