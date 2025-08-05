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


async def handle_nci_organization_search(
    name: str | None,
    organization_type: str | None,
    city: str | None,
    state: str | None,
    api_key: str | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle NCI organization domain search."""
    logger.info("Executing NCI organization search")

    try:
        from biomcp.organizations import (
            search_organizations,
            search_organizations_with_or,
        )

        # Check if name contains OR query
        if name and (" OR " in name or " or " in name):
            results = await search_organizations_with_or(
                name_query=name,
                org_type=organization_type,
                city=city,
                state=state,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        else:
            results = await search_organizations(
                name=name,
                org_type=organization_type,
                city=city,
                state=state,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

        items = results.get("organizations", [])
        total = results.get("total", len(items))

    except Exception as e:
        logger.error(f"NCI organization search failed: {e}")
        raise SearchExecutionError("nci_organization", e) from e

    logger.info(f"NCI organization search returned {total} results")
    return items, total


async def handle_nci_intervention_search(
    name: str | None,
    intervention_type: str | None,
    synonyms: bool,
    api_key: str | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle NCI intervention domain search."""
    logger.info("Executing NCI intervention search")

    try:
        from biomcp.interventions import (
            search_interventions,
            search_interventions_with_or,
        )

        # Check if name contains OR query
        if name and (" OR " in name or " or " in name):
            results = await search_interventions_with_or(
                name_query=name,
                intervention_type=intervention_type,
                synonyms=synonyms,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        else:
            results = await search_interventions(
                name=name,
                intervention_type=intervention_type,
                synonyms=synonyms,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

        items = results.get("interventions", [])
        total = results.get("total", len(items))

    except Exception as e:
        logger.error(f"NCI intervention search failed: {e}")
        raise SearchExecutionError("nci_intervention", e) from e

    logger.info(f"NCI intervention search returned {total} results")
    return items, total


async def handle_nci_biomarker_search(
    name: str | None,
    gene: str | None,
    biomarker_type: str | None,
    assay_type: str | None,
    api_key: str | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle NCI biomarker domain search."""
    logger.info("Executing NCI biomarker search")

    try:
        from biomcp.biomarkers import (
            search_biomarkers,
            search_biomarkers_with_or,
        )

        # Check if name contains OR query
        if name and (" OR " in name or " or " in name):
            results = await search_biomarkers_with_or(
                name_query=name,
                eligibility_criterion=gene,  # Map gene to eligibility_criterion
                biomarker_type=biomarker_type,
                assay_purpose=assay_type,  # Map assay_type to assay_purpose
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        else:
            results = await search_biomarkers(
                name=name,
                eligibility_criterion=gene,  # Map gene to eligibility_criterion
                biomarker_type=biomarker_type,
                assay_purpose=assay_type,  # Map assay_type to assay_purpose
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

        items = results.get("biomarkers", [])
        total = results.get("total", len(items))

    except Exception as e:
        logger.error(f"NCI biomarker search failed: {e}")
        raise SearchExecutionError("nci_biomarker", e) from e

    logger.info(f"NCI biomarker search returned {total} results")
    return items, total


async def handle_nci_disease_search(
    name: str | None,
    include_synonyms: bool,
    category: str | None,
    api_key: str | None,
    page: int,
    page_size: int,
) -> tuple[list[dict], int]:
    """Handle NCI disease domain search."""
    logger.info("Executing NCI disease search")

    try:
        from biomcp.diseases import search_diseases, search_diseases_with_or

        # Check if name contains OR query
        if name and (" OR " in name or " or " in name):
            results = await search_diseases_with_or(
                name_query=name,
                include_synonyms=include_synonyms,
                category=category,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )
        else:
            results = await search_diseases(
                name=name,
                include_synonyms=include_synonyms,
                category=category,
                page_size=page_size,
                page=page,
                api_key=api_key,
            )

        items = results.get("diseases", [])
        total = results.get("total", len(items))

    except Exception as e:
        logger.error(f"NCI disease search failed: {e}")
        raise SearchExecutionError("nci_disease", e) from e

    logger.info(f"NCI disease search returned {total} results")
    return items, total
