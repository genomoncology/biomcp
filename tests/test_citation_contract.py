from __future__ import annotations

import re
import tomllib
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
CHANGELOG_RELEASE_HEADER = re.compile(
    r"^##\s+(?P<version>[0-9][^\s]*)\s+[—-]\s+(?P<date>\d{4}-\d{2}-\d{2})$",
    re.MULTILINE,
)


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _markdown_section_block(text: str, heading: str, next_heading: str) -> str:
    start = text.index(heading)
    remainder = text[start + len(heading) :]
    end = remainder.find(next_heading)
    if end == -1:
        return remainder
    return remainder[:end]


def _citation_scalar(field_name: str) -> str:
    citation = _read("CITATION.cff")
    match = re.search(rf"^{re.escape(field_name)}:\s*(.+)$", citation, re.MULTILINE)
    assert match is not None, f"missing {field_name} in CITATION.cff"
    return match.group(1).strip().strip('"')


def test_citation_cff_exists_and_has_expected_cff_keys() -> None:
    citation_path = REPO_ROOT / "CITATION.cff"

    assert citation_path.exists()

    citation = citation_path.read_text(encoding="utf-8")

    assert "cff-version: 1.2.0" in citation
    assert "title: BioMCP" in citation
    assert "type: software" in citation
    assert "license: MIT" in citation
    assert "repository-code: https://github.com/genomoncology/biomcp" in citation
    assert "url: https://biomcp.org" in citation
    assert "authors:" in citation
    assert "given-names: Ian" in citation
    assert "family-names: Maurer" in citation
    assert "given-names: Justin" in citation
    assert "family-names: Yeakley" in citation
    assert "given-names: Anibee" in citation
    assert "family-names: Zingalis" in citation
    assert "doi:" not in citation.lower()
    assert "preferred-citation:" not in citation


def test_citation_cff_release_metadata_matches_repo_metadata() -> None:
    citation_version = _citation_scalar("version")
    citation_date = _citation_scalar("date-released")

    cargo = tomllib.loads(_read("Cargo.toml"))
    pyproject = tomllib.loads(_read("pyproject.toml"))
    changelog_match = CHANGELOG_RELEASE_HEADER.search(_read("CHANGELOG.md"))

    assert changelog_match is not None, "missing release header in CHANGELOG.md"

    assert citation_version == "0.8.25"
    assert citation_date == "2026-07-07"
    assert cargo["package"]["version"] == "0.9.0-dev.6"
    assert pyproject["project"]["version"] == "0.9.0.dev6"
    assert citation_version == changelog_match.group("version")
    assert citation_date == changelog_match.group("date")


def test_citation_guidance_does_not_claim_an_unreleased_paper_or_version() -> None:
    for path, heading, next_heading in (
        ("README.md", "## Citation", "\n## License"),
        ("docs/index.md", "## Citation", "\n## License"),
    ):
        guidance = _markdown_section_block(_read(path), heading, next_heading).lower()
        assert "0.9.0" not in guidance
        assert "doi" not in guidance


def test_readme_citation_section_points_to_root_citation_file() -> None:
    readme = _read("README.md")

    assert readme.index("## Documentation") < readme.index("## Citation")

    citation_block = _markdown_section_block(readme, "## Citation", "\n## License")

    assert "[`CITATION.cff`](CITATION.cff)" in citation_block
    assert "Cite this repository" in citation_block


def test_docs_index_citation_section_points_to_github_citation_file() -> None:
    docs_index = _read("docs/index.md")

    assert docs_index.index("## Documentation") < docs_index.index("## Citation")

    citation_block = _markdown_section_block(docs_index, "## Citation", "\n## License")

    assert (
        "[`CITATION.cff`](https://github.com/genomoncology/biomcp/blob/main/CITATION.cff)"
        in citation_block
    )
    assert "Cite this repository" in citation_block
