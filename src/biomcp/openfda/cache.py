"""
Simple in-memory caching for OpenFDA API responses.

This module provides a time-based cache to reduce API calls and improve performance.
Cache entries expire after a configurable TTL (time-to-live).
"""

import hashlib
import json
import logging
import os
from datetime import datetime, timedelta
from typing import Any

logger = logging.getLogger(__name__)

# Cache configuration
CACHE_TTL_MINUTES = int(os.environ.get("BIOMCP_FDA_CACHE_TTL", "15"))
MAX_CACHE_SIZE = int(os.environ.get("BIOMCP_FDA_MAX_CACHE_SIZE", "100"))
MAX_RESPONSE_SIZE = int(
    os.environ.get("BIOMCP_FDA_MAX_RESPONSE_SIZE", str(1024 * 1024))
)  # 1MB default

# Global cache dictionary
_cache: dict[str, tuple[Any, datetime]] = {}


def _generate_cache_key(endpoint: str, params: dict[str, Any]) -> str:
    """
    Generate a unique cache key for an API request.

    Args:
        endpoint: The API endpoint URL
        params: Query parameters

    Returns:
        A unique hash key for the request
    """
    # Remove sensitive parameters before hashing
    safe_params = {
        k: v
        for k, v in params.items()
        if k.lower() not in ["api_key", "apikey", "key", "token", "secret"]
    }

    # Sort params for consistent hashing
    sorted_params = json.dumps(safe_params, sort_keys=True)
    combined = f"{endpoint}:{sorted_params}"

    # Use SHA256 for cache key
    return hashlib.sha256(combined.encode()).hexdigest()


def get_cached_response(
    endpoint: str, params: dict[str, Any]
) -> dict[str, Any] | None:
    """
    Retrieve a cached response if available and not expired.

    Args:
        endpoint: The API endpoint URL
        params: Query parameters

    Returns:
        Cached response data or None if not found/expired
    """
    cache_key = _generate_cache_key(endpoint, params)

    if cache_key in _cache:
        data, timestamp = _cache[cache_key]

        # Check if cache entry is still valid
        age = datetime.now() - timestamp
        if age < timedelta(minutes=CACHE_TTL_MINUTES):
            logger.debug(
                f"Cache hit for {endpoint} (age: {age.total_seconds():.1f}s)"
            )
            return data
        else:
            # Remove expired entry
            del _cache[cache_key]
            logger.debug(f"Cache expired for {endpoint}")

    return None


def set_cached_response(
    endpoint: str, params: dict[str, Any], response: dict[str, Any]
) -> None:
    """
    Store a response in the cache.

    Args:
        endpoint: The API endpoint URL
        params: Query parameters
        response: Response data to cache
    """
    # Check response size limit
    import json
    import sys

    # Better size estimation using JSON serialization
    try:
        response_json = json.dumps(response)
        response_size = len(response_json.encode("utf-8"))
    except (TypeError, ValueError):
        # If can't serialize, use sys.getsizeof
        response_size = sys.getsizeof(response)

    if response_size > MAX_RESPONSE_SIZE:
        logger.warning(
            f"Response too large to cache: {response_size} bytes > {MAX_RESPONSE_SIZE} bytes"
        )
        return

    # Check cache size limit
    if len(_cache) >= MAX_CACHE_SIZE:
        # Remove oldest entries (simple FIFO)
        oldest_keys = sorted(_cache.keys(), key=lambda k: _cache[k][1])[
            : len(_cache) - MAX_CACHE_SIZE + 1
        ]

        for key in oldest_keys:
            del _cache[key]

        logger.debug(
            f"Cache size limit reached, removed {len(oldest_keys)} entries"
        )

    cache_key = _generate_cache_key(endpoint, params)
    _cache[cache_key] = (response, datetime.now())

    logger.debug(f"Cached response for {endpoint} (cache size: {len(_cache)})")


def clear_cache() -> None:
    """Clear all cached responses."""
    global _cache
    size = len(_cache)
    _cache = {}
    logger.info(f"Cleared FDA cache ({size} entries)")


def get_cache_stats() -> dict[str, Any]:
    """
    Get cache statistics.

    Returns:
        Dictionary with cache statistics
    """
    now = datetime.now()
    valid_count = 0
    total_age = 0.0

    for _data, timestamp in _cache.values():
        age = (now - timestamp).total_seconds()
        if age < CACHE_TTL_MINUTES * 60:
            valid_count += 1
            total_age += age

    avg_age = total_age / valid_count if valid_count > 0 else 0

    return {
        "total_entries": len(_cache),
        "valid_entries": valid_count,
        "expired_entries": len(_cache) - valid_count,
        "average_age_seconds": avg_age,
        "ttl_minutes": CACHE_TTL_MINUTES,
        "max_size": MAX_CACHE_SIZE,
    }


def is_cacheable_request(endpoint: str, params: dict[str, Any]) -> bool:
    """
    Determine if a request should be cached.

    Args:
        endpoint: The API endpoint URL
        params: Query parameters

    Returns:
        True if the request should be cached
    """
    # Don't cache if caching is disabled
    if CACHE_TTL_MINUTES <= 0:
        return False

    # Don't cache very large requests
    return params.get("limit", 0) <= 100
