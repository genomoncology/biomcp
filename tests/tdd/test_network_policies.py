"""Comprehensive tests for network policies and HTTP centralization."""

from pathlib import Path
from unittest.mock import patch

import pytest

from biomcp.http_client import request_api
from biomcp.utils.endpoint_registry import (
    DataType,
    EndpointCategory,
    EndpointInfo,
    EndpointRegistry,
    get_registry,
)


class TestEndpointRegistry:
    """Test the endpoint registry functionality."""

    def test_registry_initialization(self):
        """Test that registry initializes with known endpoints."""
        registry = EndpointRegistry()
        endpoints = registry.get_all_endpoints()

        # Check we have endpoints registered
        assert len(endpoints) > 0

        # Check specific endpoints exist
        assert "pubtator3_search" in endpoints
        assert "clinicaltrials_search" in endpoints
        assert "myvariant_query" in endpoints
        assert "cbioportal_api" in endpoints

    def test_get_endpoints_by_category(self):
        """Test filtering endpoints by category."""
        registry = EndpointRegistry()

        # Get biomedical literature endpoints
        lit_endpoints = registry.get_endpoints_by_category(
            EndpointCategory.BIOMEDICAL_LITERATURE
        )
        assert len(lit_endpoints) > 0
        assert all(
            e.category == EndpointCategory.BIOMEDICAL_LITERATURE
            for e in lit_endpoints.values()
        )

        # Get clinical trials endpoints
        trial_endpoints = registry.get_endpoints_by_category(
            EndpointCategory.CLINICAL_TRIALS
        )
        assert len(trial_endpoints) > 0
        assert all(
            e.category == EndpointCategory.CLINICAL_TRIALS
            for e in trial_endpoints.values()
        )

    def test_get_unique_domains(self):
        """Test getting unique domains."""
        registry = EndpointRegistry()
        domains = registry.get_unique_domains()

        assert len(domains) > 0
        assert "www.ncbi.nlm.nih.gov" in domains
        assert "clinicaltrials.gov" in domains
        assert "myvariant.info" in domains
        assert "www.cbioportal.org" in domains

    def test_endpoint_info_properties(self):
        """Test EndpointInfo dataclass properties."""
        endpoint = EndpointInfo(
            url="https://api.example.com/test",
            category=EndpointCategory.BIOMEDICAL_LITERATURE,
            data_types=[DataType.RESEARCH_ARTICLES],
            description="Test endpoint",
            compliance_notes="Test compliance",
            rate_limit="10 requests/second",
            authentication="API key required",
        )

        assert endpoint.domain == "api.example.com"
        assert endpoint.category == EndpointCategory.BIOMEDICAL_LITERATURE
        assert DataType.RESEARCH_ARTICLES in endpoint.data_types

    def test_markdown_report_generation(self):
        """Test markdown report generation."""
        registry = EndpointRegistry()
        report = registry.generate_markdown_report()

        # Check report contains expected sections
        assert "# Third-Party Endpoints Used by BioMCP" in report
        assert "## Overview" in report
        assert "## Endpoints by Category" in report
        assert "## Domain Summary" in report
        assert "## Compliance and Privacy" in report
        assert "## Network Control" in report

        # Check it mentions offline mode
        assert "BIOMCP_OFFLINE" in report

        # Check it contains actual endpoints
        assert "pubtator3" in report
        assert "clinicaltrials.gov" in report
        assert "myvariant.info" in report

    def test_save_markdown_report(self, tmp_path):
        """Test saving markdown report to file."""
        registry = EndpointRegistry()
        output_path = tmp_path / "test_endpoints.md"

        saved_path = registry.save_markdown_report(output_path)

        assert saved_path == output_path
        assert output_path.exists()

        # Read and verify content
        content = output_path.read_text()
        assert "Third-Party Endpoints Used by BioMCP" in content


class TestEndpointTracking:
    """Test endpoint tracking in HTTP client."""

    @pytest.mark.asyncio
    async def test_valid_endpoint_key(self):
        """Test that valid endpoint keys are accepted."""
        with patch("biomcp.http_client.call_http") as mock_call:
            mock_call.return_value = (200, '{"data": "test"}')

            # Should not raise an error
            result, error = await request_api(
                url="https://www.ncbi.nlm.nih.gov/research/pubtator3-api/search/",
                request={"text": "BRAF"},
                endpoint_key="pubtator3_search",
                cache_ttl=0,
            )

            assert result == {"data": "test"}
            assert error is None

    @pytest.mark.asyncio
    async def test_invalid_endpoint_key_raises_error(self):
        """Test that invalid endpoint keys raise an error."""
        with pytest.raises(ValueError, match="Unknown endpoint key"):
            await request_api(
                url="https://api.example.com/test",
                request={"test": "data"},
                endpoint_key="invalid_endpoint_key",
                cache_ttl=0,
            )

    @pytest.mark.asyncio
    async def test_no_endpoint_key_allowed(self):
        """Test that requests without endpoint keys are allowed."""
        with patch("biomcp.http_client.call_http") as mock_call:
            mock_call.return_value = (200, '{"data": "test"}')

            # Should not raise an error
            result, error = await request_api(
                url="https://api.example.com/test",
                request={"test": "data"},
                cache_ttl=0,
            )

            assert result == {"data": "test"}
            assert error is None


class TestHTTPImportChecks:
    """Test the HTTP import checking script."""

    def test_check_script_exists(self):
        """Test that the check script exists."""
        script_path = (
            Path(__file__).parent.parent.parent
            / "scripts"
            / "check_http_imports.py"
        )
        assert script_path.exists()

    def test_allowed_files_configured(self):
        """Test that allowed files are properly configured."""
        # Import the script module
        import sys

        script_path = Path(__file__).parent.parent.parent / "scripts"
        sys.path.insert(0, str(script_path))

        try:
            from check_http_imports import ALLOWED_FILES, HTTP_LIBRARIES

            # Check essential files are allowed
            assert "http_client.py" in ALLOWED_FILES
            assert "http_client_simple.py" in ALLOWED_FILES

            # Check we're checking for the right libraries
            assert "httpx" in HTTP_LIBRARIES
            assert "aiohttp" in HTTP_LIBRARIES
            assert "requests" in HTTP_LIBRARIES
        finally:
            sys.path.pop(0)


class TestGlobalRegistry:
    """Test the global registry instance."""

    def test_get_registry_returns_same_instance(self):
        """Test that get_registry returns the same instance."""
        registry1 = get_registry()
        registry2 = get_registry()

        assert registry1 is registry2

    def test_global_registry_has_endpoints(self):
        """Test that the global registry has endpoints."""
        registry = get_registry()
        endpoints = registry.get_all_endpoints()

        assert len(endpoints) > 0
