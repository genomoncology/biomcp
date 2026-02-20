"""Integration tests for SUKL API - real HTTP calls.

Run with: pytest tests/czech_integration/ -m integration -v
"""

import json

import pytest


@pytest.mark.integration
class TestSuklApiIntegration:
    """Integration tests making real HTTP calls to SUKL API."""

    @pytest.mark.asyncio
    async def test_search_ibuprofen(self):
        """Search for a common drug returns results."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        result = json.loads(await _sukl_drug_search("IBUPROFEN"))
        assert result["total"] >= 1
        assert len(result["results"]) >= 1

    @pytest.mark.asyncio
    async def test_get_drug_detail(self):
        """Get drug detail by SUKL code."""
        from biomcp.czech.sukl.search import _fetch_drug_list

        codes = await _fetch_drug_list()
        assert len(codes) > 0

        from biomcp.czech.sukl.getter import _sukl_drug_details

        result = json.loads(await _sukl_drug_details(codes[0]))
        assert "sukl_code" in result
        assert result["source"] == "SUKL"

    @pytest.mark.asyncio
    async def test_availability_check(self):
        """Check availability for a known drug."""
        from biomcp.czech.sukl.search import _fetch_drug_list

        codes = await _fetch_drug_list()
        assert len(codes) > 0

        from biomcp.czech.sukl.availability import (
            _sukl_availability_check,
        )

        result = json.loads(
            await _sukl_availability_check(codes[0])
        )
        assert result["status"] in (
            "available",
            "limited",
            "unavailable",
        )
        assert result["source"] == "SUKL"
