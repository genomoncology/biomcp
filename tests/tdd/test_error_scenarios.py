"""Tests for error scenarios and edge cases - fixed version."""

import asyncio
from unittest.mock import MagicMock, patch

import pytest

from biomcp.exceptions import (
    InvalidDomainError,
)
from biomcp.rate_limiter import RateLimiter
from biomcp.router import format_results


@pytest.fixture(autouse=True)
def enable_metrics_for_concurrent_test(monkeypatch):
    """Enable metrics for concurrent test."""
    monkeypatch.setenv("BIOMCP_METRICS_ENABLED", "true")
    # Force reload of the module to pick up the new env var
    import importlib

    import biomcp.metrics

    importlib.reload(biomcp.metrics)


def test_format_results_invalid_domain():
    """Test format_results with invalid domain."""
    with pytest.raises(InvalidDomainError) as exc_info:
        format_results([], "invalid_domain", 1, 10, 100)

    assert "invalid_domain" in str(exc_info.value)
    assert "Valid domains are:" in str(exc_info.value)


def test_format_results_handler_exception():
    """Test format_results when handler raises exception."""
    # Create a result that will cause formatting to fail
    bad_result = {"missing": "required_fields"}

    with patch(
        "biomcp.domain_handlers.ArticleHandler.format_result"
    ) as mock_format:
        mock_format.side_effect = KeyError("id")

        # Should handle the error gracefully
        result = format_results([bad_result], "article", 1, 10, 100)

        assert result["results"] == []  # Bad result is skipped


@pytest.mark.asyncio
async def test_rate_limiter_basic():
    """Test basic rate limiter functionality."""
    # Test normal operation
    limiter = RateLimiter(requests_per_second=10, burst_size=5)

    # Should allow burst through context manager
    for _ in range(5):
        async with limiter.limit():
            pass  # Should not raise


@pytest.mark.asyncio
async def test_concurrent_operations():
    """Test system behavior under concurrent load."""
    # Clear metrics
    from biomcp.metrics import (
        _metrics_collector,
        get_metric_summary,
        record_metric,
    )

    await _metrics_collector.clear()

    # Simulate concurrent metric recording
    async def record_operation(i):
        await record_metric(
            "concurrent_test",
            duration=0.1 * (i % 5),
            success=i % 10 != 0,  # 10% failure rate
        )

    # Run 100 concurrent operations
    tasks = [record_operation(i) for i in range(100)]
    await asyncio.gather(*tasks)

    # Check metrics
    summary = await get_metric_summary("concurrent_test")
    assert summary is not None
    assert summary.count == 100
    assert summary.error_rate == 0.1  # 10% errors
    assert (
        0.18 <= summary.avg_duration <= 0.22
    )  # Average of 0.1, 0.2, 0.3, 0.4


def test_cache_corruption_handling():
    """Test handling of corrupted cache data."""
    from biomcp.http_client import get_cached_response

    # Simulate corrupted cache entry
    with patch("biomcp.http_client.get_cache") as mock_get_cache:
        mock_cache = MagicMock()
        mock_cache.get.return_value = "corrupted\x00data"  # Invalid data
        mock_get_cache.return_value = mock_cache

        # Should handle corrupted data gracefully
        result = get_cached_response("test_key")
        assert (
            result == "corrupted\x00data"
        )  # Returns as-is, parsing handles it


def test_exception_hierarchy():
    """Test custom exception hierarchy and messages."""
    # Test InvalidDomainError
    exc = InvalidDomainError("bad_domain", ["article", "trial"])
    assert "bad_domain" in str(exc)
    assert "article" in str(exc)
