from __future__ import annotations

from pathlib import Path
import re

REPO_ROOT = Path(__file__).resolve().parents[1]

BANNED_MARKETING_TOKENS = [
    "<img",
    "![",
    "!!!",
    "97%",
    "100%",
    "652",
    "927",
    "7 published papers",
    "AI-powered",
    "revolutionary",
    "simply",
    "just",
]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def _paragraph_count(text: str) -> int:
    return len([part for part in re.split(r"\n\s*\n", text.strip()) if part.strip()])


def _assert_clean_marketing_block(block: str) -> None:
    for token in BANNED_MARKETING_TOKENS:
        assert token not in block


def test_readme_landing_copy_matches_public_contract() -> None:
    readme = _read("README.md")

    assert readme.index("## Description") < readme.index("## Features") < readme.index(
        "## Installation"
    ) < readme.index("## Quick start")

    hero = readme.split("\n## Description\n", 1)[0].split("# BioMCP\n\n", 1)[1].strip()
    description = _markdown_section_block(readme, "## Description\n\n", "\n## Features\n")
    features = _markdown_section_block(readme, "## Features\n\n", "\n## Installation\n")
    quick_start = _markdown_section_block(readme, "## Quick start\n\n", "\n```bash\n")

    assert _paragraph_count(hero) == 1
    assert "plus local study analytics" in description
    assert "First useful query in under 30 seconds:" in quick_start
    assert len(re.findall(r"(?m)^- \*\*[^*]+:\*\* .+", features)) == 6

    for block in [hero, description, features, quick_start]:
        _assert_clean_marketing_block(block)


def test_docs_index_landing_copy_matches_public_contract() -> None:
    docs_index = _read("docs/index.md")

    intro = docs_index.split("\n## Install\n", 1)[0].split("# BioMCP\n\n", 1)[1].strip()
    quick_start = _markdown_section_block(docs_index, "## Quick start\n\n", "\n```bash\n")
    features = _markdown_section_block(
        docs_index,
        "## Feature highlights\n\n",
        "\n## Entities and sources\n",
    )

    assert 1 <= _paragraph_count(intro) <= 2
    assert "Install to first result in under 30 seconds:" in quick_start
    assert len(re.findall(r"(?m)^- \*\*[^*]+:\*\* .+", features)) == 7

    for block in [intro, quick_start, features]:
        _assert_clean_marketing_block(block)
