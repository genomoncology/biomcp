"""Test NCI-specific MCP tools."""

from unittest.mock import patch

import pytest

from biomcp.individual_tools import (
    nci_intervention_getter,
    nci_intervention_searcher,
    nci_organization_getter,
    nci_organization_searcher,
)


class TestOrganizationTools:
    """Test organization MCP tools."""

    @pytest.mark.asyncio
    async def test_organization_searcher_tool(self):
        """Test organization searcher MCP tool."""
        mock_results = {
            "total": 2,
            "organizations": [
                {
                    "id": "ORG001",
                    "name": "Test Cancer Center",
                    "type": "Academic",
                    "city": "Boston",
                    "state": "MA",
                    "country": "US",
                },
                {
                    "id": "ORG002",
                    "name": "Another Cancer Center",
                    "type": "Academic",
                    "city": "New York",
                    "state": "NY",
                    "country": "US",
                },
            ],
        }

        with (
            patch("biomcp.organizations.search_organizations") as mock_search,
            patch(
                "biomcp.organizations.search.format_organization_results"
            ) as mock_format,
        ):
            mock_search.return_value = mock_results
            mock_format.return_value = (
                "## Organization Search Results\n\nFound 2 organizations"
            )

            result = await nci_organization_searcher(
                name="Cancer Center",
                organization_type="Academic",
                city="Boston",
                api_key="test-key",
            )

            assert "Found 2 organizations" in result
            mock_search.assert_called_once_with(
                name="Cancer Center",
                org_type="Academic",
                city="Boston",
                state=None,
                page_size=20,
                page=1,
                api_key="test-key",
            )

    @pytest.mark.asyncio
    async def test_organization_getter_tool(self):
        """Test organization getter MCP tool."""
        mock_org = {
            "id": "ORG001",
            "name": "Test Cancer Center",
            "type": "Academic",
            "address": {
                "street": "123 Medical Way",
                "city": "Boston",
                "state": "MA",
                "zip": "02115",
                "country": "US",
            },
            "contact": {"phone": "555-1234", "email": "info@testcancer.org"},
        }

        with (
            patch("biomcp.organizations.get_organization") as mock_get,
            patch(
                "biomcp.organizations.getter.format_organization_details"
            ) as mock_format,
        ):
            mock_get.return_value = mock_org
            mock_format.return_value = (
                "## Test Cancer Center\n\nType: Academic\nLocation: Boston, MA"
            )

            result = await nci_organization_getter(
                organization_id="ORG001", api_key="test-key"
            )

            assert "Test Cancer Center" in result
            assert "Academic" in result
            mock_get.assert_called_once_with(
                org_id="ORG001",
                api_key="test-key",
            )


class TestInterventionTools:
    """Test intervention MCP tools."""

    @pytest.mark.asyncio
    async def test_intervention_searcher_tool(self):
        """Test intervention searcher MCP tool."""
        mock_results = {
            "total": 1,
            "interventions": [
                {
                    "id": "INT001",
                    "name": "Pembrolizumab",
                    "type": "Drug",
                    "synonyms": ["Keytruda", "MK-3475"],
                }
            ],
        }

        with (
            patch("biomcp.interventions.search_interventions") as mock_search,
            patch(
                "biomcp.interventions.search.format_intervention_results"
            ) as mock_format,
        ):
            mock_search.return_value = mock_results
            mock_format.return_value = (
                "## Intervention Search Results\n\nFound 1 intervention"
            )

            result = await nci_intervention_searcher(
                name="pembrolizumab",
                intervention_type="Drug",
                api_key="test-key",
            )

            assert "Found 1 intervention" in result
            mock_search.assert_called_once_with(
                name="pembrolizumab",
                intervention_type="Drug",
                synonyms=True,
                page_size=None,
                page=1,
                api_key="test-key",
            )

    @pytest.mark.asyncio
    async def test_intervention_getter_tool(self):
        """Test intervention getter MCP tool."""
        mock_intervention = {
            "id": "INT001",
            "name": "Pembrolizumab",
            "type": "Drug",
            "category": "Immunotherapy",
            "synonyms": ["Keytruda", "MK-3475"],
            "mechanism": "PD-1 inhibitor",
            "fda_approved": True,
        }

        with (
            patch("biomcp.interventions.get_intervention") as mock_get,
            patch(
                "biomcp.interventions.getter.format_intervention_details"
            ) as mock_format,
        ):
            mock_get.return_value = mock_intervention
            mock_format.return_value = (
                "## Pembrolizumab\n\nType: Drug\nMechanism: PD-1 inhibitor"
            )

            result = await nci_intervention_getter(
                intervention_id="INT001", api_key="test-key"
            )

            assert "Pembrolizumab" in result
            assert "PD-1 inhibitor" in result
            mock_get.assert_called_once_with(
                intervention_id="INT001",
                api_key="test-key",
            )


class TestToolsWithoutAPIKey:
    """Test tools handle missing API key gracefully."""

    @pytest.mark.asyncio
    async def test_organization_searcher_no_api_key(self):
        """Test organization searcher without API key."""
        from biomcp.integrations.cts_api import CTSAPIError

        with patch("biomcp.organizations.search_organizations") as mock_search:
            mock_search.side_effect = CTSAPIError("NCI API key required")

            with pytest.raises(CTSAPIError, match="NCI API key required"):
                await nci_organization_searcher(name="Cancer Center")

    @pytest.mark.asyncio
    async def test_intervention_searcher_no_api_key(self):
        """Test intervention searcher without API key."""
        from biomcp.integrations.cts_api import CTSAPIError

        with patch("biomcp.interventions.search_interventions") as mock_search:
            mock_search.side_effect = CTSAPIError("NCI API key required")

            with pytest.raises(CTSAPIError, match="NCI API key required"):
                await nci_intervention_searcher(name="pembrolizumab")


class TestElasticsearchErrorHandling:
    """Test handling of Elasticsearch bucket limit errors."""

    @pytest.mark.asyncio
    async def test_organization_searcher_elasticsearch_error(self):
        """Test organization searcher handles Elasticsearch bucket limit error gracefully."""
        from biomcp.integrations.cts_api import CTSAPIError

        error_response = {
            "status": 503,
            "detail": [
                503,
                "search_phase_execution_exception",
                {
                    "error": {
                        "caused_by": {
                            "type": "too_many_buckets_exception",
                            "reason": "Trying to create too many buckets. Must be less than or equal to: [75000] but was [75001].",
                        }
                    }
                },
            ],
        }

        with patch("biomcp.organizations.search_organizations") as mock_search:
            mock_search.side_effect = CTSAPIError(str(error_response))

            result = await nci_organization_searcher(
                city="Cleveland", api_key="test-key"
            )

            assert "Search Too Broad" in result
            assert "city AND state together" in result
            assert "city='Cleveland', state='OH'" in result

    @pytest.mark.asyncio
    async def test_intervention_searcher_elasticsearch_error(self):
        """Test intervention searcher handles Elasticsearch bucket limit error gracefully."""
        from biomcp.integrations.cts_api import CTSAPIError

        error_response = {
            "status": 503,
            "detail": "too_many_buckets_exception: Trying to create too many buckets. Must be less than or equal to: [75000]",
        }

        with patch("biomcp.interventions.search_interventions") as mock_search:
            mock_search.side_effect = CTSAPIError(str(error_response))

            result = await nci_intervention_searcher(
                intervention_type="Drug", api_key="test-key"
            )

            assert "Search Too Broad" in result
            assert "pembrolizumab" in result
            assert "CAR-T" in result


class TestBiomarkerTools:
    """Test biomarker MCP tools."""

    @pytest.mark.asyncio
    async def test_biomarker_searcher_tool(self):
        """Test biomarker searcher MCP tool."""
        from biomcp.individual_tools import nci_biomarker_searcher

        mock_results = {
            "total": 2,
            "biomarkers": [
                {
                    "id": "BIO001",
                    "name": "PD-L1 Expression",
                    "gene": "CD274",
                    "type": "expression",
                    "assay_type": "IHC",
                },
                {
                    "id": "BIO002",
                    "name": "EGFR Mutation",
                    "gene": "EGFR",
                    "type": "mutation",
                    "assay_type": "NGS",
                },
            ],
        }

        with (
            patch("biomcp.biomarkers.search_biomarkers") as mock_search,
            patch(
                "biomcp.biomarkers.search.format_biomarker_results"
            ) as mock_format,
        ):
            mock_search.return_value = mock_results
            mock_format.return_value = (
                "## Biomarker Search Results (2 found)\n\nFound 2 biomarkers"
            )

            result = await nci_biomarker_searcher(
                name="PD-L1", api_key="test-key"
            )

            assert "Found 2 biomarkers" in result
            mock_search.assert_called_once_with(
                name="PD-L1",
                biomarker_type=None,
                page_size=20,
                page=1,
                api_key="test-key",
            )


class TestNCIDiseaseTools:
    """Test NCI disease MCP tools."""

    @pytest.mark.asyncio
    async def test_nci_disease_searcher_tool(self):
        """Test NCI disease searcher MCP tool."""
        from biomcp.individual_tools import nci_disease_searcher

        mock_results = {
            "total": 2,
            "diseases": [
                {
                    "id": "C4872",
                    "name": "Breast Cancer",
                    "synonyms": ["Breast Carcinoma", "Mammary Cancer"],
                    "category": "maintype",
                },
                {
                    "id": "C3790",
                    "name": "Melanoma",
                    "synonyms": ["Malignant Melanoma"],
                    "category": "maintype",
                },
            ],
        }

        with (
            patch("biomcp.diseases.search_diseases") as mock_search,
            patch(
                "biomcp.diseases.search.format_disease_results"
            ) as mock_format,
        ):
            mock_search.return_value = mock_results
            mock_format.return_value = (
                "## Disease Search Results (2 found)\n\nFound 2 diseases"
            )

            result = await nci_disease_searcher(
                name="cancer", include_synonyms=True, api_key="test-key"
            )

            assert "Found 2 diseases" in result
            mock_search.assert_called_once_with(
                name="cancer",
                include_synonyms=True,
                category=None,
                page_size=20,
                page=1,
                api_key="test-key",
            )


class TestToolsIntegration:
    """Test MCP tools integration with actual modules."""

    @pytest.mark.asyncio
    async def test_organization_searcher_imports_work(self):
        """Test that organization searcher imports work correctly."""
        # This test verifies the dynamic imports in the tool function work
        with (
            patch("biomcp.organizations.search_organizations") as mock_search,
            patch(
                "biomcp.organizations.search.format_organization_results"
            ) as mock_format,
        ):
            mock_search.return_value = {"total": 0, "organizations": []}
            mock_format.return_value = "No organizations found"

            result = await nci_organization_searcher(
                name="Nonexistent", api_key="test-key"
            )

            assert result == "No organizations found"

    @pytest.mark.asyncio
    async def test_intervention_searcher_imports_work(self):
        """Test that intervention searcher imports work correctly."""
        # This test verifies the dynamic imports in the tool function work
        with (
            patch("biomcp.interventions.search_interventions") as mock_search,
            patch(
                "biomcp.interventions.search.format_intervention_results"
            ) as mock_format,
        ):
            mock_search.return_value = {"total": 0, "interventions": []}
            mock_format.return_value = "No interventions found"

            result = await nci_intervention_searcher(
                name="Nonexistent", api_key="test-key"
            )

            assert result == "No interventions found"
