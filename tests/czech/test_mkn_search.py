"""Unit tests for MKN-10 search, getter, and browse functions."""

import json
from unittest.mock import patch

import pytest

from tests.czech.conftest import SAMPLE_CLAML_XML


@pytest.fixture(autouse=True)
def clear_module_cache():
    """Reset in-process index cache before each test."""
    import biomcp.czech.mkn.search as search_mod

    search_mod._INDEX_CACHE = None
    search_mod._XML_CACHE = None
    yield
    search_mod._INDEX_CACHE = None
    search_mod._XML_CACHE = None


@pytest.fixture(autouse=True)
def no_diskcache():
    """Bypass diskcache for all tests in this module."""
    with patch(
        "biomcp.czech.mkn.parser.get_cached_response",
        return_value=None,
    ), patch(
        "biomcp.czech.mkn.parser.cache_response",
        return_value=None,
    ):
        yield


# ---------------------------------------------------------------------------
# _mkn_search
# ---------------------------------------------------------------------------


class TestMknSearch:
    """Tests for _mkn_search()."""

    @pytest.mark.asyncio
    async def test_search_by_code_exact(self):
        """Searching 'J06.9' returns exact code match."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(
            await _mkn_search("J06.9", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["total"] >= 1
        codes = [r["code"] for r in result["results"]]
        assert "J06.9" in codes

    @pytest.mark.asyncio
    async def test_search_by_code_prefix(self):
        """Searching 'J06' returns both J06 and J06.9."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(
            await _mkn_search("J06", xml_content=SAMPLE_CLAML_XML)
        )
        codes = [r["code"] for r in result["results"]]
        assert "J06" in codes
        assert "J06.9" in codes

    @pytest.mark.asyncio
    async def test_search_by_text_czech(self):
        """Searching Czech text 'infekce' returns matching entries."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(
            await _mkn_search("infekce", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["total"] >= 1
        kinds = [r.get("kind") for r in result["results"]]
        assert any(k in ("block", "category") for k in kinds)

    @pytest.mark.asyncio
    async def test_search_no_results(self):
        """Searching unknown term returns empty result set."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(
            await _mkn_search(
                "infarkt", xml_content=SAMPLE_CLAML_XML
            )
        )
        assert result["total"] == 0
        assert result["results"] == []

    @pytest.mark.asyncio
    async def test_search_diacritics_with_diacritics(self):
        """Searching 'dýchací' finds entries matching 'dychaci'."""
        from biomcp.czech.mkn.search import _mkn_search

        result_diacritics = json.loads(
            await _mkn_search(
                "dýchací", xml_content=SAMPLE_CLAML_XML
            )
        )
        assert result_diacritics["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_diacritics_without_diacritics(self):
        """Searching 'dychaci' finds same entries as 'dýchací'."""
        from biomcp.czech.mkn.search import _mkn_search

        result_ascii = json.loads(
            await _mkn_search("dychaci", xml_content=SAMPLE_CLAML_XML)
        )
        assert result_ascii["total"] >= 1

    @pytest.mark.asyncio
    async def test_search_diacritics_same_results(self):
        """'dýchací' and 'dychaci' return the same codes."""
        from biomcp.czech.mkn.search import _mkn_search

        r1 = json.loads(
            await _mkn_search(
                "dýchací", xml_content=SAMPLE_CLAML_XML
            )
        )
        r2 = json.loads(
            await _mkn_search("dychaci", xml_content=SAMPLE_CLAML_XML)
        )
        codes1 = {r["code"] for r in r1["results"]}
        codes2 = {r["code"] for r in r2["results"]}
        assert codes1 == codes2

    @pytest.mark.asyncio
    async def test_search_max_results_respected(self):
        """max_results parameter limits result count."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(
            await _mkn_search(
                "J",
                max_results=1,
                xml_content=SAMPLE_CLAML_XML,
            )
        )
        assert len(result["results"]) <= 1

    @pytest.mark.asyncio
    async def test_search_no_xml_returns_error(self):
        """Calling _mkn_search without XML returns error JSON."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(await _mkn_search("J06"))
        assert "error" in result

    @pytest.mark.asyncio
    async def test_search_query_preserved_in_response(self):
        """Response includes the original query string."""
        from biomcp.czech.mkn.search import _mkn_search

        result = json.loads(
            await _mkn_search("J06.9", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["query"] == "J06.9"


# ---------------------------------------------------------------------------
# _mkn_get
# ---------------------------------------------------------------------------


class TestMknGet:
    """Tests for _mkn_get()."""

    @pytest.mark.asyncio
    async def test_get_leaf_code(self):
        """Getting J06.9 returns full Diagnosis dict."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(
            await _mkn_get("J06.9", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "J06.9"
        assert result["source"] == "UZIS/MKN-10"
        assert result["name_cs"]

    @pytest.mark.asyncio
    async def test_get_hierarchy_present(self):
        """Getting J06.9 includes hierarchy with chapter/block."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(
            await _mkn_get("J06.9", xml_content=SAMPLE_CLAML_XML)
        )
        hierarchy = result.get("hierarchy")
        assert hierarchy is not None
        assert hierarchy["chapter"] == "X"
        assert hierarchy["block"] == "J00-J06"

    @pytest.mark.asyncio
    async def test_get_invalid_code_returns_error(self):
        """Getting a nonexistent code returns {'error': ...}."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(
            await _mkn_get("Z99.9", xml_content=SAMPLE_CLAML_XML)
        )
        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_no_xml_returns_error(self):
        """Calling _mkn_get without XML returns error."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(await _mkn_get("J06.9"))
        assert "error" in result

    @pytest.mark.asyncio
    async def test_get_chapter_code(self):
        """Getting chapter code X returns valid result."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(
            await _mkn_get("X", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "X"
        assert "dýchací" in result["name_cs"] or result["name_cs"]

    @pytest.mark.asyncio
    async def test_get_block_code(self):
        """Getting block code J00-J06 returns valid result."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(
            await _mkn_get("J00-J06", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "J00-J06"
        assert result["source"] == "UZIS/MKN-10"

    @pytest.mark.asyncio
    async def test_get_category_code(self):
        """Getting category J06 returns includes/excludes lists."""
        from biomcp.czech.mkn.search import _mkn_get

        result = json.loads(
            await _mkn_get("J06", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "J06"
        assert isinstance(result["includes"], list)
        assert isinstance(result["excludes"], list)


# ---------------------------------------------------------------------------
# _mkn_browse
# ---------------------------------------------------------------------------


class TestMknBrowse:
    """Tests for _mkn_browse()."""

    @pytest.mark.asyncio
    async def test_browse_root_returns_chapters(self):
        """Browsing without a code returns chapter list."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse(xml_content=SAMPLE_CLAML_XML)
        )
        assert result["type"] == "chapters"
        assert isinstance(result["items"], list)
        chapter_codes = [c["code"] for c in result["items"]]
        assert "X" in chapter_codes

    @pytest.mark.asyncio
    async def test_browse_chapter_returns_children(self):
        """Browsing chapter X returns block J00-J06 as child."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse("X", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "X"
        child_codes = [c["code"] for c in result["children"]]
        assert "J00-J06" in child_codes

    @pytest.mark.asyncio
    async def test_browse_block_returns_children(self):
        """Browsing block J00-J06 returns category J06 as child."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse("J00-J06", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "J00-J06"
        child_codes = [c["code"] for c in result["children"]]
        assert "J06" in child_codes

    @pytest.mark.asyncio
    async def test_browse_leaf_has_empty_children(self):
        """Browsing leaf J06.9 returns empty children list."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse("J06.9", xml_content=SAMPLE_CLAML_XML)
        )
        assert result["code"] == "J06.9"
        assert result["children"] == []

    @pytest.mark.asyncio
    async def test_browse_invalid_code_returns_error(self):
        """Browsing unknown code returns error."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse("Z99", xml_content=SAMPLE_CLAML_XML)
        )
        assert "error" in result

    @pytest.mark.asyncio
    async def test_browse_no_xml_returns_error(self):
        """Calling _mkn_browse without XML returns error."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(await _mkn_browse())
        assert "error" in result

    @pytest.mark.asyncio
    async def test_browse_node_includes_kind(self):
        """Browse result includes the 'kind' field."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse("X", xml_content=SAMPLE_CLAML_XML)
        )
        assert result.get("kind") == "chapter"

    @pytest.mark.asyncio
    async def test_browse_parent_code_present(self):
        """Browse of J06 includes its parent_code J00-J06."""
        from biomcp.czech.mkn.search import _mkn_browse

        result = json.loads(
            await _mkn_browse("J06", xml_content=SAMPLE_CLAML_XML)
        )
        assert result.get("parent_code") == "J00-J06"
