from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (ROOT / path).read_text()


def test_author_surface_docs_describe_the_shipped_provider_exact_slice() -> None:
    functional = _read("architecture/functional/overview.md")
    runtime = _read("architecture/technical/semantic-scholar-runtime-contract.md")
    ux = _read("architecture/ux/cli-reference.md")
    source_guide = _read("docs/sources/semantic-scholar.md")

    assert "| author | Semantic Scholar provider records |" in functional
    assert "BioMCP does not yet ship an author entity" not in functional

    assert "public provider-exact `search author`" in runtime
    assert "without introducing a public author command" not in runtime

    for text in (ux, source_guide):
        assert "search author" in text
        assert "get author semanticscholar:" in text
        assert "--source semanticscholar" in text

    assert "BioMCP does not yet ship these commands" not in ux
