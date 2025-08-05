"""Unit tests for NCI CTS API integration."""

from unittest.mock import patch

import pytest

from biomcp.biomarkers import search_biomarkers
from biomcp.diseases.search import search_diseases
from biomcp.integrations.cts_api import CTSAPIError, make_cts_request
from biomcp.interventions import search_interventions
from biomcp.organizations import get_organization, search_organizations
from biomcp.trials.nci_getter import get_trial_nci
from biomcp.trials.nci_search import convert_query_to_nci, search_trials_nci
from biomcp.trials.search import TrialQuery


class TestCTSAPIIntegration:
    """Test CTS API helper functions."""

    @pytest.mark.asyncio
    async def test_make_cts_request_no_api_key(self):
        """Test that missing API key raises appropriate error."""
        with (
            patch.dict("os.environ", {}, clear=True),
            pytest.raises(CTSAPIError, match="NCI API key required"),
        ):
            await make_cts_request("https://example.com/api")

    @pytest.mark.asyncio
    async def test_make_cts_request_with_api_key(self):
        """Test successful request with API key."""
        with patch("biomcp.integrations.cts_api.request_api") as mock_request:
            mock_request.return_value = ({"data": "test"}, None)

            result = await make_cts_request(
                "https://example.com/api", api_key="test-key"
            )

            assert result == {"data": "test"}
            mock_request.assert_called_once()

            # Verify headers were included
            call_args = mock_request.call_args
            request_data = call_args.kwargs["request"]
            assert "_headers" in request_data


class TestOrganizationsModule:
    """Test organizations module functions."""

    @pytest.mark.asyncio
    async def test_search_organizations(self):
        """Test organization search."""
        with patch(
            "biomcp.organizations.search.make_cts_request"
        ) as mock_request:
            mock_request.return_value = {
                "data": [{"id": "ORG001", "name": "Test Cancer Center"}],
                "total": 1,
            }

            result = await search_organizations(
                name="Cancer Center", api_key="test-key"
            )

            assert result["total"] == 1
            assert len(result["organizations"]) == 1
            assert result["organizations"][0]["name"] == "Test Cancer Center"

    @pytest.mark.asyncio
    async def test_get_organization(self):
        """Test getting specific organization."""
        with patch(
            "biomcp.organizations.getter.make_cts_request"
        ) as mock_request:
            mock_request.return_value = {
                "data": {
                    "id": "ORG001",
                    "name": "Test Cancer Center",
                    "type": "Academic",
                }
            }

            result = await get_organization("ORG001", api_key="test-key")

            assert result["id"] == "ORG001"
            assert result["name"] == "Test Cancer Center"
            assert result["type"] == "Academic"


class TestInterventionsModule:
    """Test interventions module functions."""

    @pytest.mark.asyncio
    async def test_search_interventions(self):
        """Test intervention search."""
        with patch(
            "biomcp.interventions.search.make_cts_request"
        ) as mock_request:
            mock_request.return_value = {
                "data": [
                    {"id": "INT001", "name": "Pembrolizumab", "type": "Drug"}
                ],
                "total": 1,
            }

            result = await search_interventions(
                name="Pembrolizumab", api_key="test-key"
            )

            assert result["total"] == 1
            assert len(result["interventions"]) == 1
            assert result["interventions"][0]["name"] == "Pembrolizumab"


class TestBiomarkersModule:
    """Test biomarkers module functions."""

    @pytest.mark.asyncio
    async def test_search_biomarkers(self):
        """Test biomarker search."""
        with patch(
            "biomcp.biomarkers.search.make_cts_request"
        ) as mock_request:
            mock_request.return_value = {
                "data": [{"id": "BIO001", "name": "PD-L1", "gene": "CD274"}],
                "total": 1,
            }

            result = await search_biomarkers(name="PD-L1", api_key="test-key")

            assert result["total"] == 1
            assert len(result["biomarkers"]) == 1
            assert result["biomarkers"][0]["name"] == "PD-L1"


class TestDiseasesModule:
    """Test diseases module functions."""

    @pytest.mark.asyncio
    async def test_search_diseases_nci(self):
        """Test disease search via NCI API."""
        with patch("biomcp.diseases.search.make_cts_request") as mock_request:
            mock_request.return_value = {
                "data": [
                    {
                        "id": "DIS001",
                        "name": "Melanoma",
                        "synonyms": ["Malignant Melanoma"],
                    }
                ],
                "total": 1,
            }

            result = await search_diseases(name="Melanoma", api_key="test-key")

            assert result["total"] == 1
            assert len(result["diseases"]) == 1
            assert result["diseases"][0]["name"] == "Melanoma"


class TestNCITrialIntegration:
    """Test NCI trial search and getter."""

    @pytest.mark.asyncio
    async def test_convert_query_to_nci(self):
        """Test converting TrialQuery to NCI parameters."""
        query = TrialQuery(
            conditions=["melanoma"],
            phase="PHASE2",
            recruiting_status="OPEN",
            allow_brain_mets=True,
        )

        # Mock the disease/intervention lookups
        with (
            patch("biomcp.trials.nci_search.search_diseases") as mock_diseases,
            patch(
                "biomcp.trials.nci_search.search_interventions"
            ) as mock_interventions,
        ):
            mock_diseases.return_value = {"diseases": []}
            mock_interventions.return_value = {"interventions": []}

            params = await convert_query_to_nci(query)

            assert params["diseases"] == ["melanoma"]
            assert params["phase"] == "II"
            assert params["recruitment_status"] == [
                "recruiting",
                "enrolling_by_invitation",
            ]
            assert params["accepts_brain_mets"] is True

    @pytest.mark.asyncio
    async def test_search_trials_nci(self):
        """Test NCI trial search."""
        query = TrialQuery(conditions=["melanoma"])

        with (
            patch(
                "biomcp.trials.nci_search.convert_query_to_nci"
            ) as mock_convert,
            patch("biomcp.trials.nci_search.make_cts_request") as mock_request,
        ):
            mock_convert.return_value = {"diseases": ["melanoma"]}
            mock_request.return_value = {
                "data": [
                    {
                        "nct_id": "NCT12345",
                        "title": "Test Trial",
                        "phase": "II",
                    }
                ],
                "total": 1,
            }

            result = await search_trials_nci(query, api_key="test-key")

            assert result["total"] == 1
            assert result["source"] == "nci"
            assert len(result["trials"]) == 1
            assert result["trials"][0]["nct_id"] == "NCT12345"

    @pytest.mark.asyncio
    async def test_get_trial_nci(self):
        """Test getting specific trial from NCI."""
        with patch(
            "biomcp.trials.nci_getter.make_cts_request"
        ) as mock_request:
            mock_request.return_value = {
                "data": {
                    "nct_id": "NCT12345",
                    "title": "Test Trial",
                    "phase": "II",
                    "overall_status": "Recruiting",
                }
            }

            result = await get_trial_nci("NCT12345", api_key="test-key")

            assert result["nct_id"] == "NCT12345"
            assert result["title"] == "Test Trial"
            assert result["phase"] == "II"
