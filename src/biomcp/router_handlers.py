"""Domain-specific search handlers for the router module."""

import json
import logging
from typing import Any

from .exceptions import (
    InvalidParameterError,
    ResultParsingError,
    SearchExecutionError,
)
from .parameter_parser import ParameterParser

logger = logging.getLogger(__name__)


async def handle_article_search(
    genes: list[str] | None,
    diseases: list[str] | None,
    variants: list[str] | None,
    chemicals: list[str] | None,
    keywords: list[str] | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle article domain search."""
    logger.info("Executing article search")
    try:
        from biomcp.articles.search import PubmedRequest
        from biomcp.articles.unified import search_articles_unified

        request = PubmedRequest(
            chemicals=chemicals or [],
            diseases=diseases or [],
            genes=genes or [],
            keywords=keywords or [],
            variants=variants or [],
        )
        result_str = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=True,  # Changed to match individual tool default
            output_json=True,
        )
    except Exception as e:
        logger.error(f"Article search failed: {e}")
        raise SearchExecutionError("article", e) from e

    # Parse the JSON results
    try:
        parsed_result = json.loads(result_str)
        # Handle unified search format (may include cBioPortal data)
        if isinstance(parsed_result, dict) and "articles" in parsed_result:
            all_results = parsed_result["articles"]
            # Log if cBioPortal data was included
            if "cbioportal_summary" in parsed_result:
                logger.info("Article search included cBioPortal summary data")
        elif isinstance(parsed_result, list):
            all_results = parsed_result
        else:
            # Handle unexpected format
            logger.warning(
                f"Unexpected article result format: {type(parsed_result)}"
            )
            all_results = []
    except (json.JSONDecodeError, TypeError) as e:
        logger.error(f"Failed to parse article results: {e}")
        raise ResultParsingError("article", e) from e

    # Manual pagination
    start = (page - 1) * page_size
    end = start + page_size
    items = all_results[start:end]
    total = len(all_results)

    logger.info(
        f"Article search returned {total} total results, showing {len(items)}"
    )

    return items, total


def _parse_trial_results(result_str: str) -> tuple[list[dict], int]:
    """Parse trial search results from JSON."""
    try:
        result_dict = json.loads(result_str)
        # Handle both API v2 structure and flat structure
        if isinstance(result_dict, dict) and "studies" in result_dict:
            all_results = result_dict["studies"]
        elif isinstance(result_dict, list):
            all_results = result_dict
        else:
            all_results = [result_dict]
    except (json.JSONDecodeError, TypeError) as e:
        logger.error(f"Failed to parse trial results: {e}")
        raise ResultParsingError("trial", e) from e

    return all_results, len(all_results)


async def handle_trial_search(
    conditions: list[str] | None,
    interventions: list[str] | None,
    keywords: list[str] | None,
    recruiting_status: str | None,
    phase: str | None,
    genes: list[str] | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle trial domain search."""
    logger.info("Executing trial search")

    # Build the trial search parameters
    search_params: dict[str, Any] = {}
    if conditions:
        search_params["conditions"] = conditions
    if interventions:
        search_params["interventions"] = interventions
    if recruiting_status:
        search_params["recruiting_status"] = recruiting_status
    if phase:
        try:
            search_params["phase"] = ParameterParser.normalize_phase(phase)
        except InvalidParameterError:
            raise
    if keywords:
        search_params["keywords"] = keywords

    # Add gene support for trials
    if genes:
        # Convert genes to keywords for trial search
        if "keywords" in search_params:
            search_params["keywords"].extend(genes)
        else:
            search_params["keywords"] = genes

    try:
        from biomcp.trials.search import TrialQuery, search_trials

        # Convert search_params to TrialQuery
        trial_query = TrialQuery(**search_params, page_size=page_size)
        result_str = await search_trials(trial_query, output_json=True)
    except Exception as e:
        logger.error(f"Trial search failed: {e}")
        raise SearchExecutionError("trial", e) from e

    # Parse the JSON results
    all_results, total = _parse_trial_results(result_str)

    # Manual pagination
    start = (page - 1) * page_size
    end = start + page_size
    items = all_results[start:end]

    logger.info(
        f"Trial search returned {total} total results, showing {len(items)}"
    )

    return items, total


async def handle_variant_search(
    genes: list[str] | None,
    significance: str | None,
    keywords: list[str] | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle variant domain search."""
    logger.info("Executing variant search")

    try:
        from biomcp.variants.search import VariantQuery, search_variants

        # Build query
        queries = []
        if genes:
            queries.extend(genes)
        if keywords:
            queries.extend(keywords)

        if not queries:
            raise InvalidParameterError(
                "genes or keywords",
                None,
                "at least one search term for variant search",
            )

        request = VariantQuery(
            gene=genes[0] if genes else None,
            size=page_size,
            significance=significance,
        )
        result_str = await search_variants(request, output_json=True)
    except Exception as e:
        logger.error(f"Variant search failed: {e}")
        raise SearchExecutionError("variant", e) from e

    # Parse the JSON results
    try:
        all_results = json.loads(result_str)
    except (json.JSONDecodeError, TypeError) as e:
        logger.error(f"Failed to parse variant results: {e}")
        raise ResultParsingError("variant", e) from e

    # Variants API returns paginated results
    total = len(all_results)

    logger.info(f"Variant search returned {total} results")

    return all_results, total
