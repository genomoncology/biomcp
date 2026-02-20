"""Unit tests for the MKN-10 ClaML XML parser."""

import json
from unittest.mock import patch

import pytest

from tests.czech.conftest import SAMPLE_CLAML_XML


class TestParseClaml:
    """Tests for parse_claml() function."""

    @pytest.fixture(autouse=True)
    def clear_module_cache(self):
        """Reset in-process index cache before each test."""
        import biomcp.czech.mkn.search as search_mod

        search_mod._INDEX_CACHE = None
        search_mod._XML_CACHE = None
        yield
        search_mod._INDEX_CACHE = None
        search_mod._XML_CACHE = None

    @pytest.fixture
    def no_diskcache(self):
        """Bypass diskcache so tests are hermetic."""
        with patch(
            "biomcp.czech.mkn.parser.get_cached_response",
            return_value=None,
        ), patch(
            "biomcp.czech.mkn.parser.cache_response",
            return_value=None,
        ):
            yield

    @pytest.mark.asyncio
    async def test_parse_returns_two_indices(self, no_diskcache):
        """parse_claml returns (code_index, text_index)."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, text_index = await parse_claml(SAMPLE_CLAML_XML)
        assert isinstance(code_index, dict)
        assert isinstance(text_index, dict)

    @pytest.mark.asyncio
    async def test_chapter_parsed(self, no_diskcache):
        """Chapter class X is parsed with kind='chapter'."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert "X" in code_index
        node = code_index["X"]
        assert node["kind"] == "chapter"
        assert "dýchací" in node["name_cs"] or "dychaci" in node[
            "name_cs"
        ].lower()

    @pytest.mark.asyncio
    async def test_block_parsed(self, no_diskcache):
        """Block class J00-J06 is parsed with kind='block'."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert "J00-J06" in code_index
        node = code_index["J00-J06"]
        assert node["kind"] == "block"
        assert node["parent_code"] == "X"

    @pytest.mark.asyncio
    async def test_category_parsed(self, no_diskcache):
        """Category J06 is parsed with kind='category'."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert "J06" in code_index
        node = code_index["J06"]
        assert node["kind"] == "category"
        assert node["parent_code"] == "J00-J06"

    @pytest.mark.asyncio
    async def test_subcategory_parsed(self, no_diskcache):
        """Subcategory J06.9 is parsed with correct parent."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert "J06.9" in code_index
        node = code_index["J06.9"]
        assert node["kind"] == "category"
        assert node["parent_code"] == "J06"

    @pytest.mark.asyncio
    async def test_label_extraction_czech(self, no_diskcache):
        """Preferred Czech label is extracted for each class."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert "Akutní" in code_index["J06"]["name_cs"]
        assert "NS" in code_index["J06.9"]["name_cs"]

    @pytest.mark.asyncio
    async def test_hierarchy_subclass_links(self, no_diskcache):
        """Children are recorded as SubClass references."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert "J00-J06" in code_index["X"]["children"]
        assert "J06" in code_index["J00-J06"]["children"]
        assert "J06.9" in code_index["J06"]["children"]

    @pytest.mark.asyncio
    async def test_leaf_node_has_no_children(self, no_diskcache):
        """J06.9 is a leaf and has no SubClass children."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert code_index["J06.9"]["children"] == []

    @pytest.mark.asyncio
    async def test_text_index_contains_words(self, no_diskcache):
        """Text index is built from Czech labels."""
        from biomcp.czech.mkn.parser import parse_claml

        _, text_index = await parse_claml(SAMPLE_CLAML_XML)
        # "akutni" is the normalized form of "Akutní"
        assert any("akutni" in w or "akut" in w for w in text_index)

    @pytest.mark.asyncio
    async def test_text_index_maps_to_codes(self, no_diskcache):
        """Text index maps normalized words to code lists."""
        from biomcp.czech.mkn.parser import parse_claml

        _, text_index = await parse_claml(SAMPLE_CLAML_XML)
        # "infekce" should map to at least one category code
        matching = [
            codes
            for word, codes in text_index.items()
            if "infekce" in word
        ]
        assert matching, "Expected 'infekce' to be indexed"
        all_codes = {c for codes in matching for c in codes}
        assert "J06" in all_codes or "J00-J06" in all_codes

    @pytest.mark.asyncio
    async def test_diskcache_used_on_hit(self):
        """parse_claml returns cached result without re-parsing."""
        from biomcp.czech.mkn.parser import parse_claml

        cached_payload = json.dumps(
            {
                "code_index": {"X": {"code": "X", "name_cs": "cached"}},
                "text_index": {},
            }
        )
        with patch(
            "biomcp.czech.mkn.parser.get_cached_response",
            return_value=cached_payload,
        ):
            code_index, text_index = await parse_claml(
                SAMPLE_CLAML_XML
            )
        assert code_index == {"X": {"code": "X", "name_cs": "cached"}}
        assert text_index == {}

    @pytest.mark.asyncio
    async def test_all_four_classes_present(self, no_diskcache):
        """All four <Class> elements from sample XML are parsed."""
        from biomcp.czech.mkn.parser import parse_claml

        code_index, _ = await parse_claml(SAMPLE_CLAML_XML)
        assert len(code_index) == 4
        for code in ("X", "J00-J06", "J06", "J06.9"):
            assert code in code_index

    @pytest.mark.asyncio
    async def test_malformed_xml_raises(self, no_diskcache):
        """Malformed XML raises an error."""
        from lxml import etree

        from biomcp.czech.mkn.parser import parse_claml

        with pytest.raises(etree.XMLSyntaxError):
            await parse_claml("<broken>")
