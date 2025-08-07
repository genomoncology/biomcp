"""
Unit tests for OpenFDA device events integration.
"""

from unittest.mock import patch

import pytest

from biomcp.openfda.device_events import get_device_event, search_device_events


@pytest.mark.asyncio
async def test_search_device_events_by_device():
    """Test searching device events by device name."""
    mock_response = {
        "meta": {"results": {"total": 3}},
        "results": [
            {
                "event_type": "M",
                "date_received": "2024-01-15",
                "device": [
                    {
                        "brand_name": "FoundationOne CDx",
                        "manufacturer_d_name": "Foundation Medicine",
                        "model_number": "F1CDX",
                        "device_problem_text": ["False negative result"],
                        "openfda": {
                            "device_class": "2",
                            "medical_specialty_description": ["Pathology"],
                            "product_code": "PQP",
                        },
                    }
                ],
                "event_description": "Device failed to detect known mutation",
                "mdr_report_key": "MDR123456",
            }
        ],
    }

    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_device_events(device="FoundationOne", limit=10)

        # Verify request
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "FoundationOne" in call_args[0][1]["search"]
        # When searching for a specific device, genomic filter is not needed
        # The device search itself is sufficient

        # Check output
        assert "FDA Device Adverse Event Reports" in result
        assert "FoundationOne CDx" in result
        assert "Foundation Medicine" in result
        assert "False negative result" in result
        assert "Malfunction" in result
        assert "MDR123456" in result


@pytest.mark.asyncio
async def test_search_device_events_genomics_filter():
    """Test that genomics filter is applied by default."""
    mock_response = {"meta": {"results": {"total": 5}}, "results": []}

    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        await search_device_events(manufacturer="Illumina", genomics_only=True)

        # Verify genomic device codes are in search
        call_args = mock_request.call_args
        search_query = call_args[0][1]["search"]
        # Should contain at least one genomic product code
        assert any(
            code in search_query for code in ["OOI", "PQP", "OYD", "NYE"]
        )


@pytest.mark.asyncio
async def test_search_device_events_no_genomics_filter():
    """Test searching without genomics filter."""
    mock_response = {"meta": {"results": {"total": 10}}, "results": []}

    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        await search_device_events(device="pacemaker", genomics_only=False)

        # Verify no genomic product codes in search
        call_args = mock_request.call_args
        search_query = call_args[0][1]["search"]
        # Should not contain genomic product codes
        assert not any(code in search_query for code in ["OOI", "PQP", "OYD"])


@pytest.mark.asyncio
async def test_search_device_events_by_problem():
    """Test searching device events by problem description."""
    mock_response = {
        "meta": {"results": {"total": 8}},
        "results": [
            {
                "event_type": "IN",
                "device": [
                    {
                        "brand_name": "Test Device",
                        "device_problem_text": [
                            "Software malfunction",
                            "Data loss",
                        ],
                    }
                ],
                "mdr_report_key": "MDR789",
            }
        ],
    }

    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_device_events(problem="software malfunction")

        # Verify request
        call_args = mock_request.call_args
        assert "software malfunction" in call_args[0][1]["search"].lower()

        # Check output
        assert "Software malfunction" in result
        assert "Data loss" in result
        assert "Injury" in result  # IN = Injury


@pytest.mark.asyncio
async def test_search_device_events_no_params():
    """Test that searching without parameters returns helpful message."""
    result = await search_device_events()

    assert "Please specify" in result
    assert "device name, manufacturer, or problem" in result
    assert "Examples:" in result


@pytest.mark.asyncio
async def test_get_device_event_detail():
    """Test getting detailed device event report."""
    mock_response = {
        "results": [
            {
                "mdr_report_key": "MDR999888",
                "event_type": "D",
                "date_received": "2024-02-01",
                "date_of_event": "2024-01-20",
                "source_type": "M",
                "device": [
                    {
                        "brand_name": "Genomic Sequencer X",
                        "manufacturer_d_name": "GenTech Corp",
                        "model_number": "GSX-2000",
                        "catalog_number": "CAT123",
                        "lot_number": "LOT456",
                        "expiration_date_of_device": "2025-12-31",
                        "device_problem_text": [
                            "Critical failure",
                            "Sample contamination",
                        ],
                        "device_evaluated_by_manufacturer": "Y",
                        "openfda": {
                            "device_class": "3",
                            "medical_specialty_description": [
                                "Clinical Chemistry"
                            ],
                            "product_code": "OOI",
                        },
                    }
                ],
                "event_description": "Device failure led to incorrect cancer diagnosis",
                "manufacturer_narrative": "Investigation revealed component failure",
                "patient": [
                    {
                        "patient_age": "65",
                        "patient_sex": "F",
                        "date_of_death": "2024-01-25",
                        "life_threatening": "Y",
                    }
                ],
                "remedial_action": "Device recall initiated",
            }
        ]
    }

    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await get_device_event("MDR999888")

        # Verify request
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "MDR999888" in call_args[0][1]["search"]

        # Check detailed output
        assert "MDR999888" in result
        assert "Death" in result
        assert "Genomic Sequencer X" in result
        assert "GenTech Corp" in result
        assert "GSX-2000" in result
        assert "Critical failure" in result
        assert "Sample contamination" in result
        assert "Class III" in result
        assert "65 years" in result
        assert "Female" in result
        assert "2024-01-25" in result
        assert "Life-threatening" in result
        assert "Device recall initiated" in result
        assert "Investigation revealed component failure" in result


@pytest.mark.asyncio
async def test_get_device_event_not_found():
    """Test handling when device event report is not found."""
    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = ({"results": []}, None)

        result = await get_device_event("NOTFOUND789")

        assert "NOTFOUND789" in result
        assert "not found" in result


@pytest.mark.asyncio
async def test_search_device_events_error():
    """Test error handling in device event search."""
    with patch(
        "biomcp.openfda.device_events.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (None, "Network timeout")

        result = await search_device_events(device="test")

        assert "Error searching device events" in result
        assert "Network timeout" in result
