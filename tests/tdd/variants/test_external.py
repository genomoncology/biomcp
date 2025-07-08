"""Tests for external variant data sources."""

from unittest.mock import AsyncMock, patch

import pytest

from biomcp.variants.cbio_external_client import (
    CBioPortalExternalClient,
    CBioPortalVariantData,
)
from biomcp.variants.external import (
    EnhancedVariantAnnotation,
    ExternalVariantAggregator,
    TCGAClient,
    TCGAVariantData,
    ThousandGenomesClient,
    ThousandGenomesData,
    format_enhanced_annotations,
)


class TestTCGAClient:
    """Tests for TCGA/GDC client."""

    @pytest.mark.asyncio
    async def test_get_variant_data_success(self):
        """Test successful TCGA variant data retrieval."""
        client = TCGAClient()

        mock_response = {
            "data": {
                "hits": [
                    {
                        "ssm_id": "test-ssm-id",
                        "cosmic_id": ["COSM476"],
                        "gene_aa_change": ["BRAF V600E"],
                        "genomic_dna_change": "chr7:g.140453136A>T",
                    }
                ]
            }
        }

        mock_occ_response = {
            "data": {
                "hits": [
                    {"case": {"project": {"project_id": "TCGA-LUAD"}}},
                    {"case": {"project": {"project_id": "TCGA-LUAD"}}},
                    {"case": {"project": {"project_id": "TCGA-LUSC"}}},
                ]
            }
        }

        with patch("biomcp.http_client.request_api") as mock_request:
            # First call is for SSM search, second is for occurrences
            mock_request.side_effect = [
                (mock_response, None),
                (mock_occ_response, None),
            ]

            result = await client.get_variant_data("BRAF V600E")

            assert result is not None
            assert result.cosmic_id == "COSM476"
            assert "LUAD" in result.tumor_types
            assert "LUSC" in result.tumor_types
            assert result.affected_cases == 3
            assert result.consequence_type == "missense_variant"

    @pytest.mark.asyncio
    async def test_get_variant_data_not_found(self):
        """Test TCGA variant data when not found."""
        client = TCGAClient()

        mock_response = {"data": {"hits": []}}

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await client.get_variant_data("chr7:g.140453136A>T")

            assert result is None


class TestThousandGenomesClient:
    """Tests for 1000 Genomes client."""

    @pytest.mark.asyncio
    async def test_get_variant_data_success(self):
        """Test successful 1000 Genomes data retrieval."""
        client = ThousandGenomesClient()

        mock_response = {
            "populations": [
                {"population": "1000GENOMES:phase_3:ALL", "frequency": 0.05},
                {"population": "1000GENOMES:phase_3:EUR", "frequency": 0.08},
                {"population": "1000GENOMES:phase_3:EAS", "frequency": 0.02},
            ],
            "mappings": [
                {
                    "transcript_consequences": [
                        {"consequence_terms": ["missense_variant"]}
                    ]
                }
            ],
            "ancestral_allele": "A",
        }

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await client.get_variant_data("rs113488022")

            assert result is not None
            assert result.global_maf == 0.05
            assert result.eur_maf == 0.08
            assert result.eas_maf == 0.02
            assert result.most_severe_consequence == "missense_variant"
            assert result.ancestral_allele == "A"

    def test_extract_population_frequencies(self):
        """Test population frequency extraction."""
        client = ThousandGenomesClient()

        populations = [
            {"population": "1000GENOMES:phase_3:ALL", "frequency": 0.05},
            {"population": "1000GENOMES:phase_3:AFR", "frequency": 0.10},
            {"population": "1000GENOMES:phase_3:AMR", "frequency": 0.07},
            {"population": "1000GENOMES:phase_3:EAS", "frequency": 0.02},
            {"population": "1000GENOMES:phase_3:EUR", "frequency": 0.08},
            {"population": "1000GENOMES:phase_3:SAS", "frequency": 0.06},
            {
                "population": "OTHER:population",
                "frequency": 0.99,
            },  # Should be ignored
        ]

        result = client._extract_population_frequencies(populations)

        assert result["global_maf"] == 0.05
        assert result["afr_maf"] == 0.10
        assert result["amr_maf"] == 0.07
        assert result["eas_maf"] == 0.02
        assert result["eur_maf"] == 0.08
        assert result["sas_maf"] == 0.06
        assert "OTHER" not in str(result)


class TestCBioPortalExternalClient:
    """Tests for cBioPortal client."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_get_variant_data_success(self):
        """Test successful cBioPortal variant data retrieval using real API."""
        client = CBioPortalExternalClient()

        # Test with a known variant
        result = await client.get_variant_data("BRAF V600E")

        assert result is not None
        assert result.total_cases > 0
        assert len(result.studies) > 0
        assert "Missense_Mutation" in result.mutation_types
        assert result.mutation_types["Missense_Mutation"] > 0
        assert result.mean_vaf is not None
        assert result.mean_vaf > 0.0
        assert result.mean_vaf < 1.0

        # Check cancer type distribution
        assert len(result.cancer_type_distribution) > 0
        # BRAF V600E is common in melanoma and colorectal
        cancer_types = list(result.cancer_type_distribution.keys())
        assert any(
            "glioma" in ct.lower()
            or "lung" in ct.lower()
            or "colorectal" in ct.lower()
            for ct in cancer_types
        )

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_get_variant_data_not_found(self):
        """Test cBioPortal variant data when not found using real API."""
        client = CBioPortalExternalClient()

        # Test with a variant that's extremely rare or doesn't exist
        result = await client.get_variant_data("BRAF X999Z")

        # Should return None for non-existent variants
        assert result is None

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_get_variant_data_invalid_format(self):
        """Test cBioPortal with invalid gene/AA format."""
        client = CBioPortalExternalClient()

        result = await client.get_variant_data("InvalidFormat")

        assert result is None

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_get_variant_data_gene_not_found(self):
        """Test cBioPortal when gene is not found."""
        client = CBioPortalExternalClient()

        # Test with a non-existent gene
        result = await client.get_variant_data("FAKEGENE123 V600E")

        assert result is None


class TestExternalVariantAggregator:
    """Tests for external variant aggregator."""

    @pytest.mark.asyncio
    async def test_get_enhanced_annotations_all_sources(self):
        """Test aggregating data from all sources."""
        aggregator = ExternalVariantAggregator()

        # Mock all clients
        mock_tcga_data = TCGAVariantData(
            cosmic_id="COSM476", tumor_types=["LUAD"], affected_cases=10
        )

        mock_1000g_data = ThousandGenomesData(global_maf=0.05, eur_maf=0.08)

        mock_cbio_data = CBioPortalVariantData(
            total_cases=42, studies=["tcga_pan_can_atlas_2018"]
        )

        aggregator.tcga_client.get_variant_data = AsyncMock(
            return_value=mock_tcga_data
        )
        aggregator.thousand_genomes_client.get_variant_data = AsyncMock(
            return_value=mock_1000g_data
        )
        aggregator.cbioportal_client.get_variant_data = AsyncMock(
            return_value=mock_cbio_data
        )

        # Mock variant data to extract gene/AA change
        variant_data = {
            "cadd": {"gene": {"genename": "BRAF"}},
            "docm": {"aa_change": "p.V600E"},
        }

        result = await aggregator.get_enhanced_annotations(
            "chr7:g.140453136A>T", variant_data=variant_data
        )

        assert result.variant_id == "chr7:g.140453136A>T"
        assert result.tcga is not None
        assert result.tcga.cosmic_id == "COSM476"
        assert result.thousand_genomes is not None
        assert result.thousand_genomes.global_maf == 0.05
        assert result.cbioportal is not None
        assert result.cbioportal.total_cases == 42
        assert "tcga_pan_can_atlas_2018" in result.cbioportal.studies

    @pytest.mark.asyncio
    async def test_get_enhanced_annotations_with_errors(self):
        """Test aggregation when some sources fail."""
        aggregator = ExternalVariantAggregator()

        # Mock TCGA to succeed
        mock_tcga_data = TCGAVariantData(cosmic_id="COSM476")
        aggregator.tcga_client.get_variant_data = AsyncMock(
            return_value=mock_tcga_data
        )

        # Mock 1000G to fail
        aggregator.thousand_genomes_client.get_variant_data = AsyncMock(
            side_effect=Exception("Network error")
        )

        result = await aggregator.get_enhanced_annotations(
            "chr7:g.140453136A>T", include_tcga=True, include_1000g=True
        )

        assert result.tcga is not None
        assert result.thousand_genomes is None
        assert "thousand_genomes" in result.error_sources


class TestFormatEnhancedAnnotations:
    """Tests for formatting enhanced annotations."""

    def test_format_all_annotations(self):
        """Test formatting when all annotations are present."""
        annotation = EnhancedVariantAnnotation(
            variant_id="chr7:g.140453136A>T",
            tcga=TCGAVariantData(
                cosmic_id="COSM476",
                tumor_types=["LUAD", "LUSC"],
                affected_cases=10,
            ),
            thousand_genomes=ThousandGenomesData(
                global_maf=0.05, eur_maf=0.08, ancestral_allele="A"
            ),
            cbioportal=CBioPortalVariantData(
                total_cases=42,
                studies=["tcga_pan_can_atlas_2018", "msk_impact_2017"],
                cancer_type_distribution={
                    "Melanoma": 30,
                    "Thyroid Cancer": 12,
                },
                mutation_types={
                    "Missense_Mutation": 40,
                    "Nonsense_Mutation": 2,
                },
                hotspot_count=35,
                mean_vaf=0.285,
                sample_types={"Primary": 25, "Metastatic": 17},
            ),
        )

        result = format_enhanced_annotations(annotation)

        assert result["variant_id"] == "chr7:g.140453136A>T"
        assert "tcga" in result["external_annotations"]
        assert result["external_annotations"]["tcga"]["cosmic_id"] == "COSM476"
        assert "1000_genomes" in result["external_annotations"]
        assert (
            result["external_annotations"]["1000_genomes"]["global_maf"]
            == 0.05
        )
        assert "cbioportal" in result["external_annotations"]
        cbio = result["external_annotations"]["cbioportal"]
        assert cbio["total_cases"] == 42
        assert "tcga_pan_can_atlas_2018" in cbio["studies"]
        assert cbio["cancer_types"]["Melanoma"] == 30
        assert cbio["mutation_types"]["Missense_Mutation"] == 40
        assert cbio["hotspot_samples"] == 35
        assert cbio["mean_vaf"] == 0.285
        assert cbio["sample_types"]["Primary"] == 25

    def test_format_partial_annotations(self):
        """Test formatting when only some annotations are present."""
        annotation = EnhancedVariantAnnotation(
            variant_id="chr7:g.140453136A>T",
            tcga=TCGAVariantData(cosmic_id="COSM476"),
            error_sources=["thousand_genomes"],
        )

        result = format_enhanced_annotations(annotation)

        assert "tcga" in result["external_annotations"]
        assert "1000_genomes" not in result["external_annotations"]
        assert "errors" in result["external_annotations"]
        assert "thousand_genomes" in result["external_annotations"]["errors"]
