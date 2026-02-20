"""Unit tests for NRPZS healthcare provider search functionality."""

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

# ---------------------------------------------------------------------------
# Shared fixtures
# ---------------------------------------------------------------------------

MOCK_SEARCH_RESPONSE = {
    "celkem": 2,
    "strankovani": {"stranka": 1, "velikostStranky": 10},
    "zaznamy": [
        {
            "id": "12345",
            "nazev": "MUDr. Jan Novák",
            "obec": "Praha",
            "odbornosti": ["kardiologie"],
        },
        {
            "id": "67890",
            "nazev": "Nemocnice Brno",
            "obec": "Brno",
            "odbornosti": ["chirurgie", "interna"],
        },
    ],
}

MOCK_EMPTY_RESPONSE = {
    "celkem": 0,
    "strankovani": {"stranka": 1, "velikostStranky": 10},
    "zaznamy": [],
}


def _make_httpx_response(data: dict, status_code: int = 200):
    """Create a mock httpx.Response-like object."""
    response = MagicMock()
    response.status_code = status_code
    response.json.return_value = data
    if status_code >= 400:
        import httpx

        response.raise_for_status.side_effect = (
            httpx.HTTPStatusError(
                f"HTTP {status_code}",
                request=MagicMock(),
                response=response,
            )
        )
    else:
        response.raise_for_status = MagicMock()
    return response


class TestNrpzsSearch:
    """Tests for _nrpzs_search function."""

    @pytest.mark.asyncio
    async def test_search_by_city(self):
        """Search filtered by city returns matching providers."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        city_response = {
            "celkem": 1,
            "strankovani": {"stranka": 1, "velikostStranky": 10},
            "zaznamy": [
                {
                    "id": "12345",
                    "nazev": "MUDr. Jan Novák",
                    "obec": "Praha",
                    "odbornosti": ["kardiologie"],
                }
            ],
        }

        mock_resp = _make_httpx_response(city_response)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(city="Praha")
            )

        assert result["total"] == 1
        assert result["results"][0]["city"] == "Praha"
        assert result["results"][0]["provider_id"] == "12345"

    @pytest.mark.asyncio
    async def test_search_by_specialty(self):
        """Search filtered by specialty returns matching providers."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        mock_resp = _make_httpx_response(MOCK_SEARCH_RESPONSE)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(specialty="kardiologie")
            )

        assert result["total"] == 2
        kardiologie_results = [
            r for r in result["results"]
            if "kardiologie" in r["specialties"]
        ]
        assert len(kardiologie_results) >= 1

    @pytest.mark.asyncio
    async def test_search_by_name(self):
        """Search by name query returns providers matching the name."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        mock_resp = _make_httpx_response(MOCK_SEARCH_RESPONSE)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(query="Novák")
            )

        assert result["total"] >= 1
        names = [r["name"] for r in result["results"]]
        assert any("Nov" in n for n in names)

    @pytest.mark.asyncio
    async def test_search_combined_filters(self):
        """Search with multiple filters combines them correctly."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        combined_response = {
            "celkem": 1,
            "strankovani": {"stranka": 1, "velikostStranky": 10},
            "zaznamy": [
                {
                    "id": "12345",
                    "nazev": "MUDr. Jan Novák",
                    "obec": "Praha",
                    "odbornosti": ["kardiologie"],
                }
            ],
        }

        mock_resp = _make_httpx_response(combined_response)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(
                    query="Novák",
                    city="Praha",
                    specialty="kardiologie",
                )
            )

        assert result["total"] == 1
        assert result["results"][0]["provider_id"] == "12345"

        # Verify all three params were forwarded
        call_kwargs = mock_client.get.call_args
        sent_params = call_kwargs[1].get("params", {})
        assert "nazev" in sent_params
        assert "obec" in sent_params
        assert "odbornost" in sent_params

    @pytest.mark.asyncio
    async def test_search_empty_results(self):
        """Search with no matches returns empty result set."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        mock_resp = _make_httpx_response(MOCK_EMPTY_RESPONSE)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(query="nonexistentxyz")
            )

        assert result["total"] == 0
        assert result["results"] == []

    @pytest.mark.asyncio
    async def test_search_diacritics(self):
        """Search normalises diacritics before sending to API."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        mock_resp = _make_httpx_response(MOCK_SEARCH_RESPONSE)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(query="Novak")  # no diacritics
            )

        # Should still hit the API and return results
        assert result["total"] >= 0
        call_kwargs = mock_client.get.call_args
        sent_params = call_kwargs[1].get("params", {})
        # Normalised value should be lower-case ASCII
        assert sent_params.get("nazev") == "novak"

    @pytest.mark.asyncio
    async def test_search_returns_cached_response(self):
        """Cache hit returns stored response without HTTP call."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        cached_payload = json.dumps(
            {
                "total": 5,
                "page": 1,
                "page_size": 10,
                "results": [],
            }
        )

        with patch(
            "biomcp.czech.nrpzs.search.get_cached_response",
            return_value=cached_payload,
        ), patch("httpx.AsyncClient") as mock_cls:
            result = json.loads(
                await _nrpzs_search(query="Praha")
            )

        assert result["total"] == 5
        mock_cls.assert_not_called()

    @pytest.mark.asyncio
    async def test_search_api_error_returns_error_json(self):
        """API errors are caught and returned as error JSON."""
        import httpx as _httpx

        from biomcp.czech.nrpzs.search import _nrpzs_search

        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(
            side_effect=_httpx.ConnectError("Connection refused")
        )

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(query="test")
            )

        assert "error" in result
        assert result["total"] == 0

    @pytest.mark.asyncio
    async def test_search_pagination_params(self):
        """Pagination parameters are forwarded to the API."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        paginated_response = {
            "celkem": 50,
            "strankovani": {"stranka": 3, "velikostStranky": 5},
            "zaznamy": [],
        }

        mock_resp = _make_httpx_response(paginated_response)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(
                await _nrpzs_search(page=3, page_size=5)
            )

        call_kwargs = mock_client.get.call_args
        sent_params = call_kwargs[1].get("params", {})
        assert sent_params["strana"] == 3
        assert sent_params["velikostStranky"] == 5
        assert result["page"] == 3
        assert result["page_size"] == 5
