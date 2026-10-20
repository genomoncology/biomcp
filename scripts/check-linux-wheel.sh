#!/usr/bin/env bash
set -euo pipefail

# Read the release job itself so its platform, image, build and install cannot drift.
uv run --no-sync python - <<'PY'
import os
import platform
import subprocess
import tempfile
import tomllib
from pathlib import Path

import yaml

root = Path.cwd()
workflow = yaml.safe_load((root / '.github/workflows/release.yml').read_text())
arch = {'arm64': 'aarch64', 'aarch64': 'aarch64', 'x86_64': 'x86_64', 'AMD64': 'x86_64'}[platform.machine()]
target = f'{arch}-unknown-linux-gnu'
build = workflow['jobs']['pypi-build']
entry = next(row for row in build['strategy']['matrix']['include'] if row['target'] == target)
smoke = workflow['jobs']['wheel-smoke']
smoke_entry = next(row for row in smoke['strategy']['matrix']['include'] if row['artifact'] == f'wheel-{target}')
image = entry['container']
assert image == smoke_entry['container'] and '@sha256:' in image
step = next(row for row in build['steps'] if row.get('name') == 'Build wheels inside the manylinux 2_28 container')
smoke_step = next(row for row in smoke['steps'] if row.get('name') == 'Run the wheel inside the manylinux 2_28 container')
version = tomllib.loads((root / 'pyproject.toml').read_text())['project']['version']
linux_platform = 'linux/arm64' if arch == 'aarch64' else 'linux/amd64'
cache = root / '.cache' / 'installed-wheel'
cache.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='run-', dir=cache) as scratch:
    scratch = Path(scratch)
    wheel_dir = scratch / 'wheels'
    wheel_dir.mkdir()
    container_wheels = '/io/' + wheel_dir.relative_to(root).as_posix()
    command = step['run'].replace('${{ matrix.target }}', target).replace('${{ matrix.container }}', image)
    command = command.replace('target/wheels', container_wheels)
    # Build each wheel with an empty target owned and removed by this run.
    container_target = '/io/' + (scratch / 'target').relative_to(root).as_posix()
    command = command.replace('docker run --rm', f'docker run --rm --platform {linux_platform} --label pm.repo=biomcp --label pm.ticket=2288 -e CARGO_TARGET_DIR={container_target}', 1)
    env = os.environ | {'RUNNER_TEMP': str(scratch), 'TAG': f'v{version}', 'BIOMCP_BUILD_VERSION': version}
    subprocess.run(['bash', '-euo', 'pipefail', '-c', command], env=env, check=True)
    command = smoke_step['run'].replace('${{ matrix.container }}', image)
    command = command.replace('docker run --rm', f'docker run --rm --platform {linux_platform} --label pm.repo=biomcp --label pm.ticket=2288', 1)
    env['RUNNER_TEMP'] = str(scratch)
    wheel_dir.rename(scratch / 'wheel')
    subprocess.run(['bash', '-euo', 'pipefail', '-c', command], env=env, check=True)
PY
