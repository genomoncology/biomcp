from __future__ import annotations

import os
from pathlib import Path
import subprocess


ROOT = Path(__file__).resolve().parents[2]
PUBLIC_DOCS = [
    ROOT / "README.md",
    ROOT / "docs" / "index.md",
]


def _read(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def test_public_docs_label_search_only_entities() -> None:
    documents = [(str(path), _read(path)) for path in PUBLIC_DOCS]
    biomcp_bin = Path(os.environ.get("BIOMCP_BIN", ROOT / "target" / "release" / "biomcp"))
    result = subprocess.run(
        [str(biomcp_bin), "list"],
        check=True,
        capture_output=True,
        text=True,
    )
    documents.append(("biomcp list", result.stdout))

    for source, text in documents:
        assert "Search-Only Entities" in text or "Search-only entities" in text, source
        assert "gwas" in text and "search gwas" in text, source
        assert "phenotype" in text and "search phenotype" in text, source


def test_public_docs_do_not_imply_get_gwas_or_get_phenotype() -> None:
    forbidden = ["get gwas", "get phenotype"]
    documents = [(str(path), _read(path)) for path in PUBLIC_DOCS]
    documents.append(("list template", _read(ROOT / "src" / "cli" / "list_reference.md")))
    for source, text in documents:
        text = text.lower()
        for phrase in forbidden:
            assert phrase not in text, f"{source} must not document `{phrase}`"


def test_spec_architecture_routes_public_search_only_surfaces() -> None:
    text = _read(ROOT / "architecture" / "technical" / "spec-v2.md")
    assert "`gwas` is covered by `spec/entity/variant.md`" in text
    assert "CDC WONDER VAERS aggregate lane in `spec/entity/vaers.md`" in text
