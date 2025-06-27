"""Tests for request caching utilities."""

import asyncio

import pytest

from biomcp.utils.request_cache import (
    clear_cache,
    get_cached,
    request_cache,
    set_cached,
)


class TestRequestCache:
    """Test request caching functionality."""

    @pytest.fixture(autouse=True)
    async def clear_cache_before_test(self):
        """Clear cache before each test."""
        await clear_cache()
        yield
        await clear_cache()

    @pytest.mark.asyncio
    async def test_basic_caching(self):
        """Test basic cache get/set operations."""
        # Initially should be empty
        result = await get_cached("test_key")
        assert result is None

        # Set a value
        await set_cached("test_key", "test_value", ttl=10)

        # Should retrieve the value
        result = await get_cached("test_key")
        assert result == "test_value"

    @pytest.mark.asyncio
    async def test_cache_expiry(self):
        """Test that cached values expire."""
        # Set with very short TTL
        await set_cached("test_key", "test_value", ttl=0.1)

        # Should be available immediately
        result = await get_cached("test_key")
        assert result == "test_value"

        # Wait for expiry
        await asyncio.sleep(0.2)

        # Should be expired
        result = await get_cached("test_key")
        assert result is None

    @pytest.mark.asyncio
    async def test_request_cache_decorator(self):
        """Test the @request_cache decorator."""
        call_count = 0

        @request_cache(ttl=10)
        async def expensive_function(arg1, arg2):
            nonlocal call_count
            call_count += 1
            return f"{arg1}-{arg2}-{call_count}"

        # First call should execute function
        result1 = await expensive_function("a", "b")
        assert result1 == "a-b-1"
        assert call_count == 1

        # Second call with same args should use cache
        result2 = await expensive_function("a", "b")
        assert result2 == "a-b-1"  # Same result
        assert call_count == 1  # Function not called again

        # Different args should execute function
        result3 = await expensive_function("c", "d")
        assert result3 == "c-d-2"
        assert call_count == 2

    @pytest.mark.asyncio
    async def test_skip_cache_option(self):
        """Test that skip_cache bypasses caching."""
        call_count = 0

        @request_cache(ttl=10)
        async def cached_function():
            nonlocal call_count
            call_count += 1
            return call_count

        # Normal call - cached
        result1 = await cached_function()
        assert result1 == 1

        # Skip cache - new execution
        result2 = await cached_function(skip_cache=True)
        assert result2 == 2

        # Normal call again - still cached
        result3 = await cached_function()
        assert result3 == 1

    @pytest.mark.asyncio
    async def test_none_values_not_cached(self):
        """Test that None return values are not cached."""
        call_count = 0

        @request_cache(ttl=10)
        async def sometimes_none_function(return_none=False):
            nonlocal call_count
            call_count += 1
            return None if return_none else call_count

        # Return None - should not cache
        result1 = await sometimes_none_function(return_none=True)
        assert result1 is None
        assert call_count == 1

        # Call again - should execute again (not cached)
        result2 = await sometimes_none_function(return_none=True)
        assert result2 is None
        assert call_count == 2

        # Return value - should cache
        result3 = await sometimes_none_function(return_none=False)
        assert result3 == 3
        assert call_count == 3

        # Call again - should use cache
        result4 = await sometimes_none_function(return_none=False)
        assert result4 == 3
        assert call_count == 3
