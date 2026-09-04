from __future__ import annotations

import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

import pytest

REPO_ROOT = Path(__file__).resolve().parents[1]
AUDIT_TOOL = REPO_ROOT / "tools" / "check-source-capture-receipts.py"
SOURCES_ROOT = REPO_ROOT / "testdata" / "sources"


def _audit(source_root: Path) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [sys.executable, str(AUDIT_TOOL), "--root", str(source_root), "--json"],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )


def test_clingen_live_replacements_have_receipted_manifest_summary_and_detail_captures() -> (
    None
):
    manifest = json.loads(
        (SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8")
    )
    classifications = {
        entry["path"]: entry["classification"] for entry in manifest["entries"]
    }

    assert (
        classifications.get("clingen_cspec/atm-manifest.json") == "real_and_receipted"
    )
    assert (
        classifications.get("clingen_cspec/atm-gn020-1.5.1.json")
        == "real_and_receipted"
    )
    assert (
        classifications.get("clingen_cspec/pten-gn003-3.2.1.json")
        == "real_and_receipted"
    )
    assert classifications.get("clingen_erepo/apc-summary.json") == "real_and_receipted"
    assert classifications.get("clingen_erepo/apc-detail.json") == "real_and_receipted"
    assert (
        classifications.get("clingen_erepo/pten-gene-limit-26.json")
        == "real_and_receipted"
    )


def test_clingen_car_and_ldh_live_replacements_have_receipted_captures() -> None:
    manifest = json.loads(
        (SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8")
    )
    classifications = {
        entry["path"]: entry["classification"] for entry in manifest["entries"]
    }

    expected_paths = {
        "clingen_allele_registry/tp53-nm_000546.6-c.215c-g.json",
        "clingen_allele_registry/tp53-nm_000546.6-c.215c-g-empty.json",
        "clingen_allele_registry/tp53-nm_000546.6-c.215c-g-malformed.json",
        "clingen_ldh/ca288251-medium.json",
        "clingen_ldh/ca288251-medium-empty.json",
        "clingen_ldh/ca288251-pmc8710334-direct.json",
        "clingen_ldh/ca288251-pmc8710334-direct-malformed.json",
    }

    assert {
        path
        for path in expected_paths
        if classifications.get(path) == "real_and_receipted"
    } == expected_paths


def test_article_663_source_contract_captures_are_receipted() -> None:
    manifest = json.loads(
        (SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8")
    )
    classifications = {
        entry["path"]: entry["classification"] for entry in manifest["entries"]
    }
    expected_paths = {
        "europepmc/pmc3040717-supplementary-not-open-access.xml",
        "europepmc/search_pmid_20516115.json",
        "ncbi_efetch/pmc3040717.xml",
        "pmc_article/pmc3040717-supplementary-tables-pow.html",
        "pmc_article/pmc3040717.html",
        "pmc_oa/pmc3040717-not-open-access.xml",
        "pmc_oa/pmc3040717-versions.xml",
        "pmc_oa/pmc3040717.1.json",
        "pmc_oa/pmc3040717.1.xml",
        "pubtator/export_20516115.json",
        "semantic_scholar/pmid20516115-batch.json",
        "semantic_scholar/pmid20516115-citations.json",
        "semantic_scholar/pmid20516115-recommendations.json",
        "semantic_scholar/pmid20516115-references.json",
    }

    assert {
        path
        for path in expected_paths
        if classifications.get(path) == "real_and_receipted"
    } == expected_paths


def test_seven_variant_article_corpus_maps_each_landmark_to_receipted_decoded_evidence() -> (
    None
):
    manifest = json.loads(
        (SOURCES_ROOT / "capture-receipts.json").read_text(encoding="utf-8")
    )
    receipts = {
        entry["path"]: entry["receipt"]
        for entry in manifest["entries"]
        if entry["path"].startswith("variant_articles_683/")
        and entry["classification"] == "real_and_receipted"
    }
    map_data = json.loads(
        (SOURCES_ROOT / "variant_articles_683/panel-landmark-map.json").read_text(
            encoding="utf-8"
        )
    )
    expected_landmarks = {
        "32461654",
        "22799487",
        "11805335",
        "11410501",
        "20516115",
        "21990146",
        "18033691",
        "19142183",
        "19493351",
        "26951660",
        "31433521",
        "17427195",
    }

    assert {
        record["landmark_pmid"] for record in map_data["landmarks"]
    } == expected_landmarks
    assert set(map_data["derived_internal_routes"]) == {
        "strict",
        "pubtator_variant",
        "exact_lexical",
        "source_citation",
        "best_effort_free_text",
    }

    for record in map_data["landmarks"]:
        path = record["capture_path"]
        assert receipts[path]["request"] == record["safe_request"]
        assert (
            hashlib.sha256(record["safe_request"].encode()).hexdigest()
            == record["request_sha256"]
        )

        body = json.loads((SOURCES_ROOT / path).read_text(encoding="utf-8"))
        if record["provider"] == "pubmed":
            observed_pmids = set(body.get("esearchresult", {}).get("idlist", []))
        else:
            observed_pmids = {
                result["pmid"]
                for result in body["resultList"]["result"]
                if "pmid" in result
            }

        assert (record["landmark_pmid"] in observed_pmids) is record["present"]
        if record["present"]:
            assert record["internal_route"] is not None
        else:
            assert record["internal_route"] is None
            assert record["absence_evidence"]["capture_path"] == path

    states = {evidence["state"] for evidence in map_data["state_evidence"]}
    assert {"positive", "empty", "degraded", "not_attempted"} <= states
    degraded = next(
        evidence
        for evidence in map_data["state_evidence"]
        if evidence["state"] == "degraded"
    )
    assert "error" in json.loads(
        (SOURCES_ROOT / degraded["capture_path"]).read_text(encoding="utf-8")
    )
    assert {
        evidence["route"]
        for evidence in map_data["state_evidence"]
        if evidence["state"] == "not_attempted"
    } == {"car", "ldh"}


def test_repository_audit_classifies_every_source_file_and_preserves_erepo_history() -> (
    None
):
    result = _audit(SOURCES_ROOT)

    assert result.returncode == 0, result.stderr
    report = json.loads(result.stdout)
    assert report["classified_files"] == report["audited_files"]
    assert report["fixture_keys_checked"] > 0
    assert report["fixture_key_exceptions"] == 15
    assert report["confirmed_byte_unfaithful"] == 0
    assert set(report["classifications"]) == {
        "authored",
        "real_and_receipted",
        "synthetic_and_ineligible",
        "pending_verification",
    }
    assert sum(report["classifications"].values()) == report["audited_files"]
    assert any(
        correction["path"] == "clingen_erepo/apc-detail.json"
        and correction["status"] == "recaptured"
        for correction in report["historical_corrections"]
    )


def _valid_receipt(body: bytes) -> dict[str, str]:
    return {
        "provider": "Example Provider",
        "request": "https://example.test/v1/record/42",
        "captured_at": "2026-08-03T00:00:00Z",
        "sha256": hashlib.sha256(body).hexdigest(),
        "minimization_or_redaction": "none; bytes are unmodified",
        "provider_origin_statement": "Bytes were recorded from Example Provider before minimization.",
    }


def _write_fixture_contract_repo(
    repository_root: Path,
    rust_object: str,
    *,
    declare_inline: bool = True,
    disk_fixture: object | None = None,
    declare_disk: bool = True,
) -> Path:
    source_root = repository_root / "testdata" / "sources"
    schema = [
        {
            "name": "protocolSection",
            "type": "ProtocolSection",
            "children": [
                {
                    "name": "armsInterventionsModule",
                    "type": "ArmsInterventionsModule",
                    "children": [
                        {
                            "name": "armGroups",
                            "type": "ArmGroup[]",
                            "children": [
                                {"name": "label", "type": "text"},
                                {"name": "type", "type": "ArmGroupType"},
                            ],
                        }
                    ],
                },
                {
                    "name": "contactsLocationsModule",
                    "type": "ContactsLocationsModule",
                    "children": [
                        {"name": "centralContacts", "type": "Contact[]"},
                        {
                            "name": "locations",
                            "type": "Location[]",
                            "children": [{"name": "facility", "type": "text"}],
                        },
                    ],
                },
            ],
        }
    ]
    nci_capture = {"data": [{"nct_id": "NCI-1", "diseases": []}], "total": 1}
    payloads = {
        "ctgov/schema.json": json.dumps(schema).encode(),
        "nci_cts/full.json": json.dumps(nci_capture).encode(),
    }
    entries: list[dict[str, object]] = []
    for path, body in payloads.items():
        destination = source_root / path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(body)
        entries.append(
            {
                "path": path,
                "classification": "real_and_receipted",
                "receipt": _valid_receipt(body),
            }
        )

    on_disk: list[dict[str, str]] = []
    if disk_fixture is not None:
        body = json.dumps(disk_fixture).encode()
        destination = source_root / "clinicaltrials" / "authored.json"
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(body)
        entries.append(
            {
                "path": "clinicaltrials/authored.json",
                "classification": "authored",
                "authored_reason": "Person-bearing values cannot be recorded.",
            }
        )
        parsing = repository_root / "src" / "sources" / "clinicaltrials" / "tests.rs"
        parsing.parent.mkdir(parents=True, exist_ok=True)
        parsing.write_text(
            'include_str!("../../../../testdata/sources/clinicaltrials/authored.json");\n',
            encoding="utf-8",
        )
        if declare_disk:
            on_disk.append(
                {
                    "path": "clinicaltrials/authored.json",
                    "selector": "/",
                    "endpoint": "ctgov",
                }
            )

    rust_path = repository_root / "src" / "transform" / "trial" / "tests.rs"
    rust_path.parent.mkdir(parents=True, exist_ok=True)
    rust_path.write_text(
        "fn fixture() {\n"
        f"  let study = serde_json::from_value(json!({rust_object}));\n"
        "  let _trial = from_ctgov_study(&study);\n"
        "}\n",
        encoding="utf-8",
    )
    inline = (
        [
            {
                "path": "src/transform/trial/tests.rs",
                "selector": "fixture:json:1",
                "endpoint": "ctgov",
            }
        ]
        if declare_inline
        else []
    )
    manifest = {
        "schema_version": 1,
        "entries": entries,
        "fixture_key_contract": {
            "attestors": [
                {
                    "endpoint": "ctgov",
                    "label": "ClinicalTrials.gov test endpoint",
                    "kind": "ctgov_schema",
                    "path": "ctgov/schema.json",
                },
                {
                    "endpoint": "nci",
                    "label": "NCI test endpoint",
                    "kind": "nci_top_level_capture",
                    "path": "nci_cts/full.json",
                    "selector": "/data/*",
                    "limitation": "top-level keys only",
                },
            ],
            "on_disk": on_disk,
            "inline": inline,
            "exceptions": [],
        },
        "confirmed_byte_unfaithful": 0,
        "historical_corrections": [],
    }
    (source_root / "capture-receipts.json").write_text(
        json.dumps(manifest), encoding="utf-8"
    )
    return source_root


@pytest.mark.parametrize(
    ("rust_object", "expected_path"),
    (
        (
            '{"protocolSection": {"armsInterventionsModule": '
            '{"armGroups": [{"label": "A", "armGroupType": "EXPERIMENTAL"}]}}}',
            "protocolSection.armsInterventionsModule.armGroups[].armGroupType",
        ),
        (
            '{"protocolSection": {"contactsLocationsModule": '
            '{"locations": [{"facility": "A", "centralContacts": []}]}}}',
            "protocolSection.contactsLocationsModule.locations[].centralContacts",
        ),
    ),
)
def test_fixture_key_contract_rejects_unknown_and_wrong_path(
    tmp_path: Path, rust_object: str, expected_path: str
) -> None:
    source_root = _write_fixture_contract_repo(tmp_path / "repo", rust_object)

    result = _audit(source_root)

    assert result.returncode != 0
    assert "src/transform/trial/tests.rs" in result.stderr
    assert expected_path in result.stderr
    assert "ClinicalTrials.gov test endpoint" in result.stderr


def _copy_current_trial_contract(repository_root: Path) -> Path:
    shutil.copytree(SOURCES_ROOT, repository_root / "testdata" / "sources")
    for relative in (
        "src/transform/trial/tests.rs",
        "src/transform/trial/tests",
        "src/sources/clinicaltrials/tests/parsing.rs",
        "src/sources/nci_cts/tests/parsing.rs",
    ):
        source = REPO_ROOT / relative
        destination = repository_root / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        if source.is_dir():
            shutil.copytree(source, destination)
        else:
            shutil.copy2(source, destination)
    return repository_root / "testdata" / "sources"


def test_actual_ctgov_arms_fixture_rejects_reintroduced_arm_group_type(
    tmp_path: Path,
) -> None:
    repository_root = tmp_path / "repo"
    source_root = _copy_current_trial_contract(repository_root)
    fixture_path = repository_root / "src" / "transform" / "trial" / "tests.rs"
    source = fixture_path.read_text(encoding="utf-8")
    fixture_path.write_text(
        source.replace('"type": "EXPERIMENTAL"', '"armGroupType": "EXPERIMENTAL"', 1),
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "from_ctgov_study_preserves_provider_type_fields_in_json" in result.stderr
    assert (
        "protocolSection.armsInterventionsModule.armGroups[].armGroupType"
        in result.stderr
    )
    assert "ClinicalTrials.gov v2 study detail" in result.stderr


def test_inline_discovery_ignores_adversarial_comments(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{/* }) from_nci_trial(&json!({"invented": 1})) */ '
        '"protocolSection": {"armsInterventionsModule": '
        '{"armGroups": [{"label": "A", "type": "EXPERIMENTAL"}]}}}',
    )
    rust_path = tmp_path / "repo" / "src" / "transform" / "trial" / "tests.rs"
    source = rust_path.read_text(encoding="utf-8")
    rust_path.write_text(
        source.replace(
            "fn fixture() {",
            'fn fixture() { // }) json!({"invented": 1}) from_nci_hit(\n',
        ),
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode == 0, result.stderr


def test_fixture_key_contract_accepts_schema_path_and_checks_authored_fixture(
    tmp_path: Path,
) -> None:
    valid = {"protocolSection": {"contactsLocationsModule": {"centralContacts": []}}}
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {"armsInterventionsModule": '
        '{"armGroups": [{"label": "A", "type": "EXPERIMENTAL"}]}}}',
        disk_fixture=valid,
    )

    accepted = _audit(source_root)

    assert accepted.returncode == 0, accepted.stderr
    invalid = {
        "protocolSection": {"contactsLocationsModule": {"inventedAuthoredKey": "bad"}}
    }
    (source_root / "clinicaltrials" / "authored.json").write_text(
        json.dumps(invalid), encoding="utf-8"
    )
    rejected = _audit(source_root)
    assert rejected.returncode != 0
    assert "clinicaltrials/authored.json" in rejected.stderr
    assert (
        "protocolSection.contactsLocationsModule.inventedAuthoredKey" in rejected.stderr
    )


def test_fixture_key_contract_fails_closed_on_undeclared_inline_fixture(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}', declare_inline=False
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "fixture:json:1" in result.stderr
    assert "inline fixture is undeclared" in result.stderr


def test_inline_discovery_follows_local_reassignment_before_converter_call(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}'
    )
    rust_path = tmp_path / "repo" / "src" / "transform" / "trial" / "tests.rs"
    rust_path.write_text(
        "fn fixture() {\n"
        "  let mut record = serde_json::Value::Null;\n"
        '  record = json!({"inventedNciKey": 1});\n'
        "  let _trial = from_nci_trial(&record);\n"
        "}\n",
        encoding="utf-8",
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["inline"][0]["endpoint"] = "nci"
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert "fixture:json:1" in result.stderr
    assert "inventedNciKey" in result.stderr
    assert "NCI test endpoint" in result.stderr


def test_inline_discovery_does_not_treat_comparison_as_reassignment(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}'
    )
    rust_path = tmp_path / "repo" / "src" / "transform" / "trial" / "tests.rs"
    rust_path.write_text(
        "fn fixture() {\n"
        '  let record = json!({"inventedNciKey": 1});\n'
        "  let _same = record == json!({});\n"
        "  let _trial = from_nci_trial(&record);\n"
        "}\n",
        encoding="utf-8",
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["inline"][0]["endpoint"] = "nci"
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert "inventedNciKey" in result.stderr
    assert "NCI test endpoint" in result.stderr


def test_inline_discovery_fails_closed_on_unsupported_converter_argument(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}'
    )
    rust_path = tmp_path / "repo" / "src" / "transform" / "trial" / "tests.rs"
    rust_path.write_text(
        "fn fixture() {\n"
        '  let wrapper = json!({"study": {"nct_id": "NCI-1"}});\n'
        '  let _trial = from_nci_trial(&wrapper["study"]);\n'
        "}\n",
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "src/transform/trial/tests.rs:fixture" in result.stderr
    assert "unsupported argument into from_nci_trial" in result.stderr


def test_fixture_key_contract_fails_closed_on_undeclared_consumed_file(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
        declare_disk=False,
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "clinicaltrials/authored.json" in result.stderr
    assert "consumed trial fixture is undeclared" in result.stderr


def test_fixture_key_contract_rejects_dynamic_consumed_fixture_reference(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
    )
    parsing = tmp_path / "repo" / "src" / "sources" / "clinicaltrials" / "tests.rs"
    parsing.write_text(
        'const ROOT: &str = "/testdata/sources/clinicaltrials/";\n'
        "fn dynamic(name: &str) { let _ = fixture!(name); }\n",
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "dynamic fixture! reference is unsupported" in result.stderr
    assert "clinicaltrials" in result.stderr


def test_consumed_fixture_discovery_assembles_split_concat_literal_path(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
    )
    parsing = tmp_path / "repo" / "src" / "sources" / "clinicaltrials" / "tests.rs"
    parsing.write_text(
        'const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), '
        '"/testdata/sources/", "clinicaltrials/", "authored.json"));\n',
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode == 0, result.stderr


def test_consumed_fixture_discovery_rejects_dynamic_split_concat_path(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
    )
    parsing = tmp_path / "repo" / "src" / "sources" / "clinicaltrials" / "tests.rs"
    parsing.write_text(
        'const DIRECTORY: &str = "/testdata/sources/clinicaltrials/";\n'
        'const NAME: &str = "authored.json";\n'
        'const _: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), DIRECTORY, NAME));\n',
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "dynamic include fixture reference is unsupported" in result.stderr


def test_fixture_key_contract_rejects_duplicate_on_disk_declaration(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    declaration = manifest["fixture_key_contract"]["on_disk"][0]
    manifest["fixture_key_contract"]["on_disk"].append(dict(declaration))
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert "duplicate on-disk fixture declaration" in result.stderr


def test_fixture_key_contract_rejects_scalar_selected_record(tmp_path: Path) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture="not a trial object",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "selector / selected a non-object trial record" in result.stderr


def test_fixture_key_contract_rejects_unattested_nci_top_level_key(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}'
    )
    rust_path = tmp_path / "repo" / "src" / "transform" / "trial" / "tests.rs"
    rust_path.write_text(
        "fn fixture() {\n"
        '  let record = json!({"inventedNciKey": 1});\n'
        "  let _trial = from_nci_trial(&record);\n"
        "}\n",
        encoding="utf-8",
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["inline"][0]["endpoint"] = "nci"
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert "inventedNciKey" in result.stderr
    assert "NCI test endpoint" in result.stderr


def test_fixture_key_contract_rejects_altered_authorized_exception_reason(
    tmp_path: Path,
) -> None:
    source_root = _copy_current_trial_contract(tmp_path / "repo")
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["exceptions"][0]["reason"] += " Altered."
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert "fixture-key exception has unauthorized reason" in result.stderr


def test_fixture_key_contract_rejects_extra_used_exception(tmp_path: Path) -> None:
    repository_root = tmp_path / "repo"
    source_root = _copy_current_trial_contract(repository_root)
    rust_path = repository_root / "src" / "transform" / "trial" / "tests.rs"
    source = rust_path.read_text(encoding="utf-8")
    rust_path.write_text(
        source.replace(
            '"diseases": ["Melanoma"]',
            '"diseases": ["Melanoma"], "extraLegacyAlias": true',
            1,
        ),
        encoding="utf-8",
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["exceptions"].append(
        {
            "path": "src/transform/trial/tests.rs",
            "selector": "from_nci_trial_maps_supported_alias_fields:json:1",
            "checked_path": "extraLegacyAlias",
            "reason": (
                "Synthetic unit input pins an accepted legacy alias and does not attest the "
                "NCI wire contract; ticket 1138 owns its code-side disposition."
            ),
        }
    )
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert (
        "fixture-key exceptions differ from the authorized ticket 1126 set"
        in result.stderr
    )


def _write_real_capture_inventory(
    source_root: Path, body: bytes, receipt: dict[str, str]
) -> None:
    payload = source_root / "example" / "record.json"
    payload.parent.mkdir(parents=True)
    payload.write_bytes(body)
    (source_root / "capture-receipts.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "entries": [
                    {
                        "path": "example/record.json",
                        "classification": "real_and_receipted",
                        "receipt": receipt,
                    }
                ],
                "historical_corrections": [],
            }
        ),
        encoding="utf-8",
    )


@pytest.mark.parametrize(
    "missing_field",
    (
        "provider",
        "request",
        "captured_at",
        "sha256",
        "minimization_or_redaction",
        "provider_origin_statement",
    ),
)
def test_real_capture_receipts_reject_every_missing_required_field(
    tmp_path: Path, missing_field: str
) -> None:
    body = b'{"record": 42}\n'
    receipt = _valid_receipt(body)
    del receipt[missing_field]
    source_root = tmp_path / "sources"
    _write_real_capture_inventory(source_root, body, receipt)

    result = _audit(source_root)

    assert result.returncode != 0
    assert missing_field in result.stderr


def test_real_capture_receipts_reject_byte_drift(tmp_path: Path) -> None:
    body = b'{"record": 42}\n'
    receipt = _valid_receipt(body)
    receipt["sha256"] = "0" * 64
    source_root = tmp_path / "sources"
    _write_real_capture_inventory(source_root, body, receipt)

    result = _audit(source_root)

    assert result.returncode != 0
    assert "sha256" in result.stderr


@pytest.mark.parametrize(
    ("field", "value", "error"),
    (
        (
            "request",
            "https://storage.googleapis.com/object?X-Goog-Signature=secret",
            "unsafe",
        ),
        ("request", "https://example.test/record#opaque-fragment", "unsafe"),
        ("captured_at", "2026-08-03 00:00:00Z", "RFC3339 UTC"),
    ),
)
def test_real_capture_receipts_reject_unsafe_request_and_non_rfc3339_timestamp(
    tmp_path: Path, field: str, value: str, error: str
) -> None:
    body = b'{"record": 42}\n'
    receipt = _valid_receipt(body)
    receipt[field] = value
    source_root = tmp_path / "sources"
    _write_real_capture_inventory(source_root, body, receipt)

    result = _audit(source_root)

    assert result.returncode != 0
    assert error in result.stderr


def test_repository_audit_does_not_ignore_nested_manifest_named_fixture(
    tmp_path: Path,
) -> None:
    source_root = tmp_path / "sources"
    nested_fixture = source_root / "example" / "capture-receipts.json"
    nested_fixture.parent.mkdir(parents=True)
    nested_fixture.write_text('{"provider": "Example"}\n', encoding="utf-8")
    (source_root / "capture-receipts.json").write_text(
        json.dumps(
            {
                "schema_version": 1,
                "entries": [],
                "historical_corrections": [],
            }
        ),
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "example/capture-receipts.json" in result.stderr
