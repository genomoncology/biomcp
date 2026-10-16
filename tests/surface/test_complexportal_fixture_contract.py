from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]


def _read_repo(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def test_complexportal_fixture_enforces_p15056_search_request_contract() -> None:
    script = _read_repo("spec/fixtures/setup-complexportal-spec-fixture.sh")
    protein_spec = _read_repo("spec/entity/protein.md")
    runner = _read_repo("scripts/run-specs.sh")
    health_catalog = _read_repo("src/cli/health/catalog.rs")

    assert 'parsed.path != "/search/P15056"' in script
    assert 'number != "25"' in script
    assert 'EXPECTED_FILTER = \'species_f:("Homo sapiens")\'' in script
    assert "BIOMCP_COMPLEXPORTAL_BASE" in script
    assert "BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG" in script

    # 2080: the routine runner starts one replay server instead of per-page
    # fixture scripts, and provider-backed pages (protein.md among them)
    # leave its path list until their recordings exist. The launch pins move
    # to that form: the runner names the replay server and no fixture setup,
    # while the complexportal fixture script stays for the protein page's
    # conversion and still enforces the contract below.
    assert "tools/replay-server" in runner
    assert "setup-complexportal-spec-fixture.sh" not in runner
    assert "spec/entity/protein.md" not in runner
    assert "BIOMCP_COMPLEXPORTAL_FIXTURE_REQUEST_LOG" in protein_spec
    assert 'GET /search/P15056 number=25 filters=species_f:("Homo sapiens")' in protein_spec

    assert 'api: "ComplexPortal"' in health_catalog
    assert 'protein complex membership section' in health_catalog
