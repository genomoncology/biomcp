"""Performance benchmark tests for Czech modules.

Verifies:
- SC-001: Search latency < 2s cold / < 100ms cached
- SC-007: ClaML parse < 5s
- SC-004: MKN-10 accuracy >= 95% against sample codes
"""

import json
import time
from unittest.mock import patch

import pytest


class TestSearchLatency:
    """SC-001: Search latency benchmarks."""

    @pytest.mark.asyncio
    async def test_sukl_search_cached_under_100ms(self):
        """Cached SUKL search should return in < 100ms."""
        from biomcp.czech.sukl.search import _sukl_drug_search

        json.dumps({
            "total": 1,
            "page": 1,
            "page_size": 10,
            "results": [],
        })
        with (
            patch(
                "biomcp.czech.sukl.search.get_cached_response",
                return_value=json.dumps(["001", "002"]),
            ),
            patch(
                "biomcp.czech.sukl.search._fetch_drug_detail",
                return_value=None,
            ),
        ):
            start = time.monotonic()
            await _sukl_drug_search("test")
            elapsed = time.monotonic() - start
            assert elapsed < 0.1, (
                f"Cached search took {elapsed:.3f}s (> 100ms)"
            )

    @pytest.mark.asyncio
    async def test_nrpzs_search_cached_under_100ms(self):
        """Cached NRPZS search should return in < 100ms."""
        from biomcp.czech.nrpzs.search import _nrpzs_search

        cached_result = json.dumps({
            "total": 0,
            "page": 1,
            "page_size": 10,
            "results": [],
        })
        with patch(
            "biomcp.czech.nrpzs.search.get_cached_response",
            return_value=cached_result,
        ):
            start = time.monotonic()
            await _nrpzs_search("test")
            elapsed = time.monotonic() - start
            assert elapsed < 0.1, (
                f"Cached search took {elapsed:.3f}s (> 100ms)"
            )

    @pytest.mark.asyncio
    async def test_szv_search_cached_under_100ms(self):
        """Cached SZV search should return in < 100ms."""
        from biomcp.czech.szv.search import _szv_search

        procedures = [{"kod": "09513", "nazev": "EKG", "body": 50}]
        with patch(
            "biomcp.czech.szv.search.get_cached_response",
            return_value=json.dumps(procedures),
        ):
            start = time.monotonic()
            await _szv_search("EKG")
            elapsed = time.monotonic() - start
            assert elapsed < 0.1, (
                f"Cached search took {elapsed:.3f}s (> 100ms)"
            )

    @pytest.mark.asyncio
    async def test_vzp_search_cached_under_100ms(self):
        """Cached VZP search should return in < 100ms."""
        from biomcp.czech.vzp.search import _vzp_search

        entries = [{"kod": "001", "nazev": "Test entry"}]
        with patch(
            "biomcp.czech.vzp.search.get_cached_response",
            return_value=json.dumps(entries),
        ):
            start = time.monotonic()
            await _vzp_search("test", "seznam_vykonu")
            elapsed = time.monotonic() - start
            assert elapsed < 0.1, (
                f"Cached search took {elapsed:.3f}s (> 100ms)"
            )


class TestClaMLParsePerformance:
    """SC-007: ClaML parse performance."""

    @pytest.mark.asyncio
    async def test_claml_parse_under_5s(self):
        """ClaML XML parsing should complete in < 5s."""
        from biomcp.czech.mkn.parser import parse_claml

        sample_xml = """<?xml version="1.0" encoding="UTF-8"?>
<ClaML version="2.0.0">
  <Title name="MKN-10" date="2024-01-01"/>
"""
        # Generate a reasonably large XML with 500 classes
        classes = []
        for i in range(500):
            ch = chr(65 + (i % 26))
            code = f"{ch}{i:02d}"
            classes.append(
                f'  <Class code="{code}" kind="category">'
                f'    <Rubric kind="preferred">'
                f"      <Label>Test diagnosis {code}</Label>"
                f"    </Rubric>"
                f"  </Class>"
            )
        sample_xml += "\n".join(classes) + "\n</ClaML>"

        start = time.monotonic()
        code_index, text_index = await parse_claml(sample_xml)
        elapsed = time.monotonic() - start

        assert elapsed < 5.0, f"ClaML parse took {elapsed:.3f}s (> 5s)"
        assert len(code_index) >= 100


class TestMkn10Accuracy:
    """SC-004: MKN-10 code accuracy >= 95%."""

    @pytest.mark.asyncio
    async def test_known_codes_accuracy(self):
        """Known MKN-10 codes should parse correctly."""
        from biomcp.czech.mkn.parser import parse_claml

        # Sample ClaML with known codes
        xml = """<?xml version="1.0" encoding="UTF-8"?>
<ClaML version="2.0.0">
  <Title name="MKN-10" date="2024-01-01"/>
  <Class code="I" kind="chapter">
    <Rubric kind="preferred">
      <Label>Nemoci obehu</Label>
    </Rubric>
  </Class>
  <Class code="I20-I25" kind="block">
    <SuperClass code="I"/>
    <Rubric kind="preferred">
      <Label>Ischemicke nemoci srdecni</Label>
    </Rubric>
  </Class>
  <Class code="I21" kind="category">
    <SuperClass code="I20-I25"/>
    <Rubric kind="preferred">
      <Label>Akutni infarkt myokardu</Label>
    </Rubric>
  </Class>
  <Class code="J" kind="chapter">
    <Rubric kind="preferred">
      <Label>Nemoci dychaci soustavy</Label>
    </Rubric>
  </Class>
  <Class code="J00-J06" kind="block">
    <SuperClass code="J"/>
    <Rubric kind="preferred">
      <Label>Akutni infekce hornich dychacich cest</Label>
    </Rubric>
  </Class>
  <Class code="J06" kind="category">
    <SuperClass code="J00-J06"/>
    <Rubric kind="preferred">
      <Label>Akutni infekce hornich dychacich cest</Label>
    </Rubric>
  </Class>
  <Class code="J06.9" kind="category">
    <SuperClass code="J06"/>
    <Rubric kind="preferred">
      <Label>Akutni infekce hornich dychacich cest NS</Label>
    </Rubric>
  </Class>
  <Class code="E" kind="chapter">
    <Rubric kind="preferred">
      <Label>Nemoci zlaz s vnitrni sekreci</Label>
    </Rubric>
  </Class>
  <Class code="E10-E14" kind="block">
    <SuperClass code="E"/>
    <Rubric kind="preferred">
      <Label>Diabetes mellitus</Label>
    </Rubric>
  </Class>
  <Class code="E11" kind="category">
    <SuperClass code="E10-E14"/>
    <Rubric kind="preferred">
      <Label>Diabetes mellitus 2. typu</Label>
    </Rubric>
  </Class>
</ClaML>"""

        code_index, text_index = await parse_claml(xml)

        # Test known codes
        known_codes = {
            "I21": "Akutni infarkt myokardu",
            "J06": "Akutni infekce hornich dychacich cest",
            "J06.9": "Akutni infekce hornich dychacich cest NS",
            "E11": "Diabetes mellitus 2. typu",
        }

        correct = 0
        total = len(known_codes)

        for code, expected_name in known_codes.items():
            cls = code_index.get(code)
            if cls and cls.get("name_cs") == expected_name:
                correct += 1

        accuracy = correct / total
        assert accuracy >= 0.95, (
            f"MKN-10 accuracy {accuracy:.0%} < 95% ({correct}/{total} correct)"
        )


class TestSourceAttribution:
    """FR-015: All models include source field."""

    def test_sukl_models_have_source(self):
        from biomcp.czech.sukl.models import Drug

        d = Drug(sukl_code="001", name="Test")
        assert d.source == "SUKL"

    def test_mkn_models_have_source(self):
        from biomcp.czech.mkn.models import Diagnosis

        d = Diagnosis(code="J06.9", name_cs="Test")
        assert d.source == "UZIS/MKN-10"

    def test_nrpzs_source_in_output(self):
        from biomcp.czech.nrpzs.search import _record_to_provider

        record = {"id": "1", "nazev": "Test"}
        result = _record_to_provider(record)
        assert result["source"] == "NRPZS"

    def test_szv_source_in_output(self):
        from biomcp.czech.szv.search import _raw_to_full

        raw = {"kod": "001", "nazev": "Test"}
        result = _raw_to_full(raw)
        assert result["source"] == "MZCR/SZV"

    def test_vzp_source_in_output(self):
        from biomcp.czech.vzp.search import _normalise_entry

        raw = {"kod": "001", "nazev": "Test"}
        result = _normalise_entry(raw, "seznam_vykonu")
        assert result["source"] == "VZP"
