from __future__ import annotations

import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]

EXPECTED_GITATTRIBUTES = """*.pdf binary
*.png binary
*.gif binary
*.ico binary

# The vendored tokenizer cache is an offline asset, not reviewable text
benchmarks/output-footprint/tokenizer-cache/* -diff
"""


def _git_check_attr(path: str) -> dict[str, str]:
    result = subprocess.run(
        ["git", "check-attr", "binary", "diff", "merge", "text", "--", path],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=True,
    )
    attrs: dict[str, str] = {}
    for line in result.stdout.splitlines():
        _, attribute, value = line.split(": ", maxsplit=2)
        attrs[attribute] = value
    return attrs


def test_binary_asset_gitattributes_contract_is_exact() -> None:
    assert (REPO_ROOT / ".gitattributes").read_text(encoding="utf-8") == (
        EXPECTED_GITATTRIBUTES
    )


def test_pdf_fixture_is_treated_as_binary_by_git() -> None:
    assert _git_check_attr(
        "tests/fixtures/article/fulltext/pdf/cdc_sti_guideline.pdf"
    ) == {
        "binary": "set",
        "diff": "unset",
        "merge": "unset",
        "text": "unset",
    }


def test_binary_asset_extensions_are_treated_as_binary_by_git() -> None:
    for path in [
        "tests/fixtures/article/fulltext/pdf/example.pdf",
        "tests/fixtures/article/fulltext/png/example.png",
        "tests/fixtures/article/fulltext/gif/example.gif",
        "tests/fixtures/article/fulltext/ico/example.ico",
    ]:
        assert _git_check_attr(path)["binary"] == "set"
