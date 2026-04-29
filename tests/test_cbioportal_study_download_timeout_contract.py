from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]


def _read_repo(relative: str) -> str:
    return (REPO_ROOT / relative).read_text(encoding="utf-8")


def test_data_sources_documents_cbioportal_study_download_idle_policy() -> None:
    data_sources = _read_repo("docs/reference/data-sources.md")

    assert "cBioPortal DataHub archive downloads" in data_sources
    assert "no total archive timeout" in data_sources
    assert "idle" in data_sources
    assert "stalled" in data_sources
    assert "no bytes" in data_sources or "no progress" in data_sources
