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
            "biomcp.articles.search_optimized.article_searcher_optimized"
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

            # Check the parameters were passed correctly
            kwargs = mock_search.call_args[1]
            assert kwargs["call_benefit"] == "Testing search functionality"
            assert kwargs["chemicals"] == "aspirin,ibuprofen"
            assert kwargs["diseases"] == "cancer,diabetes"
            assert kwargs["genes"] == "BRAF,TP53"
            assert kwargs["keywords"] == "mutation,therapy"
            assert kwargs["variants"] == "V600E,R175H"
            assert kwargs["include_preprints"] is True
            assert kwargs.get("include_cbioportal", True) is True

    @pytest.mark.asyncio
    async def test_article_searcher_with_lists(self):
        """Test article_searcher with list inputs."""
        with patch(
            "biomcp.articles.search_optimized.article_searcher_optimized"
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
            kwargs = mock_search.call_args[1]
            assert kwargs["call_benefit"] == "Testing with lists"
            assert kwargs["chemicals"] == ["drug1", "drug2"]
            assert kwargs["diseases"] == ["disease1"]
            assert kwargs["genes"] == ["GENE1"]
            assert kwargs["include_preprints"] is False

    @pytest.mark.asyncio
    async def test_article_searcher_minimal_params(self):
        """Test article_searcher with minimal parameters."""
        with patch(
            "biomcp.articles.search_optimized.article_searcher_optimized"
        ) as mock_search:
            mock_search.return_value = "## No results"

            await _article_searcher(call_benefit="Minimal test")

            # Should still work with no search parameters
            kwargs = mock_search.call_args[1]
            assert kwargs["call_benefit"] == "Minimal test"
            assert kwargs.get("chemicals") is None
            assert kwargs.get("diseases") is None
            assert kwargs.get("genes") is None
            assert kwargs.get("keywords") is None
            assert kwargs.get("variants") is None

    @pytest.mark.asyncio
    async def test_article_searcher_empty_strings(self):
        """Test article_searcher with empty strings."""
        with patch(
            "biomcp.articles.search_optimized.article_searcher_optimized"
        ) as mock_search:
            mock_search.return_value = "## Results"

            await _article_searcher(
                call_benefit="Empty string test",
                chemicals="",
                diseases="",
                genes="",
            )

            # Empty strings are passed through
            kwargs = mock_search.call_args[1]
            assert kwargs["call_benefit"] == "Empty string test"
            assert kwargs["chemicals"] == ""
            assert kwargs["diseases"] == ""
            assert kwargs["genes"] == ""
