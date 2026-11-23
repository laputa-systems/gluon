Prefix: GLU
Next: 42

### GLU-19 · P3 · shared libraries have no symbol-extractor edge

- **Area:** meson
- **Where:** `crates/meson/src/backend.rs`, `tests/differential/fixtures/meson/shared-args/skip-edge-count`
- **Problem:** Meson 1.12.1 adds a `SHSYM` edge per shared library so relinks skip dependents whose exported symbols did not change; gluon emits none, so the `gnu-symbol-visibility` and `shared-args` fixtures skip the edge-count comparison.
- **Recommendation:** Emit the symbol-extractor edge and make dependents' links depend on the `.symbols` output, then delete the `skip-edge-count` markers.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-20 · P3 · install accepts options it does not apply

- **Area:** meson
- **Where:** `crates/meson/src/cli.rs`, `crates/meson/src/install.rs`
- **Problem:** `meson install --tags` and `--skip-subprojects` are parsed and silently ignored, and `configure_file`'s `install_mode` never reaches the install plan, so those installs get default modes.
- **Recommendation:** Apply each as Meson 1.12.1's `minstall.py` does, or reject it with an error until it is implemented.
- **Fix with:** reference-parity
- **Effort:** S

### GLU-21 · P3 · shared-library aliases are missing from the build directory

- **Area:** meson
- **Where:** `crates/meson/src/backend.rs`, `crates/meson/src/install.rs`
- **Problem:** Meson creates `libfoo.so`/`libfoo.dylib` and `libfoo.so.<soversion>` symlinks in the build directory at setup (`generate_shlib_aliases`). gluon creates them only in the install plan, so build-tree consumers that open the unversioned name fail and `intro-installed.json` and `build all` disagree with Meson about the build outputs.
- **Recommendation:** Create the same aliases next to the versioned build output at setup, matching `SharedLibrary.get_aliases()`.
- **Fix with:** reference-parity
- **Effort:** S

### GLU-22 · P3 · install does not rewrite dependent load paths

- **Area:** meson
- **Where:** `crates/meson/src/install.rs`
- **Problem:** ELF RPATH/RUNPATH is rewritten in process and `install_name_tool -id` is applied to installed dylibs, but Meson also runs `install_name_tool -change` from `get_link_deps_mapping()` for build targets the library links. A macOS library that links another target of the same project keeps the build-tree install name after install.
- **Recommendation:** Pass the linked targets' install-name mappings to `install_name_tool`.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-23 · P3 · pkgconfig ignores library targets in requires

- **Area:** meson
- **Where:** `crates/meson/src/modules/pkgconfig.rs`
- **Problem:** Meson maps a library target passed to `requires`/`requires_private` to the `filebase` of the `.pc` generated for it (and errors otherwise). gluon only handles strings and dependency objects, silently dropping library objects.
- **Recommendation:** Track generated `.pc` metadata per target id so `DependenciesHelper` can recurse as Meson does.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-24 · P3 · uninstalled pkgconfig file is approximate

- **Area:** meson
- **Where:** `crates/meson/src/modules/pkgconfig.rs`
- **Problem:** the `meson-uninstalled/*-uninstalled.pc` Cflags are derived from each target's `include_directories` plus subdir. Meson builds them from `IncludeDirs.incdirs` plus `extra_build_dirs` for every processed library and nested link target, so paths that come from implicit or dependency include directories can be missing.
- **Recommendation:** Record the uninstalled include directories while processing the dependency graph, as `DependenciesHelper._add_uninstalled_incdirs` does.
- **Fix with:** reference-parity
- **Effort:** S

### GLU-26 · P3 · find_library with dirs skips the link check and system directories

- **Area:** meson
- **Where:** `crates/meson/src/compilers.rs` (`find_library`)
- **Problem:** with `dirs:` Meson tries each naming pattern in the given directories and then in the compiler's system library directories, and link-checks each candidate (except static-only requests). gluon only checks whether the file exists in `dirs` and returns the first match, so a library present only in a system directory is reported missing, and a file that does not link is accepted.
- **Recommendation:** Mirror `_find_library_real`: search `extra_dirs` followed by `get_library_dirs()` and probe candidates by linking a shared library.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-31 · P3 · differential no-op check counts the original build's edges

- **Area:** tests
- **Where:** `tests/differential/run.sh:213-227`
- **Problem:** After the no-op rebuild, the harness compares edge counts from `build.log` and `cand_build.log` (the first builds) instead of `noop.log` and `cand_noop.log`, so a candidate that reruns every edge on every invocation passes the no-op contract.
- **Recommendation:** Count edges in the no-op logs and require zero for both, as Meson's no-op contract states. Acceptance: a candidate that rebuilds one edge on a no-op run fails the fixture.
- **Fix with:** test-audit
- **Effort:** S

### GLU-32 · P3 · differential harness passes fixtures where both builds fail

- **Area:** tests
- **Where:** `tests/differential/run.sh:118-128`, `tests/differential/run.sh:139-144`
- **Problem:** Matching nonzero setup or build exits are reported as `PASS ... both-fail`, including for positive fixtures such as a trivial executable, so a missing compiler or another shared environmental failure makes the parity gate pass without comparing any output.
- **Recommendation:** Allow a both-fail pass only for fixtures marked as expected to fail, and fail positive fixtures whose reference build does not succeed. Acceptance: removing the C compiler from PATH fails the positive fixtures instead of passing them.
- **Fix with:** test-audit
- **Effort:** S

### GLU-33 · P3 · musl build check pins a Docker image that does not exist

- **Area:** tests
- **Where:** `tests/upstream/PINS:14-17`, `tests/upstream/musl.sh:3-18`
- **Problem:** `musl.sh` runs `docker run rust:nightly-2026-09-15`, the `RUST_DOCKER_IMAGE` pin, but Docker Hub's `rust` repository has no dated nightly tags: `docker manifest inspect rust:nightly-2026-09-15` and `rust:nightly-2026-11-09` both fail with not found (2026-11-20), so the static-musl check cannot start. `TOOLCHAIN_CHANNEL`, `CARGO_VERSION` and `RUSTC_VERSION` have no reader and already disagree with `rust-toolchain.toml` (`nightly-2026-11-09`). Laputa's vendored copy patched these lines locally, which its vendoring rule forbids.
- **Recommendation:** Pin an image that exists by digest (for example a `rust` stable slim image) and let rustup install the channel from `rust-toolchain.toml` inside it, then delete the unread toolchain fields from PINS so `rust-toolchain.toml` is the only toolchain pin. Acceptance: `tests/upstream/musl.sh` pulls its image, builds both static binaries and runs their `--version` on a clean host.
- **Fix with:** single-source-of-truth
- **Effort:** S

### GLU-34 · P3 · Ninja depfile parsing corrupts UTF-8 and escaped hashes

- **Area:** Ninja dependency paths
- **Where:** crates/state/src/depfile.rs:63,84
- **Problem:** Compile with cc -MMD and deps = gcc while including café.h or name#part.h. Both Gluon builds recompile on a warm invocation. Ninja 1.13.2 stays idle and recompiles after an actual edit. The parser converts UTF-8 a byte at a time and treats the escaped hash as a comment.
- **Recommendation:** Preserve path bytes and consume Make depfile escapes before interpreting comments. Check a warm no-op and a header edit.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-35 · P3 · Meson target introspection omits custom targets

- **Area:** Meson introspection
- **Where:** crates/meson/src/backend.rs:92,1741
- **Problem:** Declare custom_target('copy', output: 'data.out', command: [find_program('touch'), '@OUTPUT@']). Reference Meson includes copy in intro-targets.json. Gluon emits the Ninja target and returns an empty target array.
- **Recommendation:** Generate entries for supported custom targets using the owning build data. Compare ID, outputs and type to pinned Meson.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-36 · P3 · Meson setup silently accepts malformed option files

- **Area:** Meson option parsing
- **Where:** crates/meson/src/options.rs:392, crates/meson/src/builtins/functions.rs:427
- **Problem:** Use project('m') in meson.build and an unterminated option( in meson.options. Gluon setup exits zero. Meson 1.12.1 exits one and reports the unexpected end at meson.options:1:8.
- **Recommendation:** Surface read and syntax failures with file and position. Preserve optional and valid option files.
- **Fix with:** error-model
- **Effort:** S

### GLU-37 · P3 · decide whether to retire unused deps log recompaction API

- **Area:** Needs owner · state public API
- **Where:** crates/state/src/deps_log.rs:46,248,256; crates/ninja/src/build/mod.rs:5
- **Problem:** The exported needs_recompaction field has no reader and remains false; recompact is called only by a unit test and close by that method and tests. Removing them changes the state crate API, also re-exported by ninja, and possible external consumers are unknown.
- **Recommendation:** Decide whether consumers may lose these members; if so remove them and the recompact-only test, preserving the supported on-disk format.
- **Fix with:** dead-code-archaeology
- **Effort:** S

### GLU-38 · P3 · measure verified end-to-end build journeys

- **Area:** benchmark coverage
- **Where:** tests/benchmarks/run.sh:57-79; crates/ninja/benches/ninja.rs:23
- **Problem:** Component benchmarks use synthetic data and the CLI corpus lacks equivalent-work witnesses; no trusted workload covers real setup, reconfigure, no-op, source and header rebuilds or consumer-scale throughput and resource usage. Regressions in these journeys can escape measurement.
- **Recommendation:** After DSLP-11, add pinned representative consumer journeys with output and completed-work checks, paired baselines and scoped CPU, memory and I/O measurements.
- **Fix with:** refactor-benchmarks
- **Effort:** L

### GLU-39 · P3 · add attribution for build performance regressions

- **Area:** benchmark observability
- **Where:** tests/benchmarks/run.sh:30-42; crates/engine/benches/engine.rs:2
- **Problem:** The suite reports elapsed time and limited process RSS or allocator counts but has no documented workload-tree CPU, allocation, I/O or trace attribution. A change in a benchmark cannot be localized reliably, and child peaks are not workload-tree peaks.
- **Recommendation:** Document separate diagnostic runs using platform CPU sampling, allocation profiling and syscall or I/O tracing with operation boundaries, including scope and overhead.
- **Fix with:** bench-pyramid
- **Effort:** M

### GLU-40 · P3 · meson.build changes do not regenerate build.ninja

- **Area:** meson · backend
- **Where:** crates/meson/src/backend.rs regeneration edge
- **Problem:** The regeneration edge depends only on command-based configure_file inputs, so editing meson.build does not rerun setup on the next ninja invocation. Meson regenerates when any build definition file changes.
- **Recommendation:** Add the processed meson.build and meson_options files to the regeneration edge inputs and test that editing meson.build regenerates once and then idles.
- **Fix with:** reference-parity
- **Effort:** S

### GLU-41 · P3 · Run Muon as an independent Meson oracle

- **Area:** Tests · Meson semantics
- **Where:** `tests/upstream/PINS`, `tests/differential/run.sh`, `tests/differential/references.sh`, `docs/testing.md`, `docs/decisions.md` (P-3)
- **Problem:** `MUON_VERSION=0.7.0` is already pinned and P-3 calls for informational Muon comparisons, but the differential runner only invokes upstream Meson/Ninja and Gluon. There is no executable third-implementation comparison to investigate disagreements or ambiguous Meson behavior.
- **Recommendation:** Extend the existing harness with an explicitly invoked Muon diagnostic lane over a small shared C fixture subset (options, custom targets, dependency propagation and no-op builds). Reuse the current pin, record immutable source/checksum provenance and validate the reference executable; use Muon's native CLI adapter. Run all three configure frontends with the same pinned upstream Ninja to isolate frontend behavior, preserving the existing Gluon-Ninja parity lane. Set equivalent options explicitly and compare observable statuses, artifacts and rebuild behavior, not generated manifest text. Classify disagreements against the declared Meson compatibility contract and [Muon's documented differences](https://docs.muon.build/differences); agreement by two implementations is not a specification. Keep Muon divergences informational under P-3 and the tool external and test-only; import no GPL implementation or fixture code.
- **Acceptance:** Document a reproducible command and a bounded three-way report with tool provenance, per-case outcomes and explicit unsupported/incomplete reasons on Linux/musl and macOS. Positive reference-stage failures or missing tools in a requested run cannot count as clean coverage, and zero completed comparisons cannot pass. A known Muon difference exercises classification, and an injected changed output proves that comparisons detect a mismatch. Minimize actual disagreements into independently authored regressions with a recorded disposition; a bounded clean result is valid and finding a new defect is not required.
- **Fix with:** reference-parity
- **Effort:** M

### GLU-42 · P3 · compiler commands omit the target private include directory

- **Area:** meson · backend
- **Where:** crates/meson/src/backend.rs compile-edge arguments
- **Problem:** For `project('p', 'c')` and `executable('app', 'main.c')`, reference Meson 1.12.1 includes `-Iapp.p` in the C compile command; gluon omits it. Generated headers in the target private directory are therefore not found through the implicit include path.
- **Reproducer:** Configure that project with each implementation. The reference command begins `cc -Iapp.p -I. -I../src ... -c ../src/main.c`; gluon begins `cc -I<source> -I<build> ... -c <source>/main.c`.
- **Recommendation:** Add the target private directory to compile-edge include arguments and compare the generated command with the pinned Meson reference.
- **Fix with:** reference-parity
- **Effort:** S
