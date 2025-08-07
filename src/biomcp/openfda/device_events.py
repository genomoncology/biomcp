"""
OpenFDA Device Adverse Events (MAUDE) integration.

Focus on genomic/diagnostic devices relevant to precision oncology.
"""

import logging

from .constants import (
    GENOMIC_DEVICE_PRODUCT_CODES,
    OPENFDA_DEFAULT_LIMIT,
    OPENFDA_DEVICE_EVENTS_URL,
    OPENFDA_DISCLAIMER,
    OPENFDA_MAX_LIMIT,
)
from .device_events_helpers import (
    analyze_device_problems,
    format_detailed_device_info,
    format_device_detail_header,
    format_device_distribution,
    format_device_report_summary,
    format_patient_details,
    format_top_problems,
)
from .utils import clean_text, format_count, make_openfda_request

logger = logging.getLogger(__name__)


def _build_device_search_query(
    device: str | None,
    manufacturer: str | None,
    problem: str | None,
    product_code: str | None,
    genomics_only: bool,
) -> str:
    """Build the search query for device events."""
    search_parts = []

    if device:
        # Build flexible search queries
        device_queries = []

        # First try exact match
        device_queries.extend([
            f'device.brand_name:"{device}"',
            f'device.generic_name:"{device}"',
            f'device.openfda.device_name:"{device}"',
        ])

        # For multi-word terms, also search for key words with wildcards
        # This helps match "FoundationOne CDx" to "F1CDX" or similar variations
        words = device.split()

        # If it's a multi-word query, add wildcard searches for significant words
        for word in words:
            # Skip common words and very short ones
            if len(word) > 3 and word.lower() not in [
                "test",
                "system",
                "device",
            ]:
                # Use prefix wildcard for better performance
                device_queries.append(f"device.brand_name:{word}*")
                device_queries.append(f"device.generic_name:{word}*")

        # Also try searching by removing spaces (e.g., "Foundation One" -> "FoundationOne")
        if len(words) > 1:
            combined = "".join(words)
            device_queries.append(f'device.brand_name:"{combined}"')
            device_queries.append(f'device.generic_name:"{combined}"')

        search_parts.append(f"({' OR '.join(device_queries)})")

    if manufacturer:
        # Search manufacturer field with both exact and wildcard matching
        mfr_queries = [
            f'device.manufacturer_d_name:"{manufacturer}"',
            f"device.manufacturer_d_name:*{manufacturer}*",
        ]
        search_parts.append(f"({' OR '.join(mfr_queries)})")

    if problem:
        search_parts.append(f'device.device_problem_text:"{problem}"')

    if product_code:
        search_parts.append(f'device.openfda.product_code:"{product_code}"')
    elif (
        genomics_only and not device
    ):  # Only apply genomics filter if no specific device is named
        # Filter to genomic device product codes
        code_parts = [
            f'device.openfda.product_code:"{code}"'
            for code in GENOMIC_DEVICE_PRODUCT_CODES
        ]
        if code_parts:
            search_parts.append(f"({' OR '.join(code_parts)})")

    return " AND ".join(search_parts)


def _format_search_summary(
    device: str | None,
    manufacturer: str | None,
    problem: str | None,
    genomics_only: bool,
    total: int,
) -> list[str]:
    """Format the search summary section."""
    output = []

    search_desc = []
    if device:
        search_desc.append(f"**Device**: {device}")
    if manufacturer:
        search_desc.append(f"**Manufacturer**: {manufacturer}")
    if problem:
        search_desc.append(f"**Problem**: {problem}")
    if genomics_only:
        search_desc.append("**Type**: Genomic/Diagnostic Devices")

    if search_desc:
        output.append(" | ".join(search_desc))
    output.append(
        f"**Total Reports Found**: {format_count(total, 'report')}\n"
    )

    return output


async def search_device_events(
    device: str | None = None,
    manufacturer: str | None = None,
    problem: str | None = None,
    product_code: str | None = None,
    genomics_only: bool = True,
    limit: int = OPENFDA_DEFAULT_LIMIT,
    skip: int = 0,
    api_key: str | None = None,
) -> str:
    """
    Search FDA device adverse event reports (MAUDE).

    Args:
        device: Device name to search for
        manufacturer: Manufacturer name
        problem: Device problem description
        product_code: FDA product code
        genomics_only: Filter to genomic/diagnostic devices only
        limit: Maximum number of results
        skip: Number of results to skip
        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with device event information
    """
    if not device and not manufacturer and not product_code and not problem:
        return (
            "⚠️ Please specify a device name, manufacturer, or problem to search.\n\n"
            "Examples:\n"
            "- Search by device: --device 'FoundationOne'\n"
            "- Search by manufacturer: --manufacturer 'Illumina'\n"
            "- Search by problem: --problem 'false positive'"
        )

    # Build and execute search
    search_query = _build_device_search_query(
        device, manufacturer, problem, product_code, genomics_only
    )
    params = {
        "search": search_query,
        "limit": min(limit, OPENFDA_MAX_LIMIT),
        "skip": skip,
    }

    response, error = await make_openfda_request(
        OPENFDA_DEVICE_EVENTS_URL, params, "openfda_device_events", api_key
    )

    if error:
        return f"⚠️ Error searching device events: {error}"

    if not response or not response.get("results"):
        return _format_no_results(device, manufacturer, problem, genomics_only)

    results = response["results"]
    total = (
        response.get("meta", {}).get("results", {}).get("total", len(results))
    )

    # Build output
    output = ["## FDA Device Adverse Event Reports\n"]
    output.extend(
        _format_search_summary(
            device, manufacturer, problem, genomics_only, total
        )
    )

    # Analyze and format problems
    all_problems, all_device_names, _ = analyze_device_problems(results)
    output.extend(format_top_problems(all_problems, results))

    # Show device distribution if searching by problem
    if problem:
        output.extend(format_device_distribution(all_device_names, results))

    # Display sample reports
    output.append(
        f"### Sample Reports (showing {min(len(results), 3)} of {total}):\n"
    )
    for i, result in enumerate(results[:3], 1):
        output.extend(format_device_report_summary(result, i))

    # Add tips
    if genomics_only:
        output.append(
            "\n💡 **Note**: Results filtered to genomic/diagnostic devices. "
            "Use --no-genomics-only to search all medical devices."
        )

    output.append(f"\n{OPENFDA_DISCLAIMER}")
    return "\n".join(output)


def _format_no_results(
    device: str | None,
    manufacturer: str | None,
    problem: str | None,
    genomics_only: bool,
) -> str:
    """Format no results message."""
    search_desc = []
    if device:
        search_desc.append(f"device '{device}'")
    if manufacturer:
        search_desc.append(f"manufacturer '{manufacturer}'")
    if problem:
        search_desc.append(f"problem '{problem}'")

    desc = " and ".join(search_desc)
    if genomics_only:
        desc += " (filtered to genomic/diagnostic devices)"

    return f"No device adverse event reports found for {desc}."


async def get_device_event(
    mdr_report_key: str, api_key: str | None = None
) -> str:
    """
    Get detailed information for a specific device event report.

    Args:
        mdr_report_key: MDR report key
        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with detailed report information
    """
    params = {
        "search": f'mdr_report_key:"{mdr_report_key}"',
        "limit": 1,
    }

    response, error = await make_openfda_request(
        OPENFDA_DEVICE_EVENTS_URL,
        params,
        "openfda_device_event_detail",
        api_key,
    )

    if error:
        return f"⚠️ Error retrieving device event report: {error}"

    if not response or not response.get("results"):
        return f"Device event report '{mdr_report_key}' not found."

    result = response["results"][0]

    # Build detailed output
    output = format_device_detail_header(result, mdr_report_key)

    # Device details
    if devices := result.get("device", []):
        output.extend(format_detailed_device_info(devices))

    # Event narrative
    if event_desc := result.get("event_description"):
        output.append("### Event Description")
        output.append(clean_text(event_desc))
        output.append("")

    # Manufacturer narrative
    if mfr_narrative := result.get("manufacturer_narrative"):
        output.append("### Manufacturer's Analysis")
        output.append(clean_text(mfr_narrative))
        output.append("")

    # Patient information
    if patient := result.get("patient", []):
        output.extend(format_patient_details(patient))

    # Remedial action
    if remedial := result.get("remedial_action"):
        output.append("### Remedial Action")
        if isinstance(remedial, list):
            output.append(", ".join(remedial))
        else:
            output.append(remedial)
        output.append("")

    output.append(f"\n{OPENFDA_DISCLAIMER}")
    return "\n".join(output)
