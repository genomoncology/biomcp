"""Tests for FDA drug shortages module."""

from datetime import datetime
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.openfda.drug_shortages import (
    get_drug_shortage,
    search_drug_shortages,
)


class TestDrugShortages:
    """Test drug shortages functionality."""

    @pytest.mark.asyncio
    async def test_search_drug_shortages_no_data_available(self):
        """Test drug shortage search when FDA data is unavailable."""
        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            mock_get_data.return_value = None

            result = await search_drug_shortages(drug="cisplatin")

            assert "Drug Shortage Data Temporarily Unavailable" in result
            assert "FDA drug shortage database cannot be accessed" in result
            assert (
                "https://www.accessdata.fda.gov/scripts/drugshortages/"
                in result
            )
            assert (
                "https://www.ashp.org/drug-shortages/current-shortages"
                in result
            )

    @pytest.mark.asyncio
    async def test_get_drug_shortage_no_data_available(self):
        """Test getting specific drug shortage when FDA data is unavailable."""
        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            mock_get_data.return_value = None

            result = await get_drug_shortage("cisplatin")

            assert "Drug Shortage Data Temporarily Unavailable" in result
            assert "FDA drug shortage database cannot be accessed" in result
            assert "Alternative Options:" in result

    @pytest.mark.asyncio
    async def test_mock_data_not_used_in_production(self):
        """Test that mock data is never returned in production scenarios."""
        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            # Simulate no data available (cache miss and fetch failure)
            mock_get_data.return_value = None

            result = await search_drug_shortages(drug="test")

            assert "Drug Shortage Data Temporarily Unavailable" in result
            # Ensure mock data is not present
            assert "Cisplatin Injection" not in result
            assert "Methotrexate" not in result

    # Cache functionality test removed - was testing private implementation details
    # The public API is tested through search_drug_shortages and get_drug_shortage

    # Cache expiry test removed - was testing private implementation details
    # The caching behavior is an implementation detail not part of the public API

    @pytest.mark.asyncio
    async def test_search_with_filters(self):
        """Test drug shortage search with various filters."""
        mock_data = {
            "_fetched_at": datetime.now().isoformat(),
            "shortages": [
                {
                    "generic_name": "Drug A",
                    "brand_names": ["Brand A"],
                    "status": "Current Shortage",
                    "therapeutic_category": "Oncology",
                },
                {
                    "generic_name": "Drug B",
                    "brand_names": ["Brand B"],
                    "status": "Resolved",
                    "therapeutic_category": "Cardiology",
                },
                {
                    "generic_name": "Drug C",
                    "brand_names": ["Brand C"],
                    "status": "Current Shortage",
                    "therapeutic_category": "Oncology",
                },
            ],
        }

        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            mock_get_data.return_value = mock_data

            # Test status filter
            result = await search_drug_shortages(status="current")
            assert "Drug A" in result
            assert "Drug C" in result
            assert "Drug B" not in result

            # Test therapeutic category filter
            result = await search_drug_shortages(
                therapeutic_category="Oncology"
            )
            assert "Drug A" in result
            assert "Drug C" in result
            assert "Drug B" not in result

            # Test drug name filter
            result = await search_drug_shortages(drug="Drug B")
            assert "Drug B" in result
            assert "Drug A" not in result

    @pytest.mark.asyncio
    async def test_get_specific_drug_shortage(self):
        """Test getting details for a specific drug shortage."""
        mock_data = {
            "_fetched_at": datetime.now().isoformat(),
            "shortages": [
                {
                    "generic_name": "Cisplatin Injection",
                    "brand_names": ["Platinol"],
                    "status": "Current Shortage",
                    "shortage_start_date": "2023-02-10",
                    "estimated_resolution": "Q2 2024",
                    "reason": "Manufacturing delays",
                    "therapeutic_category": "Oncology",
                    "notes": "Limited supplies available",
                },
            ],
        }

        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            mock_get_data.return_value = mock_data

            result = await get_drug_shortage("cisplatin")

            assert "Cisplatin Injection" in result
            assert "Current Shortage" in result
            assert "Manufacturing delays" in result
            assert "Oncology" in result
            assert "Limited supplies available" in result

    @pytest.mark.asyncio
    async def test_get_drug_shortage_not_found(self):
        """Test getting drug shortage for non-existent drug."""
        mock_data = {
            "_fetched_at": datetime.now().isoformat(),
            "shortages": [
                {
                    "generic_name": "Drug A",
                    "status": "Current Shortage",
                },
            ],
        }

        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            mock_get_data.return_value = mock_data

            result = await get_drug_shortage("nonexistent-drug")

            assert "No shortage information found" in result
            assert "nonexistent-drug" in result

    @pytest.mark.asyncio
    async def test_api_key_parameter_ignored(self):
        """Test that API key parameter is accepted but not used (FDA limitation)."""
        mock_data = {
            "_fetched_at": datetime.now().isoformat(),
            "shortages": [
                {
                    "generic_name": "Test Drug",
                    "status": "Current Shortage",
                    "therapeutic_category": "Test Category",
                }
            ],
        }

        with patch(
            "biomcp.openfda.drug_shortages._get_cached_shortage_data",
            new_callable=AsyncMock,
        ) as mock_get_data:
            mock_get_data.return_value = mock_data

            # API key should be accepted but not affect functionality
            result = await search_drug_shortages(
                drug="test",
                api_key="test-key",
            )

            # When there's data, it should format results
            assert "FDA Drug Shortage Information" in result
            assert "Test Drug" in result

    # Mock data function has been removed - no longer needed
