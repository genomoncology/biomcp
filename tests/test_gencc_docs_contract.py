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
