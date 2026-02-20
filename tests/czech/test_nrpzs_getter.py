"""Unit tests for NRPZS healthcare provider getter functionality."""

import json
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

MOCK_PROVIDER_DETAIL = {
    "id": "12345",
    "nazev": "MUDr. Jan Novák",
    "pravniForma": "fyzická osoba",
    "ico": "12345678",
    "ulice": "Hlavní 123",
    "obec": "Praha",
    "psc": "11000",
    "kraj": "Praha",
    "odbornosti": ["kardiologie"],
    "druhyPece": ["ambulantní"],
    "registracniCislo": "REG-12345",
    "pracoviste": [
        {
            "id": "W001",
            "nazev": "Ordinace kardiologie",
            "ulice": "Hlavní 123",
            "obec": "Praha",
            "psc": "11000",
            "kraj": "Praha",
            "odbornosti": ["kardiologie"],
            "telefon": "+420 123 456 789",
            "email": "novak@example.cz",
            "www": None,
        }
    ],
}

MOCK_PROVIDER_MULTI_WORKPLACE = {
    **MOCK_PROVIDER_DETAIL,
    "id": "99999",
    "nazev": "Nemocnice Praha",
    "pracoviste": [
        {
            "id": "WP1",
            "nazev": "Interní oddělení",
            "ulice": "Nemocniční 1",
            "obec": "Praha",
            "psc": "12000",
            "kraj": "Praha",
            "odbornosti": ["interna"],
            "telefon": "+420 111 222 333",
            "email": None,
            "www": "https://nemocnice.cz",
        },
        {
            "id": "WP2",
            "nazev": "Chirurgické oddělení",
            "ulice": "Nemocniční 1",
            "obec": "Praha",
            "psc": "12000",
            "kraj": "Praha",
            "odbornosti": ["chirurgie"],
            "telefon": "+420 444 555 666",
            "email": "chirurgie@nemocnice.cz",
            "www": None,
        },
    ],
}


def _make_httpx_response(data: dict, status_code: int = 200):
    """Create a mock httpx.Response-like object."""
    response = MagicMock()
    response.status_code = status_code
    response.json.return_value = data
    if status_code == 404:
        import httpx

        response.raise_for_status.side_effect = (
            httpx.HTTPStatusError(
                "HTTP 404",
                request=MagicMock(),
                response=response,
            )
        )
    elif status_code >= 400:
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


class TestNrpzsGetter:
    """Tests for _nrpzs_get function."""

    @pytest.mark.asyncio
    async def test_get_provider_details(self):
        """Get provider by ID returns full HealthcareProvider data."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        mock_resp = _make_httpx_response(MOCK_PROVIDER_DETAIL)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("12345"))

        assert result["provider_id"] == "12345"
        assert result["name"] == "MUDr. Jan Novák"
        assert result["legal_form"] == "fyzická osoba"
        assert result["ico"] == "12345678"
        assert result["source"] == "NRPZS"
        assert result["registration_number"] == "REG-12345"
        assert "kardiologie" in result["specialties"]
        assert "ambulantní" in result["care_types"]

    @pytest.mark.asyncio
    async def test_get_provider_address(self):
        """Provider details include correctly mapped address."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        mock_resp = _make_httpx_response(MOCK_PROVIDER_DETAIL)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("12345"))

        address = result["address"]
        assert address is not None
        assert address["street"] == "Hlavní 123"
        assert address["city"] == "Praha"
        assert address["postal_code"] == "11000"
        assert address["region"] == "Praha"

    @pytest.mark.asyncio
    async def test_get_provider_workplaces(self):
        """Provider details include workplace list with contact info."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        mock_resp = _make_httpx_response(MOCK_PROVIDER_MULTI_WORKPLACE)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("99999"))

        workplaces = result["workplaces"]
        assert len(workplaces) == 2

        wp1 = next(w for w in workplaces if w["workplace_id"] == "WP1")
        assert wp1["name"] == "Interní oddělení"
        assert "interna" in wp1["specialties"]
        assert wp1["contact"]["phone"] == "+420 111 222 333"
        assert wp1["contact"]["website"] == "https://nemocnice.cz"

        wp2 = next(w for w in workplaces if w["workplace_id"] == "WP2")
        assert wp2["name"] == "Chirurgické oddělení"
        assert "chirurgie" in wp2["specialties"]
        assert wp2["contact"]["email"] == "chirurgie@nemocnice.cz"

    @pytest.mark.asyncio
    async def test_get_provider_single_workplace_contact(self):
        """Single workplace contact data is mapped correctly."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        mock_resp = _make_httpx_response(MOCK_PROVIDER_DETAIL)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("12345"))

        wp = result["workplaces"][0]
        assert wp["workplace_id"] == "W001"
        assert wp["contact"]["phone"] == "+420 123 456 789"
        assert wp["contact"]["email"] == "novak@example.cz"
        assert wp["contact"]["website"] is None

    @pytest.mark.asyncio
    async def test_get_invalid_id(self):
        """Get provider with unknown ID returns error JSON."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        mock_resp = _make_httpx_response({}, status_code=404)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("nonexistent999"))

        assert "error" in result
        assert "nonexistent999" in result["error"]

    @pytest.mark.asyncio
    async def test_get_api_error_returns_error_json(self):
        """Network errors are caught and returned as error JSON."""
        import httpx as _httpx

        from biomcp.czech.nrpzs.search import _nrpzs_get

        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(
            side_effect=_httpx.ConnectError("Connection refused")
        )

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("12345"))

        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_returns_cached_response(self):
        """Cache hit returns stored response without HTTP call."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        cached_payload = json.dumps(
            {
                "provider_id": "12345",
                "name": "Cached Provider",
                "source": "NRPZS",
            }
        )

        with patch(
            "biomcp.czech.nrpzs.search.get_cached_response",
            return_value=cached_payload,
        ), patch("httpx.AsyncClient") as mock_cls:
            result = json.loads(await _nrpzs_get("12345"))

        assert result["name"] == "Cached Provider"
        mock_cls.assert_not_called()

    @pytest.mark.asyncio
    async def test_get_provider_no_workplaces(self):
        """Provider without workplaces returns empty list."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        provider_no_wp = {
            **MOCK_PROVIDER_DETAIL,
            "pracoviste": [],
        }
        mock_resp = _make_httpx_response(provider_no_wp)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("12345"))

        assert result["workplaces"] == []

    @pytest.mark.asyncio
    async def test_get_provider_null_optional_fields(self):
        """Provider with missing optional fields returns None values."""
        from biomcp.czech.nrpzs.search import _nrpzs_get

        minimal_provider = {
            "id": "00001",
            "nazev": "Minimal Provider",
        }
        mock_resp = _make_httpx_response(minimal_provider)
        mock_client = AsyncMock()
        mock_client.__aenter__ = AsyncMock(return_value=mock_client)
        mock_client.__aexit__ = AsyncMock(return_value=False)
        mock_client.get = AsyncMock(return_value=mock_resp)

        with patch("biomcp.czech.nrpzs.search.get_cached_response",
                   return_value=None), \
             patch("biomcp.czech.nrpzs.search.cache_response"), \
             patch("httpx.AsyncClient", return_value=mock_client):
            result = json.loads(await _nrpzs_get("00001"))

        assert result["provider_id"] == "00001"
        assert result["name"] == "Minimal Provider"
        assert result["legal_form"] is None
        assert result["ico"] is None
        assert result["address"] is None
        assert result["specialties"] == []
        assert result["care_types"] == []
        assert result["workplaces"] == []
        assert result["source"] == "NRPZS"
