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

    assert (
        readme.index("## What is BioMCP?")
        < readme.index("## Features")
        < readme.index("## Quick start")
        < readme.index("## Installation")
    )

    above_hero = readme.split("\n## What is BioMCP?\n", 1)[0].split("# BioMCP\n\n", 1)[1].strip()
    hero = above_hero.split("\n## Data terms\n", 1)[0].strip()
    description = _markdown_section_block(
        readme, "## What is BioMCP?\n\n", "\n## Features\n"
    )
    features = _markdown_section_block(readme, "## Features\n\n", "\n## Quick start\n")
    quick_start = _markdown_section_block(readme, "## Quick start\n\n", "\n```bash\n")

    # The hero block is the tagline alone again (2026-09-29 second
    # review): the warning lives in its own section after the hero.
    assert _paragraph_count(hero) == 1
    # 2026-09-28: the count is computed from sources.json so it
    # cannot drift from the registry the way "~30" did.
    import json
    from pathlib import Path as _Path
    sources = json.loads(
        (_Path(__file__).resolve().parents[1] / "docs" / "reference" / "sources.json").read_text(
            encoding="utf-8"
        )
    )
    direct = sum(1 for item in sources if item.get("integration_mode") == "direct_api")
    indirect = len(sources) - direct
    assert f"single command grammar that reaches {direct} trusted" in description
    # The README names the indirect count in words today; accept the
    # word form for eight and ten and the digit form otherwise, so a
    # drift forces an intentional edit either way.
    words = {8: "eight further", 10: "ten further"}
    expected = words.get(indirect, f"{indirect} further")
    assert expected in description, f"indirect count {indirect} not stated: {expected!r}"
    assert "MCP (Model Context Protocol) server" in description
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

def test_readme_carries_the_licence_warning_and_link() -> None:
    """Ian's 2026-09-29 direction: the README states plainly that
    upstream terms govern the retrieved data and links to the
    licensing page. This check fails if either goes missing.
    """
    readme = (REPO_ROOT / "README.md").read_text(encoding="utf-8")
    # The warning's own section, verbatim requirements (2026-09-29
    # second review): the earlier or-forms accepted almost anything.
    top = readme[: readme.index("## What is BioMCP?")]
    assert "## Data terms" in top, "the Data terms section must precede the hero"
    section = _markdown_section_block(readme, "## Data terms\n\n", "\n## What is BioMCP?")
    assert "terms govern how you use the data" in section.lower(), (
        "the warning must say the upstream terms govern the retrieved data"
    )
    assert "restrict commercial or clinical use" in section, (
        "the warning must name the restriction class"
    )
    assert "obtain any licence you need" in section or "license you need" in section, (
        "the warning must tell the reader to obtain their own licence"
    )
    assert "[Source Licensing and Terms](docs/reference/source-licensing.md)" in section, (
        "the warning must link to the Source Licensing and Terms page"
    )
