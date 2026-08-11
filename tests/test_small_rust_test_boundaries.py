from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def test_provider_capacity_uses_an_injected_limit_and_small_fixture() -> None:
    source = (ROOT / "src/cache/provider_capture.rs").read_text()

    assert "max_retained_bytes: u64" in source
    assert "fn plan_capacity_evictions(" in source
    assert "with_max_retained_bytes" in source
    capacity_test = source.split(
        "fn enforces_namespace_capacity_with_deterministic_lru_eviction()", 1
    )[1].split("\n    #[test]", 1)[0]
    assert "MAX_CAPTURE_BYTES" not in capacity_test
    assert "1..17" not in capacity_test


def test_session_capacity_is_planned_in_memory_without_1030_rewrites() -> None:
    source = (ROOT / "src/cli/article/session.rs").read_text()
    tests = (ROOT / "src/cli/article/session/tests.rs").read_text()

    assert "fn prune_sessions(" in source
    assert "max_active_sessions: usize" in source
    assert "0..1_030" not in tests
    assert "capacity_pruning_resolves_ties_stably_in_memory" in tests
    assert "capacity_pruning_covers_exact_and_plus_one_limits" in tests


def test_fulltext_classification_matrix_uses_a_no_retry_client() -> None:
    source = (ROOT / "src/entities/article/fulltext.rs").read_text()

    matrix = source.split(
        "async fn html_and_pdf_attempts_classify_transport_and_conversion_failures()", 1
    )[1].split("\n    #[test]", 1)[0]
    assert "test_client" in matrix
    assert "html_with_client" in matrix
    assert "pdf_with_client" in matrix
