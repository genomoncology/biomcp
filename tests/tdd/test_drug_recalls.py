"""Tests for FDA drug recalls module."""

import json
from pathlib import Path
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.openfda.drug_recalls import (
    get_drug_recall,
    search_drug_recalls,
)

# Load mock data
MOCK_DIR = Path(__file__).parent.parent / "data" / "openfda"
MOCK_RECALLS_SEARCH = json.loads(
    (MOCK_DIR / "enforcement_search.json").read_text()
)
MOCK_RECALL_DETAIL = json.loads(
    (MOCK_DIR / "enforcement_detail.json").read_text()
)


class TestDrugRecalls:
    """Test drug recalls functionality."""

    @pytest.mark.asyncio
    async def test_search_drug_recalls_success(self):
        """Test successful drug recall search."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALLS_SEARCH, None)

            result = await search_drug_recalls(
                drug="valsartan",
                limit=10,
            )

            assert "Drug Recall" in result or "FDA Drug Recall" in result
            assert "valsartan" in result.lower()
            # Check for presence of key recall info
            assert "Recall" in result or "recall" in result.lower()
            mock_request.assert_called_once()

    @pytest.mark.asyncio
    async def test_search_drug_recalls_with_filters(self):
        """Test drug recall search with multiple filters."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALLS_SEARCH, None)

            result = await search_drug_recalls(
                drug="metformin",
                recall_class="2",
                status="ongoing",
                reason="contamination",
                since_date="20230101",
                limit=5,
                api_key="test-key",
            )

            assert "Drug Recall" in result or "FDA Drug Recall" in result
            # Verify API key was passed as the 4th positional argument
            call_args = mock_request.call_args
            assert (
                call_args[0][3] == "test-key"
            )  # api_key is 4th positional arg

    @pytest.mark.asyncio
    async def test_search_drug_recalls_no_results(self):
        """Test drug recall search with no results."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = ({"results": []}, None)

            result = await search_drug_recalls(drug="nonexistent-drug")

            assert "No drug recall records found" in result

    @pytest.mark.asyncio
    async def test_search_drug_recalls_api_error(self):
        """Test drug recall search with API error."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (None, "API rate limit exceeded")

            result = await search_drug_recalls(drug="test")

            assert "Error searching drug recalls" in result
            assert "API rate limit exceeded" in result

    @pytest.mark.asyncio
    async def test_get_drug_recall_success(self):
        """Test getting specific drug recall details."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALL_DETAIL, None)

            result = await get_drug_recall("D-0001-2023")

            assert "Drug Recall" in result or "D-0001-2023" in result
            assert "D-0001-2023" in result
            # Check for key details in the output (formats may vary)
            assert "product" in result.lower() or "valsartan" in result.lower()

    @pytest.mark.asyncio
    async def test_get_drug_recall_not_found(self):
        """Test getting drug recall that doesn't exist."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = ({"results": []}, None)

            result = await get_drug_recall("INVALID-RECALL")

            assert "No recall record found" in result
            assert "INVALID-RECALL" in result

    @pytest.mark.asyncio
    async def test_get_drug_recall_with_api_key(self):
        """Test getting drug recall with API key."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALL_DETAIL, None)

            result = await get_drug_recall(
                "D-0001-2023",
                api_key="test-api-key",
            )

            assert "Drug Recall" in result or "D-0001-2023" in result
            # Verify API key was passed as the 4th positional argument
            call_args = mock_request.call_args
            assert (
                call_args[0][3] == "test-api-key"
            )  # api_key is 4th positional arg

    @pytest.mark.asyncio
    async def test_recall_class_validation(self):
        """Test that recall class is validated."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALLS_SEARCH, None)

            # Valid recall classes
            for recall_class in ["1", "2", "3"]:
                result = await search_drug_recalls(recall_class=recall_class)
                assert "Drug Recall" in result or "FDA Drug Recall" in result

            # Test with Class I, II, III format
            result = await search_drug_recalls(recall_class="Class I")
            call_args = mock_request.call_args
            params = call_args[0][1]  # params is 2nd positional arg
            assert 'classification:"Class I"' in params["search"]

    @pytest.mark.asyncio
    async def test_recall_status_mapping(self):
        """Test that recall status is properly mapped."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALLS_SEARCH, None)

            # Test ongoing status
            await search_drug_recalls(status="ongoing")
            call_args = mock_request.call_args
            params = call_args[0][1]  # params is 2nd positional arg
            assert "Ongoing" in params["search"]

            # Test completed status
            await search_drug_recalls(status="completed")
            call_args = mock_request.call_args
            params = call_args[0][1]  # params is 2nd positional arg
            assert "Completed" in params["search"]

    @pytest.mark.asyncio
    async def test_search_drug_recalls_pagination(self):
        """Test drug recall search pagination."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_response = {
                "meta": {"results": {"total": 150}},
                "results": MOCK_RECALLS_SEARCH["results"],
            }
            mock_request.return_value = (mock_response, None)

            result = await search_drug_recalls(
                drug="aspirin",
                limit=10,
                skip=30,
            )

            # Check for total count instead of specific pagination format
            assert "150" in result
            # Verify skip parameter was passed
            call_args = mock_request.call_args
            assert (
                call_args[0][1]["skip"] == "30"
            )  # params is 2nd positional arg, value is string

    @pytest.mark.asyncio
    async def test_date_filtering(self):
        """Test that date filtering works correctly."""
        with patch(
            "biomcp.openfda.drug_recalls.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_RECALLS_SEARCH, None)

            await search_drug_recalls(
                since_date="20230615",
            )

            # Check that date was properly formatted in query
            call_args = mock_request.call_args
            params = call_args[0][1]  # params is 2nd positional arg
            assert "recall_initiation_date" in params["search"]
            assert "[2023-06-15 TO *]" in params["search"]
