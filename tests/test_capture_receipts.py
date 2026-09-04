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
    assert report["fixture_key_exceptions"] == 0
    assert report["code_keys_checked"] == 124
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
    disk_directory: str = "clinicaltrials",
    disk_selector: str = "/",
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
                            "children": [
                                {"name": "facility", "type": "text"},
                                {"name": "geoPoint", "type": "GeoPoint"},
                            ],
                        },
                    ],
                },
            ],
        }
    ]
    nci_capture = {
        "data": [
            {
                "nct_id": "NCI-1",
                "brief_title": "Trial",
                "diseases": [],
                "eligibility": {},
            }
        ],
        "total": 1,
    }
    ctgov_full = {
        "protocolSection": {
            "contactsLocationsModule": {
                "locations": [{"geoPoint": {"lat": 1.0, "lon": 2.0}}]
            }
        }
    }
    payloads = {
        "ctgov/schema.json": json.dumps(schema).encode(),
        "ctgov/get_nct06131398_full_20260903.json": json.dumps(ctgov_full).encode(),
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
        fixture_path = f"{disk_directory}/authored.json"
        destination = source_root / fixture_path
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(body)
        entries.append(
            {
                "path": fixture_path,
                "classification": "authored",
                "authored_reason": "Person-bearing values cannot be recorded.",
            }
        )
        parsing = repository_root / "src" / "sources" / "clinicaltrials" / "tests.rs"
        parsing.parent.mkdir(parents=True, exist_ok=True)
        parsing.write_text(
            f'include_str!("../../../../testdata/sources/{fixture_path}");\n',
            encoding="utf-8",
        )
        if declare_disk:
            on_disk.append(
                {
                    "path": fixture_path,
                    "selector": disk_selector,
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
    ctgov_source = repository_root / "src" / "sources" / "clinicaltrials.rs"
    ctgov_source.parent.mkdir(parents=True, exist_ok=True)
    ctgov_source.write_text(
        '#[serde(rename_all = "camelCase")]\n'
        "pub struct CtGovStudy { pub protocol_section: Option<CtGovProtocolSection>, }\n"
        '#[serde(rename_all = "camelCase")]\n'
        "pub struct CtGovProtocolSection { pub contacts_locations_module: Option<CtGovContactsLocationsModule>, }\n"
        '#[serde(rename_all = "camelCase")]\n'
        "pub struct CtGovContactsLocationsModule { pub central_contacts: Vec<String>, pub locations: Vec<CtGovLocation>, }\n"
        '#[serde(rename_all = "camelCase")]\n'
        "pub struct CtGovLocation { pub facility: Option<String>, pub geo_point: Option<CtGovGeoPoint>, }\n"
        "pub struct CtGovGeoPoint { pub lat: Option<f64>, pub lon: Option<f64>, }\n",
        encoding="utf-8",
    )
    transform_source = repository_root / "src" / "transform" / "trial.rs"
    transform_source.parent.mkdir(parents=True, exist_ok=True)
    transform_source.write_text(
        'fn from_nci_hit(hit: &Value) { json_get_string(hit, &["nct_id", "brief_title"]); }\n'
        'fn from_nci_trial(trial: &Value) { nci_conditions(trial, &["diseases"]); }\n',
        encoding="utf-8",
    )
    get_source = repository_root / "src" / "entities" / "trial" / "get.rs"
    get_source.parent.mkdir(parents=True, exist_ok=True)
    get_source.write_text(
        'fn nci_eligibility_text(trial: &Value) { trial.get("eligibility"); }\n',
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
        "code_key_contract": {
            "boundaries": [
                {
                    "endpoint": "ctgov",
                    "source": "src/sources/clinicaltrials.rs",
                    "root_type": "CtGovStudy",
                },
                {
                    "endpoint": "nci",
                    "source": "src/transform/trial.rs",
                    "function": "from_nci_hit",
                    "root_parameter": "hit",
                },
                {
                    "endpoint": "nci",
                    "source": "src/transform/trial.rs",
                    "function": "from_nci_trial",
                    "root_parameter": "trial",
                },
                {
                    "endpoint": "nci",
                    "source": "src/entities/trial/get.rs",
                    "function": "nci_eligibility_text",
                    "root_parameter": "trial",
                },
            ],
            "supplemental_attestations": [
                {
                    "endpoint": "ctgov",
                    "path": "protocolSection.contactsLocationsModule.locations[].geoPoint.lat",
                    "limitation": "The recorded CTGov schema exposes geoPoint as an opaque GeoPoint leaf.",
                    "evidence_path": "ctgov/get_nct06131398_full_20260903.json",
                },
                {
                    "endpoint": "ctgov",
                    "path": "protocolSection.contactsLocationsModule.locations[].geoPoint.lon",
                    "limitation": "The recorded CTGov schema exposes geoPoint as an opaque GeoPoint leaf.",
                    "evidence_path": "ctgov/get_nct06131398_full_20260903.json",
                },
            ],
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
        "src/sources/clinicaltrials.rs",
        "src/transform/trial/tests.rs",
        "src/transform/trial/tests",
        "src/transform/trial.rs",
        "src/entities/trial/get.rs",
        "src/entities/trial/get/tests.rs",
        "src/entities/trial/search/ctgov/tests.rs",
        "src/entities/trial/search/nci/tests.rs",
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


def test_fixture_key_contract_discovers_declared_ctgov_capture(tmp_path: Path) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
        disk_directory="ctgov",
    )

    result = _audit(source_root)

    assert result.returncode == 0, result.stderr


def test_fixture_key_contract_accepts_ctgov_search_record_selector(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"studies": [{"protocolSection": {}}]},
        disk_directory="ctgov",
        disk_selector="/studies/*",
    )

    result = _audit(source_root)

    assert result.returncode == 0, result.stderr


def test_fixture_key_contract_rejects_undeclared_consumed_ctgov_capture(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {}}',
        disk_fixture={"protocolSection": {}},
        disk_directory="ctgov",
        declare_disk=False,
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "ctgov/authored.json: consumed trial fixture is undeclared" in result.stderr


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


def test_fixture_key_contract_rejects_dynamic_ctgov_fixture_reference(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}'
    )
    parsing = tmp_path / "repo" / "src" / "sources" / "clinicaltrials" / "tests.rs"
    parsing.parent.mkdir(parents=True, exist_ok=True)
    parsing.write_text(
        'const ROOT: &str = "/testdata/sources/ctgov/";\n'
        "fn dynamic(name: &str) { let _ = fixture!(name); }\n",
        encoding="utf-8",
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "dynamic fixture! reference is unsupported for ctgov" in result.stderr


def test_fixture_key_contract_accepts_closed_ctgov_geo_supplement(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {"contactsLocationsModule": {"locations": '
        '[{"geoPoint": {"lat": 1.0, "lon": 2.0}}]}}}',
    )

    result = _audit(source_root)

    assert result.returncode == 0, result.stderr


def test_fixture_key_contract_rejects_unknown_opaque_geo_child(tmp_path: Path) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo",
        '{"protocolSection": {"contactsLocationsModule": {"locations": '
        '[{"geoPoint": {"altitude": 1.0}}]}}}',
    )

    result = _audit(source_root)

    assert result.returncode != 0
    assert "geoPoint.altitude" in result.stderr


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


def test_fixture_key_contract_rejects_any_exception(
    tmp_path: Path,
) -> None:
    source_root = _write_fixture_contract_repo(
        tmp_path / "repo", '{"protocolSection": {}}'
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["exceptions"].append(
        {
            "path": "src/transform/trial/tests.rs",
            "selector": "fixture:json:1",
            "checked_path": "invented",
            "reason": "No exceptions are authorized.",
        }
    )
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")

    result = _audit(source_root)

    assert result.returncode != 0
    assert "fixture-key exceptions are closed and must be empty" in result.stderr


def test_fixture_key_contract_rejects_extra_used_exception(tmp_path: Path) -> None:
    repository_root = tmp_path / "repo"
    source_root = _write_fixture_contract_repo(
        repository_root, '{"protocolSection": {}}'
    )
    rust_path = repository_root / "src" / "transform" / "trial" / "tests.rs"
    source = rust_path.read_text(encoding="utf-8")
    rust_path.write_text(
        source.replace(
            '"protocolSection": {}',
            '"protocolSection": {}, "extraLegacyAlias": true',
            1,
        ),
        encoding="utf-8",
    )
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    manifest["fixture_key_contract"]["exceptions"].append(
        {
            "path": "src/transform/trial/tests.rs",
            "selector": "fixture:json:1",
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
    assert "fixture-key exceptions are closed and must be empty" in result.stderr


def _mutate_current_contract(
    tmp_path: Path, relative: str, old: str, new: str
) -> subprocess.CompletedProcess[str]:
    repository_root = tmp_path / "repo"
    source_root = _copy_current_trial_contract(repository_root)
    path = repository_root / relative
    source = path.read_text(encoding="utf-8")
    assert old in source
    path.write_text(source.replace(old, new, 1), encoding="utf-8")
    return _audit(source_root)


@pytest.mark.parametrize(
    ("old", "new", "expected"),
    (
        ('&["nct_id"]', '&["nct_id", "nctId"]', "nctId"),
        ('&["nct_id"]', '&["invented_only"]', "invented_only"),
    ),
)
def test_code_key_contract_checks_each_nci_alternative_independently(
    tmp_path: Path, old: str, new: str, expected: str
) -> None:
    result = _mutate_current_contract(tmp_path, "src/transform/trial.rs", old, new)
    assert result.returncode != 0
    assert "nci:src/transform/trial.rs:from_nci_hit" in result.stderr
    assert "json_get_string#1" in result.stderr
    assert expected in result.stderr


def test_code_key_contract_rejects_unattested_direct_root_get(tmp_path: Path) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/entities/trial/get.rs",
        'trial.get("eligibility")',
        'trial.get("inventedEligibility")',
    )
    assert result.returncode != 0
    assert "nci_eligibility_text" in result.stderr
    assert "get#1" in result.stderr
    assert "inventedEligibility" in result.stderr


@pytest.mark.parametrize(
    ("old", "new", "expected"),
    (
        (
            '#[serde(rename = "type")]\n    pub intervention_type',
            "pub intervention_type",
            "interventionType",
        ),
        (
            '#[serde(rename = "type")]\n    pub arm_group_type',
            "pub arm_group_type",
            "armGroupType",
        ),
        (
            '#[serde(rename = "type")]\n    pub reference_type',
            "pub reference_type",
            "referenceType",
        ),
        (
            "pub contacts: Vec<CtGovContact>,\n    pub geo_point",
            "pub contacts: Vec<CtGovContact>,\n    pub central_contacts: Vec<CtGovContact>,\n    pub geo_point",
            "protocolSection.contactsLocationsModule.locations[].centralContacts",
        ),
    ),
)
def test_code_key_contract_rejects_historical_ctgov_wrong_paths(
    tmp_path: Path, old: str, new: str, expected: str
) -> None:
    result = _mutate_current_contract(
        tmp_path, "src/sources/clinicaltrials.rs", old, new
    )
    assert result.returncode != 0
    assert "CtGovStudy" in result.stderr
    assert expected in result.stderr


def test_code_key_contract_accepts_module_level_central_contacts(
    tmp_path: Path,
) -> None:
    source_root = _copy_current_trial_contract(tmp_path / "repo")
    result = _audit(source_root)
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    ("replacement", "expected"),
    (
        (
            'let alias = hit; json_get_string(alias, &["nct_id"]);',
            "unsupported root access",
        ),
        ('let _ = hit["nct_id"].clone();', "unsupported root access"),
        ('let _ = hit.pointer("/nct_id");', "unsupported root access"),
        ('let key = "nct_id"; let _ = hit.get(key);', "computed key"),
        ("let _ = unknown_helper(hit);", "unsupported root access"),
    ),
)
def test_code_key_contract_fails_closed_on_unsupported_nci_root_forms(
    tmp_path: Path, replacement: str, expected: str
) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/transform/trial.rs",
        'let nct_id = json_get_string(hit, &["nct_id"]).unwrap_or_default();',
        replacement,
    )
    assert result.returncode != 0
    assert "from_nci_hit" in result.stderr
    assert expected in result.stderr


def test_code_key_contract_checks_root_of_chained_read_only(tmp_path: Path) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/entities/trial/get.rs",
        'trial.get("eligibility")',
        'trial.get("eligibility").and_then(|value| value.get("not_top_level"))',
    )
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize("change", ("missing", "altered", "duplicate", "extra"))
def test_code_key_contract_boundary_is_closed(tmp_path: Path, change: str) -> None:
    source_root = _copy_current_trial_contract(tmp_path / "repo")
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    boundaries = manifest["code_key_contract"]["boundaries"]
    if change == "missing":
        boundaries.pop()
    elif change == "altered":
        boundaries[1]["function"] = "not_a_function"
    elif change == "duplicate":
        boundaries.append(dict(boundaries[0]))
    else:
        boundaries.append(
            {
                "endpoint": "nci",
                "source": "src/transform/trial.rs",
                "function": "extra",
                "root_parameter": "trial",
            }
        )
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
    result = _audit(source_root)
    assert result.returncode != 0
    assert "code-key boundar" in result.stderr


@pytest.mark.parametrize(
    "change", ("missing", "altered", "limitation", "duplicate", "extra")
)
def test_code_key_contract_supplemental_attestations_are_closed(
    tmp_path: Path, change: str
) -> None:
    source_root = _copy_current_trial_contract(tmp_path / "repo")
    manifest_path = source_root / "capture-receipts.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    supplements = manifest["code_key_contract"]["supplemental_attestations"]
    if change == "missing":
        supplements.pop()
    elif change == "altered":
        supplements[0]["evidence_path"] = "ctgov/wrong.json"
    elif change == "limitation":
        supplements[0]["limitation"] = "Opaque enough to sound plausible."
    elif change == "duplicate":
        supplements.append(dict(supplements[0]))
    else:
        supplements.append(
            {
                "endpoint": "ctgov",
                "path": "protocolSection.invented",
                "limitation": "opaque schema leaf",
                "evidence_path": "ctgov/get_nct06131398_full_20260903.json",
            }
        )
    manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
    result = _audit(source_root)
    assert result.returncode != 0
    assert "supplement" in result.stderr.lower()
    if change == "limitation":
        assert "altered supplemental declaration" in result.stderr


def test_code_key_discovery_ignores_commented_fake_reads_and_declarations(
    tmp_path: Path,
) -> None:
    source_root = _copy_current_trial_contract(tmp_path / "repo")
    path = tmp_path / "repo" / "src" / "transform" / "trial.rs"
    source = path.read_text(encoding="utf-8")
    path.write_text(
        '// fn from_nci_hit(hit: &Value) { hit.get("invented"); }\n'
        '/* fn from_nci_hit(hit: &Value) { hit.get("inventedBlock"); } */\n'
        'const FAKE: &str = r#"fn from_nci_hit(hit: &Value) { hit.get("invented"); }"#;\n'
        'const FAKE_BYTES: &[u8] = br#"hit.get("inventedBytes")"#;\n'
        "const FAKE_CHAR: char = '}';\n" + source,
        encoding="utf-8",
    )
    result = _audit(source_root)
    assert result.returncode == 0, result.stderr


def test_code_key_discovery_rejects_unclosed_covered_construct(tmp_path: Path) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/entities/trial/get.rs",
        'trial.get("eligibility")',
        'trial.get("eligibility"',
    )
    assert result.returncode != 0
    assert "unclosed" in result.stderr


def test_code_key_discovery_requires_declared_root_parameter_in_source(
    tmp_path: Path,
) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/entities/trial/get.rs",
        "fn nci_eligibility_text(trial: &serde_json::Value)",
        "fn nci_eligibility_text(record: &serde_json::Value)",
    )
    assert result.returncode != 0
    assert "declared root parameter trial does not exist" in result.stderr


def test_ctgov_attribute_prefix_ignores_comment_delimiters(tmp_path: Path) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/sources/clinicaltrials.rs",
        '#[serde(rename_all = "camelCase")]\npub struct CtGovStudy',
        '#[serde(alias = "inventedStudy")]\n/* } ; */\n'
        '#[serde(rename_all = "camelCase")]\npub struct CtGovStudy',
    )
    assert result.returncode != 0
    assert "CtGovStudy: unsupported struct serde attribute" in result.stderr


def test_nci_direct_get_treats_comments_as_whitespace(tmp_path: Path) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/entities/trial/get.rs",
        'trial.get("eligibility")',
        'trial.get(/* provider top-level key */ "eligibility")',
    )
    assert result.returncode == 0, result.stderr


def test_nci_helper_does_not_discover_commented_key_literal(tmp_path: Path) -> None:
    result = _mutate_current_contract(
        tmp_path,
        "src/transform/trial.rs",
        'json_get_string(hit, &["nct_id"])',
        'json_get_string(hit, &["nct_id", /* "inventedCommentKey" */])',
    )
    assert result.returncode == 0, result.stderr


@pytest.mark.parametrize(
    ("old", "new"),
    (
        (
            'json_get_string(hit, &["nct_id"])',
            'json_get_string(/* hit */ hit, &["nct_id"])',
        ),
        (
            'nci_conditions(hit, &["diseases"])',
            'nci_conditions(/* hit */ hit, &["diseases"])',
        ),
    ),
)
def test_nci_helpers_track_live_root_after_comment(
    tmp_path: Path, old: str, new: str
) -> None:
    result = _mutate_current_contract(tmp_path, "src/transform/trial.rs", old, new)
    assert result.returncode == 0, result.stderr


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
