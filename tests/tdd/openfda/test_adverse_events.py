"""
Unit tests for OpenFDA adverse events integration.
"""

from unittest.mock import patch

import pytest

from biomcp.openfda.adverse_events import (
    get_adverse_event,
    search_adverse_events,
)


@pytest.mark.asyncio
async def test_search_adverse_events_by_drug():
    """Test searching adverse events by drug name."""
    mock_response = {
        "meta": {"results": {"total": 100}},
        "results": [
            {
                "patient": {
                    "drug": [
                        {
                            "medicinalproduct": "IMATINIB",
                            "openfda": {
                                "brand_name": ["GLEEVEC"],
                                "generic_name": ["IMATINIB MESYLATE"],
                            },
                        }
                    ],
                    "reaction": [
                        {"reactionmeddrapt": "NAUSEA"},
                        {"reactionmeddrapt": "FATIGUE"},
                    ],
                    "patientonsetage": "45",
                    "patientsex": 2,
                },
                "serious": "1",
                "seriousnesshospitalization": "1",
                "receivedate": "20240115",
            }
        ],
    }

    with patch(
        "biomcp.openfda.adverse_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_adverse_events(drug="imatinib", limit=10)

        # Verify the request was made correctly
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "imatinib" in call_args[0][1]["search"].lower()

        # Check the output contains expected information
        assert "FDA Adverse Event Reports" in result
        assert "imatinib" in result.lower()
        assert "NAUSEA" in result
        assert "FATIGUE" in result
        assert "100 reports" in result


@pytest.mark.asyncio
async def test_search_adverse_events_by_reaction():
    """Test searching adverse events by reaction."""
    mock_response = {
        "meta": {"results": {"total": 50}},
        "results": [
            {
                "patient": {
                    "drug": [{"medicinalproduct": "ASPIRIN"}],
                    "reaction": [{"reactionmeddrapt": "HEADACHE"}],
                },
                "serious": "0",
                "receivedate": "20240201",
            }
        ],
    }

    with patch(
        "biomcp.openfda.adverse_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_adverse_events(reaction="headache", limit=10)

        # Verify the request
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "headache" in call_args[0][1]["search"].lower()

        # Check output
        assert "HEADACHE" in result
        assert "50 reports" in result


@pytest.mark.asyncio
async def test_search_adverse_events_no_params():
    """Test that searching without parameters returns helpful message."""
    result = await search_adverse_events()

    assert "Please specify" in result
    assert "drug name or reaction" in result
    assert "Examples:" in result


@pytest.mark.asyncio
async def test_search_adverse_events_no_results():
    """Test handling when no results are found."""
    with patch(
        "biomcp.openfda.adverse_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = ({"results": []}, None)

        result = await search_adverse_events(drug="nonexistentdrug")

        assert "No adverse event reports found" in result
        assert "nonexistentdrug" in result


@pytest.mark.asyncio
async def test_search_adverse_events_error():
    """Test error handling in adverse event search."""
    with patch(
        "biomcp.openfda.adverse_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (None, "API rate limit exceeded")

        result = await search_adverse_events(drug="aspirin")

        assert "Error searching adverse events" in result
        assert "API rate limit exceeded" in result


@pytest.mark.asyncio
async def test_get_adverse_event_detail():
    """Test getting detailed adverse event report."""
    mock_response = {
        "results": [
            {
                "safetyreportid": "12345678",
                "patient": {
                    "patientonsetage": "55",
                    "patientsex": 1,
                    "patientweight": "75",
                    "drug": [
                        {
                            "medicinalproduct": "DRUG A",
                            "drugindication": "HYPERTENSION",
                            "drugdosagetext": "100mg daily",
                            "drugadministrationroute": "048",
                            "actiondrug": 4,
                        }
                    ],
                    "reaction": [
                        {"reactionmeddrapt": "DIZZINESS", "reactionoutcome": 1}
                    ],
                },
                "serious": "1",
                "seriousnesshospitalization": "1",
                "receivedate": "20240115",
                "reporttype": 1,
            }
        ]
    }

    with patch(
        "biomcp.openfda.adverse_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await get_adverse_event("12345678")

        # Verify request
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "12345678" in call_args[0][1]["search"]

        # Check detailed output
        assert "12345678" in result
        assert "Patient Information" in result
        assert "55 years" in result
        assert "Male" in result
        assert "75 kg" in result
        assert "DRUG A" in result
        assert "HYPERTENSION" in result
        assert "100mg daily" in result
        assert "DIZZINESS" in result
        assert "Recovered/Resolved" in result


@pytest.mark.asyncio
async def test_get_adverse_event_not_found():
    """Test handling when adverse event report is not found."""
    with patch(
        "biomcp.openfda.adverse_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = ({"results": []}, None)

        result = await get_adverse_event("NOTFOUND123")

        assert "NOTFOUND123" in result
        assert "not found" in result
