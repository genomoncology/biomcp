from __future__ import annotations

import json
import os
import stat
import subprocess
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]


def _read(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def _load_json(path: str) -> object:
    with (REPO_ROOT / path).open(encoding="utf-8") as handle:
        return json.load(handle)


def _assert_shape(actual: object, expected: object) -> None:
    if isinstance(expected, type):
        assert isinstance(actual, expected)
        return

    if isinstance(expected, dict):
        assert isinstance(actual, dict)
        assert list(actual) == list(expected)
        for key, value in expected.items():
            _assert_shape(actual[key], value)
        return

    assert actual == expected


def test_root_metadata_contract_for_paper_release() -> None:
    contributing = _read("CONTRIBUTING.md")
    code_of_conduct = _read("CODE_OF_CONDUCT.md")
    zenodo = _load_json(".zenodo.json")
    citation = _read("CITATION.cff")

    assert "# Contributing to BioMCP" in contributing
    assert "does not accept outside pull requests" in contributing
    assert "GitHub Issues" in contributing
    assert "GitHub Discussions" in contributing
    assert "supply-chain" in contributing
    assert "AI-assisted code" in contributing

    assert "# BioMCP Code of Conduct" in code_of_conduct
    assert "respectful" in code_of_conduct.lower()
    assert "harassment" in code_of_conduct.lower()
    assert "Issues" in code_of_conduct
    assert "Discussions" in code_of_conduct
    assert "GitHub" in code_of_conduct

    assert zenodo == {
        "title": "BioMCP",
        "description": (
            "BioMCP is a single-binary CLI and MCP server for querying biomedical "
            "databases with one command grammar, compact markdown output, and local "
            "study analytics."
        ),
        "license": "MIT",
        "upload_type": "software",
        "creators": [
            {"name": "Maurer, Ian"},
            {"name": "Yeakley, Justin"},
            {"name": "Zingalis, Anibee"},
        ],
        "keywords": [
            "bioinformatics",
            "biomedical-databases",
            "clinical-trials",
            "genomics",
            "mcp",
        ],
    }

    assert "preferred-citation:" in citation
    assert "publisher: Zenodo" in citation
    assert "doi: 10.5281/zenodo.XXXXXXX" in citation
    assert "version: 0.9.0" in citation
    assert "Release-time placeholder" in citation
    assert citation.count("given-names: Ian") == 2
    assert citation.count("family-names: Maurer") == 2
    assert citation.count("given-names: Justin") == 2
    assert citation.count("family-names: Yeakley") == 2
    assert citation.count("given-names: Anibee") == 2
    assert citation.count("family-names: Zingalis") == 2


def test_paper_readme_and_layout_contract() -> None:
    readme = _read("paper/README.md")
    gitignore = _read(".gitignore")

    expected_paths = [
        "paper/data",
        "paper/supplementary",
        "paper/scripts",
        "paper/scripts/run-traceability-audit.sh",
        "paper/scripts/run-workflows.sh",
        "paper/scripts/run-normalization.sh",
        "paper/scripts/measure-tokens.py",
    ]
    for rel_path in expected_paths:
        assert (REPO_ROOT / rel_path).exists(), rel_path

    assert "# Paper Package" in readme
    assert "`paper/data/` currently contains placeholder schemas" in readme
    assert "`paper/generated/`" in readme
    assert "`BIOMCP_BIN`" in readme
    assert "`./target/release/biomcp`" in readme
    assert "`biomcp` on `PATH`" in readme
    assert "`run-traceability-audit.sh` is runnable immediately" in readme
    assert "`run-workflows.sh` and `run-normalization.sh` require archived release data" in readme
    assert "paper/generated/" in gitignore


def test_supplementary_stub_contract() -> None:
    expected_tables = {
        "paper/supplementary/table-s1-sources.md": (
            "# Table S1. Leaf-style source enumeration",
            [
                "#",
                "Source",
                "How BioMCP reaches it",
                "Entity surfaces",
                "Verification note",
            ],
        ),
        "paper/supplementary/table-s2-comparison.md": (
            "# Table S2. Landscape comparison matrix",
            [
                "System",
                "Paper / artifact",
                "Year / venue",
                "Source coverage signal",
                "Open source",
                "CLI",
                "MCP",
                "Install surface",
                "Citation count",
                "BioMCP-relevant note",
            ],
        ),
        "paper/supplementary/table-s3-stress-test.md": (
            "# Table S3. Normalization and notation-acceptance stress test",
            [
                "Category",
                "Input",
                "Expected canonical",
                "Resolved",
                "Match note",
            ],
        ),
        "paper/supplementary/table-s4-source-citations.md": (
            "# Table S4. Source citations",
            ["Source", "Citation", "DOI", "PMID", "Status"],
        ),
        "paper/supplementary/table-s5-token-cost.md": (
            "# Table S5. Token-cost measurements",
            [
                "Workflow",
                "Compact tokens",
                "Naive tokens",
                "Token reduction",
                "Compact bytes",
                "Naive bytes",
                "Cold median (s)",
                "Warm median (s)",
            ],
        ),
        "paper/supplementary/table-s6-engineering.md": (
            "# Table S6. Engineering and health metrics",
            ["Metric", "Value", "Method / source"],
        ),
    }

    for rel_path, (heading, columns) in expected_tables.items():
        text = _read(rel_path)
        assert text.startswith(f"{heading}\n")
        assert "## Expected columns" in text
        assert "| Column | Meaning |" in text
        for column in columns:
            assert f"| `{column}` |" in text


def test_data_stub_json_contract() -> None:
    expected_shapes = {
        "paper/data/traceability-audit.json": {
            "_description": str,
            "_is_stub": True,
            "metadata": {
                "audit_label": str,
                "bioMCP_version": str,
                "sample_design": str,
            },
            "summary": {
                "claims": 0,
                "source_labeled": 0,
                "url_present": 0,
                "url_correct": 0,
                "api_datum_match": 0,
                "live_api_datum_match": 0,
                "contract_plus_raw": 0,
                "clickthrough_match": 0,
            },
            "by_entity_type": [],
            "claims": [],
        },
        "paper/data/workflow-adjudication.json": {
            "_description": str,
            "_is_stub": True,
            "metadata": {
                "assessment_label": str,
                "scorer_count": 0,
                "runtime_source": str,
                "timing_reuse_note": str,
            },
            "summary": {
                "workflow_count": 0,
                "passed_all_workflows": 0,
                "passed_all_fraction": 0.0,
            },
            "workflows": [],
        },
        "paper/data/normalization-benchmark.json": {
            "_description": str,
            "_is_stub": True,
            "metadata": {
                "gene_aliases_note": str,
                "drug_brands_note": str,
                "variant_input_parsing_note": str,
            },
            "gene_aliases": {
                "total": 0,
                "resolved": 0,
                "rate_pct": 0.0,
                "results": [],
            },
            "drug_brands": {
                "total": 0,
                "resolved": 0,
                "rate_pct": 0.0,
                "results": [],
            },
            "variant_input_parsing": {
                "total": 0,
                "resolved": 0,
                "rate_pct": 0.0,
                "results": [],
            },
        },
        "paper/data/token-cost.json": {
            "_description": str,
            "_is_stub": True,
            "metadata": {
                "tokenizer": str,
                "secondary_metric": str,
                "note": str,
            },
            "per_workflow": [],
            "totals": {
                "compact_tokens": 0,
                "naive_tokens": 0,
                "token_reduction_pct": 0.0,
                "compact_bytes": 0,
                "naive_bytes": 0,
                "byte_reduction_pct": 0.0,
            },
        },
        "paper/data/conflict-cases.json": {
            "_description": str,
            "_is_stub": True,
            "metadata": {
                "case_count": 0,
                "design_note": str,
            },
            "cases": [],
        },
        "paper/data/health-snapshot.json": {
            "_description": str,
            "_is_stub": True,
            "healthy": 0,
            "total": 0,
            "rows": [],
        },
    }

    for rel_path, expected in expected_shapes.items():
        data = _load_json(rel_path)
        _assert_shape(data, expected)


def test_paper_script_contract() -> None:
    traceability = _read("paper/scripts/run-traceability-audit.sh")
    workflows = _read("paper/scripts/run-workflows.sh")
    normalization = _read("paper/scripts/run-normalization.sh")
    measure_tokens = _read("paper/scripts/measure-tokens.py")

    for rel_path in [
        "paper/scripts/run-traceability-audit.sh",
        "paper/scripts/run-workflows.sh",
        "paper/scripts/run-normalization.sh",
        "paper/scripts/measure-tokens.py",
    ]:
        assert os.access(REPO_ROOT / rel_path, os.X_OK), rel_path

    assert traceability.startswith("#!/usr/bin/env bash\nset -euo pipefail\n")
    assert '"${BIOMCP_BIN:-}"' in traceability
    assert 'DEFAULT_OUTPUT_DIR="$ROOT/paper/generated/traceability"' in traceability
    assert "manifest.tsv" in traceability
    for entity in [
        "gene",
        "variant",
        "trial",
        "article",
        "disease",
        "drug",
        "pathway",
        "pgx",
        "adverse-event",
    ]:
        assert entity in traceability
    assert "paper/data/" not in traceability

    assert workflows.startswith("#!/usr/bin/env bash\nset -euo pipefail\n")
    assert "paper/data/workflow-adjudication.json" in workflows
    assert "Replace paper/data/workflow-adjudication.json with archived release data" in workflows
    assert "paper/generated/workflows" in workflows
    assert "timings.json" in workflows

    assert normalization.startswith("#!/usr/bin/env bash\nset -euo pipefail\n")
    assert "paper/data/normalization-benchmark.json" in normalization
    assert "Replace paper/data/normalization-benchmark.json with archived release data" in normalization
    assert "paper/generated/normalization" in normalization
    assert "manifest.json" in normalization

    assert measure_tokens.startswith("#!/usr/bin/env -S uv run --script")
    assert '# requires-python = ">=3.11"' in measure_tokens
    assert '"tiktoken"' in measure_tokens
    assert 'DEFAULT_INPUT_DIR = "paper/generated/workflows"' in measure_tokens
    assert 'DEFAULT_OUTPUT_PATH = "paper/generated/workflows/token-summary.json"' in measure_tokens


def test_paper_script_smoke_test() -> None:
    """Behavioral smoke tests: actually execute the paper scripts."""
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)

        # Fake BIOMCP_BIN that writes minimal JSON to stdout
        fake_bin = tmp_path / "fake-biomcp"
        fake_bin.write_text("#!/bin/sh\nprintf '{}'\n")
        fake_bin.chmod(fake_bin.stat().st_mode | stat.S_IEXEC | stat.S_IXGRP | stat.S_IXOTH)

        env = {**os.environ, "BIOMCP_BIN": str(fake_bin)}

        # --- run-traceability-audit.sh ---
        out_dir = tmp_path / "traceability-out"
        data_files_before = {p.name for p in (REPO_ROOT / "paper/data").iterdir()}

        result = subprocess.run(
            ["bash", str(REPO_ROOT / "paper/scripts/run-traceability-audit.sh"), str(out_dir)],
            env=env,
            capture_output=True,
        )
        assert result.returncode == 0, result.stderr.decode()

        # Nine entity captures plus manifest.tsv
        assert (out_dir / "manifest.tsv").exists()
        json_captures = list(out_dir.glob("*.json"))
        assert len(json_captures) == 9, [p.name for p in json_captures]

        # paper/data/ must not have been modified
        data_files_after = {p.name for p in (REPO_ROOT / "paper/data").iterdir()}
        assert data_files_before == data_files_after

        # --- run-workflows.sh must refuse stub data ---
        result = subprocess.run(
            ["bash", str(REPO_ROOT / "paper/scripts/run-workflows.sh")],
            env=env,
            capture_output=True,
        )
        assert result.returncode != 0
        assert b"Replace paper/data/workflow-adjudication.json" in result.stderr

        # --- run-normalization.sh must refuse stub data ---
        result = subprocess.run(
            ["bash", str(REPO_ROOT / "paper/scripts/run-normalization.sh")],
            env=env,
            capture_output=True,
        )
        assert result.returncode != 0
        assert b"Replace paper/data/normalization-benchmark.json" in result.stderr
