"""
Enrichr API client for functional enrichment analysis.

Inspired by gget enrichr (https://github.com/pachterlab/gget).
Citation: Luebbert & Pachter (2023). Bioinformatics, 39(1), btac836.
BioMCP directly integrates with Enrichr API rather than using gget as a dependency.

API Documentation: https://maayanlab.cloud/Enrichr/help#api
"""

import logging

from pydantic import BaseModel, Field

from .. import http_client
from .databases import get_database_name

logger = logging.getLogger(__name__)

# Enrichr API endpoints
ENRICHR_BASE_URL = "https://maayanlab.cloud/Enrichr"
ENRICHR_ADDLIST_URL = f"{ENRICHR_BASE_URL}/addList"
ENRICHR_ENRICH_URL = f"{ENRICHR_BASE_URL}/enrich"


class EnrichmentTerm(BaseModel):
    """
    A single enrichment term result from Enrichr.

    Based on the example format provided in requirements.
    """

    rank: int
    path_name: str = Field(description="Pathway or term name with ID")
    p_val: float = Field(description="P-value")
    z_score: float = Field(description="Z-score")
    combined_score: float = Field(description="Combined score")
    overlapping_genes: list[str] = Field(
        description="Genes from input that overlap with this term"
    )
    adj_p_val: float = Field(description="Adjusted p-value (FDR)")
    database: str = Field(description="Enrichr database name")


class EnrichrClient:
    """Client for Enrichr functional enrichment analysis API."""

    def __init__(self):
        """Initialize the Enrichr client."""
        self.logger = logger

    async def submit_gene_list(
        self, genes: list[str] | str, description: str = "BioMCP gene list"
    ) -> str | None:
        """
        Submit a gene list to Enrichr and get a user_list_id.

        Args:
            genes: List of gene symbols or newline-separated string
            description: Description for the gene list

        Returns:
            user_list_id string if successful, None otherwise
        """
        try:
            # Convert list to newline-separated string if needed
            if isinstance(genes, list):
                gene_string = "\n".join(genes)
            else:
                gene_string = genes

            # Enrichr expects multipart/form-data
            # Following gget's approach: https://github.com/pachterlab/gget
            # Use centralized HTTP client with _files parameter
            files = {
                "list": (None, gene_string),
                "description": (None, description),
            }

            # Make request using centralized HTTP client
            response, error = await http_client.request_api(
                url=ENRICHR_ADDLIST_URL,
                request={"_files": files},
                method="POST",
                domain="enrichr",
                cache_ttl=0,  # Don't cache submissions
            )

            if error or not response:
                self.logger.warning(
                    f"Failed to submit gene list to Enrichr: {error}"
                )
                return None

            # Response format: {"userListId": 123456, "shortId": "abc"}
            # response is a dict when no response_model_type is provided
            user_list_id = (
                response.get("userListId")
                if isinstance(response, dict)
                else None
            )
            if user_list_id is None:
                self.logger.warning(
                    f"No userListId in Enrichr response: {response}"
                )
                return None

            return str(user_list_id)

        except Exception as e:
            self.logger.error(f"Error submitting gene list to Enrichr: {e}")
            return None

    async def get_enrichment(
        self, user_list_id: str, database: str
    ) -> list[EnrichmentTerm] | None:
        """
        Get enrichment results for a submitted gene list.

        Args:
            user_list_id: The ID returned from submit_gene_list
            database: Enrichr database name (e.g., "KEGG_2021_Human")

        Returns:
            List of EnrichmentTerm objects, or None if request failed
        """
        try:
            # Enrichr enrich endpoint uses query parameters
            params = {"userListId": user_list_id, "backgroundType": database}

            response, error = await http_client.request_api(
                url=ENRICHR_ENRICH_URL,
                request=params,
                method="GET",
                domain="enrichr",
            )

            if error or not response:
                self.logger.warning(
                    f"Failed to get enrichment from Enrichr for database {database}: {error}"
                )
                return None

            # Response format is a dictionary with the database name as key
            # e.g., {"KEGG_2021_Human": [[rank, term, p_val, z_score, combined_score, genes, adj_p_val], ...]}
            enrichment_data = response.get(database)
            if not enrichment_data:
                self.logger.warning(
                    f"No enrichment data for database {database} in response"
                )
                return None

            # Parse enrichment results
            terms = []
            for i, result in enumerate(enrichment_data):
                try:
                    # Each result is an array:
                    # [rank, term_name, p_val, z_score, combined_score, genes_str, adj_p_val, ...]
                    # genes_str is semicolon-separated
                    if len(result) < 7:
                        continue

                    rank = result[0]
                    term_name = result[1]
                    p_val = result[2]
                    z_score = result[3]
                    combined_score = result[4]
                    genes_str = result[5]
                    adj_p_val = result[6]

                    # Parse overlapping genes (can be semicolon-separated string or list)
                    if isinstance(genes_str, list):
                        overlapping_genes = genes_str
                    elif isinstance(genes_str, str):
                        overlapping_genes = [
                            g.strip()
                            for g in genes_str.split(";")
                            if g.strip()
                        ]
                    else:
                        overlapping_genes = []

                    term = EnrichmentTerm(
                        rank=rank,
                        path_name=term_name,
                        p_val=p_val,
                        z_score=z_score,
                        combined_score=combined_score,
                        overlapping_genes=overlapping_genes,
                        adj_p_val=adj_p_val,
                        database=database,
                    )
                    terms.append(term)

                except (IndexError, ValueError, TypeError) as e:
                    self.logger.warning(
                        f"Failed to parse enrichment result {i}: {e}"
                    )
                    continue

            return terms

        except Exception as e:
            self.logger.error(f"Error getting enrichment from Enrichr: {e}")
            return None

    async def enrich(
        self,
        genes: list[str] | str,
        database: str = "pathway",
        description: str = "BioMCP gene list",
    ) -> list[EnrichmentTerm] | None:
        """
        Perform enrichment analysis in one step.

        This is a convenience method that combines submit_gene_list and get_enrichment.

        Args:
            genes: List of gene symbols or newline-separated string
            database: Database category (e.g., "pathway", "ontology") or full name
            description: Description for the gene list

        Returns:
            List of EnrichmentTerm objects, or None if request failed
        """
        # Convert database category to full name
        try:
            db_name = get_database_name(database)
        except ValueError as e:
            self.logger.error(f"Invalid database: {e}")
            return None

        # Submit gene list
        user_list_id = await self.submit_gene_list(genes, description)
        if not user_list_id:
            return None

        # Get enrichment results
        return await self.get_enrichment(user_list_id, db_name)
