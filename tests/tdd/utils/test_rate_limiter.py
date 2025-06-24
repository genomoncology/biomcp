"""Tests for rate limiting utilities."""

import asyncio
import time

import pytest

from biomcp.utils.rate_limiter import RateLimiter


class TestRateLimiter:
    """Test rate limiting functionality."""

    @pytest.mark.asyncio
    async def test_basic_rate_limiting(self):
        """Test basic rate limiting behavior."""
        # Create limiter with 2 requests per second
        limiter = RateLimiter(rate=2, per_seconds=1)

        # First two requests should be allowed
        allowed1, wait1 = await limiter.check_rate_limit()
        assert allowed1 is True
        assert wait1 is None

        allowed2, wait2 = await limiter.check_rate_limit()
        assert allowed2 is True
        assert wait2 is None

        # Third request should be denied with wait time
        allowed3, wait3 = await limiter.check_rate_limit()
        assert allowed3 is False
        assert wait3 is not None
        assert wait3 > 0

    @pytest.mark.asyncio
    async def test_rate_limit_replenishment(self):
        """Test that tokens replenish over time."""
        # Create limiter with 1 request per second
        limiter = RateLimiter(rate=1, per_seconds=1)

        # Use the token
        allowed1, _ = await limiter.check_rate_limit()
        assert allowed1 is True

        # Should be denied immediately
        allowed2, wait2 = await limiter.check_rate_limit()
        assert allowed2 is False

        # Wait for replenishment
        await asyncio.sleep(1.1)

        # Should be allowed now
        allowed3, _ = await limiter.check_rate_limit()
        assert allowed3 is True

    @pytest.mark.asyncio
    async def test_multiple_keys(self):
        """Test rate limiting with different keys."""
        limiter = RateLimiter(rate=1, per_seconds=1)

        # Use token for key1
        allowed1, _ = await limiter.check_rate_limit("key1")
        assert allowed1 is True

        # key2 should still have tokens
        allowed2, _ = await limiter.check_rate_limit("key2")
        assert allowed2 is True

        # key1 should be limited
        allowed3, wait3 = await limiter.check_rate_limit("key1")
        assert allowed3 is False
        assert wait3 is not None

    @pytest.mark.asyncio
    async def test_wait_if_needed(self):
        """Test the wait_if_needed helper."""
        limiter = RateLimiter(rate=1, per_seconds=1)

        # First call should not wait
        start = time.time()
        await limiter.wait_if_needed()
        elapsed = time.time() - start
        assert elapsed < 0.1

        # Second call should wait
        start = time.time()
        await limiter.wait_if_needed()
        elapsed = time.time() - start
        assert elapsed >= 0.9  # Should wait approximately 1 second
