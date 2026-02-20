"""Unit tests for SUKL drug availability check."""

import json
from unittest.mock import AsyncMock, patch

import pytest


class TestSuklAvailability:
    """Tests for sukl_availability_checker MCP tool."""

    @pytest.fixture
    def mock_drug_in_list(self):
        """Drug present in active distribution list."""
        return {
            "kodSukl": "0000123",
            "nazev": "NUROFEN 400MG",
            "kodAtc": "M01AE01",
        }

    @pytest.fixture
    def mock_vpois_response(self):
        """VPOIS (holder info service) response."""
        return {
            "nazevSpolecnosti": "Reckitt Benckiser",
            "web": "https://www.rb.com",
            "email": "info@rb.com",
            "telefon": "+420123456789",
        }

    @pytest.mark.asyncio
    async def test_available_drug(self, mock_drug_in_list):
        """Drug in active list shows as available."""
        from biomcp.czech.sukl.availability import (
            _sukl_availability_check,
        )

        with patch(
            "biomcp.czech.sukl.availability._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_in_list,
        ), patch(
            "biomcp.czech.sukl.availability._check_distribution",
            new_callable=AsyncMock,
            return_value="available",
        ):
            result = json.loads(
                await _sukl_availability_check("0000123")
            )
            assert result["sukl_code"] == "0000123"
            assert result["status"] == "available"
            assert result["source"] == "SUKL"

    @pytest.mark.asyncio
    async def test_limited_drug(self, mock_drug_in_list):
        """Drug with limited availability."""
        from biomcp.czech.sukl.availability import (
            _sukl_availability_check,
        )

        with patch(
            "biomcp.czech.sukl.availability._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_in_list,
        ), patch(
            "biomcp.czech.sukl.availability._check_distribution",
            new_callable=AsyncMock,
            return_value="limited",
        ):
            result = json.loads(
                await _sukl_availability_check("0000123")
            )
            assert result["status"] == "limited"

    @pytest.mark.asyncio
    async def test_unavailable_drug(self, mock_drug_in_list):
        """Drug not in distribution is unavailable."""
        from biomcp.czech.sukl.availability import (
            _sukl_availability_check,
        )

        with patch(
            "biomcp.czech.sukl.availability._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_in_list,
        ), patch(
            "biomcp.czech.sukl.availability._check_distribution",
            new_callable=AsyncMock,
            return_value="unavailable",
        ):
            result = json.loads(
                await _sukl_availability_check("0000123")
            )
            assert result["status"] == "unavailable"

    @pytest.mark.asyncio
    async def test_invalid_code(self):
        """Invalid SUKL code returns error."""
        from biomcp.czech.sukl.availability import (
            _sukl_availability_check,
        )

        with patch(
            "biomcp.czech.sukl.availability._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=None,
        ):
            result = json.loads(
                await _sukl_availability_check("9999999")
            )
            assert "error" in result

    @pytest.mark.asyncio
    async def test_availability_includes_timestamp(
        self, mock_drug_in_list
    ):
        """Result includes last_checked timestamp."""
        from biomcp.czech.sukl.availability import (
            _sukl_availability_check,
        )

        with patch(
            "biomcp.czech.sukl.availability._fetch_drug_detail",
            new_callable=AsyncMock,
            return_value=mock_drug_in_list,
        ), patch(
            "biomcp.czech.sukl.availability._check_distribution",
            new_callable=AsyncMock,
            return_value="available",
        ):
            result = json.loads(
                await _sukl_availability_check("0000123")
            )
            assert "last_checked" in result
