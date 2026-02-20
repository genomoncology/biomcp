"""Unit tests for SZV health procedure detail retrieval."""

import json
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.czech.szv.models import HealthProcedure

# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

_FULL_MOCK_PROCEDURE = {
    "kod": "09513",
    "nazev": "EKG 12ti svodové",
    "body": 113,
    "cas": 10,
    "skupina": "09",
    "skupina_nazev": "Kardiologická vyšetření",
    "odbornosti": ["101", "102"],
    "omezeni_frekvence": "1x per year",
    "materialni_pozadavky": "EKG přístroj",
    "poznamky": "Standardní vyšetření",
}


class TestSzvGetter:
    """Tests for _szv_get function."""

    @pytest.mark.asyncio
    async def test_get_procedure_details(self):
        """Fetching by code returns a full procedure object."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_PROCEDURE,
        ):
            result = json.loads(await _szv_get("09513"))

        assert result["code"] == "09513"
        assert result["name"] == "EKG 12ti svodové"
        assert result["source"] == "MZCR/SZV"

    @pytest.mark.asyncio
    async def test_get_includes_point_value(self):
        """Returned procedure contains point_value."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_PROCEDURE,
        ):
            result = json.loads(await _szv_get("09513"))

        assert result["point_value"] == 113

    @pytest.mark.asyncio
    async def test_get_includes_time_minutes(self):
        """Returned procedure contains time_minutes."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_PROCEDURE,
        ):
            result = json.loads(await _szv_get("09513"))

        assert result["time_minutes"] == 10

    @pytest.mark.asyncio
    async def test_get_includes_specialty_codes(self):
        """Returned procedure includes specialty_codes list."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_PROCEDURE,
        ):
            result = json.loads(await _szv_get("09513"))

        assert isinstance(result["specialty_codes"], list)
        assert "101" in result["specialty_codes"]

    @pytest.mark.asyncio
    async def test_get_invalid_code(self):
        """Unknown code returns a JSON error payload."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=None,
        ), patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[],
        ):
            result = json.loads(await _szv_get("INVALID_CODE"))

        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_falls_back_to_list_scan(self):
        """When detail endpoint returns None, list scan is used."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=None,
        ), patch(
            "biomcp.czech.szv.search._fetch_procedure_list",
            new_callable=AsyncMock,
            return_value=[_FULL_MOCK_PROCEDURE],
        ):
            result = json.loads(await _szv_get("09513"))

        # Should succeed via fallback
        assert result.get("code") == "09513"

    @pytest.mark.asyncio
    async def test_get_validates_against_model(self):
        """Result validates against HealthProcedure Pydantic model."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_PROCEDURE,
        ):
            raw = await _szv_get("09513")

        procedure = HealthProcedure.model_validate_json(raw)
        assert procedure.code == "09513"
        assert procedure.point_value == 113

    @pytest.mark.asyncio
    async def test_get_category_name(self):
        """Returned procedure includes category_name when available."""
        from biomcp.czech.szv.search import _szv_get

        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_PROCEDURE,
        ):
            result = json.loads(await _szv_get("09513"))

        assert result["category_name"] == "Kardiologická vyšetření"

    @pytest.mark.asyncio
    async def test_get_minimal_procedure(self):
        """Minimal API response (code + name only) still succeeds."""
        from biomcp.czech.szv.search import _szv_get

        minimal = {"kod": "00001", "nazev": "Základní vyšetření"}
        with patch(
            "biomcp.czech.szv.search._fetch_procedure_detail",
            new_callable=AsyncMock,
            return_value=minimal,
        ):
            result = json.loads(await _szv_get("00001"))

        assert result["code"] == "00001"
        assert result["point_value"] is None
        assert result["specialty_codes"] == []
