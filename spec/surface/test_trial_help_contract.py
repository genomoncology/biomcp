from __future__ import annotations

import os
import re
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BIOMCP_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/release/biomcp"))
LONG_FLAG = re.compile(r"(?<![\w-])--[a-z][a-z0-9-]*")


def _run_help(*args: str) -> str:
    assert BIOMCP_BIN.exists(), f"missing biomcp binary: {BIOMCP_BIN}"
    result = subprocess.run(
        [str(BIOMCP_BIN), *args],
        cwd=REPO_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout


def _section(text: str, start_heading: str, end_heading: str) -> str:
    capture = False
    lines: list[str] = []
    for line in text.splitlines():
        if line == start_heading:
            capture = True
            continue
        if capture and line == end_heading:
            break
        if capture:
            lines.append(line)
    return "\n".join(lines)


def test_get_trial_help_examples_reference_only_declared_options() -> None:
    help_text = _run_help("get", "trial", "--help")
    examples = _section(help_text, "EXAMPLES:", "See also: biomcp list trial")
    options = _section(help_text, "Options:", "EXAMPLES:")

    example_flags = set(LONG_FLAG.findall(examples))
    option_flags = set(LONG_FLAG.findall(options))

    missing = sorted(example_flags - option_flags)
    assert not missing, (
        "get trial help examples reference flags missing from the declared "
        f"Options block: {', '.join(missing)}"
    )
