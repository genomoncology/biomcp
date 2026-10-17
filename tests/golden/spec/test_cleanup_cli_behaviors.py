"""Caller-visible replacements for redundant article help tests."""

from __future__ import annotations

import pytest

from replay_env import ReplayEnvironment
from test_golden_output import BINARY

pytestmark = pytest.mark.needs_binary


@pytest.mark.parametrize(
    ("command", "fragments"),
    [
        (
            ("search", "article", "--help"),
            (
                "When to use:", "QUERY FORMULATION:", "Tokens are not secrets",
                "Published after date (YYYY, YYYY-MM, or YYYY-MM-DD)",
                "--max-per-source <N>", "Prefer --type review",
            ),
        ),
        (
            ("get", "article", "--help"),
            (
                "Allow Semantic Scholar PDF as a final fulltext fallback",
                "--output <FILE>",
                "Binary or unknown-type assets are refused when standard output is a terminal",
            ),
        ),
    ],
)
def test_article_help_through_the_command_line(
    replay: ReplayEnvironment, command: tuple[str, ...], fragments: tuple[str, ...]
) -> None:
    result = replay.run(BINARY, command).process
    assert result.returncode == 0, result.stderr
    for fragment in fragments:
        assert fragment in result.stdout
