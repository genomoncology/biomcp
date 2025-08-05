"""Search functionality for biomarkers via NCI CTS API.

Note: Biomarker data availability may be limited in CTRP.
This module focuses on biomarkers used in trial eligibility criteria.
"""

import logging
from typing import Any

from ..constants import NCI_BIOMARKERS_URL
from ..integrations.cts_api import CTSAPIError, make_cts_request
from ..utils import parse_or_query

logger = logging.getLogger(__name__)


def _build_biomarker_params(
    name: str | None,
    eligibility_criterion: str | None,
    biomarker_type: str | None,
    codes: list[str] | None,
    assay_purpose: str | None,
    include: list[str] | None,
    sort: str | None,
    order: str | None,
    page_size: int,
) -> dict[str, Any]:
    """Build query parameters for biomarker search."""
    params: dict[str, Any] = {"size": page_size}

    # Add search filters with correct API parameter names
    if name:
        params["name"] = name
    if eligibility_criterion:
        params["eligibility_criterion"] = eligibility_criterion
    if biomarker_type:
        params["type"] = biomarker_type
    if codes:
        params["codes"] = ",".join(codes) if isinstance(codes, list) else codes
    if assay_purpose:
        params["assay_purpose"] = assay_purpose
    if include:
        params["include"] = (
            ",".join(include) if isinstance(include, list) else include
        )
    if sort:
        params["sort"] = sort
        if order:
            params["order"] = order.lower()

    return params


def _process_biomarker_response(
    response: dict[str, Any],
    page: int,
    page_size: int,
) -> dict[str, Any]:
    """Process biomarker API response."""
    biomarkers = response.get("data", response.get("biomarkers", []))
    total = response.get("total", len(biomarkers))

    result = {
        "biomarkers": biomarkers,
        "total": total,
        "page": page,
        "page_size": page_size,
    }

    # Add note about data limitations if response indicates it
    if response.get("limited_data") or not biomarkers:
        result["note"] = (
            "Biomarker data availability is limited in CTRP. "
            "Results show biomarkers referenced in trial eligibility criteria. "
            "For detailed variant annotations, use variant_searcher with MyVariant.info."
        )

    return result


async def search_biomarkers(
    name: str | None = None,
    eligibility_criterion: str | None = None,
    biomarker_type: str | None = None,
    codes: list[str] | None = None,
    assay_purpose: str | None = None,
    include: list[str] | None = None,
    sort: str | None = None,
    order: str | None = None,
    page_size: int = 20,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for biomarkers in the NCI CTS database.

    Note: Biomarker data availability may be limited per CTRP documentation.
    Results focus on biomarkers used in clinical trial eligibility criteria.

    Args:
        name: Biomarker name to search for (e.g., "PD-L1", "EGFR mutation")
        eligibility_criterion: Eligibility criterion text
        biomarker_type: Type of biomarker ("reference_gene" or "branch")
        codes: List of biomarker codes
        assay_purpose: Purpose of the assay
        include: Fields to include in response
        sort: Sort field
        order: Sort order ('asc' or 'desc')
        page_size: Number of results per page
        page: Page number
        api_key: Optional API key (if not provided, uses NCI_API_KEY env var)

    Returns:
        Dictionary with search results containing:
        - biomarkers: List of biomarker records
        - total: Total number of results
        - page: Current page
        - page_size: Results per page
        - note: Any limitations about the data

    Raises:
        CTSAPIError: If the API request fails
    """
    # Build query parameters
    params = _build_biomarker_params(
        name,
        eligibility_criterion,
        biomarker_type,
        codes,
        assay_purpose,
        include,
        sort,
        order,
        page_size,
    )

    try:
        # Make API request
        response = await make_cts_request(
            url=NCI_BIOMARKERS_URL,
            params=params,
            api_key=api_key,
        )

        # Process response
        return _process_biomarker_response(response, page, page_size)

    except CTSAPIError:
        raise
    except Exception as e:
        logger.error(f"Failed to search biomarkers: {e}")
        raise CTSAPIError(f"Biomarker search failed: {e!s}") from e


def _format_biomarker_header(total: int, note: str) -> list[str]:
    """Format the header section of biomarker results."""
    lines = [
        f"## Biomarker Search Results ({total} found)",
        "",
    ]

    if note:
        lines.extend([
            f"*Note: {note}*",
            "",
        ])

    return lines


def _format_single_biomarker(biomarker: dict[str, Any]) -> list[str]:
    """Format a single biomarker record."""
    bio_id = biomarker.get("id", biomarker.get("biomarker_id", "Unknown"))
    name = biomarker.get("name", "Unknown Biomarker")
    gene = biomarker.get("gene", biomarker.get("gene_symbol", ""))
    bio_type = biomarker.get("type", biomarker.get("category", ""))

    lines = [
        f"### {name}",
        f"- **ID**: {bio_id}",
    ]

    if gene:
        lines.append(f"- **Gene**: {gene}")
    if bio_type:
        lines.append(f"- **Type**: {bio_type}")

    # Add assay information if available
    if biomarker.get("assay_type"):
        lines.append(f"- **Assay**: {biomarker['assay_type']}")

    # Add criteria examples if available
    if biomarker.get("criteria_examples"):
        examples = biomarker["criteria_examples"]
        if isinstance(examples, list) and examples:
            lines.append("- **Example Criteria**:")
            for ex in examples[:3]:  # Show up to 3 examples
                lines.append(f"  - {ex}")
            if len(examples) > 3:
                lines.append(f"  *(and {len(examples) - 3} more)*")

    # Add trial count if available
    if biomarker.get("trial_count"):
        lines.append(
            f"- **Trials Using This Biomarker**: {biomarker['trial_count']}"
        )

    lines.append("")
    return lines


async def search_biomarkers_with_or(
    name_query: str,
    eligibility_criterion: str | None = None,
    biomarker_type: str | None = None,
    codes: list[str] | None = None,
    assay_purpose: str | None = None,
    include: list[str] | None = None,
    sort: str | None = None,
    order: str | None = None,
    page_size: int = 20,
    page: int = 1,
    api_key: str | None = None,
) -> dict[str, Any]:
    """
    Search for biomarkers with OR query support.

    This function handles OR queries by making multiple API calls and combining results.
    For example: "PD-L1 OR CD274 OR programmed death ligand 1" will search for each term.

    Args:
        name_query: Name query that may contain OR operators
        Other args same as search_biomarkers

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

    # Collect all unique biomarkers
    all_biomarkers = {}
    total_found = 0

    # Search for each term
    for term in search_terms:
        logger.info(f"Searching biomarkers for term: {term}")
        try:
            results = await search_biomarkers(
                name=term,
                eligibility_criterion=eligibility_criterion,
                biomarker_type=biomarker_type,
                codes=codes,
                assay_purpose=assay_purpose,
                include=include,
                sort=sort,
                order=order,
                page_size=page_size,  # Get full page size for each term
                page=page,
                api_key=api_key,
            )

            # Add unique biomarkers (deduplicate by ID)
            for biomarker in results.get("biomarkers", []):
                bio_id = biomarker.get("id", biomarker.get("biomarker_id"))
                if bio_id and bio_id not in all_biomarkers:
                    all_biomarkers[bio_id] = biomarker

            total_found += results.get("total", 0)

        except Exception as e:
            logger.warning(f"Failed to search for term '{term}': {e}")
            # Continue with other terms

    # Convert back to list and apply pagination
    unique_biomarkers = list(all_biomarkers.values())

    # Sort if requested (by name by default for consistent results)
    if sort == "name" or sort is None:
        unique_biomarkers.sort(key=lambda x: x.get("name", "").lower())

    # Apply pagination to combined results
    start_idx = (page - 1) * page_size
    end_idx = start_idx + page_size
    paginated_biomarkers = unique_biomarkers[start_idx:end_idx]

    return {
        "biomarkers": paginated_biomarkers,
        "total": len(unique_biomarkers),
        "page": page,
        "page_size": page_size,
        "search_terms": search_terms,  # Include what we searched for
        "total_found_across_terms": total_found,  # Total before deduplication
    }


def format_biomarker_results(results: dict[str, Any]) -> str:
    """
    Format biomarker search results as markdown.

    Args:
        results: Search results dictionary

    Returns:
        Formatted markdown string
    """
    biomarkers = results.get("biomarkers", [])
    total = results.get("total", 0)
    note = results.get("note", "")

    if not biomarkers:
        msg = "No biomarkers found matching the search criteria."
        if note:
            msg += f"\n\n*Note: {note}*"
        return msg

    # Build markdown output
    lines = _format_biomarker_header(total, note)

    for biomarker in biomarkers:
        lines.extend(_format_single_biomarker(biomarker))

    return "\n".join(lines)
