"""
Helper functions for drug shortage search to reduce complexity.
"""

from datetime import datetime
from typing import Any


def matches_drug_filter(shortage: dict[str, Any], drug: str | None) -> bool:
    """Check if shortage matches drug name filter."""
    if not drug:
        return True

    drug_lower = drug.lower()
    generic = shortage.get("generic_name", "").lower()
    brands = [b.lower() for b in shortage.get("brand_names", [])]

    return drug_lower in generic or any(drug_lower in b for b in brands)


def matches_status_filter(
    shortage: dict[str, Any], status: str | None
) -> bool:
    """Check if shortage matches status filter."""
    if not status:
        return True

    status_lower = status.lower()
    shortage_status = shortage.get("status", "").lower()

    if status_lower == "current":
        return "current" in shortage_status
    elif status_lower == "resolved":
        return "resolved" in shortage_status

    return False


def matches_category_filter(
    shortage: dict[str, Any], therapeutic_category: str | None
) -> bool:
    """Check if shortage matches therapeutic category filter."""
    if not therapeutic_category:
        return True

    cat_lower = therapeutic_category.lower()
    shortage_cat = shortage.get("therapeutic_category", "").lower()

    return cat_lower in shortage_cat


def filter_shortages(
    shortages: list[dict[str, Any]],
    drug: str | None,
    status: str | None,
    therapeutic_category: str | None,
) -> list[dict[str, Any]]:
    """Filter shortage list based on criteria."""
    filtered = []

    for shortage in shortages:
        if not matches_drug_filter(shortage, drug):
            continue
        if not matches_status_filter(shortage, status):
            continue
        if not matches_category_filter(shortage, therapeutic_category):
            continue

        filtered.append(shortage)

    return filtered


def format_shortage_search_header(
    drug: str | None,
    status: str | None,
    therapeutic_category: str | None,
    last_updated: str | None,
) -> list[str]:
    """Format header for shortage search results."""
    output = []

    # Add last updated time
    if last_updated:
        try:
            updated_dt = datetime.fromisoformat(last_updated)
            output.append(
                f"*Last Updated: {updated_dt.strftime('%Y-%m-%d %H:%M')}*\n"
            )
        except (ValueError, TypeError):
            pass

    if drug:
        output.append(f"**Drug**: {drug}")
    if status:
        output.append(f"**Status Filter**: {status}")
    if therapeutic_category:
        output.append(f"**Category**: {therapeutic_category}")

    return output


def format_cache_timestamp(data: dict[str, Any]) -> str | None:
    """Format cache timestamp from data."""
    last_updated = data.get("last_updated") or data.get("_fetched_at")
    if not last_updated:
        return None

    try:
        updated_dt = datetime.fromisoformat(last_updated)
        return f"*Data Updated: {updated_dt.strftime('%Y-%m-%d %H:%M')}*\n"
    except (ValueError, TypeError):
        return None
