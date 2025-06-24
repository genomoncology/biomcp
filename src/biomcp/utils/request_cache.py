"""Simple request-level caching for API calls."""

import asyncio
import time
from collections.abc import Awaitable, Callable
from functools import wraps
from typing import Any, TypeVar

# Simple in-memory cache with TTL
_cache: dict[str, tuple[Any, float]] = {}
_cache_lock = asyncio.Lock()

# Default TTL in seconds (15 minutes)
DEFAULT_TTL = 900

T = TypeVar("T")


def cache_key(*args, **kwargs) -> str:
    """Generate a cache key from function arguments."""
    key_parts = [str(arg) for arg in args]
    key_parts.extend(f"{k}={v}" for k, v in sorted(kwargs.items()))
    return ":".join(key_parts)


async def get_cached(key: str) -> Any | None:
    """Get a value from cache if not expired."""
    async with _cache_lock:
        if key in _cache:
            value, expiry = _cache[key]
            if time.time() < expiry:
                return value
            else:
                # Remove expired entry
                del _cache[key]
    return None


async def set_cached(key: str, value: Any, ttl: int = DEFAULT_TTL) -> None:
    """Set a value in cache with TTL."""
    async with _cache_lock:
        _cache[key] = (value, time.time() + ttl)


def request_cache(ttl: int = DEFAULT_TTL) -> Callable:
    """Decorator for caching async function results.

    Args:
        ttl: Time to live in seconds

    Returns:
        Decorated function with caching
    """

    def decorator(
        func: Callable[..., Awaitable[T]],
    ) -> Callable[..., Awaitable[T]]:
        @wraps(func)
        async def wrapper(*args, **kwargs) -> T:
            # Skip caching if explicitly disabled
            if kwargs.pop("skip_cache", False):
                return await func(*args, **kwargs)

            # Generate cache key
            key = f"{func.__module__}.{func.__name__}:{cache_key(*args, **kwargs)}"

            # Check cache
            cached_value = await get_cached(key)
            if cached_value is not None:
                return cached_value

            # Call function and cache result
            result = await func(*args, **kwargs)
            if result is not None:  # Only cache non-None results
                await set_cached(key, result, ttl)

            return result

        return wrapper

    return decorator


async def clear_cache() -> None:
    """Clear all cached entries."""
    async with _cache_lock:
        _cache.clear()
