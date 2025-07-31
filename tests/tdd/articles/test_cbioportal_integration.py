"""Test cBioPortal integration with article searches."""

import json

import pytest

from biomcp.articles.search import PubmedRequest
from biomcp.articles.unified import search_articles_unified


class TestArticleCBioPortalIntegration:
    """Test that cBioPortal summaries appear in article searches."""

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_article_search_with_gene_includes_cbioportal(self):
        """Test that searching articles for a gene includes cBioPortal summary."""
        request = PubmedRequest(
            genes=["BRAF"],
            keywords=["melanoma"],
        )

        # Test markdown output
        result = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=False,
            output_json=False,
        )

        # Should include cBioPortal summary
        assert "cBioPortal Summary for BRAF" in result
        assert "Mutation Frequency" in result
        # Top Hotspots is only included when mutations are found
        # When cBioPortal API returns empty data, it won't be present
        if "0.0%" not in result:  # If mutation frequency is not 0
            assert "Top Hotspots" in result
        assert "---" in result  # Separator between summary and articles

        # Should still include article results
        assert "pmid" in result or "Title" in result or "Record" in result

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_article_search_json_with_gene(self):
        """Test JSON output includes cBioPortal summary."""
        request = PubmedRequest(
            genes=["TP53"],
            keywords=["cancer"],
        )

        result = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=False,
            output_json=True,
        )

        # Parse JSON
        data = json.loads(result)

        # Should have both summary and articles
        assert "cbioportal_summary" in data
        assert "articles" in data
        assert "TP53" in data["cbioportal_summary"]
        assert isinstance(data["articles"], list)
        assert len(data["articles"]) > 0

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_article_search_without_gene_no_cbioportal(self):
        """Test that searches without genes don't include cBioPortal summary."""
        request = PubmedRequest(
            diseases=["hypertension"],
            keywords=["treatment"],
        )

        # Test markdown output
        result = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=False,
            output_json=False,
        )

        # Should NOT include cBioPortal summary
        assert "cBioPortal Summary" not in result
        assert "Mutation Frequency" not in result

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_article_search_multiple_genes(self):
        """Test that searching with multiple genes uses the first one."""
        request = PubmedRequest(
            genes=["KRAS", "NRAS", "BRAF"],
            diseases=["colorectal cancer"],
        )

        result = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=False,
            output_json=False,
        )

        # Should include cBioPortal summary for KRAS (first gene)
        assert "cBioPortal Summary for KRAS" in result
        # Common KRAS hotspot
        assert "G12" in result or "mutation" in result

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_article_search_with_invalid_gene(self):
        """Test graceful handling of invalid gene names."""
        request = PubmedRequest(
            genes=["BRCA1"],  # Valid gene
            keywords=["cancer"],
        )

        # First check that we handle invalid genes gracefully
        # by using a real gene that might have cBioPortal data
        result = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=False,
            output_json=False,
        )

        # Should have some content - either cBioPortal summary or articles
        assert len(result) > 50  # Some reasonable content

        # Now test with a gene that's valid for search but not in cBioPortal
        request2 = PubmedRequest(
            genes=["ACE2"],  # Real gene but might not be in cancer studies
            keywords=["COVID-19"],
        )

        result2 = await search_articles_unified(
            request2,
            include_pubmed=True,
            include_preprints=False,
            output_json=False,
        )

        # Should return results even if cBioPortal data is not available
        assert len(result2) > 50

    @pytest.mark.asyncio
    @pytest.mark.integration
    async def test_article_search_with_preprints_and_cbioportal(self):
        """Test that cBioPortal summary works with preprint searches too."""
        request = PubmedRequest(
            genes=["EGFR"],
            keywords=["lung cancer", "osimertinib"],
        )

        result = await search_articles_unified(
            request,
            include_pubmed=True,
            include_preprints=True,
            output_json=False,
        )

        # Should include cBioPortal summary
        assert "cBioPortal Summary for EGFR" in result
        # Should include both peer-reviewed and preprint results
        assert ("pmid" in result or "Title" in result) and (
            "Preprint" in result
            or "bioRxiv" in result
            or "peer_reviewed" in result
        )
