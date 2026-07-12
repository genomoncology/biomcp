from __future__ import annotations

from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
JSON_ERROR_CONTRACT = REPO_ROOT / "tests" / "json_error_contract.rs"


def test_mygene_fixture_lifetime_is_owned_by_bounded_child_command() -> None:
    source = JSON_ERROR_CONTRACT.read_text(encoding="utf-8")
    fixture_impl = source.split("impl MyGeneFixture {", 1)[1].split(
        "impl Drop for MyGeneFixture", 1
    )[0]

    assert "let deadline = Instant::now() + Duration::from_secs(10);" in source
    assert "biomcp timed out after 10s" in source
    assert "Instant" not in fixture_impl
    assert "deadline" not in fixture_impl
    assert "fixture received no request within 5s" not in source
    assert "recv_timeout(POST_CHILD_FIXTURE_RESULT_TIMEOUT)" in fixture_impl

    gene_contract = source.split(
        "fn json_mode_gene_not_found_error_writes_json_stdout_and_exit_1()", 1
    )[1].split("\n#[test]", 1)[0]
    assert gene_contract.index("run_biomcp_with_env(") < gene_contract.index(
        "fixture.received_request()"
    )

    drop_impl = source.split("impl Drop for MyGeneFixture", 1)[1].split(
        "fn serve_mygene_request", 1
    )[0]
    assert "self.stop.store(true, Ordering::Relaxed);" in drop_impl
    assert "thread.join()" in drop_impl
