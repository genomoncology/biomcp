"""
FDA drug shortages integration with caching.

Note: FDA does not yet provide an OpenFDA endpoint for drug shortages.
This module fetches from the FDA Drug Shortages JSON feed and caches it locally.
"""

import json
import logging
import os
import tempfile
from datetime import datetime, timedelta
from pathlib import Path
from typing import Any

# Platform-specific file locking
try:
    import fcntl

    HAS_FCNTL = True
except ImportError:
    # Windows doesn't have fcntl
    HAS_FCNTL = False

from ..http_client import request_api
from .constants import OPENFDA_DEFAULT_LIMIT, OPENFDA_SHORTAGE_DISCLAIMER
from .drug_shortages_detail_helpers import (
    format_shortage_details_section,
    format_shortage_names,
    format_shortage_status,
    format_shortage_timeline,
)
from .drug_shortages_helpers import (
    filter_shortages,
    format_shortage_search_header,
)
from .utils import clean_text, format_count, truncate_text

logger = logging.getLogger(__name__)

# FDA Drug Shortages feed URL
FDA_SHORTAGES_URL = (
    "https://www.accessdata.fda.gov/scripts/drugshortages/default.cfm"
)
# Alternative: Direct JSON feed if available
FDA_SHORTAGES_JSON_URL = "https://www.fda.gov/media/169066/download"  # Example URL, update as needed

# Cache configuration
CACHE_DIR = Path(tempfile.gettempdir()) / "biomcp_cache"
CACHE_FILE = CACHE_DIR / "drug_shortages.json"
CACHE_TTL_HOURS = int(os.environ.get("BIOMCP_SHORTAGE_CACHE_TTL", "24"))


async def _fetch_shortage_data() -> dict[str, Any] | None:
    """
    Fetch drug shortage data from FDA.

    Returns:
        Dictionary with shortage data or None if fetch fails
    """
    try:
        # Try to fetch the JSON feed
        # Note: The actual URL may need to be updated based on FDA's current API
        response, error = await request_api(
            url=FDA_SHORTAGES_JSON_URL,
            request={},
            method="GET",
            domain="fda_drug_shortages",
        )

        if error:
            logger.error(f"API error: {error}")
            return None  # Don't return mock data in production

        if response and hasattr(response, "model_dump"):
            data = response.model_dump()
        elif isinstance(response, dict):
            data = response
        else:
            data = {}

        # Add fetch timestamp
        data["_fetched_at"] = datetime.now().isoformat()

        return data

    except Exception as e:
        logger.error(f"Failed to fetch shortage data: {e}")
        return None  # Don't return mock data in production


def _read_cache_file() -> dict[str, Any] | None:
    """Read and validate cache file if it exists and is recent."""
    if not CACHE_FILE.exists():
        return None

    try:
        with open(CACHE_FILE) as f:
            # Acquire shared lock for reading (Unix only)
            if HAS_FCNTL:
                fcntl.flock(f.fileno(), fcntl.LOCK_SH)
            try:
                data = json.load(f)
            finally:
                # Release lock (Unix only)
                if HAS_FCNTL:
                    fcntl.flock(f.fileno(), fcntl.LOCK_UN)

        # Check cache age
        fetched_at = datetime.fromisoformat(data.get("_fetched_at", ""))
        cache_age = datetime.now() - fetched_at

        if cache_age < timedelta(hours=CACHE_TTL_HOURS):
            logger.debug(f"Using cached shortage data (age: {cache_age})")
            return data

        logger.debug(f"Cache expired (age: {cache_age}), fetching new data")
        return None
    except (OSError, json.JSONDecodeError, ValueError) as e:
        logger.warning(f"Failed to read cache: {e}")
        return None


def _write_cache_file(data: dict[str, Any]) -> None:
    """Write data to cache file with atomic operation."""
    temp_file = CACHE_FILE.with_suffix(".tmp")
    try:
        with open(temp_file, "w") as f:
            # Acquire exclusive lock for writing (Unix only)
            if HAS_FCNTL:
                fcntl.flock(f.fileno(), fcntl.LOCK_EX)
            try:
                json.dump(data, f, indent=2)
            finally:
                # Release lock (Unix only)
                if HAS_FCNTL:
                    fcntl.flock(f.fileno(), fcntl.LOCK_UN)

        # Atomic rename
        temp_file.replace(CACHE_FILE)
        logger.debug(f"Saved shortage data to cache: {CACHE_FILE}")
    except (OSError, json.JSONDecodeError) as e:
        logger.warning(f"Failed to save cache: {e}")
        # Clean up temp file if it exists
        if temp_file.exists():
            temp_file.unlink()


async def _get_cached_shortage_data() -> dict[str, Any] | None:
    """
    Get shortage data from cache if valid, otherwise fetch new data.

    Returns:
        Dictionary with shortage data or None if unavailable
    """
    # Ensure cache directory exists
    CACHE_DIR.mkdir(parents=True, exist_ok=True)

    # Try to read from cache
    cached_data = _read_cache_file()
    if cached_data:
        return cached_data

    # Fetch new data
    data = await _fetch_shortage_data()

    # Save to cache if we got data
    if data:
        _write_cache_file(data)

    return data


async def search_drug_shortages(
    drug: str | None = None,
    status: str | None = None,
    therapeutic_category: str | None = None,
    limit: int = OPENFDA_DEFAULT_LIMIT,
    skip: int = 0,
    api_key: str | None = None,
) -> str:
    """
    Search FDA drug shortage records.

    Args:
        drug: Drug name (generic or brand) to search for
        status: Shortage status (current, resolved, discontinued)
        therapeutic_category: Therapeutic category to filter by
        limit: Maximum number of results to return
        skip: Number of results to skip (for pagination)
        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with drug shortage information
    """
    # Get shortage data (from cache or fresh)
    data = await _get_cached_shortage_data()

    if not data:
        return (
            "⚠️ **Drug Shortage Data Temporarily Unavailable**\n\n"
            "The FDA drug shortage database cannot be accessed at this time. "
            "This feature requires FDA to provide a machine-readable API endpoint.\n\n"
            "**Alternative Options:**\n"
            "• Visit FDA Drug Shortages Database: https://www.accessdata.fda.gov/scripts/drugshortages/\n"
            "• Check ASHP Drug Shortages: https://www.ashp.org/drug-shortages/current-shortages\n\n"
            "Note: FDA currently provides shortage data only as PDF/HTML, not as a queryable API."
        )

    shortages = data.get("shortages", [])

    # Filter results based on criteria
    filtered = filter_shortages(shortages, drug, status, therapeutic_category)

    # Apply pagination
    total = len(filtered)
    filtered = filtered[skip : skip + limit]

    if not filtered:
        return "No drug shortages found matching your criteria."

    # Format the results
    output = ["## FDA Drug Shortage Information\n"]

    # Add header information
    last_updated = data.get("last_updated") or data.get("_fetched_at")
    output.extend(
        format_shortage_search_header(
            drug, status, therapeutic_category, last_updated
        )
    )

    output.append(
        f"**Total Shortages Found**: {format_count(total, 'shortage')}\n"
    )

    # Summary by status
    if len(filtered) > 1:
        output.extend(_format_shortage_summary(filtered))

    # Show results
    output.append(f"### Shortages (showing {len(filtered)} of {total}):\n")

    for i, shortage in enumerate(filtered, 1):
        output.extend(_format_shortage_entry(shortage, i))

    output.append(f"\n---\n{OPENFDA_SHORTAGE_DISCLAIMER}")

    return "\n".join(output)


async def get_drug_shortage(
    drug: str,
    api_key: str | None = None,
) -> str:
    """
    Get detailed shortage information for a specific drug.

    Args:
        drug: Generic or brand name of the drug
        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with detailed shortage information
    """
    # Get shortage data
    data = await _get_cached_shortage_data()

    if not data:
        return (
            "⚠️ **Drug Shortage Data Temporarily Unavailable**\n\n"
            "The FDA drug shortage database cannot be accessed at this time. "
            "This feature requires FDA to provide a machine-readable API endpoint.\n\n"
            "**Alternative Options:**\n"
            "• Visit FDA Drug Shortages Database: https://www.accessdata.fda.gov/scripts/drugshortages/\n"
            "• Check ASHP Drug Shortages: https://www.ashp.org/drug-shortages/current-shortages\n\n"
            "Note: FDA currently provides shortage data only as PDF/HTML, not as a queryable API."
        )

    shortages = data.get("shortages", [])

    # Find the specific drug
    drug_lower = drug.lower()
    matched = None

    for shortage in shortages:
        generic = shortage.get("generic_name", "").lower()
        brands = [b.lower() for b in shortage.get("brand_names", [])]

        if drug_lower in generic or any(drug_lower in b for b in brands):
            matched = shortage
            break

    if not matched:
        return f"No shortage information found for {drug}"

    # Format detailed information
    output = [
        f"## Drug Shortage Details: {matched.get('generic_name', drug)}\n"
    ]

    # Last updated
    last_updated = data.get("last_updated") or data.get("_fetched_at")
    if last_updated:
        try:
            updated_dt = datetime.fromisoformat(last_updated)
            output.append(
                f"*Data Updated: {updated_dt.strftime('%Y-%m-%d %H:%M')}*\n"
            )
        except (ValueError, TypeError):
            pass

    output.extend(_format_shortage_detail(matched))

    output.append(f"\n---\n{OPENFDA_SHORTAGE_DISCLAIMER}")

    return "\n".join(output)


def _format_shortage_summary(shortages: list[dict[str, Any]]) -> list[str]:
    """Format summary of shortage statuses."""
    output = []

    # Count by status
    current_count = sum(
        1 for s in shortages if "current" in s.get("status", "").lower()
    )
    resolved_count = sum(
        1 for s in shortages if "resolved" in s.get("status", "").lower()
    )

    if current_count or resolved_count:
        output.append("### Status Summary:")
        if current_count:
            output.append(f"- **Current Shortages**: {current_count}")
        if resolved_count:
            output.append(f"- **Resolved**: {resolved_count}")
        output.append("")

    return output


def _format_shortage_entry(shortage: dict[str, Any], num: int) -> list[str]:
    """Format a single shortage entry."""
    output = []

    generic = shortage.get("generic_name", "Unknown Drug")
    status = shortage.get("status", "Unknown")

    # Status indicator
    status_emoji = "🔴" if "current" in status.lower() else "🟢"

    output.append(f"#### {num}. {generic}")
    output.append(f"{status_emoji} **Status**: {status}")

    # Brand names
    brands = shortage.get("brand_names")
    if brands and brands[0]:  # Check for non-empty brands
        output.append(f"**Brand Names**: {', '.join(brands)}")

    # Dates
    if start_date := shortage.get("shortage_start_date"):
        output.append(f"**Shortage Started**: {start_date}")

    if resolution_date := shortage.get("resolution_date"):
        output.append(f"**Resolved**: {resolution_date}")
    elif estimated := shortage.get("estimated_resolution"):
        output.append(f"**Estimated Resolution**: {estimated}")

    # Reason
    if reason := shortage.get("reason"):
        output.append(f"**Reason**: {reason}")

    # Therapeutic category
    if category := shortage.get("therapeutic_category"):
        output.append(f"**Therapeutic Category**: {category}")

    # Notes
    if notes := shortage.get("notes"):
        cleaned_notes = truncate_text(clean_text(notes), 200)
        output.append(f"\n**Notes**: {cleaned_notes}")

    output.append("")
    return output


def _format_shortage_detail(shortage: dict[str, Any]) -> list[str]:
    """Format detailed shortage information."""
    output = ["### Shortage Information"]

    # Status
    output.extend(format_shortage_status(shortage))

    # Names
    output.extend(format_shortage_names(shortage))

    # Manufacturers
    if manufacturers := shortage.get("manufacturers"):
        output.append(f"**Manufacturers**: {', '.join(manufacturers)}")

    # Therapeutic category
    if category := shortage.get("therapeutic_category"):
        output.append(f"**Therapeutic Category**: {category}")

    # Timeline
    output.append("")
    output.extend(format_shortage_timeline(shortage))

    # Details
    output.append("")
    output.extend(format_shortage_details_section(shortage))

    # Alternatives if available
    if alternatives := shortage.get("alternatives"):
        output.append("\n### Alternative Products")
        if isinstance(alternatives, list):
            output.append(", ".join(alternatives))
        else:
            output.append(str(alternatives))

    return output
