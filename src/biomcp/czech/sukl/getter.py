"""SUKL drug detail, SmPC, and PIL retrieval.

Uses SUKL DLP API v1 for drug details, composition, and
document metadata.
"""

import json
import logging

import httpx

from biomcp.constants import SUKL_API_URL
from biomcp.http_client import (
    cache_response,
    generate_cache_key,
    get_cached_response,
)

logger = logging.getLogger(__name__)

_SUKL_DLP_V1 = f"{SUKL_API_URL.rstrip('/api')}/v1"
_CACHE_TTL = 60 * 60 * 24 * 7  # 1 week


async def _fetch_drug_detail(sukl_code: str) -> dict | None:
    """Fetch drug detail from DLP API."""
    url = f"{_SUKL_DLP_V1}/lecive-pripravky/{sukl_code}"
    cache_key = generate_cache_key("GET", url, {})

    cached = get_cached_response(cache_key)
    if cached:
        return json.loads(cached)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(url)
            if resp.status_code == 404:
                return None
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError:
        logger.warning("Failed to fetch drug detail for %s", sukl_code)
        return None

    cache_response(cache_key, json.dumps(data), _CACHE_TTL)
    return data


async def _fetch_composition(sukl_code: str) -> list[dict]:
    """Fetch drug composition (active substances)."""
    url = f"{_SUKL_DLP_V1}/slozeni/{sukl_code}"
    cache_key = generate_cache_key("GET", url, {})

    cached = get_cached_response(cache_key)
    if cached:
        return json.loads(cached)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(url)
            if resp.status_code == 404:
                return []
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError:
        logger.warning("Failed to fetch composition for %s", sukl_code)
        return []

    cache_response(cache_key, json.dumps(data), _CACHE_TTL)
    return data if isinstance(data, list) else []


async def _fetch_doc_metadata(
    sukl_code: str, typ: str | None = None
) -> list[dict]:
    """Fetch document metadata for a drug."""
    url = f"{_SUKL_DLP_V1}/dokumenty-metadata/{sukl_code}"
    params = {"typ": typ} if typ else {}
    cache_key = generate_cache_key("GET", url, params)

    cached = get_cached_response(cache_key)
    if cached:
        return json.loads(cached)

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(url, params=params)
            if resp.status_code == 404:
                return []
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError:
        logger.warning("Failed to fetch doc metadata for %s", sukl_code)
        return []

    result = data if isinstance(data, list) else []
    cache_response(cache_key, json.dumps(result), _CACHE_TTL)
    return result


def _build_doc_url(sukl_code: str, doc_type: str) -> str:
    """Build document download URL."""
    return f"{_SUKL_DLP_V1}/dokumenty/{sukl_code}/{doc_type}"


def _composition_to_substances(
    composition: list[dict],
) -> list[dict]:
    """Convert API composition to active substances list."""
    substances = []
    for item in composition:
        name = item.get("nazevLatky", "")
        amount = item.get("mnozstvi", "")
        unit = item.get("jednotka", "")
        strength = f"{amount} {unit}".strip() if amount else None
        substances.append({"name": name, "strength": strength})
    return substances


async def _sukl_drug_details(sukl_code: str) -> str:
    """Get full drug details by SUKL code.

    Returns JSON with full Drug entity fields.
    """
    detail = await _fetch_drug_detail(sukl_code)
    if not detail:
        return json.dumps(
            {"error": f"Drug not found: {sukl_code}"},
            ensure_ascii=False,
        )

    composition = await _fetch_composition(sukl_code)
    doc_meta = await _fetch_doc_metadata(sukl_code)

    spc_docs = [d for d in doc_meta if d.get("typ") == "spc"]
    pil_docs = [d for d in doc_meta if d.get("typ") == "pil"]

    spc_url = _build_doc_url(sukl_code, "spc") if spc_docs else None
    pil_url = _build_doc_url(sukl_code, "pil") if pil_docs else None

    result = {
        "sukl_code": detail.get("kodSukl", sukl_code),
        "name": detail.get("nazev", ""),
        "active_substances": _composition_to_substances(composition),
        "pharmaceutical_form": detail.get("nazevFormy"),
        "atc_code": detail.get("kodAtc"),
        "registration_number": detail.get("registracniCislo"),
        "mah": detail.get("nazevDrzitele"),
        "registration_valid_to": detail.get("platnostRegistrace"),
        "spc_url": spc_url,
        "pil_url": pil_url,
        "source": "SUKL",
    }

    return json.dumps(result, ensure_ascii=False)


async def _sukl_spc_getter(sukl_code: str) -> str:
    """Get Summary of Product Characteristics (SmPC).

    Returns JSON with sukl_code, name, spc_text, spc_url, source.
    """
    detail = await _fetch_drug_detail(sukl_code)
    if not detail:
        return json.dumps(
            {"error": f"Drug not found: {sukl_code}"},
            ensure_ascii=False,
        )

    doc_meta = await _fetch_doc_metadata(sukl_code, typ="spc")
    if not doc_meta:
        return json.dumps(
            {
                "error": f"SmPC not available for {sukl_code}",
                "sukl_code": sukl_code,
                "name": detail.get("nazev", ""),
                "spc_text": None,
                "spc_url": None,
                "source": "SUKL",
            },
            ensure_ascii=False,
        )

    spc_url = _build_doc_url(sukl_code, "spc")

    return json.dumps(
        {
            "sukl_code": sukl_code,
            "name": detail.get("nazev", ""),
            "spc_text": f"SmPC document available at: {spc_url}",
            "spc_url": spc_url,
            "source": "SUKL",
        },
        ensure_ascii=False,
    )


async def _sukl_pil_getter(sukl_code: str) -> str:
    """Get Patient Information Leaflet (PIL).

    Returns JSON with sukl_code, name, pil_text, pil_url, source.
    """
    detail = await _fetch_drug_detail(sukl_code)
    if not detail:
        return json.dumps(
            {"error": f"Drug not found: {sukl_code}"},
            ensure_ascii=False,
        )

    doc_meta = await _fetch_doc_metadata(sukl_code, typ="pil")
    if not doc_meta:
        return json.dumps(
            {
                "error": f"PIL not available for {sukl_code}",
                "sukl_code": sukl_code,
                "name": detail.get("nazev", ""),
                "pil_text": None,
                "pil_url": None,
                "source": "SUKL",
            },
            ensure_ascii=False,
        )

    pil_url = _build_doc_url(sukl_code, "pil")

    return json.dumps(
        {
            "sukl_code": sukl_code,
            "name": detail.get("nazev", ""),
            "pil_text": f"PIL document available at: {pil_url}",
            "pil_url": pil_url,
            "source": "SUKL",
        },
        ensure_ascii=False,
    )
