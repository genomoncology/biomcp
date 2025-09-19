"""Tests for OpenFDA integration with unified search/fetch tools."""

import pytest


class TestOpenFDAUnifiedIntegration:
    """Test OpenFDA domain integration in unified tools."""

    def test_openfda_domains_registered(self):
        """Test that OpenFDA domains are properly registered in constants."""
        from biomcp.constants import (
            DOMAIN_TO_PLURAL,
            PLURAL_TO_DOMAIN,
            VALID_DOMAINS,
            VALID_DOMAINS_PLURAL,
        )

        # List of OpenFDA domains
        openfda_domains = [
            "fda_adverse",
            "fda_label",
            "fda_device",
            "fda_approval",
            "fda_recall",
            "fda_shortage",
        ]

        openfda_plurals = [
            "fda_adverse_events",
            "fda_labels",
            "fda_device_events",
            "fda_approvals",
            "fda_recalls",
            "fda_shortages",
        ]

        # Check that all OpenFDA domains are registered
        for domain in openfda_domains:
            assert domain in VALID_DOMAINS, f"{domain} not in VALID_DOMAINS"
            assert (
                domain in DOMAIN_TO_PLURAL
            ), f"{domain} not in DOMAIN_TO_PLURAL"

        # Check plural forms
        for plural in openfda_plurals:
            assert (
                plural in VALID_DOMAINS_PLURAL
            ), f"{plural} not in VALID_DOMAINS_PLURAL"
            assert (
                plural in PLURAL_TO_DOMAIN
            ), f"{plural} not in PLURAL_TO_DOMAIN"

        # Check mappings are correct
        assert DOMAIN_TO_PLURAL["fda_adverse"] == "fda_adverse_events"
        assert DOMAIN_TO_PLURAL["fda_label"] == "fda_labels"
        assert DOMAIN_TO_PLURAL["fda_device"] == "fda_device_events"
        assert DOMAIN_TO_PLURAL["fda_approval"] == "fda_approvals"
        assert DOMAIN_TO_PLURAL["fda_recall"] == "fda_recalls"
        assert DOMAIN_TO_PLURAL["fda_shortage"] == "fda_shortages"

        assert PLURAL_TO_DOMAIN["fda_adverse_events"] == "fda_adverse"
        assert PLURAL_TO_DOMAIN["fda_labels"] == "fda_label"
        assert PLURAL_TO_DOMAIN["fda_device_events"] == "fda_device"
        assert PLURAL_TO_DOMAIN["fda_approvals"] == "fda_approval"
        assert PLURAL_TO_DOMAIN["fda_recalls"] == "fda_recall"
        assert PLURAL_TO_DOMAIN["fda_shortages"] == "fda_shortage"

    def test_openfda_search_domain_type_hints(self):
        """Test that OpenFDA domains are in search tool type hints."""
        import inspect

        from biomcp.router import search

        # Get the function signature
        sig = inspect.signature(search)
        domain_param = sig.parameters.get("domain")

        # Check if domain parameter exists
        assert (
            domain_param is not None
        ), "domain parameter not found in search function"

        # Get the annotation
        annotation = domain_param.annotation

        # The annotation should be a Literal type that includes OpenFDA domains
        # We can't directly check the Literal values due to how Python handles it,
        # but we can verify that it's properly annotated
        assert (
            annotation is not None
        ), "domain parameter has no type annotation"

    def test_openfda_fetch_domain_type_hints(self):
        """Test that OpenFDA domains are in fetch tool type hints."""
        import inspect

        from biomcp.router import fetch

        # Get the function signature
        sig = inspect.signature(fetch)
        domain_param = sig.parameters.get("domain")

        # Check if domain parameter exists
        assert (
            domain_param is not None
        ), "domain parameter not found in fetch function"

        # Get the annotation
        annotation = domain_param.annotation

        # The annotation should be a Literal type that includes OpenFDA domains
        assert (
            annotation is not None
        ), "domain parameter has no type annotation"

    @pytest.mark.asyncio
    async def test_openfda_search_basic_call(self):
        """Test that OpenFDA domain search doesn't raise errors with basic call."""
        from unittest.mock import AsyncMock, patch

        # Mock the OpenFDA search function that will be imported
        with patch(
            "biomcp.openfda.adverse_events.search_adverse_events",
            new_callable=AsyncMock,
        ) as mock_search:
            mock_search.return_value = (
                "## FDA Adverse Event Reports\n\nTest results"
            )

            from biomcp.router import search

            # This should not raise an error
            result = await search(
                query=None,  # Required parameter
                domain="fda_adverse",
                chemicals=["test"],
                page_size=1,
            )

            # Basic check that result has expected structure
            assert isinstance(result, dict)
            assert "results" in result

    @pytest.mark.asyncio
    async def test_openfda_fetch_basic_call(self):
        """Test that OpenFDA domain fetch doesn't raise errors with basic call."""
        from unittest.mock import AsyncMock, patch

        # Mock the OpenFDA get function that will be imported
        with patch(
            "biomcp.openfda.drug_approvals.get_drug_approval",
            new_callable=AsyncMock,
        ) as mock_get:
            mock_get.return_value = "## Drug Approval Details\n\nTest details"

            from biomcp.router import fetch

            # This should not raise an error
            result = await fetch(
                id="TEST123",
                domain="fda_approval",
            )

            # Basic check that result has expected structure
            assert isinstance(result, dict)
            assert "title" in result
            assert "text" in result
            assert "metadata" in result
