"""Unit tests for VZP insurance codebook search functionality."""

import json
from unittest.mock import patch

import pytest

from biomcp.czech.vzp.models import CodebookSearchResult

# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

_MOCK_ENTRY = {
    "typ": "seznam_vykonu",
    "kod": "09513",
    "nazev": "EKG",
    "popis": "Elektrokardiografie",
}

_MOCK_ENTRY_DIAGNOZA = {
    "typ": "diagnoza",
    "kod": "I10",
    "nazev": "Esenciální hypertenze",
    "popis": "Primární hypertenze",
}


def _patch_codebook(codebook_type: str, entries: list[dict]):
    """Helper to patch _fetch_codebook for a specific type."""

    async def _mock(ctype: str = "seznam_vykonu"):
        if ctype == codebook_type:
            return entries
        return []

    return patch(
        "biomcp.czech.vzp.search._fetch_codebook",
        new=_mock,
    )


class TestVzpSearch:
    """Tests for _vzp_search function."""

    @pytest.mark.asyncio
    async def test_search_codebook_by_code(self):
        """Search by code returns matching entry."""
        from biomcp.czech.vzp.search import _vzp_search

        with _patch_codebook("seznam_vykonu", [_MOCK_ENTRY]):
            result = json.loads(
                await _vzp_search("09513", "seznam_vykonu")
            )

        assert result["total"] >= 1
        assert result["results"][0]["code"] == "09513"

    @pytest.mark.asyncio
    async def test_search_codebook_by_name(self):
        """Search by name fragment returns matching entry."""
        from biomcp.czech.vzp.search import _vzp_search

        with _patch_codebook("seznam_vykonu", [_MOCK_ENTRY]):
            result = json.loads(
                await _vzp_search("EKG", "seznam_vykonu")
            )

        assert result["total"] >= 1
        assert result["results"][0]["name"] == "EKG"

    @pytest.mark.asyncio
    async def test_search_with_type_filter(self):
        """codebook_type filter restricts search to that type only."""
        from biomcp.czech.vzp.search import _vzp_search

        async def mock_fetch(ctype="seznam_vykonu"):
            if ctype == "seznam_vykonu":
                return [_MOCK_ENTRY]
            if ctype == "diagnoza":
                return [_MOCK_ENTRY_DIAGNOZA]
            return []

        with patch(
            "biomcp.czech.vzp.search._fetch_codebook",
            new=mock_fetch,
        ):
            result_sv = json.loads(
                await _vzp_search("09513", "seznam_vykonu")
            )
            result_dx = json.loads(
                await _vzp_search("09513", "diagnoza")
            )

        assert result_sv["total"] >= 1
        assert result_dx["total"] == 0

    @pytest.mark.asyncio
    async def test_search_empty_results(self):
        """Query with no matches returns empty results."""
        from biomcp.czech.vzp.search import _vzp_search

        with _patch_codebook("seznam_vykonu", [_MOCK_ENTRY]):
            result = json.loads(
                await _vzp_search("NONEXISTENT99999", "seznam_vykonu")
            )

        assert result["total"] == 0
        assert result["results"] == []

    @pytest.mark.asyncio
    async def test_search_returns_valid_schema(self):
        """Result validates against CodebookSearchResult."""
        from biomcp.czech.vzp.search import _vzp_search

        with _patch_codebook("seznam_vykonu", [_MOCK_ENTRY]):
            raw = await _vzp_search("EKG", "seznam_vykonu")

        parsed = CodebookSearchResult.model_validate_json(raw)
        assert parsed.total >= 1

    @pytest.mark.asyncio
    async def test_search_result_has_required_keys(self):
        """Each result dict has codebook_type, code, and name."""
        from biomcp.czech.vzp.search import _vzp_search

        with _patch_codebook("seznam_vykonu", [_MOCK_ENTRY]):
            result = json.loads(
                await _vzp_search("09513", "seznam_vykonu")
            )

        assert result["total"] >= 1
        entry = result["results"][0]
        for key in ("codebook_type", "code", "name"):
            assert key in entry, f"Missing key '{key}' in result"

    @pytest.mark.asyncio
    async def test_search_without_type_filter_searches_all(self):
        """Omitting codebook_type searches all known types."""
        from biomcp.czech.vzp.search import _vzp_search

        async def mock_fetch(ctype="seznam_vykonu"):
            if ctype == "seznam_vykonu":
                return [_MOCK_ENTRY]
            if ctype == "diagnoza":
                return [_MOCK_ENTRY_DIAGNOZA]
            return []

        with patch(
            "biomcp.czech.vzp.search._fetch_codebook",
            new=mock_fetch,
        ):
            # EKG appears only in seznam_vykonu
            result = json.loads(await _vzp_search("EKG"))

        assert result["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_respects_max_results(self):
        """max_results parameter is respected."""
        from biomcp.czech.vzp.search import _vzp_search

        many = [
            {"kod": f"{i:05d}", "nazev": "EKG variant"}
            for i in range(20)
        ]
        with _patch_codebook("seznam_vykonu", many):
            result = json.loads(
                await _vzp_search("EKG", "seznam_vykonu", max_results=3)
            )

        assert result["total"] <= 3
        assert len(result["results"]) <= 3

    @pytest.mark.asyncio
    async def test_search_diacritics_transparent(self):
        """Diacritics-insensitive query matches accented names."""
        from biomcp.czech.vzp.search import _vzp_search

        with _patch_codebook("diagnoza", [_MOCK_ENTRY_DIAGNOZA]):
            result = json.loads(
                await _vzp_search(
                    "esencialni hypertenze", "diagnoza"
                )
            )

        assert result["total"] >= 1
