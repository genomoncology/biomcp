from __future__ import annotations

import json
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _read_json(path: str) -> object:
    return json.loads(_read(path))


def test_bioasq_module_readme_exists_and_uses_uv_run_script() -> None:
    readme = _read("benchmarks/bioasq/README.md")

    assert "# BioASQ Benchmark Module" in readme
    assert "uv run --quiet --script benchmarks/bioasq/ingest_public.py --bundle hf-public-pre2026" in readme
    assert "datasets/raw/" in readme
    assert "datasets/normalized/" in readme
    assert "annotations/" in readme


def test_ranking_calibration_readme_records_verified_scenarios_and_provenance() -> None:
    readme = _read("benchmarks/bioasq/ranking-calibration/README.md")

    assert "# BioASQ Ranking Calibration" in readme
    for snippet in [
        "hybrid_default_weights_orders_example_one",
        "lexical_mode_matches_current_ordering",
        "hybrid_entity_only_falls_back_without_nan",
        "hybrid_custom_weights_shift_ordering",
        "hybrid_scoring_is_zero_safe",
        "hybrid_uses_litsense2_signal_for_semantic_score",
        "semantic_mode_ignores_non_litsense2_raw_scores",
        "hf-public-pre2026",
        "uv run --quiet --script benchmarks/bioasq/ingest_public.py --bundle hf-public-pre2026",
        "benchmarks/bioasq/datasets/manifest.json",
        "benchmarks/bioasq/datasets/README.md",
        "docs/reference/bioasq-benchmark.md",
        "spec/entity/article.md",
        "including zero semantic score on rows without",
        "LitSense2 provenance",
        "Historical leads",
        "unresolved",
    ]:
        assert snippet in readme


def test_ranking_calibration_links_are_present_in_benchmark_docs() -> None:
    benchmark_readme = _read("benchmarks/bioasq/README.md")
    reference_doc = _read("docs/reference/bioasq-benchmark.md")

    assert "## Ranking calibration" in benchmark_readme
    assert "ranking-calibration/README.md" in benchmark_readme

    assert "## Ranking calibration" in reference_doc
    assert "benchmarks/bioasq/ranking-calibration/README.md" in reference_doc


def test_manifest_has_required_top_level_keys_and_bundle_ids() -> None:
    manifest = _read_json("benchmarks/bioasq/datasets/manifest.json")

    assert set(manifest) >= {
        "schema_version",
        "reviewed_on",
        "recommended_public_bundle_id",
        "bundles",
    }

    bundle_ids = {bundle["id"] for bundle in manifest["bundles"]}
    assert bundle_ids == {
        "hf-public-pre2026",
        "mirage-yesno-2024",
        "official-task-b-participant-download",
    }


def test_official_bundle_is_metadata_only_and_requires_registration() -> None:
    manifest = _read_json("benchmarks/bioasq/datasets/manifest.json")
    official_bundle = next(
        bundle
        for bundle in manifest["bundles"]
        if bundle["id"] == "official-task-b-participant-download"
    )

    assert official_bundle["lane"] == "official_competition"
    assert official_bundle["official"] is True
    assert official_bundle["registration_required"] is True
    assert official_bundle["output_path"] is None
    assert official_bundle["official_training_count"] == 5389


def test_recommended_public_bundle_is_hf_public_pre2026() -> None:
    manifest = _read_json("benchmarks/bioasq/datasets/manifest.json")

    assert manifest["recommended_public_bundle_id"] == "hf-public-pre2026"


def test_validity_overlay_schema_exposes_required_fields_and_enum_values() -> None:
    schema = _read_json("benchmarks/bioasq/annotations/validity.schema.json")

    assert schema["type"] == "object"
    assert set(schema["required"]) >= {
        "question_id",
        "bundle_id",
        "validity_status",
        "reason",
        "reviewed_at",
        "reviewer",
        "time_anchor_required",
        "invalid_after",
        "notes",
    }
    assert schema["properties"]["validity_status"]["enum"] == [
        "unknown",
        "valid",
        "stale",
        "invalid",
    ]
    assert (REPO_ROOT / "benchmarks/bioasq/annotations/validity.jsonl").exists()


def test_bioasq_reference_doc_covers_lane_split_provenance_and_official_runbook() -> None:
    doc = _read("docs/reference/bioasq-benchmark.md")

    for heading in [
        "## Two lanes",
        "## Public historical lane",
        "## Recommended bundle",
        "## Ranking calibration",
        "## Provenance and terms",
        "## Validity overlay",
        "## Official competition lane",
        "## Evidence value matrix",
    ]:
        assert heading in doc

    for snippet in [
        "hf-public-pre2026",
        "mirage-yesno-2024",
        "5389",
        "5399",
        "Edit Profile",
        "Phase A",
        "Phase A+",
        "Phase B",
        "24 hours",
        "public historical benchmark lane",
        "official competition lane",
        "benchmarks/bioasq/ranking-calibration/README.md",
    ]:
        assert snippet in doc


def test_navigation_and_docs_link_the_bioasq_reference_page() -> None:
    readme = _read("README.md")
    docs_index = _read("docs/index.md")
    benchmarks = _read("docs/reference/benchmarks.md")
    mkdocs = _read("mkdocs.yml")

    assert "[BioASQ Benchmark](docs/reference/bioasq-benchmark.md)" in readme
    assert "[BioASQ Benchmark](reference/bioasq-benchmark.md)" in docs_index
    assert "[BioASQ Benchmark](bioasq-benchmark.md)" in benchmarks
    assert "      - BioASQ Benchmark: reference/bioasq-benchmark.md" in mkdocs
