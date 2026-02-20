"""Tests for SUKL HTTP fetch paths to improve coverage."""

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest


class TestFetchDrugList:
    """Cover _fetch_drug_list in search.py."""

    @pytest.mark.asyncio
    async def test_fetch_cached(self):
        from biomcp.czech.sukl.search import _fetch_drug_list

        with patch(
            "biomcp.czech.sukl.search.get_cached_response",
            return_value=json.dumps(["001", "002"]),
        ):
            result = await _fetch_drug_list()
            assert result == ["001", "002"]

    @pytest.mark.asyncio
    async def test_fetch_from_api(self):
        from biomcp.czech.sukl.search import _fetch_drug_list

        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = ["001", "002"]
        mock_resp.raise_for_status = MagicMock()

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.search.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.search.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.search.cache_response",
        ):
            result = await _fetch_drug_list()
            assert result == ["001", "002"]


class TestFetchDrugDetailSearch:
    """Cover _fetch_drug_detail in search.py."""

    @pytest.mark.asyncio
    async def test_fetch_cached(self):
        from biomcp.czech.sukl.search import (
            _fetch_drug_detail,
        )

        data = {"kodSukl": "001", "nazev": "Test"}
        with patch(
            "biomcp.czech.sukl.search.get_cached_response",
            return_value=json.dumps(data),
        ):
            result = await _fetch_drug_detail("001")
            assert result["kodSukl"] == "001"

    @pytest.mark.asyncio
    async def test_fetch_from_api(self):
        from biomcp.czech.sukl.search import (
            _fetch_drug_detail,
        )

        data = {"kodSukl": "001", "nazev": "Test"}
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = data
        mock_resp.raise_for_status = MagicMock()

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.search.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.search.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.search.cache_response",
        ):
            result = await _fetch_drug_detail("001")
            assert result["kodSukl"] == "001"

    @pytest.mark.asyncio
    async def test_fetch_404(self):
        from biomcp.czech.sukl.search import (
            _fetch_drug_detail,
        )

        mock_resp = MagicMock()
        mock_resp.status_code = 404

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.search.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.search.httpx.AsyncClient",
            return_value=mock_client,
        ):
            result = await _fetch_drug_detail("999")
            assert result is None


class TestFetchDrugDetailGetter:
    """Cover _fetch_drug_detail in getter.py."""

    @pytest.mark.asyncio
    async def test_fetch_cached(self):
        from biomcp.czech.sukl.getter import (
            _fetch_drug_detail,
        )

        data = {"kodSukl": "001", "nazev": "Test"}
        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=json.dumps(data),
        ):
            result = await _fetch_drug_detail("001")
            assert result["kodSukl"] == "001"

    @pytest.mark.asyncio
    async def test_fetch_from_api(self):
        from biomcp.czech.sukl.getter import (
            _fetch_drug_detail,
        )

        data = {"kodSukl": "001", "nazev": "Test"}
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = data
        mock_resp.raise_for_status = MagicMock()

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.getter.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.getter.cache_response",
        ):
            result = await _fetch_drug_detail("001")
            assert result["kodSukl"] == "001"

    @pytest.mark.asyncio
    async def test_fetch_404(self):
        from biomcp.czech.sukl.getter import (
            _fetch_drug_detail,
        )

        mock_resp = MagicMock()
        mock_resp.status_code = 404

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.getter.httpx.AsyncClient",
            return_value=mock_client,
        ):
            result = await _fetch_drug_detail("999")
            assert result is None

    @pytest.mark.asyncio
    async def test_fetch_composition_from_api(self):
        from biomcp.czech.sukl.getter import (
            _fetch_composition,
        )

        data = [{"nazevLatky": "TEST", "mnozstvi": "10", "jednotka": "MG"}]
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = data
        mock_resp.raise_for_status = MagicMock()

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.getter.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.getter.cache_response",
        ):
            result = await _fetch_composition("001")
            assert len(result) == 1

    @pytest.mark.asyncio
    async def test_fetch_doc_metadata_from_api(self):
        from biomcp.czech.sukl.getter import (
            _fetch_doc_metadata,
        )

        data = [{"typ": "spc", "idDokumentu": "D1"}]
        mock_resp = MagicMock()
        mock_resp.status_code = 200
        mock_resp.json.return_value = data
        mock_resp.raise_for_status = MagicMock()

        mock_client = AsyncMock()
        mock_client.get.return_value = mock_resp
        mock_client.__aenter__ = AsyncMock(
            return_value=mock_client
        )
        mock_client.__aexit__ = AsyncMock(return_value=False)

        with patch(
            "biomcp.czech.sukl.getter.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.sukl.getter.httpx.AsyncClient",
            return_value=mock_client,
        ), patch(
            "biomcp.czech.sukl.getter.cache_response",
        ):
            result = await _fetch_doc_metadata("001")
            assert len(result) == 1
