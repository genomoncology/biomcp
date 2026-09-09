import copy
import json
from pathlib import Path

import jsonschema
import pytest


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


def _gene_with_gencc() -> dict:
    return {
        "symbol": "ODC1",
        "name": "ornithine decarboxylase 1",
        "gencc": {
            "assertions": [
                {
                    "id": "SGC-1.1",
                    "sgc_id": "SGC-1",
                    "version": 1,
                    "gene": {"id": "HGNC:8109", "label": "ODC1"},
                    "disease": {"id": "MONDO:0000001", "label": "Disease"},
                    "classification": {
                        "id": "GENCC:100001",
                        "label": "Definitive",
                        "code": "definitive",
                    },
                    "mode_of_inheritance": {
                        "id": "HP:0000006",
                        "label": "Autosomal dominant inheritance",
                    },
                    "submitter": {"id": "GENCC:000001", "label": "Submitter"},
                    "evaluated_date": "2026-09-06",
                    "submitted_date": None,
                    "source_record_url": "https://thegencc.org/submissions/SGC-1.1",
                    "public_report_url": None,
                    "assertion_criteria_url": "https://example.org/criteria",
                    "publications": [
                        {"pmid": "123", "url": "https://pubmed.ncbi.nlm.nih.gov/123/"}
                    ],
                }
            ],
            "total_matching_assertions": 1,
            "truncated": False,
            "status": {
                "freshness": "fresh",
                "result": "data",
                "operation": "local_query",
                "checked_at": "2026-09-06T06:00:29Z",
                "retrieved_at": "2026-09-06T06:00:29Z",
                "attempted_at": "2026-09-06T06:00:29Z",
                "etag": '"fixture"',
                "last_modified": "Sun, 06 Sep 2026 06:00:29 GMT",
                "upstream_version": None,
                "message": None,
            },
        },
    }


def test_gencc_schema_couples_classification_and_closes_wire_shapes() -> None:
    schema = json.loads((ROOT / "skills/schemas/gene.json").read_text())
    validator = jsonschema.Draft202012Validator(
        schema, format_checker=jsonschema.FormatChecker()
    )
    document = _gene_with_gencc()
    tuples = [
        ("100001", "Definitive", "definitive"),
        ("100002", "Strong", "strong"),
        ("100003", "Moderate", "moderate"),
        ("100004", "Limited", "limited"),
        ("100005", "Disputed Evidence", "disputed_evidence"),
        ("100006", "Refuted Evidence", "refuted_evidence"),
        ("100007", "Animal Model Only", "animal_model_only"),
        ("100008", "No Known Disease Relationship", "no_known_disease_relationship"),
        ("100009", "Supportive", "supportive"),
    ]
    for identifier, label, code in tuples:
        assertion = document["gencc"]["assertions"][0]
        assertion["classification"] = {
            "id": f"GENCC:{identifier}",
            "label": label,
            "code": code,
        }
        validator.validate(document)
    mutations = [
        ("gene.id", "NCBIGene:4953"),
        ("disease.id", "OMIM:1"),
        ("mode_of_inheritance.id", "MONDO:0000001"),
        ("submitter.id", "GENCC:1"),
        ("evaluated_date", "2026-02-30"),
        ("source_record_url", "https://example.org/SGC-1.1"),
        ("classification.label", "Strong"),
    ]
    for path, value in mutations:
        invalid = copy.deepcopy(_gene_with_gencc())
        target = invalid["gencc"]["assertions"][0]
        parts = path.split(".")
        for part in parts[:-1]:
            target = target[part]
        target[parts[-1]] = value
        with pytest.raises(jsonschema.ValidationError):
            validator.validate(invalid)
    for field, value in [
        ("checked_at", "2026-09-06T06:00:29+00:00"),
        ("etag", "fixture"),
        ("message", "arbitrary"),
    ]:
        invalid = copy.deepcopy(_gene_with_gencc())
        invalid["gencc"]["status"][field] = value
        with pytest.raises(jsonschema.ValidationError):
            validator.validate(invalid)
