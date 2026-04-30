from __future__ import annotations

import os
import re
import shlex
import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
BIOMCP_BIN = Path(os.environ.get("BIOMCP_BIN", REPO_ROOT / "target/release/biomcp"))
LONG_FLAG = re.compile(r"(?<![\w-])--[a-z][a-z0-9-]*")
TRIAL_SECTIONS = {"eligibility", "locations", "outcomes", "arms", "references", "all"}


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


def _get_trial_examples(help_text: str) -> list[str]:
    examples = _section(help_text, "EXAMPLES:", "See also: biomcp list trial")
    commands = [
        line.strip()
        for line in examples.splitlines()
        if line.strip().startswith("biomcp get trial ")
    ]
    assert commands, "get trial help should include copy-pasteable examples"
    return commands


def _flags_after_first_section(command: str) -> list[str]:
    tokens = shlex.split(command)
    seen_section = False
    late_flags: list[str] = []
    for token in tokens[4:]:
        if token in TRIAL_SECTIONS:
            seen_section = True
            continue
        if seen_section and token.startswith("--"):
            late_flags.append(token)
    return late_flags


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


def test_get_trial_help_examples_place_options_before_sections() -> None:
    help_text = _run_help("get", "trial", "--help")
    late_flags_by_command = {
        command: _flags_after_first_section(command)
        for command in _get_trial_examples(help_text)
    }
    late_flags_by_command = {
        command: late_flags
        for command, late_flags in late_flags_by_command.items()
        if late_flags
    }

    assert not late_flags_by_command, (
        "get trial help examples place named options after section tokens, "
        f"which trailing section parsing treats as sections: {late_flags_by_command}"
    )
