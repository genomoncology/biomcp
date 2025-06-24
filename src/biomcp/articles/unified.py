"""Unified article search combining PubMed and preprint sources."""

import asyncio
import json
import logging

from .. import render
from .preprints import search_preprints
from .search import PubmedRequest, search_articles

logger = logging.getLogger(__name__)


def _deduplicate_articles(articles: list[dict]) -> list[dict]:
    """Remove duplicate articles based on DOI."""
    seen_dois = set()
    unique_articles = []
    for article in articles:
        doi = article.get("doi")
        if doi and doi in seen_dois:
            continue
        if doi:
            seen_dois.add(doi)
        unique_articles.append(article)
    return unique_articles


def _parse_search_results(results: list) -> list[dict]:
    """Parse search results from JSON strings."""
    all_articles = []
    for result in results:
        if isinstance(result, str):
            try:
                articles = json.loads(result)
                if isinstance(articles, list):
                    all_articles.extend(articles)
            except json.JSONDecodeError:
                continue
    return all_articles


def _extract_mutation_pattern(
    keywords: list[str],
) -> tuple[str | None, str | None]:
    """Extract mutation pattern from keywords."""
    if not keywords:
        return None, None

    import re

    for keyword in keywords:
        # Check for specific mutations (e.g., F57Y, V600E)
        if re.match(r"^[A-Z]\d+[A-Z*]$", keyword):
            if keyword.endswith("*"):
                return keyword, None  # mutation_pattern
            else:
                return None, keyword  # specific_mutation
    return None, None


async def _get_mutation_summary(
    gene: str, mutation: str | None, pattern: str | None
) -> str | None:
    """Get mutation-specific cBioPortal summary."""
    from ..variants.cbioportal_mutations import (
        CBioPortalMutationClient,
        format_mutation_search_result,
    )

    mutation_client = CBioPortalMutationClient()

    if mutation:
        logger.info(f"Searching for specific mutation {gene} {mutation}")
        result = await mutation_client.search_specific_mutation(
            gene=gene, mutation=mutation, max_studies=20
        )
    else:
        logger.info(f"Searching for mutation pattern {gene} {pattern}")
        result = await mutation_client.search_specific_mutation(
            gene=gene, pattern=pattern, max_studies=20
        )

    return format_mutation_search_result(result) if result else None


async def _get_gene_summary(gene: str) -> str | None:
    """Get regular gene cBioPortal summary."""
    from ..variants.cbioportal_search import (
        CBioPortalSearchClient,
        format_cbioportal_search_summary,
    )

    client = CBioPortalSearchClient()
    summary = await client.get_gene_search_summary(gene, max_studies=5)
    return format_cbioportal_search_summary(summary) if summary else None


async def _get_cbioportal_summary(request: PubmedRequest) -> str | None:
    """Get cBioPortal summary for the search request."""
    if not request.genes:
        return None

    try:
        gene = request.genes[0]
        mutation_pattern, specific_mutation = _extract_mutation_pattern(
            request.keywords
        )

        if specific_mutation or mutation_pattern:
            return await _get_mutation_summary(
                gene, specific_mutation, mutation_pattern
            )
        else:
            return await _get_gene_summary(gene)

    except Exception as e:
        logger.warning(
            f"Failed to get cBioPortal summary for gene search: {e}"
        )
        return None


async def search_articles_unified(
    request: PubmedRequest,
    include_pubmed: bool = True,
    include_preprints: bool = False,
    output_json: bool = False,
) -> str:
    """Search for articles across PubMed and preprint sources."""
    tasks = []

    if include_pubmed:
        tasks.append(search_articles(request, output_json=True))

    if include_preprints:
        tasks.append(search_preprints(request, output_json=True))

    if not tasks:
        return json.dumps([]) if output_json else render.to_markdown([])

    # Run searches in parallel
    results = await asyncio.gather(*tasks, return_exceptions=True)

    # Parse and deduplicate results
    all_articles = _parse_search_results(results)
    unique_articles = _deduplicate_articles(all_articles)

    # Sort by publication state (peer-reviewed first) and then by date
    unique_articles.sort(
        key=lambda x: (
            0
            if x.get("publication_state", "peer_reviewed") == "peer_reviewed"
            else 1,
            x.get("date", "0000-00-00"),
        ),
        reverse=True,
    )

    # Get cBioPortal summary if genes are specified
    cbioportal_summary = await _get_cbioportal_summary(request)

    if unique_articles and not output_json:
        result = render.to_markdown(unique_articles)
        if cbioportal_summary:
            # Add cBioPortal summary at the beginning
            result = cbioportal_summary + "\n\n---\n\n" + result
        return result
    else:
        if cbioportal_summary:
            return json.dumps(
                {
                    "cbioportal_summary": cbioportal_summary,
                    "articles": unique_articles,
                },
                indent=2,
            )
        return json.dumps(unique_articles, indent=2)
