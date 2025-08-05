"""Search functionality for organizations via NCI CTS API."""

import logging
from typing import Any

from ..constants import NCI_ORGANIZATIONS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request
from ..utils import parse_or_query

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


async def search_organizations_with_or(
    name_query: str,
    org_type: str | None = None,
    city: str | None = None,
    state: str | None = None,
    page_size: int = 20,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for organizations with OR query support.

    This function handles OR queries by making multiple API calls and combining results.
    For example: "MD Anderson OR Mayo Clinic" will search for each term.

    Args:
        name_query: Name query that may contain OR operators
        Other args same as search_organizations

    Returns:
        Combined results from all searches with duplicates removed
    """
    # Check if this is an OR query
    if " OR " in name_query or " or " in name_query:
        search_terms = parse_or_query(name_query)
        logger.info(f"Parsed OR query into terms: {search_terms}")
    else:
        # Single term search
        search_terms = [name_query]

    # Collect all unique organizations
    all_organizations = {}
    total_found = 0

    # Search for each term
    for term in search_terms:
        logger.info(f"Searching organizations for term: {term}")
        try:
            results = await search_organizations(
                name=term,
                org_type=org_type,
                city=city,
                state=state,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

            # Add unique organizations (deduplicate by ID)
            for org in results.get("organizations", []):
                org_id = org.get("id", org.get("org_id"))
                if org_id and org_id not in all_organizations:
                    all_organizations[org_id] = org

            total_found += results.get("total", 0)

        except Exception as e:
            logger.warning(f"Failed to search for term '{term}': {e}")
            # Continue with other terms

    # Convert back to list and apply pagination
    unique_organizations = list(all_organizations.values())

    # Sort by name for consistent results
    unique_organizations.sort(key=lambda x: x.get("name", "").lower())

    # Apply pagination to combined results
    start_idx = (page - 1) * page_size
    end_idx = start_idx + page_size
    paginated_organizations = unique_organizations[start_idx:end_idx]

    return {
        "organizations": paginated_organizations,
        "total": len(unique_organizations),
        "page": page,
        "page_size": page_size,
        "search_terms": search_terms,  # Include what we searched for
        "total_found_across_terms": total_found,  # Total before deduplication
    }
