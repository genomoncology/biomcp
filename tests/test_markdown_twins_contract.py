from __future__ import annotations

import os
from pathlib import Path
import subprocess

import pytest

ROOT = Path(
    os.environ.get("BIOMCP_TEST_ROOT", Path(__file__).resolve().parents[1])
).resolve()
DOCS = ROOT / "docs"


@pytest.fixture(scope="module")
def built_site(tmp_path_factory: pytest.TempPathFactory) -> Path:
    site = tmp_path_factory.mktemp("markdown-twins-site")
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


def _page_routes(source: Path) -> tuple[str, str]:
    relative = source.relative_to(DOCS)
    markdown_route = "/" + relative.as_posix()
    if relative == Path("index.md"):
        return "/", markdown_route
    if relative.name == "index.md":
        return "/" + relative.parent.as_posix() + "/", markdown_route
    return "/" + relative.with_suffix("").as_posix() + "/", markdown_route


def test_docs_build_publishes_an_exact_markdown_twin_for_every_page(
    built_site: Path,
) -> None:
    missing_or_changed = []
    for source in sorted(DOCS.rglob("*.md")):
        twin = built_site / source.relative_to(DOCS)
        if not twin.is_file():
            missing_or_changed.append(f"missing {twin.relative_to(built_site)}")
        elif twin.read_bytes() != source.read_bytes():
            missing_or_changed.append(f"changed {twin.relative_to(built_site)}")

    assert not missing_or_changed, "\n".join(missing_or_changed)
    assert (built_site / "CNAME").read_bytes() == (DOCS / "CNAME").read_bytes()


def test_llms_txt_documents_explicit_markdown_routes_without_negotiation() -> None:
    agent_index = (DOCS / "llms.txt").read_text(encoding="utf-8").lower()

    assert "accept: text/markdown" not in agent_index
    assert "by replacing its trailing\nslash with `.md`" in agent_index
