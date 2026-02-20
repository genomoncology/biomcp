"""SUKL drug availability check.

Uses SUKL DLP API v1 to check distribution status.
"""

import json
import logging
from datetime import datetime, timezone

import httpx

from biomcp.constants import SUKL_API_URL
from biomcp.http_client import (
    cache_response,
    generate_cache_key,
    get_cached_response,
)

logger = logging.getLogger(__name__)

_SUKL_DLP_V1 = f"{SUKL_API_URL.rstrip('/api')}/v1"
_CACHE_TTL = 60 * 60  # 1 hour for availability


async def _fetch_drug_detail(sukl_code: str) -> dict | None:
    """Fetch basic drug detail to verify code exists."""
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

    cache_response(cache_key, json.dumps(data), 60 * 60 * 24 * 7)
    return data


async def _check_distribution(sukl_code: str) -> str:
    """Check if drug is in active distribution.

    Checks VPOIS endpoint for holder info, which indicates
    the drug is actively distributed.

    Returns: 'available', 'limited', or 'unavailable'
    """
    url = f"{_SUKL_DLP_V1}/vpois/{sukl_code}"
    cache_key = generate_cache_key("GET", url, {})

    cached = get_cached_response(cache_key)
    if cached:
        data = json.loads(cached)
        if data.get("_status"):
            return data["_status"]

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            resp = await client.get(url)
            if resp.status_code == 404:
                status = "unavailable"
            elif resp.is_success:
                status = "available"
            else:
                status = "unavailable"
    except httpx.HTTPError:
        status = "unavailable"

    cache_response(
        cache_key,
        json.dumps({"_status": status}),
        _CACHE_TTL,
    )
    return status


async def _sukl_availability_check(sukl_code: str) -> str:
    """Check current drug market availability.

    Args:
        sukl_code: SUKL drug identifier

    Returns:
        JSON with sukl_code, name, status, last_checked, note,
        source
    """
    detail = await _fetch_drug_detail(sukl_code)
    if not detail:
        return json.dumps(
            {"error": f"Drug not found: {sukl_code}"},
            ensure_ascii=False,
        )

    status = await _check_distribution(sukl_code)
    now = datetime.now(timezone.utc).isoformat()

    return json.dumps(
        {
            "sukl_code": detail.get("kodSukl", sukl_code),
            "name": detail.get("nazev", ""),
            "status": status,
            "last_checked": now,
            "note": None,
            "source": "SUKL",
        },
        ensure_ascii=False,
    )
