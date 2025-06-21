"""Tests for external variant data sources."""

from unittest.mock import AsyncMock, patch

import pytest

from biomcp.variants.external import (
    CBioPortalClient,
    CBioPortalVariantData,
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


class TestCBioPortalClient:
    """Tests for cBioPortal client."""

    @pytest.mark.asyncio
    async def test_get_variant_data_success(self):
        """Test successful cBioPortal variant data retrieval."""
        # Mock gene lookup response
        mock_gene_response = {
            "entrezGeneId": 673,
            "hugoGeneSymbol": "BRAF",
        }

        # Mock mutations response
        mock_mutations_response = [
            {
                "sampleId": "TCGA-AA-1234",
                "proteinChange": "V600E",
                "mutationType": "Missense_Mutation",
                "studyId": "coadread_mskcc",
                "keyword": "BRAF V600 hotspot",
                "tumorAltCount": 50,
                "tumorRefCount": 150,
            },
            {
                "sampleId": "TCGA-AA-5678",
                "proteinChange": "V600E",
                "mutationType": "Missense_Mutation",
                "studyId": "coadread_mskcc",
                "tumorAltCount": 30,
                "tumorRefCount": 170,
            },
            {
                "sampleId": "TCGA-AA-9012",
                "proteinChange": "V600E",
                "mutationType": "Missense_Mutation",
                "studyId": "coadread_mskcc",
            },
        ]

        with (
            patch(
                "biomcp.variants.external.RetryableHTTPClient"
            ) as mock_retry_class,
            patch(
                "biomcp.variants.external.httpx.AsyncClient"
            ) as mock_client_class,
        ):
            # Mock retry client
            mock_retry = mock_retry_class.return_value
            mock_retry.get_with_retry = AsyncMock()

            # Create mock client instance
            mock_client = AsyncMock()
            mock_client_class.return_value.__aenter__.return_value = (
                mock_client
            )

            # Create client after mocks are set up
            client = CBioPortalClient()
            client._study_cache.clear()

            # Mock gene lookup response
            gene_resp = AsyncMock()
            gene_resp.status_code = 200
            gene_resp.json = lambda: mock_gene_response

            # Mock molecular profiles response
            profiles_resp = AsyncMock()
            profiles_resp.status_code = 200
            profiles_resp.json = lambda: [
                {
                    "molecularProfileId": "coadread_mskcc_mutations",
                    "studyId": "coadread_mskcc",
                    "molecularAlterationType": "MUTATION_EXTENDED",
                }
            ]

            # Mock study metadata response
            study_resp = AsyncMock()
            study_resp.status_code = 200
            study_resp.json = lambda: {
                "studyId": "coadread_mskcc",
                "cancerType": {"name": "Colorectal Adenocarcinoma"},
            }

            # Mock mutations response
            mutations_resp = AsyncMock()
            mutations_resp.status_code = 200
            mutations_resp.json = lambda: mock_mutations_response

            # Mock the retry client to return responses in order
            mock_retry.get_with_retry.side_effect = [
                gene_resp,  # gene lookup
                profiles_resp,  # profiles lookup (parallel with gene)
                study_resp,  # study metadata
                mutations_resp,  # mutations
            ]

            result = await client.get_variant_data("BRAF V600E")

            assert result is not None
            assert result.total_cases == 3
            # The studies come from the mutations data
            assert "coadread_mskcc" in result.studies
            assert len(result.studies) == 1
            # Check enhanced fields
            assert (
                result.cancer_type_distribution["Colorectal Adenocarcinoma"]
                == 3
            )
            assert result.mutation_types["Missense_Mutation"] == 3
            assert (
                result.hotspot_count == 1
            )  # One mutation has "hotspot" in keyword
            assert (
                result.mean_vaf is not None
            )  # VAF calculated from tumor counts

    @pytest.mark.asyncio
    async def test_get_variant_data_not_found(self):
        """Test cBioPortal variant data when not found."""
        client = CBioPortalClient()

        # Mock gene lookup response
        mock_gene_response = {
            "entrezGeneId": 673,
            "hugoGeneSymbol": "BRAF",
        }

        # Mock empty mutations response

        with (
            patch(
                "biomcp.variants.external.httpx.AsyncClient"
            ) as mock_client_class,
            patch(
                "biomcp.variants.external.RetryableHTTPClient"
            ) as mock_retry_class,
        ):
            # Create mock client instance
            mock_client = AsyncMock()
            mock_client_class.return_value.__aenter__.return_value = (
                mock_client
            )

            # Mock retry client
            mock_retry = mock_retry_class.return_value
            mock_retry.get_with_retry = AsyncMock()

            # Mock gene lookup response
            gene_resp = AsyncMock()
            gene_resp.status_code = 200
            gene_resp.json = lambda: mock_gene_response

            # Mock molecular profiles response
            profiles_resp = AsyncMock()
            profiles_resp.status_code = 200
            profiles_resp.json = lambda: [
                {
                    "molecularProfileId": "coadread_mskcc_mutations",
                    "studyId": "coadread_mskcc",
                    "molecularAlterationType": "MUTATION_EXTENDED",
                }
            ]

            # Mock study metadata response
            study_resp = AsyncMock()
            study_resp.status_code = 200
            study_resp.json = lambda: {
                "studyId": "coadread_mskcc",
                "cancerType": {"name": "Colorectal Adenocarcinoma"},
            }

            # Mock empty mutations response for V601E
            empty_resp = AsyncMock()
            empty_resp.status_code = 200
            empty_resp.json = lambda: []

            # Mock the retry client
            mock_retry.get_with_retry.side_effect = [
                gene_resp,
                profiles_resp,
                study_resp,
                empty_resp,
            ]

            result = await client.get_variant_data("BRAF V601E")

            assert result is None

    @pytest.mark.asyncio
    async def test_get_variant_data_invalid_format(self):
        """Test cBioPortal with invalid gene/AA format."""
        client = CBioPortalClient()

        result = await client.get_variant_data("InvalidFormat")

        assert result is None

    @pytest.mark.asyncio
    async def test_get_variant_data_gene_not_found(self):
        """Test cBioPortal when gene is not found."""
        client = CBioPortalClient()

        with (
            patch(
                "biomcp.variants.external.httpx.AsyncClient"
            ) as mock_client_class,
            patch(
                "biomcp.variants.external.RetryableHTTPClient"
            ) as mock_retry_class,
        ):
            # Create mock client instance
            mock_client = AsyncMock()
            mock_client_class.return_value.__aenter__.return_value = (
                mock_client
            )

            # Mock retry client to return None (failure after retries)
            mock_retry = mock_retry_class.return_value
            mock_retry.get_with_retry = AsyncMock(return_value=None)

            result = await client.get_variant_data("FAKEGENE V600E")

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
