"""Tests for performance metrics collection."""

import asyncio
import time
from datetime import datetime
from unittest.mock import patch

import pytest

from biomcp.metrics import (
    MetricSample,
    MetricsCollector,
    MetricSummary,
    Timer,
    get_all_metrics,
    get_metric_summary,
    record_metric,
    track_performance,
)


@pytest.fixture(autouse=True)
def enable_metrics(monkeypatch):
    """Enable metrics for all tests in this module."""
    monkeypatch.setenv("BIOMCP_METRICS_ENABLED", "true")
    # Force reload of the module to pick up the new env var
    import importlib

    import biomcp.metrics

    importlib.reload(biomcp.metrics)


def test_metric_sample():
    """Test MetricSample dataclass."""
    sample = MetricSample(
        timestamp=datetime.now(),
        duration=1.5,
        success=True,
        error=None,
        tags={"domain": "article"},
    )

    assert sample.duration == 1.5
    assert sample.success is True
    assert sample.error is None
    assert sample.tags["domain"] == "article"


def test_metric_summary_from_samples():
    """Test MetricSummary calculation from samples."""
    now = datetime.now()
    samples = [
        MetricSample(timestamp=now, duration=0.1, success=True),
        MetricSample(timestamp=now, duration=0.2, success=True),
        MetricSample(
            timestamp=now, duration=0.3, success=False, error="timeout"
        ),
        MetricSample(timestamp=now, duration=0.4, success=True),
        MetricSample(timestamp=now, duration=0.5, success=True),
    ]

    summary = MetricSummary.from_samples("test_metric", samples)

    assert summary.name == "test_metric"
    assert summary.count == 5
    assert summary.success_count == 4
    assert summary.error_count == 1
    assert summary.total_duration == 1.5
    assert summary.min_duration == 0.1
    assert summary.max_duration == 0.5
    assert summary.avg_duration == 0.3
    assert summary.error_rate == 0.2  # 1/5

    # Check percentiles
    assert summary.p50_duration == 0.3  # median
    assert 0.4 <= summary.p95_duration <= 0.5
    assert 0.4 <= summary.p99_duration <= 0.5


def test_metric_summary_empty():
    """Test MetricSummary with no samples."""
    summary = MetricSummary.from_samples("empty", [])

    assert summary.count == 0
    assert summary.success_count == 0
    assert summary.error_count == 0
    assert summary.total_duration == 0.0
    assert summary.error_rate == 0.0


@pytest.mark.asyncio
async def test_metrics_collector():
    """Test MetricsCollector functionality."""
    collector = MetricsCollector(max_samples_per_metric=3)

    # Record some metrics
    await collector.record("api_call", 0.1, success=True)
    await collector.record("api_call", 0.2, success=True)
    await collector.record("api_call", 0.3, success=False, error="timeout")

    # Get summary
    summary = await collector.get_summary("api_call")
    assert summary is not None
    assert summary.count == 3
    assert summary.success_count == 2
    assert summary.error_count == 1

    # Test max samples limit
    await collector.record("api_call", 0.4, success=True)
    await collector.record("api_call", 0.5, success=True)

    summary = await collector.get_summary("api_call")
    assert summary.count == 3  # Still 3 due to limit
    assert summary.min_duration == 0.3  # Oldest samples dropped

    # Test clear
    await collector.clear("api_call")
    summary = await collector.get_summary("api_call")
    assert summary is None


@pytest.mark.asyncio
async def test_global_metrics_functions():
    """Test global metrics functions."""
    # Clear any existing metrics
    from biomcp.metrics import _metrics_collector

    await _metrics_collector.clear()

    # Record metrics
    await record_metric("test_op", 0.5, success=True)
    await record_metric("test_op", 0.7, success=False, error="failed")

    # Get summary
    summary = await get_metric_summary("test_op")
    assert summary is not None
    assert summary.count == 2
    assert summary.success_count == 1

    # Get all metrics
    all_metrics = await get_all_metrics()
    assert "test_op" in all_metrics


@pytest.mark.asyncio
async def test_track_performance_decorator_async():
    """Test track_performance decorator on async functions."""
    from biomcp.metrics import _metrics_collector

    await _metrics_collector.clear()

    @track_performance("test_async_func")
    async def slow_operation():
        await asyncio.sleep(0.1)
        return "done"

    result = await slow_operation()
    assert result == "done"

    # Check metric was recorded
    summary = await get_metric_summary("test_async_func")
    assert summary is not None
    assert summary.count == 1
    assert summary.success_count == 1
    assert summary.min_duration >= 0.1


@pytest.mark.asyncio
async def test_track_performance_decorator_async_error():
    """Test track_performance decorator on async functions with errors."""
    from biomcp.metrics import _metrics_collector

    await _metrics_collector.clear()

    @track_performance("test_async_error")
    async def failing_operation():
        await asyncio.sleep(0.05)
        raise ValueError("Test error")

    with pytest.raises(ValueError, match="Test error"):
        await failing_operation()

    # Check metric was recorded with error
    summary = await get_metric_summary("test_async_error")
    assert summary is not None
    assert summary.count == 1
    assert summary.success_count == 0
    assert summary.error_count == 1


def test_track_performance_decorator_sync():
    """Test track_performance decorator on sync functions."""

    @track_performance("test_sync_func")
    def fast_operation():
        time.sleep(0.05)
        return "done"

    # Need to run in an event loop context
    async def run_test():
        from biomcp.metrics import _metrics_collector

        await _metrics_collector.clear()

        result = fast_operation()
        assert result == "done"

        # Give time for the metric to be recorded
        await asyncio.sleep(0.1)

        summary = await get_metric_summary("test_sync_func")
        assert summary is not None
        assert summary.count == 1
        assert summary.success_count == 1

    asyncio.run(run_test())


@pytest.mark.asyncio
async def test_timer_context_manager():
    """Test Timer context manager."""
    from biomcp.metrics import _metrics_collector

    await _metrics_collector.clear()

    # Test async timer
    async with Timer("test_timer", tags={"operation": "test"}):
        await asyncio.sleep(0.1)

    summary = await get_metric_summary("test_timer")
    assert summary is not None
    assert summary.count == 1
    assert summary.success_count == 1
    assert summary.min_duration >= 0.1

    # Test sync timer (in async context)
    with Timer("test_sync_timer"):
        time.sleep(0.05)

    # Give time for metric to be recorded
    await asyncio.sleep(0.1)

    summary = await get_metric_summary("test_sync_timer")
    assert summary is not None
    assert summary.count == 1


@pytest.mark.asyncio
async def test_timer_with_exception():
    """Test Timer context manager with exceptions."""
    from biomcp.metrics import _metrics_collector

    await _metrics_collector.clear()

    # Test async timer with exception
    with pytest.raises(ValueError):
        async with Timer("test_timer_error"):
            await asyncio.sleep(0.05)
            raise ValueError("Test error")

    summary = await get_metric_summary("test_timer_error")
    assert summary is not None
    assert summary.count == 1
    assert summary.success_count == 0
    assert summary.error_count == 1


def test_timer_without_event_loop():
    """Test Timer when no event loop is running."""
    # This simulates using Timer in a non-async context
    with patch("biomcp.metrics.logger") as mock_logger:
        with Timer("test_no_loop"):
            time.sleep(0.01)

        # Should log instead of recording metric
        mock_logger.debug.assert_called_once()
        call_args = mock_logger.debug.call_args[0][0]
        assert "test_no_loop" in call_args
        assert "duration=" in call_args
