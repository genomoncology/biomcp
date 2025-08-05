"""Get specific intervention details via NCI CTS API."""

import logging
from typing import Any

from ..constants import NCI_INTERVENTIONS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request

logger = logging.getLogger(__name__)


async def get_intervention(
    intervention_id: str,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Get detailed information about a specific intervention.

    Args:
        intervention_id: Intervention ID
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with intervention details

    Raises:
        CTSAPIError: If the API request fails or intervention not found
    """
    try:
        # Make API request
        url = f"{NCI_INTERVENTIONS_URL}/{intervention_id}"
        response = await make_cts_request(
            url=url,
            api_key=api_key,
        )

        # Return the intervention data
        if "data" in response:
            return response["data"]
        elif "intervention" in response:
            return response["intervention"]
        else:
            return response

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to get intervention {intervention_id}: {e}")
        raise CTSAPIError(f"Failed to retrieve intervention: {e!s}") from e


def _format_intervention_header(intervention: dict[str, Any]) -> list[str]:
    """Format intervention header and basic info."""
    int_id = intervention.get(
        "id", intervention.get("intervention_id", "Unknown")
    )
    name = intervention.get("name", "Unknown Intervention")
    int_type = intervention.get(
        "type", intervention.get("category", "Unknown")
    )

    return [
        f"## Intervention: {name}",
        "",
        "### Basic Information",
        f"- **ID**: {int_id}",
        f"- **Type**: {int_type}",
    ]


def _format_intervention_synonyms(synonyms: Any) -> list[str]:
    """Format intervention synonyms section."""
    if not synonyms:
        return []

    lines = ["", "### Synonyms"]
    if isinstance(synonyms, list):
        for syn in synonyms:
            lines.append(f"- {syn}")
    else:
        lines.append(f"- {synonyms}")

    return lines


def _format_intervention_regulatory(intervention: dict[str, Any]) -> list[str]:
    """Format regulatory information section."""
    if not intervention.get("fda_approved"):
        return []

    lines = [
        "",
        "### Regulatory Status",
        f"- **FDA Approved**: {'Yes' if intervention['fda_approved'] else 'No'}",
    ]

    if intervention.get("approval_date"):
        lines.append(f"- **Approval Date**: {intervention['approval_date']}")

    return lines


def _format_intervention_indications(indications: Any) -> list[str]:
    """Format clinical indications section."""
    if not indications:
        return []

    lines = ["", "### Clinical Indications"]
    if isinstance(indications, list):
        for indication in indications:
            lines.append(f"- {indication}")
    else:
        lines.append(f"- {indications}")

    return lines


def format_intervention_details(intervention: dict[str, Any]) -> str:
    """
    Format intervention details as markdown.

    Args:
        intervention: Intervention data dictionary

    Returns:
        Formatted markdown string
    """
    lines = _format_intervention_header(intervention)

    # Add synonyms
    lines.extend(
        _format_intervention_synonyms(intervention.get("synonyms", []))
    )

    # Add description
    if intervention.get("description"):
        lines.extend([
            "",
            "### Description",
            intervention["description"],
        ])

    # Add mechanism of action for drugs
    if intervention.get("mechanism_of_action"):
        lines.extend([
            "",
            "### Mechanism of Action",
            intervention["mechanism_of_action"],
        ])

    # Add regulatory info
    lines.extend(_format_intervention_regulatory(intervention))

    # Add clinical indications
    lines.extend(
        _format_intervention_indications(intervention.get("indications"))
    )

    # Add related trials count if available
    if intervention.get("trial_count"):
        lines.extend([
            "",
            "### Clinical Trial Activity",
            f"- **Number of Trials**: {intervention['trial_count']}",
        ])

    return "\n".join(lines)
