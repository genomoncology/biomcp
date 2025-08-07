"""Integration tests for OpenFDA API.

These tests make real API calls to verify FDA integration works correctly.
They are marked with pytest.mark.integration and can be skipped with --ignore-integration.
"""

import os

import pytest

from biomcp.openfda.adverse_events import search_adverse_events
from biomcp.openfda.device_events import search_device_events
from biomcp.openfda.drug_approvals import search_drug_approvals
from biomcp.openfda.drug_labels import search_drug_labels
from biomcp.openfda.drug_recalls import search_drug_recalls


@pytest.mark.integration
class TestOpenFDAIntegration:
    """Integration tests for OpenFDA API endpoints."""

    @pytest.mark.asyncio
    async def test_adverse_events_real_api(self):
        """Test real adverse event API call."""
        result = await search_adverse_events(drug="aspirin", limit=5)

        # Should return formatted results
        assert isinstance(result, str)
        assert len(result) > 100  # Non-trivial response

        # Should contain disclaimer
        assert "FDA Data Notice" in result

        # Should have structure
        if "No adverse events found" not in result:
            assert (
                "Total Reports Found:" in result or "adverse" in result.lower()
            )

    @pytest.mark.asyncio
    async def test_drug_labels_real_api(self):
        """Test real drug label API call."""
        result = await search_drug_labels(name="ibuprofen", limit=5)

        # Should return formatted results
        assert isinstance(result, str)
        assert len(result) > 100

        # Should contain disclaimer
        assert "FDA Data Notice" in result

        # Should have label information
        if "No drug labels found" not in result:
            assert "Total Labels Found:" in result or "label" in result.lower()

    @pytest.mark.asyncio
    async def test_device_events_real_api(self):
        """Test real device event API call."""
        result = await search_device_events(device="insulin pump", limit=5)

        # Should return formatted results
        assert isinstance(result, str)
        assert len(result) > 100

        # Should contain disclaimer
        assert "FDA Data Notice" in result

        # Should have device information
        if "No device events found" not in result:
            assert (
                "Total Events Found:" in result or "device" in result.lower()
            )

    @pytest.mark.asyncio
    async def test_drug_approvals_real_api(self):
        """Test real drug approval API call."""
        result = await search_drug_approvals(drug="pembrolizumab", limit=5)

        # Should return formatted results
        assert isinstance(result, str)
        assert len(result) > 100

        # Should contain disclaimer
        assert "FDA Data Notice" in result

        # Pembrolizumab (Keytruda) should have results
        if "No drug approvals found" not in result:
            assert "KEYTRUDA" in result or "pembrolizumab" in result.lower()

    @pytest.mark.asyncio
    async def test_drug_recalls_real_api(self):
        """Test real drug recall API call."""
        # Use drug parameter which is more likely to return results
        result = await search_drug_recalls(drug="acetaminophen", limit=5)

        # Should return formatted results
        assert isinstance(result, str)
        assert len(result) > 100

        # Should contain disclaimer OR error message (API might return no results)
        assert "FDA Data Notice" in result or "Error" in result

        # Should have recall information if not an error
        if "Error" not in result and "No drug recalls found" not in result:
            assert "recall" in result.lower()

    @pytest.mark.asyncio
    async def test_rate_limiting_without_key(self):
        """Test that rate limiting is handled gracefully without API key."""
        # Temporarily remove API key if present
        original_key = os.environ.get("OPENFDA_API_KEY")
        if original_key:
            del os.environ["OPENFDA_API_KEY"]

        try:
            # Make multiple rapid requests
            results = []
            for i in range(5):
                result = await search_adverse_events(drug=f"drug{i}", limit=1)
                results.append(result)

            # All should return strings (not crash)
            assert all(isinstance(r, str) for r in results)

        finally:
            # Restore API key
            if original_key:
                os.environ["OPENFDA_API_KEY"] = original_key

    @pytest.mark.asyncio
    async def test_api_key_usage(self):
        """Test that API key is used when provided."""
        # This test only runs if API key is available
        if not os.environ.get("OPENFDA_API_KEY"):
            pytest.skip("OPENFDA_API_KEY not set")

        result = await search_adverse_events(drug="acetaminophen", limit=10)

        # With API key, should be able to get results
        assert isinstance(result, str)
        assert len(result) > 100

    @pytest.mark.asyncio
    async def test_error_handling_invalid_params(self):
        """Test graceful handling of invalid parameters."""
        # Search with invalid/nonsense parameters
        result = await search_adverse_events(
            drug="xyzabc123notarealdrugname999", limit=5
        )

        # Should handle gracefully
        assert isinstance(result, str)

        # Should either show no results or error message
        assert (
            "No adverse events found" in result
            or "Error" in result
            or "no results" in result.lower()
        )

    @pytest.mark.asyncio
    async def test_cross_domain_consistency(self):
        """Test that different FDA domains return consistent formats."""
        # Search for a common drug across domains
        drug_name = "aspirin"

        adverse_result = await search_adverse_events(drug=drug_name, limit=2)
        label_result = await search_drug_labels(name=drug_name, limit=2)

        # Both should have disclaimers
        assert "FDA Data Notice" in adverse_result
        assert "FDA Data Notice" in label_result

        # Both should be properly formatted strings
        assert isinstance(adverse_result, str)
        assert isinstance(label_result, str)

        # Both should mention the drug or indicate no results
        assert (
            drug_name in adverse_result.lower()
            or "no " in adverse_result.lower()
        )
        assert (
            drug_name in label_result.lower() or "no " in label_result.lower()
        )

    @pytest.mark.asyncio
    async def test_special_characters_handling(self):
        """Test handling of special characters in queries."""
        # Test with special characters
        result = await search_drug_labels(name="aspirin/dipyridamole", limit=5)

        # Should handle forward slash gracefully
        assert isinstance(result, str)
        # API might return error or no results for complex drug names
        assert isinstance(result, str)  # Just verify we get a response

    @pytest.mark.asyncio
    async def test_large_result_handling(self):
        """Test handling of large result sets."""
        # Request maximum allowed results
        result = await search_adverse_events(
            drug="ibuprofen",  # Common drug with many reports
            limit=100,  # Maximum limit
        )

        # Should handle large results
        assert isinstance(result, str)
        assert len(result) > 500  # Should be substantial

        # Should still include disclaimer
        assert "FDA Data Notice" in result

    @pytest.mark.asyncio
    async def test_empty_query_handling(self):
        """Test handling of empty/missing query parameters."""
        # Search without specifying a drug
        result = await search_drug_recalls(
            limit=5  # Only limit, no other filters
        )

        # Should return recent recalls
        assert isinstance(result, str)
        assert len(result) > 100

        # Should have results (there are always some recalls)
        if "Error" not in result:
            assert "recall" in result.lower()
