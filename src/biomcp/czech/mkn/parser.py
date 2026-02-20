"""ClaML XML parser for MKN-10 (Czech ICD-10) classification.

Parses the UZIS ClaML XML format and builds two in-memory indices:
- code_index: maps code -> node dict (code, name_cs, kind,
  parent_code, children)
- text_index: maps normalized word -> list of codes

Results are cached using diskcache to avoid re-parsing on each
server start.
"""

import hashlib
import logging

from lxml import etree

from biomcp.czech.diacritics import normalize_query
from biomcp.http_client import (
    cache_response,
    generate_cache_key,
    get_cached_response,
)

logger = logging.getLogger(__name__)

# ClaML XML namespace for xml:lang attribute
_XML_NS = "http://www.w3.org/XML/1998/namespace"
_CACHE_TTL = 60 * 60 * 24 * 30  # 30 days

# Type aliases for the two index types
CodeIndex = dict[str, dict]
TextIndex = dict[str, list[str]]


def _extract_preferred_label(
    class_el: etree._Element,
) -> str:
    """Extract preferred Czech label from a <Class> element.

    Searches for:
      <Rubric kind="preferred"><Label xml:lang="cs">TEXT</Label>

    Falls back to any Label text if no Czech-specific one exists.

    Args:
        class_el: The <Class> lxml element.

    Returns:
        Label text, or empty string if not found.
    """
    for rubric in class_el.iterfind("Rubric"):
        if rubric.get("kind") != "preferred":
            continue
        for label in rubric.iterfind("Label"):
            lang = label.get(f"{{{_XML_NS}}}lang", "")
            if lang == "cs":
                return (label.text or "").strip()
        # Fallback: first label without lang attribute
        first = rubric.find("Label")
        if first is not None:
            return (first.text or "").strip()
    return ""


def _build_text_index(
    code: str,
    name_cs: str,
    text_index: TextIndex,
) -> None:
    """Index all words of a Czech name into the text index.

    Args:
        code: The MKN-10 code to index.
        name_cs: Czech label text.
        text_index: Mutable text index to update in place.
    """
    normalized = normalize_query(name_cs)
    for word in normalized.split():
        if len(word) < 2:  # skip single chars
            continue
        if word not in text_index:
            text_index[word] = []
        if code not in text_index[word]:
            text_index[word].append(code)


def _content_hash(xml_content: str) -> str:
    """Return a short SHA-256 prefix for cache keying."""
    return hashlib.sha256(xml_content.encode("utf-8")).hexdigest()[:16]


async def parse_claml(
    xml_content: str,
) -> tuple[CodeIndex, TextIndex]:
    """Parse ClaML XML and return (code_index, text_index).

    Parses all <Class> elements and builds:
    - code_index: {code: {code, name_cs, kind, parent_code,
      children: [str]}}
    - text_index: {normalized_word: [code1, code2, ...]}

    Results are cached with diskcache using a content hash key.

    Args:
        xml_content: Raw ClaML XML string.

    Returns:
        Tuple of (code_index, text_index).

    Raises:
        etree.XMLSyntaxError: If the XML is malformed.
    """
    import json

    content_hash = _content_hash(xml_content)
    cache_key = generate_cache_key("CLAML", f"mkn10:{content_hash}", {})

    cached = get_cached_response(cache_key)
    if cached:
        payload = json.loads(cached)
        return payload["code_index"], payload["text_index"]

    code_index: CodeIndex = {}
    text_index: TextIndex = {}

    root = etree.fromstring(xml_content.encode("utf-8"))

    for class_el in root.iterfind("Class"):
        code = class_el.get("code", "").strip()
        kind = class_el.get("kind", "").strip()
        if not code:
            continue

        name_cs = _extract_preferred_label(class_el)

        parent_code: str | None = None
        super_el = class_el.find("SuperClass")
        if super_el is not None:
            parent_code = super_el.get("code", "").strip() or None

        children: list[str] = []
        for sub_el in class_el.iterfind("SubClass"):
            child_code = sub_el.get("code", "").strip()
            if child_code:
                children.append(child_code)

        code_index[code] = {
            "code": code,
            "name_cs": name_cs,
            "kind": kind,
            "parent_code": parent_code,
            "children": children,
        }

        if name_cs:
            _build_text_index(code, name_cs, text_index)

    payload = json.dumps(
        {"code_index": code_index, "text_index": text_index},
        ensure_ascii=False,
    )
    cache_response(cache_key, payload, _CACHE_TTL)

    logger.debug(
        "Parsed %d MKN-10 entries, %d text tokens",
        len(code_index),
        len(text_index),
    )
    return code_index, text_index
