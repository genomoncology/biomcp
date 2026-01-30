"""
Integration tests for Enrichr API.

These tests make real API calls to Enrichr and may fail due to network issues.
They are marked with @pytest.mark.integration and are allowed to fail in CI.
"""

import pytest

from biomcp.enrichr import EnrichrClient


@pytest.mark.integration
class TestEnrichrIntegration:
    """Integration tests for Enrichr API."""

    @pytest.mark.asyncio
    async def test_submit_and_retrieve_real_api(self):
        """Test real API call to submit gene list and retrieve enrichment."""
        client = EnrichrClient()

        # Submit a well-known gene list
        genes = ["TP53", "BRCA1", "EGFR", "MYC", "KRAS"]
        user_list_id = await client.submit_gene_list(
            genes=genes, description="Integration test gene list"
        )

        assert user_list_id is not None
        assert isinstance(user_list_id, str)
        assert len(user_list_id) > 0

        # Get enrichment results for GO Biological Process
        results = await client.get_enrichment(
            user_list_id=user_list_id, database="GO_Biological_Process_2021"
        )

        assert results is not None
        assert isinstance(results, list)
        # Should have some enrichment results for these cancer genes
        assert len(results) > 0

        # Check structure of first result
        first_result = results[0]
        assert hasattr(first_result, "rank")
        assert hasattr(first_result, "path_name")
        assert hasattr(first_result, "p_val")
        assert hasattr(first_result, "z_score")
        assert hasattr(first_result, "combined_score")
        assert hasattr(first_result, "overlapping_genes")
        assert hasattr(first_result, "adj_p_val")
        assert hasattr(first_result, "database")

        # Validate data types
        assert isinstance(first_result.rank, int)
        assert isinstance(first_result.path_name, str)
        assert isinstance(first_result.p_val, int | float)
        assert isinstance(first_result.overlapping_genes, list)
        assert len(first_result.overlapping_genes) > 0

    @pytest.mark.asyncio
    async def test_enrich_kegg_pathway(self):
        """Test enrichment with KEGG pathway database."""
        client = EnrichrClient()

        # Use the convenience enrich() method
        results = await client.enrich(
            genes=["TP53", "BRCA1", "EGFR"],
            database="pathway",  # Friendly name
            description="KEGG pathway test",
        )

        assert results is not None
        assert isinstance(results, list)
        assert len(results) > 0

        # Should find cancer-related pathways
        first_result = results[0]
        assert first_result.database == "KEGG_2021_Human"
        assert len(first_result.overlapping_genes) > 0

    @pytest.mark.asyncio
    async def test_enrich_go_molecular_function(self):
        """Test enrichment with GO Molecular Function database."""
        client = EnrichrClient()

        results = await client.enrich(
            genes=["TP53", "MDM2", "ATM"],
            database="go_molecular",
            description="GO molecular function test",
        )

        assert results is not None
        assert isinstance(results, list)
        assert len(results) > 0
        assert results[0].database == "GO_Molecular_Function_2021"

    @pytest.mark.asyncio
    async def test_enrich_with_full_database_name(self):
        """Test enrichment using full database name."""
        client = EnrichrClient()

        results = await client.enrich(
            genes=["BRCA1", "BRCA2", "PALB2"],
            database="Reactome_2022",  # Full database name
            description="Reactome test",
        )

        assert results is not None
        assert isinstance(results, list)
        # May have results depending on the gene set
        # Just verify it doesn't error

    @pytest.mark.asyncio
    async def test_single_gene_enrichment(self):
        """Test enrichment with a single gene."""
        client = EnrichrClient()

        results = await client.enrich(
            genes=["TP53"], database="ontology", description="Single gene test"
        )

        # Single gene should still work
        assert results is not None
        assert isinstance(results, list)
        # TP53 is well-studied, should have enrichment terms
        assert len(results) > 0
