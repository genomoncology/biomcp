"""VZP insurance codebook search implementation.

Fetches and searches VZP codebook data from the VZP public API.
Results are cached with diskcache to avoid repeated network calls.
"""

import json
import logging

import httpx

from biomcp.constants import VZP_BASE_URL
from biomcp.czech.diacritics import normalize_query
from biomcp.http_client import (
    cache_response,
    generate_cache_key,
    get_cached_response,
)

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Internal constants
# ---------------------------------------------------------------------------

_VZP_API_BASE = f"{VZP_BASE_URL}/o-vzp/vzajemne-informace/ciselnik-vykonu"
_VZP_CODEBOOK_CACHE_TTL = 60 * 60 * 24  # 24 hours
_VZP_ENTRY_CACHE_TTL = 60 * 60 * 24 * 7  # 1 week

# Supported codebook type identifiers.  The primary one for health
# procedures is ``seznam_vykonu``.
_KNOWN_CODEBOOK_TYPES = [
    "seznam_vykonu",
    "diagnoza",
    "lekarsky_predpis",
    "atc",
]


# ---------------------------------------------------------------------------
# Low-level fetchers
# ---------------------------------------------------------------------------


async def _fetch_codebook(
    codebook_type: str = "seznam_vykonu",
) -> list[dict]:
    """Fetch all entries for a VZP codebook type.

    Returns a list of raw entry dicts.  Results are cached for 24 h.
    """
    url = f"{_VZP_API_BASE}/{codebook_type}"
    cache_key = generate_cache_key("GET", url, {})

    cached = get_cached_response(cache_key)
    if cached:
        return json.loads(cached)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(
                url,
                headers={"Accept": "application/json"},
            )
            if resp.status_code == 404:
                return []
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError as exc:
        logger.warning(
            "VZP API unavailable for codebook '%s': %s",
            codebook_type,
            exc,
        )
        return []

    entries = (
        data
        if isinstance(data, list)
        else data.get("data", data.get("polozky", []))
    )
    cache_response(
        cache_key,
        json.dumps(entries),
        _VZP_CODEBOOK_CACHE_TTL,
    )
    return entries


async def _fetch_entry(codebook_type: str, code: str) -> dict | None:
    """Fetch a single codebook entry by type and code.

    Tries the direct API endpoint first, then falls back to scanning
    the full codebook list.
    """
    url = f"{_VZP_API_BASE}/{codebook_type}/{code}"
    cache_key = generate_cache_key("GET", url, {})

    cached = get_cached_response(cache_key)
    if cached:
        return json.loads(cached)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(
                url,
                headers={"Accept": "application/json"},
            )
            if resp.status_code == 404:
                return None
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError as exc:
        logger.warning(
            "Failed to fetch VZP entry %s/%s: %s",
            codebook_type,
            code,
            exc,
        )
        return None

    cache_response(
        cache_key,
        json.dumps(data),
        _VZP_ENTRY_CACHE_TTL,
    )
    return data


# ---------------------------------------------------------------------------
# Mapping helpers
# ---------------------------------------------------------------------------


def _normalise_entry(raw: dict, codebook_type: str) -> dict:
    """Normalise a raw API dict into a canonical codebook entry dict."""
    return {
        "codebook_type": codebook_type,
        "code": raw.get("kod", raw.get("code", "")),
        "name": raw.get("nazev", raw.get("name", "")),
        "description": raw.get("popis", raw.get("description")),
        "valid_from": raw.get("platnost_od", raw.get("valid_from")),
        "valid_to": raw.get("platnost_do", raw.get("valid_to")),
        "rules": raw.get("pravidla", raw.get("rules", [])),
        "related_codes": raw.get(
            "souvisejici_kody", raw.get("related_codes", [])
        ),
        "source": "VZP",
    }


def _entry_to_summary(raw: dict, codebook_type: str) -> dict:
    """Return a lightweight summary dict for search results."""
    return {
        "codebook_type": codebook_type,
        "code": raw.get("kod", raw.get("code", "")),
        "name": raw.get("nazev", raw.get("name", "")),
    }


def _matches_query(raw: dict, normalized_q: str, codebook_type: str) -> bool:
    """Return True if the entry matches the normalised query."""
    code = (raw.get("kod") or raw.get("code") or "").lower()
    name = normalize_query(raw.get("nazev") or raw.get("name") or "")
    desc = normalize_query(raw.get("popis") or raw.get("description") or "")
    ctype = normalize_query(codebook_type)
    return (
        normalized_q in code
        or normalized_q in name
        or normalized_q in desc
        or normalized_q in ctype
    )


# ---------------------------------------------------------------------------
# Public search functions
# ---------------------------------------------------------------------------


async def _vzp_search(
    query: str,
    codebook_type: str | None = None,
    max_results: int = 10,
) -> str:
    """Search VZP insurance codebooks.

    Performs a diacritics-insensitive search across one or all
    known codebook types.

    Args:
        query: Search text (code fragment, name, or description).
        codebook_type: Optional codebook type filter.  When *None*,
            all known codebook types are searched.
        max_results: Maximum number of results to return.

    Returns:
        JSON string with keys ``total`` and ``results`` (list of
        codebook entry summary dicts).
    """
    types_to_search = (
        [codebook_type] if codebook_type else _KNOWN_CODEBOOK_TYPES
    )
    normalized_q = normalize_query(query)
    matches: list[dict] = []

    for ctype in types_to_search:
        if len(matches) >= max_results:
            break
        try:
            entries = await _fetch_codebook(ctype)
        except Exception as exc:
            logger.warning("Failed fetching codebook '%s': %s", ctype, exc)
            continue

        for raw in entries:
            if _matches_query(raw, normalized_q, ctype):
                matches.append(_entry_to_summary(raw, ctype))
                if len(matches) >= max_results:
                    break

    return json.dumps(
        {"total": len(matches), "results": matches},
        ensure_ascii=False,
    )


async def _vzp_get(codebook_type: str, code: str) -> str:
    """Get full codebook entry details by type and code.

    Args:
        codebook_type: Codebook type identifier.
        code: Entry code.

    Returns:
        JSON string with full ``CodebookEntry`` fields or an error
        object when the entry is not found.
    """
    # Try the direct endpoint first
    raw = await _fetch_entry(codebook_type, code)

    if raw is None:
        # Fall back to scanning the full codebook
        try:
            entries = await _fetch_codebook(codebook_type)
        except Exception as exc:
            logger.error(
                "Failed to fetch codebook '%s': %s",
                codebook_type,
                exc,
            )
            entries = []

        code_lower = code.lower()
        for item in entries:
            item_code = (item.get("kod") or item.get("code") or "").lower()
            if item_code == code_lower:
                raw = item
                break

    if raw is None:
        return json.dumps(
            {"error": (f"Codebook entry not found: {codebook_type}/{code}")},
            ensure_ascii=False,
        )

    return json.dumps(_normalise_entry(raw, codebook_type), ensure_ascii=False)
