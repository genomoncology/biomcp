"""Helper module for cBioPortal integration across tools.

This module centralizes cBioPortal summary generation logic to avoid duplication.
"""

import logging

logger = logging.getLogger(__name__)


async def get_cbioportal_summary_for_genes(
    genes: list[str] | None, request_params: dict | None = None
) -> str | None:
    """Get cBioPortal summary for given genes.

    Args:
        genes: List of gene symbols to get summaries for
        request_params: Optional additional parameters for the request

    Returns:
        Formatted cBioPortal summary or None if unavailable
    """
    if not genes:
        return None

    try:
        from biomcp.articles.search import PubmedRequest
        from biomcp.articles.unified import _get_cbioportal_summary

        # Create a request object for cBioPortal summary
        request = PubmedRequest(genes=genes)

        # Add any additional parameters if provided
        if request_params:
            for key, value in request_params.items():
                if hasattr(request, key):
                    setattr(request, key, value)

        cbioportal_summary = await _get_cbioportal_summary(request)
        return cbioportal_summary

    except Exception as e:
        logger.warning(f"Failed to get cBioPortal summary: {e}")
        return None


async def get_variant_cbioportal_summary(gene: str | None) -> str | None:
    """Get cBioPortal summary for variant searches.

    Args:
        gene: Gene symbol to get summary for

    Returns:
        Formatted cBioPortal summary or None if unavailable
    """
    if not gene:
        return None

    try:
        from biomcp.variants.cbioportal_search import (
            CBioPortalSearchClient,
            format_cbioportal_search_summary,
        )

        client = CBioPortalSearchClient()
        summary = await client.get_gene_search_summary(gene)
        if summary:
            return format_cbioportal_search_summary(summary)
        return None

    except Exception as e:
        logger.warning(
            f"Failed to get cBioPortal summary for variant search: {e}"
        )
        return None
