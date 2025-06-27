"""Getter module for retrieving variant details."""

import json
import logging
from typing import Annotated

from .. import ensure_list, http_client, render
from ..constants import MYVARIANT_GET_URL
from .external import ExternalVariantAggregator, format_enhanced_annotations
from .filters import filter_variants
from .links import inject_links

logger = logging.getLogger(__name__)


async def get_variant(
    variant_id: str,
    output_json: bool = False,
    include_external: bool = False,
) -> str:
    """
    Get variant details from MyVariant.info using the variant identifier.

    The identifier can be a full HGVS-style string (e.g. "chr7:g.140453136A>T")
    or an rsID (e.g. "rs113488022"). The API response is expected to include a
    "hits" array; this function extracts the first hit.

    If output_json is True, the result is returned as a formatted JSON string;
    otherwise, it is rendered as Markdown.
    """
    response, error = await http_client.request_api(
        url=f"{MYVARIANT_GET_URL}/{variant_id}",
        request={"fields": "all"},
        method="GET",
        domain="myvariant",
    )

    data_to_return: list = ensure_list(response)

    # Inject database links into the variant data
    if not error:
        data_to_return = inject_links(data_to_return)
        data_to_return = filter_variants(data_to_return)

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

    if error:
        data_to_return = [{"error": f"Error {error.code}: {error.message}"}]

    if output_json:
        return json.dumps(data_to_return, indent=2)
    else:
        return render.to_markdown(data_to_return)


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
) -> str:
    """
    Retrieves detailed information for a *single* genetic variant.

    Parameters:
    - call_benefit: Define and summarize why this function is being called and the intended benefit
    - variant_id: A variant identifier ("chr7:g.140453136A>T")
    - include_external: Include annotations from TCGA, 1000 Genomes, cBioPortal, and Mastermind

    Process: Queries the MyVariant.info GET endpoint, optionally fetching
            additional annotations from external databases
    Output: A Markdown formatted string containing comprehensive
            variant annotations (genomic context, frequencies,
            predictions, clinical data, external annotations). Returns error if invalid.
    Note: Use the variant_searcher to find the variant id first.
    """
    return await get_variant(
        variant_id, output_json=False, include_external=include_external
    )
