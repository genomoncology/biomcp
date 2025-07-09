"""Simple request-level caching for API calls."""

import asyncio
import time
from collections import OrderedDict
from collections.abc import Awaitable, Callable
from functools import wraps
from typing import Any, TypeVar


# LRU cache with size limit
class LRUCache:
    """Simple LRU cache with TTL support."""

    def __init__(self, max_size: int = 1000):
        self.cache: OrderedDict[str, tuple[Any, float]] = OrderedDict()
        self.max_size = max_size
        self._lock = asyncio.Lock()

    async def get(self, key: str) -> Any | None:
        """Get item from cache if not expired."""
        async with self._lock:
            if key not in self.cache:
                return None

            value, expiry = self.cache[key]
            if time.time() > expiry:
                del self.cache[key]
                return None

            # Move to end (most recently used)
            self.cache.move_to_end(key)
            return value

    async def set(self, key: str, value: Any, ttl: float):
        """Set item in cache with TTL."""
        async with self._lock:
            # Remove oldest items if at capacity
            while len(self.cache) >= self.max_size:
                self.cache.popitem(last=False)

            expiry = time.time() + ttl
            self.cache[key] = (value, expiry)


# Global LRU cache instance
_cache = LRUCache(max_size=1000)

# Default TTL in seconds (15 minutes)
DEFAULT_TTL = 900

# Named caches for different purposes
_named_caches: dict[str, LRUCache] = {}


def get_cache(
    name: str, ttl_seconds: int = 300, max_size: int = 100
) -> LRUCache:
    """Get or create a named cache."""
    if name not in _named_caches:
        _named_caches[name] = LRUCache(max_size=max_size)
    return _named_caches[name]


T = TypeVar("T")


def cache_key(*args, **kwargs) -> str:
    """Generate a cache key from function arguments."""
    key_parts = [str(arg) for arg in args]
    key_parts.extend(f"{k}={v}" for k, v in sorted(kwargs.items()))
    return ":".join(key_parts)


async def get_cached(key: str) -> Any | None:
    """Get a value from cache if not expired."""
    return await _cache.get(key)


async def set_cached(key: str, value: Any, ttl: int = DEFAULT_TTL) -> None:
    """Set a value in cache with TTL."""
    await _cache.set(key, value, ttl)


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
    # Use the LRU cache's clear method
    _cache.cache.clear()
