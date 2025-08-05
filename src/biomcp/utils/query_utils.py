"""Utilities for query parsing and manipulation."""

import re
from typing import Any


def parse_or_query(query: str) -> list[str]:
    """Parse OR query into individual search terms.

    Handles formats like:
    - "term1 OR term2"
    - 'term1 OR term2 OR "term with spaces"'
    - "TERM1 or term2 or term3" (case insensitive)

    Args:
        query: Query string that may contain OR operators

    Returns:
        List of individual search terms with quotes and whitespace cleaned

    Examples:
        >>> parse_or_query("PD-L1 OR CD274")
        ['PD-L1', 'CD274']

        >>> parse_or_query('BRAF OR "v-raf murine" OR ARAF')
        ['BRAF', 'v-raf murine', 'ARAF']
    """
    # Split by OR (case insensitive)
    terms = re.split(r"\s+OR\s+", query, flags=re.IGNORECASE)

    # Clean up each term - remove quotes and extra whitespace
    cleaned_terms = []
    for term in terms:
        # Remove surrounding quotes (both single and double)
        term = term.strip().strip('"').strip("'").strip()
        if term:
            cleaned_terms.append(term)

    return cleaned_terms


def contains_or_operator(query: str) -> bool:
    """Check if a query contains OR operators.

    Args:
        query: Query string to check

    Returns:
        True if query contains " OR " or " or ", False otherwise
    """
    return " OR " in query or " or " in query


async def search_with_or_support(
    query: str,
    search_func: Any,
    search_params: dict[str, Any],
    id_field: str = "id",
    fallback_id_field: str | None = None,
) -> dict[str, Any]:
    """Generic OR query search handler.

    This function handles OR queries by making multiple API calls and combining results.

    Args:
        query: Query string that may contain OR operators
        search_func: Async search function to call for each term
        search_params: Base parameters to pass to search function (excluding the query term)
        id_field: Primary field name for deduplication (default: "id")
        fallback_id_field: Alternative field name if primary is missing

    Returns:
        Combined results from all searches with duplicates removed
    """
    # Check if this is an OR query
    if contains_or_operator(query):
        search_terms = parse_or_query(query)
    else:
        search_terms = [query]

    # Collect all unique results
    all_results = {}
    total_found = 0

    # Search for each term
    for term in search_terms:
        try:
            # Call the search function with the term
            results = await search_func(**{**search_params, "name": term})

            # Extract results list (handle different response formats)
            items_key = None
            for key in [
                "biomarkers",
                "organizations",
                "interventions",
                "diseases",
                "data",
                "items",
            ]:
                if key in results:
                    items_key = key
                    break

            if not items_key:
                continue

            # Add unique items (deduplicate by ID)
            for item in results.get(items_key, []):
                item_id = item.get(id_field)
                if not item_id and fallback_id_field:
                    item_id = item.get(fallback_id_field)

                if item_id and item_id not in all_results:
                    all_results[item_id] = item

            total_found += results.get("total", 0)

        except Exception as e:
            # Log the error and continue with other terms
            import logging

            logger = logging.getLogger(__name__)
            logger.warning(f"Failed to search for term '{term}': {e}")
            continue

    # Convert back to list
    unique_items = list(all_results.values())

    # Return in standard format
    return {
        "items": unique_items,
        "total": len(unique_items),
        "search_terms": search_terms,
        "total_found_across_terms": total_found,
    }
