"""Tests for Europe PMC article fetching via DOI."""

import json
from unittest.mock import Mock, patch

import pytest

from biomcp.articles.fetch import _article_details, is_doi, is_pmid
from biomcp.articles.preprints import fetch_europe_pmc_article


class TestDOIDetection:
    """Test DOI and PMID detection functions."""

    def test_valid_dois(self):
        """Test that valid DOIs are correctly identified."""
        valid_dois = [
            "10.1101/2024.01.20.23288905",
            "10.1038/nature12373",
            "10.1016/j.cell.2023.05.001",
            "10.1126/science.abc1234",
        ]
        for doi in valid_dois:
            assert (
                is_doi(doi) is True
            ), f"Expected {doi} to be identified as DOI"
            assert (
                is_pmid(doi) is False
            ), f"Expected {doi} NOT to be identified as PMID"

    def test_valid_pmids(self):
        """Test that valid PMIDs are correctly identified."""
        valid_pmids = [
            "35271234",
            "12345678",
            "1",
            "999999999",
        ]
        for pmid in valid_pmids:
            assert (
                is_pmid(pmid) is True
            ), f"Expected {pmid} to be identified as PMID"
            assert (
                is_doi(pmid) is False
            ), f"Expected {pmid} NOT to be identified as DOI"

    def test_invalid_identifiers(self):
        """Test that invalid identifiers are rejected by both functions."""
        invalid_ids = [
            "PMC11193658",  # PMC ID
            "abc123",  # Random string
            "10.1101",  # Incomplete DOI
            "nature12373",  # DOI without prefix
            "",  # Empty string
        ]
        for identifier in invalid_ids:
            assert (
                is_doi(identifier) is False
            ), f"Expected {identifier} NOT to be identified as DOI"
            assert (
                is_pmid(identifier) is False
            ), f"Expected {identifier} NOT to be identified as PMID"


class TestEuropePMCFetch:
    """Test Europe PMC article fetching."""

    @pytest.mark.asyncio
    async def test_fetch_europe_pmc_article_success(self):
        """Test successful fetch from Europe PMC."""
        # Mock the response
        mock_response = Mock()
        mock_response.hitCount = 1
        mock_response.results = [
            Mock(
                id="PPR790987",
                source="PPR",
                pmid=None,
                pmcid=None,
                doi="10.1101/2024.01.20.23288905",
                title="Test Article Title",
                authorString="Author A, Author B, Author C",
                journalTitle=None,
                pubYear="2024",
                firstPublicationDate="2024-01-23",
                abstractText="This is the abstract text.",
            )
        ]

        with patch(
            "biomcp.articles.preprints.http_client.request_api"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await fetch_europe_pmc_article(
                "10.1101/2024.01.20.23288905", output_json=True
            )
            data = json.loads(result)

            assert len(data) == 1
            article = data[0]
            assert article["doi"] == "10.1101/2024.01.20.23288905"
            assert article["title"] == "Test Article Title"
            assert article["journal"] == "Preprint Server (preprint)"
            assert article["date"] == "2024-01-23"
            assert article["authors"] == ["Author A", "Author B", "Author C"]
            assert article["abstract"] == "This is the abstract text."
            assert article["source"] == "Europe PMC"
            assert article["pmid"] is None
            assert "europepmc.org" in article["pmc_url"]

    @pytest.mark.asyncio
    async def test_fetch_europe_pmc_article_not_found(self):
        """Test fetch when article is not found in Europe PMC."""
        mock_response = Mock()
        mock_response.hitCount = 0
        mock_response.results = []

        with patch(
            "biomcp.articles.preprints.http_client.request_api"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await fetch_europe_pmc_article(
                "10.1101/invalid.doi", output_json=True
            )
            data = json.loads(result)

            assert len(data) == 1
            assert data[0]["error"] == "Article not found in Europe PMC"

    @pytest.mark.asyncio
    async def test_fetch_europe_pmc_article_error(self):
        """Test fetch when Europe PMC API returns an error."""
        mock_error = Mock()
        mock_error.code = 500
        mock_error.message = "Internal Server Error"

        with patch(
            "biomcp.articles.preprints.http_client.request_api"
        ) as mock_request:
            mock_request.return_value = (None, mock_error)

            result = await fetch_europe_pmc_article(
                "10.1101/2024.01.20.23288905", output_json=True
            )
            data = json.loads(result)

            assert len(data) == 1
            assert data[0]["error"] == "Error 500: Internal Server Error"


class TestArticleDetailsRouting:
    """Test that _article_details correctly routes DOIs to Europe PMC."""

    @pytest.mark.asyncio
    async def test_doi_routes_to_europe_pmc(self):
        """Test that DOIs are routed to fetch_europe_pmc_article."""
        test_doi = "10.1101/2024.01.20.23288905"

        with patch(
            "biomcp.articles.preprints.fetch_europe_pmc_article"
        ) as mock_europe_pmc:
            mock_europe_pmc.return_value = "Europe PMC result"

            result = await _article_details("Test", test_doi)

            mock_europe_pmc.assert_called_once_with(test_doi, output_json=True)
            assert result == "Europe PMC result"

    @pytest.mark.asyncio
    async def test_pmid_routes_to_pubtator(self):
        """Test that PMIDs are routed to fetch_articles."""
        test_pmid = "35271234"

        with patch(
            "biomcp.articles.fetch.fetch_articles"
        ) as mock_fetch_articles:
            mock_fetch_articles.return_value = "PubTator result"

            result = await _article_details("Test", test_pmid)

            mock_fetch_articles.assert_called_once_with(
                [35271234], full=True, output_json=True
            )
            assert result == "PubTator result"

    @pytest.mark.asyncio
    async def test_invalid_identifier_returns_error(self):
        """Test that invalid identifiers return an error."""
        invalid_id = "PMC12345"

        result = await _article_details("Test", invalid_id)

        data = json.loads(result)
        assert len(data) == 1
        assert "Invalid identifier format" in data[0]["error"]
        assert "PMC12345" in data[0]["error"]
