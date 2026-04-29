import re
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


def _read_repo(relative: str) -> str:
    return (REPO_ROOT / relative).read_text(encoding="utf-8")


def test_data_sources_documents_cbioportal_study_download_idle_policy() -> None:
    data_sources = _read_repo("docs/reference/data-sources.md").lower()

    for term in ("cbioportal", "datahub", "archive", "download"):
        assert term in data_sources
    assert re.search(
        r"(no|without|omit\w*|not governed by|not use\w*).{0,100}total.{0,40}timeout"
        r"|total.{0,40}timeout.{0,100}(does not apply|not apply|omitted|disabled|not used)",
        data_sources,
        re.DOTALL,
    )
    assert (
        "idle" in data_sources
        or "no-progress" in data_sources
        or "no progress" in data_sources
    )
    assert "stall" in data_sources or "no progress" in data_sources
    assert "no bytes" in data_sources or "no progress" in data_sources
