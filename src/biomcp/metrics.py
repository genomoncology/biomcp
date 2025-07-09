"""Performance monitoring and metrics collection for BioMCP."""

import asyncio
import functools
import logging
import os
import time
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime

from .constants import (
    MAX_METRIC_SAMPLES,
    METRIC_PERCENTILE_50,
    METRIC_PERCENTILE_95,
    METRIC_PERCENTILE_99,
)

logger = logging.getLogger(__name__)

# Check if metrics are enabled via environment variable
METRICS_ENABLED = (
    os.getenv("BIOMCP_METRICS_ENABLED", "false").lower() == "true"
)


@dataclass
class MetricSample:
    """Single metric measurement."""

    timestamp: datetime
    duration: float
    success: bool
    error: str | None = None
    tags: dict[str, str] = field(default_factory=dict)


@dataclass
class MetricSummary:
    """Summary statistics for a metric."""

    name: str
    count: int
    success_count: int
    error_count: int
    total_duration: float
    min_duration: float
    max_duration: float
    avg_duration: float
    p50_duration: float
    p95_duration: float
    p99_duration: float
    error_rate: float

    @classmethod
    def from_samples(
        cls, name: str, samples: list[MetricSample]
    ) -> "MetricSummary":
        """Calculate summary statistics from samples."""
        if not samples:
            return cls(
                name=name,
                count=0,
                success_count=0,
                error_count=0,
                total_duration=0.0,
                min_duration=0.0,
                max_duration=0.0,
                avg_duration=0.0,
                p50_duration=0.0,
                p95_duration=0.0,
                p99_duration=0.0,
                error_rate=0.0,
            )

        durations = sorted([s.duration for s in samples])
        success_count = sum(1 for s in samples if s.success)
        error_count = len(samples) - success_count

        def percentile(data: list[float], p: float) -> float:
            """Calculate percentile."""
            if not data:
                return 0.0
            k = (len(data) - 1) * p
            f = int(k)
            c = k - f
            if f >= len(data) - 1:
                return data[-1]
            return data[f] + c * (data[f + 1] - data[f])

        return cls(
            name=name,
            count=len(samples),
            success_count=success_count,
            error_count=error_count,
            total_duration=sum(durations),
            min_duration=min(durations),
            max_duration=max(durations),
            avg_duration=sum(durations) / len(durations),
            p50_duration=percentile(durations, METRIC_PERCENTILE_50),
            p95_duration=percentile(durations, METRIC_PERCENTILE_95),
            p99_duration=percentile(durations, METRIC_PERCENTILE_99),
            error_rate=error_count / len(samples) if samples else 0.0,
        )


class MetricsCollector:
    """Collects and manages performance metrics."""

    def __init__(self, max_samples_per_metric: int = MAX_METRIC_SAMPLES):
        """Initialize metrics collector.

        Args:
            max_samples_per_metric: Maximum samples to keep per metric
        """
        self._metrics: dict[str, list[MetricSample]] = defaultdict(list)
        self._max_samples = max_samples_per_metric
        self._lock = asyncio.Lock()

    async def record(
        self,
        name: str,
        duration: float,
        success: bool = True,
        error: str | None = None,
        tags: dict[str, str] | None = None,
    ) -> None:
        """Record a metric sample.

        Args:
            name: Metric name
            duration: Duration in seconds
            success: Whether operation succeeded
            error: Error message if failed
            tags: Additional metadata tags
        """
        sample = MetricSample(
            timestamp=datetime.now(),
            duration=duration,
            success=success,
            error=error,
            tags=tags or {},
        )

        async with self._lock:
            samples = self._metrics[name]
            samples.append(sample)

            # Keep only the most recent samples
            if len(samples) > self._max_samples:
                self._metrics[name] = samples[-self._max_samples :]

    async def get_summary(self, name: str) -> MetricSummary | None:
        """Get summary statistics for a metric.

        Args:
            name: Metric name

        Returns:
            Summary statistics or None if metric not found
        """
        async with self._lock:
            samples = self._metrics.get(name, [])
            if not samples:
                return None
            return MetricSummary.from_samples(name, samples)

    async def get_all_summaries(self) -> dict[str, MetricSummary]:
        """Get summaries for all metrics.

        Returns:
            Dictionary of metric name to summary
        """
        async with self._lock:
            return {
                name: MetricSummary.from_samples(name, samples)
                for name, samples in self._metrics.items()
            }

    async def clear(self, name: str | None = None) -> None:
        """Clear metrics.

        Args:
            name: Specific metric to clear, or None to clear all
        """
        async with self._lock:
            if name:
                self._metrics.pop(name, None)
            else:
                self._metrics.clear()


# Global metrics collector instance
_metrics_collector = MetricsCollector()


async def record_metric(
    name: str,
    duration: float,
    success: bool = True,
    error: str | None = None,
    tags: dict[str, str] | None = None,
) -> None:
    """Record a metric to the global collector.

    Note: This is a no-op if BIOMCP_METRICS_ENABLED is not set to true.

    Args:
        name: Metric name
        duration: Duration in seconds
        success: Whether operation succeeded
        error: Error message if failed
        tags: Additional metadata tags
    """
    if METRICS_ENABLED:
        await _metrics_collector.record(name, duration, success, error, tags)


async def get_metric_summary(name: str) -> MetricSummary | None:
    """Get summary statistics for a metric.

    Args:
        name: Metric name

    Returns:
        Summary statistics or None if metric not found
    """
    return await _metrics_collector.get_summary(name)


async def get_all_metrics() -> dict[str, MetricSummary]:
    """Get summaries for all metrics.

    Returns:
        Dictionary of metric name to summary
    """
    return await _metrics_collector.get_all_summaries()


def track_performance(metric_name: str | None = None):
    """Decorator to track function performance.

    Args:
        metric_name: Custom metric name (defaults to function name)

    Returns:
        Decorated function
    """

    def decorator(func):
        name = metric_name or f"{func.__module__}.{func.__name__}"

        @functools.wraps(func)
        async def async_wrapper(*args, **kwargs):
            start_time = time.perf_counter()
            success = True
            error_msg = None

            try:
                result = await func(*args, **kwargs)
                return result
            except Exception as exc:
                success = False
                error_msg = str(exc)
                raise
            finally:
                duration = time.perf_counter() - start_time
                await record_metric(
                    name=name,
                    duration=duration,
                    success=success,
                    error=error_msg,
                )

        @functools.wraps(func)
        def sync_wrapper(*args, **kwargs):
            start_time = time.perf_counter()
            success = True
            error_msg = None

            try:
                result = func(*args, **kwargs)
                return result
            except Exception as exc:
                success = False
                error_msg = str(exc)
                raise
            finally:
                duration = time.perf_counter() - start_time
                # Schedule metric recording in the event loop
                try:
                    loop = asyncio.get_running_loop()
                    # Fire and forget the metric recording
                    task = loop.create_task(
                        record_metric(
                            name=name,
                            duration=duration,
                            success=success,
                            error=error_msg,
                        )
                    )
                    # Add error handler to prevent unhandled exceptions
                    task.add_done_callback(
                        lambda t: t.exception() if t.done() else None
                    )
                except RuntimeError:
                    # No event loop running, log instead
                    logger.debug(
                        f"Metric {name}: duration={duration:.3f}s, "
                        f"success={success}, error={error_msg}"
                    )

        # Return appropriate wrapper based on function type
        if asyncio.iscoroutinefunction(func):
            return async_wrapper
        else:
            return sync_wrapper

    return decorator


# Context manager for timing operations
class Timer:
    """Context manager for timing operations."""

    def __init__(self, metric_name: str, tags: dict[str, str] | None = None):
        """Initialize timer.

        Args:
            metric_name: Name for the metric
            tags: Additional metadata tags
        """
        self.metric_name = metric_name
        self.tags = tags or {}
        self.start_time: float | None = None

    def __enter__(self):
        """Start timing."""
        self.start_time = time.perf_counter()
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Stop timing and record metric."""
        if self.start_time is None or not METRICS_ENABLED:
            return False

        duration = time.perf_counter() - self.start_time
        success = exc_type is None
        error_msg = str(exc_val) if exc_val else None

        # Schedule metric recording
        try:
            loop = asyncio.get_running_loop()
            # Fire and forget the metric recording
            task = loop.create_task(
                record_metric(
                    name=self.metric_name,
                    duration=duration,
                    success=success,
                    error=error_msg,
                    tags=self.tags,
                )
            )
            # Add error handler to prevent unhandled exceptions
            task.add_done_callback(
                lambda t: t.exception() if t.done() else None
            )
        except RuntimeError:
            # No event loop running, log instead
            logger.debug(
                f"Metric {self.metric_name}: duration={duration:.3f}s, "
                f"success={success}, error={error_msg}, tags={self.tags}"
            )

        # Don't suppress exceptions
        return False

    async def __aenter__(self):
        """Async enter."""
        self.start_time = time.perf_counter()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        """Async exit."""
        if self.start_time is None or not METRICS_ENABLED:
            return False

        duration = time.perf_counter() - self.start_time
        success = exc_type is None
        error_msg = str(exc_val) if exc_val else None

        await record_metric(
            name=self.metric_name,
            duration=duration,
            success=success,
            error=error_msg,
            tags=self.tags,
        )

        # Don't suppress exceptions
        return False
