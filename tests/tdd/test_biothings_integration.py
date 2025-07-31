"""Unit tests for BioThings API integration."""

from unittest.mock import AsyncMock, patch

import pytest

from biomcp.integrations import BioThingsClient, DiseaseInfo, GeneInfo


@pytest.fixture
def mock_http_client():
    """Mock the http_client.request_api function."""
    with patch("biomcp.integrations.biothings_client.http_client") as mock:
        yield mock


@pytest.fixture
def biothings_client():
    """Create a BioThings client instance."""
    return BioThingsClient()


class TestGeneInfo:
    """Test gene information retrieval."""

    @pytest.mark.asyncio
    async def test_get_gene_by_symbol(
        self, biothings_client, mock_http_client
    ):
        """Test getting gene info by symbol."""
        # Mock query response
        mock_http_client.request_api = AsyncMock(
            side_effect=[
                (
                    {
                        "hits": [
                            {
                                "_id": "7157",
                                "symbol": "TP53",
                                "name": "tumor protein p53",
                                "taxid": 9606,
                            }
                        ]
                    },
                    None,
                ),
                # Mock get response
                (
                    {
                        "_id": "7157",
                        "symbol": "TP53",
                        "name": "tumor protein p53",
                        "summary": "This gene encodes a tumor suppressor protein...",
                        "alias": ["p53", "LFS1"],
                        "type_of_gene": "protein-coding",
                        "entrezgene": 7157,
                    },
                    None,
                ),
            ]
        )

        result = await biothings_client.get_gene_info("TP53")

        assert result is not None
        assert isinstance(result, GeneInfo)
        assert result.symbol == "TP53"
        assert result.name == "tumor protein p53"
        assert result.gene_id == "7157"
        assert "p53" in result.alias

    @pytest.mark.asyncio
    async def test_get_gene_by_id(self, biothings_client, mock_http_client):
        """Test getting gene info by Entrez ID."""
        # Mock direct get response
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {
                    "_id": "7157",
                    "symbol": "TP53",
                    "name": "tumor protein p53",
                    "summary": "This gene encodes a tumor suppressor protein...",
                },
                None,
            )
        )

        result = await biothings_client.get_gene_info("7157")

        assert result is not None
        assert result.symbol == "TP53"
        assert result.gene_id == "7157"

    @pytest.mark.asyncio
    async def test_gene_not_found(self, biothings_client, mock_http_client):
        """Test handling of gene not found."""
        mock_http_client.request_api = AsyncMock(
            return_value=({"hits": []}, None)
        )

        result = await biothings_client.get_gene_info("INVALID_GENE")
        assert result is None

    @pytest.mark.asyncio
    async def test_batch_get_genes(self, biothings_client, mock_http_client):
        """Test batch gene retrieval."""
        mock_http_client.request_api = AsyncMock(
            return_value=(
                [
                    {
                        "_id": "7157",
                        "symbol": "TP53",
                        "name": "tumor protein p53",
                    },
                    {
                        "_id": "673",
                        "symbol": "BRAF",
                        "name": "B-Raf proto-oncogene",
                    },
                ],
                None,
            )
        )

        results = await biothings_client.batch_get_genes(["TP53", "BRAF"])

        assert len(results) == 2
        assert results[0].symbol == "TP53"
        assert results[1].symbol == "BRAF"


class TestDiseaseInfo:
    """Test disease information retrieval."""

    @pytest.mark.asyncio
    async def test_get_disease_by_name(
        self, biothings_client, mock_http_client
    ):
        """Test getting disease info by name."""
        # Mock query response
        mock_http_client.request_api = AsyncMock(
            side_effect=[
                (
                    {
                        "hits": [
                            {
                                "_id": "MONDO:0007959",
                                "name": "melanoma",
                                "mondo": {"mondo": "MONDO:0007959"},
                            }
                        ]
                    },
                    None,
                ),
                # Mock get response
                (
                    {
                        "_id": "MONDO:0007959",
                        "name": "melanoma",
                        "mondo": {
                            "definition": "A malignant neoplasm composed of melanocytes.",
                            "synonym": {
                                "exact": [
                                    "malignant melanoma",
                                    "naevocarcinoma",
                                ]
                            },
                        },
                    },
                    None,
                ),
            ]
        )

        result = await biothings_client.get_disease_info("melanoma")

        assert result is not None
        assert isinstance(result, DiseaseInfo)
        assert result.name == "melanoma"
        assert result.disease_id == "MONDO:0007959"
        assert "malignant melanoma" in result.synonyms

    @pytest.mark.asyncio
    async def test_get_disease_by_id(self, biothings_client, mock_http_client):
        """Test getting disease info by MONDO ID."""
        mock_http_client.request_api = AsyncMock(
            return_value=(
                {
                    "_id": "MONDO:0016575",
                    "name": "GIST",
                    "mondo": {
                        "definition": "Gastrointestinal stromal tumor...",
                    },
                },
                None,
            )
        )

        result = await biothings_client.get_disease_info("MONDO:0016575")

        assert result is not None
        assert result.name == "GIST"
        assert result.disease_id == "MONDO:0016575"

    @pytest.mark.asyncio
    async def test_get_disease_synonyms(
        self, biothings_client, mock_http_client
    ):
        """Test getting disease synonyms for query expansion."""
        mock_http_client.request_api = AsyncMock(
            side_effect=[
                (
                    {
                        "hits": [
                            {
                                "_id": "MONDO:0018076",
                                "name": "GIST",
                            }
                        ]
                    },
                    None,
                ),
                (
                    {
                        "_id": "MONDO:0018076",
                        "name": "gastrointestinal stromal tumor",
                        "mondo": {
                            "synonym": {
                                "exact": [
                                    "GIST",
                                    "gastrointestinal stromal tumour",
                                    "GI stromal tumor",
                                ]
                            }
                        },
                    },
                    None,
                ),
            ]
        )

        synonyms = await biothings_client.get_disease_synonyms("GIST")

        assert "GIST" in synonyms
        assert "gastrointestinal stromal tumor" in synonyms
        assert len(synonyms) <= 5  # Limited to 5


class TestTrialSynonymExpansion:
    """Test disease synonym expansion in trial searches."""

    @pytest.mark.asyncio
    async def test_trial_search_with_synonym_expansion(self):
        """Test that trial search expands disease synonyms."""
        from biomcp.trials.search import TrialQuery, convert_query

        with patch("biomcp.trials.search.BioThingsClient") as mock_client:
            # Mock synonym expansion
            mock_instance = mock_client.return_value
            mock_instance.get_disease_synonyms = AsyncMock(
                return_value=[
                    "GIST",
                    "gastrointestinal stromal tumor",
                    "GI stromal tumor",
                ]
            )

            query = TrialQuery(
                conditions=["GIST"],
                expand_synonyms=True,
            )

            params = await convert_query(query)

            # Check that conditions were expanded
            assert "query.cond" in params
            cond_value = params["query.cond"][0]
            assert "GIST" in cond_value
            assert "gastrointestinal stromal tumor" in cond_value

    @pytest.mark.asyncio
    async def test_trial_search_without_synonym_expansion(self):
        """Test that trial search works without synonym expansion."""
        from biomcp.trials.search import TrialQuery, convert_query

        query = TrialQuery(
            conditions=["GIST"],
            expand_synonyms=False,
        )

        params = await convert_query(query)

        # Check that conditions were not expanded
        assert "query.cond" in params
        assert params["query.cond"] == ["GIST"]


class TestErrorHandling:
    """Test error handling in BioThings integration."""

    @pytest.mark.asyncio
    async def test_api_error_handling(
        self, biothings_client, mock_http_client
    ):
        """Test handling of API errors."""
        from biomcp.http_client import RequestError

        mock_http_client.request_api = AsyncMock(
            return_value=(
                None,
                RequestError(code=500, message="Internal server error"),
            )
        )

        result = await biothings_client.get_gene_info("TP53")
        assert result is None

    @pytest.mark.asyncio
    async def test_invalid_response_format(
        self, biothings_client, mock_http_client
    ):
        """Test handling of invalid API responses."""
        mock_http_client.request_api = AsyncMock(
            return_value=({"invalid": "response"}, None)
        )

        result = await biothings_client.get_gene_info("TP53")
        assert result is None
