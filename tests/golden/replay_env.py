"""The golden suite's replay environment.

Starts `tools/replay-server`, points every provider base override at it, and
runs commands with a cache-free, secret-free, config-free environment so the
golden files pin command output alone. One shared `BIOMCP_TEST_UNPACED_ORIGIN`
unpaces every provider because they all share the replay origin, and the same
loopback signal is what the stricter provider clients require before they
accept a loopback base override.
"""

from __future__ import annotations

import json
import os
import shlex
import subprocess
import sys
import urllib.request
from contextlib import contextmanager
from dataclasses import dataclass, field
from pathlib import Path
from tempfile import TemporaryDirectory
from typing import Iterator

# The replay fixture is also imported by standalone subprocess probes.
sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from cache_fixture import set_cache_root  # noqa: E402

REPO_ROOT = Path(__file__).resolve().parents[2]
REPLAY_SERVER = REPO_ROOT / "tools" / "replay-server"

# Provider API credentials change the request URLs and the replies; a golden
# run must not inherit any of them from the calling environment.
STRIPPED_SECRETS = (
    "NCBI_API_KEY",
    "S2_API_KEY",
    "OPENFDA_API_KEY",
    "NCI_API_KEY",
    "ONCOKB_TOKEN",
    "DISGENET_API_KEY",
    "ALPHAGENOME_API_KEY",
    "UMLS_API_KEY",
    "ORCID_ACCESS_TOKEN",
)

# Local dataset directories a command may read or populate; each run gets its
# own empty directory so a dataset download replays through the server too.
DATASET_DIRS = (
    "BIOMCP_GENCC_DIR",
    "BIOMCP_EMA_DIR",
    "BIOMCP_WHO_DIR",
    "BIOMCP_WHO_IVD_DIR",
    "BIOMCP_GTR_DIR",
    "BIOMCP_DDINTER_DIR",
    "BIOMCP_CVX_DIR",
)


def isolated_command_env(
    run_dir: Path, provider_env: dict[str, str], origin: str
) -> dict[str, str]:
    """Isolate CLI home, config, cache, datasets and provider credentials."""
    env = {
        key: value
        for key, value in os.environ.items()
        if key not in STRIPPED_SECRETS and key != "COLUMNS"
    }
    set_cache_root(env, run_dir / "cache")
    env.update(provider_env)
    env.update(
        {
            "BIOMCP_TEST_UNPACED_ORIGIN": origin,
            "BIOMCP_CACHE_MODE": "off",
            "XDG_CACHE_HOME": str(run_dir / "xdg-cache"),
            "XDG_CONFIG_HOME": str(run_dir / "xdg-config"),
            "HOME": str(run_dir),
            "RUST_LOG": "error",
            "NO_COLOR": "1",
            "TERM": "dumb",
        }
    )
    for name in DATASET_DIRS:
        stem = name.removeprefix("BIOMCP_").lower()
        env[name] = str(run_dir / stem)
    return env


def retained_cache_env(
    run_dir: Path, provider_env: dict[str, str], origin: str,
    *, cache_dir: Path | None = None, max_size: int = 10000000,
) -> dict[str, str]:
    """Keep fixture responses independently of host cache and free-space policy."""
    env = isolated_command_env(run_dir, provider_env, origin)
    env.update({
        "BIOMCP_CACHE_MODE": "default",
        "BIOMCP_CACHE_MAX_SIZE": str(max_size),
        "BIOMCP_CACHE_MAX_AGE": "86400",
    })
    if cache_dir is not None:
        set_cache_root(env, cache_dir)
    return env


@dataclass
class ReplayRun:
    """One finished command run with its replay accounting."""

    process: subprocess.CompletedProcess
    request_log_lines: list[dict]
    run_dir: Path


@dataclass
class ReplayEnvironment:
    """A running replay server and the environment that points commands at it."""

    origin: str
    provider_env: dict[str, str]
    request_log: Path
    work_dir: Path
    record: bool
    process: subprocess.Popen = field(repr=False)
    _run_counter: int = 0

    def _next_run_dir(self) -> Path:
        self._run_counter += 1
        run_dir = self.work_dir / f"run-{self._run_counter}"
        (run_dir / "cache").mkdir(parents=True)
        for name in DATASET_DIRS:
            (run_dir / name.removeprefix("BIOMCP_").lower()).mkdir()
        return run_dir

    def command_env(self, run_dir: Path) -> dict[str, str]:
        return isolated_command_env(run_dir, self.provider_env, self.origin)

    def request_log_entries(self, since: int = 0) -> list[dict]:
        entries = []
        if self.request_log.is_file():
            lines = self.request_log.read_text(encoding="utf-8").splitlines()
        else:
            lines = []
        for line in lines[since:]:
            entries.append(json.loads(line))
        return entries

    def run(
        self,
        binary: Path,
        argv: tuple[str, ...],
        timeout: float = 120.0,
    ) -> ReplayRun:
        """Run one command against the replay server with a fresh cache."""
        run_dir = self._next_run_dir()
        before = self.request_log_line_count()
        process = subprocess.run(
            [str(binary), *argv],
            env=self.command_env(run_dir),
            capture_output=True,
            text=True,
            timeout=timeout,
            cwd=str(REPO_ROOT),
        )
        entries = self.request_log_entries(since=before)
        return ReplayRun(process=process, request_log_lines=entries, run_dir=run_dir)

    def request_log_line_count(self) -> int:
        if not self.request_log.is_file():
            return 0
        return len(self.request_log.read_text(encoding="utf-8").splitlines())

    def misses(self, since: int = 0) -> list[dict]:
        return [
            entry
            for entry in self.request_log_entries(since=since)
            if entry.get("result") in ("miss", "record-error")
        ]


def _read_env_file(path: Path) -> dict[str, str]:
    exports: dict[str, str] = {}
    for line in path.read_text(encoding="utf-8").splitlines():
        name, value = line.removeprefix("export ").split("=", 1)
        exports[name] = shlex.split(value)[0]
    return exports


@contextmanager
def replay_environment(record: bool = False) -> Iterator[ReplayEnvironment]:
    """Yield a running replay server with every provider base pointed at it."""
    with TemporaryDirectory(prefix="biomcp-golden-replay.") as work:
        work_dir = Path(work)
        env_file = work_dir / "replay-env"
        request_log = work_dir / "request.log"
        argv = [
            str(REPLAY_SERVER),
            "--root",
            str(REPO_ROOT),
            "--env-file",
            str(env_file),
            "--request-log",
            str(request_log),
        ]
        if record:
            argv.append("--record")
        process = subprocess.Popen(
            argv,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            ready = process.stdout.readline() if process.stdout else ""
            if not ready:
                stderr = process.stderr.read() if process.stderr else ""
                raise RuntimeError(f"replay server exited before serving: {stderr.strip()}")
            marker, _, origin = ready.strip().partition(" ")
            if marker != "READY":
                raise RuntimeError(f"replay server greeted unexpectedly: {ready.strip()!r}")
            provider_env = _read_env_file(env_file)
            with urllib.request.urlopen(f"{origin}/healthz", timeout=5) as probe:
                if probe.status != 200:
                    raise RuntimeError("replay server health probe failed")
            yield ReplayEnvironment(
                origin=origin,
                provider_env=provider_env,
                request_log=request_log,
                work_dir=work_dir,
                record=record,
                process=process,
            )
        finally:
            process.terminate()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.wait(timeout=10)
