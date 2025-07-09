"""Comprehensive tests for the unified router module."""

import json
from unittest.mock import patch

import pytest

from biomcp.exceptions import (
    InvalidDomainError,
    InvalidParameterError,
    QueryParsingError,
    SearchExecutionError,
)
from biomcp.router import fetch, format_results, search


class TestFormatResults:
    """Test the format_results function."""

    def test_format_article_results(self):
        """Test formatting article results."""
        results = [
            {
                "pmid": "12345",
                "title": "Test Article",
                "abstract": "This is a test abstract",
                # Note: url in input is ignored, always generates PubMed URL
            }
        ]

        # Mock thinking tracker to prevent reminder
        with patch("biomcp.router.get_thinking_reminder", return_value=""):
            formatted = format_results(results, "article", 1, 10, 1)

        assert "results" in formatted
        assert len(formatted["results"]) == 1
        result = formatted["results"][0]
        assert result["id"] == "12345"
        assert result["title"] == "Test Article"
        assert "test abstract" in result["text"]
        assert result["url"] == "https://pubmed.ncbi.nlm.nih.gov/12345/"

    def test_format_trial_results_api_v2(self):
        """Test formatting trial results with API v2 structure."""
        results = [
            {
                "protocolSection": {
                    "identificationModule": {
                        "nctId": "NCT12345",
                        "briefTitle": "Test Trial",
                    },
                    "descriptionModule": {
                        "briefSummary": "This is a test trial summary"
                    },
                    "statusModule": {"overallStatus": "RECRUITING"},
                    "designModule": {"phases": ["PHASE3"]},
                }
            }
        ]

        # Mock thinking tracker to prevent reminder
        with patch("biomcp.router.get_thinking_reminder", return_value=""):
            formatted = format_results(results, "trial", 1, 10, 1)

        assert "results" in formatted
        assert len(formatted["results"]) == 1
        result = formatted["results"][0]
        assert result["id"] == "NCT12345"
        assert result["title"] == "Test Trial"
        assert "test trial summary" in result["text"]
        assert "NCT12345" in result["url"]

    def test_format_trial_results_legacy(self):
        """Test formatting trial results with legacy structure."""
        results = [
            {
                "NCT Number": "NCT67890",
                "Study Title": "Legacy Trial",
                "Brief Summary": "Legacy trial summary",
                "Study Status": "COMPLETED",
                "Phases": "Phase 2",
            }
        ]

        # Mock thinking tracker to prevent reminder
        with patch("biomcp.router.get_thinking_reminder", return_value=""):
            formatted = format_results(results, "trial", 1, 10, 1)

        assert "results" in formatted
        assert len(formatted["results"]) == 1
        result = formatted["results"][0]
        assert result["id"] == "NCT67890"
        assert result["title"] == "Legacy Trial"
        assert "Legacy trial summary" in result["text"]

    def test_format_variant_results(self):
        """Test formatting variant results."""
        results = [
            {
                "_id": "chr7:g.140453136A>T",
                "dbsnp": {"rsid": "rs121913529"},
                "dbnsfp": {"genename": "BRAF"},
                "clinvar": {"rcv": {"clinical_significance": "Pathogenic"}},
            }
        ]

        # Mock thinking tracker to prevent reminder
        with patch("biomcp.router.get_thinking_reminder", return_value=""):
            formatted = format_results(results, "variant", 1, 10, 1)

        assert "results" in formatted
        assert len(formatted["results"]) == 1
        result = formatted["results"][0]
        assert result["id"] == "chr7:g.140453136A>T"
        assert "BRAF" in result["title"]
        assert "Pathogenic" in result["text"]
        assert "rs121913529" in result["url"]

    def test_format_results_invalid_domain(self):
        """Test format_results with invalid domain."""
        with pytest.raises(InvalidDomainError) as exc_info:
            format_results([], "invalid_domain", 1, 10, 0)

        assert "Unknown domain: invalid_domain" in str(exc_info.value)

    def test_format_results_malformed_data(self):
        """Test format_results handles malformed data gracefully."""
        results = [
            {"title": "Good Article", "pmid": "123"},
            None,  # Malformed - will be skipped
            {
                "invalid": "data"
            },  # Missing required fields but won't fail (treated as preprint)
        ]

        # Mock thinking tracker to prevent reminder
        with patch("biomcp.router.get_thinking_reminder", return_value=""):
            formatted = format_results(results, "article", 1, 10, 3)

        # Should skip None but include the third (treated as preprint with empty fields)
        assert len(formatted["results"]) == 2
        assert formatted["results"][0]["id"] == "123"
        assert formatted["results"][1]["id"] == ""  # Empty ID for invalid data


@pytest.mark.asyncio
class TestSearchFunction:
    """Test the unified search function."""

    async def test_search_article_domain(self):
        """Test search with article domain."""
        mock_result = json.dumps([
            {"pmid": "123", "title": "Test", "abstract": "Abstract"}
        ])

        with patch(
            "biomcp.articles.unified.search_articles_unified"
        ) as mock_search:
            mock_search.return_value = mock_result

            # Mock thinking tracker to prevent reminder
            with patch("biomcp.router.get_thinking_reminder", return_value=""):
                result = await search(
                    query="",
                    domain="article",
                    genes="BRAF",
                    diseases=["cancer"],
                    page_size=10,
                )

            assert "results" in result
            assert len(result["results"]) == 1
            assert result["results"][0]["id"] == "123"

    async def test_search_trial_domain(self):
        """Test search with trial domain."""
        mock_result = json.dumps({
            "studies": [
                {
                    "protocolSection": {
                        "identificationModule": {"nctId": "NCT123"},
                    }
                }
            ]
        })

        with patch("biomcp.trials.search.search_trials") as mock_search:
            mock_search.return_value = mock_result

            # Mock thinking tracker to prevent reminder
            with patch("biomcp.router.get_thinking_reminder", return_value=""):
                result = await search(
                    query="",
                    domain="trial",
                    conditions=["cancer"],
                    phase="Phase 3",
                    page_size=20,
                )

            assert "results" in result
            mock_search.assert_called_once()

    async def test_search_variant_domain(self):
        """Test search with variant domain."""
        mock_result = json.dumps([
            {"_id": "rs123", "gene": {"symbol": "BRAF"}}
        ])

        with patch("biomcp.variants.search.search_variants") as mock_search:
            mock_search.return_value = mock_result

            # Mock thinking tracker to prevent reminder
            with patch("biomcp.router.get_thinking_reminder", return_value=""):
                result = await search(
                    query="",
                    domain="variant",
                    genes="BRAF",
                    significance="pathogenic",
                    page_size=10,
                )

            assert "results" in result
            assert len(result["results"]) == 1

    async def test_search_unified_query(self):
        """Test search with unified query language."""
        with patch("biomcp.router._unified_search") as mock_unified:
            mock_unified.return_value = {
                "results": [{"id": "1", "title": "Test"}]
            }

            result = await search(
                query="gene:BRAF AND disease:cancer",
                max_results_per_domain=20,
            )

            assert "results" in result
            mock_unified.assert_called_once_with(
                query="gene:BRAF AND disease:cancer",
                max_results_per_domain=20,
                domains=None,
                explain_query=False,
            )

    async def test_search_no_domain_or_query(self):
        """Test search without domain or query raises error."""
        with pytest.raises(InvalidParameterError) as exc_info:
            await search(query="")

        assert "query or domain" in str(exc_info.value)

    async def test_search_invalid_domain(self):
        """Test search with invalid domain."""
        with pytest.raises(InvalidDomainError):
            await search(query="", domain="invalid_domain")

    async def test_search_get_schema(self):
        """Test search with get_schema flag."""
        result = await search(query="", get_schema=True)

        assert "domains" in result
        assert "cross_domain_fields" in result
        assert "domain_fields" in result
        assert isinstance(result["cross_domain_fields"], dict)

    async def test_search_pagination_validation(self):
        """Test search with invalid pagination parameters."""
        with pytest.raises(InvalidParameterError) as exc_info:
            await search(
                query="",
                domain="article",
                page=0,  # Invalid - must be >= 1
                page_size=10,
            )

        assert "page" in str(exc_info.value)

    async def test_search_parameter_parsing(self):
        """Test parameter parsing for list inputs."""
        mock_result = json.dumps([])

        with patch(
            "biomcp.articles.unified.search_articles_unified"
        ) as mock_search:
            mock_search.return_value = mock_result

            # Test with JSON array string
            await search(
                query="",
                domain="article",
                genes='["BRAF", "KRAS"]',
                diseases="cancer,melanoma",  # Comma-separated
            )

            # Check the request was parsed correctly
            call_args = mock_search.call_args[0][0]
            assert call_args.genes == ["BRAF", "KRAS"]
            assert call_args.diseases == ["cancer", "melanoma"]


@pytest.mark.asyncio
class TestFetchFunction:
    """Test the unified fetch function."""

    async def test_fetch_article(self):
        """Test fetching article details."""
        mock_result = json.dumps([
            {
                "pmid": 12345,
                "title": "Test Article",
                "abstract": "Full abstract",
                "full_text": "Full text content",
            }
        ])

        with patch("biomcp.articles.fetch.fetch_articles") as mock_fetch:
            mock_fetch.return_value = mock_result

            result = await fetch(
                domain="article",
                id="12345",
            )

            assert result["id"] == "12345"
            assert result["title"] == "Test Article"
            assert result["text"] == "Full text content"
            assert "metadata" in result

    async def test_fetch_article_invalid_pmid(self):
        """Test fetching article with invalid identifier."""
        result = await fetch(domain="article", id="not_a_number")

        # Should return an error since "not_a_number" is neither a valid PMID nor DOI
        assert "error" in result
        assert "Invalid identifier format" in result["error"]
        assert "not_a_number" in result["error"]

    async def test_fetch_trial_all_sections(self):
        """Test fetching trial with all sections."""
        mock_protocol = json.dumps({
            "title": "Test Trial",
            "nct_id": "NCT123",
            "brief_summary": "Summary",
        })
        mock_locations = json.dumps({"locations": [{"city": "Boston"}]})
        mock_outcomes = json.dumps({
            "outcomes": {"primary_outcomes": ["Outcome1"]}
        })
        mock_references = json.dumps({"references": [{"pmid": "456"}]})

        with (
            patch("biomcp.trials.getter._trial_protocol") as mock_p,
            patch("biomcp.trials.getter._trial_locations") as mock_l,
            patch("biomcp.trials.getter._trial_outcomes") as mock_o,
            patch("biomcp.trials.getter._trial_references") as mock_r,
        ):
            mock_p.return_value = mock_protocol
            mock_l.return_value = mock_locations
            mock_o.return_value = mock_outcomes
            mock_r.return_value = mock_references

            result = await fetch(domain="trial", id="NCT123", detail="all")

            assert result["id"] == "NCT123"
            assert "metadata" in result
            assert "locations" in result["metadata"]
            assert "outcomes" in result["metadata"]
            assert "references" in result["metadata"]

    async def test_fetch_trial_invalid_detail(self):
        """Test fetching trial with invalid detail parameter."""
        with pytest.raises(InvalidParameterError) as exc_info:
            await fetch(
                domain="trial",
                id="NCT123",
                detail="invalid_section",
            )

        assert "one of:" in str(exc_info.value)

    async def test_fetch_variant(self):
        """Test fetching variant details."""
        mock_result = json.dumps([
            {
                "_id": "rs123",
                "gene": {"symbol": "BRAF"},
                "clinvar": {"clinical_significance": "Pathogenic"},
                "tcga": {"cancer_types": {}},
                "external_links": {"dbSNP": "https://example.com"},
            }
        ])

        with patch("biomcp.variants.getter.get_variant") as mock_get:
            mock_get.return_value = mock_result

            result = await fetch(domain="variant", id="rs123")

            assert result["id"] == "rs123"
            assert "TCGA Data: Available" in result["text"]
            assert "external_links" in result["metadata"]

    async def test_fetch_variant_list_response(self):
        """Test fetching variant when API returns list."""
        mock_result = json.dumps([
            {"_id": "rs123", "gene": {"symbol": "BRAF"}}
        ])

        with patch("biomcp.variants.getter.get_variant") as mock_get:
            mock_get.return_value = mock_result

            result = await fetch(domain="variant", id="rs123")

            assert result["id"] == "rs123"

    async def test_fetch_invalid_domain(self):
        """Test fetch with invalid domain."""
        with pytest.raises(InvalidDomainError):
            await fetch(domain="invalid", id="123")

    async def test_fetch_error_handling(self):
        """Test fetch error handling."""
        with patch("biomcp.articles.fetch.fetch_articles") as mock_fetch:
            mock_fetch.side_effect = Exception("API Error")

            with pytest.raises(SearchExecutionError) as exc_info:
                await fetch(domain="article", id="123")

            assert "Failed to execute search" in str(exc_info.value)

    async def test_fetch_domain_auto_detection_pmid(self):
        """Test domain auto-detection for PMID."""
        with patch("biomcp.articles.fetch._article_details") as mock_fetch:
            mock_fetch.return_value = json.dumps([
                {"pmid": "12345", "title": "Test"}
            ])

            # Numeric ID should auto-detect as article
            result = await fetch(id="12345")
            assert result["id"] == "12345"
            mock_fetch.assert_called_once()

    async def test_fetch_domain_auto_detection_nct(self):
        """Test domain auto-detection for NCT ID."""
        with patch("biomcp.trials.getter.get_trial") as mock_get:
            mock_get.return_value = json.dumps({
                "protocolSection": {
                    "identificationModule": {"briefTitle": "Test Trial"}
                }
            })

            # NCT ID should auto-detect as trial
            result = await fetch(id="NCT12345")
            assert "NCT12345" in result["url"]
            mock_get.assert_called()

    async def test_fetch_domain_auto_detection_doi(self):
        """Test domain auto-detection for DOI."""
        with patch("biomcp.articles.fetch._article_details") as mock_fetch:
            mock_fetch.return_value = json.dumps([
                {"doi": "10.1038/nature12345", "title": "Test"}
            ])

            # DOI should auto-detect as article
            await fetch(id="10.1038/nature12345")
            mock_fetch.assert_called_once()

    async def test_fetch_domain_auto_detection_variant(self):
        """Test domain auto-detection for variant IDs."""
        with patch("biomcp.variants.getter.get_variant") as mock_get:
            mock_get.return_value = json.dumps([{"_id": "rs12345"}])

            # rsID should auto-detect as variant
            await fetch(id="rs12345")
            mock_get.assert_called_once()

        # Test HGVS notation
        with patch("biomcp.variants.getter.get_variant") as mock_get:
            mock_get.return_value = json.dumps([
                {"_id": "chr7:g.140453136A>T"}
            ])

            await fetch(id="chr7:g.140453136A>T")
            mock_get.assert_called_once()


@pytest.mark.asyncio
class TestUnifiedSearch:
    """Test the _unified_search internal function."""

    async def test_unified_search_explain_query(self):
        """Test unified search with explain_query flag."""
        from biomcp.router import _unified_search

        result = await _unified_search(
            query="gene:BRAF AND disease:cancer", explain_query=True
        )

        assert "original_query" in result
        assert "parsed_structure" in result
        assert "routing_plan" in result
        assert "schema" in result

    async def test_unified_search_execution(self):
        """Test unified search normal execution."""
        from biomcp.router import _unified_search

        with patch("biomcp.query_router.execute_routing_plan") as mock_execute:
            mock_execute.return_value = {
                "articles": json.dumps([{"pmid": "123", "title": "Article 1"}])
            }

            result = await _unified_search(
                query="gene:BRAF", max_results_per_domain=10
            )

            assert "results" in result
            assert isinstance(result["results"], list)

    async def test_unified_search_parse_error(self):
        """Test unified search with invalid query."""
        from biomcp.router import _unified_search

        with patch("biomcp.query_parser.QueryParser.parse") as mock_parse:
            mock_parse.side_effect = Exception("Parse error")

            with pytest.raises(QueryParsingError):
                await _unified_search(
                    query="invalid::query", max_results_per_domain=10
                )
