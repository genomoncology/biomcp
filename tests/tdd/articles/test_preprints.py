"""Tests for preprint search functionality."""

from unittest.mock import AsyncMock, patch

import pytest

from biomcp.articles.preprints import (
    BiorxivClient,
    BiorxivResponse,
    BiorxivResult,
    EuropePMCClient,
    EuropePMCResponse,
    PreprintSearcher,
)
from biomcp.articles.search import PubmedRequest, ResultItem
from biomcp.core import PublicationState


class TestBiorxivClient:
    """Tests for BiorxivClient."""

    @pytest.mark.asyncio
    async def test_search_biorxiv_success(self):
        """Test successful bioRxiv search."""
        client = BiorxivClient()

        # Mock response
        mock_response = BiorxivResponse(
            collection=[
                BiorxivResult(
                    doi="10.1101/2024.01.01.123456",
                    title="Test BRAF Mutation Study",
                    authors="Smith, J.; Doe, J.",
                    date="2024-01-01",
                    abstract="Study about BRAF mutations in cancer.",
                    server="biorxiv",
                )
            ],
            total=1,
        )

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (mock_response, None)

            results = await client.search("BRAF")

            assert len(results) == 1
            assert results[0].doi == "10.1101/2024.01.01.123456"
            assert results[0].title == "Test BRAF Mutation Study"
            assert results[0].publication_state == PublicationState.PREPRINT
            assert "preprint" in results[0].journal.lower()

    @pytest.mark.asyncio
    async def test_search_biorxiv_no_results(self):
        """Test bioRxiv search with no results."""
        client = BiorxivClient()

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (
                None,
                {"code": 404, "message": "Not found"},
            )

            results = await client.search("nonexistent")

            assert len(results) == 0


class TestEuropePMCClient:
    """Tests for EuropePMCClient."""

    @pytest.mark.asyncio
    async def test_search_europe_pmc_success(self):
        """Test successful Europe PMC search."""
        client = EuropePMCClient()

        # Mock response
        mock_response = EuropePMCResponse(
            hitCount=1,
            resultList={
                "result": [
                    {
                        "id": "PPR123456",
                        "doi": "10.1101/2024.01.02.654321",
                        "title": "TP53 Mutation Analysis",
                        "authorString": "Johnson, A., Williams, B.",
                        "journalTitle": "bioRxiv",
                        "firstPublicationDate": "2024-01-02",
                        "abstractText": "Analysis of TP53 mutations.",
                    }
                ]
            },
        )

        with patch("biomcp.http_client.request_api") as mock_request:
            mock_request.return_value = (mock_response, None)

            results = await client.search("TP53")

            assert len(results) == 1
            assert results[0].doi == "10.1101/2024.01.02.654321"
            assert results[0].title == "TP53 Mutation Analysis"
            assert results[0].publication_state == PublicationState.PREPRINT


class TestPreprintSearcher:
    """Tests for PreprintSearcher."""

    @pytest.mark.asyncio
    async def test_search_combined_sources(self):
        """Test searching across multiple preprint sources."""
        searcher = PreprintSearcher()

        # Mock both clients
        mock_biorxiv_results = [
            ResultItem(
                doi="10.1101/2024.01.01.111111",
                title="BRAF Study 1",
                date="2024-01-01",
                publication_state=PublicationState.PREPRINT,
            )
        ]

        mock_europe_results = [
            ResultItem(
                doi="10.1101/2024.01.02.222222",
                title="BRAF Study 2",
                date="2024-01-02",
                publication_state=PublicationState.PREPRINT,
            )
        ]

        searcher.biorxiv_client.search = AsyncMock(
            return_value=mock_biorxiv_results
        )
        searcher.europe_pmc_client.search = AsyncMock(
            return_value=mock_europe_results
        )

        request = PubmedRequest(genes=["BRAF"])
        response = await searcher.search(request)

        assert response.count == 2
        assert len(response.results) == 2
        # Results should be sorted by date (newest first)
        assert response.results[0].doi == "10.1101/2024.01.02.222222"
        assert response.results[1].doi == "10.1101/2024.01.01.111111"

    @pytest.mark.asyncio
    async def test_search_duplicate_removal(self):
        """Test that duplicate DOIs are removed."""
        searcher = PreprintSearcher()

        # Create duplicate results with same DOI
        duplicate_doi = "10.1101/2024.01.01.999999"

        mock_biorxiv_results = [
            ResultItem(
                doi=duplicate_doi,
                title="Duplicate Study",
                date="2024-01-01",
                publication_state=PublicationState.PREPRINT,
            )
        ]

        mock_europe_results = [
            ResultItem(
                doi=duplicate_doi,
                title="Duplicate Study",
                date="2024-01-01",
                publication_state=PublicationState.PREPRINT,
            )
        ]

        searcher.biorxiv_client.search = AsyncMock(
            return_value=mock_biorxiv_results
        )
        searcher.europe_pmc_client.search = AsyncMock(
            return_value=mock_europe_results
        )

        request = PubmedRequest(keywords=["test"])
        response = await searcher.search(request)

        assert response.count == 1
        assert len(response.results) == 1
        assert response.results[0].doi == duplicate_doi
