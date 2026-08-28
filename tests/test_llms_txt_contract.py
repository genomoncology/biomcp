from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess
from urllib.parse import urlsplit

import pytest

ROOT = Path(__file__).resolve().parents[1]
BIOMCP_URL = re.compile(r"https://biomcp\.org(?:/[^\s)>`\]]*)?")
INDEX_ENTRY = re.compile(
    r"^- \[[^]]+\]\((https://biomcp\.org(?:/[^)]*)?)\):\s+\S.*$",
    re.MULTILINE,
)


@pytest.fixture(scope="module")
def built_site(tmp_path_factory: pytest.TempPathFactory) -> Path:
    site = tmp_path_factory.mktemp("llms-site")
    completed = subprocess.run(
        [
            "uv",
            "run",
            "--project",
            str(ROOT),
            "--no-sync",
            "mkdocs",
            "build",
            "--strict",
            "--site-dir",
            str(site),
        ],
        cwd=ROOT,
        env=os.environ | {"NO_MKDOCS_2_WARNING": "1"},
        capture_output=True,
        text=True,
        check=False,
    )
    assert completed.returncode == 0, completed.stdout + completed.stderr
    return site


def _published_path(site: Path, url: str) -> Path:
    parsed = urlsplit(url)
    assert parsed.netloc == "biomcp.org"
    relative = parsed.path.removeprefix("/")
    if not relative or relative.endswith("/"):
        return site / relative / "index.html"
    return site / relative


def _assert_biomcp_urls_are_published(site: Path, text: str) -> None:
    urls = set(BIOMCP_URL.findall(text))
    assert urls, "agent index has no biomcp.org URLs"
    missing = sorted(url for url in urls if not _published_path(site, url).is_file())
    assert not missing, f"agent index links to unpublished URLs: {missing}"


def _docs_url(source: Path) -> str:
    relative = source.relative_to(ROOT / "docs")
    if relative.name == "index.md":
        route = relative.parent.as_posix()
    else:
        route = relative.with_suffix("").as_posix()
    suffix = f"{route}/" if route != "." else ""
    return f"https://biomcp.org/{suffix}"


def test_mkdocs_serves_a_curated_agent_entry_point(built_site: Path) -> None:
    curated = (built_site / "llms.txt").read_text(encoding="utf-8")
    full = (built_site / "llms-full.txt").read_text(encoding="utf-8")

    words = set(re.findall(r"[\w-]+", curated.lower()))
    assert {"biomedical", "cli", "mcp"} <= words
    assert "curl -fsSL https://biomcp.org/install.sh | bash" in curated
    assert "search <entity> [filters]" in curated
    assert "get <entity> <id> [section...]" in curated
    for tool in (
        "biomcp",
        "search",
        "get",
        "variant_normalize_car",
        "variant_erepo",
        "gene_cspec",
        "variant_articles",
    ):
        assert tool in words
    assert "https://github.com/genomoncology/biomcp/tree/main/docs" in curated

    curated_urls = set(BIOMCP_URL.findall(curated))
    full_urls = set(BIOMCP_URL.findall(full))
    assert curated_urls < full_urls, "llms.txt must remain a curated entry point"
    _assert_biomcp_urls_are_published(built_site, curated)


def test_full_agent_index_describes_every_published_docs_page(
    built_site: Path,
) -> None:
    full = (built_site / "llms-full.txt").read_text(encoding="utf-8")
    indexed_urls = set(INDEX_ENTRY.findall(full))
    expected_urls = {_docs_url(path) for path in (ROOT / "docs").rglob("*.md")}

    assert expected_urls <= indexed_urls
    _assert_biomcp_urls_are_published(built_site, full)
