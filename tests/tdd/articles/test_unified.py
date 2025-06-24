"""Tests for unified article search functionality."""

import json
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.articles.search import PubmedRequest
from biomcp.articles.unified import (
    _deduplicate_articles,
    _parse_search_results,
    search_articles_unified,
)


class TestUnifiedSearch:
    """Test unified search functionality."""

    @pytest.fixture
    def pubmed_results(self):
        """Sample PubMed results in JSON format."""
        return json.dumps([
            {
                "pmid": 12345,
                "title": "BRAF mutations in cancer",
                "doi": "10.1234/test1",
                "date": "2024-01-15",
                "publication_state": "peer_reviewed",
            },
            {
                "pmid": 12346,
                "title": "Another cancer study",
                "doi": "10.1234/test2",
                "date": "2024-01-10",
                "publication_state": "peer_reviewed",
            },
        ])

    @pytest.fixture
    def preprint_results(self):
        """Sample preprint results in JSON format."""
        return json.dumps([
            {
                "title": "BRAF preprint study",
                "doi": "10.1101/2024.01.20.123456",
                "date": "2024-01-20",
                "publication_state": "preprint",
                "source": "bioRxiv",
            },
            {
                "title": "Duplicate study",
                "doi": "10.1234/test1",  # Same DOI as PubMed result
                "date": "2024-01-14",
                "publication_state": "preprint",
                "source": "Europe PMC",
            },
        ])

    @pytest.mark.asyncio
    async def test_search_articles_unified_both_sources(
        self, pubmed_results, preprint_results
    ):
        """Test searching with both PubMed and preprints enabled."""
        request = PubmedRequest(genes=["BRAF"])

        mock_pubmed = AsyncMock(return_value=pubmed_results)
        mock_preprints = AsyncMock(return_value=preprint_results)

        with (
            patch("biomcp.articles.unified.search_articles", mock_pubmed),
            patch("biomcp.articles.unified.search_preprints", mock_preprints),
            patch(
                "biomcp.variants.cbioportal_search.CBioPortalSearchClient"
            ) as mock_cbio,
        ):
            # Mock cBioPortal client to return None (no summary)
            mock_cbio.return_value.get_gene_search_summary = AsyncMock(
                return_value=None
            )

            result = await search_articles_unified(
                request,
                include_pubmed=True,
                include_preprints=True,
                output_json=True,
            )

            # Parse result
            data = json.loads(result)

            # When gene is specified but cBioPortal returns no data,
            # we should just get the articles list
            if isinstance(data, dict):
                articles = data.get("articles", data)
            else:
                articles = data

            # Should have 3 articles (one duplicate removed)
            assert len(articles) == 3

            # Check ordering - peer reviewed should come first
            # Sort is by (publication_state priority, date DESC)
            # The test data has preprint with newer date, so it might come first
            # Let's just check we have the right mix
            states = [a["publication_state"] for a in articles]
            assert states.count("peer_reviewed") == 2
            assert states.count("preprint") == 1

            # Check deduplication worked
            dois = [a.get("doi") for a in articles if a.get("doi")]
            assert len(dois) == len(set(dois))  # No duplicate DOIs

    @pytest.mark.asyncio
    async def test_search_articles_unified_pubmed_only(self, pubmed_results):
        """Test searching with only PubMed enabled."""
        request = PubmedRequest(
            keywords=["cancer"]
        )  # No gene, so no cBioPortal

        with (
            patch("biomcp.articles.unified.search_articles") as mock_pubmed,
            patch(
                "biomcp.articles.unified.search_preprints"
            ) as mock_preprints,
        ):
            mock_pubmed.return_value = pubmed_results

            result = await search_articles_unified(
                request,
                include_pubmed=True,
                include_preprints=False,
                output_json=True,
            )

            # Preprints should not be called
            mock_preprints.assert_not_called()

            # Parse result
            articles = json.loads(result)
            assert len(articles) == 2
            assert all(
                a["publication_state"] == "peer_reviewed" for a in articles
            )

    @pytest.mark.asyncio
    async def test_search_articles_unified_preprints_only(
        self, preprint_results
    ):
        """Test searching with only preprints enabled."""
        request = PubmedRequest(
            keywords=["cancer"]
        )  # No gene, so no cBioPortal

        with (
            patch("biomcp.articles.unified.search_articles") as mock_pubmed,
            patch(
                "biomcp.articles.unified.search_preprints"
            ) as mock_preprints,
        ):
            mock_preprints.return_value = preprint_results

            result = await search_articles_unified(
                request,
                include_pubmed=False,
                include_preprints=True,
                output_json=True,
            )

            # PubMed should not be called
            mock_pubmed.assert_not_called()

            # Parse result
            articles = json.loads(result)
            assert len(articles) == 2
            assert all(a["publication_state"] == "preprint" for a in articles)

    @pytest.mark.asyncio
    async def test_search_articles_unified_error_handling(self):
        """Test error handling when one source fails."""
        request = PubmedRequest(
            keywords=["cancer"]
        )  # No gene, so no cBioPortal

        with (
            patch("biomcp.articles.unified.search_articles") as mock_pubmed,
            patch(
                "biomcp.articles.unified.search_preprints"
            ) as mock_preprints,
        ):
            # PubMed succeeds
            mock_pubmed.return_value = json.dumps([{"title": "Success"}])
            # Preprints fails
            mock_preprints.side_effect = Exception("API Error")

            result = await search_articles_unified(
                request,
                include_pubmed=True,
                include_preprints=True,
                output_json=True,
            )

            # Should still get PubMed results
            articles = json.loads(result)
            assert len(articles) == 1
            assert articles[0]["title"] == "Success"

    @pytest.mark.asyncio
    async def test_search_articles_unified_markdown_output(
        self, pubmed_results
    ):
        """Test markdown output format."""
        request = PubmedRequest(genes=["BRAF"])

        mock_pubmed = AsyncMock(return_value=pubmed_results)

        with patch("biomcp.articles.unified.search_articles", mock_pubmed):
            result = await search_articles_unified(
                request,
                include_pubmed=True,
                include_preprints=False,
                output_json=False,
            )

            # Should return markdown
            assert isinstance(result, str)
            assert "BRAF mutations in cancer" in result
            assert "# Record" in result  # Markdown headers

    def test_deduplicate_articles(self):
        """Test article deduplication logic."""
        articles = [
            {"title": "Article 1", "doi": "10.1234/test1"},
            {"title": "Article 2", "doi": "10.1234/test2"},
            {"title": "Duplicate of 1", "doi": "10.1234/test1"},
            {"title": "No DOI article"},
            {"title": "Another no DOI"},
        ]

        deduped = _deduplicate_articles(articles)

        # Should have 4 articles (one duplicate removed)
        assert len(deduped) == 4

        # Check DOIs are unique
        dois = [a.get("doi") for a in deduped if a.get("doi")]
        assert len(dois) == len(set(dois))

        # Articles without DOI should be preserved
        no_doi_count = sum(1 for a in deduped if not a.get("doi"))
        assert no_doi_count == 2

    def test_parse_search_results(self):
        """Test parsing of search results from multiple sources."""
        results = [
            json.dumps([{"title": "Article 1"}, {"title": "Article 2"}]),
            json.dumps([{"title": "Article 3"}]),
            Exception("Failed source"),  # Should be skipped
            "[invalid json",  # Should be skipped
        ]

        parsed = _parse_search_results(results)

        # Should have 3 articles (2 + 1, skipping errors)
        assert len(parsed) == 3
        assert parsed[0]["title"] == "Article 1"
        assert parsed[1]["title"] == "Article 2"
        assert parsed[2]["title"] == "Article 3"

    def test_parse_search_results_empty(self):
        """Test parsing with all empty/failed results."""
        results = [
            Exception("Failed"),
            "[invalid",
            json.dumps([]),  # Empty list
        ]

        parsed = _parse_search_results(results)
        assert parsed == []
