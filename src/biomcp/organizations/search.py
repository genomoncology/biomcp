"""Search functionality for organizations via NCI CTS API."""

import logging
from typing import Any

from ..constants import NCI_ORGANIZATIONS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request

logger = logging.getLogger(__name__)


async def search_organizations(
    name: str | None = None,
    org_type: str | None = None,
    city: str | None = None,
    state: str | None = None,
    page_size: int = 20,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for organizations in the NCI CTS database.

    Args:
        name: Organization name to search for (partial match)
        org_type: Type of organization (e.g., "industry", "academic")
        city: City location
        state: State location (2-letter code)
        page_size: Number of results per page
        page: Page number
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with search results containing:
        - organizations: List of organization records
        - total: Total number of results
        - page: Current page
        - page_size: Results per page

    Raises:
        CTSAPIError: If the API request fails
    """
    # Build query parameters
    params: dict[str, Any] = {
        "size": page_size,
    }

    # Note: The NCI API doesn't support offset/page pagination for organizations
    # It uses cursor-based pagination or returns all results up to size limit

    # Add search filters with correct API parameter names
    if name:
        params["name"] = name
    if org_type:
        params["type"] = org_type
    if city:
        params["org_city"] = city
    if state:
        params["org_state_or_province"] = state

    try:
        # Make API request
        response = await make_cts_request(
            url=NCI_ORGANIZATIONS_URL,
            params=params,
            api_key=api_key,
        )

        # Process response - adapt to actual API format
        # This is a reasonable structure based on typical REST APIs
        organizations = response.get("data", response.get("organizations", []))
        total = response.get("total", len(organizations))

        return {
            "organizations": organizations,
            "total": total,
            "page": page,
            "page_size": page_size,
        }

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to search organizations: {e}")
        raise CTSAPIError(f"Organization search failed: {e!s}") from e


def format_organization_results(results: dict[str, Any]) -> str:
    """
    Format organization search results as markdown.

    Args:
        results: Search results dictionary

    Returns:
        Formatted markdown string
    """
    organizations = results.get("organizations", [])
    total = results.get("total", 0)

    if not organizations:
        return "No organizations found matching the search criteria."

    # Build markdown output
    lines = [
        f"## Organization Search Results ({total} found)",
        "",
    ]

    for org in organizations:
        org_id = org.get("id", org.get("org_id", "Unknown"))
        name = org.get("name", "Unknown Organization")
        org_type = org.get("type", org.get("category", "Unknown"))
        city = org.get("city", "")
        state = org.get("state", "")

        lines.append(f"### {name}")
        lines.append(f"- **ID**: {org_id}")
        lines.append(f"- **Type**: {org_type}")

        if city or state:
            location_parts = [p for p in [city, state] if p]
            lines.append(f"- **Location**: {', '.join(location_parts)}")

        lines.append("")

    return "\n".join(lines)
