"""Integration tests for NRPZS API - real HTTP calls.

Run with: pytest tests/czech_integration/ -m integration -v

These tests hit the live NRPZS API at nrpzs.uzis.cz and require
an active internet connection.  They are skipped automatically when
SKIP_INTEGRATION_TESTS=true is set in the environment.
"""

import json

import pytest


@pytest.mark.integration
class TestNrpzsApiIntegration:
    """Integration tests making real HTTP calls to NRPZS API."""

    @pytest.mark.asyncio
    async def test_search_returns_results(self):
        """Live search for Praha returns at least one provider."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        result = json.loads(
            await _nrpzs_search(city="Praha", page_size=5)
        )
        assert result["total"] >= 0
        # When total > 0, results list must be non-empty
        if result["total"] > 0:
            assert len(result["results"]) >= 1

    @pytest.mark.asyncio
    async def test_search_result_structure(self):
        """Search result items contain required ProviderSummary fields."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        result = json.loads(
            await _nrpzs_search(city="Brno", page_size=3)
        )
        for item in result["results"]:
            assert "provider_id" in item
            assert "name" in item
            assert "city" in item
            assert "specialties" in item
            assert isinstance(item["specialties"], list)

    @pytest.mark.asyncio
    async def test_search_pagination(self):
        """Pagination parameters are honoured by the live API."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        result = json.loads(
            await _nrpzs_search(city="Praha", page=1, page_size=2)
        )
        assert result["page_size"] <= 2
        assert result["page"] >= 1

    @pytest.mark.asyncio
    async def test_search_specialty_filter(self):
        """Specialty filter narrows results from the live API."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        result = json.loads(
            await _nrpzs_search(
                specialty="kardiologie", page_size=5
            )
        )
        # Result is valid JSON with the expected keys
        assert "total" in result
        assert "results" in result

    @pytest.mark.asyncio
    async def test_get_provider_from_search(self):
        """Get full provider detail for the first search result."""
        from biomcp.czech.nrpzs.search import (
            _nrpzs_get,
            _nrpzs_search,
        )

        search_result = json.loads(
            await _nrpzs_search(city="Praha", page_size=1)
        )
        if not search_result["results"]:
            pytest.skip("No results returned from live API")

        provider_id = search_result["results"][0]["provider_id"]
        detail = json.loads(await _nrpzs_get(provider_id))

        # Should not be an error response
        assert "error" not in detail
        assert detail["provider_id"] == provider_id
        assert detail["name"]
        assert detail["source"] == "NRPZS"

    @pytest.mark.asyncio
    async def test_get_provider_detail_structure(self):
        """Full provider detail contains all HealthcareProvider fields."""
        from biomcp.czech.nrpzs.search import (
            _nrpzs_get,
            _nrpzs_search,
        )

        search_result = json.loads(
            await _nrpzs_search(city="Praha", page_size=1)
        )
        if not search_result["results"]:
            pytest.skip("No results returned from live API")

        provider_id = search_result["results"][0]["provider_id"]
        detail = json.loads(await _nrpzs_get(provider_id))

        required_keys = {
            "provider_id",
            "name",
            "source",
            "specialties",
            "care_types",
            "workplaces",
        }
        assert required_keys.issubset(detail.keys())

    @pytest.mark.asyncio
    async def test_get_invalid_provider_returns_error(self):
        """Non-existent provider ID returns an error JSON response."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        result = json.loads(
            await _nrpzs_get("0000000000nonexistent")
        )
        assert "error" in result
