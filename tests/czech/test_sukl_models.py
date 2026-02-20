"""Unit tests for SUKL Pydantic models."""

from biomcp.czech.sukl.models import (
    ActiveSubstance,
    AvailabilityStatus,
    Drug,
    DrugSearchResult,
    DrugSummary,
)


class TestSuklModels:
    def test_active_substance(self):
        s = ActiveSubstance(name="Ibuprofen", strength="400 mg")
        assert s.name == "Ibuprofen"
        assert s.strength == "400 mg"

    def test_active_substance_no_strength(self):
        s = ActiveSubstance(name="Ibuprofen")
        assert s.strength is None

    def test_availability_status(self):
        a = AvailabilityStatus(status="available")
        assert a.status == "available"
        assert a.last_checked is None
        assert a.note is None

    def test_drug_minimal(self):
        d = Drug(sukl_code="0000123", name="Test Drug")
        assert d.sukl_code == "0000123"
        assert d.source == "SUKL"
        assert d.active_substances == []

    def test_drug_full(self):
        d = Drug(
            sukl_code="0000123",
            name="Nurofen",
            active_substances=[
                ActiveSubstance(
                    name="Ibuprofen", strength="400 mg"
                )
            ],
            pharmaceutical_form="tablety",
            atc_code="M01AE01",
            registration_number="07/123/01-C",
            mah="Reckitt",
            registration_valid_to="2028-12-31",
            availability=AvailabilityStatus(
                status="available"
            ),
            spc_url="https://example.com/spc",
            pil_url="https://example.com/pil",
        )
        assert len(d.active_substances) == 1
        assert d.availability.status == "available"

    def test_drug_summary(self):
        s = DrugSummary(
            sukl_code="0000123", name="Test"
        )
        assert s.active_substance is None

    def test_drug_search_result(self):
        r = DrugSearchResult(
            total=1,
            page=1,
            page_size=10,
            results=[
                DrugSummary(
                    sukl_code="001", name="Drug1"
                )
            ],
        )
        assert r.total == 1
        assert len(r.results) == 1

    def test_drug_search_result_empty(self):
        r = DrugSearchResult(
            total=0, page=1, page_size=10
        )
        assert r.results == []
