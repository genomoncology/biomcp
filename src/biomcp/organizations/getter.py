"""Get specific organization details via NCI CTS API."""

import logging
from typing import Any

from ..constants import NCI_ORGANIZATIONS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request

logger = logging.getLogger(__name__)


async def get_organization(
    org_id: str,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Get detailed information about a specific organization.

    Args:
        org_id: Organization ID
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with organization details

    Raises:
        CTSAPIError: If the API request fails or organization not found
    """
    try:
        # Make API request
        url = f"{NCI_ORGANIZATIONS_URL}/{org_id}"
        response = await make_cts_request(
            url=url,
            api_key=api_key,
        )

        # Return the organization data
        # Handle different possible response formats
        if "data" in response:
            return response["data"]
        elif "organization" in response:
            return response["organization"]
        else:
            return response

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to get organization {org_id}: {e}")
        raise CTSAPIError(f"Failed to retrieve organization: {e!s}") from e


def _format_address_fields(org: dict[str, Any]) -> list[str]:
    """Extract and format address fields from organization data."""
    address_fields = []

    if org.get("address"):
        addr = org["address"]
        if isinstance(addr, dict):
            fields = [
                addr.get("street", ""),
                addr.get("city", ""),
                addr.get("state", ""),
                addr.get("zip", ""),
            ]
            address_fields = [f for f in fields if f]

            country = addr.get("country", "")
            if country and country != "United States":
                address_fields.append(country)
    else:
        # Try individual fields
        city = org.get("city", "")
        state = org.get("state", "")
        address_fields = [p for p in [city, state] if p]

    return address_fields


def _format_contact_info(org: dict[str, Any]) -> list[str]:
    """Format contact information lines."""
    lines = []
    if org.get("phone"):
        lines.append(f"- **Phone**: {org['phone']}")
    if org.get("email"):
        lines.append(f"- **Email**: {org['email']}")
    if org.get("website"):
        lines.append(f"- **Website**: {org['website']}")
    return lines


def format_organization_details(org: dict[str, Any]) -> str:
    """
    Format organization details as markdown.

    Args:
        org: Organization data dictionary

    Returns:
        Formatted markdown string
    """
    # Extract fields with defaults
    org_id = org.get("id", org.get("org_id", "Unknown"))
    name = org.get("name", "Unknown Organization")
    org_type = org.get("type", org.get("category", "Unknown"))

    # Build markdown output
    lines = [
        f"## Organization: {name}",
        "",
        "### Basic Information",
        f"- **ID**: {org_id}",
        f"- **Type**: {org_type}",
    ]

    # Add location if available
    address_fields = _format_address_fields(org)
    if address_fields:
        lines.append(f"- **Location**: {', '.join(address_fields)}")

    # Add contact info
    lines.extend(_format_contact_info(org))

    # Add description if available
    if org.get("description"):
        lines.extend([
            "",
            "### Description",
            org["description"],
        ])

    # Add parent organization metadata
    if org.get("parent_org"):
        lines.extend([
            "",
            "### Parent Organization",
            f"- **Name**: {org['parent_org'].get('name', 'Unknown')}",
            f"- **ID**: {org['parent_org'].get('id', 'Unknown')}",
        ])

    return "\n".join(lines)
