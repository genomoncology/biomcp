"""Tests for SUKL module internal functions to improve coverage."""

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest


class TestSuklSearchInternals:
    """Test internal search functions."""

    def test_matches_query_by_name(self):
        from biomcp.czech.sukl.search import _matches_query

        detail = {"nazev": "NUROFEN 400MG", "doplnekNazvu": "", "kodAtc": "", "nazevDrzitele": ""}
        assert _matches_query(detail, "nurofen") is True

    def test_matches_query_by_atc(self):
        from biomcp.czech.sukl.search import _matches_query

        detail = {"nazev": "", "doplnekNazvu": "", "kodAtc": "M01AE01", "nazevDrzitele": ""}
        assert _matches_query(detail, "m01ae01") is True

    def test_matches_query_by_holder(self):
        from biomcp.czech.sukl.search import _matches_query

        detail = {"nazev": "", "doplnekNazvu": "", "kodAtc": "", "nazevDrzitele": "Reckitt Benckiser"}
        assert _matches_query(detail, "reckitt") is True

    def test_matches_query_no_match(self):
        from biomcp.czech.sukl.search import _matches_query

        detail = {"nazev": "ABC", "doplnekNazvu": "", "kodAtc": "", "nazevDrzitele": ""}
        assert _matches_query(detail, "xyz") is False

    def test_matches_query_none_detail(self):
        from biomcp.czech.sukl.search import _matches_query

        assert _matches_query(None, "test") is False

    def test_detail_to_summary(self):
        from biomcp.czech.sukl.search import _detail_to_summary

        detail = {
            "kodSukl": "0000123",
            "nazev": "NUROFEN",
            "kodAtc": "M01AE01",
            "nazevFormy": "tableta",
        }
        s = _detail_to_summary(detail)
        assert s["sukl_code"] == "0000123"
        assert s["name"] == "NUROFEN"
        assert s["atc_code"] == "M01AE01"


class TestSuklGetterInternals:
    """Test internal getter functions."""

    def test_composition_to_substances(self):
        from biomcp.czech.sukl.getter import (
            _composition_to_substances,
        )

        comp = [
            {
                "nazevLatky": "IBUPROFENUM",
                "mnozstvi": "400",
                "jednotka": "MG",
            }
        ]
        result = _composition_to_substances(comp)
        assert len(result) == 1
        assert result[0]["name"] == "IBUPROFENUM"
        assert result[0]["strength"] == "400 MG"

    def test_composition_to_substances_empty(self):
        from biomcp.czech.sukl.getter import (
            _composition_to_substances,
        )

        assert _composition_to_substances([]) == []

    def test_composition_no_amount(self):
        from biomcp.czech.sukl.getter import (
            _composition_to_substances,
        )

        comp = [{"nazevLatky": "TEST", "mnozstvi": "", "jednotka": ""}]
        result = _composition_to_substances(comp)
        assert result[0]["strength"] is None

    def test_build_doc_url(self):
        from biomcp.czech.sukl.getter import _build_doc_url

        url = _build_doc_url("0000123", "spc")
        assert "0000123" in url
        assert "spc" in url

    @pytest.mark.asyncio
    async def test_fetch_doc_metadata_404(self):
        from biomcp.czech.sukl.getter import (
            _fetch_doc_metadata,
        )

        mock_resp = MagicMock()
        mock_resp.status_code = 404

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(
            return_value=False
        )

        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.getter.httpx.AsyncClient",
            return_value=mock_client,
        ):
            result = await _fetch_doc_metadata("9999999")
            assert result == []

    @pytest.mark.asyncio
    async def test_fetch_composition_404(self):
        from biomcp.czech.sukl.getter import (
            _fetch_composition,
        )

        mock_resp = MagicMock()
        mock_resp.status_code = 404

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(
            return_value=False
        )

        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.getter.httpx.AsyncClient",
            return_value=mock_client,
        ):
            result = await _fetch_composition("9999999")
            assert result == []


class TestSuklAvailabilityInternals:
    """Test internal availability functions."""

    @pytest.mark.asyncio
    async def test_check_distribution_404(self):
        from biomcp.czech.sukl.availability import (
            _check_distribution,
        )

        mock_resp = MagicMock()
        mock_resp.status_code = 404

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(
            return_value=False
        )

        with patch(
            "biomcp.czech.sukl.availability.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.availability.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.availability.cache_response",
        ):
            result = await _check_distribution("9999999")
            assert result == "unavailable"

    @pytest.mark.asyncio
    async def test_check_distribution_success(self):
        from biomcp.czech.sukl.availability import (
            _check_distribution,
        )

        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.is_success = True

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(
            return_value=False
        )

        with patch(
            "biomcp.czech.sukl.availability.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.availability.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.availability.cache_response",
        ):
            result = await _check_distribution("0000123")
            assert result == "available"

    @pytest.mark.asyncio
    async def test_check_distribution_cached(self):
        from biomcp.czech.sukl.availability import (
            _check_distribution,
        )

        with patch(
            "biomcp.czech.sukl.availability.get_cached_response",
            return_value=json.dumps(
                {"_status": "limited"}
            ),
        ):
            result = await _check_distribution("0000123")
            assert result == "limited"

    @pytest.mark.asyncio
    async def test_fetch_drug_detail_cached(self):
        from biomcp.czech.sukl.availability import (
            _fetch_drug_detail,
        )

        cached_data = {"kodSukl": "0000123", "nazev": "Test"}
        with patch(
            "biomcp.czech.sukl.availability.get_cached_response",
            return_value=json.dumps(cached_data),
        ):
            result = await _fetch_drug_detail("0000123")
            assert result["kodSukl"] == "0000123"
