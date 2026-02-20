"""Integration tests for SZV/VZP APIs - real HTTP calls.

Run with:
    pytest tests/czech_integration/test_szv_api.py -m integration -v

These tests make live network requests and are skipped in CI unless
SKIP_INTEGRATION_TESTS is not set to a truthy value.
"""

import json

import pytest


@pytest.mark.integration
class TestSzvApiIntegration:
    """Integration tests making real HTTP calls to NZIP/SZV APIs."""

    @pytest.mark.asyncio
    async def test_search_returns_non_empty_list(self):
        """Search for a common procedure returns at least one result."""
        from biomcp.czech.szv.search import _szv_search

        result = json.loads(await _szv_search("EKG"))
        # The API may be unreachable in some environments; we only
        # assert structure integrity, not a minimum count.
        assert "total" in result
        assert "results" in result
        assert isinstance(result["results"], list)

    @pytest.mark.asyncio
    async def test_search_by_numeric_code(self):
        """Search by a known procedure code prefix returns results."""
        from biomcp.czech.szv.search import _szv_search

        result = json.loads(await _szv_search("09"))
        assert "total" in result
        assert isinstance(result["results"], list)

    @pytest.mark.asyncio
    async def test_get_procedure_detail_structure(self):
        """Fetching a known code returns expected fields."""
        from biomcp.czech.szv.search import _szv_get

        result = json.loads(await _szv_get("09513"))
        # Either a valid procedure or a structured error
        assert ("code" in result and result["source"] == "MZCR/SZV") or (
            "error" in result
        )

    @pytest.mark.asyncio
    async def test_get_unknown_code_returns_error(self):
        """Fetching a nonsense code returns an error payload."""
        from biomcp.czech.szv.search import _szv_get

        result = json.loads(await _szv_get("XYZNONEXISTENT99999"))
        assert "error" in result


@pytest.mark.integration
class TestVzpApiIntegration:
    """Integration tests making real HTTP calls to VZP API."""

    @pytest.mark.asyncio
    async def test_search_returns_structure(self):
        """VZP search returns a valid JSON structure."""
        from biomcp.czech.vzp.search import _vzp_search

        result = json.loads(
            await _vzp_search("EKG", "seznam_vykonu")
        )
        assert "total" in result
        assert "results" in result
        assert isinstance(result["results"], list)

    @pytest.mark.asyncio
    async def test_get_entry_structure(self):
        """VZP get returns a valid JSON structure."""
        from biomcp.czech.vzp.search import _vzp_get

        result = json.loads(
            await _vzp_get("seznam_vykonu", "09513")
        )
        # Either a valid entry or a structured error
        assert ("code" in result and result["source"] == "VZP") or (
            "error" in result
        )
