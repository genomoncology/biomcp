"""NRPZS healthcare provider search and detail retrieval.

Uses the NRPZS (Národní registr poskytovatelů zdravotních služeb)
API at nrpzs.uzis.cz with diskcache for response caching.
"""

import json
import logging

import httpx

from biomcp.constants import NRPZS_PROVIDERS_URL
from biomcp.czech.diacritics import normalize_query
from biomcp.http_client import (
    cache_response,
    generate_cache_key,
    get_cached_response,
)

logger = logging.getLogger(__name__)

_SEARCH_CACHE_TTL = 60 * 60 * 24  # 24 hours
_DETAIL_CACHE_TTL = 60 * 60 * 24 * 7  # 1 week


def _build_search_params(
    query: str | None,
    city: str | None,
    specialty: str | None,
    page: int,
    page_size: int,
) -> dict:
    """Build query parameters for the NRPZS search endpoint."""
    params: dict = {
        "strana": page,
        "velikostStranky": page_size,
    }
    if query:
        params["nazev"] = normalize_query(query)
    if city:
        params["obec"] = normalize_query(city)
    if specialty:
        params["odbornost"] = normalize_query(specialty)
    return params


def _record_to_summary(record: dict) -> dict:
    """Convert an API record to a ProviderSummary dict."""
    odbornosti = record.get("odbornosti") or []
    if isinstance(odbornosti, str):
        odbornosti = [odbornosti] if odbornosti else []

    return {
        "provider_id": str(record.get("id", "")),
        "name": record.get("nazev", ""),
        "city": record.get("obec"),
        "specialties": odbornosti,
    }


def _record_to_provider(record: dict) -> dict:
    """Convert a detailed API record to a HealthcareProvider dict."""
    odbornosti = record.get("odbornosti") or []
    if isinstance(odbornosti, str):
        odbornosti = [odbornosti] if odbornosti else []

    druhy_pece = record.get("druhyPece") or []
    if isinstance(druhy_pece, str):
        druhy_pece = [druhy_pece] if druhy_pece else []

    address: dict | None = None
    if any(record.get(k) for k in ("ulice", "obec", "psc", "kraj")):
        address = {
            "street": record.get("ulice"),
            "city": record.get("obec"),
            "postal_code": record.get("psc"),
            "region": record.get("kraj"),
        }

    workplaces_raw = record.get("pracoviste") or []
    workplaces = [_raw_workplace_to_dict(wp) for wp in workplaces_raw]

    return {
        "provider_id": str(record.get("id", "")),
        "name": record.get("nazev", ""),
        "legal_form": record.get("pravniForma"),
        "ico": record.get("ico"),
        "address": address,
        "specialties": odbornosti,
        "care_types": druhy_pece,
        "workplaces": workplaces,
        "registration_number": record.get("registracniCislo"),
        "source": "NRPZS",
    }


def _raw_workplace_to_dict(wp: dict) -> dict:
    """Convert a raw workplace dict from the API."""
    specialties = wp.get("odbornosti") or []
    if isinstance(specialties, str):
        specialties = [specialties] if specialties else []

    address: dict | None = None
    if any(wp.get(k) for k in ("ulice", "obec", "psc", "kraj")):
        address = {
            "street": wp.get("ulice"),
            "city": wp.get("obec"),
            "postal_code": wp.get("psc"),
            "region": wp.get("kraj"),
        }

    contact: dict | None = None
    if any(wp.get(k) for k in ("telefon", "email", "www")):
        contact = {
            "phone": wp.get("telefon"),
            "email": wp.get("email"),
            "website": wp.get("www"),
        }

    return {
        "workplace_id": str(wp.get("id", "")),
        "name": wp.get("nazev", ""),
        "address": address,
        "specialties": specialties,
        "contact": contact,
    }


async def _nrpzs_search(
    query: str | None = None,
    city: str | None = None,
    specialty: str | None = None,
    page: int = 1,
    page_size: int = 10,
) -> str:
    """Search Czech healthcare providers in the NRPZS registry.

    Args:
        query: Provider or facility name (diacritics-insensitive).
        city: Filter by municipality name.
        specialty: Filter by medical specialty.
        page: Page number (1-based).
        page_size: Results per page (1-100).

    Returns:
        JSON string with ProviderSearchResult fields.
    """
    params = _build_search_params(query, city, specialty, page, page_size)
    cache_key = generate_cache_key("GET", NRPZS_PROVIDERS_URL, params)

    cached = get_cached_response(cache_key)
    if cached:
        return cached

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(NRPZS_PROVIDERS_URL, params=params)
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError as exc:
        logger.error("NRPZS search request failed: %s", exc)
        return json.dumps(
            {
                "total": 0,
                "page": page,
                "page_size": page_size,
                "results": [],
                "error": f"NRPZS API unavailable: {exc}",
            },
            ensure_ascii=False,
        )

    records = data.get("zaznamy") or []
    pagination = data.get("strankovani") or {}

    result = {
        "total": data.get("celkem", 0),
        "page": pagination.get("stranka", page),
        "page_size": pagination.get("velikostStranky", page_size),
        "results": [_record_to_summary(r) for r in records],
    }

    serialized = json.dumps(result, ensure_ascii=False)
    cache_response(cache_key, serialized, _SEARCH_CACHE_TTL)
    return serialized


async def _nrpzs_get(provider_id: str) -> str:
    """Get full healthcare provider details by NRPZS provider ID.

    Args:
        provider_id: The NRPZS provider identifier.

    Returns:
        JSON string with HealthcareProvider fields, or an error dict.
    """
    url = f"{NRPZS_PROVIDERS_URL}/{provider_id}"
    cache_key = generate_cache_key("GET", url, {})

    cached = get_cached_response(cache_key)
    if cached:
        return cached

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(url)
            if resp.status_code == 404:
                return json.dumps(
                    {"error": (f"Provider not found: {provider_id}")},
                    ensure_ascii=False,
                )
            resp.raise_for_status()
            data = resp.json()
    except httpx.HTTPError as exc:
        logger.error(
            "NRPZS detail request failed for %s: %s",
            provider_id,
            exc,
        )
        return json.dumps(
            {"error": f"NRPZS API unavailable: {exc}"},
            ensure_ascii=False,
        )

    result = _record_to_provider(data)
    serialized = json.dumps(result, ensure_ascii=False)
    cache_response(cache_key, serialized, _DETAIL_CACHE_TTL)
    return serialized
