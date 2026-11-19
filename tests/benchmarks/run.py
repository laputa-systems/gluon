from __future__ import annotations

import filecmp
import hashlib
import os
import shlex
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PAIRS = 5
SIZES = (1000, 10000, 100000)
ENV = {**os.environ, "LC_ALL": "C", "NINJA_STATUS": "[%f/%t] "}


@dataclass(frozen=True)
class Tools:
    meson: Path
    ninja: Path


@dataclass(frozen=True)
class Sample:
    wall: float
    cpu: float
    rss_mib: float


def tool(name: str) -> Path:
    path = shutil.which(name)
    if path is None:
        raise RuntimeError(f"{name} is required")
    return Path(path).resolve()


def check(status: int, output: str, command: list[str | Path]) -> str:
    if status:
        raise RuntimeError(f"{shlex.join(map(str, command))} exited {status}\n{output.rstrip()}")
    return output.strip()


def run(cwd: Path, *command: str | Path) -> str:
    result = subprocess.run(command, check=False, cwd=cwd, env=ENV, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True)
    return check(result.returncode, result.stdout, list(command))


def timed(cwd: Path, *command: str | Path) -> tuple[Sample, str]:
    with tempfile.TemporaryFile() as log:
        start = time.perf_counter()
        with subprocess.Popen(command, cwd=cwd, env=ENV, stdout=log, stderr=log) as child:
            _, status, usage = os.wait4(child.pid, 0)
            returncode = os.waitstatus_to_exitcode(status)
            child.returncode = returncode
        wall = time.perf_counter() - start
        log.seek(0)
        output = check(returncode, log.read().decode(errors="replace"), list(command))
    rss_unit = 1 if sys.platform == "darwin" else 1024
    return Sample(wall, usage.ru_utime + usage.ru_stime, usage.ru_maxrss * rss_unit / 1024**2), output


def load_engines() -> tuple[Tools, Tools]:
    pins = dict(line.split("=", 1) for line in (ROOT / "tests/upstream/PINS").read_text().splitlines()
                if line and not line.startswith("#"))
    reference = Tools(tool("meson"), tool("ninja"))
    candidate = Tools(
        Path(os.environ.get("GLUON_MESON", ROOT / "target/dist/meson")).resolve(),
        Path(os.environ.get("GLUON_NINJA", ROOT / "target/dist/ninja")).resolve(),
    )
    for name in ("meson", "ninja"):
        upstream = getattr(reference, name)
        gluon = getattr(candidate, name)
        if run(ROOT, upstream, "--version") != pins[f"{name.upper()}_VERSION"]:
            raise RuntimeError(f"unexpected reference {name} version at {upstream}")
        if filecmp.cmp(upstream, gluon, shallow=False):
            raise RuntimeError(f"reference {name} matches the candidate")
        run(ROOT, gluon, "--version")
        print(f"{name} reference {upstream} · candidate {gluon}", flush=True)
    return reference, candidate


def witness(work: Path, value: int) -> None:
    found = run(work, "./app")
    if found != str(value):
        raise RuntimeError(f"{work}: expected app output {value}, got {found!r}")


def idle(work: Path, output: str) -> None:
    if output != "ninja: no work to do.":
        raise RuntimeError(f"{work}: expected an idle build, got {output!r}")


def meson_pair(work: Path, engines: Tools, *, reference: bool) -> tuple[Sample, Sample]:
    src = work / "src"
    src.mkdir(parents=True)
    (src / "meson.build").write_text(
        "project('example', 'c')\n"
        "if get_option('buildtype') == 'debug'\n"
        "  add_project_arguments('-DVALUE=2', language: 'c')\n"
        "else\n"
        "  add_project_arguments('-DVALUE=1', language: 'c')\n"
        "endif\n"
        "executable('app', 'main.c')\n"
    )
    (src / "main.c").write_text('#include <stdio.h>\nint main(void) { printf("%d\\n", VALUE); }\n')
    build = work / "build"
    setup, output = timed(work, engines.meson, "setup", "--buildtype=release", build, src)
    if reference and "oxidized-meson" in output:
        raise RuntimeError("reference setup ran Gluon Meson")
    run(build, engines.ninja, "-j2")
    witness(build, 1)
    reconfigure, _ = timed(work, engines.meson, "setup", "--reconfigure", "--buildtype=debug", build, src)
    run(build, engines.ninja, "-j2")
    witness(build, 2)
    idle(build, run(build, engines.ninja, "-j2"))
    return setup, reconfigure


def graph(work: Path, size: int, modules: int) -> list[Path]:
    work.mkdir(parents=True)
    (work / "value.h").write_text("#define OFFSET 1\n")
    (work / "main.c").write_text(
        '#include <stdio.h>\n#include "value.h"\n'
        + "".join(f"int module_{i}(void);\n" for i in range(modules))
        + 'int main(void) { printf("%d\\n", OFFSET'
        + "".join(f" + module_{i}()" for i in range(modules))
        + "); }\n"
    )
    objects = ["main.o"]
    with (work / "build.ninja").open("w") as manifest:
        manifest.write(
            "rule cc\n  command = cc -MMD -MF $out.d -c $in -o $out\n"
            "  depfile = $out.d\n  deps = gcc\n  description = compile $out\n"
            "rule link\n  command = cc $in -o $out\n  description = link $out\n"
            "build main.o: cc main.c\n"
        )
        for index in range(modules):
            stem = f"module_{index}"
            (work / f"{stem}.c").write_text(f"int {stem}(void) {{ return {index}; }}\n")
            objects.append(f"{stem}.o")
            manifest.write(f"build {stem}.o: cc {stem}.c\n")
        manifest.write(f"build app: link {' '.join(objects)}\n")
        aliases = [f"check_{index}" for index in range(size - len(objects) - 2)]
        for index, alias in enumerate(aliases):
            manifest.write(f"build {alias}: phony {objects[index % len(objects)]}\n")
        manifest.write(f"build all: phony app {' '.join(aliases)}\ndefault all\n")
    return [work / name for name in ["app", ".ninja_log", ".ninja_deps", *objects]]


def snapshot(paths: list[Path]) -> list[tuple[int, bytes]]:
    return [(p.stat().st_mtime_ns, hashlib.sha256(p.read_bytes()).digest()) for p in paths]


def incremental_pair(work: Path, ninja: Path) -> tuple[Sample, Sample]:
    graph(work, 8, 2)
    run(work, ninja, "-j2")
    witness(work, 2)
    idle(work, run(work, ninja, "-j2"))
    (work / "module_1.c").write_text("int module_1(void) { return 5; }\n")
    source, output = timed(work, ninja, "-j2")
    if [line.split("] ", 1)[-1] for line in output.splitlines()] != ["compile module_1.o", "link app"]:
        raise RuntimeError(f"{work}: unexpected source rebuild commands\n{output}")
    witness(work, 6)
    (work / "value.h").write_text("#define OFFSET 10\n")
    header, output = timed(work, ninja, "-j2")
    if [line.split("] ", 1)[-1] for line in output.splitlines()] != ["compile main.o", "link app"]:
        raise RuntimeError(f"{work}: unexpected header rebuild commands\n{output}")
    witness(work, 15)
    idle(work, run(work, ninja, "-j2"))
    return source, header


def bench(work: Path, engines: tuple[Tools, Tools]) -> dict[str, list[list[Sample]]]:
    reports: dict[str, list[list[Sample]]] = {}
    labels = ("meson setup", "meson reconfigure", "ninja rebuild source", "ninja rebuild header")
    reports.update((label, [[], []]) for label in labels)
    for pair in range(PAIRS):
        for side in (pair % 2, 1 - pair % 2):
            setup = meson_pair(work / f"meson-{pair}-{side}", engines[side], reference=side == 0)
            rebuild = incremental_pair(work / f"incremental-{pair}-{side}", engines[side].ninja)
            for label, sample in zip(labels, setup + rebuild):
                reports[label][side].append(sample)
    for size in SIZES:
        key = f"ninja warm no-op ({size} edges)"
        reports[key] = [[], []]
        modules = 32
        warm = [work / f"warm-{size}-{side}" for side in range(2)]
        files = [graph(path, size, modules) for path in warm]
        for path, engine in zip(warm, engines):
            run(path, engine.ninja, "-j2")
            witness(path, 1 + sum(range(modules)))
        initial = [snapshot(paths) for paths in files]
        for pair in range(PAIRS):
            for side in (pair % 2, 1 - pair % 2):
                sample, output = timed(warm[side], engines[side].ninja, "-j2")
                idle(warm[side], output)
                reports[key][side].append(sample)
        if initial != [snapshot(paths) for paths in files]:
            raise RuntimeError(f"{key}: output or build log changed on a no-op")
    return reports


def main() -> int:
    try:
        engines = load_engines()
        if subprocess.run(["perf-gate-ready"], check=False, cwd=ROOT).returncode:
            print("perf gate deferred: host busy", file=sys.stderr)
            return 75
        scratch = ROOT / ".work/tmp"
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(prefix="bench-", dir=scratch) as directory:
            reports = bench(Path(directory), engines)
            if subprocess.run(["perf-gate-ready", "-1"], check=False, cwd=ROOT).returncode:
                print("perf gate deferred: host busy after timing", file=sys.stderr)
                return 75
    except (OSError, RuntimeError, ValueError) as error:
        print(f"benchmarks: FAIL: {error}", file=sys.stderr)
        return 1

    for label, runs in reports.items():
        print(label)
        for name, samples in zip(("reference", "candidate"), runs):
            wall = [s.wall for s in samples]
            print(f"  {name}: wall {statistics.median(wall):.4f} s "
                  f"({min(wall):.4f}–{max(wall):.4f}), "
                  f"CPU {statistics.median(s.cpu for s in samples):.4f} s, "
                  f"peak RSS {max(s.rss_mib for s in samples):.1f} MiB")
    print(f"benchmarks: PASS ({PAIRS} pairs per workload, outputs verified)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
