"""Integration tests for external variant data sources."""

import asyncio

import pytest

from biomcp.variants.external import (
    ExternalVariantAggregator,
    TCGAClient,
    ThousandGenomesClient,
)


class TestTCGAIntegration:
    """Integration tests for TCGA/GDC API."""

    @pytest.mark.asyncio
    async def test_tcga_real_variant(self):
        """Test real TCGA API with known variant."""
        client = TCGAClient()

        # Try with BRAF V600E - a well-known cancer mutation
        # TCGA can search by gene AA change format
        result = await client.get_variant_data("BRAF V600E")

        print(f"TCGA result: {result}")

        if result:
            print(f"COSMIC ID: {result.cosmic_id}")
            print(f"Tumor types: {result.tumor_types}")
            print(f"Affected cases: {result.affected_cases}")
            print(f"Consequence: {result.consequence_type}")
        else:
            print("No TCGA data found for this variant")


class TestThousandGenomesIntegration:
    """Integration tests for 1000 Genomes via Ensembl."""

    @pytest.mark.asyncio
    async def test_1000g_real_variant(self):
        """Test real 1000 Genomes API with known variant."""
        client = ThousandGenomesClient()

        # Try with a known rsID
        result = await client.get_variant_data("rs7412")  # APOE variant

        print(f"1000 Genomes result: {result}")

        if result:
            print(f"Global MAF: {result.global_maf}")
            print(f"EUR MAF: {result.eur_maf}")
            print(f"AFR MAF: {result.afr_maf}")
            print(f"Consequence: {result.most_severe_consequence}")
            print(f"Ancestral allele: {result.ancestral_allele}")

            # This variant should have frequency data
            assert result.global_maf is not None
        else:
            print("No 1000 Genomes data found")


class TestExternalAggregatorIntegration:
    """Integration tests for the aggregator."""

    @pytest.mark.asyncio
    async def test_aggregator_basic(self):
        """Test aggregator with basic functionality."""
        aggregator = ExternalVariantAggregator()

        # Test with a known variant
        result = await aggregator.get_enhanced_annotations(
            "rs7412",  # APOE variant
            include_tcga=True,
            include_1000g=True,
        )

        print(f"Variant ID: {result.variant_id}")
        print(f"TCGA data: {'Present' if result.tcga else 'Not found'}")
        print(
            f"1000G data: {'Present' if result.thousand_genomes else 'Not found'}"
        )
        print(f"Errors: {result.error_sources}")

        # Should still work
        assert result.variant_id == "rs7412"

    @pytest.mark.asyncio
    async def test_aggregator_partial_failures(self):
        """Test aggregator handles partial failures gracefully."""
        aggregator = ExternalVariantAggregator()

        # Use a variant that might not be in all databases
        result = await aggregator.get_enhanced_annotations(
            "chr1:g.12345678A>G",  # Arbitrary variant
            include_tcga=True,
            include_1000g=True,
        )

        print("Results for arbitrary variant:")
        print(f"- TCGA: {'Found' if result.tcga else 'Not found'}")
        print(
            f"- 1000G: {'Found' if result.thousand_genomes else 'Not found'}"
        )
        print(f"- Errors: {result.error_sources}")

        # Should complete without crashing
        assert result.variant_id == "chr1:g.12345678A>G"


if __name__ == "__main__":
    print("Testing TCGA/GDC...")
    asyncio.run(TestTCGAIntegration().test_tcga_real_variant())

    print("\n" + "=" * 50 + "\n")
    print("Testing 1000 Genomes...")
    asyncio.run(TestThousandGenomesIntegration().test_1000g_real_variant())

    print("\n" + "=" * 50 + "\n")
    print("Testing aggregator...")
    asyncio.run(TestExternalAggregatorIntegration().test_aggregator_basic())

    print("\n" + "=" * 50 + "\n")
    print("Testing aggregator with partial failures...")
    asyncio.run(
        TestExternalAggregatorIntegration().test_aggregator_partial_failures()
    )
