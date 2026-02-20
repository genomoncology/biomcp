"""SZV health procedures search implementation.

Uses NZIP Open Data API v3 (nzip.cz) as primary source and
szv.mzcr.cz as supplementary source.  Results are cached with
diskcache to reduce repeated network calls.
"""

import json
import logging

import httpx

from biomcp.constants import NZIP_BASE_URL, SZV_BASE_URL
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

_NZIP_API_V3 = f"{NZIP_BASE_URL}/api/v3"
_PROCEDURE_LIST_CACHE_TTL = 60 * 60 * 24  # 24 hours
_PROCEDURE_DETAIL_CACHE_TTL = 60 * 60 * 24 * 7  # 1 week

# NZIP endpoint that returns a list of health procedures
_NZIP_PROCEDURES_URL = f"{_NZIP_API_V3}/vykony"

# Supplementary SZV endpoint
_SZV_PROCEDURES_URL = f"{SZV_BASE_URL}/szv/vykony"


# ---------------------------------------------------------------------------
# Low-level fetchers
# ---------------------------------------------------------------------------


async def _fetch_procedure_list() -> list[dict]:
    """Fetch the full procedure list from NZIP Open Data API.

    Returns a list of raw procedure dicts from the API.  The result
    is cached for 24 hours.
    """
    cache_key = generate_cache_key("GET", _NZIP_PROCEDURES_URL, {})
    cached = get_cached_response(cache_key)
    if cached:
        return json.loads(cached)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(
                _NZIP_PROCEDURES_URL,
                headers={"Accept": "application/json"},
            )
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError as exc:
        logger.warning("NZIP API unavailable: %s", exc)
        # Fall back to empty list; callers handle this gracefully
        return []

    procedures = (
        data
        if isinstance(data, list)
        else data.get("data", data.get("vykony", []))
    )
    cache_response(
        cache_key,
        json.dumps(procedures),
        _PROCEDURE_LIST_CACHE_TTL,
    )
    return procedures


async def _fetch_procedure_detail(code: str) -> dict | None:
    """Fetch a single procedure from NZIP by procedure code.

    Returns the raw API dict or *None* when the code is unknown.
    """
    url = f"{_NZIP_PROCEDURES_URL}/{code}"
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
            "Failed to fetch procedure detail for %s: %s", code, exc
        )
        return None

    cache_response(
        cache_key,
        json.dumps(data),
        _PROCEDURE_DETAIL_CACHE_TTL,
    )
    return data


# ---------------------------------------------------------------------------
# Mapping helpers
# ---------------------------------------------------------------------------


def _raw_to_summary(raw: dict) -> dict:
    """Convert a raw API procedure dict to a lightweight summary."""
    return {
        "code": raw.get("kod", raw.get("code", "")),
        "name": raw.get("nazev", raw.get("name", "")),
        "point_value": raw.get("body", raw.get("point_value")),
        "category": raw.get("skupina", raw.get("category")),
    }


def _raw_to_full(raw: dict) -> dict:
    """Convert a raw API procedure dict to a full HealthProcedure dict."""
    code = raw.get("kod", raw.get("code", ""))
    category = raw.get("skupina", raw.get("category"))
    return {
        "code": code,
        "name": raw.get("nazev", raw.get("name", "")),
        "category": category,
        "category_name": raw.get("skupina_nazev", raw.get("category_name")),
        "point_value": raw.get("body", raw.get("point_value")),
        "time_minutes": raw.get("cas", raw.get("time_minutes")),
        "frequency_limit": raw.get(
            "omezeni_frekvence", raw.get("frequency_limit")
        ),
        "specialty_codes": raw.get(
            "odbornosti", raw.get("specialty_codes", [])
        ),
        "material_requirements": raw.get(
            "materialni_pozadavky", raw.get("material_requirements")
        ),
        "notes": raw.get("poznamky", raw.get("notes")),
        "source": "MZCR/SZV",
    }


def _matches_query(raw: dict, normalized_q: str) -> bool:
    """Return True if the raw procedure dict matches the query."""
    code = (raw.get("kod") or raw.get("code") or "").lower()
    name = normalize_query(raw.get("nazev") or raw.get("name") or "")
    category = normalize_query(raw.get("skupina") or raw.get("category") or "")
    return (
        normalized_q in code
        or normalized_q in name
        or normalized_q in category
    )


# ---------------------------------------------------------------------------
# Public search functions
# ---------------------------------------------------------------------------


async def _szv_search(
    query: str,
    max_results: int = 10,
) -> str:
    """Search health procedures by code or name.

    Performs a diacritics-insensitive search across procedure codes
    and names from the NZIP Open Data API.

    Args:
        query: Procedure code (e.g. ``"09513"``) or name fragment.
        max_results: Maximum number of results to return.

    Returns:
        JSON string with keys ``total`` and ``results`` (list of
        procedure summary dicts).
    """
    try:
        procedures = await _fetch_procedure_list()
    except Exception as exc:
        logger.error("Failed to fetch procedure list: %s", exc)
        return json.dumps(
            {
                "total": 0,
                "results": [],
                "error": f"SZV/NZIP API unavailable: {exc}",
            },
            ensure_ascii=False,
        )

    normalized_q = normalize_query(query)
    matches: list[dict] = []

    for raw in procedures:
        if _matches_query(raw, normalized_q):
            matches.append(_raw_to_summary(raw))
            if len(matches) >= max_results:
                break

    return json.dumps(
        {"total": len(matches), "results": matches},
        ensure_ascii=False,
    )


async def _szv_get(code: str) -> str:
    """Get full procedure details by procedure code.

    Args:
        code: Procedure code (e.g. ``"09513"``).

    Returns:
        JSON string with full ``HealthProcedure`` fields or an error
        object when the code is not found.
    """
    # First try the direct detail endpoint
    raw = await _fetch_procedure_detail(code)

    if raw is None:
        # Fall back: scan the list for an exact code match
        try:
            procedures = await _fetch_procedure_list()
        except Exception as exc:
            logger.error("Failed to fetch procedure list: %s", exc)
            procedures = []

        code_lower = code.lower()
        for item in procedures:
            item_code = (item.get("kod") or item.get("code") or "").lower()
            if item_code == code_lower:
                raw = item
                break

    if raw is None:
        return json.dumps(
            {"error": f"Procedure not found: {code}"},
            ensure_ascii=False,
        )

    return json.dumps(_raw_to_full(raw), ensure_ascii=False)
