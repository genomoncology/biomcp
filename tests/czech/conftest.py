"""Shared test fixtures for Czech healthcare module tests."""

import json
from unittest.mock import MagicMock

import pytest


@pytest.fixture
def mock_httpx_response():
    """Create a mock httpx response factory."""

    def _make_response(
        data: dict | list | str,
        status_code: int = 200,
    ):
        response = MagicMock()
        response.status_code = status_code
        response.is_success = 200 <= status_code < 300
        if isinstance(data, str):
            response.text = data
            response.json.return_value = json.loads(data)
        else:
            response.text = json.dumps(data)
            response.json.return_value = data
        response.raise_for_status = MagicMock()
        if status_code >= 400:
            from httpx import HTTPStatusError

            response.raise_for_status.side_effect = HTTPStatusError(
                f"HTTP {status_code}",
                request=MagicMock(),
                response=response,
            )
        return response

    return _make_response


@pytest.fixture
def sample_sukl_drug():
    """Sample SUKL drug data for testing."""
    return {
        "sukl_code": "0000123",
        "name": "Nurofen 400mg",
        "active_substances": [
            {"name": "Ibuprofen", "strength": "400 mg"}
        ],
        "pharmaceutical_form": "potahované tablety",
        "atc_code": "M01AE01",
        "registration_number": "07/123/01-C",
        "mah": "Reckitt Benckiser",
        "registration_valid_to": "2028-12-31",
        "availability": {
            "status": "available",
            "last_checked": "2026-02-17T10:00:00Z",
            "note": None,
        },
        "spc_url": "https://prehledy.sukl.cz/spc/0000123",
        "pil_url": "https://prehledy.sukl.cz/pil/0000123",
        "source": "SUKL",
    }


@pytest.fixture
def sample_sukl_search_response():
    """Sample SUKL search API response."""
    return {
        "total": 2,
        "page": 1,
        "page_size": 10,
        "results": [
            {
                "sukl_code": "0000123",
                "name": "Nurofen 400mg",
                "active_substance": "Ibuprofen",
                "atc_code": "M01AE01",
                "pharmaceutical_form": "potahované tablety",
            },
            {
                "sukl_code": "0000456",
                "name": "Ibuprofen AL 400",
                "active_substance": "Ibuprofen",
                "atc_code": "M01AE01",
                "pharmaceutical_form": "potahované tablety",
            },
        ],
    }


@pytest.fixture
def sample_nrpzs_provider():
    """Sample NRPZS provider data."""
    return {
        "provider_id": "12345",
        "name": "MUDr. Jan Novák",
        "legal_form": "fyzická osoba",
        "ico": "12345678",
        "address": {
            "street": "Hlavní 123",
            "city": "Praha",
            "postal_code": "11000",
            "region": "Praha",
        },
        "specialties": ["kardiologie"],
        "care_types": ["ambulantní"],
        "workplaces": [
            {
                "workplace_id": "W001",
                "name": "Ordinace kardiologie",
                "address": {
                    "street": "Hlavní 123",
                    "city": "Praha",
                    "postal_code": "11000",
                    "region": "Praha",
                },
                "specialties": ["kardiologie"],
                "contact": {
                    "phone": "+420 123 456 789",
                    "email": "novak@example.cz",
                    "website": None,
                },
            }
        ],
        "registration_number": "REG-12345",
        "source": "NRPZS",
    }


SAMPLE_CLAML_XML = """\
<?xml version="1.0" encoding="UTF-8"?>
<ClaML version="2.0.0">
  <Title name="MKN-10" version="2018"/>
  <Class code="X" kind="chapter">
    <Rubric kind="preferred">
      <Label xml:lang="cs">Nemoci dýchací soustavy</Label>
    </Rubric>
    <SubClass code="J00-J06"/>
  </Class>
  <Class code="J00-J06" kind="block">
    <SuperClass code="X"/>
    <Rubric kind="preferred">
      <Label xml:lang="cs">\
Akutní infekce horních cest dýchacích</Label>
    </Rubric>
    <SubClass code="J06"/>
  </Class>
  <Class code="J06" kind="category">
    <SuperClass code="J00-J06"/>
    <Rubric kind="preferred">
      <Label xml:lang="cs">\
Akutní infekce horních cest dýchacích na více \
a neurčených místech</Label>
    </Rubric>
    <SubClass code="J06.9"/>
  </Class>
  <Class code="J06.9" kind="category">
    <SuperClass code="J06"/>
    <Rubric kind="preferred">
      <Label xml:lang="cs">\
Akutní infekce horních cest dýchacích NS</Label>
    </Rubric>
  </Class>
</ClaML>
"""
