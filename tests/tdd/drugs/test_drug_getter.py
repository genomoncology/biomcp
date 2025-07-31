"""Unit tests for drug information retrieval."""

import json

import pytest

from biomcp.drugs.getter import get_drug


class TestDrugGetter:
    """Test drug information retrieval."""

    @pytest.fixture
    def mock_drug_response(self):
        """Mock drug response from MyChem.info."""
        return {
            "_id": "CHEMBL941",
            "name": "Imatinib",
            "drugbank": {
                "id": "DB00619",
                "name": "Imatinib",
                "description": "Imatinib is a tyrosine kinase inhibitor...",
                "indication": "Treatment of chronic myeloid leukemia...",
                "mechanism_of_action": "Inhibits BCR-ABL tyrosine kinase...",
                "products": {"name": ["Gleevec", "Glivec"]},
            },
            "chembl": {
                "molecule_chembl_id": "CHEMBL941",
                "pref_name": "IMATINIB",
            },
            "pubchem": {"cid": 5291},
            "chebi": {"id": "CHEBI:45783", "name": "imatinib"},
            "inchikey": "KTUFNOKKBVMGRW-UHFFFAOYSA-N",
            "formula": "C29H31N7O",
        }

    @pytest.mark.asyncio
    async def test_get_drug_by_name(self, monkeypatch, mock_drug_response):
        """Test getting drug by name."""
        # Mock the API call
        call_count = 0
        responses = [
            # Query response
            ({"hits": [{"_id": "CHEMBL941"}]}, None),
            # Get response
            (mock_drug_response, None),
        ]

        async def mock_request_api(url, request, method, domain):
            nonlocal call_count
            result = responses[call_count]
            call_count += 1
            return result

        monkeypatch.setattr("biomcp.http_client.request_api", mock_request_api)

        result = await get_drug("imatinib")

        assert "## Drug: Imatinib" in result
        assert "DrugBank ID**: DB00619" in result
        assert "ChEMBL ID**: CHEMBL941" in result
        assert "Formula**: C29H31N7O" in result
        assert "Trade Names**: Gleevec, Glivec" in result
        assert "External Links" in result
        assert "DrugBank](https://www.drugbank.ca/drugs/DB00619)" in result

    @pytest.mark.asyncio
    async def test_get_drug_by_id(self, monkeypatch, mock_drug_response):
        """Test getting drug by DrugBank ID."""

        # Mock the API call
        async def mock_request_api(url, request, method, domain):
            return (mock_drug_response, None)

        monkeypatch.setattr("biomcp.http_client.request_api", mock_request_api)

        result = await get_drug("DB00619")

        assert "## Drug: Imatinib" in result
        assert "DrugBank ID**: DB00619" in result

    @pytest.mark.asyncio
    async def test_get_drug_json_output(self, monkeypatch, mock_drug_response):
        """Test getting drug with JSON output."""

        # Mock the API call
        async def mock_request_api(url, request, method, domain):
            return (mock_drug_response, None)

        monkeypatch.setattr("biomcp.http_client.request_api", mock_request_api)

        result = await get_drug("DB00619", output_json=True)
        data = json.loads(result)

        assert data["drug_id"] == "CHEMBL941"
        assert data["name"] == "Imatinib"
        assert data["drugbank_id"] == "DB00619"
        assert (
            data["_links"]["DrugBank"]
            == "https://www.drugbank.ca/drugs/DB00619"
        )

    @pytest.mark.asyncio
    async def test_drug_not_found(self, monkeypatch):
        """Test drug not found."""

        # Mock the API call
        async def mock_request_api(url, request, method, domain):
            return ({"hits": []}, None)

        monkeypatch.setattr("biomcp.http_client.request_api", mock_request_api)

        result = await get_drug("INVALID_DRUG_XYZ")

        assert "Drug 'INVALID_DRUG_XYZ' not found" in result

    @pytest.mark.asyncio
    async def test_drug_with_description_truncation(self, monkeypatch):
        """Test drug with long description gets truncated."""
        long_desc = "A" * 600
        mock_response = {
            "_id": "TEST001",
            "name": "TestDrug",
            "drugbank": {"id": "DB99999", "description": long_desc},
        }

        async def mock_request_api(url, request, method, domain):
            return (mock_response, None)

        monkeypatch.setattr("biomcp.http_client.request_api", mock_request_api)

        result = await get_drug("DB99999")

        assert "Description" in result
        assert "A" * 500 in result
        assert "..." in result  # Truncation indicator

    @pytest.mark.asyncio
    async def test_drug_error_handling(self, monkeypatch):
        """Test error handling."""

        # Mock the API call to raise an exception
        async def mock_request_api(url, request, method, domain):
            raise Exception("API error")

        monkeypatch.setattr("biomcp.http_client.request_api", mock_request_api)

        result = await get_drug("imatinib")

        # When an exception occurs, it's caught and the drug is reported as not found
        assert "Drug 'imatinib' not found in MyChem.info" in result
