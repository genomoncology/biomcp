"""Unit tests for SUKL drug getter functionality."""

import json
from unittest.mock import AsyncMock, patch

import pytest


class TestSuklDrugGetter:
    """Tests for sukl_drug_getter MCP tool."""

    @pytest.fixture
    def mock_drug_api_response(self):
        """Full drug detail response from SUKL API."""
        return {
            "kodSukl": "0000123",
            "nazev": "NUROFEN 400MG",
            "doplnekNazvu": "400MG TBL FLM 24",
            "kodFormy": "TBL FLM",
            "nazevFormy": "potahovaná tableta",
            "kodAtc": "M01AE01",
            "registracniCislo": "07/123/01-C",
            "nazevDrzitele": "Reckitt Benckiser Healthcare",
            "platnostRegistrace": "2028-12-31",
        }

    @pytest.fixture
    def mock_composition(self):
        """Drug composition response."""
        return [
            {
                "kodLatky": "1234",
                "nazevLatky": "IBUPROFENUM",
                "mnozstvi": "400",
                "jednotka": "MG",
            }
        ]

    @pytest.fixture
    def mock_doc_metadata_spc(self):
        """Document metadata for SmPC."""
        return [
            {
                "typ": "spc",
                "idDokumentu": "DOC001",
                "nazevSouboru": "spc_0000123.pdf",
            }
        ]

    @pytest.fixture
    def mock_doc_metadata_pil(self):
        """Document metadata for PIL."""
        return [
            {
                "typ": "pil",
                "idDokumentu": "DOC002",
                "nazevSouboru": "pil_0000123.pdf",
            }
        ]

    @pytest.mark.asyncio
    async def test_get_drug_details(
        self, mock_drug_api_response, mock_composition
    ):
        """Get drug details returns full Drug entity."""
        from biomcp.czech.sukl.getter import _sukl_drug_details

        with patch(
            "biomcp.czech.sukl.getter._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_api_response,
        ), patch(
            "biomcp.czech.sukl.getter._fetch_composition",
            new_callable=AsyncMock,
            return_value=mock_composition,
        ), patch(
            "biomcp.czech.sukl.getter._fetch_doc_metadata",
            new_callable=AsyncMock,
            return_value=[],
        ):
            result = json.loads(
                await _sukl_drug_details("0000123")
            )
            assert result["sukl_code"] == "0000123"
            assert result["name"] == "NUROFEN 400MG"
            assert result["atc_code"] == "M01AE01"
            assert result["source"] == "SUKL"
            assert len(result["active_substances"]) >= 1

    @pytest.mark.asyncio
    async def test_get_drug_invalid_code(self):
        """Get drug with invalid code returns error."""
        from biomcp.czech.sukl.getter import _sukl_drug_details

        with patch(
            "biomcp.czech.sukl.getter._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=None,
        ):
            result = json.loads(
                await _sukl_drug_details("9999999")
            )
            assert "error" in result

    @pytest.mark.asyncio
    async def test_get_spc(
        self,
        mock_drug_api_response,
        mock_doc_metadata_spc,
    ):
        """Get SmPC returns document info."""
        from biomcp.czech.sukl.getter import _sukl_spc_getter

        with patch(
            "biomcp.czech.sukl.getter._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_api_response,
        ), patch(
            "biomcp.czech.sukl.getter._fetch_doc_metadata",
            new_callable=AsyncMock,
            return_value=mock_doc_metadata_spc,
        ):
            result = json.loads(
                await _sukl_spc_getter("0000123")
            )
            assert result["sukl_code"] == "0000123"
            assert "spc_url" in result
            assert result["source"] == "SUKL"

    @pytest.mark.asyncio
    async def test_get_pil(
        self,
        mock_drug_api_response,
        mock_doc_metadata_pil,
    ):
        """Get PIL returns document info."""
        from biomcp.czech.sukl.getter import _sukl_pil_getter

        with patch(
            "biomcp.czech.sukl.getter._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_api_response,
        ), patch(
            "biomcp.czech.sukl.getter._fetch_doc_metadata",
            new_callable=AsyncMock,
            return_value=mock_doc_metadata_pil,
        ):
            result = json.loads(
                await _sukl_pil_getter("0000123")
            )
            assert result["sukl_code"] == "0000123"
            assert "pil_url" in result
            assert result["source"] == "SUKL"

    @pytest.mark.asyncio
    async def test_get_spc_not_available(
        self, mock_drug_api_response
    ):
        """SmPC not available returns error."""
        from biomcp.czech.sukl.getter import _sukl_spc_getter

        with patch(
            "biomcp.czech.sukl.getter._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_api_response,
        ), patch(
            "biomcp.czech.sukl.getter._fetch_doc_metadata",
            new_callable=AsyncMock,
            return_value=[],
        ):
            result = json.loads(
                await _sukl_spc_getter("0000123")
            )
            assert (
                "error" in result
                or result.get("spc_text") is None
            )
