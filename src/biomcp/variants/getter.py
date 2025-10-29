"""Getter module for retrieving variant details."""

import inspect
import json
import logging
from typing import Annotated

from .. import ensure_list, http_client, render
from ..constants import DEFAULT_ASSEMBLY, MYVARIANT_GET_URL
from ..oncokb_helper import get_oncokb_annotation_for_variant
from .external import ExternalVariantAggregator, format_enhanced_annotations
from .filters import filter_variants
from .formatter import consolidate_multi_allelic_variants
from .links import inject_links

logger = logging.getLogger(__name__)


def _format_error_response(
    error: http_client.RequestError, variant_id: str
) -> list[dict[str, str]]:
    """Format error response for consistent rendering.

    Args:
        error: The error object from the API request
        variant_id: The variant identifier that was queried

    Returns:
        List containing error dict for markdown/json rendering
    """
    if error.code == 404:
        error_msg = (
            f"Variant '{variant_id}' not found in MyVariant.info database. "
            "Please verify the variant identifier (e.g., rs113488022 or "
            "chr7:g.140453136A>T)."
        )
    else:
        error_msg = f"Error {error.code}: {error.message}"

    return [{"error": error_msg}]


async def get_variant(  # noqa: C901
    variant_id: str,
    output_json: bool = False,
    include_external: bool = False,
    assembly: str = DEFAULT_ASSEMBLY,
    extensive: bool = False,
) -> str:
    """
    Get variant details from MyVariant.info using the variant identifier.

    The identifier can be a full HGVS-style string (e.g. "chr7:g.140453136A>T")
    or an rsID (e.g. "rs113488022"). The API response is expected to include a
    "hits" array; this function extracts the first hit.

    Args:
        variant_id: Variant identifier (HGVS or rsID)
        output_json: Return JSON format if True, else Markdown
        include_external: Include external annotations (TCGA, 1000 Genomes, cBioPortal)
        assembly: Genome assembly (hg19 or hg38), defaults to hg19
        extensive: Show full details (default: compact format for all variants)

    Returns:
        Formatted variant data as JSON or Markdown string

    If output_json is True, the result is returned as a formatted JSON string;
    otherwise, it is rendered as Markdown.
    """
    response, error = await http_client.request_api(
        url=f"{MYVARIANT_GET_URL}/{variant_id}",
        request={"fields": "all", "assembly": assembly},
        method="GET",
        domain="myvariant",
    )

    # Handle errors gracefully with user-friendly messages
    if error:
        data_to_return = _format_error_response(error, variant_id)
        # Skip all processing for error responses
        if output_json:
            return json.dumps(data_to_return, indent=2)
        else:
            return render.to_markdown(data_to_return)

    data_to_return = ensure_list(response)

    # Inject database links into the variant data
    data_to_return = inject_links(data_to_return)
    data_to_return = filter_variants(data_to_return)

    # Collect OncoKB annotations separately for markdown appendage
    oncokb_annotations: list[str] = []

    # Add external annotations if requested
    if include_external and data_to_return:
        logger.info(
            f"Adding external annotations for {len(data_to_return)} variants"
        )
        aggregator = ExternalVariantAggregator()

        for _i, variant_data in enumerate(data_to_return):
            logger.info(
                f"Processing variant {_i}: keys={list(variant_data.keys())}"
            )
            # Get enhanced annotations
            enhanced = await aggregator.get_enhanced_annotations(
                variant_id,
                include_tcga=True,
                include_1000g=True,
                include_cbioportal=True,
                variant_data=variant_data,
            )

            # Add formatted annotations to the variant data
            formatted = format_enhanced_annotations(enhanced)
            logger.info(
                f"Formatted external annotations: {formatted['external_annotations'].keys()}"
            )
            variant_data.update(formatted["external_annotations"])

            # Get formatted OncoKB annotation separately
            gene_aa = aggregator._extract_gene_aa_change(variant_data)
            # Handle case where method might be mocked as async in tests
            if inspect.iscoroutine(gene_aa) or inspect.isawaitable(gene_aa):
                gene_aa = await gene_aa
            if gene_aa:
                parts = gene_aa.split(" ", 1)
                if len(parts) == 2:
                    gene, variant = parts
                    oncokb_formatted = await get_oncokb_annotation_for_variant(
                        gene, variant
                    )
                    if oncokb_formatted:
                        oncokb_annotations.append(oncokb_formatted)

    if output_json:
        return json.dumps(data_to_return, indent=2)
    else:
        # Apply compact formatting unless extensive flag set
        if not extensive:
            logger.info(
                f"Applying compact format for {len(data_to_return)} variant(s)"
            )
            consolidated = consolidate_multi_allelic_variants(data_to_return)
            base_markdown = render.to_markdown(consolidated)
        else:
            # Render full markdown
            base_markdown = render.to_markdown(data_to_return)

        if oncokb_annotations:
            # Append OncoKB annotations as separate markdown sections
            return base_markdown + "\n" + "\n".join(oncokb_annotations)
        return base_markdown


async def _variant_details(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    variant_id: str,
    include_external: Annotated[
        bool,
        "Include annotations from external sources (TCGA, 1000 Genomes, cBioPortal)",
    ] = True,
    assembly: Annotated[
        str,
        "Genome assembly (hg19 or hg38). Default: hg19",
    ] = DEFAULT_ASSEMBLY,
    extensive: Annotated[
        bool,
        "Show full details for multi-allelic variants (default: compact format)",
    ] = False,
) -> str:
    """
    Retrieves detailed information for a *single* genetic variant.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - variant_id: A variant identifier ("chr7:g.140453136A>T")
    - include_external: Include annotations from TCGA, 1000 Genomes, cBioPortal, and Mastermind
    - assembly: Genome assembly (hg19 or hg38). Default: hg19
    - extensive: Show full details (default: compact format optimized for LLM consumption)

    Process: Queries the MyVariant.info GET endpoint, optionally fetching
            additional annotations from external databases. Returns compact format
            by default to reduce token usage (~90% reduction). Use extensive=True
            for full raw details.
    Output: A Markdown formatted string containing comprehensive
            variant annotations (genomic context, frequencies,
            predictions, clinical data, external annotations). Returns error if invalid.
    Note: Use the variant_searcher to find the variant id first.
    """
    return await get_variant(
        variant_id,
        output_json=False,
        include_external=include_external,
        assembly=assembly,
        extensive=extensive,
    )
