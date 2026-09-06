from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_gencc_source_page_pins_public_and_operational_boundaries() -> None:
    page = (ROOT / "docs/sources/gencc.md").read_text(encoding="utf-8")
    for required in (
        "biomcp get gene ODC1 gencc",
        "biomcp gencc sync",
        "submission-level",
        "seven days",
        "once per day",
        "20 successful downloads",
        "CC0 1.0",
        "not a",
        "stale zero-match",
        "--no-cache",
    ):
        assert required in page


def test_gencc_schema_and_receipt_are_registered() -> None:
    schema = (ROOT / "skills/schemas/gene.json").read_text(encoding="utf-8")
    receipts = (ROOT / "testdata/sources/capture-receipts.json").read_text(
        encoding="utf-8"
    )
    assert '"gencc"' in schema
    assert "gencc/submissions-new-odc1.csv" in receipts


def test_gencc_operator_and_source_references_are_complete() -> None:
    references = "\n".join(
        (ROOT / path).read_text(encoding="utf-8")
        for path in (
            "docs/reference/configuration.md",
            "docs/reference/data-sources.md",
            "docs/reference/source-licensing.md",
            "docs/user-guide/cli-reference.md",
            "docs/troubleshooting.md",
            "docs/llms-full.txt",
        )
    )
    for required in (
        "BIOMCP_GENCC_DIR",
        "gene gencc",
        "GenCC",
        "gencc sync",
        "thegencc.org",
        "CC0 1.0",
        "gene gencc section",
    ):
        assert required in references
