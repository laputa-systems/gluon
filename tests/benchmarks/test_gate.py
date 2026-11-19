from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path


class BenchmarkGate(unittest.TestCase):
    def run_gate(self, meson: str, ninja: str) -> subprocess.CompletedProcess[str]:
        source = Path(__file__).resolve().parent
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            benchmark = root / "tests/benchmarks"
            benchmark.mkdir(parents=True)
            for script in ("run.sh", "run.py"):
                if (source / script).exists():
                    shutil.copy(source / script, benchmark)
            pins = root / "tests/upstream"
            pins.mkdir()
            (pins / "PINS").write_text("MESON_VERSION=fixture\nNINJA_VERSION=fixture\n")
            tools = root / "tools"
            tools.mkdir()
            release = root / "target/dist"
            release.mkdir(parents=True)
            for name, body in (("meson", meson), ("ninja", ninja)):
                for directory in (tools, release):
                    tool = directory / name
                    tool.write_text(
                        '#!/bin/sh\nif [ "$1" = --version ]; then echo fixture; exit 0; fi\n'
                        f"{body}\n# {directory}\n"
                    )
                    tool.chmod(0o755)
            gate = tools / "perf-gate-ready"
            gate.write_text("#!/bin/sh\nexit 0\n")
            gate.chmod(0o755)
            env = {
                **os.environ, "PATH": f"{tools}:{os.environ['PATH']}",
                "GLUON_MESON": str(release / "meson"), "GLUON_NINJA": str(release / "ninja"),
            }
            return subprocess.run(
                ["bash", benchmark / "run.sh"], check=False, env=env,
                capture_output=True, text=True, timeout=10,
            )

    def test_failed_setup_cannot_report_a_pass(self) -> None:
        result = self.run_gate("echo injected-meson-failure >&2\nexit 7", "exit 0")
        output = result.stdout + result.stderr
        self.assertEqual(result.returncode, 1, output)
        self.assertIn("injected-meson-failure", output)
        self.assertNotIn("benchmarks: PASS", output)

    def test_success_without_the_expected_artifact_fails(self) -> None:
        result = self.run_gate('mkdir "$3"', "exit 0")
        output = result.stdout + result.stderr
        self.assertEqual(result.returncode, 1, output)
        self.assertIn("./app", output)
        self.assertNotIn("benchmarks: PASS", output)


if __name__ == "__main__":
    unittest.main()
