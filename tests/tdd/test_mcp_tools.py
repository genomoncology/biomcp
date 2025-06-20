"""Tests for MCP tool wrappers."""

import json
from unittest.mock import patch

import pytest

from biomcp.articles.search import _article_searcher


class TestArticleSearcherMCPTool:
    """Test the _article_searcher MCP tool."""

    @pytest.mark.asyncio
    async def test_article_searcher_with_all_params(self):
        """Test article_searcher with all parameters."""
        mock_results = [{"title": "Test Article", "pmid": 12345}]

        with patch(
            "biomcp.articles.unified.search_articles_unified"
        ) as mock_search:
            mock_search.return_value = json.dumps(mock_results)

            await _article_searcher(
                call_benefit="Testing search functionality",
                chemicals="aspirin,ibuprofen",
                diseases="cancer,diabetes",
                genes="BRAF,TP53",
                keywords="mutation,therapy",
                variants="V600E,R175H",
                include_preprints=True,
            )

            # Verify the function was called
            mock_search.assert_called_once()

            # Check the request object was created properly
            args = mock_search.call_args[0]
            request = args[0]

            assert request.chemicals == ["aspirin", "ibuprofen"]
            assert request.diseases == ["cancer", "diabetes"]
            assert request.genes == ["BRAF", "TP53"]
            assert request.keywords == ["mutation", "therapy"]
            assert request.variants == ["V600E", "R175H"]

            # Check other parameters
            kwargs = mock_search.call_args[1]
            assert kwargs["include_pubmed"] is True
            assert kwargs["include_preprints"] is True

    @pytest.mark.asyncio
    async def test_article_searcher_with_lists(self):
        """Test article_searcher with list inputs."""
        with patch(
            "biomcp.articles.unified.search_articles_unified"
        ) as mock_search:
            mock_search.return_value = "## Results"

            await _article_searcher(
                call_benefit="Testing with lists",
                chemicals=["drug1", "drug2"],
                diseases=["disease1"],
                genes=["GENE1"],
                include_preprints=False,
            )

            # Check list parameters were passed correctly
            args = mock_search.call_args[0]
            request = args[0]

            assert request.chemicals == ["drug1", "drug2"]
            assert request.diseases == ["disease1"]
            assert request.genes == ["GENE1"]

            # Check include_preprints was respected
            kwargs = mock_search.call_args[1]
            assert kwargs["include_preprints"] is False

    @pytest.mark.asyncio
    async def test_article_searcher_minimal_params(self):
        """Test article_searcher with minimal parameters."""
        with patch(
            "biomcp.articles.unified.search_articles_unified"
        ) as mock_search:
            mock_search.return_value = "## No results"

            await _article_searcher(call_benefit="Minimal test")

            # Should still work with no search parameters
            args = mock_search.call_args[0]
            request = args[0]

            assert request.chemicals == []
            assert request.diseases == []
            assert request.genes == []
            assert request.keywords == []
            assert request.variants == []

    @pytest.mark.asyncio
    async def test_article_searcher_empty_strings(self):
        """Test article_searcher with empty strings."""
        with patch(
            "biomcp.articles.unified.search_articles_unified"
        ) as mock_search:
            mock_search.return_value = "## Results"

            await _article_searcher(
                call_benefit="Empty string test",
                chemicals="",
                diseases="",
                genes="",
            )

            # Empty strings with split_strings=True result in lists with empty string
            args = mock_search.call_args[0]
            request = args[0]

            assert request.chemicals == [""]
            assert request.diseases == [""]
            assert request.genes == [""]
