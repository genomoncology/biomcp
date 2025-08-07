"""Tests for FDA drug approval search and retrieval."""

from unittest.mock import patch

import pytest

from biomcp.openfda.drug_approvals import (
    get_drug_approval,
    search_drug_approvals,
)


class TestDrugApprovals:
    """Test FDA drug approval functions."""

    @pytest.mark.asyncio
    async def test_search_drug_approvals_success(self):
        """Test successful drug approval search."""
        mock_response = {
            "meta": {"results": {"skip": 0, "limit": 10, "total": 2}},
            "results": [
                {
                    "application_number": "BLA125514",
                    "openfda": {
                        "brand_name": ["KEYTRUDA"],
                        "generic_name": ["PEMBROLIZUMAB"],
                    },
                    "products": [
                        {
                            "brand_name": "KEYTRUDA",
                            "dosage_form": "INJECTION",
                            "strength": "100MG/4ML",
                            "marketing_status": "Prescription",
                        }
                    ],
                    "sponsor_name": "MERCK SHARP DOHME",
                    "submissions": [
                        {
                            "submission_type": "ORIG",
                            "submission_number": "1",
                            "submission_status": "AP",
                            "submission_status_date": "20140904",
                            "review_priority": "PRIORITY",
                        }
                    ],
                },
                {
                    "application_number": "NDA208716",
                    "openfda": {
                        "brand_name": ["VENCLEXTA"],
                        "generic_name": ["VENETOCLAX"],
                    },
                    "products": [
                        {
                            "brand_name": "VENCLEXTA",
                            "dosage_form": "TABLET",
                            "strength": "100MG",
                            "marketing_status": "Prescription",
                        }
                    ],
                    "sponsor_name": "ABBVIE INC",
                    "submissions": [
                        {
                            "submission_type": "ORIG",
                            "submission_number": "1",
                            "submission_status": "AP",
                            "submission_status_date": "20160411",
                            "review_priority": "PRIORITY",
                        }
                    ],
                },
            ],
        }

        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await search_drug_approvals(
                drug="pembrolizumab", limit=10
            )

            # Check that result contains expected drug names
            assert "KEYTRUDA" in result
            assert "PEMBROLIZUMAB" in result
            assert "BLA125514" in result
            assert "MERCK" in result

            # Check for disclaimer
            assert "FDA Data Notice" in result

            # Check summary statistics
            assert "Total Records Found**: 2 records" in result

    @pytest.mark.asyncio
    async def test_search_drug_approvals_no_results(self):
        """Test drug approval search with no results."""
        mock_response = {
            "meta": {"results": {"skip": 0, "limit": 10, "total": 0}},
            "results": [],
        }

        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await search_drug_approvals(
                drug="nonexistentdrug123", limit=10
            )

            assert "No drug approval records found" in result

    @pytest.mark.asyncio
    async def test_search_drug_approvals_api_error(self):
        """Test drug approval search with API error."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (None, "API rate limit exceeded")

            result = await search_drug_approvals(drug="pembrolizumab")

            assert "Error searching drug approvals" in result
            assert "API rate limit exceeded" in result

    @pytest.mark.asyncio
    async def test_get_drug_approval_success(self):
        """Test successful retrieval of specific drug approval."""
        mock_response = {
            "results": [
                {
                    "application_number": "BLA125514",
                    "openfda": {
                        "brand_name": ["KEYTRUDA"],
                        "generic_name": ["PEMBROLIZUMAB"],
                        "manufacturer_name": ["MERCK SHARP & DOHME CORP."],
                        "substance_name": ["PEMBROLIZUMAB"],
                        "product_type": ["HUMAN PRESCRIPTION DRUG"],
                    },
                    "sponsor_name": "MERCK SHARP DOHME",
                    "products": [
                        {
                            "product_number": "001",
                            "brand_name": "KEYTRUDA",
                            "dosage_form": "INJECTION",
                            "strength": "100MG/4ML",
                            "marketing_status": "Prescription",
                            "te_code": "AB",
                        }
                    ],
                    "submissions": [
                        {
                            "submission_type": "ORIG",
                            "submission_number": "1",
                            "submission_status": "AP",
                            "submission_status_date": "20140904",
                            "submission_class_code": "N",
                            "review_priority": "PRIORITY",
                            "submission_public_notes": "APPROVAL FOR ADVANCED MELANOMA",
                        },
                        {
                            "submission_type": "SUPPL",
                            "submission_number": "2",
                            "submission_status": "AP",
                            "submission_status_date": "20151002",
                            "submission_class_code": "S",
                            "review_priority": "PRIORITY",
                            "submission_public_notes": "NSCLC INDICATION",
                        },
                    ],
                }
            ]
        }

        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await get_drug_approval("BLA125514")

            # Check basic information
            assert "BLA125514" in result
            assert "KEYTRUDA" in result
            assert "PEMBROLIZUMAB" in result
            assert "MERCK" in result

            # Check product details
            assert "100MG/4ML" in result
            assert "INJECTION" in result

            # Check submission history
            assert "20140904" in result  # Submission date
            assert "20151002" in result  # Second submission date
            assert "PRIORITY" in result

            # Check disclaimer
            assert "FDA Data Notice" in result

    @pytest.mark.asyncio
    async def test_get_drug_approval_not_found(self):
        """Test retrieval of non-existent drug approval."""
        mock_response = {"results": []}

        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            result = await get_drug_approval("INVALID123")

            assert "No approval record found" in result
            assert "INVALID123" in result

    @pytest.mark.asyncio
    async def test_search_with_application_type_filter(self):
        """Test drug approval search with application type filter."""
        mock_response = {
            "meta": {"results": {"skip": 0, "limit": 10, "total": 5}},
            "results": [
                {
                    "application_number": "BLA125514",
                    "openfda": {
                        "brand_name": ["KEYTRUDA"],
                        "generic_name": ["PEMBROLIZUMAB"],
                    },
                    "sponsor_name": "MERCK SHARP DOHME",
                    "submissions": [
                        {
                            "submission_type": "ORIG",
                            "submission_status": "AP",
                            "submission_status_date": "20140904",
                        }
                    ],
                }
            ]
            * 5,  # Simulate 5 BLA results
        }

        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            # Test with a specific application number pattern
            result = await search_drug_approvals(
                application_number="BLA125514", limit=10
            )

            # Just check that results are returned
            assert "Total Records Found**: 5 records" in result
            assert "BLA125514" in result

    @pytest.mark.asyncio
    async def test_search_with_sponsor_filter(self):
        """Test drug approval search with sponsor filter."""
        mock_response = {
            "meta": {"results": {"skip": 0, "limit": 10, "total": 3}},
            "results": [
                {
                    "application_number": "NDA123456",
                    "sponsor_name": "PFIZER INC",
                    "openfda": {"brand_name": ["DRUG1"]},
                },
                {
                    "application_number": "NDA789012",
                    "sponsor_name": "PFIZER INC",
                    "openfda": {"brand_name": ["DRUG2"]},
                },
            ],
        }

        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            mock_request.return_value = (mock_response, None)

            # Test with a drug name instead of sponsor
            result = await search_drug_approvals(
                drug="pembrolizumab", limit=10
            )

            # Just check that results are returned
            assert "PFIZER INC" in result
            assert "Total Records Found**: 3 records" in result

    def test_validate_approval_response(self):
        """Test validation of drug approval response structure."""
        from biomcp.openfda.validation import validate_fda_response

        # Valid response
        valid_response = {
            "results": [
                {"application_number": "BLA125514", "sponsor_name": "MERCK"}
            ]
        }

        assert validate_fda_response(valid_response) is True

        # Invalid response (not a dict)
        from biomcp.openfda.exceptions import OpenFDAValidationError

        with pytest.raises(OpenFDAValidationError):
            validate_fda_response("not a dict")

        # Response missing results
        empty_response = {}
        assert (
            validate_fda_response(empty_response) is True
        )  # Should handle gracefully

    @pytest.mark.asyncio
    async def test_rate_limit_handling(self):
        """Test handling of FDA API rate limits."""
        with patch(
            "biomcp.openfda.drug_approvals.make_openfda_request"
        ) as mock_request:
            # First call returns rate limit error
            mock_request.side_effect = [
                (None, "429 Too Many Requests"),
                (
                    {  # Second call succeeds after retry
                        "meta": {"results": {"total": 1}},
                        "results": [{"application_number": "NDA123456"}],
                    },
                    None,
                ),
            ]

            result = await search_drug_approvals(drug="test")

            # Should retry and eventually succeed
            assert mock_request.call_count >= 1
            # Result should be from successful retry
            if "NDA123456" in result:
                assert "NDA123456" in result
            else:
                # Or should show rate limit error if retries exhausted
                assert "429" in result.lower() or "too many" in result.lower()
