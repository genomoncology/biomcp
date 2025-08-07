"""
Unit tests for OpenFDA drug labels integration.
"""

from unittest.mock import patch

import pytest

from biomcp.openfda.drug_labels import get_drug_label, search_drug_labels


@pytest.mark.asyncio
async def test_search_drug_labels_by_name():
    """Test searching drug labels by name."""
    mock_response = {
        "meta": {"results": {"total": 5}},
        "results": [
            {
                "set_id": "abc123",
                "openfda": {
                    "brand_name": ["KEYTRUDA"],
                    "generic_name": ["PEMBROLIZUMAB"],
                    "application_number": ["BLA125514"],
                    "manufacturer_name": ["MERCK"],
                    "route": ["INTRAVENOUS"],
                },
                "indications_and_usage": [
                    "KEYTRUDA is indicated for the treatment of patients with unresectable or metastatic melanoma."
                ],
                "boxed_warning": [
                    "Immune-mediated adverse reactions can occur."
                ],
            }
        ],
    }

    with patch(
        "biomcp.openfda.drug_labels.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_drug_labels(name="pembrolizumab", limit=10)

        # Verify request
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "pembrolizumab" in call_args[0][1]["search"].lower()

        # Check output
        assert "FDA Drug Labels" in result
        assert "KEYTRUDA" in result
        assert "PEMBROLIZUMAB" in result
        assert "melanoma" in result
        assert "BOXED WARNING" in result
        assert "Immune-mediated" in result
        assert "abc123" in result


@pytest.mark.asyncio
async def test_search_drug_labels_by_indication():
    """Test searching drug labels by indication."""
    mock_response = {
        "meta": {"results": {"total": 10}},
        "results": [
            {
                "set_id": "xyz789",
                "openfda": {
                    "brand_name": ["DRUG X"],
                    "generic_name": ["GENERIC X"],
                },
                "indications_and_usage": [
                    "Indicated for breast cancer treatment"
                ],
            }
        ],
    }

    with patch(
        "biomcp.openfda.drug_labels.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_drug_labels(indication="breast cancer")

        # Verify request
        call_args = mock_request.call_args
        assert "breast cancer" in call_args[0][1]["search"].lower()

        # Check output
        assert "breast cancer" in result
        assert "10 labels" in result


@pytest.mark.asyncio
async def test_search_drug_labels_no_params():
    """Test that searching without parameters returns helpful message."""
    result = await search_drug_labels()

    assert "Please specify" in result
    assert "drug name, indication, or label section" in result
    assert "Examples:" in result


@pytest.mark.asyncio
async def test_search_drug_labels_boxed_warning_filter():
    """Test filtering for drugs with boxed warnings."""
    mock_response = {
        "meta": {"results": {"total": 3}},
        "results": [
            {
                "set_id": "warn123",
                "openfda": {"brand_name": ["WARNING DRUG"]},
                "boxed_warning": ["Serious warning text"],
            }
        ],
    }

    with patch(
        "biomcp.openfda.drug_labels.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await search_drug_labels(boxed_warning=True)

        # Verify boxed warning filter in search
        call_args = mock_request.call_args
        assert "_exists_:boxed_warning" in call_args[0][1]["search"]

        # Check output
        assert "WARNING DRUG" in result
        assert "Serious warning" in result


@pytest.mark.asyncio
async def test_get_drug_label_detail():
    """Test getting detailed drug label."""
    mock_response = {
        "results": [
            {
                "set_id": "detail123",
                "openfda": {
                    "brand_name": ["DETAILED DRUG"],
                    "generic_name": ["GENERIC DETAILED"],
                    "application_number": ["NDA123456"],
                    "manufacturer_name": ["PHARMA CORP"],
                    "route": ["ORAL"],
                    "pharm_class_epc": ["KINASE INHIBITOR"],
                },
                "boxed_warning": ["Serious boxed warning"],
                "indications_and_usage": ["Indicated for cancer"],
                "dosage_and_administration": ["Take once daily"],
                "contraindications": ["Do not use if allergic"],
                "warnings_and_precautions": ["Monitor liver function"],
                "adverse_reactions": ["Common: nausea, fatigue"],
                "drug_interactions": ["Avoid with CYP3A4 inhibitors"],
                "clinical_pharmacology": ["Mechanism of action details"],
                "clinical_studies": ["Phase 3 trial results"],
            }
        ]
    }

    with patch(
        "biomcp.openfda.drug_labels.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        result = await get_drug_label("detail123")

        # Verify request
        mock_request.assert_called_once()
        call_args = mock_request.call_args
        assert "detail123" in call_args[0][1]["search"]

        # Check detailed output
        assert "DETAILED DRUG" in result
        assert "GENERIC DETAILED" in result
        assert "NDA123456" in result
        assert "PHARMA CORP" in result
        assert "ORAL" in result
        assert "KINASE INHIBITOR" in result
        assert "BOXED WARNING" in result
        assert "Serious boxed warning" in result
        assert "INDICATIONS AND USAGE" in result
        assert "Indicated for cancer" in result
        assert "DOSAGE AND ADMINISTRATION" in result
        assert "Take once daily" in result
        assert "CONTRAINDICATIONS" in result
        assert "WARNINGS AND PRECAUTIONS" in result
        assert "ADVERSE REACTIONS" in result
        assert "DRUG INTERACTIONS" in result


@pytest.mark.asyncio
async def test_get_drug_label_specific_sections():
    """Test getting specific sections of drug label."""
    mock_response = {
        "results": [
            {
                "set_id": "section123",
                "openfda": {"brand_name": ["SECTION DRUG"]},
                "indications_and_usage": ["Cancer indication"],
                "adverse_reactions": ["Side effects list"],
                "clinical_studies": ["Study data"],
            }
        ]
    }

    with patch(
        "biomcp.openfda.drug_labels.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = (mock_response, None)

        sections = ["indications_and_usage", "adverse_reactions"]
        result = await get_drug_label("section123", sections)

        # Check that requested sections are included
        assert "INDICATIONS AND USAGE" in result
        assert "Cancer indication" in result
        assert "ADVERSE REACTIONS" in result
        assert "Side effects list" in result
        # Clinical studies should not be in output since not requested
        assert "CLINICAL STUDIES" not in result


@pytest.mark.asyncio
async def test_get_drug_label_not_found():
    """Test handling when drug label is not found."""
    with patch(
        "biomcp.openfda.drug_labels.make_openfda_request"
    ) as mock_request:
        mock_request.return_value = ({"results": []}, None)

        result = await get_drug_label("NOTFOUND456")

        assert "NOTFOUND456" in result
        assert "not found" in result
