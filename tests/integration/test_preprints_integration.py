"""Integration tests for preprint search functionality."""

import asyncio

import pytest

from biomcp.articles.preprints import (
    BiorxivClient,
    EuropePMCClient,
    PreprintSearcher,
)
from biomcp.articles.search import PubmedRequest
from biomcp.core import PublicationState


class TestBiorxivIntegration:
    """Integration tests for bioRxiv API."""

    @pytest.mark.asyncio
    async def test_biorxiv_real_search(self):
        """Test real bioRxiv API search."""
        client = BiorxivClient()

        # Try multiple search terms to find one with results
        search_terms = ["cancer", "gene", "cell", "protein", "RNA", "DNA"]
        results = []
        successful_term = None

        for term in search_terms:
            results = await client.search(term)
            if len(results) > 0:
                successful_term = term
                break

        # If no results with any term, the API might be down or have no recent articles
        if len(results) == 0:
            pytest.skip(
                "No results found with any search term - API may be down or have no matching recent articles"
            )

        # Check the structure of results
        first_result = results[0]
        assert first_result.doi is not None
        assert first_result.title is not None
        assert first_result.publication_state == PublicationState.PREPRINT
        assert "preprint" in first_result.journal.lower()

        print(
            f"Found {len(results)} bioRxiv results for term '{successful_term}'"
        )
        print(f"First result: {first_result.title}")


class TestEuropePMCIntegration:
    """Integration tests for Europe PMC API."""

    @pytest.mark.asyncio
    async def test_europe_pmc_real_search(self):
        """Test real Europe PMC API search for preprints."""
        client = EuropePMCClient()

        # Try multiple search terms to find one with results
        search_terms = [
            "cancer",
            "gene",
            "cell",
            "protein",
            "SARS-CoV-2",
            "COVID",
        ]
        results = []
        successful_term = None

        for term in search_terms:
            results = await client.search(term)
            if len(results) > 0:
                successful_term = term
                break

        # If no results with any term, the API might be down
        if len(results) == 0:
            pytest.skip(
                "No results found with any search term - Europe PMC API may be down"
            )

        # Check the structure
        first_result = results[0]
        assert first_result.title is not None
        assert first_result.publication_state == PublicationState.PREPRINT

        print(
            f"Found {len(results)} Europe PMC preprint results for term '{successful_term}'"
        )
        print(f"First result: {first_result.title}")
        if first_result.doi:
            print(f"DOI: {first_result.doi}")


class TestPreprintSearcherIntegration:
    """Integration tests for combined preprint search."""

    @pytest.mark.asyncio
    async def test_combined_search_real(self):
        """Test searching across both preprint sources."""
        searcher = PreprintSearcher()

        # Try different search combinations
        search_configs = [
            {"genes": ["TP53"], "diseases": ["cancer"]},
            {"keywords": ["protein", "structure"]},
            {"genes": ["BRAF"], "diseases": ["melanoma"]},
            {"keywords": ["gene", "expression"]},
        ]

        response = None
        successful_config = None

        for config in search_configs:
            request = PubmedRequest(**config)
            response = await searcher.search(request)
            if response.count > 0:
                successful_config = config
                break

        print(f"Total results: {response.count if response else 0}")

        # Check if we got any results
        if response and response.count > 0:
            # Check result structure
            first = response.results[0]
            assert first.title is not None
            assert first.publication_state == PublicationState.PREPRINT

            print(f"Successful search config: {successful_config}")
            print(f"First result: {first.title}")
            print(f"Date: {first.date}")
            print(f"Journal: {first.journal}")
        else:
            pytest.skip(
                "No results found with any search configuration - APIs may be down"
            )


if __name__ == "__main__":
    # Run the tests directly
    asyncio.run(TestBiorxivIntegration().test_biorxiv_real_search())
    print("\n" + "=" * 50 + "\n")
    asyncio.run(TestEuropePMCIntegration().test_europe_pmc_real_search())
    print("\n" + "=" * 50 + "\n")
    asyncio.run(TestPreprintSearcherIntegration().test_combined_search_real())
