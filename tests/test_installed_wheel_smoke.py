"""Exercise the shared installed-artifact smoke without contacting providers."""
from __future__ import annotations

import os
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


@pytest.mark.parametrize("failed_stage,version", [("build", "0.9.4"), ("smoke", "0.9.4"), ("", "0.9.4"), ("", "0.9.5.dev1")])
def test_local_wheel_gate_runs_release_steps_and_propagates_failure(
    tmp_path: Path, failed_stage: str, version: str,
) -> None:
    import shutil

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
        f"#!{sys.executable}\nFAILED_STAGE = {failed_stage!r}\nEXPECTED_VERSION = {version!r}\n"
        '''import os, pathlib, sys
args = sys.argv[1:]
assert '--platform' in args and '--label' in args
assert any('@sha256:' in arg for arg in args)
script = args[-1]
stage = 'build' if 'maturin build' in script else 'smoke'
with pathlib.Path('docker.calls').open('a') as log:
    log.write(stage + '\\n')
if stage == FAILED_STAGE:
    print(stage + ' stdout failure')
    print(stage + ' stderr failure', file=sys.stderr)
    sys.exit(17)
if stage == 'build':
    assert os.environ['BIOMCP_BUILD_VERSION'] == EXPECTED_VERSION
    destination = script.split(' -o ', 1)[1].split()[0]
    destination = pathlib.Path.cwd() / destination.removeprefix('/io/')
    (destination / 'fixture.whl').touch()
else:
    mount = next(arg for arg in args if arg.endswith(':/wheel:ro'))
    assert list(pathlib.Path(mount.removesuffix(':/wheel:ro')).glob('*.whl'))
    assert 'installed-wheel-smoke.sh' in script
''', encoding="utf-8",
    )
    for executable in tools.iterdir():
        executable.chmod(0o755)
    result = subprocess.run(
        ["bash", "scripts/check-linux-wheel.sh"], cwd=tmp_path,
        env=os.environ | {"PATH": str(tools) + os.pathsep + os.environ["PATH"]},
        capture_output=True, text=True, check=False,
    )
    assert bool(result.returncode) == bool(failed_stage), result.stdout + result.stderr
    if failed_stage:
        assert failed_stage + ' stdout failure' in result.stdout
        assert failed_stage + ' stderr failure' in result.stderr
    assert (tmp_path / 'docker.calls').read_text().splitlines() == (
        ['build'] if failed_stage == 'build' else ['build', 'smoke']
    )
