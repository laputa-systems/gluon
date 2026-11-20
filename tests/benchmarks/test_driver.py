import os
import shutil
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

    def test_corpus_rejects_failed_build_commands(self):
        scratch = ROOT / '.work/tmp'
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            root = pathlib.Path(directory)
            scripts = root / 'tests/benchmarks'
            scripts.mkdir(parents=True)
            shutil.copy(ROOT / 'tests/benchmarks/run.sh', scripts / 'run.sh')
            binaries = root / 'target/release'
            binaries.mkdir(parents=True)
            fakebin = root / 'bin'
            fakebin.mkdir()
            for path in [fakebin / 'meson', fakebin / 'ninja', binaries / 'meson', binaries / 'ninja']:
                path.write_text('#!/bin/sh\nexit 7\n')
                path.chmod(0o755)
            for name in ['perf-gate-ready', 'python3', 'sleep']:
                path = fakebin / name
                path.write_text('#!/bin/sh\nexit 0\n')
                path.chmod(0o755)
            env = os.environ.copy()
            env['PATH'] = str(fakebin) + os.pathsep + env['PATH']
            result = subprocess.run(['bash', str(scripts / 'run.sh')], text=True, capture_output=True, env=env)
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertNotIn('benchmarks: PASS', result.stdout)
            self.assertIn('meson setup', result.stderr)

    def test_corpus_rejects_ignored_header_changes(self):
        scratch = ROOT / '.work/tmp'
        scratch.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=scratch) as directory:
            root = pathlib.Path(directory)
            scripts = root / 'tests/benchmarks'
            scripts.mkdir(parents=True)
            shutil.copy(ROOT / 'tests/benchmarks/run.sh', scripts / 'run.sh')
            binaries = root / 'target/release'
            binaries.mkdir(parents=True)
            fakebin = root / 'bin'
            fakebin.mkdir()
            ninja = """#!/usr/bin/python3
import hashlib, os, pathlib, subprocess, sys
args = sys.argv[1:]
if '-n' in args:
    sys.exit(0)
d = pathlib.Path(args[args.index('-C') + 1]) if '-C' in args else pathlib.Path.cwd()
src = d / 'a.c'
state = d / '.source-hash'
digest = hashlib.sha256(src.read_bytes()).hexdigest() + str(src.stat().st_mtime_ns)
if os.environ.get('TRACK_HEADER') and (d / 'a.h').exists():
    digest += hashlib.sha256((d / 'a.h').read_bytes()).hexdigest()
if state.exists() and state.read_text() == digest:
    print('ninja: no work to do.')
    sys.exit(0)
line = next(line for line in (d / 'build.ninja').read_text().splitlines() if line.strip().startswith('command ='))
command = line.split('=', 1)[1].strip().replace('$in', 'a.c').replace('$out', 'a.o')
subprocess.run(command, cwd=d, shell=True, check=True)
state.write_text(digest)
print('[1/1] compile')
"""
            meson = "#!/bin/sh\nif [ $# -ge 3 ]; then mkdir -p \"$2\"; touch \"$2/build.ninja\"; else mkdir -p builddir; touch builddir/build.ninja; fi\n"
            for path in [fakebin / 'ninja', binaries / 'ninja']:
                path.write_text(ninja)
                path.chmod(0o755)
            for path in [fakebin / 'meson', binaries / 'meson']:
                path.write_text(meson)
                path.chmod(0o755)
            for name, script in {
                'perf-gate-ready': 'exit 0',
                'sleep': 'exit 0',
                'python3': 'if [ "$1" = - ]; then mkdir -p "$2"; touch "$2/build.ninja" "$2/o1.o" "$2/o$3.o"; fi; exit 0',
            }.items():
                path = fakebin / name
                path.write_text('#!/bin/sh\n' + script + '\n')
                path.chmod(0o755)
            env = os.environ.copy()
            env['PATH'] = str(fakebin) + os.pathsep + env['PATH']
            result = subprocess.run(['bash', str(scripts / 'run.sh')], text=True, capture_output=True, env=env)
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertIn('header', result.stderr)
            env['TRACK_HEADER'] = '1'
            result = subprocess.run(['bash', str(scripts / 'run.sh')], text=True, capture_output=True, env=env)
            self.assertEqual(result.returncode, 0, result.stderr)

    def test_corpus_preflight_is_available(self):
        result = self.invoke('check', '--tier', 'Scenario', '--name', 'cli-corpus')
        self.assertEqual(result.returncode, 0, result.stderr)



if __name__ == '__main__':
    unittest.main()
