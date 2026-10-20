"""Exercise the shared installed-artifact smoke without contacting providers."""
from __future__ import annotations

import os
import shutil
import subprocess
import sys
from pathlib import Path

import pytest
import yaml

from cache_fixture import set_cache_root

ROOT = Path(__file__).resolve().parents[1]
HELPER = ROOT / "scripts/installed-wheel-smoke.sh"


def fake_cli(tmp_path: Path, fault: str) -> Path:
    executable = tmp_path / "biomcp"
    executable.write_text(
        f"#!{sys.executable}\nFAULT = {fault!r}\n"
        '''import json, os, pathlib, sys
args = sys.argv[1:]
home = pathlib.Path(os.environ['HOME'])
assert os.environ['BIOMCP_DISABLE_SOURCES'] == 'commercial-restricted,kegg,semantic-scholar'
assert not any(key in os.environ for key in ('NCBI_API_KEY', 'S2_API_KEY', 'BIOMCP_CACHE_DIR', 'XDG_CACHE_HOME'))
with pathlib.Path(__file__).with_suffix('.calls').open('a') as log:
    log.write(json.dumps(args) + '\\n')
if args == ['--version']:
    print('biomcp 0.9.4')
elif args == ['cache', 'stats', '--json']:
    assert not (home / '.cache').exists()
    assert not (home / 'Library' / 'Caches').exists()
    assert not (home / 'AppData' / 'Local').exists()
    if FAULT == 'cold-root':
        print('{"error":{"code":"io","message":"missing cache ancestor"}}')
        print('cache stderr detail', file=sys.stderr)
        sys.exit(1)
    print('{}')
elif args[:2] == ['ddinter', 'sync'] or args[:2] == ['drug', 'interactions']:
    print('DDInter is disabled by the commercial-restricted group')
    sys.exit(1)
elif args[:2] == ['drug', 'adverse-events']:
    print('unexpected FAERS response' if FAULT == 'faers' else 'Drug not found in FAERS')
elif '-j' in args or '--json' in args:
    if FAULT == 'json':
        print('{"error":"json stdout detail"}')
        print('json stderr detail', file=sys.stderr)
        sys.exit(1)
    print('json shape stdout detail' if FAULT == 'json-shape' else '{}')
elif args == ['skill', 'list']:
    print('article-follow-up')
elif args == ['skill', 'article-follow-up']:
    print('Article follow-up via citations and recommendations')
elif args == ['chart', 'bar']:
    print('Bar Chart')
elif FAULT == 'panic':
    print('panic stdout detail')
    print('panic stderr detail', file=sys.stderr)
    sys.exit(101)
else:
    print('ok')
''', encoding="utf-8",
    )
    executable.chmod(0o755)
    return executable


@pytest.mark.parametrize("fault,details", [
    ("cold-root", ("missing cache ancestor", "cache stderr detail")),
    ("json", ("json stdout detail", "json stderr detail")),
    ("json-shape", ("json shape stdout detail", "json mode must print an object")),
    ("panic", ("panic stdout detail", "panic stderr detail")),
    ("faers", ("unexpected FAERS response", "expected a successful FAERS not-found response")),
    ("", ()),
])
def test_installed_smoke_is_cold_keyless_and_propagates_product_failures(
    tmp_path: Path, fault: str, details: tuple[str, ...],
) -> None:
    executable = fake_cli(tmp_path, fault)
    env = set_cache_root(os.environ | {
        "HOME": str(tmp_path), "NCBI_API_KEY": "fixture-only",
        "S2_API_KEY": "fixture-only",
        "XDG_CACHE_HOME": str(tmp_path),
        "BIOMCP_WHEEL_SMOKE_ISOLATED": "" if fault else "1",
    }, tmp_path)
    result = subprocess.run(
        ["bash", str(HELPER), str(executable), "0.9.4"],
        env=env, capture_output=True, text=True, check=False,
    )
    assert result.returncode == (1 if fault else 0), result.stdout + result.stderr
    for detail in details:
        assert detail in result.stdout + result.stderr
    if fault == "cold-root":
        # A broken cache directory must stop before any live source command.
        assert len(executable.with_suffix('.calls').read_text().splitlines()) == 2


def test_main_and_release_use_the_shared_installed_wheel_smoke() -> None:
    makefile = (ROOT / "Makefile").read_text()
    main = makefile.split("main-gate:\n", 1)[1].split("\n\n", 1)[0]
    assert "$(MAKE) installed-wheel-smoke" in main
    runner = (ROOT / "scripts/check-linux-wheel.sh").read_text()
    assert "step['run']" in runner and "smoke_step['run']" in runner
    jobs = yaml.safe_load((ROOT / '.github/workflows/release.yml').read_text())['jobs']
    smoke = jobs['wheel-smoke']
    steps = smoke['steps']
    assert 'actions/checkout@' in steps[0]['uses']
    consumers = [step for step in steps if 'installed-wheel-smoke.sh' in step.get('run', '')]
    assert len(consumers) == 2
    assert {step['if'] for step in consumers} == {"runner.os == 'Linux'", "runner.os != 'Linux'"}
    for entry in jobs['pypi-build']['strategy']['matrix']['include']:
        if 'container' in entry:
            matching = next(row for row in smoke['strategy']['matrix']['include'] if row['artifact'] == 'wheel-' + entry['target'])
            assert matching['container'] == entry['container']
            assert '@sha256:' in entry['container']


@pytest.mark.parametrize("root_owned", [False, True])
@pytest.mark.parametrize("failed_stage,version", [("build", "0.9.4"), ("smoke", "0.9.4"), ("", "0.9.4"), ("", "0.9.5.dev1"), ("cleanup", "0.9.4"), ("build,cleanup", "0.9.4")])
def test_local_wheel_gate_runs_release_steps_and_propagates_failure(
    tmp_path: Path, failed_stage: str, version: str, root_owned: bool,
) -> None:
    # The opt-in Linux control exercises real Docker ownership without compiling.
    real_docker = shutil.which("docker") if root_owned else None
    if root_owned and (sys.platform != "linux" or not os.environ.get("BIOMCP_TEST_ROOT_OWNED_WHEEL")):
        pytest.skip("real Linux Docker ownership control is opt-in")
    assert not root_owned or real_docker

    for relative in (".github/workflows/release.yml", "scripts/check-linux-wheel.sh"):
        destination = tmp_path / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(ROOT / relative, destination)
    metadata = tmp_path / "pyproject.toml"
    metadata.write_text(f'[project]\nversion = "{version}"\n')
    tools = tmp_path / "tools"
    tools.mkdir()
    (tools / "with-build-identity").write_text(
        '#!/bin/sh\nprintf "BIOMCP_BUILD_VERSION=%s\\n" "$BIOMCP_BUILD_VERSION"\n',
        encoding="utf-8",
    )
    (tools / "uv").write_text(
        f"#!{sys.executable}\nimport os, sys\n"
        "assert sys.argv[1:4] == ['run', '--no-sync', 'python']\n"
        "os.execv(sys.executable, [sys.executable] + sys.argv[4:])\n", encoding="utf-8",
    )
    (tools / "docker").write_text(
        f"#!{sys.executable}\nFAILED_STAGE = {failed_stage!r}.split(',')\nEXPECTED_VERSION = {version!r}\nREAL_DOCKER = {real_docker!r}\n"
        '''import os, pathlib, shutil, subprocess, sys
args = sys.argv[1:]
assert '--platform' in args and '--label' in args
assert any('@sha256:' in arg for arg in args)
script = args[-1]
stage = 'cleanup' if '/scratch' in args else ('build' if 'maturin build' in script else 'smoke')
with pathlib.Path('docker.calls').open('a') as log:
    log.write(stage + '\\n')
if stage == 'cleanup':
    if stage in FAILED_STAGE:
        sys.exit(19)
    if REAL_DOCKER:
        sys.exit(subprocess.call([REAL_DOCKER] + args))
    mount = args[args.index('--mount') + 1]
    scratch = pathlib.Path(mount.split('src=', 1)[1].split(',', 1)[0])
    for child in scratch.iterdir():
        if child.is_dir() and not child.is_symlink():
            shutil.rmtree(child)
        else:
            child.unlink()
    sys.exit(0)
if stage == 'build':
    target_arg = next(arg for arg in args if arg.startswith('CARGO_TARGET_DIR='))
    target = pathlib.Path.cwd() / target_arg.split('=', 1)[1].removeprefix('/io/')
    if REAL_DOCKER:
        image = next(arg for arg in args if '@sha256:' in arg)
        subprocess.run([REAL_DOCKER, 'run', '--rm', '--label', 'pm.repo=biomcp', '--label', 'pm.ticket=2310', '-v', str(pathlib.Path.cwd()) + ':/io', '--entrypoint', '/bin/sh', image, '-c', 'mkdir -p "$1/private"; touch "$1/private/root-owned"; chmod 700 "$1/private"', 'sh', target_arg.split('=', 1)[1]], check=True)
        assert target.stat().st_uid == 0
if stage in FAILED_STAGE:
    print(stage + ' stdout failure')
    print(stage + ' stderr failure', file=sys.stderr)
    sys.exit(17)
if stage == 'build':
    assert os.environ['BIOMCP_BUILD_VERSION'] == EXPECTED_VERSION
    destination = script.split(' -o ', 1)[1].split()[0]
    destination = pathlib.Path.cwd() / destination.removeprefix('/io/')
    target_arg = next(arg for arg in args if arg.startswith('CARGO_TARGET_DIR='))
    target = pathlib.Path.cwd() / target_arg.split('=', 1)[1].removeprefix('/io/')
    assert target == destination.parent / 'target', 'build target must belong to this run'
    if not REAL_DOCKER:
        assert not target.exists(), 'build target must start empty'
        target.mkdir()
    (destination / 'fixture.whl').touch()
else:
    mount = next(arg for arg in args if arg.endswith(':/wheel:ro'))
    assert list(pathlib.Path(mount.removesuffix(':/wheel:ro')).glob('*.whl'))
    assert 'installed-wheel-smoke.sh' in script
''', encoding="utf-8",
    )
    for executable in tools.iterdir():
        executable.chmod(0o755)
    (tmp_path / 'unrelated').write_text('keep')
    result = subprocess.run(
        ["bash", "scripts/check-linux-wheel.sh"], cwd=tmp_path,
        env=os.environ | {"PATH": str(tools) + os.pathsep + os.environ["PATH"]},
        capture_output=True, text=True, check=False,
    )
    # Clean up the control's deliberately restricted directory even on a red run.
    cache = tmp_path / '.cache' / 'installed-wheel'
    remaining = list(cache.glob('run-*'))
    if real_docker and remaining:
        image = next(row['container'] for row in yaml.safe_load((ROOT / '.github/workflows/release.yml').read_text())['jobs']['pypi-build']['strategy']['matrix']['include'] if row.get('target') == 'x86_64-unknown-linux-gnu')
        for scratch in remaining:
            subprocess.run([real_docker, 'run', '--rm', '--label', 'pm.repo=biomcp', '--label', 'pm.ticket=2310', '-v', str(scratch) + ':/scratch', '--entrypoint', '/bin/sh', image, '-c', 'find /scratch -mindepth 1 -maxdepth 1 -exec rm -rf -- {} +'], check=True)
    assert bool(result.returncode) == bool(failed_stage), result.stdout + result.stderr
    primary = failed_stage.split(',')[0]
    if primary in ('build', 'smoke'):
        assert primary + ' stdout failure' in result.stdout
        assert primary + ' stderr failure' in result.stderr
        assert 'exit status 17' in result.stderr
    if 'cleanup' in failed_stage:
        assert 'exit status 19' in result.stderr
    assert (tmp_path / 'docker.calls').read_text().splitlines() == (
        ['build', 'cleanup'] if primary == 'build' else ['build', 'smoke', 'cleanup']
    )
    assert bool(remaining) == ('cleanup' in failed_stage)
    for scratch in remaining:
        shutil.rmtree(scratch)
    assert (tmp_path / 'unrelated').read_text() == 'keep'

