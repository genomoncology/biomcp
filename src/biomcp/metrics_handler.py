"""MCP handler for metrics collection."""

from typing import Annotated

from biomcp.core import mcp_app
from biomcp.metrics import get_all_metrics, get_metric_summary


@mcp_app.tool()
async def get_performance_metrics(
    metric_name: Annotated[
        str | None,
        "Specific metric name to retrieve, or None for all metrics",
    ] = None,
) -> str:
    """Get performance metrics for BioMCP operations.

    Returns performance statistics including:
    - Request counts and success rates
    - Response time percentiles (p50, p95, p99)
    - Error rates and types
    - Domain-specific performance breakdown

    Parameters:
        metric_name: Optional specific metric to retrieve

    Returns:
        Formatted metrics report
    """
    if metric_name:
        summary = await get_metric_summary(metric_name)
        if not summary:
            return f"No metrics found for '{metric_name}'"

        return _format_summary(summary)
    else:
        all_summaries = await get_all_metrics()
        if not all_summaries:
            return "No metrics collected yet"

        lines = ["# BioMCP Performance Metrics\n"]
        for name in sorted(all_summaries.keys()):
            summary = all_summaries[name]
            lines.append(f"## {name}")
            lines.append(_format_summary(summary))
            lines.append("")

        return "\n".join(lines)


def _format_summary(summary) -> str:
    """Format a metric summary for display."""
    lines = [
        f"- Total requests: {summary.count}",
        f"- Success rate: {(1 - summary.error_rate) * 100:.1f}%",
        f"- Errors: {summary.error_count}",
        "",
        "### Response Times",
        f"- Average: {summary.avg_duration * 1000:.1f}ms",
        f"- Min: {summary.min_duration * 1000:.1f}ms",
        f"- Max: {summary.max_duration * 1000:.1f}ms",
        f"- P50: {summary.p50_duration * 1000:.1f}ms",
        f"- P95: {summary.p95_duration * 1000:.1f}ms",
        f"- P99: {summary.p99_duration * 1000:.1f}ms",
    ]

    return "\n".join(lines)
