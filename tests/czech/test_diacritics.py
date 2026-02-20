"""Tests for diacritics normalization utility."""

from biomcp.czech.diacritics import normalize_query, strip_diacritics


class TestStripDiacritics:
    def test_czech_characters(self):
        assert strip_diacritics("léky") == "leky"
        assert strip_diacritics("Ústí") == "usti"
        assert strip_diacritics("říjen") == "rijen"
        assert strip_diacritics("žluťoučký") == "zlutoucky"

    def test_preserves_ascii(self):
        assert strip_diacritics("ibuprofen") == "ibuprofen"
        assert strip_diacritics("ABC123") == "abc123"

    def test_empty_string(self):
        assert strip_diacritics("") == ""

    def test_mixed_content(self):
        assert strip_diacritics("Léčivý přípravek") == (
            "lecivy pripravek"
        )

    def test_all_czech_diacritics(self):
        assert strip_diacritics("čďěňřšťžú") == "cdenrstzu"
        assert strip_diacritics("ČĎĚŇŘŠŤŽÚ") == "cdenrstzu"

    def test_numbers_and_special(self):
        assert strip_diacritics("J06.9") == "j06.9"
        assert strip_diacritics("M01AE01") == "m01ae01"


class TestNormalizeQuery:
    def test_strips_whitespace(self):
        assert normalize_query("  ibuprofen  ") == "ibuprofen"

    def test_strips_diacritics(self):
        assert normalize_query("léky") == "leky"

    def test_equivalent_results(self):
        assert normalize_query("léky") == normalize_query("leky")
        assert normalize_query("Ústí") == normalize_query("Usti")
        assert normalize_query("říjen") == normalize_query("rijen")
