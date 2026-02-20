"""Unit tests for SZV health procedure search functionality."""

import json
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.czech.szv.models import ProcedureSearchResult

# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

_MOCK_PROCEDURE = {
    "kod": "09513",
    "nazev": "EKG 12ti svodové",
    "body": 113,
    "cas": 10,
    "skupina": "09",
}

_MOCK_PROCEDURE_WITH_DIACRITICS = {
    "kod": "12345",
    "nazev": "Elektrokardiografické vyšetření",
    "body": 200,
    "cas": 15,
    "skupina": "12",
}


class TestSzvSearch:
    """Tests for _szv_search function."""

    @pytest.mark.asyncio
    async def test_search_by_code(self):
        """Search by procedure code returns matching results."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE],
        ):
            result = json.loads(await _szv_search("09513"))

        assert result["total"] >= 1
        assert len(result["results"]) >= 1
        assert result["results"][0]["code"] == "09513"

    @pytest.mark.asyncio
    async def test_search_by_name(self):
        """Search by procedure name returns matching results."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE],
        ):
            result = json.loads(await _szv_search("EKG"))

        assert result["total"] >= 1
        assert result["results"][0]["name"] == "EKG 12ti svodové"

    @pytest.mark.asyncio
    async def test_search_by_name_partial_match(self):
        """Partial name match returns the procedure."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE],
        ):
            result = json.loads(await _szv_search("svodove"))

        assert result["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_empty_results(self):
        """Search with no matches returns an empty result set."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE],
        ):
            result = json.loads(
                await _szv_search("XYZNONEXISTENT99999")
            )

        assert result["total"] == 0
        assert result["results"] == []

    @pytest.mark.asyncio
    async def test_search_returns_valid_schema(self):
        """Search result validates against ProcedureSearchResult."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE],
        ):
            raw = await _szv_search("EKG")

        parsed = ProcedureSearchResult.model_validate_json(raw)
        assert parsed.total >= 1

    @pytest.mark.asyncio
    async def test_search_result_has_required_keys(self):
        """Each result dict contains code, name, point_value, category."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE],
        ):
            result = json.loads(await _szv_search("09513"))

        assert result["total"] >= 1
        entry = result["results"][0]
        for key in ("code", "name", "point_value", "category"):
            assert key in entry, f"Missing key '{key}' in result"

    @pytest.mark.asyncio
    async def test_search_diacritics_transparent(self):
        """Diacritics-insensitive matching works on both sides."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_MOCK_PROCEDURE_WITH_DIACRITICS],
        ):
            # Query without diacritics should match name with diacritics
            result = json.loads(
                await _szv_search("elektrokardiograficke vysetreni")
            )

        assert result["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_respects_max_results(self):
        """max_results parameter limits the number of returned entries."""
        from biomcp.czech.szv.search import _szv_search

        many = [
            {"kod": f"0{i:04d}", "nazev": "EKG test", "body": i}
            for i in range(20)
        ]
        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=many,
        ):
            result = json.loads(
                await _szv_search("EKG", max_results=5)
            )

        assert result["total"] <= 5
        assert len(result["results"]) <= 5

    @pytest.mark.asyncio
    async def test_search_api_error_returns_error_payload(self):
        """API errors return a JSON payload with an 'error' key."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            side_effect=Exception("Network failure"),
        ):
            result = json.loads(await _szv_search("EKG"))

        assert "error" in result

    @pytest.mark.asyncio
    async def test_search_empty_list_returns_zero(self):
        """Empty procedure list returns zero results."""
        from biomcp.czech.szv.search import _szv_search

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[],
        ):
            result = json.loads(await _szv_search("EKG"))

        assert result["total"] == 0
        assert result["results"] == []
