"""Search functionality for interventions via NCI CTS API."""

import logging
from typing import Any

from ..constants import NCI_INTERVENTIONS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request
from ..utils import parse_or_query

logger = logging.getLogger(__name__)


# Intervention types based on ClinicalTrials.gov categories
INTERVENTION_TYPES = [
    "Drug",
    "Device",
    "Biological",
    "Procedure",
    "Radiation",
    "Behavioral",
    "Genetic",
    "Dietary",
    "Diagnostic Test",
    "Other",
]


def _build_intervention_params(
    name: str | None,
    intervention_type: str | None,
    category: str | None,
    codes: list[str] | None,
    include: list[str] | None,
    sort: str | None,
    order: str | None,
    page_size: int | None,
) -> dict[str, Any]:
    """Build query parameters for intervention search."""
    params: dict[str, Any] = {}

    if name:
        params["name"] = name

    if intervention_type:
        params["type"] = intervention_type.lower()

    if category:
        params["category"] = category

    if codes:
        params["codes"] = ",".join(codes) if isinstance(codes, list) else codes

    if include:
        params["include"] = (
            ",".join(include) if isinstance(include, list) else include
        )

    if sort:
        params["sort"] = sort
        if order:
            params["order"] = order.lower()

    # Only add size if explicitly requested and > 0
    if page_size and page_size > 0:
        params["size"] = page_size

    return params


def _process_intervention_response(
    response: Any,
    page: int,
    page_size: int | None,
) -> dict[str, Any]:
    """Process intervention search response."""
    if isinstance(response, dict):
        # Standard response format from the API
        interventions = response.get("data", [])
        # When size parameter is used, API doesn't return 'total'
        total = response.get("total", len(interventions))
    elif isinstance(response, list):
        # Direct list of interventions
        interventions = response
        total = len(interventions)
    else:
        # Unexpected response format
        logger.warning(f"Unexpected response type: {type(response)}")
        interventions = []
        total = 0

    return {
        "interventions": interventions,
        "total": total,
        "page": page,
        "page_size": page_size,
    }


async def search_interventions(
    name: str | None = None,
    intervention_type: str | None = None,
    category: str | None = None,
    codes: list[str] | None = None,
    include: list[str] | None = None,
    sort: str | None = None,
    order: str | None = None,
    synonyms: bool = True,  # Kept for backward compatibility but ignored
    page_size: int | None = None,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for interventions in the NCI CTS database.

    Args:
        name: Intervention name to search for (partial match)
        intervention_type: Type of intervention (Drug, Device, Procedure, etc.)
        category: Category filter (agent, agent category, other)
        codes: List of intervention codes to search for (e.g., ["C82416", "C171257"])
        include: Fields to include in response (all fields, name, category, codes, etc.)
        sort: Sort field (default: 'name', also supports 'count')
        order: Sort order ('asc' or 'desc', required when using sort)
        synonyms: [Deprecated] Kept for backward compatibility but ignored
        page_size: Number of results per page (when used, 'total' field not returned)
        page: Page number (Note: API doesn't support offset pagination)
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with search results containing:
        - interventions: List of intervention records
        - total: Total number of results (only when size not specified)
        - page: Current page
        - page_size: Results per page

    Raises:
        CTSAPIError: If the API request fails
    """
    # Build query parameters
    params = _build_intervention_params(
        name,
        intervention_type,
        category,
        codes,
        include,
        sort,
        order,
        page_size,
    )

    logger.info(
        f"Searching interventions at {NCI_INTERVENTIONS_URL} with params: {params}"
    )

    try:
        # Make API request
        response = await make_cts_request(
            url=NCI_INTERVENTIONS_URL,
            params=params,
            api_key=api_key,
        )

        # Log response info
        logger.debug(f"Response type: {type(response)}")

        # Process response
        return _process_intervention_response(response, page, page_size)

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to search interventions: {e}")
        raise CTSAPIError(f"Intervention search failed: {e!s}") from e


def format_intervention_results(results: dict[str, Any]) -> str:
    """
    Format intervention search results as markdown.

    Args:
        results: Search results dictionary

    Returns:
        Formatted markdown string
    """
    interventions = results.get("interventions", [])
    total = results.get("total", 0)

    if not interventions:
        return "No interventions found matching the search criteria."

    # Build markdown output
    actual_count = len(interventions)
    if actual_count < total:
        lines = [
            f"## Intervention Search Results (showing {actual_count} of {total} found)",
            "",
        ]
    else:
        lines = [
            f"## Intervention Search Results ({total} found)",
            "",
        ]

    for intervention in interventions:
        int_id = intervention.get(
            "id", intervention.get("intervention_id", "Unknown")
        )
        name = intervention.get("name", "Unknown Intervention")
        int_type = intervention.get(
            "type", intervention.get("category", "Unknown")
        )

        lines.append(f"### {name}")
        lines.append(f"- **ID**: {int_id}")
        lines.append(f"- **Type**: {int_type}")

        # Add synonyms if available
        synonyms = intervention.get("synonyms", [])
        if synonyms:
            if isinstance(synonyms, list):
                lines.append(f"- **Synonyms**: {', '.join(synonyms[:5])}")
                if len(synonyms) > 5:
                    lines.append(f"  *(and {len(synonyms) - 5} more)*")
            elif isinstance(synonyms, str):
                lines.append(f"- **Synonyms**: {synonyms}")

        # Add description if available
        if intervention.get("description"):
            desc = intervention["description"]
            if len(desc) > 200:
                desc = desc[:197] + "..."
            lines.append(f"- **Description**: {desc}")

        lines.append("")

    return "\n".join(lines)


async def search_interventions_with_or(
    name_query: str,
    intervention_type: str | None = None,
    category: str | None = None,
    codes: list[str] | None = None,
    include: list[str] | None = None,
    sort: str | None = None,
    order: str | None = None,
    synonyms: bool = True,
    page_size: int | None = None,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for interventions with OR query support.

    This function handles OR queries by making multiple API calls and combining results.
    For example: "pembrolizumab OR nivolumab" will search for each term.

    Args:
        name_query: Name query that may contain OR operators
        Other args same as search_interventions

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

    # Collect all unique interventions
    all_interventions = {}
    total_found = 0

    # Search for each term
    for term in search_terms:
        logger.info(f"Searching interventions for term: {term}")
        try:
            results = await search_interventions(
                name=term,
                intervention_type=intervention_type,
                category=category,
                codes=codes,
                include=include,
                sort=sort,
                order=order,
                synonyms=synonyms,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

            # Add unique interventions (deduplicate by ID)
            for intervention in results.get("interventions", []):
                int_id = intervention.get(
                    "id", intervention.get("intervention_id")
                )
                if int_id and int_id not in all_interventions:
                    all_interventions[int_id] = intervention

            total_found += results.get("total", 0)

        except Exception as e:
            logger.warning(f"Failed to search for term '{term}': {e}")
            # Continue with other terms

    # Convert back to list and apply pagination
    unique_interventions = list(all_interventions.values())

    # Sort by name for consistent results
    unique_interventions.sort(key=lambda x: x.get("name", "").lower())

    # Apply pagination to combined results
    if page_size:
        start_idx = (page - 1) * page_size
        end_idx = start_idx + page_size
        paginated_interventions = unique_interventions[start_idx:end_idx]
    else:
        paginated_interventions = unique_interventions

    return {
        "interventions": paginated_interventions,
        "total": len(unique_interventions),
        "page": page,
        "page_size": page_size,
        "search_terms": search_terms,  # Include what we searched for
        "total_found_across_terms": total_found,  # Total before deduplication
    }
