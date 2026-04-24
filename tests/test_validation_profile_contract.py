from __future__ import annotations

import re
import subprocess
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
PROFILE_PATH = REPO_ROOT / ".march" / "validation-profiles.toml"
EXPECTED_COMMANDS = {
    "preflight": "cargo check --all-targets",
    "baseline": "cargo check --all-targets",
    "focused": "cargo test --lib && cargo clippy --lib --tests -- -D warnings",
    "full-blocking": "make release-gate",
    "full-contracts": "make release-gate",
}


def _read_profile() -> str:
    return PROFILE_PATH.read_text(encoding="utf-8")


def test_validation_profiles_file_is_tracked_and_matches_reserved_contract() -> None:
    assert PROFILE_PATH.is_file(), "missing .march/validation-profiles.toml"

    tracked = subprocess.run(
        ["git", "ls-files", "--error-unmatch", ".march/validation-profiles.toml"],
        cwd=REPO_ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    assert tracked.returncode == 0, tracked.stderr
    assert tracked.stdout.strip() == ".march/validation-profiles.toml"

    data = tomllib.loads(_read_profile())
    profiles = data["profile"]

    assert set(profiles) == set(EXPECTED_COMMANDS)
    for name, command in EXPECTED_COMMANDS.items():
        assert profiles[name]["command"] == command


def test_validation_profiles_record_observed_timings_above_each_table() -> None:
    content = _read_profile()

    for name in EXPECTED_COMMANDS:
        assert re.search(
            rf"(?m)^# observed .+\n\[profile\.{re.escape(name)}\]$",
            content,
        ), f"missing observed timing comment immediately above profile.{name}"
