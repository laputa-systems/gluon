import os
import pathlib
import tempfile
import subprocess
import sys
import unittest

ROOT = pathlib.Path(__file__).resolve().parents[2]
DRIVER = ROOT / 'tests/benchmarks/driver.py'


class DriverTests(unittest.TestCase):
    def invoke(self, *args):
        return subprocess.run([sys.executable, str(DRIVER), *args], cwd=ROOT, text=True, capture_output=True)

    def test_selection_and_dry_run(self):
        before = set((ROOT / '.work/bench').glob('*'))
        result = self.invoke('run', '--tier', 'Change', '--area', 'engine', '--dry-run')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn('engine: cargo bench', result.stdout)
        self.assertIn('excluded cli-corpus: tier ceiling', result.stdout)
        self.assertEqual(before, set((ROOT / '.work/bench').glob('*')))
        for args in [('--name', 'bad'), ('--area', 'bad'), ('--skip', 'bad'), ('--tier', 'Edit', '--name', 'engine'), ('--name', 'engine', '--skip', 'engine')]:
            self.assertNotEqual(self.invoke('run', *args).returncode, 0)

    def test_failed_workload_is_recorded(self):
        scratch = ROOT / '.work/tmp'
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            cargo = pathlib.Path(directory) / 'cargo'
            cargo.write_text('#!/bin/sh\nexit 7\n')
            cargo.chmod(0o755)
            env = os.environ.copy()
            env['PATH'] = directory + os.pathsep + env['PATH']
            result = subprocess.run([sys.executable, str(DRIVER), 'run', '--name', 'engine'], cwd=ROOT, text=True, capture_output=True, env=env)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn('engine: exit=7', result.stdout)
            run_dir = pathlib.Path(result.stdout.split('run directory: ')[1].strip())
            self.assertIn('engine: exit=7', (run_dir / 'summary.txt').read_text())

    def test_failed_preflight_runs_nothing(self):
        before = set((ROOT / '.work/bench').glob('*'))
        result = self.invoke('run', '--tier', 'Scenario', '--name', 'cli-corpus')
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('DSLP-11', result.stderr)
        self.assertEqual(before, set((ROOT / '.work/bench').glob('*')))


if __name__ == '__main__':
    unittest.main()
