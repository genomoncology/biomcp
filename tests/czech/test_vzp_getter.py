"""Unit tests for VZP insurance codebook entry detail retrieval."""

import json
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.czech.vzp.models import CodebookEntry

# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

_FULL_MOCK_ENTRY = {
    "typ": "seznam_vykonu",
    "kod": "09513",
    "nazev": "EKG",
    "popis": "Elektrokardiografie",
    "platnost_od": "2020-01-01",
    "platnost_do": None,
    "pravidla": ["Jednou za rok", "Vyžaduje odbornost 101"],
    "souvisejici_kody": ["09514", "09515"],
}


class TestVzpGetter:
    """Tests for _vzp_get function."""

    @pytest.mark.asyncio
    async def test_get_entry(self):
        """Fetching by type and code returns a full entry object."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_ENTRY,
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "09513")
            )

        assert result["code"] == "09513"
        assert result["name"] == "EKG"
        assert result["source"] == "VZP"

    @pytest.mark.asyncio
    async def test_get_entry_description(self):
        """Returned entry includes description field."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_ENTRY,
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "09513")
            )

        assert result["description"] == "Elektrokardiografie"

    @pytest.mark.asyncio
    async def test_get_entry_includes_rules(self):
        """Returned entry includes rules list."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_ENTRY,
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "09513")
            )

        assert isinstance(result["rules"], list)
        assert len(result["rules"]) == 2

    @pytest.mark.asyncio
    async def test_get_entry_includes_related_codes(self):
        """Returned entry includes related_codes list."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_ENTRY,
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "09513")
            )

        assert "09514" in result["related_codes"]

    @pytest.mark.asyncio
    async def test_get_invalid_entry(self):
        """Unknown code returns a JSON error payload."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=None,
        ), patch(
            "biomcp.czech.vzp.search._fetch_codebook",
            new_callable=AsyncMock,
            return_value=[],
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "INVALID")
            )

        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_falls_back_to_list_scan(self):
        """When entry endpoint returns None, list scan is used."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=None,
        ), patch(
            "biomcp.czech.vzp.search._fetch_codebook",
            new_callable=AsyncMock,
            return_value=[_FULL_MOCK_ENTRY],
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "09513")
            )

        assert result.get("code") == "09513"

    @pytest.mark.asyncio
    async def test_get_validates_against_model(self):
        """Result validates against CodebookEntry Pydantic model."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_ENTRY,
        ):
            raw = await _vzp_get("seznam_vykonu", "09513")

        entry = CodebookEntry.model_validate_json(raw)
        assert entry.code == "09513"
        assert entry.source == "VZP"
        assert entry.codebook_type == "seznam_vykonu"

    @pytest.mark.asyncio
    async def test_get_codebook_type_preserved(self):
        """codebook_type parameter is preserved in the result."""
        from biomcp.czech.vzp.search import _vzp_get

        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=_FULL_MOCK_ENTRY,
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "09513")
            )

        assert result["codebook_type"] == "seznam_vykonu"

    @pytest.mark.asyncio
    async def test_get_minimal_entry(self):
        """Minimal API response (code + name only) still succeeds."""
        from biomcp.czech.vzp.search import _vzp_get

        minimal = {"kod": "00001", "nazev": "Základní výkon"}
        with patch(
            "biomcp.czech.vzp.search._fetch_entry",
            new_callable=AsyncMock,
            return_value=minimal,
        ):
            result = json.loads(
                await _vzp_get("seznam_vykonu", "00001")
            )

        assert result["code"] == "00001"
        assert result["description"] is None
        assert result["rules"] == []
        assert result["related_codes"] == []
