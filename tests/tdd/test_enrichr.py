"""
Unit tests for Enrichr integration.

Tests use mocked HTTP responses to avoid actual API calls.
"""

from unittest.mock import AsyncMock, patch

import pytest

from biomcp.enrichr import EnrichrClient, get_database_name
from biomcp.enrichr.databases import ENRICHR_DATABASES


@pytest.fixture
def mock_http_client():
    """Mock the http_client.request_api function."""
    with patch("biomcp.enrichr.client.http_client") as mock:
        yield mock


@pytest.fixture
def enrichr_client():
    """Create an Enrichr client instance."""
    return EnrichrClient()


class TestDatabaseMapping:
    """Test database name mapping."""

    def test_get_database_name_pathway(self):
        """Test mapping for pathway database."""
        assert get_database_name("pathway") == "KEGG_2021_Human"
        assert get_database_name("kegg") == "KEGG_2021_Human"

    def test_get_database_name_ontology(self):
        """Test mapping for ontology database."""
        assert get_database_name("ontology") == "GO_Biological_Process_2021"
        assert get_database_name("go_process") == "GO_Biological_Process_2021"

    def test_get_database_name_full_name(self):
        """Test that full database names pass through."""
        full_name = "GO_Biological_Process_2021"
        assert get_database_name(full_name) == full_name

    def test_get_database_name_invalid(self):
        """Test error handling for invalid database names."""
        with pytest.raises(ValueError, match="Unknown database category"):
            get_database_name("invalid_database")

    def test_all_database_mappings(self):
        """Test that all database mappings are valid."""
        for category in ENRICHR_DATABASES:
            db_name = get_database_name(category)
            assert db_name is not None
            assert isinstance(db_name, str)
            assert len(db_name) > 0


class TestSubmitGeneList:
    """Test submitting gene lists to Enrichr."""

    @pytest.mark.asyncio
    async def test_submit_gene_list_success(
        self, enrichr_client, mock_http_client
    ):
        """Test successful gene list submission."""
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {"userListId": 123456, "shortId": "abc123"},
                None,
            )
        )

        genes = ["TP53", "BRCA1", "EGFR"]
        user_list_id = await enrichr_client.submit_gene_list(genes)

        assert user_list_id == "123456"
        mock_http_client.request_api.assert_called_once()

        # Verify the call parameters
        call_args = mock_http_client.request_api.call_args
        assert call_args.kwargs["method"] == "POST"
        assert call_args.kwargs["domain"] == "enrichr"
        # Check the _files parameter contains the gene list
        files = call_args.kwargs["request"]["_files"]
        assert "list" in files
        gene_list_content = files["list"][1]  # Second element is the content
        assert "TP53" in gene_list_content
        assert "BRCA1" in gene_list_content

    @pytest.mark.asyncio
    async def test_submit_gene_list_string(
        self, enrichr_client, mock_http_client
    ):
        """Test gene list submission with string input."""
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {"userListId": 789, "shortId": "xyz"},
                None,
            )
        )

        genes_string = "TP53\nBRCA1\nEGFR"
        user_list_id = await enrichr_client.submit_gene_list(genes_string)

        assert user_list_id == "789"

    @pytest.mark.asyncio
    async def test_submit_gene_list_error(
        self, enrichr_client, mock_http_client
    ):
        """Test handling of submission errors."""
        mock_http_client.request_api = AsyncMock(
            return_value=(None, "API Error")
        )

        user_list_id = await enrichr_client.submit_gene_list(["TP53"])

        assert user_list_id is None

    @pytest.mark.asyncio
    async def test_submit_gene_list_no_id_in_response(
        self, enrichr_client, mock_http_client
    ):
        """Test handling when response lacks userListId."""
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {"shortId": "abc"},  # Missing userListId
                None,
            )
        )

        user_list_id = await enrichr_client.submit_gene_list(["TP53"])

        assert user_list_id is None


class TestGetEnrichment:
    """Test retrieving enrichment results."""

    @pytest.mark.asyncio
    async def test_get_enrichment_success(
        self, enrichr_client, mock_http_client
    ):
        """Test successful enrichment retrieval."""
        database = "GO_Biological_Process_2021"
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {
                    database: [
                        [
                            1,  # rank
                            "regulation of gene expression (GO:0010468)",  # term
                            0.1532821089,  # p_val
                            8.7750463822,  # z_score
                            16.4573819271,  # combined_score
                            "TP53;BRCA1",  # genes (semicolon-separated)
                            0.1558156974,  # adj_p_val
                        ],
                        [
                            2,
                            "apoptotic process (GO:0006915)",
                            0.0123456,
                            12.5,
                            25.0,
                            "TP53",
                            0.0234567,
                        ],
                    ]
                },
                None,
            )
        )

        results = await enrichr_client.get_enrichment("123456", database)

        assert results is not None
        assert len(results) == 2

        # Check first term
        first_term = results[0]
        assert first_term.rank == 1
        assert "regulation of gene expression" in first_term.path_name
        assert first_term.p_val == pytest.approx(0.1532821089)
        assert first_term.z_score == pytest.approx(8.7750463822)
        assert first_term.combined_score == pytest.approx(16.4573819271)
        assert first_term.overlapping_genes == ["TP53", "BRCA1"]
        assert first_term.adj_p_val == pytest.approx(0.1558156974)
        assert first_term.database == database

    @pytest.mark.asyncio
    async def test_get_enrichment_error(
        self, enrichr_client, mock_http_client
    ):
        """Test handling of enrichment retrieval errors."""
        mock_http_client.request_api = AsyncMock(
            return_value=(None, "API Error")
        )

        results = await enrichr_client.get_enrichment(
            "123456", "GO_Biological_Process_2021"
        )

        assert results is None

    @pytest.mark.asyncio
    async def test_get_enrichment_no_database_in_response(
        self, enrichr_client, mock_http_client
    ):
        """Test handling when database is not in response."""
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {"OtherDatabase": []},  # Wrong database key
                None,
            )
        )

        results = await enrichr_client.get_enrichment(
            "123456", "GO_Biological_Process_2021"
        )

        assert results is None

    @pytest.mark.asyncio
    async def test_get_enrichment_malformed_result(
        self, enrichr_client, mock_http_client
    ):
        """Test handling of malformed enrichment results."""
        database = "GO_Biological_Process_2021"
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {
                    database: [
                        [
                            1,
                            "valid term",
                            0.05,
                            10.0,
                            20.0,
                            "TP53",
                            0.06,
                        ],  # Valid
                        [2, "incomplete"],  # Malformed - missing fields
                        [
                            3,
                            "another valid",
                            0.01,
                            15.0,
                            30.0,
                            "BRCA1",
                            0.02,
                        ],  # Valid
                    ]
                },
                None,
            )
        )

        results = await enrichr_client.get_enrichment("123456", database)

        # Should return only the valid terms
        assert results is not None
        assert len(results) == 2
        assert results[0].path_name == "valid term"
        assert results[1].path_name == "another valid"


class TestEnrichOneStep:
    """Test the convenience enrich() method."""

    @pytest.mark.asyncio
    async def test_enrich_success(self, enrichr_client, mock_http_client):
        """Test successful one-step enrichment."""
        database = "KEGG_2021_Human"

        # Mock both submit and get enrichment calls
        mock_http_client.request_api = AsyncMock(
            side_effect=[
                # First call: submit_gene_list
                ({"userListId": 123456, "shortId": "abc"}, None),
                # Second call: get_enrichment
                (
                    {
                        database: [
                            [1, "Pathway A", 0.01, 10.0, 20.0, "TP53", 0.02],
                        ]
                    },
                    None,
                ),
            ]
        )

        results = await enrichr_client.enrich(
            genes=["TP53"],
            database="pathway",  # Use friendly name
            description="Test enrichment",
        )

        assert results is not None
        assert len(results) == 1
        assert results[0].path_name == "Pathway A"
        assert results[0].database == database

    @pytest.mark.asyncio
    async def test_enrich_invalid_database(
        self, enrichr_client, mock_http_client
    ):
        """Test error handling for invalid database category."""
        results = await enrichr_client.enrich(
            genes=["TP53"], database="invalid_category"
        )

        assert results is None

    @pytest.mark.asyncio
    async def test_enrich_submit_fails(self, enrichr_client, mock_http_client):
        """Test handling when gene list submission fails."""
        mock_http_client.request_api = AsyncMock(
            return_value=(None, "Submit failed")
        )

        results = await enrichr_client.enrich(
            genes=["TP53"], database="pathway"
        )

        assert results is None
