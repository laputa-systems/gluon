# Benchmarks

`make bench-list` lists workloads, tiers, areas and commands. `make bench`
defaults to the Change ceiling. Set `BENCH_ARGS`, for example:

```
make bench-check BENCH_ARGS='--tier Change --name engine'
make bench BENCH_ARGS='--tier Change --area engine --dry-run'
make bench BENCH_ARGS='--tier Edit'
make bench BENCH_ARGS='--name engine --skip controls'
```

`--name` selects workload names; `--area` selects areas. Each can be repeated;
within each kind they are alternatives, while kinds intersect. `--skip` takes
names. Unknown values and empty selections fail. The dry run writes nothing.
Full is manual-only: explicitly request `--tier Full`; it includes the
Scenario corpus, which currently fails preflight. The driver checks selected
executables and disk space before running anything. Cargo builds its own
artifacts from the pinned toolchain and lockfile. The CLI corpus is blocked
until DSLP-11 gives it reliable build failure and output checks. No external
service is used. Host readiness and leftover processes are not checked; these runs do not
establish a timing gate.

| Workload | Tier / area | Purpose, requirements and estimated cost |
| --- | --- | --- |
| controls | Edit / harness | Selection and preflight controls; Python 3; seconds. |
| engine | Change / engine | Queue operations; Cargo; minutes on a cold build. |
| graph | Change / graph | Interning, arena and producer lookup; Cargo; minutes cold. |
| state | Change / state | Log, dependency and snapshot parsing/I/O; Cargo and writable TMPDIR; minutes cold. |
| meson | Change / meson | Lexer/parser snippets; Cargo; minutes cold. |
| ninja | Change / ninja | Synthetic manifest parsing, planning, expansion and hashing; Cargo; minutes cold. |
| cli-corpus | Scenario / cli | Meson setup and Ninja dry-run/rebuild; pinned references, compiler and release binaries; tens of minutes estimated; blocked by DSLP-11. |
| all-components | Full / components | Entire Rustybench workspace; Cargo; minutes estimated. |

Commands are printed by `bench-list` and `--dry-run`. Runs write logs and a
summary under `.work/bench/<date-time>/`; workload exit codes and exclusions
are recorded. Rustybench timing and allocation measurements are diagnostic,
not an accepted performance verdict. The scripts do not compare against an
accepted revision or verify representative completed work. For a performance
gate use a separately verified paired workload and `perf-gate-ready` before
and `perf-gate-ready -1` immediately afterwards; discard overlapping runs.

For diagnosis, run platform CPU sampling (for example Instruments or Linux
perf with child processes), an allocator profiler, and syscall/I/O tracing
separately from timed measurements. Their overhead and process scope must be
recorded. The existing RSS is not a simultaneous workload-tree peak.
