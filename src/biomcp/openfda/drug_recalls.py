"""
OpenFDA drug recalls (Enforcement) integration.
"""

import logging
from typing import Any

from .constants import (
    OPENFDA_DEFAULT_LIMIT,
    OPENFDA_DISCLAIMER,
    OPENFDA_DRUG_ENFORCEMENT_URL,
)
from .drug_recalls_helpers import (
    build_recall_search_params,
)
from .utils import (
    clean_text,
    format_count,
    make_openfda_request,
    truncate_text,
)

logger = logging.getLogger(__name__)


async def search_drug_recalls(
    drug: str | None = None,
    recall_class: str | None = None,
    status: str | None = None,
    reason: str | None = None,
    since_date: str | None = None,
    limit: int = OPENFDA_DEFAULT_LIMIT,
    skip: int = 0,
    api_key: str | None = None,
) -> str:
    """
    Search FDA drug recall records from Enforcement database.

    Args:
        drug: Drug name (brand or generic) to search for
        recall_class: Classification (1, 2, or 3)
        status: Recall status (ongoing, completed, terminated)
        reason: Search text in recall reason
        since_date: Only show recalls after this date (YYYYMMDD format)
        limit: Maximum number of results to return
        skip: Number of results to skip (for pagination)

        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with drug recall information
    """
    # Build search parameters
    search_params = build_recall_search_params(
        drug, recall_class, status, reason, since_date, limit, skip
    )

    # Make the request
    response, error = await make_openfda_request(
        OPENFDA_DRUG_ENFORCEMENT_URL, search_params, "openfda_recalls", api_key
    )

    if error:
        return f"⚠️ Error searching drug recalls: {error}"

    if not response or not response.get("results"):
        return "No drug recall records found matching your criteria."

    # Format the results
    results = response["results"]
    total = (
        response.get("meta", {}).get("results", {}).get("total", len(results))
    )

    output = ["## FDA Drug Recall Records\n"]

    if drug:
        output.append(f"**Drug**: {drug}")
    if recall_class:
        output.append(f"**Classification**: Class {recall_class}")
    if status:
        output.append(f"**Status**: {status}")
    if since_date:
        output.append(f"**Since**: {since_date}")

    output.append(
        f"**Total Recalls Found**: {format_count(total, 'recall')}\n"
    )

    # Summary of recall classes if multiple results
    if len(results) > 1:
        output.extend(_format_recall_class_summary(results))

    # Show results
    output.append(f"### Recalls (showing {len(results)} of {total}):\n")

    for i, recall in enumerate(results, 1):
        output.extend(_format_recall_summary(recall, i))

    output.append(f"\n{OPENFDA_DISCLAIMER}")

    return "\n".join(output)


async def get_drug_recall(
    recall_number: str,
    api_key: str | None = None,
) -> str:
    """
    Get detailed drug recall information for a specific recall.

    Args:
        recall_number: FDA recall number

        api_key: Optional OpenFDA API key (overrides OPENFDA_API_KEY env var)

    Returns:
        Formatted string with detailed recall information
    """
    # Search for the specific recall
    search_params = {"search": f'recall_number:"{recall_number}"', "limit": 1}

    response, error = await make_openfda_request(
        OPENFDA_DRUG_ENFORCEMENT_URL, search_params, "openfda_recalls", api_key
    )

    if error:
        return f"⚠️ Error retrieving drug recall: {error}"

    if not response or not response.get("results"):
        return f"No recall record found for {recall_number}"

    recall = response["results"][0]

    # Format detailed recall information
    output = [f"## Drug Recall Details: {recall_number}\n"]

    # Basic information
    output.extend(_format_recall_header(recall))

    # Reason and details
    output.extend(_format_recall_details(recall))

    # Distribution information
    output.extend(_format_distribution_info(recall))

    # OpenFDA metadata
    if openfda := recall.get("openfda"):
        output.extend(_format_recall_openfda(openfda))

    output.append(f"\n{OPENFDA_DISCLAIMER}")

    return "\n".join(output)


def _format_recall_class_summary(results: list[dict[str, Any]]) -> list[str]:
    """Format summary of recall classifications."""
    output = []

    # Count by classification
    class_counts = {"Class I": 0, "Class II": 0, "Class III": 0}
    for recall in results:
        classification = recall.get("classification", "")
        if classification in class_counts:
            class_counts[classification] += 1

    if any(class_counts.values()):
        output.append("### Classification Summary:")
        if class_counts["Class I"]:
            output.append(
                f"- **Class I** (most serious): {class_counts['Class I']} recalls"
            )
        if class_counts["Class II"]:
            output.append(
                f"- **Class II** (moderate): {class_counts['Class II']} recalls"
            )
        if class_counts["Class III"]:
            output.append(
                f"- **Class III** (least serious): {class_counts['Class III']} recalls"
            )
        output.append("")

    return output


def _format_recall_summary(recall: dict[str, Any], num: int) -> list[str]:
    """Format a single recall summary."""
    output = [f"#### {num}. Recall {recall.get('recall_number', 'Unknown')}"]

    # Classification and status
    classification = recall.get("classification", "Unknown")
    status = recall.get("status", "Unknown")

    # Add severity indicator
    severity_emoji = {
        "Class I": "🔴",  # Most serious
        "Class II": "🟡",  # Moderate
        "Class III": "🟢",  # Least serious
    }.get(classification, "⚪")

    output.append(f"{severity_emoji} **{classification}** - {status}")

    # Date
    if init_date := recall.get("recall_initiation_date"):
        formatted_date = f"{init_date[:4]}-{init_date[4:6]}-{init_date[6:]}"
        output.append(f"**Initiated**: {formatted_date}")

    # Product description
    if product_desc := recall.get("product_description"):
        cleaned = truncate_text(clean_text(product_desc), 200)
        output.append(f"**Product**: {cleaned}")

    # OpenFDA names
    openfda = recall.get("openfda", {})
    if brand_names := openfda.get("brand_name"):
        output.append(f"**Brand**: {', '.join(brand_names[:3])}")

    # Reason for recall
    if reason := recall.get("reason_for_recall"):
        cleaned_reason = truncate_text(clean_text(reason), 300)
        output.append(f"\n**Reason**: {cleaned_reason}")

    # Firm name
    if firm := recall.get("recalling_firm"):
        output.append(f"\n**Recalling Firm**: {firm}")

    output.append("")
    return output


def _format_recall_header(recall: dict[str, Any]) -> list[str]:
    """Format the header section of detailed recall."""
    output = ["### Recall Information"]

    output.append(
        f"**Recall Number**: {recall.get('recall_number', 'Unknown')}"
    )
    output.append(
        f"**Classification**: {recall.get('classification', 'Unknown')}"
    )
    output.append(f"**Status**: {recall.get('status', 'Unknown')}")

    if event_id := recall.get("event_id"):
        output.append(f"**Event ID**: {event_id}")

    # Dates
    if init_date := recall.get("recall_initiation_date"):
        formatted = f"{init_date[:4]}-{init_date[4:6]}-{init_date[6:]}"
        output.append(f"**Initiation Date**: {formatted}")

    if report_date := recall.get("report_date"):
        formatted = f"{report_date[:4]}-{report_date[4:6]}-{report_date[6:]}"
        output.append(f"**Report Date**: {formatted}")

    if term_date := recall.get("termination_date"):
        formatted = f"{term_date[:4]}-{term_date[4:6]}-{term_date[6:]}"
        output.append(f"**Termination Date**: {formatted}")

    output.append("")
    return output


def _format_recall_details(recall: dict[str, Any]) -> list[str]:
    """Format recall details and reason."""
    output = ["### Product and Reason"]

    if product_desc := recall.get("product_description"):
        output.append(f"**Product Description**:\n{clean_text(product_desc)}")

    if reason := recall.get("reason_for_recall"):
        output.append(f"\n**Reason for Recall**:\n{clean_text(reason)}")

    if quantity := recall.get("product_quantity"):
        output.append(f"\n**Product Quantity**: {quantity}")

    if code_info := recall.get("code_info"):
        output.append(f"\n**Code Information**:\n{clean_text(code_info)}")

    output.append("")
    return output


def _format_distribution_info(recall: dict[str, Any]) -> list[str]:
    """Format distribution information."""
    output = ["### Distribution Information"]

    if firm := recall.get("recalling_firm"):
        output.append(f"**Recalling Firm**: {firm}")

    if city := recall.get("city"):
        state = recall.get("state", "")
        country = recall.get("country", "")
        location = city
        if state:
            location += f", {state}"
        if country:
            location += f", {country}"
        output.append(f"**Location**: {location}")

    if dist_pattern := recall.get("distribution_pattern"):
        output.append(
            f"\n**Distribution Pattern**:\n{clean_text(dist_pattern)}"
        )

    if action := recall.get("voluntary_mandated"):
        output.append(f"\n**Action Type**: {action}")

    output.append("")
    return output


def _format_recall_openfda(openfda: dict[str, Any]) -> list[str]:
    """Format OpenFDA metadata for recall."""
    output = ["### Drug Information"]

    if brand_names := openfda.get("brand_name"):
        output.append(f"**Brand Names**: {', '.join(brand_names)}")

    if generic_names := openfda.get("generic_name"):
        output.append(f"**Generic Names**: {', '.join(generic_names)}")

    if manufacturers := openfda.get("manufacturer_name"):
        output.append(f"**Manufacturers**: {', '.join(manufacturers[:3])}")

    if ndas := openfda.get("application_number"):
        output.append(f"**Application Numbers**: {', '.join(ndas[:5])}")

    if routes := openfda.get("route"):
        output.append(f"**Routes**: {', '.join(routes)}")

    if pharm_class := openfda.get("pharm_class_epc"):
        output.append(f"**Pharmacologic Class**: {', '.join(pharm_class[:3])}")

    output.append("")
    return output
