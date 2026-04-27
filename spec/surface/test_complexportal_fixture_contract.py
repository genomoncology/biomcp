from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_complexportal_fixture_enforces_p15056_search_request_contract() -> None:
    script = _read_repo("spec/fixtures/setup-complexportal-spec-fixture.sh")
    protein_spec = _read_repo("spec/entity/protein.md")
    health_catalog = _read_repo("src/cli/health/catalog.rs")

    assert 'parsed.path != "/search/P15056"' in script
    assert 'number != "25"' in script
    assert 'EXPECTED_FILTER = \'species_f:("Homo sapiens")\'' in script
    assert "BIOMCP_COMPLEXPORTAL_BASE" in script
    assert "BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG" in script

    assert "setup-complexportal-spec-fixture.sh" in protein_spec
    assert "BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG" in protein_spec
    assert 'GET /search/P15056 number=25 filters=species_f:("Homo sapiens")' in protein_spec

    assert 'api: "ComplexPortal"' in health_catalog
    assert 'protein complex membership section' in health_catalog
