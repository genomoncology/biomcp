"""
Gene information retrieval from MyGene.info.

Enrichment functionality inspired by gget enrichr (https://github.com/pachterlab/gget).
Citation: Luebbert & Pachter (2023). Bioinformatics, 39(1), btac836.
BioMCP directly integrates with Enrichr API rather than using gget as a dependency.
"""

import json
import logging
from typing import Annotated

from pydantic import Field

from ..enrichr import EnrichrClient
from ..integrations import BioThingsClient
from ..render import to_markdown

logger = logging.getLogger(__name__)


async def get_gene(
    gene_id_or_symbol: str,
    output_json: bool = False,
    include_enrichment: bool = False,
    enrichment_database: str = "GO_Biological_Process_2021",
) -> str:
    """
    Get gene information from MyGene.info with optional enrichment analysis.

    Args:
        gene_id_or_symbol: Gene ID (Entrez, Ensembl) or symbol (e.g., "TP53", "7157")
        output_json: Return as JSON instead of markdown
        include_enrichment: Whether to include Enrichr functional enrichment
        enrichment_database: Enrichr database to use (default: GO_Biological_Process_2021)

    Returns:
        Gene information as markdown or JSON string
    """
    client = BioThingsClient()

    try:
        gene_info = await client.get_gene_info(gene_id_or_symbol)

        if not gene_info:
            error_data = {
                "error": f"Gene '{gene_id_or_symbol}' not found",
                "suggestion": "Please check the gene symbol or ID",
            }
            return (
                json.dumps(error_data, indent=2)
                if output_json
                else to_markdown([error_data])
            )

        # Convert to dict for rendering
        result = gene_info.model_dump(exclude_none=True)

        # Add helpful links
        if gene_info.entrezgene:
            result["_links"] = {
                "NCBI Gene": f"https://www.ncbi.nlm.nih.gov/gene/{gene_info.entrezgene}",
                "PubMed": f"https://pubmed.ncbi.nlm.nih.gov/?term={gene_info.symbol}",
            }

        # Format aliases nicely
        if gene_info.alias:
            result["alias"] = ", ".join(
                gene_info.alias[:10]
            )  # Limit to first 10
            if len(gene_info.alias) > 10:
                result["alias"] += f" (and {len(gene_info.alias) - 10} more)"

        # Add enrichment if requested
        if include_enrichment and gene_info.symbol:
            try:
                enrichr_client = EnrichrClient()
                enrichment_results = await enrichr_client.enrich(
                    genes=[gene_info.symbol],
                    database=enrichment_database,
                    description=f"Enrichment for {gene_info.symbol}",
                )

                if enrichment_results:
                    # Convert enrichment terms to dicts and limit to top 10
                    result["enrichment"] = {
                        "database": enrichment_database,
                        "terms": [
                            term.model_dump()
                            for term in enrichment_results[:10]
                        ],
                    }
                else:
                    result["enrichment"] = {
                        "database": enrichment_database,
                        "terms": [],
                        "note": "No significant enrichment terms found",
                    }
            except Exception as e:
                logger.warning(
                    f"Failed to get enrichment for {gene_info.symbol}: {e}"
                )
                # Don't fail the entire request if enrichment fails
                result["enrichment"] = {
                    "error": f"Enrichment analysis failed: {e!s}"
                }

        if output_json:
            return json.dumps(result, indent=2)
        else:
            return to_markdown([result])

    except Exception as e:
        logger.error(f"Error fetching gene info for {gene_id_or_symbol}: {e}")
        error_data = {
            "error": "Failed to retrieve gene information",
            "details": str(e),
        }
        return (
            json.dumps(error_data, indent=2)
            if output_json
            else to_markdown([error_data])
        )


async def _gene_details(
    call_benefit: Annotated[
        str,
        "Define and summarize why this function is being called and the intended benefit",
    ],
    gene_id_or_symbol: Annotated[
        str,
        Field(description="Gene symbol (e.g., TP53, BRAF) or ID (e.g., 7157)"),
    ],
) -> str:
    """
    Retrieves detailed information for a single gene from MyGene.info.

    This tool provides real-time gene annotations including:
    - Official gene name and symbol
    - Gene summary/description
    - Aliases and alternative names
    - Gene type (protein-coding, etc.)
    - Links to external databases

    Parameters:
    - call_benefit: Define why this function is being called
    - gene_id_or_symbol: Gene symbol (e.g., "TP53") or Entrez ID (e.g., "7157")

    Process: Queries MyGene.info API for up-to-date gene annotations
    Output: Markdown formatted gene information with description and metadata

    Note: For variant information, use variant_searcher. For articles about genes, use article_searcher.
    """
    return await get_gene(gene_id_or_symbol, output_json=False)
