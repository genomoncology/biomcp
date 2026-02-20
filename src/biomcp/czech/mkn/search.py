"""MKN-10 search, lookup, and hierarchy browsing.

Provides three public async functions:
- _mkn_search: free-text or code-based search
- _mkn_get: full diagnosis detail by code
- _mkn_browse: hierarchy navigation

All functions return a JSON string.
"""

import json
import logging
import re

from biomcp.czech.diacritics import normalize_query
from biomcp.czech.mkn.parser import CodeIndex, TextIndex, parse_claml

logger = logging.getLogger(__name__)

# MKN-10 chapter codes are Roman numerals or single letters;
# leaf categories look like "J06" or "J06.9"
_CODE_RE = re.compile(
    r"^[A-Z0-9]{1,3}(?:\.[0-9]{1,2})?$|^[A-Z]{1,5}-[A-Z0-9]{2,5}$",
    re.IGNORECASE,
)

# Module-level cache for parsed indices (reset per process).
_INDEX_CACHE: tuple[CodeIndex, TextIndex] | None = None
_XML_CACHE: str | None = None


async def _get_index(
    xml_content: str,
) -> tuple[CodeIndex, TextIndex]:
    """Return cached or freshly parsed (code_index, text_index).

    Args:
        xml_content: Raw ClaML XML string.

    Returns:
        Tuple of (code_index, text_index).
    """
    global _INDEX_CACHE, _XML_CACHE
    if _INDEX_CACHE is None or xml_content != _XML_CACHE:
        _INDEX_CACHE = await parse_claml(xml_content)
        _XML_CACHE = xml_content
    return _INDEX_CACHE


def _resolve_hierarchy(
    code: str,
    code_index: CodeIndex,
) -> dict | None:
    """Walk up parent links to resolve chapter/block/category.

    Args:
        code: The MKN-10 code to resolve hierarchy for.
        code_index: Full parsed code index.

    Returns:
        Dict with chapter, chapter_name, block, block_name,
        category keys, or None if hierarchy cannot be resolved.
    """
    node = code_index.get(code)
    if node is None:
        return None

    chapter_code = ""
    chapter_name = ""
    block_code = ""
    block_name = ""
    category_code = ""

    # Traverse up the tree collecting chapter/block/category
    current = node
    chain: list[dict] = [current]
    while current.get("parent_code"):
        parent = code_index.get(current["parent_code"])
        if parent is None:
            break
        chain.append(parent)
        current = parent

    # chain[0] is the node itself, last element is root ancestor
    for ancestor in reversed(chain):
        kind = ancestor.get("kind", "")
        if kind == "chapter":
            chapter_code = ancestor["code"]
            chapter_name = ancestor.get("name_cs", "")
        elif kind == "block":
            block_code = ancestor["code"]
            block_name = ancestor.get("name_cs", "")
        elif kind == "category":
            category_code = ancestor["code"]

    # If the node itself is a category, use it
    if node.get("kind") == "category" and not category_code:
        category_code = node["code"]

    if not chapter_code:
        return None

    return {
        "chapter": chapter_code,
        "chapter_name": chapter_name,
        "block": block_code,
        "block_name": block_name,
        "category": category_code or code,
    }


def _node_to_diagnosis(
    code: str,
    code_index: CodeIndex,
) -> dict | None:
    """Build a Diagnosis-shaped dict from the code index.

    Args:
        code: MKN-10 code.
        code_index: Full parsed code index.

    Returns:
        Dict matching the Diagnosis model, or None if not found.
    """
    node = code_index.get(code)
    if node is None:
        return None

    hierarchy = _resolve_hierarchy(code, code_index)

    return {
        "code": node["code"],
        "name_cs": node.get("name_cs", ""),
        "name_en": None,
        "definition": None,
        "hierarchy": hierarchy,
        "includes": [],
        "excludes": [],
        "modifiers": [],
        "source": "UZIS/MKN-10",
    }


def _search_by_code(
    query: str,
    code_index: CodeIndex,
    max_results: int,
) -> list[dict]:
    """Return nodes whose code starts with the query prefix.

    Args:
        query: Code prefix (case-insensitive).
        code_index: Full parsed code index.
        max_results: Maximum number of results.

    Returns:
        List of matching node dicts.
    """
    upper_q = query.upper()
    results = []
    for code, node in code_index.items():
        if code.upper().startswith(upper_q):
            results.append(node)
            if len(results) >= max_results:
                break
    return results


def _search_by_text(
    query: str,
    code_index: CodeIndex,
    text_index: TextIndex,
    max_results: int,
) -> list[dict]:
    """Full-text search using the inverted text index.

    Each query word must appear in the candidate's label.
    Results are scored by how many query words match.

    Args:
        query: Free-text search query.
        code_index: Full parsed code index.
        text_index: Inverted word->code index.
        max_results: Maximum number of results.

    Returns:
        List of matching node dicts, best-scored first.
    """
    normalized = normalize_query(query)
    words = [w for w in normalized.split() if len(w) >= 2]

    if not words:
        return []

    # Candidate codes: intersection across all query words
    candidate_sets: list[set[str]] = []
    for word in words:
        matching: set[str] = set()
        for indexed_word, codes in text_index.items():
            if word in indexed_word or indexed_word in word:
                matching.update(codes)
        candidate_sets.append(matching)

    if not candidate_sets:
        return []

    # Intersection: all words must match
    candidates = candidate_sets[0]
    for s in candidate_sets[1:]:
        candidates = candidates & s

    results = []
    for code in candidates:
        node = code_index.get(code)
        if node:
            results.append(node)

    results.sort(key=lambda n: n["code"])
    return results[:max_results]


async def _mkn_search(
    query: str,
    max_results: int = 10,
    xml_content: str = "",
) -> str:
    """Search MKN-10 by code prefix or free text.

    If the query looks like a code (e.g., "J06", "J06.9"), it
    performs a prefix match on codes. Otherwise full-text search
    is used with diacritics normalization.

    Args:
        query: Code or free-text search string.
        max_results: Maximum number of results to return.
        xml_content: ClaML XML string (for testing/injection).

    Returns:
        JSON string: {"results": [...], "total": N, "query": str}
    """
    if not xml_content:
        return json.dumps(
            {"error": "No MKN-10 data loaded.", "results": []},
            ensure_ascii=False,
        )

    try:
        code_index, text_index = await _get_index(xml_content)
    except Exception as exc:
        logger.error("Failed to parse MKN-10 XML: %s", exc)
        return json.dumps(
            {"error": f"Parse error: {exc}", "results": []},
            ensure_ascii=False,
        )

    stripped = query.strip()
    if _CODE_RE.match(stripped):
        nodes = _search_by_code(stripped, code_index, max_results)
    else:
        nodes = _search_by_text(stripped, code_index, text_index, max_results)

    results = [
        {
            "code": n["code"],
            "name_cs": n.get("name_cs", ""),
            "kind": n.get("kind", ""),
        }
        for n in nodes
    ]

    return json.dumps(
        {"query": stripped, "total": len(results), "results": results},
        ensure_ascii=False,
    )


async def _mkn_get(
    code: str,
    xml_content: str = "",
) -> str:
    """Get full diagnosis details for a single MKN-10 code.

    Args:
        code: Exact MKN-10 code (e.g., "J06.9").
        xml_content: ClaML XML string (for testing/injection).

    Returns:
        JSON string with Diagnosis fields, or {"error": ...}.
    """
    if not xml_content:
        return json.dumps(
            {"error": "No MKN-10 data loaded."},
            ensure_ascii=False,
        )

    try:
        code_index, _ = await _get_index(xml_content)
    except Exception as exc:
        logger.error("Failed to parse MKN-10 XML: %s", exc)
        return json.dumps(
            {"error": f"Parse error: {exc}"},
            ensure_ascii=False,
        )

    diagnosis = _node_to_diagnosis(code.strip(), code_index)
    if diagnosis is None:
        return json.dumps(
            {"error": f"Code not found: {code}"},
            ensure_ascii=False,
        )

    return json.dumps(diagnosis, ensure_ascii=False)


async def _mkn_browse(
    code: str | None = None,
    xml_content: str = "",
) -> str:
    """Browse the MKN-10 category hierarchy.

    If code is None, returns all root-level chapters.
    If code is provided, returns that node with its immediate
    children expanded.

    Args:
        code: MKN-10 code to browse, or None for root chapters.
        xml_content: ClaML XML string (for testing/injection).

    Returns:
        JSON string with hierarchy node(s).
    """
    if not xml_content:
        return json.dumps(
            {"error": "No MKN-10 data loaded."},
            ensure_ascii=False,
        )

    try:
        code_index, _ = await _get_index(xml_content)
    except Exception as exc:
        logger.error("Failed to parse MKN-10 XML: %s", exc)
        return json.dumps(
            {"error": f"Parse error: {exc}"},
            ensure_ascii=False,
        )

    if code is None:
        chapters = [
            {
                "code": node["code"],
                "name_cs": node.get("name_cs", ""),
                "kind": node.get("kind", ""),
                "children": node.get("children", []),
            }
            for node in code_index.values()
            if node.get("kind") == "chapter"
        ]
        chapters.sort(key=lambda n: n["code"])
        return json.dumps(
            {"type": "chapters", "items": chapters},
            ensure_ascii=False,
        )

    node = code_index.get(code.strip())
    if node is None:
        return json.dumps(
            {"error": f"Code not found: {code}"},
            ensure_ascii=False,
        )

    child_nodes = []
    for child_code in node.get("children", []):
        child = code_index.get(child_code)
        if child:
            child_nodes.append({
                "code": child["code"],
                "name_cs": child.get("name_cs", ""),
                "kind": child.get("kind", ""),
                "children": child.get("children", []),
            })

    result = {
        "code": node["code"],
        "name_cs": node.get("name_cs", ""),
        "kind": node.get("kind", ""),
        "parent_code": node.get("parent_code"),
        "children": child_nodes,
    }
    return json.dumps(result, ensure_ascii=False)
