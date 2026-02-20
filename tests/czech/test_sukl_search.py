"""Unit tests for SUKL drug search functionality."""

import json
from unittest.mock import AsyncMock, patch

import pytest


class TestSuklDrugSearch:
    """Tests for _sukl_drug_search function."""

    @pytest.fixture
    def mock_drug_list_response(self):
        """Mock response from /lecive-pripravky endpoint."""
        return ["0000123", "0000456", "0000789"]

    @pytest.fixture
    def mock_drug_detail(self):
        """Mock drug detail from /lecive-pripravky/{kod}."""
        return {
            "kodSukl": "0000123",
            "nazev": "NUROFEN 400MG",
            "doplnekNazvu": "400MG TBL FLM 24",
            "kodFormy": "TBL FLM",
            "nazevFormy": "potahovaná tableta",
            "kodAtc": "M01AE01",
            "registracniCislo": "07/123/01-C",
            "nazevDrzitele": "Reckitt Benckiser Healthcare",
        }

    @pytest.fixture
    def mock_substance_response(self):
        """Mock substance codelist response."""
        return [
            {
                "kodLatky": "1234",
                "nazev": "IBUPROFENUM",
                "nazevCesky": "IBUPROFEN",
            },
        ]

    @pytest.fixture
    def mock_composition_response(self):
        """Mock drug composition response."""
        return [
            {
                "kodLatky": "1234",
                "nazevLatky": "IBUPROFENUM",
                "mnozstvi": "400",
                "jednotka": "MG",
            }
        ]

    @pytest.mark.asyncio
    async def test_search_by_name(self, mock_drug_detail):
        """Search by drug name returns matching results."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        with patch(
            "biomcp.czech.sukl.search._fetch_drug_list",
            new_callable=AsyncMock,
            return_value=["0000123"],
        ), patch(
            "biomcp.czech.sukl.search._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_detail,
        ):
            result = json.loads(await _sukl_drug_search("Nurofen"))
            assert result["total"] >= 1
            assert len(result["results"]) >= 1
            assert result["results"][0]["sukl_code"] == "0000123"

    @pytest.mark.asyncio
    async def test_search_by_atc_code(self, mock_drug_detail):
        """Search by ATC code returns matching drugs."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        with patch(
            "biomcp.czech.sukl.search._fetch_drug_list",
            new_callable=AsyncMock,
            return_value=["0000123"],
        ), patch(
            "biomcp.czech.sukl.search._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_detail,
        ):
            result = json.loads(
                await _sukl_drug_search("M01AE01")
            )
            assert result["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_pagination(self, mock_drug_detail):
        """Search supports pagination parameters."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        details = [
            {
                **mock_drug_detail,
                "kodSukl": f"000{i:04d}",
                "nazev": f"DRUG {i}",
            }
            for i in range(5)
        ]

        async def mock_fetch_detail(code, **kwargs):
            for d in details:
                if d["kodSukl"] == code:
                    return d
            return None

        codes = [d["kodSukl"] for d in details]
        with patch(
            "biomcp.czech.sukl.search._fetch_drug_list",
            new_callable=AsyncMock,
            return_value=codes,
        ), patch(
            "biomcp.czech.sukl.search._fetch_drug_detail",
            new_callable=AsyncMock,
            side_effect=mock_fetch_detail,
        ):
            result = json.loads(
                await _sukl_drug_search("DRUG", page=2, page_size=2)
            )
            assert result["page"] == 2
            assert result["page_size"] == 2
            assert len(result["results"]) == 2

    @pytest.mark.asyncio
    async def test_search_empty_results(self):
        """Search with no matches returns empty result set."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        with patch(
            "biomcp.czech.sukl.search._fetch_drug_list",
            new_callable=AsyncMock,
            return_value=[],
        ):
            result = json.loads(
                await _sukl_drug_search("nonexistentdrug12345")
            )
            assert result["total"] == 0
            assert result["results"] == []

    @pytest.mark.asyncio
    async def test_search_diacritics_handling(
        self, mock_drug_detail
    ):
        """Search handles Czech diacritics transparently."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        detail_with_diacritics = {
            **mock_drug_detail,
            "nazev": "LÉČIVÝ PŘÍPRAVEK",
        }

        with patch(
            "biomcp.czech.sukl.search._fetch_drug_list",
            new_callable=AsyncMock,
            return_value=["0000123"],
        ), patch(
            "biomcp.czech.sukl.search._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=detail_with_diacritics,
        ):
            result = json.loads(
                await _sukl_drug_search("lecivy pripravek")
            )
            assert result["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_api_error_returns_cached_or_empty(self):
        """Search handles API errors gracefully."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        with patch(
            "biomcp.czech.sukl.search._fetch_drug_list",
            new_callable=AsyncMock,
            side_effect=Exception("API unavailable"),
        ):
            result = json.loads(
                await _sukl_drug_search("test")
            )
            assert "error" in result or result["total"] == 0
