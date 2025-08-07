"""
Utility functions for OpenFDA API integration.
"""

import logging
import os
from typing import Any

from ..http_client import request_api

logger = logging.getLogger(__name__)


def get_api_key() -> str | None:
    """Get OpenFDA API key from environment variable."""
    api_key = os.environ.get("OPENFDA_API_KEY")
    if not api_key:
        logger.debug("No OPENFDA_API_KEY found in environment")
    return api_key


async def make_openfda_request(
    endpoint: str,
    params: dict[str, Any],
    domain: str = "openfda",
    api_key: str | None = None,
) -> tuple[dict[str, Any] | None, str | None]:
    """
    Make a request to the OpenFDA API.

    Args:
        endpoint: Full URL to the OpenFDA endpoint
        params: Query parameters
        domain: Domain name for metrics tracking
        api_key: Optional API key (overrides environment variable)

    Returns:
        Tuple of (response_data, error_message)
    """
    # Use provided API key or get from environment
    if not api_key:
        api_key = get_api_key()
    if api_key:
        params["api_key"] = api_key

    try:
        response, error = await request_api(
            url=endpoint, request=params, method="GET", domain=domain
        )

        if error:
            error_msg = (
                error.message if hasattr(error, "message") else str(error)
            )
            logger.error(f"OpenFDA API error: {error_msg}")
            return None, error_msg

        return response, None

    except Exception as e:
        logger.error(f"OpenFDA request failed: {e}")
        return None, str(e)


def format_count(count: int, label: str) -> str:
    """Format a count with appropriate singular/plural label."""
    if count == 1:
        return f"1 {label}"
    return f"{count:,} {label}s"


def truncate_text(text: str, max_length: int = 500) -> str:
    """Truncate text to a maximum length with ellipsis."""
    if len(text) <= max_length:
        return text
    return text[: max_length - 3] + "..."


def clean_text(text: str | None) -> str:
    """Clean and normalize text from FDA data."""
    if not text:
        return ""

    # Remove extra whitespace and newlines
    text = " ".join(text.split())

    # Remove common FDA formatting artifacts
    text = text.replace("\\n", " ")
    text = text.replace("\\r", " ")
    text = text.replace("\\t", " ")

    return text.strip()


def build_search_query(
    field_map: dict[str, str], operator: str = "AND"
) -> str:
    """
    Build an OpenFDA search query from field mappings.

    Args:
        field_map: Dictionary mapping field names to search values
        operator: Logical operator (AND/OR) to combine fields

    Returns:
        Formatted search query string
    """
    query_parts = []

    for field, value in field_map.items():
        if value:
            # Escape special characters
            escaped_value = value.replace('"', '\\"')
            # Add quotes for multi-word values
            if " " in escaped_value:
                escaped_value = f'"{escaped_value}"'
            query_parts.append(f"{field}:{escaped_value}")

    return f" {operator} ".join(query_parts)


def extract_drug_names(result: dict[str, Any]) -> list[str]:
    """Extract drug names from an OpenFDA result."""
    drug_names = set()

    # Check patient drug info (for adverse events)
    if "patient" in result:
        drugs = result.get("patient", {}).get("drug", [])
        for drug in drugs:
            if "medicinalproduct" in drug:
                drug_names.add(drug["medicinalproduct"])
            # Check OpenFDA fields
            openfda = drug.get("openfda", {})
            if "brand_name" in openfda:
                drug_names.update(openfda["brand_name"])
            if "generic_name" in openfda:
                drug_names.update(openfda["generic_name"])

    # Check direct OpenFDA fields (for labels)
    if "openfda" in result:
        openfda = result["openfda"]
        if "brand_name" in openfda:
            drug_names.update(openfda["brand_name"])
        if "generic_name" in openfda:
            drug_names.update(openfda["generic_name"])

    return sorted(drug_names)


def extract_reactions(result: dict[str, Any]) -> list[str]:
    """Extract reaction terms from an adverse event result."""
    reactions = []

    patient = result.get("patient", {})
    reaction_list = patient.get("reaction", [])

    for reaction in reaction_list:
        if "reactionmeddrapt" in reaction:
            reactions.append(reaction["reactionmeddrapt"])

    return reactions


def format_drug_list(drugs: list[str], max_items: int = 5) -> str:
    """Format a list of drug names for display."""
    if not drugs:
        return "None specified"

    if len(drugs) <= max_items:
        return ", ".join(drugs)

    shown = drugs[:max_items]
    remaining = len(drugs) - max_items
    return f"{', '.join(shown)} (+{remaining} more)"
