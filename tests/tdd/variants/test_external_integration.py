"""Integration tests for external variant data sources with real API calls."""

import pytest

from biomcp.variants.cbio_external_client import CBioPortalExternalClient
from biomcp.variants.external import (
    ExternalVariantAggregator,
    TCGAClient,
    ThousandGenomesClient,
)


class TestTCGAIntegration:
    """Integration tests for TCGA/GDC API."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_braf_v600e_variant(self):
        """Test fetching BRAF V600E data from TCGA."""
        client = TCGAClient()

        # Try different formats
        variants_to_test = [
            "BRAF V600E",  # Gene AA change format that TCGA supports
            "chr7:g.140453136A>T",
            "7:g.140453136A>T",
        ]

        found_data = False
        for variant in variants_to_test:
            result = await client.get_variant_data(variant)
            if result:
                found_data = True
                # BRAF V600E is common in melanoma and thyroid cancer
                assert result.tumor_types is not None
                assert len(result.tumor_types) > 0
                # Should have affected cases if data found
                if result.affected_cases:
                    assert result.affected_cases > 0
                break

        # Note: TCGA might not have data for all variants
        if not found_data:
            pytest.skip("TCGA API did not return data for BRAF V600E variants")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_tp53_variant(self):
        """Test fetching TP53 variant data from TCGA."""
        client = TCGAClient()

        # TP53 R273H - common tumor suppressor mutation
        result = await client.get_variant_data("chr17:g.7577120G>A")

        # TP53 mutations are very common in cancer
        if result:
            assert result.tumor_types is not None
            assert len(result.tumor_types) > 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_nonexistent_variant(self):
        """Test TCGA response for non-existent variant."""
        client = TCGAClient()

        # Made-up variant that shouldn't exist
        result = await client.get_variant_data("chr99:g.999999999A>T")

        assert result is None


class TestThousandGenomesIntegration:
    """Integration tests for 1000 Genomes via Ensembl REST API."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_common_variant_with_rsid(self):
        """Test fetching common variant data by rsID."""
        client = ThousandGenomesClient()

        # rs113488022 is BRAF V600E
        result = await client.get_variant_data("rs113488022")

        if result:
            # This is a rare variant, so MAF should be low or None
            if result.global_maf is not None:
                assert result.global_maf < 0.01  # Less than 1%

            # Consequence information might not be available for all variants
            # Just verify the data structure is correct
            assert hasattr(result, "most_severe_consequence")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_variant_population_frequencies(self):
        """Test population frequency data retrieval."""
        client = ThousandGenomesClient()

        # Use a more common variant for testing population frequencies
        # rs1800734 - common variant in MLH1 promoter
        result = await client.get_variant_data("rs1800734")

        if result:
            # Should have at least global MAF
            assert result.global_maf is not None
            assert 0 <= result.global_maf <= 1

            # Check that we get population-specific frequencies
            pop_freqs = [
                result.afr_maf,
                result.amr_maf,
                result.eas_maf,
                result.eur_maf,
                result.sas_maf,
            ]

            # At least some populations should have data
            non_null_freqs = [f for f in pop_freqs if f is not None]
            assert len(non_null_freqs) > 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_invalid_variant_id(self):
        """Test 1000 Genomes response for invalid variant."""
        client = ThousandGenomesClient()

        # Invalid rsID
        result = await client.get_variant_data("rs999999999999")

        assert result is None


class TestCBioPortalIntegration:
    """Integration tests for cBioPortal API."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_braf_v600e_variant(self):
        """Test fetching BRAF V600E data from cBioPortal."""
        client = CBioPortalExternalClient()

        result = await client.get_variant_data("BRAF V600E")

        if result:
            # BRAF V600E is common in melanoma and other cancers
            assert result.total_cases is not None
            assert result.total_cases > 0
            assert len(result.studies) > 0
            # Should have data from various studies
            print(
                f"Found {result.total_cases} cases in {len(result.studies)} studies: {result.studies}"
            )

            # Check enhanced fields
            assert result.cancer_type_distribution is not None
            assert len(result.cancer_type_distribution) > 0
            print(
                f"Cancer types: {list(result.cancer_type_distribution.keys())}"
            )

            assert result.mutation_types is not None
            assert "Missense_Mutation" in result.mutation_types

            assert result.mean_vaf is not None
            print(f"Mean VAF: {result.mean_vaf}")
        else:
            pytest.skip("cBioPortal API did not return data for BRAF V600E")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_kras_g12d_variant(self):
        """Test fetching KRAS G12D data from cBioPortal."""
        client = CBioPortalExternalClient()

        result = await client.get_variant_data("KRAS G12D")

        if result:
            # KRAS G12D is a common mutation in multiple cancer types
            assert result.total_cases is not None
            assert result.total_cases > 0
            assert len(result.studies) > 0
        else:
            pytest.skip("cBioPortal API did not return data for KRAS G12D")

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_invalid_variant(self):
        """Test cBioPortal response for invalid variant."""
        client = CBioPortalExternalClient()

        # Invalid gene name
        result = await client.get_variant_data("FAKEGENE V600E")

        assert result is None


class TestExternalVariantAggregatorIntegration:
    """Integration tests for the external variant aggregator."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_aggregate_all_sources(self):
        """Test aggregating data from all available sources."""
        aggregator = ExternalVariantAggregator()

        # Use rs1045642 which is a common variant that should have 1000 Genomes data
        # Also provide variant data for cBioPortal
        variant_data = {
            "cadd": {"gene": {"genename": "ABCB1"}},
            "docm": {"aa_change": "p.I1145I"},
        }

        result = await aggregator.get_enhanced_annotations(
            "rs1045642",
            include_tcga=True,
            include_1000g=True,
            include_cbioportal=True,
            variant_data=variant_data,
        )

        assert result.variant_id == "rs1045642"

        # Check which sources returned data
        sources_with_data = []
        if result.tcga:
            sources_with_data.append("tcga")
        if result.thousand_genomes:
            sources_with_data.append("1000g")
        if result.cbioportal:
            sources_with_data.append("cbioportal")

        # This common variant should have at least 1000 Genomes data
        assert len(sources_with_data) > 0
        # Specifically, it should have 1000 Genomes data
        assert result.thousand_genomes is not None

        # No errors should be reported for successful queries
        # (though some sources might not have data, which is different from errors)
        assert len(result.error_sources) == 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_selective_source_inclusion(self):
        """Test including only specific sources."""
        aggregator = ExternalVariantAggregator()

        # Only request 1000 Genomes data
        result = await aggregator.get_enhanced_annotations(
            "rs1800734",  # Common variant
            include_tcga=False,
            include_1000g=True,
        )

        # Should only attempt to fetch 1000 Genomes data
        assert result.tcga is None
        # 1000 Genomes might have data for this common variant
        # (but it's okay if it doesn't)

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_error_handling_resilience(self):
        """Test that aggregator handles individual source failures gracefully."""
        aggregator = ExternalVariantAggregator()

        # Use an invalid variant format that might cause errors
        result = await aggregator.get_enhanced_annotations(
            "INVALID_VARIANT_FORMAT_12345",
            include_tcga=True,
            include_1000g=True,
        )

        # Should still return a result even if all sources fail
        assert result is not None
        assert result.variant_id == "INVALID_VARIANT_FORMAT_12345"

        # Sources should return None or be in error_sources
        assert result.tcga is None
        assert result.thousand_genomes is None
