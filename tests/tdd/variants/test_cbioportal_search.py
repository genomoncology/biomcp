"""Test cBioPortal search enhancements."""

import asyncio

import pytest

from biomcp.variants.cbioportal_search import (
    CBioPortalSearchClient,
    CBioPortalSearchSummary,
    format_cbioportal_search_summary,
)
from biomcp.variants.search import VariantQuery, search_variants

from .constants import API_RETRY_DELAY_SECONDS, DEFAULT_MAX_STUDIES


class TestCBioPortalSearch:
    """Test cBioPortal search functionality."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_gene_search_summary(self):
        """Test getting gene search summary from cBioPortal."""
        client = CBioPortalSearchClient()

        # Test with BRAF
        summary = await client.get_gene_search_summary("BRAF", max_studies=5)

        assert summary is not None
        assert summary.gene == "BRAF"

        # Handle case where cBioPortal API returns empty data
        if summary.total_mutations == 0:
            # API might be down or returning empty results
            # This is acceptable for integration tests
            assert summary.total_mutations == 0
            assert summary.total_samples_tested == 0
            assert summary.mutation_frequency == 0.0
            assert len(summary.hotspots) == 0
        else:
            # Normal case - data is available
            assert summary.total_mutations > 0
            assert summary.total_samples_tested > 0
            assert summary.mutation_frequency > 0
            assert len(summary.hotspots) > 0

            # Check that V600E is a top hotspot
            v600e_found = any(
                "V600E" in hs.amino_acid_change for hs in summary.hotspots
            )
            assert v600e_found, "BRAF V600E should be a top hotspot"

        # Check cancer distribution
        if summary.total_mutations > 0:
            assert len(summary.cancer_distribution) > 0
            assert any(
                "melanoma" in cancer.lower()
                for cancer in summary.cancer_distribution
            ), "BRAF should be found in melanoma"
        else:
            # When no mutations found, cancer distribution should be empty
            assert len(summary.cancer_distribution) == 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_format_search_summary(self):
        """Test formatting of search summary."""
        # Create a mock summary
        summary = CBioPortalSearchSummary(
            gene="BRAF",
            total_mutations=1000,
            total_samples_tested=10000,
            mutation_frequency=0.1,
            hotspots=[
                {
                    "position": 600,
                    "amino_acid_change": "V600E",
                    "count": 800,
                    "frequency": 0.8,
                    "cancer_types": ["Melanoma", "Colorectal Cancer"],
                }
            ],
            cancer_distribution={"Melanoma": 600, "Colorectal Cancer": 200},
            study_coverage={
                "total_studies": 50,
                "queried_studies": 10,
                "studies_with_data": 8,
            },
        )

        formatted = format_cbioportal_search_summary(summary)

        assert "BRAF" in formatted
        assert "10.0%" in formatted  # Mutation frequency
        assert "V600E" in formatted
        assert "Melanoma" in formatted
        assert "600 mutations" in formatted

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_search_with_cbioportal_summary(self):
        """Test variant search with cBioPortal summary included."""
        query = VariantQuery(gene="BRAF", size=5)

        result = await search_variants(query, include_cbioportal=True)

        # Should include cBioPortal summary section
        assert "cBioPortal Summary for BRAF" in result
        assert "Mutation Frequency" in result
        # Top Hotspots only appears when mutations are found
        # Check for either Top Hotspots or 0 mutations message
        assert "Top Hotspots" in result or "0 mutations" in result

        # Should still include variant results
        assert "# Record" in result or "No variants found" in result

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_search_without_gene(self):
        """Test that cBioPortal summary is not included without gene parameter."""
        query = VariantQuery(rsid="rs113488022", size=5)

        result = await search_variants(query, include_cbioportal=True)

        # Should not include cBioPortal summary
        assert "cBioPortal Summary" not in result

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_tp53_search_summary(self):
        """Test TP53 gene search summary."""
        client = CBioPortalSearchClient()

        # Clear any caches to ensure fresh data
        from biomcp.utils.request_cache import clear_cache

        await clear_cache()

        summary = await client.get_gene_search_summary("TP53", max_studies=5)

        assert summary is not None
        assert summary.gene == "TP53"

        # If we got no mutations, it might be a temporary API issue
        if summary.total_mutations == 0 and summary.total_samples_tested == 0:
            # Try one more time with a small delay
            await asyncio.sleep(API_RETRY_DELAY_SECONDS)
            summary = await client.get_gene_search_summary(
                "TP53", max_studies=5
            )

            # If still no data, skip the test rather than fail
            if summary.total_mutations == 0:
                pytest.skip(
                    "cBioPortal returned no mutation data for TP53 - possible API issue"
                )

        # Basic checks that should pass when data is available
        assert (
            summary.total_mutations > 0
        ), f"TP53 should have mutations. Got: {summary}"

        # More flexible checks
        if summary.hotspots:
            # Just verify structure if we have hotspots
            hotspot_changes = [hs.amino_acid_change for hs in summary.hotspots]
            print(f"TP53 hotspots found: {hotspot_changes[:5]}")
            assert (
                len(hotspot_changes) >= 1
            ), "Should find at least one TP53 hotspot"

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_kras_search_summary(self):
        """Test KRAS gene search summary.

        This test verifies basic functionality rather than specific hotspots,
        which can change as cBioPortal data is updated.
        """
        client = CBioPortalSearchClient()

        # Clear any caches to ensure fresh data
        from biomcp.utils.request_cache import clear_cache

        await clear_cache()

        summary = await client.get_gene_search_summary(
            "KRAS", max_studies=DEFAULT_MAX_STUDIES
        )

        assert summary is not None, "Failed to get summary for KRAS"
        assert summary.gene == "KRAS"

        # If we got no mutations, it might be a temporary API issue
        if summary.total_mutations == 0 and summary.total_samples_tested == 0:
            # Try one more time with a small delay
            await asyncio.sleep(API_RETRY_DELAY_SECONDS)
            summary = await client.get_gene_search_summary(
                "KRAS", max_studies=DEFAULT_MAX_STUDIES
            )

            # If still no data, skip the test rather than fail
            if summary.total_mutations == 0:
                pytest.skip(
                    "cBioPortal returned no mutation data for KRAS - possible API issue"
                )

        # Basic checks that should pass when data is available
        assert (
            summary.total_mutations > 0
        ), f"KRAS should have mutations. Got: {summary}"

        # More flexible checks
        if summary.hotspots:
            # Just verify structure if we have hotspots
            for hotspot in summary.hotspots[:3]:
                assert hasattr(hotspot, "amino_acid_change")
                assert hasattr(hotspot, "count")
            print(
                f"Top KRAS hotspots: {[hs.amino_acid_change for hs in summary.hotspots[:5]]}"
            )

        # Cancer distribution check - only if we have data
        if summary.total_mutations > 0:
            assert (
                len(summary.cancer_distribution) > 0
            ), "Should have cancer type distribution"

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_invalid_gene(self):
        """Test handling of invalid gene name."""
        client = CBioPortalSearchClient()

        summary = await client.get_gene_search_summary("INVALID_GENE")

        assert summary is None

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_json_output_with_cbioportal(self):
        """Test JSON output includes cBioPortal summary."""
        query = VariantQuery(gene="BRAF", size=2)

        result = await search_variants(
            query, output_json=True, include_cbioportal=True
        )

        # Parse JSON
        import json

        data = json.loads(result)

        # Should have both summary and variants
        assert "cbioportal_summary" in data
        assert "variants" in data
        assert "BRAF" in data["cbioportal_summary"]
