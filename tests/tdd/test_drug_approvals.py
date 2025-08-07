"""Tests for FDA drug approvals module."""

import json
from pathlib import Path
from unittest.mock import AsyncMock, patch

import pytest

from biomcp.openfda.drug_approvals import (
    get_drug_approval,
    search_drug_approvals,
)

# Load mock data
MOCK_DIR = Path(__file__).parent.parent / "data" / "openfda"
MOCK_APPROVALS_SEARCH = json.loads(
    (MOCK_DIR / "drugsfda_search.json").read_text()
)
MOCK_APPROVAL_DETAIL = json.loads(
    (MOCK_DIR / "drugsfda_detail.json").read_text()
)


class TestDrugApprovals:
    """Test drug approvals functionality."""

    @pytest.mark.asyncio
    async def test_search_drug_approvals_success(self):
        """Test successful drug approval search."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_APPROVALS_SEARCH, None)

            result = await search_drug_approvals(
                drug="pembrolizumab",
                limit=10,
            )

            assert "FDA Drug Approval Records" in result
            assert "pembrolizumab" in result.lower()
            assert "Application" in result
            assert "BLA125514" in result
            mock_request.assert_called_once()

    @pytest.mark.asyncio
    async def test_search_drug_approvals_with_filters(self):
        """Test drug approval search with multiple filters."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_APPROVALS_SEARCH, None)

            result = await search_drug_approvals(
                drug="keytruda",
                application_number="BLA125514",
                approval_year="2014",
                limit=5,
                api_key="test-key",
            )

            assert "FDA Drug Approval Records" in result
            # Verify API key was passed as the 4th positional argument
            call_args = mock_request.call_args
            assert (
                call_args[0][3] == "test-key"
            )  # api_key is 4th positional arg

    @pytest.mark.asyncio
    async def test_search_drug_approvals_no_results(self):
        """Test drug approval search with no results."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = ({"results": []}, None)

            result = await search_drug_approvals(drug="nonexistent-drug")

            assert "No drug approval records found" in result

    @pytest.mark.asyncio
    async def test_search_drug_approvals_api_error(self):
        """Test drug approval search with API error."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (None, "API rate limit exceeded")

            result = await search_drug_approvals(drug="test")

            assert "Error searching drug approvals" in result
            assert "API rate limit exceeded" in result

    @pytest.mark.asyncio
    async def test_get_drug_approval_success(self):
        """Test getting specific drug approval details."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_APPROVAL_DETAIL, None)

            result = await get_drug_approval("BLA125514")

            # Should have detailed approval info
            assert "BLA125514" in result or "Drug Approval Details" in result
            assert "BLA125514" in result
            assert "Products" in result
            assert "Submission" in result

    @pytest.mark.asyncio
    async def test_get_drug_approval_not_found(self):
        """Test getting drug approval that doesn't exist."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = ({"results": []}, None)

            result = await get_drug_approval("INVALID123")

            assert "No approval record found" in result
            assert "INVALID123" in result

    @pytest.mark.asyncio
    async def test_get_drug_approval_with_api_key(self):
        """Test getting drug approval with API key."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_APPROVAL_DETAIL, None)

            result = await get_drug_approval(
                "BLA125514",
                api_key="test-api-key",
            )

            # Should have detailed approval info
            assert "BLA125514" in result or "Drug Approval Details" in result
            # Verify API key was passed as the 4th positional argument
            call_args = mock_request.call_args
            assert (
                call_args[0][3] == "test-api-key"
            )  # api_key is 4th positional arg

    @pytest.mark.asyncio
    async def test_search_drug_approvals_pagination(self):
        """Test drug approval search pagination."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_response = {
                "meta": {"results": {"total": 100}},
                "results": MOCK_APPROVALS_SEARCH["results"],
            }
            mock_request.return_value = (mock_response, None)

            result = await search_drug_approvals(
                drug="cancer",
                limit=10,
                skip=20,
            )

            # The output format is different - just check for the total
            assert "100" in result
            # Verify skip parameter was passed (2nd positional arg)
            call_args = mock_request.call_args
            assert (
                call_args[0][1]["skip"] == "20"
            )  # params is 2nd positional arg, value is string

    @pytest.mark.asyncio
    async def test_approval_year_validation(self):
        """Test that approval year is properly formatted."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request",
            new_callable=AsyncMock,
        ) as mock_request:
            mock_request.return_value = (MOCK_APPROVALS_SEARCH, None)

            await search_drug_approvals(
                approval_year="2023",
            )

            # Check that year was properly formatted in query
            call_args = mock_request.call_args
            params = call_args[0][1]  # params is 2nd positional arg
            assert "marketing_status_date" in params["search"]
            assert "[2023-01-01 TO 2023-12-31]" in params["search"]
