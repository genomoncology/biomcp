"""Search functionality for diseases via NCI CTS API."""

import logging
from typing import Any

from ..constants import NCI_DISEASES_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request
from ..utils import parse_or_query

logger = logging.getLogger(__name__)


def _build_disease_params(
    name: str | None,
    disease_type: str | None,
    category: str | None,
    codes: list[str] | None,
    parent_ids: list[str] | None,
    ancestor_ids: list[str] | None,
    include: list[str] | None,
    sort: str | None,
    order: str | None,
    page_size: int,
) -> dict[str, Any]:
    """Build query parameters for disease search."""
    params: dict[str, Any] = {"size": page_size}

    if name:
        params["name"] = name

    # Use 'type' parameter instead of 'category'
    if disease_type:
        params["type"] = disease_type
    elif category:  # Backward compatibility
        params["type"] = category

    if codes:
        params["codes"] = ",".join(codes) if isinstance(codes, list) else codes

    if parent_ids:
        params["parent_ids"] = (
            ",".join(parent_ids)
            if isinstance(parent_ids, list)
            else parent_ids
        )

    if ancestor_ids:
        params["ancestor_ids"] = (
            ",".join(ancestor_ids)
            if isinstance(ancestor_ids, list)
            else ancestor_ids
        )

    if include:
        params["include"] = (
            ",".join(include) if isinstance(include, list) else include
        )

    if sort:
        params["sort"] = sort
        if order:
            params["order"] = order.lower()

    return params


async def search_diseases(
    name: str | None = None,
    include_synonyms: bool = True,  # Deprecated - kept for backward compatibility
    category: str | None = None,
    disease_type: str | None = None,
    codes: list[str] | None = None,
    parent_ids: list[str] | None = None,
    ancestor_ids: list[str] | None = None,
    include: list[str] | None = None,
    sort: str | None = None,
    order: str | None = None,
    page_size: int = 20,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for diseases in the NCI CTS database.

    This provides access to NCI's controlled vocabulary of cancer conditions
    used in clinical trials, with official terms and synonyms.

    Args:
        name: Disease name to search for (partial match, searches synonyms automatically)
        include_synonyms: [Deprecated] This parameter is ignored - API always searches synonyms
        category: Disease category/type filter (deprecated - use disease_type)
        disease_type: Type of disease (e.g., 'maintype', 'subtype', 'stage')
        codes: List of disease codes (e.g., ['C3868', 'C5806'])
        parent_ids: List of parent disease IDs
        ancestor_ids: List of ancestor disease IDs
        include: Fields to include in response
        sort: Sort field
        order: Sort order ('asc' or 'desc')
        page_size: Number of results per page
        page: Page number
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with search results containing:
        - diseases: List of disease records with names and synonyms
        - total: Total number of results
        - page: Current page
        - page_size: Results per page

    Raises:
        CTSAPIError: If the API request fails
    """
    # Build query parameters
    params = _build_disease_params(
        name,
        disease_type,
        category,
        codes,
        parent_ids,
        ancestor_ids,
        include,
        sort,
        order,
        page_size,
    )

    try:
        # Make API request
        response = await make_cts_request(
            url=NCI_DISEASES_URL,
            params=params,
            api_key=api_key,
        )

        # Process response
        diseases = response.get("data", response.get("diseases", []))
        total = response.get("total", len(diseases))

        return {
            "diseases": diseases,
            "total": total,
            "page": page,
            "page_size": page_size,
        }

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to search diseases: {e}")
        raise CTSAPIError(f"Disease search failed: {e!s}") from e


async def get_disease_by_id(
    disease_id: str,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Get detailed information about a specific disease by ID.

    Args:
        disease_id: Disease ID from NCI CTS
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with disease details including synonyms

    Raises:
        CTSAPIError: If the API request fails
    """
    try:
        # Make API request
        url = f"{NCI_DISEASES_URL}/{disease_id}"
        response = await make_cts_request(
            url=url,
            api_key=api_key,
        )

        # Return the disease data
        if "data" in response:
            return response["data"]
        elif "disease" in response:
            return response["disease"]
        else:
            return response

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to get disease {disease_id}: {e}")
        raise CTSAPIError(f"Failed to retrieve disease: {e!s}") from e


def _format_disease_synonyms(synonyms: Any) -> list[str]:
    """Format disease synonyms section."""
    lines: list[str] = []
    if not synonyms:
        return lines

    if isinstance(synonyms, list) and synonyms:
        lines.append("- **Synonyms**:")
        for syn in synonyms[:5]:  # Show up to 5 synonyms
            lines.append(f"  - {syn}")
        if len(synonyms) > 5:
            lines.append(f"  *(and {len(synonyms) - 5} more)*")
    elif isinstance(synonyms, str):
        lines.append(f"- **Synonyms**: {synonyms}")

    return lines


def _format_disease_codes(codes: Any) -> list[str]:
    """Format disease code mappings."""
    if not codes or not isinstance(codes, dict):
        return []

    code_items = []
    for system, code in codes.items():
        code_items.append(f"{system}: {code}")

    if code_items:
        return [f"- **Codes**: {', '.join(code_items)}"]
    return []


def _format_single_disease(disease: dict[str, Any]) -> list[str]:
    """Format a single disease record."""
    disease_id = disease.get("id", disease.get("disease_id", "Unknown"))
    name = disease.get(
        "name", disease.get("preferred_name", "Unknown Disease")
    )
    category = disease.get("category", disease.get("type", ""))

    lines = [
        f"### {name}",
        f"- **ID**: {disease_id}",
    ]

    if category:
        lines.append(f"- **Category**: {category}")

    # Add synonyms
    lines.extend(_format_disease_synonyms(disease.get("synonyms", [])))

    # Add code mappings
    lines.extend(_format_disease_codes(disease.get("codes")))

    lines.append("")
    return lines


def format_disease_results(results: dict[str, Any]) -> str:
    """
    Format disease search results as markdown.

    Args:
        results: Search results dictionary

    Returns:
        Formatted markdown string
    """
    diseases = results.get("diseases", [])
    total = results.get("total", 0)

    if not diseases:
        return "No diseases found matching the search criteria."

    # Build markdown output
    lines = [
        f"## Disease Search Results ({total} found)",
        "",
    ]

    for disease in diseases:
        lines.extend(_format_single_disease(disease))

    return "\n".join(lines)


async def search_diseases_with_or(
    name_query: str,
    include_synonyms: bool = True,
    category: str | None = None,
    disease_type: str | None = None,
    codes: list[str] | None = None,
    parent_ids: list[str] | None = None,
    ancestor_ids: list[str] | None = None,
    include: list[str] | None = None,
    sort: str | None = None,
    order: str | None = None,
    page_size: int = 20,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for diseases with OR query support.

    This function handles OR queries by making multiple API calls and combining results.
    For example: "melanoma OR lung cancer" will search for each term.

    Args:
        name_query: Name query that may contain OR operators
        Other args same as search_diseases

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

    # Collect all unique diseases
    all_diseases = {}
    total_found = 0

    # Search for each term
    for term in search_terms:
        logger.info(f"Searching diseases for term: {term}")
        try:
            results = await search_diseases(
                name=term,
                include_synonyms=include_synonyms,
                category=category,
                disease_type=disease_type,
                codes=codes,
                parent_ids=parent_ids,
                ancestor_ids=ancestor_ids,
                include=include,
                sort=sort,
                order=order,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

            # Add unique diseases (deduplicate by ID)
            for disease in results.get("diseases", []):
                disease_id = disease.get("id", disease.get("disease_id"))
                if disease_id and disease_id not in all_diseases:
                    all_diseases[disease_id] = disease

            total_found += results.get("total", 0)

        except Exception as e:
            logger.warning(f"Failed to search for term '{term}': {e}")
            # Continue with other terms

    # Convert back to list and apply pagination
    unique_diseases = list(all_diseases.values())

    # Sort by name for consistent results
    unique_diseases.sort(
        key=lambda x: x.get("name", x.get("preferred_name", "")).lower()
    )

    # Apply pagination to combined results
    start_idx = (page - 1) * page_size
    end_idx = start_idx + page_size
    paginated_diseases = unique_diseases[start_idx:end_idx]

    return {
        "diseases": paginated_diseases,
        "total": len(unique_diseases),
        "page": page,
        "page_size": page_size,
        "search_terms": search_terms,  # Include what we searched for
        "total_found_across_terms": total_found,  # Total before deduplication
    }
